//! The full event editor (`US-05-02`), for a new event and an existing one: the title,
//! description, start, end, time zone, all-day, location, busy or free, visibility and calendar
//! (`MOD-05-F04`…`F08`). Opened from the quick-entry box, it keeps what was typed there
//! (`MOD-05-F03`); opened on an existing event, it shows it in its own time zone and saves only
//! what changed, as a patch against the version opened (`MOD-05-F11`), moving it to another
//! calendar of the same account when another is picked (`MOD-05-F15`).

use chrono::{Duration, NaiveDate, NaiveDateTime, Timelike};
use chrono_tz::Tz;

use crate::create::{NO_CALENDAR, NO_TITLE, NewEvent, QuickEntry, wall};
use crate::model::{About, Calendar, Event, Show, Timing, Visibility};
use crate::regional::Regional;

/// What the editor says when the end is set before the start.
pub const ENDS_BEFORE_STARTS: &str = "The event ends before it starts.";

/// The times an all-day event gets when it is made a timed one.
const ALL_DAY_TIMES: (u32, u32) = (9, 10);

/// The fields of the full editor; dates and times as typed, in the user's formats.
#[derive(Clone, Debug, PartialEq)]
pub struct Editor {
    /// The event changed and its calendar, as opened; `None` for a new event.
    pub opened: Option<(Event, Calendar)>,
    pub title: String,
    pub description: String,
    pub location: String,
    pub all_day: bool,
    pub start_date: String,
    pub start_time: String,
    /// For an all-day event, its last day.
    pub end_date: String,
    pub end_time: String,
    /// The zone the times are in: the event's own (`MOD-03-F18`).
    pub zone: Tz,
    /// `None` when no calendar can take events.
    pub calendar: Option<Calendar>,
    pub show: Show,
    pub visibility: Visibility,
    /// The start as last read from the fields, which the end follows.
    start: Option<NaiveDateTime>,
}

/// What saving the editor does.
#[derive(Clone, Debug, PartialEq)]
pub enum Save {
    Create(Calendar, NewEvent),
    Change(Box<Edit>),
}

/// The fields of an event to change; `None` stays as it is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Patch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub timing: Option<Timing>,
    pub show: Option<Show>,
    pub visibility: Option<Visibility>,
}

impl Patch {
    pub fn is_empty(&self) -> bool {
        *self == Patch::default()
    }
}

/// A change to an existing event.
#[derive(Clone, Debug, PartialEq)]
pub struct Edit {
    /// The event as opened: its ID and the version the change is made to.
    pub event: Event,
    pub calendar: Calendar,
    pub patch: Patch,
    /// Another calendar of the same account it moves to.
    pub to: Option<Calendar>,
}

impl Edit {
    /// Whether nothing is changed.
    pub fn is_empty(&self) -> bool {
        self.patch.is_empty() && self.to.is_none()
    }

    /// The event as it is once changed, for the views to show while it is saved.
    pub fn changed(&self) -> Event {
        let mut event = self.event.clone();
        let patch = self.patch.clone();
        if let Some(title) = patch.title {
            event.title = if title.is_empty() {
                NO_TITLE.to_string()
            } else {
                title
            };
        }
        if let Some(timing) = patch.timing {
            event.timing = timing;
        }
        if let Some(description) = patch.description {
            event.about.description = description;
        }
        if let Some(location) = patch.location {
            event.about.location = location;
        }
        if let Some(show) = patch.show {
            event.about.show = show;
        }
        if let Some(visibility) = patch.visibility {
            event.about.visibility = visibility;
        }
        if let Some(to) = &self.to {
            event.calendar_id = to.id.clone();
            event.color = to.color;
        }
        event
    }
}

/// The text of a title as the editor shows it: an untitled event has an empty one.
fn own_title(event: &Event) -> &str {
    if event.title == NO_TITLE {
        ""
    } else {
        &event.title
    }
}

