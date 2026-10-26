//! Working from the cache without network (`US-02-05`): everything cached can be browsed, the
//! main window says the app is offline (`MOD-02-U02`), and changes, which need Google, are refused
//! with the reason (`MOD-02-F15`, `DEC-25`). Days the cache does not hold are fetched while online
//! and shown empty with a note while offline (`MOD-02-F12`).

use chrono::NaiveDate;

use crate::local;
use crate::model::Calendar;
use crate::sync::Window;
use crate::texts::{NO_CHANGES_OFFLINE, NOT_CACHED, OFFLINE_INDICATOR};

/// The offline indicator, while offline (`MOD-02-U02`).
pub fn indicator(online: bool) -> Option<&'static str> {
    (!online).then_some(OFFLINE_INDICATOR)
}

/// Why creating, editing or deleting an event or answering an invitation is refused now; `None`
/// while online.
pub fn refused(online: bool) -> Option<&'static str> {
    (!online).then_some(NO_CHANGES_OFFLINE)
}

/// Why creating or editing an event in the calendar is refused now: as `refused`, but never in a
/// local calendar, which needs no Google (MOD-09 *Constraints and edge cases*).
pub fn refused_in(online: bool, calendar: Option<&Calendar>) -> Option<&'static str> {
    refused(online)
        .filter(|_| calendar.is_none_or(|calendar| calendar.account_id != local::ACCOUNT_ID))
}

/// What becomes of the days on screen, from `first` to `last`, that the cache window does not
/// hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Browsed {
    /// The span to fetch from Google, online.
    pub fetch: Option<Window>,
    /// The note shown over them, offline.
    pub note: Option<&'static str>,
}

pub fn browsed(window: Window, first: NaiveDate, last: NaiveDate, online: bool) -> Browsed {
    let outside = window.outside(first, last);
    Browsed {
        fetch: outside.filter(|_| online),
        note: (outside.is_some() && !online).then_some(NOT_CACHED),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;
    use chrono_tz::Europe::Warsaw;

    fn window() -> Window {
        Window::around(d(2026, 11, 11), Warsaw)
    }

    #[test]
    fn offline_a_local_calendar_still_takes_new_events_and_a_google_one_does_not() {
        let mine = local::my_calendar(Warsaw);
        let google = Calendar {
            account_id: "1".to_string(),
            ..mine.clone()
        };

        assert_eq!(refused_in(false, Some(&mine)), None);
        assert_eq!(refused_in(false, Some(&google)), Some(NO_CHANGES_OFFLINE));
        assert_eq!(refused_in(false, None), Some(NO_CHANGES_OFFLINE));
        assert_eq!(refused_in(true, Some(&google)), None);
    }

    #[test]
    fn offline_every_change_is_refused_with_the_reason_and_online_none_is() {
        assert_eq!(indicator(false), Some(OFFLINE_INDICATOR));
        assert_eq!(indicator(true), None);
        assert_eq!(refused(false), Some(NO_CHANGES_OFFLINE));
        assert!(NO_CHANGES_OFFLINE.contains("not available offline"));
        assert_eq!(refused(true), None);
    }

    #[test]
    fn online_days_outside_the_cache_window_are_fetched_without_a_note() {
        let browsed = browsed(window(), d(2030, 3, 1), d(2030, 3, 31), true);

        assert_eq!(
            browsed.fetch,
            window().outside(d(2030, 3, 1), d(2030, 3, 31))
        );
        assert!(browsed.fetch.is_some());
        assert_eq!(browsed.note, None);
    }

    #[test]
    fn offline_days_outside_the_cache_window_are_not_fetched_and_get_the_note() {
        let browsed = browsed(window(), d(2030, 3, 1), d(2030, 3, 31), false);

        assert_eq!(browsed.fetch, None);
        assert_eq!(browsed.note, Some(NOT_CACHED));
    }

    #[test]
    fn days_the_cache_holds_need_neither_a_fetch_nor_a_note() {
        for online in [true, false] {
            let browsed = browsed(window(), d(2026, 11, 9), d(2026, 11, 15), online);
            assert_eq!(
                browsed,
                Browsed {
                    fetch: None,
                    note: None
                }
            );
        }
    }
}
