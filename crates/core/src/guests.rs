//! An event's guests (`US-06-01`): added by e-mail address (`MOD-06-F01`), marked required or
//! optional (`MOD-06-F06`), and suggested while the user types from the account's contacts, the
//! addresses it has invited (`MOD-06-F02`, `F03`) and, on Workspace accounts, its organization's
//! directory (`MOD-06-F04`).

use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::accounts::Account;
use crate::model::{Event, Timing};

/// How many people are suggested at most while the user types.
pub const SUGGESTIONS: usize = 8;

/// What the guest field says when what was typed is not an e-mail address.
pub const NOT_AN_ADDRESS: &str = "This is not a valid e-mail address.";

/// A guest of an event, as the provider keeps them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guest {
    pub email: String,
    /// The name the provider gives them; empty when it has none.
    pub name: String,
    /// Marked optional; otherwise required (`MOD-06-F06`).
    pub optional: bool,
    /// Their answer to the invitation, kept as it is when the guests are changed.
    pub response: Response,
    /// The account whose calendar the event is on, which the editor does not list as a guest it
    /// invites.
    pub own: bool,
}

impl Guest {
    /// A guest just invited: required, not answered yet.
    pub fn invited(person: &Person) -> Self {
        Guest {
            email: person.email.clone(),
            name: person.name.clone(),
            optional: false,
            response: Response::Awaiting,
            own: false,
        }
    }
}

/// A guest's answer to an invitation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Response {
    #[default]
    Awaiting,
    Accepted,
    Declined,
    Maybe,
}

impl Response {
    /// How the guest list shows it (`MOD-06-U01`).
    pub fn icon(self) -> &'static str {
        match self {
            Response::Accepted => "✔",
            Response::Declined => "✖",
            Response::Maybe => "❓",
            Response::Awaiting => "⏳",
        }
    }

    /// What the icon means, in words.
    pub fn label(self) -> &'static str {
        match self {
            Response::Accepted => "Accepted",
            Response::Declined => "Declined",
            Response::Maybe => "Maybe",
            Response::Awaiting => "Awaiting",
        }
    }
}

/// Someone who can be invited: from the contacts, the directory or an earlier event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    /// Empty when only the address is known.
    pub name: String,
    pub email: String,
}

impl Person {
    /// How a suggestion is shown: the name and the address, or the address alone.
    pub fn label(&self) -> String {
        if self.name.is_empty() {
            self.email.clone()
        } else {
            format!("{} <{}>", self.name, self.email)
        }
    }
}

/// Where the provider may look for people to suggest, as far as the account allowed it at
/// sign-in (`DEC-52`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reach {
    /// The account's contacts and the people it has e-mailed.
    pub contacts: bool,
    /// Its organization's directory, on Workspace accounts (`DEC-39`).
    pub directory: bool,
}

impl Reach {
    /// Where the account allowed the app to look when it signed in.
    pub fn of(account: &Account) -> Self {
        Reach {
            contacts: account.contacts,
            directory: account.directory,
        }
    }
}

/// The address typed, trimmed; `None` when it is not one: one `@` with something before it, and
/// after it a domain of dot-separated parts; no spaces.
pub fn address(typed: &str) -> Option<String> {
    let typed = typed.trim();
    let (local, domain) = typed.split_once('@')?;
    let parts: Vec<&str> = domain.split('.').collect();
    let valid = !local.is_empty()
        && parts.len() > 1
        && parts.iter().all(|part| !part.is_empty())
        && !typed.contains(char::is_whitespace)
        && !domain.contains('@');
    valid.then(|| typed.to_string())
}

/// Whether `query` starts the person's name, a word of it or their address, ignoring case.
fn matches(person: &Person, query: &str) -> bool {
    let query = query.to_lowercase();
    let name = person.name.to_lowercase();
    name.starts_with(&query)
        || name.split_whitespace().any(|word| word.starts_with(&query))
        || person.email.to_lowercase().starts_with(&query)
}

/// The guests of the account's events, but the account itself, each address once: those of the
/// latest events first.
pub fn recent(events: &[Event], account_id: &str) -> Vec<Person> {
    let mut invited: Vec<&Event> = (events.iter())
        .filter(|event| event.account_id == account_id && !event.about.guests.is_empty())
        .collect();
    invited.sort_by_key(|event| std::cmp::Reverse(begins(event)));
    let mut people: Vec<Person> = Vec::new();
    let mut seen = HashSet::new();
    for guest in invited.iter().flat_map(|event| &event.about.guests) {
        if !guest.own && seen.insert(guest.email.to_ascii_lowercase()) {
            people.push(Person {
                name: guest.name.clone(),
                email: guest.email.clone(),
            });
        }
    }
    people
}

