//! Changing and deleting an occurrence of a series (`US-05-07`): the three scopes Google Calendar
//! asks for, this event, this and following events, or all events (`MOD-05-F20`, `MOD-05-F21`),
//! and what each asks of Google. "This event" changes or deletes the occurrence alone; "all
//! events" changes or deletes the series' master; "this and following" ends the series before the
//! occurrence and, for a change, starts a new series there (MOD-05 *Technical notes*). A series'
//! rule is never rewritten but for its end, so one the presets cannot express keeps it
//! (`MOD-05-F29`).

use chrono::Duration;

use crate::create::NewEvent;
use crate::drag::{at, local};
use crate::editor::{Edit, Patch};
use crate::model::{About, Calendar, Event, Timing};
use crate::recurrence::{before, count_before, excluded, start_of};
use crate::sync::{Start, SyncedEvent};

/// What a change or deletion of an occurrence applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    This,
    Following,
    All,
}

impl Scope {
    /// The scopes, in the order Google Calendar asks them.
    pub const EACH: [Scope; 3] = [Scope::This, Scope::Following, Scope::All];

    /// What the question calls the scope, as Google Calendar does.
    pub fn label(self) -> &'static str {
        match self {
            Scope::This => "This event",
            Scope::Following => "This and following events",
            Scope::All => "All events",
        }
    }
}

/// Why the following or all events cannot move to another day, or between all-day and timed.
pub const OTHER_DAY: &str = "Moving the following or all events to another day changes how the \
                             series repeats; that is done in Google Calendar.";

/// Why an occurrence, or those following it, cannot move to another calendar.
pub const OTHER_CALENDAR: &str = "Only all events of a series can move to another calendar.";

/// The scopes a change of an occurrence can be saved with, in the order asked, each with why it
/// cannot be when it cannot (`MOD-05-F20`).
pub fn scopes(edit: &Edit) -> Vec<(Scope, Option<&'static str>)> {
    let other_day = match (&edit.event.series, &edit.patch.timing) {
        (Some((_, original)), Some(timing)) => !same_day(*original, &edit.event.timing, timing),
        _ => false,
    };
    let other_calendar = edit.to.is_some();
    Scope::EACH
        .map(|scope| {
            let why = match scope {
                Scope::This => other_calendar.then_some(OTHER_CALENDAR),
                _ if other_day => Some(OTHER_DAY),
                Scope::Following => other_calendar.then_some(OTHER_CALENDAR),
                Scope::All => None,
            };
            (scope, why)
        })
        .to_vec()
}

/// Whether `new` starts on the day the occurrence starts by the series' rule, `original`, and is
/// all-day as it is; `old` is the occurrence as it is, in its own zone.
fn same_day(original: Start, old: &Timing, new: &Timing) -> bool {
    match (original, old, new) {
        (
            Start::At(original),
            Timing::Timed { zone: old, .. },
            Timing::Timed { start, zone, .. },
        ) => local(original, *old).date() == local(*start, *zone).date(),
        (Start::On(original), _, Timing::AllDay { start, .. }) => original == *start,
        _ => false,
    }
}

/// A change or deletion of an occurrence of a series, waiting for the user to choose its scope.
#[derive(Clone, Debug, PartialEq)]
pub enum Question {
    Change(Box<Edit>),
    Delete(Box<(Calendar, Event)>),
    /// The user's answer to an occurrence of a recurring invitation (`MOD-06-F13`).
    Respond(Box<Edit>),
}

impl Question {
    /// The question a change asks before it is saved, for an occurrence of a series; the change
    /// itself, to save at once, for any other event.
    pub fn changing(edit: Edit) -> Result<Edit, Question> {
        match edit.event.series {
            Some(_) => Err(Question::Change(Box::new(edit))),
            None => Ok(edit),
        }
    }

    /// The question deleting `event` from `calendar` asks first, for an occurrence of a series
    /// (`MOD-05-F21`); the deletion itself, to make at once, for any other event.
    pub fn deleting(calendar: Calendar, event: Event) -> Result<(Calendar, Event), Question> {
        match event.series {
            Some(_) => Err(Question::Delete(Box::new((calendar, event)))),
            None => Ok((calendar, event)),
        }
    }

    /// The occurrence asked about.
    pub fn event(&self) -> &Event {
        match self {
            Question::Change(edit) | Question::Respond(edit) => &edit.event,
            Question::Delete(deleted) => &deleted.1,
        }
    }

