//! Undoing the last change to an event (`US-05-04`, `DEC-59`): a deletion, a move, a resize or
//! an edit Google took can be undone with Ctrl+Z or with the Undo of the notice that follows it,
//! shown for 5 seconds (`MOD-05-F27`, `MOD-05-F28`, `MOD-05-U04`). An undo is a change sent to
//! Google like any other: the deleted event is restored, the changed one patched back against
//! its version, so one changed elsewhere meanwhile is refused. Only the last change is kept; there
//! is no history.

use std::time::{Duration, Instant};

use crate::editor::{Edit, Patch, own_title};
use crate::model::{Calendar, Event};

/// How long the notice with Undo stays (`MOD-05-U04`).
pub const NOTICE: Duration = Duration::from_secs(5);

/// What the notice says after a deletion.
pub const DELETED: &str = "Event deleted.";

/// What the notice says after a change.
pub const SAVED: &str = "Event saved.";

/// What undoes a change Google took.
#[derive(Clone, Debug, PartialEq)]
pub enum Undo {
    /// Brings back the event deleted from the calendar, e-mailing its guests or not as the user
    /// answered for the deletion (`MOD-06-F12`).
    Restore(Box<(Calendar, Event)>, bool),
    /// Puts the changed event back as it was.
    Revert(Box<Edit>),
}

impl Undo {
    /// The change that puts the event back as it was before `edit`, made to the version `etag`
    /// Google gave it: the fields changed get their old values, and a moved event moves back.
    /// One only moved gets its title patched as it is, so that it too is refused when the event
    /// changed elsewhere meanwhile; a move alone is not sent against a version.
    pub fn revert(edit: &Edit, etag: &str) -> Undo {
        let (before, patch) = (&edit.event, &edit.patch);
        let mut event = edit.changed();
        event.etag = etag.to_string();
        let title = match &patch.title {
            Some(_) => Some(own_title(before).to_string()),
            None if patch.is_empty() => Some(own_title(&event).to_string()),
            None => None,
        };
        Undo::Revert(Box::new(Edit {
            calendar: edit.to.clone().unwrap_or_else(|| edit.calendar.clone()),
            patch: Patch {
                title,
                description: (patch.description.as_ref()).map(|_| before.about.description.clone()),
                location: patch
                    .location
                    .as_ref()
                    .map(|_| before.about.location.clone()),
                timing: patch.timing.map(|_| before.timing),
                show: patch.show.map(|_| before.about.show),
                visibility: patch.visibility.map(|_| before.about.visibility),
                recurrence: None,
                color: patch.color.map(|_| before.about.color),
                guests: (patch.guests.as_ref()).map(|_| before.about.guests.clone()),
                response: None,
            },
            to: edit.to.as_ref().map(|_| edit.calendar.clone()),
            // As the user answered for the change it undoes (`MOD-06-F12`).
            notify: edit.notify,
            event,
        }))
    }
}

/// The user's change sent to Google and not answered yet.
#[derive(Clone, Debug)]
enum Sent {
    Change(Box<Edit>),
    Delete(Box<(Calendar, Event)>, bool),
}

/// The last change to an event and what undoes it.
#[derive(Clone, Debug, Default)]
pub struct LastChange {
    /// The user's last change, by the number it is told by, until Google answers.
    sent: Option<(u64, Sent)>,
    /// What undoes the last change Google took, what the notice says and when it took it.
    undo: Option<(Undo, &'static str, Instant)>,
    /// The number of the undo under way.
    undoing: Option<u64>,
}

impl LastChange {
    /// The user changed the event, the change told by `local`; it becomes the last change.
    pub fn changing(&mut self, local: u64, edit: &Edit) {
        self.sent = Some((local, Sent::Change(Box::new(edit.clone()))));
        self.undo = None;
    }

    /// The user deleted the event, told by `local`, its guests e-mailed or not as `notify`
    /// says; it becomes the last change.
    pub fn deleting(&mut self, local: u64, calendar: &Calendar, event: &Event, notify: bool) {
        let deleted = Box::new((calendar.clone(), event.clone()));
        self.sent = Some((local, Sent::Delete(deleted, notify)));
        self.undo = None;
    }

    /// The user created an event: an earlier change is no longer the last one.
    pub fn creating(&mut self) {
        self.sent = None;
        self.undo = None;
    }

