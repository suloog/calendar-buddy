//! Creating an event in the quick-entry box (`US-05-01`): the times a click, a selected range or
//! the `c` key start it with, the calendar it goes to (`MOD-05-F10`), and the new events shown as
//! "saving" until Google confirms them (`MOD-05-U03`). The event goes to Google first and reaches
//! the views through synchronization; the "saving" mark is view state only.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;

use crate::accounts::Entry;
use crate::details::When;
use crate::model::{About, Calendar, Event, Timing};
use crate::settings::Settings;
use crate::side_panel::pickable;

/// The length, in minutes, of a new event started without a range until the account's own is
/// known: Google Calendar's default.
pub const DEFAULT_LENGTH: u32 = 60;

/// The step, in minutes, a click or a selection in the hours snaps to (`DEC-103`).
pub const STEP: u32 = 15;

/// What the views add to the label of an event Google has not confirmed yet (`MOD-05-U03`).
pub const SAVING: &str = "Saving…";

/// What the quick-entry box says when the user has no calendar to add events to.
pub const NO_CALENDAR: &str = "No calendar you can add events to.";

/// What Google Calendar names an event without a title.
pub const NO_TITLE: &str = "(No title)";

/// A new event as the user made it, to be created at the provider.
#[derive(Clone, Debug, PartialEq)]
pub struct NewEvent {
    pub title: String,
    pub timing: Timing,
    pub about: About,
}

/// The instant `minute` minutes after midnight of `date` on the wall clock in `zone`; a time the
/// zone skips (daylight saving) moves to the first one that exists after it.
pub(crate) fn wall(date: NaiveDate, minute: u32, zone: Tz) -> DateTime<Utc> {
    let mut local = date.and_time(NaiveTime::MIN) + Duration::minutes(minute.into());
    loop {
        if let Some(at) = zone.from_local_datetime(&local).earliest() {
            return at.with_timezone(&Utc);
        }
        local += Duration::minutes(STEP.into());
    }
}

fn floor(minute: u32) -> u32 {
    minute / STEP * STEP
}

fn ceil(minute: u32) -> u32 {
    minute.div_ceil(STEP) * STEP
}

/// The minutes of the day a range selected in the hours covers, from where the press started to
/// where it ended, in either direction, in whole steps and at least one.
pub fn snapped(pressed: u32, released: u32) -> (u32, u32) {
    let from = floor(pressed.min(released)).min(24 * 60 - STEP);
    let until = ceil(pressed.max(released)).clamp(from + STEP, 24 * 60);
    (from, until)
}

/// The times of a range selected in the hours of `date` (`MOD-05-F01`), as `snapped`.
pub fn selected(
    date: NaiveDate,
    pressed: u32,
    released: u32,
    zone: Tz,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let (from, until) = snapped(pressed, released);
    let start = wall(date, from, zone);
    // A range inside an hour the clock skips still lasts a step.
    let end = wall(date, until, zone).max(start + Duration::minutes(STEP.into()));
    (start, end)
}

/// The times of an event started by a click in the hours of `date`: from the step clicked, the
/// default length long (`MOD-05-F09`).
pub fn clicked(
    date: NaiveDate,
    minute: u32,
    length: u32,
    zone: Tz,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let start = wall(date, floor(minute).min(24 * 60 - STEP), zone);
    (start, start + Duration::minutes(length.into()))
}

/// The times of an event started with the `c` key: on the selected day, from the next full hour
/// of the clock, the default length long (`MOD-05-F09`, `DEC-103`).
pub fn keyed(
    date: NaiveDate,
    now: DateTime<Utc>,
    length: u32,
    zone: Tz,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let hour = (now.with_timezone(&zone).hour() + 1).min(23);
    let start = wall(date, hour * 60, zone);
    (start, start + Duration::minutes(length.into()))
}

