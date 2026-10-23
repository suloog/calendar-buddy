//! The first full synchronization of an account (`US-02-01`): every calendar it can access, and
//! their events within the cache window, written to the cache. The provider (Google) sits behind
//! `Provider`, so nothing here knows about it (`NFR-14`).

use chrono::{DateTime, Days, Months, NaiveDate, Utc};
use chrono_tz::Tz;

use crate::cache::{Cache, CachedCalendar};
use crate::model::{Calendar, Event, Timing};
use crate::recurrence::expand;
use crate::week::day_start;

/// Where an event or an occurrence starts: an instant, or a date for all-day ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Start {
    At(DateTime<Utc>),
    On(NaiveDate),
}

/// An event as synchronized: a single event, the master of a series, or an occurrence of a series
/// that was changed or cancelled. Series are kept as master plus exceptions, not expanded.
#[derive(Clone, Debug, PartialEq)]
pub struct SyncedEvent {
    pub id: String,
    pub title: String,
    /// When it happens; `None` for a cancelled occurrence.
    pub timing: Option<Timing>,
    /// A master's `RRULE`, `EXDATE` and `RDATE` lines, as in iCalendar; empty otherwise.
    pub recurrence: Vec<String>,
    /// For a changed or cancelled occurrence: its series' master and its original start.
    pub instance_of: Option<(String, Start)>,
}

/// The days the cache holds: one year back to two years ahead of today (`MOD-02-F11`, `DEC-23`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub from: NaiveDate,
    /// The first day after the window.
    pub until: NaiveDate,
    pub zone: Tz,
}

impl Window {
    pub fn around(today: NaiveDate, zone: Tz) -> Window {
        Window {
            from: today - Months::new(12),
            until: today + Months::new(24),
            zone,
        }
    }

    pub fn start(&self) -> DateTime<Utc> {
        day_start(self.from, self.zone)
    }

    pub fn end(&self) -> DateTime<Utc> {
        day_start(self.until, self.zone)
    }

    /// Whether any part of the event falls inside; one crossing an edge is kept whole.
    pub fn overlaps(&self, timing: &Timing) -> bool {
        match *timing {
            Timing::Timed { start, end, .. } => start < self.end() && end > self.start(),
            Timing::AllDay {
                start,
                end_exclusive,
            } => start < self.until && end_exclusive > self.from,
        }
    }

    /// The days from `first` to `last` that the window does not hold, as one span: those before
    /// it, or else those after it; `None` when it holds them all (`MOD-02-F12`).
    pub fn outside(&self, first: NaiveDate, last: NaiveDate) -> Option<Window> {
        let until = last + Days::new(1);
        let span = |from, until| Window {
            from,
            until,
            zone: self.zone,
        };
        if first < self.from {
            Some(span(first, until.min(self.from)))
        } else if until > self.until {
            Some(span(first.max(self.until), until))
        } else {
            None
        }
    }

    /// Whether the cache keeps a changed event: one inside the window, a series (its occurrences
    /// may fall inside), or a changed or cancelled occurrence (it stands in for one of them).
    pub fn keeps(&self, event: &SyncedEvent) -> bool {
        !event.recurrence.is_empty()
            || event.instance_of.is_some()
            || event.timing.is_some_and(|timing| self.overlaps(&timing))
    }
}

/// Why synchronizing an account or a calendar did not finish; never holds a token or event
/// contents (`NFR-07`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncError(pub String);

/// What a provider sends for a calendar: its events, all or only those changed, and the state to
/// ask for changes since.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fetched {
    pub events: Vec<SyncedEvent>,
    /// The IDs of events deleted since the state asked with.
    pub removed: Vec<String>,
    /// `None` when the provider gave no state; the next synchronization is then full again.
    pub sync_token: Option<String>,
}

/// One account at a calendar provider.
pub trait Provider {
    /// Every calendar the account can access, read-only and subscribed ones included.
    fn calendars(&self) -> Result<Vec<Calendar>, SyncError>;
    /// The calendar's events that fall inside the window, series as master plus exceptions.
    fn events(&self, calendar: &Calendar, window: Window) -> Result<Fetched, SyncError>;
    /// What changed in the calendar since the state, wherever it falls; `None` when the provider
    /// no longer accepts the state.
    fn changes(&self, calendar: &Calendar, sync_token: &str) -> Result<Option<Fetched>, SyncError>;
}

