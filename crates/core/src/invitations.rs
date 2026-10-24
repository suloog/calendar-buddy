//! Invitations awaiting the user's answer (`US-06-04`): marked in every view (`MOD-06-F09`,
//! `MOD-06-U04`) and counted in the side panel, whose counter opens a list of them
//! (`MOD-06-F10`); those of calendars disabled in the settings are neither shown nor counted
//! (`MOD-06-F21`).

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::accounts::Entry;
use crate::guests::{Response, same};
use crate::model::{Event, Timing};
use crate::settings::Settings;
use crate::week::day_start;

/// What the list of invitations says when none awaits an answer.
pub const NONE: &str = "No invitations await an answer.";

/// What the side panel's counter says, with the number of invitations awaiting an answer.
pub fn counter(count: usize) -> String {
    match count {
        1 => "1 invitation awaiting an answer".to_string(),
        n => format!("{n} invitations awaiting an answer"),
    }
}

/// The places in `events` of the invitations awaiting the user's answer, the earliest first: those
/// whose guests have the address of the account the event is listed under unanswered, so not a
/// colleague's in a calendar shared with the user; those not over by `now`, of calendars not
/// disabled in the settings, and a series once, by its next occurrence (`DEC-125`, `DEC-127`).
pub fn awaiting(
    events: &[Event],
    accounts: &[Entry],
    settings: &Settings,
    zone: Tz,
    now: DateTime<Utc>,
) -> Vec<usize> {
    let users = |event: &Event| {
        let email = (accounts.iter())
            .find(|entry| entry.account.id == event.account_id)
            .map(|entry| &entry.account.email);
        email.is_some_and(|email| {
            (event.about.guests.iter()).any(|guest| {
                guest.own && guest.response == Response::Awaiting && same(&guest.email, email)
            })
        })
    };
    let mut places: Vec<usize> = (0..events.len())
        .filter(|&i| {
            let event = &events[i];
            let key = (event.account_id.clone(), event.calendar_id.clone());
            users(event) && ends(event, zone) > now && !settings.disabled.contains(&key)
        })
        .collect();
    places.sort_by_key(|&i| starts(&events[i], zone));
    let mut seen = HashSet::new();
    places.retain(|&i| {
        let event = &events[i];
        let id = event
            .series
            .as_ref()
            .map_or(&event.id, |(series, _)| series);
        seen.insert((&event.account_id, &event.calendar_id, id))
    });
    places
}

/// When the event starts; an all-day one at the start of its first day in `zone`.
fn starts(event: &Event, zone: Tz) -> DateTime<Utc> {
    match event.timing {
        Timing::Timed { start, .. } => start,
        Timing::AllDay { start, .. } => day_start(start, zone),
    }
}