/// When the event starts; an all-day one at midnight UTC of its first day.
fn begins(event: &Event) -> DateTime<Utc> {
    match event.timing {
        Timing::Timed { start, .. } => start,
        Timing::AllDay { start, .. } => start.and_time(chrono::NaiveTime::MIN).and_utc(),
    }
}

/// Whether two addresses are the same, ignoring case.
pub fn same(one: &str, other: &str) -> bool {
    one.eq_ignore_ascii_case(other)
}

/// Who is suggested for what was typed: the people found, in order, each address once, but those
/// already invited and the user's own addresses; at most `SUGGESTIONS`. Nothing before anything
/// is typed.
pub fn suggestions(
    query: &str,
    found: &[Person],
    invited: &[Guest],
    own: &[String],
) -> Vec<Person> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }
    let mut suggested: Vec<Person> = Vec::new();
    for person in found.iter().filter(|person| matches(person, query)) {
        let taken = (invited.iter().map(|guest| &guest.email))
            .chain(own)
            .chain(suggested.iter().map(|p| &p.email))
            .any(|email| same(email, &person.email));
        if !taken {
            suggested.push(person.clone());
        }
        if suggested.len() == SUGGESTIONS {
            break;
        }
    }
    suggested
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{d, timed};

    fn person(name: &str, email: &str) -> Person {
        Person {
            name: name.to_string(),
            email: email.to_string(),
        }
    }

    #[test]
    fn an_address_needs_one_at_sign_a_name_and_a_domain_with_a_dot() {
        assert_eq!(
            address(" adam@example.com "),
            Some("adam@example.com".to_string())
        );
        assert_eq!(
            address("a.b+c@mail.example.pl"),
            Some("a.b+c@mail.example.pl".to_string())
        );
        for wrong in [
            "",
            "adam",
            "adam@",
            "@example.com",
            "adam@example",
            "adam@example.",
            "adam@.com",
            "ad am@example.com",
            "adam@@example.com",
            "adam@ex@ample.com",
        ] {
            assert_eq!(address(wrong), None, "{wrong}");
        }
    }

    #[test]
    fn typing_the_start_of_a_name_a_word_of_it_or_an_address_suggests_the_person() {
        let found = [
            person("Adam Kowalski", "adam@example.com"),
            person("", "ewa@example.com"),
            person("Ola Nowak", "ola@example.com"),
        ];
        let names = |query: &str| -> Vec<String> {
            suggestions(query, &found, &[], &[])
                .into_iter()
                .map(|p| p.email)
                .collect()
        };

        assert_eq!(names("Ada"), ["adam@example.com"]);
        assert_eq!(names("kow"), ["adam@example.com"]);
        assert_eq!(names("EWA"), ["ewa@example.com"]);
        assert_eq!(names("nowak"), ["ola@example.com"]);
        assert!(names("example").is_empty());
        assert!(names("  ").is_empty());
    }

    #[test]
    fn suggestions_leave_out_guests_already_invited_the_users_own_addresses_and_repeats() {
        let found = [
            person("Adam Kowalski", "adam@example.com"),
            person("Adam K.", "ADAM@example.com"),
            person("Adam Owner", "owner@example.com"),
            person("Adam Nowak", "nowak@example.com"),
        ];
        let invited = [Guest::invited(&person("", "nowak@example.com"))];
        let own = ["owner@example.com".to_string()];

        let suggested = suggestions("adam", &found, &invited, &own);

        assert_eq!(suggested, [person("Adam Kowalski", "adam@example.com")]);
    }

    #[test]
    fn at_most_eight_people_are_suggested() {
        let found: Vec<Person> = (0..12)
            .map(|i| person(&format!("Anna {i}"), &format!("anna{i}@example.com")))
            .collect();
        assert_eq!(suggestions("anna", &found, &[], &[]).len(), SUGGESTIONS);
    }

    #[test]
    fn recent_guests_are_the_accounts_invited_addresses_latest_events_first_without_its_own() {
        let mut earlier = timed("Kickoff", d(2026, 11, 2), (9, 0), (10, 0));
        earlier.about.guests = vec![
            Guest::invited(&person("", "ewa@example.com")),
            Guest::invited(&person("Adam", "adam@example.com")),
        ];
        let mut later = timed("Review", d(2026, 11, 9), (9, 0), (10, 0));
        later.about.guests = vec![
            Guest {
                own: true,
                ..Guest::invited(&person("", "owner@example.com"))
            },
            Guest::invited(&person("", "Adam@example.com")),
        ];
        let mut elsewhere = timed("Other", d(2026, 11, 9), (9, 0), (10, 0));
        elsewhere.account_id = "2".to_string();
        elsewhere.about.guests = vec![Guest::invited(&person("", "jan@example.com"))];

        let recent = recent(&[earlier, later, elsewhere], "1");

        assert_eq!(
            recent,
            [
                person("", "Adam@example.com"),
                person("", "ewa@example.com"),
            ]
        );
    }
}
