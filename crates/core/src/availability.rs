//! A preview of the guests' availability in the event editor (`US-06-08`): for the day the event
//! starts, one strip per guest with their busy times, as their calendars share them, and the
//! chosen time highlighted across all strips; a guest busy then is marked as a conflict
//! (`MOD-06-F19`, `U02`). A guest who does not share their availability shows "no data", and
//! without network there is no preview (`MOD-06-F20`).

use chrono::{DateTime, Utc};

use crate::editor::Editor;
use crate::guests::{self, Guest, Response};
use crate::model::{Event, Show, Timing};
use crate::regional::Regional;
use crate::week::day_span;

/// How many guests the preview shows at most: the first ones listed (`DEC-63`).
pub const SHOWN: usize = 20;

/// What a strip says of a guest who does not share their availability.
pub const NO_DATA: &str = "No data";

/// What a strip says of a guest busy at the chosen time.
pub const CONFLICT: &str = "Conflict";

/// A time from its start to its end.
pub type Span = (DateTime<Utc>, DateTime<Utc>);

/// One guest's busy times, as their calendar shares them; `None` when it does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Availability {
    pub email: String,
    pub busy: Option<Vec<Span>>,
}

/// What the provider is asked: the guests' busy times on a day, with the account the event is
/// saved with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asked {
    pub account_id: String,
    pub day: Span,
    pub emails: Vec<String>,
}

/// What the editor asks for its preview, and the time chosen in it; `None` when there is no
/// preview: offline, without a calendar or guests, for an all-day event, for out of office and
/// focus time, which take no guests, and while the time cannot be read. The day is the one the
/// event starts on in the editor's time zone.
pub fn asked(editor: &Editor, regional: &Regional, online: bool) -> Option<(Asked, Span)> {
    let calendar = (editor.calendar.as_ref()).filter(|_| online && !editor.kind.timed_only())?;
    let Ok(Timing::Timed { start, end, zone }) = editor.timing(regional) else {
        return None;
    };
    let emails: Vec<String> = (listed(&editor.guests).take(SHOWN))
        .map(|guest| guest.email.clone())
        .collect();
    if emails.is_empty() {
        return None;
    }
    let asked = Asked {
        account_id: calendar.account_id.clone(),
        day: day_span(start.with_timezone(&zone).date_naive(), zone),
        emails,
    };
    Some((asked, (start, end)))
}

/// One guest's strip: the parts of the day they are busy, from 0 at its start to 1 at its end,
/// or `None` for "no data"; and whether they are busy at the chosen time.
#[derive(Clone, Debug, PartialEq)]
pub struct Strip {
    /// Their name, or their address when they have none.
    pub label: String,
    pub busy: Option<Vec<(f32, f32)>>,
    pub conflict: bool,
}

/// The strips of the guests asked for, in the order listed, and the chosen time across them, as
/// parts of the day.
#[derive(Clone, Debug, PartialEq)]
pub struct Preview {
    pub strips: Vec<Strip>,
    pub chosen: (f32, f32),
}

/// The preview of what the provider answered for `asked`, with the time `chosen` in the editor.
/// For an existing event, as `saved`, the time its guests are busy with the event itself is not
/// counted as busy (`DEC-135`).
pub fn preview(
    asked: &Asked,
    chosen: Span,
    guests: &[Guest],
    saved: Option<&Event>,
    answer: &[Availability],
) -> Preview {
    let (from, until) = asked.day;
    let part = |at: DateTime<Utc>| {
        let at = at.clamp(from, until);
        ((at - from).num_seconds() as f64 / (until - from).num_seconds() as f64) as f32
    };
    let strips = (asked.emails.iter())
        .map(|email| {
            let busy = (answer.iter())
                .find(|shared| guests::same(&shared.email, email))
                .and_then(|shared| shared.busy.as_ref())
                .map(|busy| without(busy, saved.and_then(|event| busy_with(event, email))));
            let label = (listed(guests).find(|guest| guests::same(&guest.email, email)))
                .map(|guest| guest.name.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| email.clone());
            Strip {
                label,
                conflict: (busy.iter().flatten())
                    .any(|&(start, end)| start < chosen.1 && chosen.0 < end),
                busy: busy.map(|busy| {
                    (busy.into_iter())
                        .map(|(start, end)| (part(start), part(end)))
                        .filter(|(start, end)| start < end)
                        .collect()
                }),
            }
        })
        .collect();
    Preview {
        strips,
        chosen: (part(chosen.0), part(chosen.1)),
    }
}

/// The guests the editor lists: all but the account itself.
fn listed(guests: &[Guest]) -> impl Iterator<Item = &Guest> {
    guests.iter().filter(|guest| !guest.own)
}

