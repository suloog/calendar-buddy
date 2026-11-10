//! Telling of new invitations (`US-07-08`, `MOD-07-F22`, `F24`, `DEC-222`): an invitation
//! awaiting the user's answer, as the side panel counts them (`invitations::awaiting`), is told
//! once, as a desktop notification. What was told is kept, in the cache, so a restart tells
//! nothing again and what came while the app was off is told at its start. A calendar looked at
//! for the first time — newly connected, newly enabled, or the first time this version runs —
//! has its invitations kept as told without a notification; an invitation answered, over or
//! gone is forgotten, so it is told again if it awaits an answer again.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::accounts::Entry;
use crate::invitations;
use crate::model::Event;
use crate::settings::Settings;

/// A calendar, by account ID and calendar ID.
pub type CalendarKey = (String, String);

/// What was told, as kept in the cache.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Told {
    /// The calendars looked at, whose invitations from then on are told.
    pub calendars: BTreeSet<CalendarKey>,
    /// The invitations told, or kept as told on a first look: by account ID, calendar ID and
    /// the ID of their series, or their own.
    pub invitations: BTreeSet<(String, String, String)>,
}

/// What is told of the invitations among `events`, those of the `synchronized` calendars, as
/// `told` before: what to keep as told now, and the places in `events` of the invitations to
/// tell, the earliest first. `accounts` give each account's own address, and calendars
/// `disabled` in the settings are not looked at (`MOD-06-F21`); what was told of an account not
/// among them is kept as it was, as its invitations cannot be told apart yet.
pub fn tell(
    told: &Told,
    synchronized: &BTreeSet<CalendarKey>,
    events: &[Event],
    accounts: &[Entry],
    disabled: &BTreeSet<CalendarKey>,
    zone: Tz,
    now: DateTime<Utc>,
) -> (Told, Vec<usize>) {
    let settings = Settings {
        disabled: disabled.clone(),
        ..Settings::default()
    };
    let named = |account: &str| accounts.iter().any(|entry| entry.account.id == account);
    let calendars: BTreeSet<CalendarKey> = (synchronized.difference(disabled))
        .filter(|(account, _)| named(account))
        .cloned()
        .collect();
    let awaiting: Vec<(usize, (String, String, String))> =
        invitations::awaiting(events, accounts, &settings, zone, now)
            .into_iter()
            .map(|i| (i, key(&events[i])))
            .filter(|(_, (account, calendar, _))| {
                calendars.contains(&(account.clone(), calendar.clone()))
            })
            .collect();
    let new = (awaiting.iter())
        .filter(|(_, (account, calendar, _))| {
            (told.calendars).contains(&(account.clone(), calendar.clone()))
        })
        .filter(|(_, key)| !told.invitations.contains(key))
        .map(|(i, _)| *i)
        .collect();
    let unnamed = |account: &str| !named(account);
    let now_told = Told {
        calendars: (calendars.into_iter())
            .chain(told.calendars.iter().filter(|(a, _)| unnamed(a)).cloned())
            .collect(),
        invitations: (awaiting.into_iter().map(|(_, key)| key))
            .chain(
                told.invitations
                    .iter()
                    .filter(|(a, _, _)| unnamed(a))
                    .cloned(),
            )
            .collect(),
    };
    (now_told, new)
}