/// The calendars a new event can go to, under their accounts (`MOD-05-F10`): those the account
/// can change, disabled ones left out; accounts with none are left out.
pub fn choices(
    accounts: &[Entry],
    calendars: &[Calendar],
    settings: &Settings,
) -> Vec<(String, Vec<Calendar>)> {
    pickable(accounts, calendars, settings)
        .into_iter()
        .map(|(email, calendars)| {
            let editable = calendars.into_iter().filter(|c| c.access.can_edit());
            (email, editable.collect::<Vec<_>>())
        })
        .filter(|(_, calendars)| !calendars.is_empty())
        .collect()
}

/// The calendar a new event goes to (`MOD-05-F10`, `DEC-104`): the one chosen in the settings;
/// until the user chooses, or when it can no longer take events, the primary calendar of the
/// first connected account, and failing that the first calendar that can take events.
pub fn default_calendar(
    accounts: &[Entry],
    calendars: &[Calendar],
    settings: &Settings,
) -> Option<Calendar> {
    let choices: Vec<Calendar> = choices(accounts, calendars, settings)
        .into_iter()
        .flat_map(|(_, calendars)| calendars)
        .collect();
    let chosen = settings
        .default_calendar
        .as_ref()
        .and_then(|(account, id)| {
            choices
                .iter()
                .find(|c| c.account_id == *account && c.id == *id)
        });
    // Google names an account's primary calendar by the account's e-mail address.
    let primary = accounts.first().and_then(|first| {
        choices
            .iter()
            .find(|c| c.account_id == first.account.id && c.id == first.account.email)
    });
    chosen.or(primary).or(choices.first()).cloned()
}

/// What the quick-entry box holds: the title typed so far, the times it was started with and the
/// calendar picked (`MOD-05-F02`).
#[derive(Clone, Debug, PartialEq)]
pub struct QuickEntry {
    pub title: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub zone: Tz,
    /// `None` when no calendar can take events.
    pub calendar: Option<Calendar>,
}

impl QuickEntry {
    pub fn new(
        (start, end): (DateTime<Utc>, DateTime<Utc>),
        zone: Tz,
        calendar: Option<Calendar>,
    ) -> Self {
        QuickEntry {
            title: String::new(),
            start,
            end,
            zone,
            calendar,
        }
    }

    /// When the event happens, in the system's time zone, for the box to show.
    pub fn when(&self) -> When {
        When::Timed {
            start: self.start.with_timezone(&self.zone),
            end: self.end.with_timezone(&self.zone),
        }
    }

    /// The event to create and its calendar; `None` without a calendar.
    pub fn new_event(&self) -> Option<(Calendar, NewEvent)> {
        let event = NewEvent {
            title: self.title.trim().to_string(),
            timing: Timing::Timed {
                start: self.start,
                end: self.end,
                zone: self.zone,
            },
            about: About::default(),
        };
        Some((self.calendar.clone()?, event))
    }
}

/// A new or changed event shown before it comes back through synchronization.
#[derive(Clone, Debug)]
struct Pending {
    local: u64,
    event: Event,
    /// For a change, the event as it was: not shown meanwhile.
    replaces: Option<Event>,
}

/// Whether two events are the same event of the same calendar.
fn same(a: &Event, b: &Event) -> bool {
    a.id == b.id && a.calendar_id == b.calendar_id && a.account_id == b.account_id
}

/// The new and changed events sent to Google and not yet synchronized back: each is shown marked
/// "saving" until Google confirms it, then without the mark until synchronization brings it
/// (`MOD-05-U03`); one Google refused goes, and a changed one is shown as it was again.
#[derive(Clone, Debug, Default)]
pub struct Saving {
    pending: Vec<Pending>,
    next: u64,
}

