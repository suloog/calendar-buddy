//! The first full synchronization of an account (`US-02-01`): every calendar it can access, and
//! their events within the cache window, written to the cache. The provider (Google) sits behind
//! `Provider`, so nothing here knows about it (`NFR-14`).

use chrono::{DateTime, Months, NaiveDate, Utc};
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
}

/// Why synchronizing an account or a calendar did not finish; never holds a token or event
/// contents (`NFR-07`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncError(pub String);

/// One account at a calendar provider.
pub trait Provider {
    /// Every calendar the account can access, read-only and subscribed ones included.
    fn calendars(&self) -> Result<Vec<Calendar>, SyncError>;
    /// The calendar's events that fall inside the window, series as master plus exceptions.
    fn events(&self, calendar: &Calendar, window: Window) -> Result<Vec<SyncedEvent>, SyncError>;
}

/// Fetches every calendar of the account and its events within the window, replacing what the
/// cache held for the account. A calendar that fails keeps what the cache had and does not stop
/// the others; the first such failure is returned once they are done.
pub fn full_sync(
    provider: &dyn Provider,
    account_id: &str,
    window: Window,
    cache: &mut Cache,
) -> Result<(), SyncError> {
    let calendars = provider.calendars()?;
    cache.set_calendars(account_id, &calendars)?;
    let mut failed = None;
    for calendar in &calendars {
        match provider.events(calendar, window) {
            Ok(events) => cache.set_events(calendar, &events)?,
            Err(error) => {
                failed.get_or_insert(error);
            }
        }
    }
    failed.map_or(Ok(()), Err)
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
    use chrono::TimeZone;
    use chrono_tz::Europe::Warsaw;

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