impl Editor {
    fn with(
        regional: &Regional,
        (start, end): (NaiveDateTime, NaiveDateTime),
        zone: Tz,
        calendar: Option<Calendar>,
    ) -> Self {
        Editor {
            opened: None,
            title: String::new(),
            description: String::new(),
            location: String::new(),
            all_day: false,
            start_date: regional.numeric_date(start.date()),
            start_time: regional.time(start.time()),
            end_date: regional.numeric_date(end.date()),
            end_time: regional.time(end.time()),
            zone,
            calendar,
            show: Show::default(),
            visibility: Visibility::default(),
            start: Some(start),
        }
    }

    /// "More options" in the quick-entry box: the new event with what was typed there
    /// (`MOD-05-F03`).
    pub fn from_quick(entry: &QuickEntry, regional: &Regional) -> Self {
        let local = |at: chrono::DateTime<chrono::Utc>| at.with_timezone(&entry.zone).naive_local();
        Editor {
            title: entry.title.clone(),
            ..Editor::with(
                regional,
                (local(entry.start), local(entry.end)),
                entry.zone,
                entry.calendar.clone(),
            )
        }
    }

    /// An existing event of `calendar`, in its own time zone.
    pub fn of(event: &Event, calendar: Calendar, regional: &Regional) -> Self {
        let (times, zone, all_day) = match event.timing {
            Timing::Timed { start, end, zone } => (
                (
                    start.with_timezone(&zone).naive_local(),
                    end.with_timezone(&zone).naive_local(),
                ),
                zone,
                false,
            ),
            Timing::AllDay {
                start,
                end_exclusive,
            } => {
                let last = (end_exclusive - Duration::days(1)).max(start);
                let at = |date: NaiveDate, hour| date.and_hms_opt(hour, 0, 0).expect("an hour");
                (
                    (at(start, ALL_DAY_TIMES.0), at(last, ALL_DAY_TIMES.1)),
                    calendar.zone,
                    true,
                )
            }
        };
        Editor {
            title: own_title(event).to_string(),
            description: event.about.description.clone(),
            location: event.about.location.clone(),
            all_day,
            show: event.about.show,
            visibility: event.about.visibility,
            opened: Some((event.clone(), calendar.clone())),
            ..Editor::with(regional, times, zone, Some(calendar))
        }
    }

    /// The calendars the event can be saved to, under their accounts: any of `choices` for a new
    /// event, those of its own account for an existing one (`MOD-05-F15`).
    pub fn choices(&self, choices: Vec<(String, Vec<Calendar>)>) -> Vec<(String, Vec<Calendar>)> {
        match &self.opened {
            None => choices,
            Some((event, _)) => choices
                .into_iter()
                .filter(|(_, calendars)| calendars.iter().any(|c| c.account_id == event.account_id))
                .collect(),
        }
    }

    fn start_at(&self, regional: &Regional) -> Option<NaiveDateTime> {
        let date = regional.parse_date(&self.start_date)?;
        Some(date.and_time(regional.parse_time(&self.start_time)?))
    }

    fn end_at(&self, regional: &Regional) -> Option<NaiveDateTime> {
        let date = regional.parse_date(&self.end_date)?;
        Some(date.and_time(regional.parse_time(&self.end_time)?))
    }

    /// After the start fields changed: the end moves with the start, so the event keeps its
    /// length, as in Google Calendar.
    pub fn follow_start(&mut self, regional: &Regional) {
        let Some(start) = self.start_at(regional) else {
            return;
        };
        let before = self.start.replace(start);
        if let (Some(before), Some(end)) = (before, self.end_at(regional))
            && before != start
        {
            let end = end + (start - before);
            self.end_date = regional.numeric_date(end.date());
            self.end_time = regional.time(end.time());
        }
    }