impl Saving {
    /// Shows the new event as saving and returns the number it is told by.
    pub fn add(&mut self, calendar: &Calendar, event: &NewEvent) -> u64 {
        let title = if event.title.is_empty() {
            NO_TITLE
        } else {
            &event.title
        };
        let event = Event {
            id: String::new(),
            account_id: calendar.account_id.clone(),
            calendar_id: calendar.id.clone(),
            title: title.to_string(),
            color: calendar.color,
            timing: event.timing,
            busy_only: false,
            read_only: false,
            saving: true,
            recurring: false,
            about: event.about.clone(),
            etag: String::new(),
        };
        self.push(event, None)
    }

    /// Shows the changed event as saving in place of `before`, and returns the number it is told
    /// by.
    pub fn change(&mut self, before: &Event, after: Event) -> u64 {
        let after = Event {
            saving: true,
            ..after
        };
        self.push(after, Some(before.clone()))
    }

    fn push(&mut self, event: Event, replaces: Option<Event>) -> u64 {
        self.next += 1;
        self.pending.push(Pending {
            local: self.next,
            event,
            replaces,
        });
        self.next
    }

    /// Google created the event under `id`: it is saved.
    pub fn confirmed(&mut self, local: u64, id: &str) {
        if let Some(pending) = self.pending.iter_mut().find(|p| p.local == local) {
            pending.event.id = id.to_string();
            pending.event.saving = false;
        }
    }

    /// Google took the change, and the event's version is now `etag`: it is saved.
    pub fn changed(&mut self, local: u64, etag: &str) {
        if let Some(pending) = self.pending.iter_mut().find(|p| p.local == local) {
            pending.event.etag = etag.to_string();
            pending.event.saving = false;
        }
    }

    /// Google did not create or change the event: a new one goes, a changed one is shown as it
    /// was.
    pub fn refused(&mut self, local: u64) {
        self.pending.retain(|p| p.local != local);
    }

    /// Synchronization brought these events: the saved ones among them, in a version other than
    /// the one changed or in the calendar moved to, are no longer pending.
    pub fn synced(&mut self, events: &[Event]) {
        self.pending.retain(|p| {
            p.event.saving
                || !events.iter().any(|e| {
                    same(e, &p.event)
                        && p.replaces.as_ref().is_none_or(|old| {
                            e.etag != old.etag || e.calendar_id != old.calendar_id
                        })
                })
        });
    }

    /// The events to show: `cached` without those changed meanwhile, followed by the pending ones.
    pub fn shown(&self, cached: &[Event]) -> Vec<Event> {
        let replaced = |event: &Event| {
            self.pending
                .iter()
                .filter_map(|p| p.replaces.as_ref())
                .any(|old| same(old, event))
        };
        cached
            .iter()
            .filter(|event| !replaced(event))
            .chain(self.events())
            .cloned()
            .collect()
    }

    /// Whether the event is one of these, not synchronized back yet.
    pub fn holds(&self, event: &Event) -> bool {
        self.pending.iter().any(|p| same(&p.event, event))
    }

    /// The account was removed: its new events go with it.
    pub fn forget_account(&mut self, account_id: &str) {
        self.pending.retain(|p| p.event.account_id != account_id);
    }

