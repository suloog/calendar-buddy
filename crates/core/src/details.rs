//! What opening an event shows, independent of how it is drawn: its title and when it happens in
//! the system's time zone (`DEC-73`). The rest of the details come with `US-03-09` and `MOD-06`.

use chrono::{DateTime, Duration, NaiveDate};
use chrono_tz::Tz;

use crate::model::{Event, Timing};

#[derive(Clone, Debug, PartialEq)]
pub enum When {
    Timed {
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    },
    /// The first and the last day, both included.
    AllDay { first: NaiveDate, last: NaiveDate },
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventDetails {
    /// `None` for events of calendars shared as free/busy only.
    pub title: Option<String>,
    pub when: When,
}

pub fn event_details(event: &Event, zone: Tz) -> EventDetails {
    EventDetails {
        title: (!event.busy_only).then(|| event.title.clone()),
        when: match event.timing {
            Timing::Timed { start, end, .. } => When::Timed {
                start: start.with_timezone(&zone),
                end: end.with_timezone(&zone),
            },
            Timing::AllDay {
                start,
                end_exclusive,
            } => When::AllDay {
                first: start,
                last: (end_exclusive - Duration::days(1)).max(start),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{all_day, d, timed};
    use chrono::NaiveTime;
    use chrono_tz::{Europe::Warsaw, UTC};

    #[test]
    fn a_timed_event_opens_with_its_title_and_times_in_the_system_zone() {
        let event = timed("Standup", d(2026, 11, 9), (9, 0), (9, 30));
        let details = event_details(&event, UTC);
        assert_eq!(details.title.as_deref(), Some("Standup"));
        let When::Timed { start, end } = details.when else {
            panic!("timed");
        };
        assert_eq!(start.time(), NaiveTime::from_hms_opt(8, 0, 0).unwrap());
        assert_eq!(end.time(), NaiveTime::from_hms_opt(8, 30, 0).unwrap());
    }

    #[test]
    fn an_all_day_event_opens_with_its_first_and_last_day() {
        let details = event_details(&all_day("trip", d(2026, 11, 12), 3), Warsaw);
        assert_eq!(
            details.when,
            When::AllDay {
                first: d(2026, 11, 12),
                last: d(2026, 11, 14)
            }
        );
    }

    #[test]
    fn a_free_busy_event_opens_without_its_title() {
        let mut event = timed("Room booking", d(2026, 11, 13), (13, 0), (14, 0));
        event.busy_only = true;
        assert_eq!(event_details(&event, Warsaw).title, None);
    }
}