    /// Google took the change told by `local`, the event's version now `etag`: it can be undone.
    pub fn changed(&mut self, local: u64, etag: &str, now: Instant) {
        if let Some((_, Sent::Change(edit))) = self.sent.take_if(|(sent, _)| *sent == local) {
            self.undo = Some((Undo::revert(&edit, etag), SAVED, now));
        }
    }

    /// Google deleted the event told by `local`: the deletion can be undone.
    pub fn deleted(&mut self, local: u64, now: Instant) {
        if let Some((_, Sent::Delete(deleted, notify))) =
            self.sent.take_if(|(sent, _)| *sent == local)
        {
            self.undo = Some((Undo::Restore(deleted, notify), DELETED, now));
        }
    }

    /// Google refused the change told by `local`: there is nothing to undo.
    pub fn refused(&mut self, local: u64) {
        self.sent.take_if(|(sent, _)| *sent == local);
    }

    /// What the notice that follows the last change says, while it is shown, and how long it
    /// stays still.
    pub fn notice(&self, now: Instant) -> Option<(&'static str, Duration)> {
        let (_, says, at) = self.undo.as_ref()?;
        let left = NOTICE.checked_sub(now.saturating_duration_since(*at))?;
        (!left.is_zero()).then_some((*says, left))
    }

    /// Ctrl+Z or the notice's Undo: what undoes the last change, once.
    pub fn undo(&mut self) -> Option<Undo> {
        self.undo.take().map(|(undo, ..)| undo)
    }

    /// The undo goes to Google, told by `local`.
    pub fn undoing(&mut self, local: u64) {
        self.undoing = Some(local);
    }