    pub fn events(&self) -> impl Iterator<Item = &Event> {
        self.pending.iter().map(|p| &p.event)
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{Account, AccountState};
    use crate::fixtures::d;
    use crate::model::{Access, Rgb};
    use chrono_tz::Europe::Warsaw;

    fn local(at: DateTime<Utc>) -> String {
        at.with_timezone(&Warsaw).format("%F %R").to_string()
    }

    fn span((start, end): (DateTime<Utc>, DateTime<Utc>)) -> (String, String) {
        (local(start), local(end))
    }

    fn s(start: &str, end: &str) -> (String, String) {
        (start.to_string(), end.to_string())
    }

    #[test]
    fn a_selected_range_is_taken_in_whole_steps_in_either_direction() {
        let day = d(2026, 11, 12);
        assert_eq!(
            span(selected(day, 600, 690, Warsaw)),
            s("2026-11-12 10:00", "2026-11-12 11:30")
        );
        assert_eq!(
            span(selected(day, 688, 603, Warsaw)),
            s("2026-11-12 10:00", "2026-11-12 11:30")
        );
        // A press without a move still gives one step; the end of the day is midnight.
        assert_eq!(
            span(selected(day, 607, 607, Warsaw)),
            s("2026-11-12 10:00", "2026-11-12 10:15")
        );
        assert_eq!(
            span(selected(day, 1400, 1440, Warsaw)),
            s("2026-11-12 23:15", "2026-11-13 00:00")
        );
        assert_eq!(snapped(1440, 1440), (1425, 1440));
    }

    #[test]
    fn a_click_starts_an_event_of_the_default_length_at_the_step_clicked() {
        let day = d(2026, 11, 12);
        assert_eq!(
            span(clicked(day, 614, 30, Warsaw)),
            s("2026-11-12 10:00", "2026-11-12 10:30")
        );
        assert_eq!(
            span(clicked(day, 1439, 60, Warsaw)),
            s("2026-11-12 23:45", "2026-11-13 00:45")
        );
    }

    #[test]
    fn the_c_key_starts_an_event_at_the_next_full_hour_of_the_selected_day() {
        let now = Warsaw
            .with_ymd_and_hms(2026, 11, 11, 10, 20, 0)
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            span(keyed(d(2026, 11, 11), now, 60, Warsaw)),
            s("2026-11-11 11:00", "2026-11-11 12:00")
        );
        assert_eq!(
            span(keyed(d(2026, 11, 13), now, 45, Warsaw)),
            s("2026-11-13 11:00", "2026-11-13 11:45")
        );
        let late = Warsaw
            .with_ymd_and_hms(2026, 11, 11, 23, 30, 0)
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            span(keyed(d(2026, 11, 11), late, 60, Warsaw)),
            s("2026-11-11 23:00", "2026-11-12 00:00")
        );
    }

    #[test]
    fn times_follow_the_clock_across_a_daylight_saving_change() {
        // 29 March 2026: Warsaw's clocks skip from 2:00 to 3:00.
        let day = d(2026, 3, 29);
        let (start, end) = selected(day, 60, 240, Warsaw);
        assert_eq!(local(start), "2026-03-29 01:00");
        assert_eq!(local(end), "2026-03-29 04:00");
        assert_eq!(end - start, Duration::hours(2));
        // A range inside the skipped hour still lasts a step.
        assert_eq!(
            span(selected(day, 120, 150, Warsaw)),
            s("2026-03-29 03:00", "2026-03-29 03:15")
        );
        // A click in the skipped hour starts at the first time that exists.
        let (start, end) = clicked(day, 150, 60, Warsaw);
        assert_eq!(local(start), "2026-03-29 03:00");
        assert_eq!(end - start, Duration::hours(1));
        // The default length is real time: an hour from 1:30 ends at 3:30 on the clock.
        let (_, end) = clicked(day, 90, 60, Warsaw);
        assert_eq!(local(end), "2026-03-29 03:30");
    }

    fn entry(id: &str, email: &str) -> Entry {
        Entry {
            account: Account {
                id: id.to_string(),
                email: email.to_string(),
                workspace: false,
                contacts: true,
                directory: false,
            },
            state: AccountState::Connected,
        }
    }

    fn calendar(account: &str, id: &str, access: Access) -> Calendar {
        Calendar {
            id: id.to_string(),
            account_id: account.to_string(),
            name: id.to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access,
            zone: Warsaw,
        }
    }

    fn accounts() -> Vec<Entry> {
        vec![entry("1", "a@example.com"), entry("2", "b@example.com")]
    }

    fn calendars() -> Vec<Calendar> {
        vec![
            calendar("1", "holidays", Access::Reader),
            calendar("1", "team@example.com", Access::Writer),
            calendar("1", "a@example.com", Access::Owner),
            calendar("2", "b@example.com", Access::Owner),
            calendar("2", "room", Access::FreeBusyReader),
        ]
    }

    fn id(calendar: Option<Calendar>) -> Option<(String, String)> {
        calendar.map(|c| (c.account_id, c.id))
    }

    fn key(account: &str, calendar: &str) -> (String, String) {
        (account.to_string(), calendar.to_string())
    }

    #[test]
    fn until_the_user_chooses_new_events_go_to_the_first_accounts_primary_calendar() {
        let settings = Settings::default();
        assert_eq!(
            id(default_calendar(&accounts(), &calendars(), &settings)),
            Some(key("1", "a@example.com"))
        );
    }

    #[test]
    fn new_events_go_to_the_calendar_chosen_in_the_settings() {
        let settings = Settings {
            default_calendar: Some(key("2", "b@example.com")),
            ..Settings::default()
        };
        assert_eq!(
            id(default_calendar(&accounts(), &calendars(), &settings)),
            Some(key("2", "b@example.com"))
        );
    }

    #[test]
    fn a_chosen_calendar_that_can_no_longer_take_events_gives_way_to_the_primary() {
        let mut settings = Settings {
            default_calendar: Some(key("1", "holidays")),
            ..Settings::default()
        };
        assert_eq!(
            id(default_calendar(&accounts(), &calendars(), &settings)),
            Some(key("1", "a@example.com"))
        );
        settings.default_calendar = Some(key("2", "b@example.com"));
        settings.disabled.insert(key("2", "b@example.com"));
        settings.disabled.insert(key("1", "a@example.com"));
        assert_eq!(
            id(default_calendar(&accounts(), &calendars(), &settings)),
            Some(key("1", "team@example.com"))
        );
    }

    #[test]
    fn without_a_calendar_that_can_take_events_there_is_no_default() {
        let read_only = vec![calendar("1", "holidays", Access::Reader)];
        assert_eq!(
            default_calendar(&accounts(), &read_only, &Settings::default()),
            None
        );
        assert_eq!(
            default_calendar(&accounts(), &[], &Settings::default()),
            None
        );
    }

    #[test]
    fn the_picker_offers_only_calendars_that_can_take_events_under_their_accounts() {
        let mut settings = Settings::default();
        settings.disabled.insert(key("2", "b@example.com"));
        let offered: Vec<(String, Vec<String>)> = choices(&accounts(), &calendars(), &settings)
            .into_iter()
            .map(|(email, calendars)| (email, calendars.into_iter().map(|c| c.id).collect()))
            .collect();
        assert_eq!(
            offered,
            [(
                "a@example.com".to_string(),
                vec!["team@example.com".to_string(), "a@example.com".to_string()]
            )]
        );
    }

    fn planning() -> (Calendar, NewEvent) {
        let entry = QuickEntry {
            title: "  Planning ".to_string(),
            ..QuickEntry::new(
                selected(d(2026, 11, 12), 600, 690, Warsaw),
                Warsaw,
                Some(calendar("1", "a@example.com", Access::Owner)),
            )
        };
        entry.new_event().unwrap()
    }

    #[test]
    fn the_quick_entry_box_makes_the_event_with_its_title_times_and_calendar() {
        let (calendar, event) = planning();
        assert_eq!(calendar.id, "a@example.com");
        assert_eq!(event.title, "Planning");
        let Timing::Timed { start, end, zone } = event.timing else {
            panic!("a timed event");
        };
        assert_eq!(
            span((start, end)),
            s("2026-11-12 10:00", "2026-11-12 11:30")
        );
        assert_eq!(zone, Warsaw);
        assert_eq!(event.about, About::default());
        let without = QuickEntry::new((start, end), Warsaw, None);
        assert_eq!(without.new_event(), None);
    }

    fn shown(saving: &Saving) -> Vec<(String, bool)> {
        saving
            .events()
            .map(|e| (e.title.clone(), e.saving))
            .collect()
    }

    #[test]
    fn a_new_event_is_saving_until_google_confirms_it_and_shown_until_it_is_synchronized() {
        let (calendar, event) = planning();
        let mut saving = Saving::default();
        let local = saving.add(&calendar, &event);
        assert_eq!(shown(&saving), [("Planning".to_string(), true)]);

        saving.confirmed(local, "created-1");
        assert_eq!(shown(&saving), [("Planning".to_string(), false)]);
        // Synchronization has not brought it yet.
        saving.synced(&[]);
        assert_eq!(shown(&saving), [("Planning".to_string(), false)]);

        let mut synced = saving.events().next().unwrap().clone();
        synced.id = "created-1".to_string();
        saving.synced(&[synced]);
        assert!(saving.is_empty());
    }

    #[test]
    fn a_new_event_google_refused_goes() {
        let (calendar, event) = planning();
        let mut saving = Saving::default();
        let first = saving.add(&calendar, &event);
        let second = saving.add(
            &calendar,
            &NewEvent {
                title: String::new(),
                ..event
            },
        );
        saving.refused(first);
        assert_eq!(shown(&saving), [("(No title)".to_string(), true)]);
        // Still saving: an event of the same calendar synchronized does not take it away.
        let same = saving.events().next().unwrap().clone();
        saving.synced(&[same]);
        assert_eq!(saving.events().count(), 1);
        saving.forget_account("1");
        assert!(saving.is_empty());
        assert_ne!(first, second);
    }

    #[test]
    fn a_changed_event_is_shown_changed_and_saving_in_place_of_the_original_until_synchronized() {
        let before = crate::fixtures::timed("Standup", d(2026, 11, 13), (9, 0), (9, 30));
        let other = crate::fixtures::timed("Review", d(2026, 11, 13), (11, 0), (12, 0));
        let cached = vec![before.clone(), other.clone()];
        let after = Event {
            title: "Daily".to_string(),
            ..before.clone()
        };
        let mut saving = Saving::default();
        let local = saving.change(&before, after);
        let titles = |events: Vec<Event>| -> Vec<(String, bool)> {
            events.into_iter().map(|e| (e.title, e.saving)).collect()
        };
        assert_eq!(
            titles(saving.shown(&cached)),
            [("Review".to_string(), false), ("Daily".to_string(), true)]
        );
        assert!(saving.holds(&before));
        assert!(!saving.holds(&other));

        saving.changed(local, "\"2\"");
        // The version changed comes back first: still shown as changed.
        saving.synced(&cached);
        assert_eq!(
            titles(saving.shown(&cached)),
            [("Review".to_string(), false), ("Daily".to_string(), false)]
        );
        let synced = vec![
            Event {
                title: "Daily".to_string(),
                etag: "\"2\"".to_string(),
                ..before.clone()
            },
            other,
        ];
        saving.synced(&synced);
        assert!(saving.is_empty());
        assert_eq!(saving.shown(&synced), synced);
    }

    #[test]
    fn a_change_google_refused_shows_the_event_as_it_was() {
        let before = crate::fixtures::timed("Standup", d(2026, 11, 13), (9, 0), (9, 30));
        let mut saving = Saving::default();
        let local = saving.change(
            &before,
            Event {
                title: "Daily".to_string(),
                ..before.clone()
            },
        );
        saving.refused(local);
        assert_eq!(saving.shown(std::slice::from_ref(&before)), [before]);
    }

    #[test]
    fn a_moved_event_is_synchronized_back_once_it_is_in_the_calendar_moved_to() {
        let before = crate::fixtures::timed("Standup", d(2026, 11, 13), (9, 0), (9, 30));
        let moved = Event {
            calendar_id: "team".to_string(),
            ..before.clone()
        };
        let mut saving = Saving::default();
        let local = saving.change(&before, moved.clone());
        saving.changed(local, &before.etag);
        saving.synced(std::slice::from_ref(&before));
        assert!(!saving.is_empty());
        // Even if the provider kept its version.
        saving.synced(std::slice::from_ref(&moved));
        assert!(saving.is_empty());
    }
}