/// Fetches every calendar of the account and synchronizes each (`sync_calendars`); calendars no
/// longer listed go from the cache with their events.
pub fn sync_account(
    provider: &dyn Provider,
    account_id: &str,
    window: Window,
    cache: &mut Cache,
) -> Result<(), SyncError> {
    let list = sync_calendar_list(provider, account_id, cache)?;
    sync_calendars(provider, &list.calendars, window, cache).map(|_| ())
}

/// An account's calendar list as just fetched, and how it differs from the one cached before.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarList {
    pub calendars: Vec<Calendar>,
    /// The calendars no longer listed: deleted, unshared or otherwise out of reach (`MOD-02-F21`).
    pub gone: Vec<Calendar>,
    /// Whether anything differs: a calendar added or gone, renamed, recoloured or with another
    /// access.
    pub changed: bool,
}

/// Brings the account's cached calendar list up to date (`MOD-02-F05`): calendars added in the
/// provider join it, and those no longer listed go with their events (`MOD-02-F21`).
pub fn sync_calendar_list(
    provider: &dyn Provider,
    account_id: &str,
    cache: &mut Cache,
) -> Result<CalendarList, SyncError> {
    let calendars = provider.calendars()?;
    // An unreadable cached list is rewritten; nothing can be told gone from it.
    let known: Vec<Calendar> = cache
        .calendar_list()
        .unwrap_or_default()
        .into_iter()
        .filter(|calendar| calendar.account_id == account_id)
        .collect();
    cache.set_calendars(account_id, &calendars)?;
    let gone = known
        .iter()
        .filter(|old| !calendars.iter().any(|new| new.id == old.id))
        .cloned()
        .collect();
    Ok(CalendarList {
        changed: known != calendars,
        calendars,
        gone,
    })
}

/// Synchronizes each calendar (`sync_calendar`) and says whether anything changed. A calendar
/// that fails keeps what the cache had and does not stop the others; the first such failure is
/// returned once they are done.
pub fn sync_calendars(
    provider: &dyn Provider,
    calendars: &[Calendar],
    window: Window,
    cache: &mut Cache,
) -> Result<bool, SyncError> {
    let mut changed = false;
    let mut failed = None;
    for calendar in calendars {
        match sync_calendar(provider, calendar, window, cache) {
            Ok(change) => changed |= change,
            Err(error) => {
                failed.get_or_insert(error);
            }
        }
    }
    failed.map_or(Ok(changed), Err)
}

/// Synchronizes one calendar's events and says whether anything changed (`MOD-02-F10`): after its
/// first full synchronization only what changed is fetched, and kept when it falls inside the
/// window; when the provider no longer accepts the stored state, or the window has moved since
/// (`MOD-02-F11`), the calendar is fetched in full again.
pub fn sync_calendar(
    provider: &dyn Provider,
    calendar: &Calendar,
    window: Window,
    cache: &mut Cache,
) -> Result<bool, SyncError> {
    if let Some(token) = cache.sync_token(calendar, window)?
        && let Some(changes) = provider.changes(calendar, &token)?
    {
        let changed = !changes.events.is_empty() || !changes.removed.is_empty();
        let (kept, moved_out): (Vec<_>, Vec<_>) = changes
            .events
            .into_iter()
            .partition(|event| window.keeps(event));
        let mut removed = changes.removed;
        removed.extend(moved_out.into_iter().map(|event| event.id));
        cache.change_events(calendar, &kept, &removed)?;
        cache.set_sync_token(calendar, changes.sync_token.as_deref(), window)?;
        return Ok(changed);
    }
    let fetched = provider.events(calendar, window)?;
    cache.set_events(calendar, &fetched.events)?;
    cache.set_sync_token(calendar, fetched.sync_token.as_deref(), window)?;
    Ok(true)
}

