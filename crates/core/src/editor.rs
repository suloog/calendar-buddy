//! The full event editor (`US-05-02`), for a new event and an existing one: the title,
//! description, start, end, time zone, all-day, location, busy or free, visibility and calendar
//! (`MOD-05-F04`…`F08`). Opened from the quick-entry box, it keeps what was typed there
//! (`MOD-05-F03`); opened on an existing event, it shows it in its own time zone and saves only
//! what changed, as a patch against the version opened (`MOD-05-F11`), moving it to another
//! calendar of the same account when another is picked (`MOD-05-F15`). A new event can be made to
//! repeat (`US-05-06`, `MOD-05-F18`, `F19`), and made out of office or focus time where the
//! account allows it (`US-05-08`, `MOD-05-F24`). The event's own colour is picked from Google's
//! event colours (`US-05-09`, `MOD-05-F25`). Guests are added by address or from the people
//! suggested, marked required or optional, and removed (`US-06-01`, `MOD-06-F01`, `F06`, `F07`).
//! The "Add Google Meet" switch turns on by itself with the first guest (`US-06-07`,
//! `MOD-06-F16`, `F17`).

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Weekday};
use chrono_tz::Tz;

use crate::accounts::Entry;
use crate::colors::EventColor;
use crate::create::{NO_CALENDAR, NO_TITLE, NewEvent, QuickEntry, wall};
use crate::guests::{self, Guest, NOT_AN_ADDRESS, Person, Response};
use crate::model::{About, Access, Calendar, Event, Kind, Reminders, Show, Timing, Visibility};
use crate::regional::Regional;
use crate::repeat::{Ends, Pattern, rule};

/// What the editor says when the end is set before the start.
pub const ENDS_BEFORE_STARTS: &str = "The event ends before it starts.";

/// What the app says when an event saved with Google Meet came back without a call
/// (MOD-06 *Constraints and edge cases*).
pub const WITHOUT_MEET: &str =
    "The event was saved without Google Meet: Google could not add a Meet call for this account.";

/// What the editor says when a series is set to end before its first occurrence.
pub const SERIES_ENDS_BEFORE_IT_STARTS: &str = "The series ends before it starts.";

/// How many occurrences a series ending after a number of them has until another is typed:
/// Google Calendar's default.
const DEFAULT_COUNT: u32 = 13;

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
    /// The event's type: picked for a new one where offered (`MOD-05-F24`), kept for an existing
    /// one.
    pub kind: Kind,
    /// The event's own colour; `None` for its calendar's (`MOD-05-F25`).
    pub color: Option<EventColor>,
    /// How a new event repeats; `None` when it does not (`MOD-05-F18`).
    pub repeat: Option<Pattern>,
    /// How the series ends (`MOD-05-F19`).
    pub ends: Ending,
    /// The last day of a series ending on a date, as typed.
    pub last_date: String,
    /// The number of occurrences of a series ending after them, as typed.
    pub count: String,
    /// The event's guests, the account itself included when it is one of them; the editor lists
    /// the others (`MOD-06-F01`).
    pub guests: Vec<Guest>,
    /// What is typed in the guest field, not added yet.
    pub guest_field: String,
    /// Why what was typed in the guest field was not added.
    pub guest_problem: Option<String>,
    /// The "Add Google Meet" switch: on for an event with a Meet call, and turned on by the
    /// first guest added (`MOD-06-F17`).
    pub meet: bool,
    /// The start as last read from the fields, which the end follows.
    start: Option<NaiveDateTime>,
}

/// How the series of a new event ends, as chosen in the editor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ending {
    #[default]
    Never,
    OnDate,
    AfterCount,
}