    /// When the event happens, as the fields say; why it cannot be saved otherwise.
    pub fn timing(&self, regional: &Regional) -> Result<Timing, String> {
        let unread = |field: &str| format!("The {field} can't be read.");
        let date = |text: &str, field| regional.parse_date(text).ok_or_else(|| unread(field));
        let time = |text: &str, field| regional.parse_time(text).ok_or_else(|| unread(field));
        let first = date(&self.start_date, "start date")?;
        let last = date(&self.end_date, "end date")?;
        if self.all_day {
            if last < first {
                return Err(ENDS_BEFORE_STARTS.to_string());
            }
            return Ok(Timing::AllDay {
                start: first,
                end_exclusive: last + Duration::days(1),
            });
        }
        let start = instant(
            first.and_time(time(&self.start_time, "start time")?),
            self.zone,
        );
        let end = instant(last.and_time(time(&self.end_time, "end time")?), self.zone);
        if end < start {
            return Err(ENDS_BEFORE_STARTS.to_string());
        }
        Ok(Timing::Timed {
            start,
            end,
            zone: self.zone,
        })
    }

    /// The fields that say when the event happens.
    fn times(&self) -> (bool, Tz, [&str; 4]) {
        let fields = [
            &self.start_date,
            &self.start_time,
            &self.end_date,
            &self.end_time,
        ];
        (self.all_day, self.zone, fields.map(String::as_str))
    }

    /// What saving does; why the event cannot be saved otherwise.
    pub fn save(&self, regional: &Regional) -> Result<Save, String> {
        let timing = self.timing(regional)?;
        let calendar = self.calendar.clone().ok_or(NO_CALENDAR)?;
        let title = self.title.trim().to_string();
        let about = About {
            description: self.description.clone(),
            location: self.location.trim().to_string(),
            show: self.show,
            visibility: self.visibility,
        };
        let Some((event, opened_in)) = &self.opened else {
            return Ok(Save::Create(
                calendar,
                NewEvent {
                    title,
                    timing,
                    about,
                },
            ));
        };
        // Times left as opened keep the event's own: the fields hold neither seconds nor which
        // of two times a clock shows twice at a daylight-saving change.
        let opened = Editor::of(event, opened_in.clone(), regional);
        let timing = (self.times() != opened.times()).then_some(timing);
        let patch = Patch {
            title: typed(own_title(event), &self.title, title),
            description: changed(event.about.description.clone(), about.description),
            location: typed(&event.about.location, &self.location, about.location),
            timing: timing.and_then(|timing| changed(event.timing, timing)),
            show: changed(event.about.show, about.show),
            visibility: changed(event.about.visibility, about.visibility),
        };
        let moved = calendar.account_id != opened_in.account_id || calendar.id != opened_in.id;
        Ok(Save::Change(Box::new(Edit {
            event: event.clone(),
            calendar: opened_in.clone(),
            patch,
            to: moved.then_some(calendar),
        })))
    }
}

/// The value saved from a field that was `before` and now holds `field`: `after`, when the field
/// was changed to something else.
fn typed(before: &str, field: &str, after: String) -> Option<String> {
    (field != before && after != before).then_some(after)
}

/// `after` when it differs from `before`.
fn changed<T: PartialEq>(before: T, after: T) -> Option<T> {
    (before != after).then_some(after)
}