    /// The scopes the user can choose, in the order asked, each with why it cannot be chosen
    /// when it cannot; a deletion can take any, and an answer this event or all events.
    pub fn scopes(&self) -> Vec<(Scope, Option<&'static str>)> {
        match self {
            Question::Change(edit) => scopes(edit),
            Question::Delete(_) => Scope::EACH.map(|scope| (scope, None)).to_vec(),
            Question::Respond(_) => vec![(Scope::This, None), (Scope::All, None)],
        }
    }
}

/// What Google is asked to do for a change or deletion of an occurrence.
#[derive(Clone, Debug, PartialEq)]
pub enum Plan {
    /// One event changed: the occurrence itself or the series' master.
    Change(Box<Edit>),
    /// One event deleted from the calendar: the occurrence itself or the series' master.
    Delete(Box<(Calendar, Event)>),
    /// The series ended before the occurrence and, for a change, a new one from it.
    Split(Box<Split>),
}

/// A series ended before one of its occurrences: its master's rule lines patched against its
/// version, and the new series from the occurrence on, if any, a copy of the series with the
/// fields `new` holds in place of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct Split {
    pub calendar: Calendar,
    pub id: String,
    pub etag: String,
    pub recurrence: Vec<String>,
    pub new: Option<NewEvent>,
    /// The occurrences from there on changed on their own, the one split at included, deleted
    /// with the old series' end: their IDs and versions.
    pub changed: Vec<(String, String)>,
}

/// What the change `edit` of an occurrence of the series `master` asks of Google for `scope`;
/// `exceptions` are the series' occurrences changed or deleted on their own. `None` when `edit`
/// is not of an occurrence of the series.
pub fn change(
    edit: &Edit,
    scope: Scope,
    master: &SyncedEvent,
    exceptions: &[SyncedEvent],
) -> Option<Plan> {
    let (original, first, scope) = of_series(&edit.event, scope, master)?;
    let patch = &edit.patch;
    Some(match scope {
        Scope::This => Plan::Change(Box::new(edit.clone())),
        Scope::All => Plan::Change(Box::new(Edit {
            event: master_event(&edit.event, master, first),
            calendar: edit.calendar.clone(),
            patch: Patch {
                timing: patch.timing.map(|timing| on_first_day(first, timing)),
                ..patch.clone()
            },
            to: edit.to.clone(),
            notify: edit.notify,
        })),
        Scope::Following => {
            let timing = patch.timing.unwrap_or_else(|| regular(first, original));
            let about = &master.about;
            let new = NewEvent {
                title: patch.title.clone().unwrap_or_else(|| master.title.clone()),
                timing,
                about: About {
                    description: (patch.description.clone())
                        .unwrap_or_else(|| about.description.clone()),
                    location: patch
                        .location
                        .clone()
                        .unwrap_or_else(|| about.location.clone()),
                    show: patch.show.unwrap_or(about.show),
                    visibility: patch.visibility.unwrap_or(about.visibility),
                    kind: about.kind,
                    color: patch.color.unwrap_or(about.color),
                    // Sent with the rest of the series by copying the event at Google.
                    attachments: about.attachments.clone(),
                    guests: (patch.guests.clone()).unwrap_or_else(|| about.guests.clone()),
                },
                recurrence: following(master, first, original, timing, exceptions),
            };
            let split = ended(&edit.calendar, master, original, Some(new), exceptions);
            Plan::Split(Box::new(split))
        }
    })
}

/// What deleting the occurrence `event` of the series `master` from `calendar` asks of Google
/// for `scope`, `exceptions` as `change` says. `None` when `event` is not an occurrence of the
/// series.
pub fn delete(
    calendar: &Calendar,
    event: &Event,
    scope: Scope,
    master: &SyncedEvent,
    exceptions: &[SyncedEvent],
) -> Option<Plan> {
    let (original, first, scope) = of_series(event, scope, master)?;
    Some(match scope {
        Scope::This => Plan::Delete(Box::new((calendar.clone(), event.clone()))),
        Scope::All => Plan::Delete(Box::new((
            calendar.clone(),
            master_event(event, master, first),
        ))),
        Scope::Following => Plan::Split(Box::new(ended(
            calendar, master, original, None, exceptions,
        ))),
    })
}

/// Where the occurrence starts by the series' rule, where the series starts, and the scope as
/// applied: "this and following" on the first occurrence is "all events" (MOD-05 *Constraints
/// and edge cases*). `None` when the event is not an occurrence of the series.
fn of_series(event: &Event, scope: Scope, master: &SyncedEvent) -> Option<(Start, Timing, Scope)> {
    let (series, original) = event.series.as_ref()?;
    let first = master.timing.filter(|_| *series == master.id)?;
    let scope = match scope {
        Scope::Following if *original == start_of(first) => Scope::All,
        scope => scope,
    };
    Some((*original, first, scope))
}