/// What saving the editor does.
#[derive(Clone, Debug, PartialEq)]
pub enum Save {
    Create(Calendar, Box<NewEvent>),
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
    /// A series' rule lines, replaced whole (`MOD-05-F20`).
    pub recurrence: Option<Vec<String>>,
    /// The event's own colour, `Some(None)` taking it away (`MOD-05-F25`).
    pub color: Option<Option<EventColor>>,
    /// The event's guests, replaced whole (`MOD-06-F07`).
    pub guests: Option<Vec<Guest>>,
    /// The user's own answer to the invitation, given in their own guest entry alone
    /// (`MOD-06-F11`).
    pub response: Option<Response>,
    /// Whether the event has a Google Meet call: `Some(true)` makes one, `Some(false)` takes it
    /// away (`MOD-06-F16`).
    pub meet: Option<bool>,
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
    /// Whether Google e-mails the event's guests about the change, as the user answered
    /// (`MOD-06-F12`).
    pub notify: bool,
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
        if let Some(color) = patch.color {
            event.about.color = color;
        }
        if let Some(guests) = patch.guests {
            event.about.guests = guests;
        }
        if let Some(response) = patch.response {
            for guest in event.about.guests.iter_mut().filter(|guest| guest.own) {
                guest.response = response;
            }
        }
        // A new call's link is known once Google made it.
        if patch.meet == Some(false) {
            event.about.meet = None;
        }
        if let Some(to) = &self.to {
            event.calendar_id = to.id.clone();
            event.color = to.color;
        }
        event
    }
}