/// The instant of a time on the wall clock in `zone`; one the zone skips moves to the first that
/// exists after it.
fn instant(at: NaiveDateTime, zone: Tz) -> chrono::DateTime<chrono::Utc> {
    wall(at.date(), at.time().num_seconds_from_midnight() / 60, zone)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create::selected;
    use crate::fixtures::{all_day, d, timed, timed_in};
    use crate::model::{Access, Rgb};
    use chrono::{DateTime, Utc};
    use chrono_tz::America::New_York;
    use chrono_tz::Europe::Warsaw;
    use pure_rust_locales::Locale;

    fn regional() -> Regional {
        Regional::new(Locale::pl_PL)
    }

    fn calendar(account: &str, id: &str) -> Calendar {
        Calendar {
            id: id.to_string(),
            account_id: account.to_string(),
            name: id.to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
        }
    }

    fn at(zone: Tz, date: NaiveDate, hour: u32, minute: u32) -> DateTime<Utc> {
        instant(date.and_hms_opt(hour, minute, 0).unwrap(), zone)
    }

    fn quick() -> QuickEntry {
        QuickEntry {
            title: "Plan".to_string(),
            ..QuickEntry::new(
                selected(d(2026, 11, 12), 600, 690, Warsaw),
                Warsaw,
                Some(calendar("1", "a@example.com")),
            )
        }
    }

    #[test]
    fn more_options_opens_the_editor_with_what_was_typed_in_the_quick_entry_box() {
        let editor = Editor::from_quick(&quick(), &regional());
        assert_eq!(editor.title, "Plan");
        assert_eq!(
            (
                editor.start_date.as_str(),
                editor.start_time.as_str(),
                editor.end_date.as_str(),
                editor.end_time.as_str()
            ),
            ("12.11.2026", "10:00", "12.11.2026", "11:30")
        );
        assert_eq!(editor.zone, Warsaw);
        assert_eq!(editor.calendar.unwrap().id, "a@example.com");
        assert!(!editor.all_day);
    }

    #[test]
    fn a_new_event_is_saved_with_every_field_set_in_the_editor() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.title = " Review ".to_string();
        editor.description = "Q4 numbers".to_string();
        editor.location = "Room 2".to_string();
        editor.start_date = "13.11.2026".to_string();
        editor.start_time = "14:00".to_string();
        editor.end_date = "13.11.2026".to_string();
        editor.end_time = "15:00".to_string();
        editor.show = Show::Free;
        editor.visibility = Visibility::Private;

        let Ok(Save::Create(calendar, event)) = editor.save(&regional()) else {
            panic!("a new event");
        };
        assert_eq!(calendar.id, "a@example.com");
        assert_eq!(
            event,
            NewEvent {
                title: "Review".to_string(),
                timing: Timing::Timed {
                    start: at(Warsaw, d(2026, 11, 13), 14, 0),
                    end: at(Warsaw, d(2026, 11, 13), 15, 0),
                    zone: Warsaw,
                },
                about: About {
                    description: "Q4 numbers".to_string(),
                    location: "Room 2".to_string(),
                    show: Show::Free,
                    visibility: Visibility::Private,
                },
            }
        );
    }

    #[test]
    fn the_times_are_taken_in_the_zone_chosen_in_the_editor() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.zone = New_York;
        let timing = editor.timing(&regional()).unwrap();
        assert_eq!(
            timing,
            Timing::Timed {
                start: at(New_York, d(2026, 11, 12), 10, 0),
                end: at(New_York, d(2026, 11, 12), 11, 30),
                zone: New_York,
            }
        );
    }

    #[test]
    fn an_all_day_event_can_span_several_days() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.all_day = true;
        editor.end_date = "14.11.2026".to_string();
        assert_eq!(
            editor.timing(&regional()),
            Ok(Timing::AllDay {
                start: d(2026, 11, 12),
                end_exclusive: d(2026, 11, 15),
            })
        );
        // A timed event can span days too.
        editor.all_day = false;
        let Ok(Timing::Timed { start, end, .. }) = editor.timing(&regional()) else {
            panic!("a timed event");
        };
        assert_eq!(end - start, Duration::hours(49) + Duration::minutes(30));
    }

    #[test]
    fn an_event_ending_before_it_starts_cannot_be_saved_and_the_editor_says_why() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.end_time = "9:00".to_string();
        assert_eq!(
            editor.save(&regional()),
            Err(ENDS_BEFORE_STARTS.to_string())
        );
        editor.all_day = true;
        editor.end_date = "11.11.2026".to_string();
        assert_eq!(
            editor.save(&regional()),
            Err(ENDS_BEFORE_STARTS.to_string())
        );
        // Ending when it starts is allowed, as in Google Calendar.
        editor.all_day = false;
        editor.end_date = "12.11.2026".to_string();
        editor.end_time = "10:00".to_string();
        assert!(editor.save(&regional()).is_ok());
    }

    #[test]
    fn a_date_or_time_that_cannot_be_read_is_named() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.start_time = "noon".to_string();
        assert_eq!(
            editor.save(&regional()),
            Err("The start time can't be read.".to_string())
        );
        editor.start_time = "10:00".to_string();
        editor.end_date = "31.11.2026".to_string();
        assert_eq!(
            editor.save(&regional()),
            Err("The end date can't be read.".to_string())
        );
        // An all-day event has no times to read.
        editor.end_date = "12.11.2026".to_string();
        editor.all_day = true;
        editor.end_time = String::new();
        assert!(editor.save(&regional()).is_ok());
    }

    #[test]
    fn a_time_skipped_by_daylight_saving_moves_to_the_first_that_exists() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        // 29 March 2026: Warsaw's clocks skip from 2:00 to 3:00.
        editor.start_date = "29.03.2026".to_string();
        editor.start_time = "2:30".to_string();
        editor.end_date = "29.03.2026".to_string();
        editor.end_time = "4:00".to_string();
        let Ok(Timing::Timed { start, end, .. }) = editor.timing(&regional()) else {
            panic!("a timed event");
        };
        assert_eq!(start, at(Warsaw, d(2026, 3, 29), 3, 0));
        assert_eq!(end - start, Duration::hours(1));
    }

    #[test]
    fn the_end_follows_the_start_so_the_event_keeps_its_length() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.start_time = "23:".to_string();
        editor.follow_start(&regional());
        assert_eq!(editor.end_time, "11:30");
        editor.start_time = "23:00".to_string();
        editor.follow_start(&regional());
        assert_eq!(
            (editor.end_date.as_str(), editor.end_time.as_str()),
            ("13.11.2026", "00:30")
        );
        editor.start_date = "14.11.2026".to_string();
        editor.follow_start(&regional());
        assert_eq!(
            (editor.end_date.as_str(), editor.end_time.as_str()),
            ("15.11.2026", "00:30")
        );
        // The end on its own moves alone.
        editor.end_time = "1:00".to_string();
        editor.follow_start(&regional());
        assert_eq!(editor.end_time, "1:00");
    }

    fn standup() -> Event {
        let mut event = timed("Standup", d(2026, 11, 13), (9, 0), (9, 30));
        event.calendar_id = "a@example.com".to_string();
        event.about.location = "Room 1".to_string();
        event
    }

    fn change(editor: &Editor) -> Edit {
        match editor.save(&regional()) {
            Ok(Save::Change(edit)) => *edit,
            other => panic!("not a change: {other:?}"),
        }
    }

    #[test]
    fn an_existing_event_saves_only_what_changed() {
        let event = standup();
        let mut editor = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        assert_eq!(
            (editor.title.as_str(), editor.location.as_str()),
            ("Standup", "Room 1")
        );
        assert!(change(&editor).is_empty());

        editor.location = "Room 3".to_string();
        editor.visibility = Visibility::Public;
        let edit = change(&editor);
        assert_eq!(
            edit.patch,
            Patch {
                location: Some("Room 3".to_string()),
                visibility: Some(Visibility::Public),
                ..Patch::default()
            }
        );
        assert_eq!(edit.event, event);
        assert_eq!(edit.to, None);
        let changed = edit.changed();
        assert_eq!(changed.about.location, "Room 3");
        assert_eq!(changed.id, event.id);
    }

    #[test]
    fn times_left_as_opened_keep_the_events_own_through_a_daylight_saving_change() {
        // 25 October 2026: Warsaw's clocks show 2:00–3:00 twice; this is the second 2:30.
        let start = chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 10, 25, 1, 30, 0).unwrap();
        let mut event = standup();
        event.timing = Timing::Timed {
            start,
            end: start + Duration::hours(1),
            zone: Warsaw,
        };
        let mut editor = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        assert_eq!(editor.start_time, "02:30");
        editor.location = "Room 3".to_string();
        assert_eq!(change(&editor).patch.timing, None);
        // Seconds the fields cannot show are kept too.
        event.timing = Timing::Timed {
            start: start + Duration::seconds(30),
            end: start + Duration::hours(1),
            zone: Warsaw,
        };
        let editor = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        assert!(change(&editor).is_empty());
    }

    #[test]
    fn a_location_or_title_with_spaces_around_is_left_alone_unless_changed() {
        let mut event = standup();
        event.about.location = "Room 1 ".to_string();
        event.title = " Standup".to_string();
        let mut editor = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        editor.show = Show::Free;
        let patch = change(&editor).patch;
        assert_eq!((patch.location, patch.title), (None, None));
        editor.location = "Room 2 ".to_string();
        assert_eq!(change(&editor).patch.location, Some("Room 2".to_string()));
    }

    #[test]
    fn an_untitled_event_opens_with_an_empty_title() {
        let mut event = standup();
        event.title = NO_TITLE.to_string();
        let mut editor = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        assert_eq!(editor.title, "");
        assert!(change(&editor).is_empty());
        editor.title = "Named".to_string();
        assert_eq!(change(&editor).patch.title, Some("Named".to_string()));
    }

    #[test]
    fn an_event_stored_in_another_zone_is_edited_in_its_own_zone() {
        let event = timed_in("Call", d(2026, 11, 13), (9, 0), (10, 0), New_York);
        let mut editor = Editor::of(&event, calendar("1", "work"), &regional());
        assert_eq!(editor.zone, New_York);
        assert_eq!(
            (editor.start_time.as_str(), editor.end_time.as_str()),
            ("09:00", "10:00")
        );
        assert!(change(&editor).is_empty());
        editor.start_time = "8:00".to_string();
        assert_eq!(
            change(&editor).patch.timing,
            Some(Timing::Timed {
                start: at(New_York, d(2026, 11, 13), 8, 0),
                end: at(New_York, d(2026, 11, 13), 10, 0),
                zone: New_York,
            })
        );
    }

    #[test]
    fn an_all_day_event_opens_with_its_days_and_becomes_timed_from_nine() {
        let event = all_day("Trip", d(2026, 11, 12), 3);
        let mut editor = Editor::of(&event, calendar("1", "work"), &regional());
        assert!(editor.all_day);
        assert_eq!(
            (editor.start_date.as_str(), editor.end_date.as_str()),
            ("12.11.2026", "14.11.2026")
        );
        assert!(change(&editor).is_empty());
        editor.all_day = false;
        editor.end_date = editor.start_date.clone();
        assert_eq!(
            change(&editor).patch.timing,
            Some(Timing::Timed {
                start: at(Warsaw, d(2026, 11, 12), 9, 0),
                end: at(Warsaw, d(2026, 11, 12), 10, 0),
                zone: Warsaw,
            })
        );
    }

    #[test]
    fn a_timed_event_made_all_day_keeps_its_days() {
        let event = standup();
        let mut editor = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        editor.all_day = true;
        assert_eq!(
            change(&editor).patch.timing,
            Some(Timing::AllDay {
                start: d(2026, 11, 13),
                end_exclusive: d(2026, 11, 14),
            })
        );
    }

    #[test]
    fn another_calendar_of_the_same_account_moves_the_event_there() {
        let event = standup();
        let mut editor = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        let choices = vec![
            (
                "a@example.com".to_string(),
                vec![calendar("1", "a@example.com"), calendar("1", "team")],
            ),
            ("b@example.com".to_string(), vec![calendar("2", "b")]),
        ];
        // Only the event's own account's calendars are offered.
        let offered = editor.choices(choices.clone());
        assert_eq!(offered, choices[..1]);
        // A new event can go to any of them.
        assert_eq!(
            Editor::from_quick(&quick(), &regional()).choices(choices.clone()),
            choices
        );

        editor.calendar = Some(calendar("1", "team"));
        let edit = change(&editor);
        assert!(edit.patch.is_empty());
        assert_eq!(edit.calendar.id, "a@example.com");
        assert_eq!(edit.to.as_ref().unwrap().id, "team");
        assert_eq!(edit.changed().calendar_id, "team");
    }
}