/// The series' master, as the occurrence `event` of it is shown, in the version Google keeps.
fn master_event(event: &Event, master: &SyncedEvent, first: Timing) -> Event {
    Event {
        id: master.id.clone(),
        title: master.title.clone(),
        timing: first,
        series: None,
        about: master.about.clone(),
        etag: master.etag.clone(),
        ..event.clone()
    }
}

/// The series' first occurrence moved as `new` moves one of its occurrences: on the same day, to
/// the times and zone of `new`; an all-day one as long as `new`.
fn on_first_day(first: Timing, new: Timing) -> Timing {
    match (first, new) {
        (
            Timing::Timed {
                start: day,
                zone: old,
                ..
            },
            Timing::Timed { start, end, zone },
        ) => {
            let from = local(start, zone);
            let start = at(local(day, old).date().and_time(from.time()), zone);
            let end = at(local(start, zone) + (local(end, zone) - from), zone);
            Timing::Timed { start, end, zone }
        }
        (
            Timing::AllDay { start: day, .. },
            Timing::AllDay {
                start,
                end_exclusive,
            },
        ) => Timing::AllDay {
            start: day,
            end_exclusive: day + (end_exclusive - start),
        },
        (_, new) => new,
    }
}

/// The occurrence starting at `original` by the series' rule, as long as the first.
fn regular(first: Timing, original: Start) -> Timing {
    match (first, original) {
        (Timing::Timed { start, end, zone }, Start::At(at)) => Timing::Timed {
            start: at,
            end: at + (end - start),
            zone,
        },
        (
            Timing::AllDay {
                start,
                end_exclusive,
            },
            Start::On(date),
        ) => Timing::AllDay {
            start: date,
            end_exclusive: date + (end_exclusive - start),
        },
        (first, _) => first,
    }
}

/// The series ended the second before `original`, or for an all-day one the day before, its
/// rules otherwise as they were; with the new series from there, if any, and its occurrences
/// changed on their own from there on going with the old series.
fn ended(
    calendar: &Calendar,
    master: &SyncedEvent,
    original: Start,
    new: Option<NewEvent>,
    exceptions: &[SyncedEvent],
) -> Split {
    let until = match original {
        Start::At(at) => (at - Duration::seconds(1))
            .format("%Y%m%dT%H%M%SZ")
            .to_string(),
        Start::On(date) => (date - Duration::days(1)).format("%Y%m%d").to_string(),
    };
    let recurrence = (master.recurrence.iter())
        .map(|line| match line.strip_prefix("RRULE:") {
            Some(parts) => {
                let parts: Vec<&str> = (parts.split(';'))
                    .filter(|part| !part.starts_with("COUNT=") && !part.starts_with("UNTIL="))
                    .collect();
                format!("RRULE:{};UNTIL={until}", parts.join(";"))
            }
            None => line.clone(),
        })
        .collect();
    Split {
        calendar: calendar.clone(),
        id: master.id.clone(),
        etag: master.etag.clone(),
        recurrence,
        new,
        changed: (exceptions.iter())
            .filter(|exception| exception.timing.is_some())
            .filter(|exception| from(exception, master, original).is_some())
            .map(|exception| (exception.id.clone(), exception.etag.clone()))
            .collect(),
    }
}

/// Where the occurrence `exception` of the series `master` starts by its rule, when it is
/// `original` or later.
fn from(exception: &SyncedEvent, master: &SyncedEvent, original: Start) -> Option<Start> {
    let (series, start) = exception.instance_of.as_ref()?;
    (*series == master.id && !before(*start, original)).then_some(*start)
}

