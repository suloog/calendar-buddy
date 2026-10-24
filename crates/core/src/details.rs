//! What opening an event shows, independent of how it is drawn: its title and when it happens in
//! the system's time zone (`DEC-73`), and also in its own zone when that is another one
//! (`DEC-29`), its attachments as links (`MOD-05-F26`), and its guests with their answers
//! (`MOD-06-F05`).

use chrono::{DateTime, Datelike, Duration, NaiveDate};
use chrono_tz::Tz;

use crate::guests::Guest;
use crate::model::{Attachment, Event, Timing};
use crate::regional::Regional;

/// How many guests the details list before the rest is expanded (`DEC-63`).
pub const GUESTS_LISTED: usize = 10;

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
    /// The event's calendar cannot be changed (`MOD-02-U01`).
    pub read_only: bool,
    /// In the system's time zone.
    pub when: When,
    /// A timed event stored in another time zone, in that zone.
    pub own_zone: Option<When>,
    /// Listed by name, each opened in the browser (`MOD-05-F26`).
    pub attachments: Vec<Attachment>,
    /// Everyone invited, the user included, with their answers, in Google's order
    /// (`MOD-06-F05`).
    pub guests: Vec<Guest>,
}

pub fn event_details(event: &Event, zone: Tz) -> EventDetails {
    EventDetails {
        title: (!event.busy_only).then(|| event.title.clone()),
        read_only: event.read_only,
        attachments: event.about.attachments.clone(),
        guests: event.about.guests.clone(),
        own_zone: match event.timing {
            Timing::Timed {
                start,
                end,
                zone: own,
            } if own != zone => Some(When::Timed {
                start: start.with_timezone(&own),
                end: end.with_timezone(&own),
            }),
            _ => None,
        },
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

impl EventDetails {
    /// The guests listed: the first `GUESTS_LISTED` until the list is expanded (`DEC-63`).
    pub fn listed_guests(&self, expanded: bool) -> &[Guest] {
        if expanded {
            &self.guests
        } else {
            &self.guests[..self.guests.len().min(GUESTS_LISTED)]
        }
    }

    /// How many guests the collapsed list leaves out.
    pub fn more_guests(&self) -> usize {
        self.guests.len().saturating_sub(GUESTS_LISTED)
    }

    /// The time in the event's own zone, named after it, when that is not the system's.
    pub fn in_own_zone(&self, regional: &Regional) -> Option<String> {
        match &self.own_zone {
            Some(when @ When::Timed { start, .. }) => Some(format!(
                "{} ({})",
                when.describe(regional),
                start.timezone().name()
            )),
            _ => None,
        }
    }
}

impl When {
    /// As the details show it, in the system's formats: `Thursday 12 November 2026, 15:00 – 16:00`.
    pub fn describe(&self, regional: &Regional) -> String {
        match *self {
            // An event ending at midnight stays on its day: 23:00 – 00:00.
            When::Timed { start, end } => {
                let day = start.date_naive();
                let same_day = day == (end - Duration::seconds(1)).max(start).date_naive();
                if same_day {
                    format!(
                        "{}, {} – {}",
                        regional.long_date(day),
                        regional.time(start.time()),
                        regional.time(end.time())
                    )
                } else {
                    format!(
                        "{}, {} – {}, {}",
                        regional.day_month_year(day),
                        regional.time(start.time()),
                        regional.day_month_year(end.date_naive()),
                        regional.time(end.time())
                    )
                }
            }
            When::AllDay { first, last } if first == last => regional.long_date(first),
            When::AllDay { first, last } if first.year() == last.year() => format!(
                "{} – {}",
                regional.day_month(first),
                regional.day_month_year(last)
            ),
            When::AllDay { first, last } => format!(
                "{} – {}",
                regional.day_month_year(first),
                regional.day_month_year(last)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{all_day, d, timed};
    use crate::guests::{Guest, Response};
    use chrono::NaiveTime;
    use chrono_tz::{America::New_York, Europe::Warsaw, UTC};

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
    fn an_event_opens_with_its_attachments() {
        let mut event = timed("Review", d(2026, 11, 12), (9, 0), (10, 0));
        assert_eq!(event_details(&event, Warsaw).attachments, []);
        let numbers = Attachment {
            title: "Q4 numbers".into(),
            url: "https://docs.google.com/document/d/1abc/edit".into(),
        };
        event.about.attachments = vec![numbers.clone()];
        assert_eq!(event_details(&event, Warsaw).attachments, [numbers]);
    }

    fn guest(name: &str, email: &str, response: Response) -> Guest {
        Guest {
            email: email.to_string(),
            name: name.to_string(),
            optional: false,
            response,
            own: false,
        }
    }

    #[test]
    fn an_event_opens_with_each_guests_name_address_and_answer() {
        let mut event = timed("Kickoff", d(2026, 11, 13), (10, 0), (11, 0));
        assert!(event_details(&event, Warsaw).guests.is_empty());
        event.about.guests = vec![
            Guest {
                own: true,
                ..guest("", "owner@example.com", Response::Accepted)
            },
            guest("Adam Kowalski", "adam@example.com", Response::Accepted),
            guest("", "ewa@example.com", Response::Awaiting),
            guest("Ola Nowak", "ola@example.com", Response::Declined),
            guest("Jan", "jan@example.com", Response::Maybe),
        ];

        let details = event_details(&event, Warsaw);

        // All of them, the user included, in Google's order.
        assert_eq!(details.guests, event.about.guests);
        assert_eq!(details.listed_guests(false), &event.about.guests[..]);
        assert_eq!(details.more_guests(), 0);
    }

    #[test]
    fn each_answer_has_its_own_icon_and_name() {
        let answers = [
            Response::Accepted,
            Response::Declined,
            Response::Maybe,
            Response::Awaiting,
        ];
        let icons: Vec<&str> = answers.iter().map(|r| r.icon()).collect();
        assert_eq!(icons, ["✔", "✖", "❓", "⏳"]);
        let names: Vec<&str> = answers.iter().map(|r| r.label()).collect();
        assert_eq!(names, ["Accepted", "Declined", "Maybe", "Awaiting"]);
    }

    #[test]
    fn a_guest_list_of_more_than_10_shows_the_first_10_until_expanded() {
        let mut event = timed("All hands", d(2026, 11, 13), (10, 0), (11, 0));
        event.about.guests = (1..=12)
            .map(|i| guest("", &format!("p{i}@example.com"), Response::Awaiting))
            .collect();

        let details = event_details(&event, Warsaw);

        assert_eq!(details.listed_guests(false), &event.about.guests[..10]);
        assert_eq!(details.more_guests(), 2);
        assert_eq!(details.listed_guests(true), &event.about.guests[..]);

        // Exactly 10 are all listed.
        event.about.guests.truncate(10);
        let details = event_details(&event, Warsaw);
        assert_eq!(details.listed_guests(false).len(), 10);
        assert_eq!(details.more_guests(), 0);
    }

    #[test]
    fn a_free_busy_event_opens_without_its_title() {
        let mut event = timed("Room booking", d(2026, 11, 13), (13, 0), (14, 0));
        event.busy_only = true;
        assert_eq!(event_details(&event, Warsaw).title, None);
    }

    #[test]
    fn an_event_in_another_zone_also_shows_its_time_in_that_zone() {
        // System zone America/New_York; the event is stored in Europe/Warsaw.
        let event = timed("Call", d(2026, 11, 12), (15, 0), (16, 0));
        let us = Regional::new(pure_rust_locales::Locale::en_US);
        let details = event_details(&event, New_York);
        assert_eq!(
            details.when.describe(&us),
            "Thursday, November 12, 2026, 9:00 AM – 10:00 AM"
        );
        assert_eq!(
            details.in_own_zone(&us).as_deref(),
            Some("Thursday, November 12, 2026, 3:00 PM – 4:00 PM (Europe/Warsaw)")
        );
    }

    #[test]
    fn an_event_in_the_system_zone_or_all_day_shows_no_other_zone() {
        let gb = Regional::new(pure_rust_locales::Locale::en_GB);
        let event = timed("Call", d(2026, 11, 12), (15, 0), (16, 0));
        let details = event_details(&event, Warsaw);
        assert_eq!(details.own_zone, None);
        assert_eq!(details.in_own_zone(&gb), None);
        assert_eq!(
            details.when.describe(&gb),
            "Thursday 12 November 2026, 15:00 – 16:00"
        );
        assert_eq!(
            event_details(&all_day("trip", d(2026, 11, 12), 1), New_York).own_zone,
            None
        );
    }

    #[test]
    fn the_details_write_days_and_spans_in_the_system_formats() {
        let gb = Regional::new(pure_rust_locales::Locale::en_GB);
        let describe = |event: &Event| event_details(event, Warsaw).when.describe(&gb);
        assert_eq!(
            describe(&timed("late", d(2026, 11, 12), (23, 0), (24, 0))),
            "Thursday 12 November 2026, 23:00 – 00:00"
        );
        assert_eq!(
            describe(&timed("night", d(2026, 11, 12), (22, 0), (26, 0))),
            "12 November 2026, 22:00 – 13 November 2026, 02:00"
        );
        assert_eq!(
            describe(&all_day("trip", d(2026, 11, 12), 1)),
            "Thursday 12 November 2026"
        );
        assert_eq!(
            describe(&all_day("trip", d(2026, 11, 12), 3)),
            "12 November – 14 November 2026"
        );
        assert_eq!(
            describe(&all_day("trip", d(2026, 12, 30), 4)),
            "30 December 2026 – 2 January 2027"
        );
    }
}