/// The events and occurrences of the calendars in a span of days the cache does not hold, as the
/// views show them; they are fetched from the provider and not kept (`MOD-02-F12`). A calendar
/// that fails shows nothing.
pub fn fetch_events(provider: &dyn Provider, calendars: &[Calendar], span: Window) -> Vec<Event> {
    // A series' changed occurrences come where they are now: one moved out of the span by up to
    // a month is asked for too, so it does not show at its old time.
    let asked = Window {
        from: span.from - Months::new(1),
        until: span.until + Months::new(1),
        zone: span.zone,
    };
    calendars
        .iter()
        .filter_map(|calendar| {
            let fetched = provider.events(calendar, asked).ok()?;
            Some(expand(calendar, &fetched.events, span))
        })
        .flatten()
        .collect()
}

/// What the views show: the events and occurrences of every cached calendar, of all accounts
/// together, that fall inside the window (`MOD-02-F02`).
pub fn shown_events(cached: &[CachedCalendar], window: Window) -> Vec<Event> {
    cached
        .iter()
        .flat_map(|cached| expand(&cached.calendar, &cached.events, window))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;
    use crate::model::{Access, Rgb};
    use chrono::TimeZone;
    use chrono_tz::Europe::Warsaw;
    use std::cell::RefCell;

    /// A provider with one calendar, keeping its changes since each state it gave out.
    #[derive(Default)]
    struct Fake {
        events: RefCell<Vec<SyncedEvent>>,
        /// Every change so far: a changed event, or the ID of a deleted one.
        changes: RefCell<Vec<Result<SyncedEvent, String>>>,
        /// The provider no longer accepts the states it gave out.
        expired: RefCell<bool>,
        /// What was asked: "events", or "changes since <state>".
        asked: RefCell<Vec<String>>,
        /// Calendars listed after "work".
        others: RefCell<Vec<Calendar>>,
    }

    impl Fake {
        fn put(&self, event: SyncedEvent) {
            let mut events = self.events.borrow_mut();
            events.retain(|e| e.id != event.id);
            events.push(event.clone());
            self.changes.borrow_mut().push(Ok(event));
        }

        fn delete(&self, id: &str) {
            self.events.borrow_mut().retain(|e| e.id != id);
            self.changes.borrow_mut().push(Err(id.to_string()));
        }

        fn state(&self) -> Option<String> {
            Some(self.changes.borrow().len().to_string())
        }
    }

    impl Provider for Fake {
        fn calendars(&self) -> Result<Vec<Calendar>, SyncError> {
            let mut calendars = vec![calendar()];
            calendars.extend(self.others.borrow().iter().cloned());
            Ok(calendars)
        }

        fn events(&self, _: &Calendar, window: Window) -> Result<Fetched, SyncError> {
            self.asked.borrow_mut().push("events".into());
            Ok(Fetched {
                events: self
                    .events
                    .borrow()
                    .iter()
                    .filter(|e| window.keeps(e))
                    .cloned()
                    .collect(),
                removed: Vec::new(),
                sync_token: self.state(),
            })
        }

        fn changes(&self, _: &Calendar, since: &str) -> Result<Option<Fetched>, SyncError> {
            self.asked
                .borrow_mut()
                .push(format!("changes since {since}"));
            if *self.expired.borrow() {
                return Ok(None);
            }
            let mut fetched = Fetched {
                sync_token: self.state(),
                ..Fetched::default()
            };
            // Like Google, each event once, as it is now.
            for change in &self.changes.borrow()[since.parse().unwrap()..] {
                let id = match change {
                    Ok(event) => &event.id,
                    Err(id) => id,
                };
                fetched.events.retain(|e| e.id != *id);
                fetched.removed.retain(|r| r != id);
                match change.clone() {
                    Ok(event) => fetched.events.push(event),
                    Err(id) => fetched.removed.push(id),
                }
            }
            Ok(Some(fetched))
        }
    }

    fn calendar() -> Calendar {
        Calendar {
            id: "work".into(),
            account_id: "1".into(),
            name: "Work".into(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
        }
    }

    fn event(id: &str, title: &str, day: NaiveDate) -> SyncedEvent {
        SyncedEvent {
            id: id.into(),
            title: title.into(),
            timing: Some(Timing::AllDay {
                start: day,
                end_exclusive: day.succ_opt().unwrap(),
            }),
            recurrence: Vec::new(),
            instance_of: None,
        }
    }

    /// The cached events' titles, sorted.
    fn titles(cache: &Cache) -> Vec<String> {
        let mut titles: Vec<String> = cache.calendars().unwrap()[0]
            .events
            .iter()
            .map(|e| e.title.clone())
            .collect();
        titles.sort();
        titles
    }

    fn window() -> Window {
        Window::around(d(2026, 11, 11), Warsaw)
    }

    /// An account synchronized for the first time: "Standup" and "Review" in its calendar.
    fn first_sync(fake: &Fake) -> Cache {
        fake.put(event("standup", "Standup", d(2026, 11, 11)));
        fake.put(event("review", "Review", d(2026, 11, 12)));
        let mut cache = Cache::in_memory();
        sync_account(fake, "1", window(), &mut cache).unwrap();
        cache
    }

    #[test]
    fn after_the_first_full_synchronization_only_changes_are_fetched() {
        let fake = Fake::default();
        let mut cache = first_sync(&fake);
        fake.put(event("standup", "Standup 2", d(2026, 11, 11)));
        fake.delete("review");
        fake.put(event("retro", "Retro", d(2026, 11, 13)));
        // Moved past the window: it goes from the cache.
        fake.put(event("standup", "Standup 3", d(2029, 1, 1)));
        fake.put(event("planning", "Planning", d(2026, 11, 13)));

        let changed = sync_calendar(&fake, &calendar(), window(), &mut cache);

        assert_eq!(changed, Ok(true));
        assert_eq!(*fake.asked.borrow(), ["events", "changes since 2"]);
        assert_eq!(titles(&cache), ["Planning", "Retro"]);
        assert_eq!(
            cache.sync_token(&calendar(), window()).unwrap().as_deref(),
            Some("7")
        );
    }

    #[test]
    fn once_the_window_has_moved_the_calendar_is_fetched_in_full_again() {
        let fake = Fake::default();
        // Just past the window, unchanged since.
        fake.put(event("conference", "Conference", d(2028, 11, 11)));
        let mut cache = first_sync(&fake);
        assert_eq!(titles(&cache), ["Review", "Standup"]);
        let next_day = Window::around(d(2026, 11, 12), Warsaw);

        let changed = sync_calendar(&fake, &calendar(), next_day, &mut cache);

        assert_eq!(changed, Ok(true));
        assert_eq!(*fake.asked.borrow(), ["events", "events"]);
        assert_eq!(titles(&cache), ["Conference", "Review", "Standup"]);
    }

    #[test]
    fn the_calendar_list_follows_the_provider_and_says_what_is_gone() {
        let fake = Fake::default();
        let mut cache = first_sync(&fake);
        let home = Calendar {
            id: "home".into(),
            name: "Home".into(),
            ..calendar()
        };

        fake.others.borrow_mut().push(home.clone());
        let added = sync_calendar_list(&fake, "1", &mut cache).unwrap();
        sync_calendar(&fake, &home, window(), &mut cache).unwrap();

        assert_eq!(added.calendars, [calendar(), home.clone()]);
        assert!(added.changed);
        assert!(added.gone.is_empty());
        assert_eq!(cache.calendars().unwrap()[1].events.len(), 2);

        fake.others.borrow_mut().clear();
        let removed = sync_calendar_list(&fake, "1", &mut cache).unwrap();

        assert_eq!(removed.gone, std::slice::from_ref(&home));
        assert!(removed.changed);
        assert_eq!(cache.calendar_list().unwrap(), [calendar()]);
        assert_eq!(cache.sync_token(&home, window()).unwrap(), None);

        // Gone once: the next list has nothing more to say.
        let again = sync_calendar_list(&fake, "1", &mut cache).unwrap();
        assert_eq!(again.gone, []);
        assert!(!again.changed);
    }

    #[test]
    fn a_calendar_renamed_or_with_other_access_changes_the_list() {
        let fake = Fake::default();
        let mut cache = first_sync(&fake);
        let shared = Calendar {
            id: "shared".into(),
            access: Access::Writer,
            ..calendar()
        };
        fake.others.borrow_mut().push(shared.clone());
        sync_calendar_list(&fake, "1", &mut cache).unwrap();

        fake.others.borrow_mut()[0].access = Access::Reader;
        let list = sync_calendar_list(&fake, "1", &mut cache).unwrap();

        assert!(list.changed);
        assert!(list.gone.is_empty());
        assert_eq!(cache.calendar_list().unwrap()[1].access, Access::Reader);
    }

    #[test]
    fn with_nothing_changed_nothing_is_reported() {
        let fake = Fake::default();
        let mut cache = first_sync(&fake);

        let changed = sync_calendar(&fake, &calendar(), window(), &mut cache);

        assert_eq!(changed, Ok(false));
        assert_eq!(titles(&cache), ["Review", "Standup"]);
    }

    #[test]
    fn a_state_no_longer_accepted_is_followed_by_a_full_synchronization() {
        let fake = Fake::default();
        let mut cache = first_sync(&fake);
        fake.put(event("standup", "Standup 2", d(2026, 11, 11)));
        fake.delete("review");
        *fake.expired.borrow_mut() = true;

        let changed = sync_calendar(&fake, &calendar(), window(), &mut cache);

        assert_eq!(changed, Ok(true));
        assert_eq!(
            *fake.asked.borrow(),
            ["events", "changes since 2", "events"]
        );
        assert_eq!(titles(&cache), ["Standup 2"]);
        assert_eq!(
            cache.sync_token(&calendar(), window()).unwrap().as_deref(),
            Some("4")
        );
    }

    #[test]
    fn the_cache_window_is_one_year_back_to_two_years_ahead() {
        let window = Window::around(d(2026, 11, 11), Warsaw);
        assert_eq!(window.from, d(2025, 11, 11));
        assert_eq!(window.until, d(2028, 11, 11));
        assert_eq!(
            window.start(),
            Warsaw.with_ymd_and_hms(2025, 11, 11, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn the_days_on_screen_the_window_does_not_hold_are_one_span() {
        let window = Window::around(d(2026, 11, 11), Warsaw);
        let span = |from, until| Window {
            from,
            until,
            zone: Warsaw,
        };

        assert_eq!(window.outside(d(2026, 11, 9), d(2026, 11, 15)), None);
        assert_eq!(
            window.outside(d(2028, 11, 6), d(2028, 11, 12)),
            Some(span(d(2028, 11, 11), d(2028, 11, 13)))
        );
        assert_eq!(
            window.outside(d(2025, 11, 10), d(2025, 11, 16)),
            Some(span(d(2025, 11, 10), d(2025, 11, 11)))
        );
        assert_eq!(
            window.outside(d(2030, 3, 1), d(2030, 3, 31)),
            Some(span(d(2030, 3, 1), d(2030, 4, 1)))
        );
    }

    #[test]
    fn a_period_outside_the_window_is_fetched_and_not_kept() {
        let fake = Fake::default();
        let cache = first_sync(&fake);
        fake.put(event("trip", "Trip", d(2030, 3, 10)));
        fake.put(event("later", "Later", d(2030, 4, 10)));
        let march = Window {
            from: d(2030, 3, 1),
            until: d(2030, 4, 1),
            zone: Warsaw,
        };

        let fetched = fetch_events(&fake, &[calendar()], march);

        let shown: Vec<&str> = fetched.iter().map(|e| e.title.as_str()).collect();
        assert_eq!(shown, ["Trip"]);
        assert_eq!(fetched[0].calendar_id, "work");
        assert_eq!(titles(&cache), ["Review", "Standup"]);
    }

    #[test]
    fn an_event_crossing_the_window_edge_counts_as_inside() {
        let window = Window::around(d(2026, 11, 11), Warsaw);
        let crossing = Timing::AllDay {
            start: d(2025, 11, 9),
            end_exclusive: d(2025, 11, 13),
        };
        let before = Timing::AllDay {
            start: d(2025, 11, 9),
            end_exclusive: d(2025, 11, 11),
        };
        let last_day = Timing::Timed {
            start: Utc.with_ymd_and_hms(2028, 11, 10, 22, 30, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2028, 11, 11, 1, 0, 0).unwrap(),
            zone: Warsaw,
        };
        assert!(window.overlaps(&crossing));
        assert!(!window.overlaps(&before));
        assert!(window.overlaps(&last_day));
    }
}