    /// Whether `local` tells an undo.
    pub fn is_undo(&self, local: u64) -> bool {
        self.undoing == Some(local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colors::EventColor;
    use crate::create::NO_TITLE;
    use crate::fixtures::{d, timed};
    use crate::guests::{Guest, Person};
    use crate::model::{Access, Rgb, Show, Timing};
    use chrono_tz::Europe::Warsaw;

    fn calendar(id: &str) -> Calendar {
        Calendar {
            id: id.to_string(),
            account_id: "1".to_string(),
            name: id.to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
        }
    }

    fn call() -> Event {
        Event {
            etag: "\"1\"".to_string(),
            calendar_id: "personal".to_string(),
            ..timed("Call", d(2026, 11, 12), (10, 0), (11, 0))
        }
    }

    fn moved(event: &Event) -> Edit {
        Edit {
            event: event.clone(),
            calendar: calendar("personal"),
            patch: Patch {
                timing: Some(timed("Call", d(2026, 11, 12), (14, 0), (15, 0)).timing),
                ..Patch::default()
            },
            to: None,
            notify: false,
        }
    }

    #[test]
    fn undoing_a_change_patches_back_only_what_it_changed_against_the_new_version() {
        let call = call();
        let edit = moved(&call);

        let Undo::Revert(undo) = Undo::revert(&edit, "\"2\"") else {
            panic!("a change is reverted");
        };

        assert_eq!(undo.event.etag, "\"2\"");
        assert_eq!(undo.event.timing, edit.patch.timing.unwrap());
        assert_eq!(
            undo.patch,
            Patch {
                timing: Some(call.timing),
                ..Patch::default()
            }
        );
        assert_eq!(undo.calendar, calendar("personal"));
        assert_eq!(undo.to, None);
        assert_eq!(undo.changed().timing, call.timing);
    }

    #[test]
    fn undoing_an_edit_restores_every_field_changed_and_moves_the_event_back() {
        let call = Event {
            title: NO_TITLE.to_string(),
            ..call()
        };
        let edit = Edit {
            event: call.clone(),
            calendar: calendar("personal"),
            patch: Patch {
                title: Some("Call".to_string()),
                description: Some("Agenda".to_string()),
                location: Some("Room 1".to_string()),
                timing: Some(Timing::AllDay {
                    start: d(2026, 11, 12),
                    end_exclusive: d(2026, 11, 13),
                }),
                show: Some(Show::Free),
                visibility: Some(crate::model::Visibility::Private),
                recurrence: None,
                color: Some(Some(EventColor::Tomato)),
                guests: Some(vec![Guest::invited(&Person {
                    name: String::new(),
                    email: "adam@example.com".to_string(),
                })]),
                response: None,
            },
            to: Some(calendar("team")),
            notify: true,
        };

        let Undo::Revert(undo) = Undo::revert(&edit, "\"2\"") else {
            panic!("a change is reverted");
        };

        // The guests are e-mailed of the undo as they were of the change (`MOD-06-F12`).
        assert!(undo.notify);
        // An untitled event gets its empty title back, not "(No title)".
        assert_eq!(
            undo.patch,
            Patch {
                title: Some(String::new()),
                description: Some(String::new()),
                location: Some(String::new()),
                timing: Some(call.timing),
                show: Some(Show::Busy),
                visibility: Some(crate::model::Visibility::Default),
                recurrence: None,
                color: Some(None),
                guests: Some(Vec::new()),
                response: None,
            }
        );
        assert_eq!(undo.calendar, calendar("team"));
        assert_eq!(undo.event.calendar_id, "team");
        assert_eq!(undo.to, Some(calendar("personal")));
        let back = undo.changed();
        assert_eq!(
            (back.title.as_str(), back.calendar_id.as_str()),
            (NO_TITLE, "personal")
        );
    }

    #[test]
    fn undoing_a_move_alone_is_sent_against_the_version_with_the_title_as_it_is() {
        let call = call();
        let edit = Edit {
            event: call.clone(),
            calendar: calendar("personal"),
            patch: Patch::default(),
            to: Some(calendar("team")),
            notify: false,
        };

        let Undo::Revert(undo) = Undo::revert(&edit, "\"2\"") else {
            panic!("a change is reverted");
        };

        assert_eq!(
            undo.patch,
            Patch {
                title: Some("Call".to_string()),
                ..Patch::default()
            }
        );
        assert_eq!(undo.to, Some(calendar("personal")));
    }

    #[test]
    fn a_deletion_google_took_can_be_undone_once_with_a_notice_for_5_seconds() {
        let (gym, personal) = (
            timed("Gym", d(2026, 11, 12), (18, 0), (19, 0)),
            calendar("p"),
        );
        let mut last = LastChange::default();
        let start = Instant::now();
        last.deleting(3, &personal, &gym, true);
        // Nothing to undo until Google deleted it.
        assert_eq!(last.notice(start), None);

        last.deleted(3, start);

        assert_eq!(last.notice(start), Some((DELETED, NOTICE)));
        let later = start + Duration::from_millis(4900);
        assert_eq!(
            last.notice(later),
            Some((DELETED, Duration::from_millis(100)))
        );
        assert_eq!(last.notice(start + NOTICE), None);
        // Ctrl+Z still undoes it after the notice went.
        // E-mailing the guests as the deletion did (`MOD-06-F12`).
        assert_eq!(
            last.undo(),
            Some(Undo::Restore(Box::new((personal, gym)), true))
        );
        assert_eq!(last.undo(), None);
        assert_eq!(last.notice(start), None);
    }

    #[test]
    fn a_change_google_took_is_the_last_change_until_another_is_made() {
        let call = call();
        let edit = moved(&call);
        let mut last = LastChange::default();
        let now = Instant::now();
        last.changing(1, &edit);
        last.changed(1, "\"2\"", now);
        assert_eq!(last.notice(now).map(|(says, _)| says), Some(SAVED));

        // A new event replaces it as the last change.
        last.creating();
        assert_eq!(last.undo(), None);

        last.changing(2, &edit);
        last.changed(2, "\"3\"", now);
        // Another change sent: until Google takes it, nothing is undone.
        last.deleting(4, &calendar("personal"), &call, false);
        assert_eq!(last.undo(), None);
        // A refused change leaves nothing to undo.
        last.refused(4);
        last.deleted(4, now);
        assert_eq!(last.undo(), None);
    }

    #[test]
    fn an_undo_confirmed_is_not_a_change_to_undo_again() {
        let call = call();
        let mut last = LastChange::default();
        let now = Instant::now();
        last.changing(1, &moved(&call));
        last.changed(1, "\"2\"", now);
        assert!(last.undo().is_some());
        last.undoing(2);
        assert!(last.is_undo(2));
        assert!(!last.is_undo(1));

        last.changed(2, "\"3\"", now);

        assert_eq!(last.notice(now), None);
        assert_eq!(last.undo(), None);
    }
}