/// The rules of the new series that starts at `timing` in place of the occurrence at `original`:
/// the series' rules with the occurrences before it taken off their count, and its deleted
/// occurrences from there on, among `exceptions` and those its `EXDATE` lines take away, still
/// deleted at the new time. Its `RDATE` lines stay with the old series.
fn following(
    master: &SyncedEvent,
    first: Timing,
    original: Start,
    timing: Timing,
    exceptions: &[SyncedEvent],
) -> Vec<String> {
    let before_it = count_before(first, &master.recurrence, original);
    let mut rules: Vec<String> = (master.recurrence.iter())
        .filter_map(|line| {
            let parts: Vec<String> = (line.strip_prefix("RRULE:")?.split(';'))
                .map(
                    |part| match part.strip_prefix("COUNT=").map(str::parse::<usize>) {
                        Some(Ok(count)) => {
                            format!("COUNT={}", count.saturating_sub(before_it).max(1))
                        }
                        _ => part.to_string(),
                    },
                )
                .collect();
            Some(format!("RRULE:{}", parts.join(";")))
        })
        .collect();
    let cancelled = (exceptions.iter())
        .filter(|exception| exception.timing.is_none())
        .filter_map(|exception| from(exception, master, original));
    let gone: Vec<String> = (excluded(first, &master.recurrence).into_iter())
        .filter(|start| !before(*start, original))
        .chain(cancelled)
        .map(|start| match (start, first, timing) {
            (
                Start::At(gone),
                Timing::Timed { zone: old, .. },
                Timing::Timed { start, zone, .. },
            ) => {
                let day = local(gone, old).date();
                let gone = at(day.and_time(local(start, zone).time()), zone);
                gone.format("%Y%m%dT%H%M%SZ").to_string()
            }
            (Start::At(gone), ..) => gone.format("%Y%m%dT%H%M%SZ").to_string(),
            (Start::On(date), ..) => date.format("%Y%m%d").to_string(),
        })
        .collect();
    if !gone.is_empty() {
        let kind = match timing {
            Timing::Timed { .. } => "EXDATE:",
            Timing::AllDay { .. } => "EXDATE;VALUE=DATE:",
        };
        rules.push(format!("{kind}{}", gone.join(",")));
    }
    rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colors::EventColor;
    use crate::fixtures::d;
    use crate::model::{Access, Rgb};
    use crate::recurrence::expand;
    use crate::sync::Window;
    use chrono::{DateTime, NaiveDate, TimeZone, Utc};
    use chrono_tz::Europe::{London, Warsaw};
    use chrono_tz::Tz;

    fn calendar() -> Calendar {
        Calendar {
            id: "personal@example.com".into(),
            account_id: "1".into(),
            name: "Personal".into(),
            color: Rgb(0x03, 0x9B, 0xE5),
            access: Access::Owner,
            zone: Warsaw,
        }
    }

    fn at_(zone: Tz, date: NaiveDate, (h, m): (u32, u32)) -> DateTime<Utc> {
        zone.from_local_datetime(&date.and_hms_opt(h, m, 0).unwrap())
            .unwrap()
            .with_timezone(&Utc)
    }

    fn timed(zone: Tz, date: NaiveDate, start: (u32, u32), end: (u32, u32)) -> Timing {
        Timing::Timed {
            start: at_(zone, date, start),
            end: at_(zone, date, end),
            zone,
        }
    }

    fn master(timing: Timing, rules: &[&str]) -> SyncedEvent {
        SyncedEvent {
            id: "standup".into(),
            title: "Standup".into(),
            timing: Some(timing),
            recurrence: rules.iter().map(|rule| rule.to_string()).collect(),
            instance_of: None,
            about: About {
                location: "Room 1".into(),
                ..About::default()
            },
            etag: "\"3\"".into(),
            organizer_only: false,
        }
    }

    /// `TC-05-08`'s series: weekly on Monday 09:00–09:15, 10 occurrences from 16 November.
    fn standup() -> SyncedEvent {
        master(
            timed(Warsaw, d(2026, 11, 16), (9, 0), (9, 15)),
            &["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=10"],
        )
    }

    fn window() -> Window {
        Window::around(d(2026, 11, 11), Warsaw)
    }

    /// The occurrence of the series on `date`, as the views show it.
    fn occurrence(series: &SyncedEvent, date: NaiveDate) -> Event {
        expand(&calendar(), std::slice::from_ref(series), window())
            .into_iter()
            .find(|event| match event.timing {
                Timing::Timed { start, .. } => start.with_timezone(&Warsaw).date_naive() == date,
                Timing::AllDay { start, .. } => start == date,
            })
            .unwrap()
    }

    fn edit(event: &Event, patch: Patch) -> Edit {
        Edit {
            event: event.clone(),
            calendar: calendar(),
            patch,
            to: None,
            notify: false,
        }
    }

    fn moved_to(event: &Event, timing: Timing) -> Edit {
        edit(
            event,
            Patch {
                timing: Some(timing),
                ..Patch::default()
            },
        )
    }

    fn retitled(event: &Event) -> Edit {
        edit(
            event,
            Patch {
                title: Some("Daily".into()),
                ..Patch::default()
            },
        )
    }

    /// The occurrence of the series on `date` as Google keeps it once changed on its own, at
    /// `timing`, or deleted with none.
    fn exception(series: &SyncedEvent, date: NaiveDate, timing: Option<Timing>) -> SyncedEvent {
        let shown = occurrence(series, date);
        SyncedEvent {
            id: shown.id,
            timing,
            recurrence: Vec::new(),
            instance_of: shown.series,
            etag: format!("\"{}\"", date.format("%d")),
            ..series.clone()
        }
    }

    fn split(plan: Option<Plan>) -> Split {
        match plan {
            Some(Plan::Split(split)) => *split,
            other => panic!("not a split: {other:?}"),
        }
    }

    fn changed(plan: Option<Plan>) -> Edit {
        match plan {
            Some(Plan::Change(edit)) => *edit,
            other => panic!("not a change: {other:?}"),
        }
    }

    /// Local start times, in Warsaw, of the series shown as the views show them.
    fn local_starts(events: &[SyncedEvent]) -> Vec<String> {
        let mut starts: Vec<String> = expand(&calendar(), events, window())
            .iter()
            .map(|event| match event.timing {
                Timing::Timed { start, .. } => {
                    start.with_timezone(&Warsaw).format("%F %R").to_string()
                }
                Timing::AllDay { start, .. } => start.to_string(),
            })
            .collect();
        starts.sort();
        starts
    }

    /// The series as Google keeps it once the split is made.
    fn after(old: &SyncedEvent, split: &Split) -> Vec<SyncedEvent> {
        let mut events = vec![SyncedEvent {
            recurrence: split.recurrence.clone(),
            ..old.clone()
        }];
        if let Some(new) = &split.new {
            events.push(SyncedEvent {
                id: "new".into(),
                title: new.title.clone(),
                timing: Some(new.timing),
                recurrence: new.recurrence.clone(),
                instance_of: None,
                about: new.about.clone(),
                etag: String::new(),
                organizer_only: false,
            });
        }
        events
    }

    #[test]
    fn changing_or_deleting_an_occurrence_asks_the_scope_first_and_other_events_ask_nothing() {
        let fifth = occurrence(&standup(), d(2026, 12, 14));
        let single = Event {
            series: None,
            ..fifth.clone()
        };

        let asked = Question::changing(retitled(&fifth)).unwrap_err();
        let deleting = Question::deleting(calendar(), fifth.clone()).unwrap_err();

        assert_eq!(asked, Question::Change(Box::new(retitled(&fifth))));
        assert_eq!(asked.scopes(), scopes(&retitled(&fifth)));
        assert_eq!(deleting.event(), &fifth);
        assert_eq!(
            deleting.scopes(),
            [
                (Scope::This, None),
                (Scope::Following, None),
                (Scope::All, None)
            ]
        );
        assert!(Question::changing(retitled(&single)).is_ok());
        assert!(Question::deleting(calendar(), single).is_ok());
    }

    #[test]
    fn a_change_of_its_times_or_fields_can_take_any_scope() {
        let fifth = occurrence(&standup(), d(2026, 12, 14));

        let later = moved_to(&fifth, timed(Warsaw, d(2026, 12, 14), (11, 0), (11, 15)));

        assert_eq!(
            scopes(&later),
            [
                (Scope::This, None),
                (Scope::Following, None),
                (Scope::All, None)
            ]
        );
        assert_eq!(scopes(&retitled(&fifth)), scopes(&later));
    }

    #[test]
    fn only_this_event_moves_to_another_day_or_between_all_day_and_timed() {
        let fifth = occurrence(&standup(), d(2026, 12, 14));
        let tuesday = moved_to(&fifth, timed(Warsaw, d(2026, 12, 15), (9, 0), (9, 15)));
        let all_day = moved_to(
            &fifth,
            Timing::AllDay {
                start: d(2026, 12, 14),
                end_exclusive: d(2026, 12, 15),
            },
        );

        for edit in [tuesday, all_day] {
            assert_eq!(
                scopes(&edit),
                [
                    (Scope::This, None),
                    (Scope::Following, Some(OTHER_DAY)),
                    (Scope::All, Some(OTHER_DAY))
                ]
            );
        }
    }

    #[test]
    fn only_all_events_move_to_another_calendar() {
        let fifth = occurrence(&standup(), d(2026, 12, 14));
        let mut moved = retitled(&fifth);
        moved.to = Some(Calendar {
            id: "work@example.com".into(),
            ..calendar()
        });

        assert_eq!(
            scopes(&moved),
            [
                (Scope::This, Some(OTHER_CALENDAR)),
                (Scope::Following, Some(OTHER_CALENDAR)),
                (Scope::All, None)
            ]
        );
    }

    /// `TC-05-08` step 2, in the series as Google is asked to keep it.
    #[test]
    fn tc_05_08_this_and_following_from_the_fifth_of_ten_moves_five_to_ten_only() {
        let old = standup();
        let fifth = occurrence(&old, d(2026, 12, 14));

        let split = split(change(
            &moved_to(&fifth, timed(Warsaw, d(2026, 12, 14), (11, 0), (11, 15))),
            Scope::Following,
            &old,
            &[],
        ));

        // The old series ends the second before the fifth: 09:00 in Warsaw is 08:00 UTC.
        assert_eq!(
            split.recurrence,
            ["RRULE:FREQ=WEEKLY;BYDAY=MO;UNTIL=20261214T075959Z"]
        );
        assert_eq!(
            (split.id.as_str(), split.etag.as_str()),
            ("standup", "\"3\"")
        );
        let new = split.new.clone().unwrap();
        assert_eq!(new.recurrence, ["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=6"]);
        assert_eq!(
            (new.title.as_str(), new.about.location.as_str()),
            ("Standup", "Room 1")
        );
        assert_eq!(
            local_starts(&after(&old, &split)),
            [
                "2026-11-16 09:00",
                "2026-11-23 09:00",
                "2026-11-30 09:00",
                "2026-12-07 09:00",
                "2026-12-14 11:00",
                "2026-12-21 11:00",
                "2026-12-28 11:00",
                "2027-01-04 11:00",
                "2027-01-11 11:00",
                "2027-01-18 11:00",
            ]
        );
    }

    #[test]
    fn this_and_following_on_the_first_occurrence_changes_all_events() {
        let old = standup();
        let first = occurrence(&old, d(2026, 11, 16));
        let later = moved_to(&first, timed(Warsaw, d(2026, 11, 16), (11, 0), (11, 15)));

        assert_eq!(
            change(&later, Scope::Following, &old, &[]),
            change(&later, Scope::All, &old, &[])
        );
        assert_eq!(
            delete(&calendar(), &first, Scope::Following, &old, &[]),
            delete(&calendar(), &first, Scope::All, &old, &[])
        );
    }

    #[test]
    fn all_events_change_the_series_times_from_its_first_day_against_its_version() {
        let old = standup();
        let fifth = occurrence(&old, d(2026, 12, 14));

        let edit = changed(change(
            &moved_to(&fifth, timed(Warsaw, d(2026, 12, 14), (11, 0), (12, 0))),
            Scope::All,
            &old,
            &[],
        ));

        assert_eq!(
            (edit.event.id.as_str(), edit.event.etag.as_str()),
            ("standup", "\"3\"")
        );
        let timing = timed(Warsaw, d(2026, 11, 16), (11, 0), (12, 0));
        assert_eq!(edit.patch.timing, Some(timing));
        assert_eq!(edit.patch.recurrence, None);
        let series = SyncedEvent {
            timing: Some(timing),
            ..old
        };
        assert_eq!(
            local_starts(&[series])[..2],
            ["2026-11-16 11:00", "2026-11-23 11:00"]
        );
    }

    #[test]
    fn all_events_in_another_zone_keep_the_first_day_on_the_new_clock() {
        let old = standup();
        let fifth = occurrence(&old, d(2026, 12, 14));

        let edit = changed(change(
            &moved_to(&fifth, timed(London, d(2026, 12, 14), (8, 0), (8, 30))),
            Scope::All,
            &old,
            &[],
        ));

        assert_eq!(
            edit.patch.timing,
            Some(timed(London, d(2026, 11, 16), (8, 0), (8, 30)))
        );
    }

    #[test]
    fn all_events_of_an_all_day_series_take_the_new_length_from_the_first_day() {
        let old = master(
            Timing::AllDay {
                start: d(2026, 11, 16),
                end_exclusive: d(2026, 11, 17),
            },
            &["RRULE:FREQ=WEEKLY"],
        );
        let fifth = occurrence(&old, d(2026, 12, 14));
        let longer = Timing::AllDay {
            start: d(2026, 12, 14),
            end_exclusive: d(2026, 12, 16),
        };

        let edit = changed(change(&moved_to(&fifth, longer), Scope::All, &old, &[]));

        assert_eq!(
            edit.patch.timing,
            Some(Timing::AllDay {
                start: d(2026, 11, 16),
                end_exclusive: d(2026, 11, 18),
            })
        );
    }

    #[test]
    fn this_and_following_keeps_the_series_colour_or_takes_the_one_picked() {
        let mut old = standup();
        old.about.color = Some(EventColor::Sage);
        let fifth = occurrence(&old, d(2026, 12, 14));

        let kept = split(change(&retitled(&fifth), Scope::Following, &old, &[]));
        let recoloured = edit(
            &fifth,
            Patch {
                color: Some(Some(EventColor::Grape)),
                ..Patch::default()
            },
        );
        let picked = split(change(&recoloured, Scope::Following, &old, &[]));

        assert_eq!(kept.new.unwrap().about.color, Some(EventColor::Sage));
        assert_eq!(picked.new.unwrap().about.color, Some(EventColor::Grape));
    }

    #[test]
    fn this_event_changes_the_occurrence_alone() {
        let old = standup();
        let fifth = occurrence(&old, d(2026, 12, 14));
        let edit = retitled(&fifth);

        assert_eq!(
            change(&edit, Scope::This, &old, &[]),
            Some(Plan::Change(Box::new(edit)))
        );
        assert_eq!(
            delete(&calendar(), &fifth, Scope::This, &old, &[]),
            Some(Plan::Delete(Box::new((calendar(), fifth))))
        );
    }

    /// `MOD-05-F29`: a series of every second Tuesday of the month, which no preset expresses.
    #[test]
    fn a_rule_the_presets_cannot_express_is_kept_when_other_fields_change() {
        let old = master(
            timed(Warsaw, d(2026, 11, 10), (18, 0), (19, 0)),
            &["RRULE:FREQ=MONTHLY;BYDAY=2TU;COUNT=12"],
        );
        let third = occurrence(&old, d(2027, 1, 12));

        let notified = Edit {
            notify: true,
            ..retitled(&third)
        };
        let all = changed(change(&notified, Scope::All, &old, &[]));
        let split = split(change(&retitled(&third), Scope::Following, &old, &[]));

        // The guests are e-mailed of the series' change as the user answered (`MOD-06-F12`).
        assert!(all.notify);
        assert_eq!(all.patch.recurrence, None);
        assert_eq!(all.patch.title.as_deref(), Some("Daily"));
        assert_eq!(
            split.recurrence,
            ["RRULE:FREQ=MONTHLY;BYDAY=2TU;UNTIL=20270112T165959Z"]
        );
        let new = split.new.clone().unwrap();
        assert_eq!(new.recurrence, ["RRULE:FREQ=MONTHLY;BYDAY=2TU;COUNT=10"]);
        assert_eq!(new.title, "Daily");
        assert_eq!(new.timing, timed(Warsaw, d(2027, 1, 12), (18, 0), (19, 0)));
        assert_eq!(local_starts(&after(&old, &split)).len(), 12);
    }

    #[test]
    fn a_series_ending_on_a_date_or_never_keeps_that_end_after_the_split() {
        for (rule, new_rule) in [
            (
                "RRULE:FREQ=WEEKLY;BYDAY=MO;UNTIL=20270201T225959Z",
                "RRULE:FREQ=WEEKLY;BYDAY=MO;UNTIL=20270201T225959Z",
            ),
            ("RRULE:FREQ=WEEKLY;BYDAY=MO", "RRULE:FREQ=WEEKLY;BYDAY=MO"),
        ] {
            let old = master(timed(Warsaw, d(2026, 11, 16), (9, 0), (9, 15)), &[rule]);
            let fifth = occurrence(&old, d(2026, 12, 14));

            let split = split(change(&retitled(&fifth), Scope::Following, &old, &[]));

            assert_eq!(
                split.recurrence,
                ["RRULE:FREQ=WEEKLY;BYDAY=MO;UNTIL=20261214T075959Z"]
            );
            assert_eq!(split.new.unwrap().recurrence, [new_rule]);
        }
    }

    #[test]
    fn occurrences_deleted_after_the_split_stay_deleted_at_the_new_time() {
        let old = master(
            timed(Warsaw, d(2026, 11, 16), (9, 0), (9, 15)),
            &[
                "RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=10",
                "EXDATE;TZID=Europe/Warsaw:20261123T090000,20261228T090000",
            ],
        );
        let fifth = occurrence(&old, d(2026, 12, 14));
        let cancelled = [exception(&old, d(2027, 1, 4), None)];

        let split = split(change(
            &moved_to(&fifth, timed(Warsaw, d(2026, 12, 14), (11, 0), (11, 15))),
            Scope::Following,
            &old,
            &cancelled,
        ));

        let new = split.new.clone().unwrap();
        assert_eq!(
            new.recurrence,
            [
                "RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=6",
                "EXDATE:20261228T100000Z,20270104T100000Z"
            ]
        );
        assert_eq!(
            local_starts(&after(&old, &split)),
            [
                "2026-11-16 09:00",
                "2026-11-30 09:00",
                "2026-12-07 09:00",
                "2026-12-14 11:00",
                "2026-12-21 11:00",
                "2027-01-11 11:00",
                "2027-01-18 11:00",
            ]
        );
    }

    #[test]
    fn an_all_day_series_ends_the_day_before_the_split() {
        let old = master(
            Timing::AllDay {
                start: d(2026, 11, 16),
                end_exclusive: d(2026, 11, 17),
            },
            &["RRULE:FREQ=WEEKLY;COUNT=10"],
        );
        let fifth = occurrence(&old, d(2026, 12, 14));

        let split = split(change(&retitled(&fifth), Scope::Following, &old, &[]));

        assert_eq!(split.recurrence, ["RRULE:FREQ=WEEKLY;UNTIL=20261213"]);
        let new = split.new.unwrap();
        assert_eq!(new.recurrence, ["RRULE:FREQ=WEEKLY;COUNT=6"]);
        assert_eq!(
            new.timing,
            Timing::AllDay {
                start: d(2026, 12, 14),
                end_exclusive: d(2026, 12, 15),
            }
        );
    }

    #[test]
    fn a_changed_occurrence_splits_the_series_where_its_rule_starts_it() {
        let old = standup();
        let mut moved = occurrence(&old, d(2026, 12, 14));
        moved.timing = timed(Warsaw, d(2026, 12, 14), (15, 0), (15, 15));
        moved.etag = "\"5\"".into();

        let split = split(change(&retitled(&moved), Scope::Following, &old, &[]));

        assert_eq!(
            split.recurrence,
            ["RRULE:FREQ=WEEKLY;BYDAY=MO;UNTIL=20261214T075959Z"]
        );
        assert_eq!(
            split.new.unwrap().timing,
            timed(Warsaw, d(2026, 12, 14), (9, 0), (9, 15))
        );
    }

    #[test]
    fn occurrences_changed_on_their_own_from_the_split_on_go_with_the_old_series() {
        let old = standup();
        let moved = |date: NaiveDate| {
            let timing = timed(Warsaw, date, (15, 0), (15, 15));
            exception(&old, date, Some(timing))
        };
        let exceptions = [
            moved(d(2026, 11, 30)),
            moved(d(2026, 12, 14)),
            moved(d(2027, 1, 4)),
            exception(&old, d(2027, 1, 11), None),
        ];
        let mut fifth = occurrence(&old, d(2026, 12, 14));
        fifth.etag = exceptions[1].etag.clone();

        let changed = split(change(
            &retitled(&fifth),
            Scope::Following,
            &old,
            &exceptions,
        ));
        let deleted = split(delete(
            &calendar(),
            &fifth,
            Scope::Following,
            &old,
            &exceptions,
        ));

        let ids = [
            ("standup_20261214T080000Z".to_string(), "\"14\"".to_string()),
            ("standup_20270104T080000Z".to_string(), "\"04\"".to_string()),
        ];
        assert_eq!(changed.changed, ids);
        assert_eq!(deleted.changed, ids);
    }

    #[test]
    fn deleting_following_events_ends_the_series_and_all_events_deletes_it() {
        let old = standup();
        let fifth = occurrence(&old, d(2026, 12, 14));

        let split = split(delete(&calendar(), &fifth, Scope::Following, &old, &[]));
        let all = delete(&calendar(), &fifth, Scope::All, &old, &[]);

        assert_eq!(
            split.recurrence,
            ["RRULE:FREQ=WEEKLY;BYDAY=MO;UNTIL=20261214T075959Z"]
        );
        assert_eq!(split.new, None);
        assert_eq!(
            local_starts(&after(&old, &split)),
            [
                "2026-11-16 09:00",
                "2026-11-23 09:00",
                "2026-11-30 09:00",
                "2026-12-07 09:00"
            ]
        );
        let Some(Plan::Delete(deleted)) = all else {
            panic!("not a deletion: {all:?}");
        };
        let (calendar, master) = *deleted;
        assert_eq!(calendar.id, "personal@example.com");
        assert_eq!(
            (master.id.as_str(), master.etag.as_str()),
            ("standup", "\"3\"")
        );
    }

    #[test]
    fn an_event_of_another_series_is_not_planned() {
        let fifth = occurrence(&standup(), d(2026, 12, 14));
        let other = SyncedEvent {
            id: "other".into(),
            ..standup()
        };

        assert_eq!(change(&retitled(&fifth), Scope::All, &other, &[]), None);
        assert_eq!(delete(&calendar(), &fifth, Scope::All, &other, &[]), None);
    }
}