/// What tells the invitation apart: its series once, as the side panel counts it (`DEC-125`).
fn key(event: &Event) -> (String, String, String) {
    let id = (event.series.as_ref()).map_or(&event.id, |(series, _)| series);
    (
        event.account_id.clone(),
        event.calendar_id.clone(),
        id.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{Account, AccountState};
    use crate::fixtures::{d, timed};
    use crate::guests::{Guest, Response};
    use chrono::TimeZone;
    use chrono_tz::Europe::Warsaw;

    /// Thursday 2026-11-12 09:00 in Warsaw.
    fn clock() -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(2026, 11, 12, 9, 0, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

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

    fn work() -> BTreeSet<CalendarKey> {
        BTreeSet::from([("1".to_string(), "work".to_string())])
    }

    /// An invitation to the account's own address, on Friday from 10:00.
    fn invitation(id: &str) -> Event {
        let mut event = timed(id, d(2026, 11, 13), (10, 0), (11, 0));
        event.about.guests = vec![Guest {
            email: "owner@example.com".to_string(),
            name: String::new(),
            optional: false,
            response: Response::Awaiting,
            own: true,
        }];
        event
    }

    fn told_now(told: &Told, events: &[Event]) -> (Told, Vec<String>) {
        let (now, places) = tell(
            told,
            &work(),
            events,
            &accounts(),
            &BTreeSet::new(),
            Warsaw,
            clock(),
        );
        let titles = places
            .into_iter()
            .map(|i| events[i].title.clone())
            .collect();
        (now, titles)
    }

    #[test]
    fn tc_07_11_a_first_look_tells_nothing_and_a_new_invitation_is_told_once() {
        // 1. The calendar's first synchronization: "Old" is kept as told, without a notification.
        let (told, titles) = told_now(&Told::default(), &[invitation("Old")]);
        assert!(titles.is_empty());
        assert!(
            told.calendars
                .contains(&("1".to_string(), "work".to_string()))
        );

        // 2. "Planning" comes: it alone is told.
        let events = [invitation("Old"), invitation("Planning")];
        let (told, titles) = told_now(&told, &events);
        assert_eq!(titles, ["Planning"]);

        // 3. Again, as after a restart with the same cache: nothing.
        let (told, titles) = told_now(&told, &events);
        assert!(titles.is_empty());
        assert_eq!(told.invitations.len(), 2);
    }

    #[test]
    fn an_answered_invitation_is_forgotten_and_told_again_if_it_awaits_an_answer_again() {
        let (told, _) = told_now(&Told::default(), &[]);
        let (told, titles) = told_now(&told, &[invitation("Retro")]);
        assert_eq!(titles, ["Retro"]);
        let mut answered = invitation("Retro");
        answered.about.guests[0].response = Response::Accepted;
        let (told, _) = told_now(&told, &[answered]);
        assert!(told.invitations.is_empty());
        let (_, titles) = told_now(&told, &[invitation("Retro")]);
        assert_eq!(titles, ["Retro"]);
    }

    #[test]
    fn a_disabled_or_unsynchronized_calendar_is_looked_at_anew_once_it_is_back() {
        let (told, _) = told_now(&Told::default(), &[]);
        let events = [invitation("Board")];
        // Disabled: not looked at, and forgotten.
        let (told, places) = tell(
            &told,
            &work(),
            &events,
            &accounts(),
            &work(),
            Warsaw,
            clock(),
        );
        assert!(places.is_empty());
        assert!(told.calendars.is_empty());
        // Enabled again: a first look, which tells nothing.
        let (told, titles) = told_now(&told, &events);
        assert!(titles.is_empty());
        assert!(!told.calendars.is_empty());
        // Not synchronized (yet): not looked at.
        let (_, places) = tell(
            &told,
            &BTreeSet::new(),
            &[invitation("Board"), invitation("Kickoff")],
            &accounts(),
            &BTreeSet::new(),
            Warsaw,
            clock(),
        );
        assert!(places.is_empty());
    }

    #[test]
    fn tc_07_11_a_calendar_of_an_account_not_yet_known_is_looked_at_only_once_it_is() {
        let events = [invitation("Old")];
        // Synchronized before the app says whose account it is: nothing is looked at.
        let (told, places) = tell(
            &Told::default(),
            &work(),
            &events,
            &[],
            &BTreeSet::new(),
            Warsaw,
            clock(),
        );
        assert!(places.is_empty());
        assert_eq!(told, Told::default());
        // Then the first look, which tells nothing.
        let (told, titles) = told_now(&told, &events);
        assert!(titles.is_empty());
        // What was told of it stays while the account is not known again.
        let (kept, places) = tell(
            &told,
            &work(),
            &events,
            &[],
            &BTreeSet::new(),
            Warsaw,
            clock(),
        );
        assert!(places.is_empty());
        assert_eq!(kept, told);
    }

    #[test]
    fn a_series_is_told_once() {
        let (told, _) = told_now(&Told::default(), &[]);
        let occurrence = |day| {
            let mut event = invitation(&format!("Weekly_{day}"));
            event.title = "Weekly".to_string();
            event.series = Some((
                "Weekly".to_string(),
                crate::sync::Start::On(d(2026, 11, day)),
            ));
            event
        };
        let (_, titles) = told_now(&told, &[occurrence(13), occurrence(20)]);
        assert_eq!(titles, ["Weekly"]);
    }
}