/// When the event is over; an all-day one at the end of its last day in `zone`.
fn ends(event: &Event, zone: Tz) -> DateTime<Utc> {
    match event.timing {
        Timing::Timed { end, .. } => end,
        Timing::AllDay { end_exclusive, .. } => day_start(end_exclusive, zone),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{Account, AccountState};
    use crate::agenda::agenda_view;
    use crate::fixtures::{all_day, d, timed};
    use crate::guests::{Guest, Person};
    use crate::month::month_view;
    use crate::side_panel;
    use crate::sync::Start;
    use crate::view_model::grid_view;
    use crate::week::Week;
    use chrono::{TimeZone, Weekday};
    use chrono_tz::Europe::Warsaw;

    /// Thursday 2026-11-12 09:00 in Warsaw.
    fn clock() -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(2026, 11, 12, 9, 0, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    /// The connected account "1", owner@example.com.
    fn accounts() -> Vec<Entry> {
        vec![Entry {
            account: Account {
                id: "1".to_string(),
                email: "owner@example.com".to_string(),
                workspace: false,
                contacts: false,
                directory: false,
            },
            state: AccountState::Connected,
        }]
    }

    fn guest(email: &str, own: bool, response: Response) -> Guest {
        let person = Person {
            name: String::new(),
            email: email.to_string(),
        };
        Guest {
            own,
            response,
            ..Guest::invited(&person)
        }
    }

    /// The event as an invitation from someone else, the user having answered it `response`.
    fn invited(mut event: Event, response: Response) -> Event {
        event.about.guests = vec![
            guest("boss@example.com", false, Response::Accepted),
            guest("owner@example.com", true, response),
        ];
        event
    }

    fn titles(events: &[Event], places: &[usize]) -> Vec<String> {
        places.iter().map(|&i| events[i].title.clone()).collect()
    }

    /// The week of 2026-11-09: "Board" and "Lunch" await an answer, "Review" was accepted and
    /// "Standup" has no guests.
    fn tc_06_05_week() -> Vec<Event> {
        vec![
            invited(
                timed("Board", d(2026, 11, 12), (14, 0), (15, 0)),
                Response::Awaiting,
            ),
            invited(
                timed("Review", d(2026, 11, 12), (11, 0), (12, 0)),
                Response::Accepted,
            ),
            timed("Standup", d(2026, 11, 13), (9, 0), (9, 15)),
            invited(
                timed("Lunch", d(2026, 11, 13), (12, 0), (13, 0)),
                Response::Awaiting,
            ),
        ]
    }

    #[test]
    fn tc_06_05_invitations_awaiting_an_answer_are_marked_in_the_week_view() {
        let events = tc_06_05_week();
        let days = Week::containing(d(2026, 11, 12), Weekday::Mon)
            .days()
            .to_vec();

        let view = grid_view(&events, &days, Warsaw, clock());

        let marked: Vec<(String, bool)> = (view.blocks.iter())
            .map(|block| (block.title.clone().unwrap(), block.awaiting))
            .collect();
        assert_eq!(marked.len(), 4);
        for (title, awaiting) in marked {
            assert_eq!(awaiting, title == "Board" || title == "Lunch", "{title}");
        }
    }

    #[test]
    fn invitations_awaiting_an_answer_are_marked_in_the_month_view_and_the_agenda_too() {
        let events = tc_06_05_week();
        let awaits = |title: &str| title == "Board" || title == "Lunch";

        let month = month_view(&events, d(2026, 11, 12), Warsaw, clock(), Weekday::Mon);
        let agenda = agenda_view(&events, Warsaw, clock(), 7);

        let in_month: Vec<(&str, bool)> = (month.weeks.iter().flatten())
            .flat_map(|day| &day.items)
            .map(|item| (item.title.as_deref().unwrap(), item.awaiting))
            .collect();
        let in_agenda: Vec<(&str, bool)> = (agenda.days.iter())
            .flat_map(|day| &day.items)
            .map(|item| (item.title.as_deref().unwrap(), item.awaiting))
            .collect();
        assert_eq!((in_month.len(), in_agenda.len()), (4, 4));
        for (title, awaiting) in in_month.into_iter().chain(in_agenda) {
            assert_eq!(awaiting, awaits(title), "{title}");
        }
    }

    #[test]
    fn tc_06_05_the_side_panel_counts_two_invitations_and_lists_board_and_lunch() {
        let events = tc_06_05_week();

        let awaiting = awaiting(&events, &accounts(), &Settings::default(), Warsaw, clock());

        assert_eq!(counter(awaiting.len()), "2 invitations awaiting an answer");
        assert_eq!(titles(&events, &awaiting), ["Board", "Lunch"]);
    }

    #[test]
    fn invitations_of_a_calendar_disabled_in_the_settings_are_neither_shown_nor_counted() {
        let mut events = tc_06_05_week();
        events[3].calendar_id = "team".to_string();
        let mut settings = Settings::default();
        (settings.disabled).insert(("1".to_string(), "team".to_string()));

        let awaiting = awaiting(&events, &accounts(), &settings, Warsaw, clock());
        let shown = side_panel::shown(&events, &settings);

        assert_eq!(titles(&events, &awaiting), ["Board"]);
        assert!(!titles(&events, &shown).contains(&"Lunch".to_string()));
        assert_eq!(counter(awaiting.len()), "1 invitation awaiting an answer");
    }

    #[test]
    fn invitations_of_a_calendar_hidden_by_the_quick_toggle_are_still_counted() {
        let mut events = tc_06_05_week();
        events[3].calendar_id = "team".to_string();
        let mut settings = Settings::default();
        (settings.hidden).insert(("1".to_string(), "team".to_string()));

        let awaiting = awaiting(&events, &accounts(), &settings, Warsaw, clock());

        assert_eq!(titles(&events, &awaiting), ["Board", "Lunch"]);
    }

    #[test]
    fn a_colleagues_unanswered_invitation_in_their_shared_calendar_is_marked_but_not_counted() {
        // Google marks as `self` the entry of the calendar the copy is on: here the colleague's.
        let mut theirs = timed("Their 1:1", d(2026, 11, 13), (10, 0), (11, 0));
        theirs.calendar_id = "colleague@example.com".to_string();
        theirs.about.guests = vec![
            guest("boss@example.com", false, Response::Accepted),
            guest("colleague@example.com", true, Response::Awaiting),
        ];
        let events = vec![theirs];

        let awaiting = awaiting(&events, &accounts(), &Settings::default(), Warsaw, clock());

        assert!(events[0].about.awaiting());
        assert!(awaiting.is_empty());
    }

    #[test]
    fn only_the_users_own_unanswered_entry_makes_an_invitation_await_an_answer() {
        let mut someone_else = invited(
            timed("Other", d(2026, 11, 13), (9, 0), (10, 0)),
            Response::Accepted,
        );
        someone_else.about.guests[0].response = Response::Awaiting;
        for (event, awaits) in [
            (
                invited(
                    timed("A", d(2026, 11, 13), (9, 0), (10, 0)),
                    Response::Awaiting,
                ),
                true,
            ),
            (
                invited(
                    timed("B", d(2026, 11, 13), (9, 0), (10, 0)),
                    Response::Maybe,
                ),
                false,
            ),
            (
                invited(
                    timed("C", d(2026, 11, 13), (9, 0), (10, 0)),
                    Response::Declined,
                ),
                false,
            ),
            (someone_else, false),
            (timed("D", d(2026, 11, 13), (9, 0), (10, 0)), false),
        ] {
            assert_eq!(event.about.awaiting(), awaits, "{}", event.title);
        }
    }

    #[test]
    fn invitations_already_over_are_not_counted_and_an_all_day_one_counts_until_its_day_ends() {
        let events = vec![
            invited(
                timed("Yesterday", d(2026, 11, 11), (9, 0), (10, 0)),
                Response::Awaiting,
            ),
            invited(
                timed("Earlier today", d(2026, 11, 12), (7, 0), (8, 0)),
                Response::Awaiting,
            ),
            invited(
                timed("Now", d(2026, 11, 12), (8, 30), (9, 30)),
                Response::Awaiting,
            ),
            invited(all_day("Today", d(2026, 11, 12), 1), Response::Awaiting),
        ];

        let awaiting = awaiting(&events, &accounts(), &Settings::default(), Warsaw, clock());

        assert_eq!(titles(&events, &awaiting), ["Today", "Now"]);
    }

    #[test]
    fn a_recurring_invitation_is_counted_once_by_its_next_occurrence() {
        let occurrence = |date| {
            let mut event = invited(timed("Weekly", date, (10, 0), (11, 0)), Response::Awaiting);
            event.id = format!("weekly_{date}");
            event.recurring = true;
            event.series = Some(("weekly".to_string(), Start::On(date)));
            event
        };
        let events = vec![
            occurrence(d(2026, 11, 19)),
            occurrence(d(2026, 11, 5)),
            occurrence(d(2026, 11, 12)),
            occurrence(d(2026, 11, 26)),
        ];

        let awaiting = awaiting(&events, &accounts(), &Settings::default(), Warsaw, clock());

        assert_eq!(awaiting, [2]);
    }
}