/// When the guest is busy with the saved event, as free/busy counts it: a timed event shown as
/// busy, to which they are invited and have not declined.
fn busy_with(event: &Event, email: &str) -> Option<Span> {
    let Timing::Timed { start, end, .. } = event.timing else {
        return None;
    };
    let invited = (event.about.guests.iter())
        .any(|guest| guests::same(&guest.email, email) && guest.response != Response::Declined);
    (event.about.show == Show::Busy && invited).then_some((start, end))
}

/// The busy times without the time `cut`.
fn without(busy: &[Span], cut: Option<Span>) -> Vec<Span> {
    let Some((cut_from, cut_until)) = cut else {
        return busy.to_vec();
    };
    (busy.iter())
        .flat_map(|&(start, end)| {
            if end <= cut_from || cut_until <= start {
                return vec![(start, end)];
            }
            [(start, cut_from), (cut_until, end)]
                .into_iter()
                .filter(|(start, end)| start < end)
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create::{QuickEntry, selected};
    use crate::fixtures::{d, timed};
    use crate::guests::Person;
    use crate::model::{Access, Calendar, Kind, Rgb};
    use chrono::TimeZone;
    use chrono_tz::Europe::Warsaw;
    use pure_rust_locales::Locale;

    fn regional() -> Regional {
        Regional::new(Locale::pl_PL)
    }

    /// A new event on Friday 13 November 2026, 14:30–15:00 in Warsaw.
    fn editor() -> Editor {
        let calendar = Calendar {
            id: "owner@example.com".to_string(),
            account_id: "1".to_string(),
            name: "Owner".to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
            default_reminders: Vec::new(),
        };
        let entry = QuickEntry::new(
            selected(d(2026, 11, 13), 870, 900, Warsaw),
            Warsaw,
            Some(calendar),
        );
        Editor::from_quick(&entry, &regional())
    }

    fn person(name: &str, email: &str) -> Person {
        Person {
            name: name.to_string(),
            email: email.to_string(),
        }
    }

    /// 13 November 2026 at the hour and minute, in Warsaw.
    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(2026, 11, 13, hour, minute, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn with_guests(people: &[Person]) -> Editor {
        let mut editor = editor();
        for person in people {
            editor.add_guest(person, &[]);
        }
        editor
    }

    #[test]
    fn the_guests_are_asked_for_the_day_the_event_starts_in_its_time_zone() {
        let editor = with_guests(&[
            person("Anna Nowak", "anna@example.com"),
            person("", "adam@example.com"),
        ]);

        let (asked, chosen) = asked(&editor, &regional(), true).unwrap();

        assert_eq!(asked.account_id, "1");
        assert_eq!(asked.day, (at(0, 0), at(0, 0) + chrono::Duration::days(1)));
        assert_eq!(asked.emails, ["anna@example.com", "adam@example.com"]);
        assert_eq!(chosen, (at(14, 30), at(15, 0)));
    }

    #[test]
    fn offline_without_guests_and_for_an_all_day_event_there_is_no_preview() {
        let editor = with_guests(&[person("", "adam@example.com")]);
        assert!(asked(&editor, &regional(), false).is_none());
        assert!(asked(&with_guests(&[]), &regional(), true).is_none());
        let mut all_day = editor;
        all_day.all_day = true;
        assert!(asked(&all_day, &regional(), true).is_none());
    }

    #[test]
    fn the_account_itself_is_not_a_strip() {
        let mut editor = with_guests(&[person("", "adam@example.com")]);
        editor.guests.push(Guest {
            own: true,
            ..Guest::invited(&person("", "owner@example.com"))
        });

        let (asked, _) = asked(&editor, &regional(), true).unwrap();

        assert_eq!(asked.emails, ["adam@example.com"]);
    }

    #[test]
    fn of_many_guests_the_first_20_are_asked_for() {
        let people: Vec<Person> = (1..=25)
            .map(|n| person("", &format!("guest{n}@example.com")))
            .collect();
        let editor = with_guests(&people);

        let (asked, _) = asked(&editor, &regional(), true).unwrap();

        assert_eq!(asked.emails.len(), SHOWN);
        assert_eq!(asked.emails[0], "guest1@example.com");
        assert_eq!(asked.emails[19], "guest20@example.com");
    }

    #[test]
    fn each_guest_gets_a_strip_of_their_busy_times_and_one_busy_then_is_a_conflict() {
        let editor = with_guests(&[
            person("Anna Nowak", "anna@example.com"),
            person("", "adam@example.com"),
        ]);
        let (asked, chosen) = asked(&editor, &regional(), true).unwrap();
        let answer = [
            Availability {
                email: "anna@example.com".to_string(),
                busy: Some(vec![(at(14, 0), at(15, 0))]),
            },
            Availability {
                email: "adam@example.com".to_string(),
                busy: Some(vec![(at(9, 0), at(10, 0))]),
            },
        ];

        let preview = preview(&asked, chosen, &editor.guests, None, &answer);

        assert_eq!(preview.chosen, (14.5 / 24.0, 15.0 / 24.0));
        assert_eq!(
            preview.strips,
            [
                Strip {
                    label: "Anna Nowak".to_string(),
                    busy: Some(vec![(14.0 / 24.0, 15.0 / 24.0)]),
                    conflict: true,
                },
                Strip {
                    label: "adam@example.com".to_string(),
                    busy: Some(vec![(9.0 / 24.0, 10.0 / 24.0)]),
                    conflict: false,
                },
            ]
        );
    }

    #[test]
    fn a_guest_who_does_not_share_or_is_not_answered_for_has_no_data_and_no_conflict() {
        let editor = with_guests(&[
            person("", "private@example.com"),
            person("", "unanswered@example.com"),
        ]);
        let (asked, chosen) = asked(&editor, &regional(), true).unwrap();
        let answer = [Availability {
            email: "private@example.com".to_string(),
            busy: None,
        }];

        let preview = preview(&asked, chosen, &editor.guests, None, &answer);

        assert!(
            (preview.strips.iter()).all(|strip| strip.busy.is_none() && !strip.conflict),
            "{preview:?}"
        );
    }

    #[test]
    fn busy_times_touching_the_chosen_time_are_no_conflict_and_those_beyond_the_day_are_cut() {
        let editor = with_guests(&[person("", "adam@example.com")]);
        let (asked, chosen) = asked(&editor, &regional(), true).unwrap();
        let answer = [Availability {
            email: "ADAM@example.com".to_string(),
            busy: Some(vec![
                (at(13, 0), at(14, 30)),
                (at(15, 0), at(16, 0)),
                (at(23, 0), at(23, 0) + chrono::Duration::hours(2)),
            ]),
        }];

        let preview = preview(&asked, chosen, &editor.guests, None, &answer);

        let [strip] = preview.strips.as_slice() else {
            panic!("one strip");
        };
        assert!(!strip.conflict);
        assert_eq!(strip.busy.as_ref().unwrap()[2], (23.0 / 24.0, 1.0));
    }

    #[test]
    fn the_time_an_existing_event_was_saved_at_is_not_counted_as_busy() {
        let editor = with_guests(&[person("", "adam@example.com")]);
        let (asked, chosen) = asked(&editor, &regional(), true).unwrap();
        // Adam is busy 14:00–14:30 with another event, then with the event itself.
        let answer = [Availability {
            email: "adam@example.com".to_string(),
            busy: Some(vec![(at(14, 0), at(15, 0))]),
        }];

        let mut saved = timed("saved", d(2026, 11, 13), (14, 30), (15, 0));
        saved.about.guests = editor.guests.clone();

        let unchanged = preview(&asked, chosen, &editor.guests, Some(&saved), &answer);
        let moved = preview(
            &asked,
            (at(14, 0), at(14, 30)),
            &editor.guests,
            Some(&saved),
            &answer,
        );

        assert!(!unchanged.strips[0].conflict);
        assert_eq!(
            unchanged.strips[0].busy,
            Some(vec![(14.0 / 24.0, 14.5 / 24.0)])
        );
        assert!(moved.strips[0].conflict);
    }

    #[test]
    fn a_saved_event_shown_as_free_or_declined_or_not_inviting_the_guest_is_not_cut() {
        let editor = with_guests(&[person("", "adam@example.com")]);
        let (asked, chosen) = asked(&editor, &regional(), true).unwrap();
        let answer = [Availability {
            email: "adam@example.com".to_string(),
            busy: Some(vec![(at(14, 30), at(15, 0))]),
        }];
        let invited = |show, response| {
            let mut saved = timed("saved", d(2026, 11, 13), (14, 30), (15, 0));
            saved.about.show = show;
            saved.about.guests = vec![Guest {
                response,
                ..Guest::invited(&person("", "adam@example.com"))
            }];
            saved
        };
        let free = invited(Show::Free, Response::Accepted);
        let declined = invited(Show::Busy, Response::Declined);
        let mut without_adam = invited(Show::Busy, Response::Accepted);
        without_adam.about.guests.clear();

        for saved in [free, declined, without_adam] {
            let preview = preview(&asked, chosen, &editor.guests, Some(&saved), &answer);
            assert!(preview.strips[0].conflict, "{saved:?}");
        }
    }

    #[test]
    fn out_of_office_and_focus_time_have_no_preview() {
        let mut editor = with_guests(&[person("", "adam@example.com")]);
        editor.kind = Kind::OutOfOffice;
        assert!(asked(&editor, &regional(), true).is_none());
    }
}
