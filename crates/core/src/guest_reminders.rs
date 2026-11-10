//! The user's own reminders of an invitation only its organizer may change (`US-07-07`,
//! `MOD-07-F21`, `DEC-221`): Google keeps each guest's reminders apart and takes a change of them
//! alone from a guest, so its details offer them, as the editor does (`MOD-07-F04`, `F05`), while
//! the rest of the event stays the organizer's (`MOD-05-F17`, `MOD-06-F15`).

use crate::editor::{Edit, Patch, ReminderField, checked, own_fields};
use crate::model::{Calendar, Event, Method, Protected, Reminders};
use crate::series::Question;

/// The reminders of the invitation, as the user changes them in its details.
#[derive(Clone, Debug, PartialEq)]
pub struct GuestReminders {
    pub event: Event,
    pub calendar: Calendar,
    /// Its own reminders as changed; `None` while it uses its calendar's defaults.
    pub reminders: Option<Vec<ReminderField>>,
}

impl GuestReminders {
    /// How its reminders can reach the user: a guest's copy is always Google's.
    pub const METHODS: &[Method] = &[Method::Notification, Method::Email];

    /// The reminders of `event` of `calendar` to change, when it is an invitation only its
    /// organizer may change; `None` for any other event: one the user may change has the editor,
    /// and one of a calendar the account may only read takes no change at all (`DEC-22`).
    pub fn of(event: &Event, calendar: &Calendar) -> Option<Self> {
        (event.protected() == Some(Protected::OrganizerOnly) && !event.busy_only).then(|| {
            GuestReminders {
                event: event.clone(),
                calendar: calendar.clone(),
                reminders: own_fields(&event.about.reminders),
            }
        })
    }

    /// The reminders shown: its own, or its calendar's defaults.
    pub fn fields(&self) -> Vec<ReminderField> {
        match &self.reminders {
            Some(own) => own.clone(),
            None => (self.calendar.default_reminders.iter().copied())
                .map(ReminderField::of)
                .collect(),
        }
    }

    /// Whether they differ from the event's, so there is something to save: reminders that cannot
    /// be saved count as changed, to show why.
    pub fn changed(&self) -> bool {
        let reminders = match &self.reminders {
            Some(own) => checked(own).map(Reminders::Own),
            None => Ok(Reminders::Default),
        };
        reminders.as_ref() != Ok(&self.event.about.reminders)
    }

    /// The change of the reminders alone, which e-mails no one; for an occurrence of a series,
    /// the question whether it applies to this event or to all events. Why they cannot be saved
    /// otherwise.
    pub fn save(&self) -> Result<Result<Edit, Question>, String> {
        let reminders = match &self.reminders {
            Some(own) => Reminders::Own(checked(own)?),
            None => Reminders::Default,
        };
        Ok(Question::changing(Edit {
            event: self.event.clone(),
            calendar: self.calendar.clone(),
            patch: Patch {
                reminders: Some(reminders),
                ..Patch::default()
            },
            to: None,
            notify: false,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{MOST_REMINDERS, Unit};
    use crate::fixtures::{d, timed};
    use crate::model::{Access, Reminder, Rgb, Timing};
    use crate::series::Scope;
    use crate::sync::Start;
    use crate::texts::TOO_MANY_REMINDERS;
    use chrono_tz::Europe::Warsaw;

    fn work() -> Calendar {
        Calendar {
            id: "work".to_string(),
            account_id: "1".to_string(),
            name: "Work".to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
            default_reminders: vec![Reminder {
                method: Method::Notification,
                minutes: 10,
            }],
        }
    }

    /// "Review", someone else's event the user is invited to, which only its organizer may
    /// change.
    fn review() -> Event {
        Event {
            organizer_only: true,
            ..timed("Review", d(2026, 11, 12), (10, 0), (11, 0))
        }
    }

    fn half_hour() -> ReminderField {
        ReminderField {
            method: Method::Notification,
            amount: 30,
            unit: Unit::Minutes,
        }
    }

    #[test]
    fn tc_07_10_an_invitation_only_its_organizer_may_change_offers_its_reminders_and_saves_them_alone()
     {
        let mut reminders = GuestReminders::of(&review(), &work()).expect("offered");
        assert_eq!(
            reminders.fields(),
            [ReminderField {
                method: Method::Notification,
                amount: 10,
                unit: Unit::Minutes,
            }]
        );
        assert!(!reminders.changed());

        reminders.reminders = Some(vec![half_hour()]);
        assert!(reminders.changed());
        let Ok(Ok(edit)) = reminders.save() else {
            panic!("saved at once");
        };
        let own = Reminders::Own(vec![Reminder {
            method: Method::Notification,
            minutes: 30,
        }]);
        assert_eq!(
            edit.patch,
            Patch {
                reminders: Some(own.clone()),
                ..Patch::default()
            }
        );
        assert!(!edit.notify);
        assert_eq!(edit.changed().about.reminders, own);

        // Back to the calendar's defaults.
        reminders.reminders = None;
        let Ok(Ok(edit)) = reminders.save() else {
            panic!("saved at once");
        };
        assert_eq!(edit.patch.reminders, Some(Reminders::Default));
    }

    #[test]
    fn the_same_reminder_written_in_other_units_is_no_change() {
        let hour = Event {
            about: crate::model::About {
                reminders: Reminders::Own(vec![Reminder {
                    method: Method::Notification,
                    minutes: 60,
                }]),
                ..review().about
            },
            ..review()
        };
        let mut reminders = GuestReminders::of(&hour, &work()).expect("offered");
        reminders.reminders = Some(vec![ReminderField {
            amount: 60,
            ..half_hour()
        }]);
        assert!(!reminders.changed());

        // One that cannot be saved shows Save, with why.
        reminders.reminders = Some(vec![half_hour(); MOST_REMINDERS + 1]);
        assert!(reminders.changed());
    }

    #[test]
    fn tc_07_10_a_read_only_calendar_and_an_event_the_user_may_change_offer_no_reminders_here() {
        let read_only = Event {
            read_only: true,
            ..review()
        };
        assert_eq!(GuestReminders::of(&read_only, &work()), None);
        let own = timed("Plan", d(2026, 11, 12), (10, 0), (11, 0));
        assert_eq!(GuestReminders::of(&own, &work()), None);
        let busy = Event {
            busy_only: true,
            ..review()
        };
        assert_eq!(GuestReminders::of(&busy, &work()), None);
    }

    #[test]
    fn the_reminders_of_an_invitation_keep_googles_limits() {
        let mut reminders = GuestReminders::of(&review(), &work()).unwrap();
        reminders.reminders = Some(vec![half_hour(); MOST_REMINDERS + 1]);
        assert_eq!(reminders.save(), Err(TOO_MANY_REMINDERS.to_string()));
    }

    #[test]
    fn an_occurrence_of_a_recurring_invitation_asks_this_event_or_all_events() {
        let Timing::Timed { start, .. } = review().timing else {
            unreachable!("a timed event");
        };
        let occurrence = Event {
            series: Some(("Review".to_string(), Start::At(start))),
            ..review()
        };
        let mut reminders = GuestReminders::of(&occurrence, &work()).unwrap();
        reminders.reminders = Some(vec![half_hour()]);
        let Ok(Err(question)) = reminders.save() else {
            panic!("asks first");
        };
        let scopes: Vec<Scope> = question.scopes().into_iter().map(|(s, _)| s).collect();
        assert_eq!(scopes, [Scope::This, Scope::All]);
    }
}