/// The text of a title as the editor shows it: an untitled event has an empty one.
pub(crate) fn own_title(event: &Event) -> &str {
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
            kind: Kind::default(),
            color: None,
            repeat: None,
            ends: Ending::default(),
            last_date: regional.numeric_date(start.date()),
            count: DEFAULT_COUNT.to_string(),
            guests: Vec::new(),
            guest_field: String::new(),
            guest_problem: None,
            meet: false,
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
            kind: event.about.kind,
            color: event.about.color,
            guests: event.about.guests.clone(),
            meet: event.about.meet.is_some(),
            opened: Some((event.clone(), calendar.clone())),
            ..Editor::with(regional, times, zone, Some(calendar))
        }
    }

    /// The calendars the event can be saved to, under their accounts: any of `choices` for a new
    /// event, those of its own account for an existing one (`MOD-05-F15`), only its own for an
    /// existing one of a special type, which Google does not move.
    pub fn choices(&self, choices: Vec<(String, Vec<Calendar>)>) -> Vec<(String, Vec<Calendar>)> {
        let Some((event, own)) = &self.opened else {
            return choices;
        };
        let movable = event.about.kind == Kind::Ordinary;
        choices
            .into_iter()
            .filter(|(_, calendars)| calendars.iter().any(|c| c.account_id == event.account_id))
            .map(|(email, calendars)| {
                let calendars = calendars.into_iter();
                (
                    email,
                    calendars.filter(|c| movable || c.id == own.id).collect(),
                )
            })
            .collect()
    }

    /// The types a new event can be given in the calendar picked (`MOD-05-F24`): out of office
    /// and focus time besides an ordinary event on the primary calendar of a Workspace account,
    /// as Google allows them; none to pick from elsewhere or for an existing event. A type picked
    /// that the calendar now picked does not allow goes back to an ordinary event.
    pub fn kinds(&mut self, accounts: &[Entry]) -> Vec<Kind> {
        // Google names an account's primary calendar by the account's e-mail address.
        let allowed = self.opened.is_none()
            && self.calendar.as_ref().is_some_and(|calendar| {
                calendar.access == Access::Owner
                    && accounts.iter().any(|entry| {
                        entry.account.id == calendar.account_id
                            && entry.account.workspace
                            && entry.account.email == calendar.id
                    })
            });
        if allowed {
            vec![Kind::Ordinary, Kind::OutOfOffice, Kind::FocusTime]
        } else {
            if self.opened.is_none() {
                self.kind = Kind::Ordinary;
            }
            Vec::new()
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
    /// length, as in Google Calendar; so do the last day of a series and a weekly one's day
    /// when it repeats on the start's day alone.
    pub fn follow_start(&mut self, regional: &Regional) {
        let Some(start) = self.start_at(regional) else {
            return;
        };
        let before = self.start.replace(start);
        if let Some(Pattern::Weekly(days)) = &mut self.repeat
            && before.is_some_and(|before| *days == [before.weekday()])
        {
            *days = vec![start.weekday()];
        }
        if let (Some(before), Some(end)) = (before, self.end_at(regional))
            && before != start
        {
            let end = end + (start - before);
            self.end_date = regional.numeric_date(end.date());
            self.end_time = regional.time(end.time());
            if let Some(last) = regional.parse_date(&self.last_date) {
                let last = last + (start.date() - before.date());
                self.last_date = regional.numeric_date(last);
            }
        }
    }

    /// Weekly on the day of the week the event starts, as Google Calendar offers it first.
    pub fn weekly(&self) -> Pattern {
        Pattern::Weekly(
            self.start
                .map(|start| start.weekday())
                .into_iter()
                .collect(),
        )
    }

    /// Adds the day to the days a weekly series repeats on, or takes it away; the last one stays.
    pub fn toggle_day(&mut self, day: Weekday) {
        if let Some(Pattern::Weekly(days)) = &mut self.repeat {
            if !days.contains(&day) {
                days.push(day);
            } else if days.len() > 1 {
                days.retain(|d| *d != day);
            }
        }
    }

    /// The addresses of the account the event is saved with, which are never added as guests:
    /// its e-mail address and the calendar's ID, which is the address of a primary calendar.
    pub fn own_addresses(&self, accounts: &[Entry]) -> Vec<String> {
        let Some(calendar) = &self.calendar else {
            return Vec::new();
        };
        let mut own: Vec<String> = (accounts.iter())
            .filter(|entry| entry.account.id == calendar.account_id)
            .map(|entry| entry.account.email.clone())
            .chain([calendar.id.clone()])
            .collect();
        own.dedup();
        own
    }

    /// The guests the editor lists: all but the account itself.
    pub fn listed_guests(&mut self) -> impl Iterator<Item = &mut Guest> {
        self.guests.iter_mut().filter(|guest| !guest.own)
    }

    /// Invites the person, unless they are invited already or the address is one of the user's
    /// own (`own`); the guest field is emptied either way. The first guest turns Google Meet on
    /// (`MOD-06-F17`).
    pub fn add_guest(&mut self, person: &Person, own: &[String]) {
        let taken = (self.guests.iter().map(|guest| &guest.email))
            .chain(own)
            .any(|email| guests::same(email, &person.email));
        if !taken {
            self.meet |= self.guests.iter().all(|guest| guest.own);
            self.guests.push(Guest::invited(person));
        }
        self.guest_field.clear();
        self.guest_problem = None;
    }

    /// Invites the address typed in the guest field (`MOD-06-F01`); one that is not an address
    /// is not added, and the field says why.
    pub fn add_typed_guest(&mut self, own: &[String]) {
        match guests::address(&self.guest_field) {
            Some(email) => self.add_guest(
                &Person {
                    name: String::new(),
                    email,
                },
                own,
            ),
            None => self.guest_problem = Some(NOT_AN_ADDRESS.to_string()),
        }
    }

    /// Enter in the guest field: the address typed when it is one, otherwise the first person
    /// suggested, as Google Calendar picks it; with neither, the field says why nothing was added.
    pub fn enter_guest(&mut self, suggested: &[Person], own: &[String]) {
        match suggested.first() {
            Some(person) if guests::address(&self.guest_field).is_none() => {
                self.add_guest(person, own)
            }
            _ => self.add_typed_guest(own),
        }
    }

    /// No longer invites the guest with the address (`MOD-06-F07`).
    pub fn remove_guest(&mut self, email: &str) {
        self.guests.retain(|guest| guest.email != email);
    }

    /// The rule of the series the new event starts, as the repeat fields say; none when it does
    /// not repeat; why it cannot be saved otherwise.
    fn recurrence(&self, regional: &Regional, timing: &Timing) -> Result<Vec<String>, String> {
        let Some(pattern) = &self.repeat else {
            return Ok(Vec::new());
        };
        let ends = match self.ends {
            Ending::Never => Ends::Never,
            Ending::OnDate => {
                let last = (regional.parse_date(&self.last_date))
                    .ok_or("The end date of the series can't be read.")?;
                let first = match *timing {
                    Timing::Timed { start, zone, .. } => start.with_timezone(&zone).date_naive(),
                    Timing::AllDay { start, .. } => start,
                };
                if last < first {
                    return Err(SERIES_ENDS_BEFORE_IT_STARTS.to_string());
                }
                Ends::On(last)
            }
            Ending::AfterCount => match self.count.trim().parse() {
                Ok(count) if count > 0 => Ends::After(count),
                _ => return Err("The number of occurrences can't be read.".to_string()),
            },
        };
        Ok(vec![rule(pattern, ends, timing)])
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
            // Google keeps out of office and focus time only as timed events: all day is from
            // midnight to midnight, as Google Calendar makes it.
            if self.kind.timed_only() {
                return Ok(Timing::Timed {
                    start: wall(first, 0, self.zone),
                    end: wall(last + Duration::days(1), 0, self.zone),
                    zone: self.zone,
                });
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
            // Google keeps out of office and focus time busy.
            show: if self.kind.timed_only() {
                Show::Busy
            } else {
                self.show
            },
            visibility: self.visibility,
            kind: self.kind,
            color: self.color,
            // Attachments are added in Google (`DEC-37`).
            attachments: Vec::new(),
            // Google takes no guests for out of office and focus time.
            guests: if self.kind.timed_only() {
                Vec::new()
            } else {
                self.guests.clone()
            },
            // Its link is Google's to make.
            meet: None,
            // A new event gets its calendar's defaults from Google (`MOD-07-F03`).
            reminders: Reminders::Default,
        };
        // Nor a Google Meet call.
        let meet = self.meet && !self.kind.timed_only();
        let Some((event, opened_in)) = &self.opened else {
            return Ok(Save::Create(
                calendar,
                Box::new(NewEvent {
                    title,
                    recurrence: self.recurrence(regional, &timing)?,
                    timing,
                    about,
                    meet,
                }),
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
            recurrence: None,
            color: changed(event.about.color, about.color),
            guests: changed(event.about.guests.clone(), about.guests),
            response: None,
            meet: changed(event.about.meet.is_some(), meet),
        };
        let moved = calendar.account_id != opened_in.account_id || calendar.id != opened_in.id;
        Ok(Save::Change(Box::new(Edit {
            event: event.clone(),
            calendar: opened_in.clone(),
            patch,
            to: moved.then_some(calendar),
            notify: false,
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
    use crate::repeat::Pattern;
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
            default_reminders: Vec::new(),
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
            *event,
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
                    kind: Kind::Ordinary,
                    color: None,
                    attachments: Vec::new(),
                    guests: Vec::new(),
                    meet: None,
                    reminders: Reminders::Default,
                },
                recurrence: Vec::new(),
                meet: false,
            }
        );
    }

    fn account(id: &str, email: &str, workspace: bool) -> Entry {
        Entry {
            account: crate::accounts::Account {
                id: id.to_string(),
                email: email.to_string(),
                workspace,
                contacts: true,
                directory: false,
            },
            state: crate::accounts::AccountState::Connected,
        }
    }

    #[test]
    fn out_of_office_and_focus_time_are_offered_only_on_the_primary_calendar_of_a_workspace_account()
     {
        let accounts = [
            account("1", "a@example.com", true),
            account("2", "b@gmail.com", false),
        ];
        let mut editor = Editor::from_quick(&quick(), &regional());
        assert_eq!(
            editor.kinds(&accounts),
            [Kind::Ordinary, Kind::OutOfOffice, Kind::FocusTime]
        );
        editor.kind = Kind::OutOfOffice;

        // Another calendar of the Workspace account: nothing to pick, and the event is ordinary.
        editor.calendar = Some(calendar("1", "team"));
        assert_eq!(editor.kinds(&accounts), []);
        assert_eq!(editor.kind, Kind::Ordinary);
        // The primary calendar of a personal account.
        editor.calendar = Some(calendar("2", "b@gmail.com"));
        assert_eq!(editor.kinds(&accounts), []);
        // A primary calendar of the Workspace account shared with the personal one.
        editor.calendar = Some(Calendar {
            access: Access::Writer,
            ..calendar("2", "a@example.com")
        });
        assert_eq!(editor.kinds(&accounts), []);

        // An existing event keeps its type and is not offered another.
        let mut opened = Editor::from_quick(&quick(), &regional());
        opened.kind = Kind::FocusTime;
        opened.opened = Some((
            timed("focus", d(2026, 11, 12), (9, 0), (10, 0)),
            calendar("1", "a@example.com"),
        ));
        assert_eq!(opened.kinds(&accounts), []);
        assert_eq!(opened.kind, Kind::FocusTime);
    }

    #[test]
    fn an_all_day_out_of_office_is_sent_from_midnight_to_midnight_and_busy() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.kind = Kind::OutOfOffice;
        editor.all_day = true;
        editor.start_date = "13.11.2026".to_string();
        editor.end_date = "14.11.2026".to_string();
        editor.show = Show::Free;

        let Ok(Save::Create(_, event)) = editor.save(&regional()) else {
            panic!("a new event");
        };
        assert_eq!(
            event.timing,
            Timing::Timed {
                start: at(Warsaw, d(2026, 11, 13), 0, 0),
                end: at(Warsaw, d(2026, 11, 15), 0, 0),
                zone: Warsaw,
            }
        );
        assert_eq!(event.about.kind, Kind::OutOfOffice);
        assert_eq!(event.about.show, Show::Busy);
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

    fn created(editor: &Editor) -> NewEvent {
        match editor.save(&regional()) {
            Ok(Save::Create(_, event)) => *event,
            other => panic!("not a new event: {other:?}"),
        }
    }

    /// "Training" on Monday 16 November 2026, 18:00–19:00.
    fn training() -> Editor {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.title = "Training".to_string();
        editor.start_date = "16.11.2026".to_string();
        editor.start_time = "18:00".to_string();
        editor.follow_start(&regional());
        editor.end_time = "19:00".to_string();
        editor
    }

    #[test]
    fn tc_05_07_a_new_event_repeats_weekly_on_monday_and_wednesday_ending_after_10() {
        let mut editor = training();
        assert_eq!(editor.repeat, None);
        assert!(created(&editor).recurrence.is_empty());

        // Weekly starts on the day the event starts.
        editor.repeat = Some(editor.weekly());
        assert_eq!(editor.repeat, Some(Pattern::Weekly(vec![Weekday::Mon])));
        editor.toggle_day(Weekday::Wed);
        editor.ends = Ending::AfterCount;
        editor.count = "10".to_string();

        assert_eq!(
            created(&editor).recurrence,
            ["RRULE:FREQ=WEEKLY;BYDAY=MO,WE;COUNT=10"]
        );
    }

    #[test]
    fn a_new_event_repeats_with_each_preset_and_ends_on_a_date_or_never() {
        let mut editor = training();
        let presets = [
            (Pattern::Daily, "RRULE:FREQ=DAILY"),
            (Pattern::Weekdays, "RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"),
            (Pattern::Monthly, "RRULE:FREQ=MONTHLY"),
            (Pattern::Yearly, "RRULE:FREQ=YEARLY"),
        ];
        for (pattern, expected) in presets {
            editor.repeat = Some(pattern);
            assert_eq!(created(&editor).recurrence, [expected]);
        }
        editor.ends = Ending::OnDate;
        // The series ends on the day it starts until another is typed; it moves with the start.
        assert_eq!(editor.last_date, "16.11.2026");
        editor.last_date = "16.12.2026".to_string();
        assert_eq!(
            created(&editor).recurrence,
            ["RRULE:FREQ=YEARLY;UNTIL=20261216T225959Z"]
        );
    }

    #[test]
    fn the_last_day_of_a_weekly_series_stays_chosen() {
        let mut editor = training();
        editor.repeat = Some(editor.weekly());
        editor.toggle_day(Weekday::Mon);
        assert_eq!(editor.repeat, Some(Pattern::Weekly(vec![Weekday::Mon])));
        editor.toggle_day(Weekday::Fri);
        editor.toggle_day(Weekday::Mon);
        assert_eq!(editor.repeat, Some(Pattern::Weekly(vec![Weekday::Fri])));
    }

    #[test]
    fn weekly_on_the_start_day_follows_the_start_to_another_day() {
        let mut editor = training();
        editor.repeat = Some(editor.weekly());
        editor.start_date = "19.11.2026".to_string();
        editor.follow_start(&regional());
        assert_eq!(editor.repeat, Some(Pattern::Weekly(vec![Weekday::Thu])));
        // Days the user chose stay as chosen.
        editor.toggle_day(Weekday::Sat);
        editor.start_date = "20.11.2026".to_string();
        editor.follow_start(&regional());
        assert_eq!(
            editor.repeat,
            Some(Pattern::Weekly(vec![Weekday::Thu, Weekday::Sat]))
        );
    }

    #[test]
    fn weekly_chosen_while_the_start_could_not_be_read_repeats_on_the_start_day() {
        let mut editor = training();
        editor.start_date = "16.11.".to_string();
        editor.repeat = Some(editor.weekly());
        editor.start_date = "16.11.2026".to_string();
        assert_eq!(created(&editor).recurrence, ["RRULE:FREQ=WEEKLY;BYDAY=MO"]);
    }

    #[test]
    fn a_series_ending_before_it_starts_or_after_no_occurrences_cannot_be_saved() {
        let mut editor = training();
        editor.repeat = Some(Pattern::Daily);
        editor.ends = Ending::OnDate;
        editor.last_date = "15.11.2026".to_string();
        assert_eq!(
            editor.save(&regional()),
            Err(SERIES_ENDS_BEFORE_IT_STARTS.to_string())
        );
        editor.last_date = "31.11.2026".to_string();
        assert_eq!(
            editor.save(&regional()),
            Err("The end date of the series can't be read.".to_string())
        );
        editor.ends = Ending::AfterCount;
        assert_eq!(editor.count, "13");
        for count in ["0", "ten", ""] {
            editor.count = count.to_string();
            assert_eq!(
                editor.save(&regional()),
                Err("The number of occurrences can't be read.".to_string())
            );
        }
        // A series that does not repeat is not held back by its end.
        editor.repeat = None;
        assert!(editor.save(&regional()).is_ok());
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
    fn an_events_own_colour_is_shown_changed_and_taken_away_in_the_editor() {
        let event = standup();
        let mut editor = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        assert_eq!(editor.color, None);

        editor.color = Some(EventColor::Tomato);
        let edit = change(&editor);
        assert_eq!(
            edit.patch,
            Patch {
                color: Some(Some(EventColor::Tomato)),
                ..Patch::default()
            }
        );
        let changed = edit.changed();
        assert_eq!(changed.about.color, Some(EventColor::Tomato));
        assert_eq!(changed.shown_color(), EventColor::Tomato.rgb());

        // Back to the calendar's colour.
        let mut editor = Editor::of(&changed, calendar("1", "a@example.com"), &regional());
        assert_eq!(editor.color, Some(EventColor::Tomato));
        editor.color = None;
        let edit = change(&editor);
        assert_eq!(edit.patch.color, Some(None));
        assert_eq!(edit.changed().shown_color(), event.color);
    }

    #[test]
    fn a_new_event_is_created_in_the_colour_picked() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.color = Some(EventColor::Basil);
        let Ok(Save::Create(_, event)) = editor.save(&regional()) else {
            panic!("a new event");
        };
        assert_eq!(event.about.color, Some(EventColor::Basil));
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

        // An out-of-office event stays in its calendar, as Google moves only ordinary events.
        let mut away = standup();
        away.about.kind = Kind::OutOfOffice;
        let editor = Editor::of(&away, calendar("1", "a@example.com"), &regional());
        assert_eq!(
            editor.choices(choices),
            [(
                "a@example.com".to_string(),
                vec![calendar("1", "a@example.com")]
            )]
        );
    }

    fn person(name: &str, email: &str) -> Person {
        Person {
            name: name.to_string(),
            email: email.to_string(),
        }
    }

    #[test]
    fn guests_are_added_by_address_or_from_a_suggestion_and_saved_with_a_new_event() {
        let accounts = [account("1", "a@example.com", false)];
        let mut editor = Editor::from_quick(&quick(), &regional());
        let own = editor.own_addresses(&accounts);
        assert_eq!(own, ["a@example.com"]);

        editor.add_guest(&person("Adam Kowalski", "adam@example.com"), &own);
        editor.guest_field = " ewa@example.com ".to_string();
        editor.add_typed_guest(&own);
        editor.listed_guests().next().unwrap().optional = true;

        assert_eq!(editor.guest_field, "");
        let Ok(Save::Create(_, event)) = editor.save(&regional()) else {
            panic!("a new event");
        };
        assert_eq!(
            event.about.guests,
            [
                Guest {
                    optional: true,
                    ..Guest::invited(&person("Adam Kowalski", "adam@example.com"))
                },
                Guest::invited(&person("", "ewa@example.com")),
            ]
        );
    }

    #[test]
    fn an_address_that_is_not_one_is_not_added_and_the_field_says_why() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.guest_field = "adam@example".to_string();

        editor.add_typed_guest(&[]);

        assert!(editor.guests.is_empty());
        assert_eq!(editor.guest_field, "adam@example");
        assert_eq!(editor.guest_problem.as_deref(), Some(NOT_AN_ADDRESS));
        // Fixed, it is added and the field no longer complains.
        editor.guest_field = "adam@example.com".to_string();
        editor.add_typed_guest(&[]);
        assert_eq!(editor.guests.len(), 1);
        assert_eq!(editor.guest_problem, None);
    }

    #[test]
    fn a_guest_added_twice_is_added_once_and_the_users_own_address_is_ignored() {
        let accounts = [account("1", "me@example.com", false)];
        let mut editor = Editor::from_quick(&quick(), &regional());
        let own = editor.own_addresses(&accounts);

        editor.add_guest(&person("Adam", "adam@example.com"), &own);
        editor.add_guest(&person("Adam K.", "ADAM@example.com"), &own);
        editor.add_guest(&person("Me", "me@example.com"), &own);
        // The primary calendar's ID is the account's address too.
        editor.add_guest(&person("", "a@example.com"), &own);

        let emails: Vec<&str> = editor.guests.iter().map(|g| g.email.as_str()).collect();
        assert_eq!(emails, ["adam@example.com"]);
    }

    #[test]
    fn guests_are_added_to_and_removed_from_an_existing_event_the_account_itself_kept() {
        let mut event = standup();
        let own = Guest {
            own: true,
            response: crate::guests::Response::Accepted,
            ..Guest::invited(&person("", "a@example.com"))
        };
        let ewa = Guest {
            response: crate::guests::Response::Declined,
            ..Guest::invited(&person("Ewa", "ewa@example.com"))
        };
        event.about.guests = vec![own.clone(), ewa];
        let mut editor = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        let listed: Vec<String> = editor.listed_guests().map(|g| g.email.clone()).collect();
        assert_eq!(listed, ["ewa@example.com"]);

        editor.add_guest(&person("Adam", "adam@example.com"), &[]);
        editor.remove_guest("ewa@example.com");
        let edit = change(&editor);

        let adam = Guest::invited(&person("Adam", "adam@example.com"));
        assert_eq!(
            edit.patch,
            Patch {
                guests: Some(vec![own.clone(), adam.clone()]),
                ..Patch::default()
            }
        );
        assert_eq!(edit.changed().about.guests, [own, adam]);
        // Left as they were, the guests are not sent.
        let unchanged = Editor::of(&event, calendar("1", "a@example.com"), &regional());
        assert_eq!(change(&unchanged).patch.guests, None);
    }

    #[test]
    fn out_of_office_is_saved_without_guests_as_google_takes_none() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.add_guest(&person("", "adam@example.com"), &[]);
        editor.kind = Kind::OutOfOffice;
        let Ok(Save::Create(_, event)) = editor.save(&regional()) else {
            panic!("a new event");
        };
        assert!(event.about.guests.is_empty());
    }

    #[test]
    fn the_first_guest_turns_google_meet_on_and_a_new_event_is_saved_with_it() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        assert!(!editor.meet);

        editor.add_guest(&person("Adam", "adam@example.com"), &[]);
        assert!(editor.meet);
        let Ok(Save::Create(_, event)) = editor.save(&regional()) else {
            panic!("a new event");
        };
        assert!(event.meet);

        // Switched off, it stays off for the guests after the first, and without guests.
        editor.meet = false;
        editor.add_guest(&person("Ewa", "ewa@example.com"), &[]);
        assert!(!editor.meet);
        editor.remove_guest("adam@example.com");
        editor.remove_guest("ewa@example.com");
        assert!(!editor.meet);
        let Ok(Save::Create(_, event)) = editor.save(&regional()) else {
            panic!("a new event");
        };
        assert!(!event.meet);
    }

    #[test]
    fn google_meet_is_switched_on_and_off_for_an_existing_event() {
        let mut call = standup();
        call.about.meet = Some("https://meet.google.com/abc-defg-hij".into());
        let mut editor = Editor::of(&call, calendar("1", "a@example.com"), &regional());
        assert!(editor.meet);
        assert_eq!(change(&editor).patch, Patch::default());

        editor.meet = false;
        let edit = change(&editor);
        assert_eq!(edit.patch.meet, Some(false));
        assert_eq!(edit.changed().about.meet, None);

        // The account itself is no guest of its own event: the first one turns Meet on.
        let mut standup = standup();
        standup.about.guests = vec![Guest {
            own: true,
            ..Guest::invited(&person("", "a@example.com"))
        }];
        let mut editor = Editor::of(&standup, calendar("1", "a@example.com"), &regional());
        assert!(!editor.meet);
        editor.add_guest(&person("Adam", "adam@example.com"), &[]);
        assert_eq!(change(&editor).patch.meet, Some(true));
    }

    #[test]
    fn out_of_office_is_saved_without_google_meet() {
        let mut editor = Editor::from_quick(&quick(), &regional());
        editor.meet = true;
        editor.kind = Kind::OutOfOffice;
        let Ok(Save::Create(_, event)) = editor.save(&regional()) else {
            panic!("a new event");
        };
        assert!(!event.meet);
    }

    #[test]
    fn enter_adds_the_address_typed_or_else_the_first_person_suggested() {
        let suggested = [
            person("Adam Kowalski", "adam@example.com"),
            person("Adam Nowak", "nowak@example.com"),
        ];
        let mut editor = Editor::from_quick(&quick(), &regional());

        editor.guest_field = "Ada".to_string();
        editor.enter_guest(&suggested, &[]);
        editor.guest_field = "a@b.co".to_string();
        editor.enter_guest(&suggested, &[]);
        editor.guest_field = "nobody".to_string();
        editor.enter_guest(&[], &[]);

        let emails: Vec<&str> = editor.guests.iter().map(|g| g.email.as_str()).collect();
        assert_eq!(emails, ["adam@example.com", "a@b.co"]);
        assert_eq!(editor.guest_problem.as_deref(), Some(NOT_AN_ADDRESS));
    }
}
