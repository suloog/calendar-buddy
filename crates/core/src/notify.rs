//! Asking whether Google e-mails an event's guests (`US-06-03`, `DEC-40`): a change to an event
//! with guests, saved in the editor or by dragging, and its deletion wait for the user to say
//! once whether the guests get an update (`MOD-06-F12`). The answer goes to Google with every
//! request the change makes; an event without guests asks nothing.

use crate::editor::Edit;
use crate::guests::Guest;
use crate::model::{Calendar, Event};
use crate::series::{Question, Scope};

/// What the question says for a change.
pub const UPDATE: &str = "Send update e-mails to the guests?";

/// What the question says for a deletion.
pub const CANCELLATION: &str = "Send cancellation e-mails to the guests?";

/// The answers, as Google words them, and Back, which saves nothing.
pub const SEND: &str = "Send";
pub const DONT_SEND: &str = "Don't send";
pub const BACK: &str = "Back";

/// A change or deletion of an event, or of the events of its series the user chose, to save.
#[derive(Clone, Debug, PartialEq)]
pub enum Pending {
    Change(Box<Edit>, Scope),
    Delete(Box<(Calendar, Event)>, Scope),
    /// The user's answer to an invitation (`MOD-06-F11`).
    Respond(Box<Edit>, Scope),
}

impl Pending {
    /// The change or deletion the user chose the scope of.
    pub fn of(question: Question, scope: Scope) -> Pending {
        match question {
            Question::Change(edit) => Pending::Change(edit, scope),
            Question::Delete(deleted) => Pending::Delete(deleted, scope),
            Question::Respond(edit) => Pending::Respond(edit, scope),
        }
    }

    /// The question saving it asks first, when the event has guests other than the account
    /// itself, before or after the change; `None` when it is saved at once, as an answer always
    /// is.
    pub fn asks(&self) -> Option<&'static str> {
        match self {
            Pending::Change(edit, _) => (invites(&edit.event.about.guests)
                || edit.patch.guests.as_deref().is_some_and(invites))
            .then_some(UPDATE),
            Pending::Delete(deleted, _) => invites(&deleted.1.about.guests).then_some(CANCELLATION),
            Pending::Respond(..) => None,
        }
    }
}

/// Whether anyone but the account itself is invited.
fn invites(guests: &[Guest]) -> bool {
    guests.iter().any(|guest| !guest.own)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drag;
    use crate::editor::Patch;
    use crate::fixtures::{d, timed};
    use crate::guests::{Person, Response};
    use crate::model::{Access, Rgb};
    use chrono_tz::Europe::Warsaw;

    fn calendar() -> Calendar {
        Calendar {
            id: "personal".to_string(),
            account_id: "1".to_string(),
            name: "Personal".to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
            default_reminders: Vec::new(),
        }
    }

    fn guest(email: &str) -> Guest {
        Guest::invited(&Person {
            name: String::new(),
            email: email.to_string(),
        })
    }

    fn own() -> Guest {
        Guest {
            own: true,
            response: Response::Accepted,
            ..guest("me@example.com")
        }
    }

    /// "Sync", 12 November 10:00–10:30, with the guests.
    fn sync(guests: Vec<Guest>) -> Event {
        let mut sync = timed("Sync", d(2026, 11, 12), (10, 0), (10, 30));
        sync.about.guests = guests;
        sync
    }

    fn moved(event: &Event) -> Pending {
        let timing = timed("Sync", d(2026, 11, 12), (11, 0), (11, 30)).timing;
        Pending::Change(Box::new(drag::edit(event, calendar(), timing)), Scope::This)
    }

    #[test]
    fn a_change_or_deletion_of_an_event_with_guests_asks_whether_to_email_them() {
        let event = sync(vec![own(), guest("adam@example.com")]);
        let deleted = Pending::Delete(Box::new((calendar(), event.clone())), Scope::This);

        assert_eq!(moved(&event).asks(), Some(UPDATE));
        assert_eq!(deleted.asks(), Some(CANCELLATION));
    }

    #[test]
    fn an_event_without_guests_but_the_account_itself_asks_nothing() {
        for event in [sync(Vec::new()), sync(vec![own()])] {
            let deleted = Pending::Delete(Box::new((calendar(), event.clone())), Scope::All);

            assert_eq!(moved(&event).asks(), None);
            assert_eq!(deleted.asks(), None);
        }
    }

    #[test]
    fn inviting_the_first_guest_or_removing_the_last_one_asks_too() {
        let alone = sync(Vec::new());
        let invited = Edit {
            patch: Patch {
                guests: Some(vec![guest("adam@example.com")]),
                ..Patch::default()
            },
            ..drag::edit(&alone, calendar(), alone.timing)
        };
        let with_adam = sync(vec![guest("adam@example.com")]);
        let removed = Edit {
            patch: Patch {
                guests: Some(Vec::new()),
                ..Patch::default()
            },
            ..drag::edit(&with_adam, calendar(), with_adam.timing)
        };

        for edit in [invited, removed] {
            let pending = Pending::Change(Box::new(edit), Scope::This);
            assert_eq!(pending.asks(), Some(UPDATE));
        }
    }

    #[test]
    fn the_scope_chosen_for_an_occurrence_is_kept_for_the_question() {
        let event = sync(vec![guest("adam@example.com")]);
        let Pending::Change(edit, _) = moved(&event) else {
            panic!("a change");
        };
        let question = Question::Change(edit.clone());

        assert_eq!(
            Pending::of(question, Scope::Following),
            Pending::Change(edit, Scope::Following)
        );
        let question = Question::Delete(Box::new((calendar(), event.clone())));
        assert_eq!(
            Pending::of(question, Scope::All),
            Pending::Delete(Box::new((calendar(), event)), Scope::All)
        );
    }
}
