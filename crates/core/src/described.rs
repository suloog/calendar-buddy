//! What a description says (`US-10-02`, `US-10-04`): the request that asks the model to read it,
//! and the event the app makes of its answer, or the change it makes to an existing one. The
//! model only names the kind of day from a closed list and a short value; the app works the date
//! out itself in the user's time zone (`MOD-10-F04`, variant D of the test of 2026-10-27).
//! Neither the description nor the answer is kept or logged (`MOD-10-NFR-01`).

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Timelike, Utc};
use chrono_tz::Tz;
use serde_json::{Value, json};

use crate::accounts::Entry;
use crate::create::{self, Asked};
use crate::editor::{Editor, Unresolved};
use crate::guests::{self, Person};
use crate::model::{Calendar, Event, Timing};
use crate::ollama::{KEEP_ALIVE, Ollama};
use crate::regional::Regional;
use crate::settings::Settings;

/// The most characters of a description sent to the model (`DEC-210`).
pub const LIMIT: usize = 500;

/// The first `LIMIT` characters of the description — letters, not bytes.
pub fn cut(description: &str) -> &str {
    match description.char_indices().nth(LIMIT) {
        Some((end, _)) => &description[..end],
        None => description,
    }
}

/// The rules the model reads by; the same for every description, so that Ollama can reuse them
/// (`MOD-10` technical notes).
const RULES: &str = r#"You turn a request to add or change a calendar event into JSON. The request is in Polish or English; answer in the language of the request.
The user's message gives the current day and time, then the request after an empty line.
Rules:
- action: "add" when the request describes a new event; "change" when it moves or changes an existing event (przesuń, przełóż, przenieś, zmień, wydłuż, skróć, move, change, reschedule, push, make it); "none" when it is not an event, such as a question or a greeting.
- event: for "change", the words naming the event to change, as written, without the new day, time or length: "przesuń spotkanie z Anią na piątek" -> "spotkanie z Anią"; "move the standup to 10" -> "standup"; "" when the request names no event ("przesuń na 15", "move it to 3pm"). For "add" and "none": "".
- title: a short event title in the language of the request, without the date or time. Do not translate it. "Coffee with Ben tomorrow" -> "Coffee with Ben"; "Kawa z Bartkiem jutro" -> "Kawa z Bartkiem".
- guests: first names of the people taking part, as written in the request's language, in their basic (nominative) form. Do not translate names. "with Tom" -> "Tom"; "z Tomkiem" -> "Tomek"; "z Anią" -> "Ania"; "with Anna" -> "Anna". Words for roles or groups are not guests: mom, mama, team, zespół, client, klient, board, zarząd. Empty list if no person is named.
- meet: true when Google Meet or a video call is asked for; false only when it is refused ("no Meet", "bez Meeta"); null when it is not mentioned.
- duration_min: the length in minutes when the request gives one ("45 minut" -> 45, "two hours" -> 120, "godzina i kwadrans" -> 75); null when it gives none.
- calendar: the calendar's name when the request names one ("w kalendarzu Praca" -> "Praca", "in the Home calendar" -> "Home"), as written; "" when it names none. The title never holds it.
- kind and day describe the day; do not compute the date:
  kind "yesterday" (wczoraj), "today" (dziś, dzisiaj, today, tonight), "tomorrow" (jutro), "day_after_tomorrow" (pojutrze): day "";
  kind "none" when the request gives no day ("Lunch o 13", "Trening wieczorem", "Call at 3pm"): day "";
  kind "weekday" for a weekday alone ("w piątek", "on Friday"): day "fri" (mon tue wed thu fri sat sun);
  kind "next_weekday" ONLY when the request says next: "next Thursday", "w przyszły czwartek", "next week on Tuesday", "w przyszłym tygodniu we wtorek": day "thu" / "tue";
  kind "in_days" for "in N days" / "za N dni", "in a week" / "za tydzień": day "7";
  kind "date" for a date: day "MM-DD" ("14 listopada" -> "11-14", "3 Nov" -> "11-03");
  kind "same" never when adding.
- time: HH:MM 24-hour when an hour is given; "morning", "afternoon" or "evening" when only a part of the day is given (rano, po południu, wieczorem); null when no time is given.
- Only in a "change": title "", calendar "", guests [] and meet null; kind, day, time and duration_min give only what changes. kind "same" and day "" when no new day is given; a new day takes the kinds above, as when adding ("na jutro" -> kind "tomorrow", day ""; "to Friday" -> kind "weekday", day "fri"). time null when no new hour is given; duration_min null when no new length is given. A number alone is an hour: "przesuń na 15" -> kind "same", time "15:00".
Answer with compact JSON on one line, no spaces."#;

const KINDS: [&str; 10] = [
    "yesterday",
    "today",
    "tomorrow",
    "day_after_tomorrow",
    "weekday",
    "next_weekday",
    "in_days",
    "date",
    // In a change, the day stays.
    "same",
    // No day given: the window asks for it (`DEC-232`).
    "none",
];
const WEEKDAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

/// The fixed form of the answer (structured output): variant D, with whether it is an event and a
/// length left empty when none is given (`DEC-218`); Meet empty when not mentioned (`DEC-223`);
/// a change with the words naming its event and the day that stays (`DEC-228`); the calendar
/// named, and no day given (`DEC-232`, `DEC-233`).
fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "action": {"type": "string", "enum": ["add", "change", "none"]},
            "event": {"type": "string"},
            "title": {"type": "string"},
            "calendar": {"type": "string"},
            "kind": {"type": "string", "enum": KINDS},
            "day": {"type": "string"},
            "time": {"type": ["string", "null"]},
            "duration_min": {"type": ["integer", "null"]},
            "guests": {"type": "array", "items": {"type": "string"}},
            "meet": {"type": ["boolean", "null"]},
        },
        "required": [
            "action", "event", "title", "calendar", "kind", "day", "time", "duration_min",
            "guests", "meet",
        ],
    })
}

/// The user's message: the day and time now, then the description, cut to `LIMIT`.
pub fn message(description: &str, now: DateTime<Tz>) -> String {
    format!(
        "Now is {}, time zone {}.\n\n{}",
        now.format("%A %Y-%m-%d %H:%M"),
        now.timezone().name(),
        cut(description)
    )
}

/// The description in a user's message `message` made.
pub fn description_in(message: &str) -> Option<&str> {
    message
        .split_once("\n\n")
        .map(|(_, description)| description)
}

/// The request to Ollama's `/api/chat` that reads the description: the rules, the message,
/// the fixed form, no thinking, temperature 0, and the model kept for `KEEP_ALIVE` after it.
pub fn request(ollama: &Ollama, description: &str, now: DateTime<Tz>) -> Value {
    json!({
        "model": ollama.model,
        "messages": [
            {"role": "system", "content": RULES},
            {"role": "user", "content": message(description, now)},
        ],
        "stream": false,
        "format": schema(),
        "think": false,
        "keep_alive": KEEP_ALIVE,
        "options": {"temperature": 0},
    })
}

/// What the model found in a description.
#[derive(Clone, Debug, PartialEq)]
pub enum Reading {
    /// An event to add.
    Event(Described),
    /// A change of an existing event (`MOD-10-F15`).
    Change(Change),
    /// Not an event, or an answer that cannot be used (`MOD-10-F23`).
    NoEvent,
}

/// A change of an existing event, as the description gives it: the words naming the event and
/// what changes; `None` stays as it is (`MOD-10-F18`).
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    /// The words naming the event; empty when the description names none, and the event whose
    /// details are open is meant (`MOD-10-F16`).
    pub event: String,
    pub date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    /// In minutes.
    pub length: Option<u32>,
}

impl Change {
    /// Makes the change in the editor of the event, as typing it in the fields would: a new day
    /// or hour moves the start and the end follows it, the length kept, unless a new length is
    /// given. The day and the hour are the user's, in `zone`, as the week shows the event; the
    /// editor keeps them in the event's own zone. An all-day event moves to the new day and
    /// stays all day; an hour or a length, which its fields do not have, is not applied
    /// (`US-10-04` *Edge cases*).
    pub fn apply(&self, editor: &mut Editor, regional: &Regional, zone: Tz) {
        let (Some(date), Some(time)) = (
            regional.parse_date(&editor.start_date),
            regional.parse_time(&editor.start_time),
        ) else {
            return;
        };
        if editor.all_day {
            if let Some(date) = self.date {
                editor.start_date = regional.numeric_date(date);
                editor.follow_start(regional);
            }
            return;
        }
        let minute = |time: NaiveTime| time.num_seconds_from_midnight() / 60;
        let seen = create::wall(date, minute(time), editor.zone).with_timezone(&zone);
        let (date, time) = (
            self.date.unwrap_or(seen.date_naive()),
            self.time.unwrap_or(seen.time()),
        );
        let start = create::wall(date, minute(time), zone);
        let theirs = editor.zone;
        let own = |at: DateTime<Utc>| at.with_timezone(&theirs).naive_local();
        let local = own(start);
        editor.start_date = regional.numeric_date(local.date());
        editor.start_time = regional.time(local.time());
        editor.follow_start(regional);
        if let Some(length) = self.length {
            let end = own(start + Duration::minutes(length.into()));
            editor.end_date = regional.numeric_date(end.date());
            editor.end_time = regional.time(end.time());
        }
    }
}

/// The events a change names by `name`, among `events` from `today` on — none before, whatever
/// the cache holds (`MOD-10-F16`, `DEC-202`) — in the order they start. An event fits when each
/// word of the name fits a word of its title, or each word of its title a word of the name, as
/// `fits` compares them (`DEC-229`).
pub fn matching(events: &[Event], name: &str, today: NaiveDate, zone: Tz) -> Vec<usize> {
    let name = words(name);
    let starts = |event: &Event| match event.timing {
        Timing::Timed { start, .. } => (start.with_timezone(&zone).date_naive(), Some(start)),
        Timing::AllDay { start, .. } => (start, None),
    };
    let within = |some: &[String], all: &[String]| {
        !some.is_empty() && (some.iter()).all(|word| all.iter().any(|other| fits(word, other)))
    };
    let mut found: Vec<usize> = (events.iter().enumerate())
        .filter(|(_, event)| starts(event).0 >= today)
        .filter(|(_, event)| {
            let title = words(&event.title);
            within(&name, &title) || within(&title, &name)
        })
        .map(|(i, _)| i)
        .collect();
    found.sort_by_key(|&i| starts(&events[i]));
    found
}

/// The words of a name or a title, in lower case, without those shorter than three letters
/// ("z", "na") unless it has no other.
fn words(text: &str) -> Vec<String> {
    let all: Vec<String> = text.split_whitespace().map(str::to_lowercase).collect();
    let long: Vec<String> = (all.iter())
        .filter(|w| w.chars().count() >= 3)
        .cloned()
        .collect();
    if long.is_empty() { all } else { long }
}

/// Whether two words are the same word, also in another grammatical case, as Polish names an
/// event in a sentence ("dentystę" for "Dentysta", "Anią" for "Ania"): they share all but at most
/// their last two letters, and at least three.
fn fits(word: &str, other: &str) -> bool {
    let (word, other): (Vec<char>, Vec<char>) = (word.chars().collect(), other.chars().collect());
    let shared = word.iter().zip(&other).take_while(|(a, b)| a == b).count();
    let shorter = word.len().min(other.len());
    shared == word.len().max(other.len()) || (shared >= 3 && shared + 2 >= shorter)
}

/// An event to add, as the description gives it.
#[derive(Clone, Debug, PartialEq)]
pub struct Described {
    pub title: String,
    /// `None` when the description gives no day (`MOD-10-F13`).
    pub date: Option<NaiveDate>,
    /// `None` when the description gives no time (`MOD-10-F14`).
    pub time: Option<NaiveTime>,
    /// In minutes; `None` when the description gives no length.
    pub length: Option<u32>,
    pub guests: Vec<String>,
    /// Google Meet asked for (`true`), refused (`false`), or not mentioned (`None`, `DEC-223`).
    pub meet: Option<bool>,
    /// The calendar's name, as the description gives it (`MOD-10-F09`).
    pub calendar: Option<String>,
}

impl Described {
    /// Whether it opens the full editor: it names guests or asks for Meet, which the "New
    /// event" window has not (`MOD-10-F06`, `DEC-205`).
    pub fn full(&self) -> bool {
        !self.guests.is_empty() || self.meet == Some(true)
    }

    /// The times of the event: from its time on its day in `zone`, its length long, or the
    /// default length of a new event when it gives none (`MOD-10-F05`). A day or a time it does
    /// not give is not guessed but asked for (`missing`); until then the event stands on `today`
    /// or at midnight, which the window does not show (`DEC-234`).
    pub fn times(
        &self,
        default_length: u32,
        today: NaiveDate,
        zone: Tz,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        let length = self.length.unwrap_or(default_length);
        let time = self.time.unwrap_or(NaiveTime::MIN);
        let minute = time.signed_duration_since(NaiveTime::MIN).num_minutes();
        let start = create::wall(self.date.unwrap_or(today), minute as u32, zone);
        (start, start + Duration::minutes(length.into()))
    }

    /// What the window asks for: the day and the time the description does not give
    /// (`MOD-10-F13`, `MOD-10-F14`, `MOD-10-U07`).
    pub fn missing(&self) -> Option<Asked> {
        Asked::new(self.date.is_none(), self.time.is_none())
    }
}

/// The calendar a description names by `name` (`MOD-10-F09`): of those a new event can go to,
/// in the side panel's order, the first whose name is `name`, letter case aside; one whose
/// events are hidden counts as not found, as one that is disabled or read-only, which a new
/// event cannot go to (`US-10-05` *Edge cases*, `DEC-233`). `None` when none fits.
pub fn calendar_named(
    name: &str,
    accounts: &[Entry],
    calendars: &[Calendar],
    settings: &Settings,
) -> Option<Calendar> {
    let name = name.trim().to_lowercase();
    (create::choices(accounts, calendars, settings).into_iter())
        .flat_map(|(_, calendars)| calendars)
        .filter(|c| !(settings.hidden).contains(&(c.account_id.clone(), c.id.clone())))
        .find(|c| c.name.trim().to_lowercase() == name)
}

/// Fills the guests and Meet of a description into the editor of its new event (`US-10-03`).
/// `found` holds, for each name, the people found for it as when it is typed in the guest field
/// — Google's search, then the recent guests (`MOD-10-F11`); `None` when nothing was looked up,
/// offline, and then the names are left out (`DEC-214`). A name the guest field would suggest
/// exactly one person for invites them; any other waits with its suggestions (`MOD-10-F12`). Meet
/// is as the description says, or as the guests added left it (`MOD-10-F07`, `MOD-06-F17`). A
/// local calendar takes neither (`MOD-10-F25`).
pub fn fill(
    editor: &mut Editor,
    described: &Described,
    found: Option<&[(String, Vec<Person>)]>,
    own: &[String],
) {
    if !editor.google() {
        return;
    }
    for (name, people) in found.into_iter().flatten() {
        let mut suggested = guests::suggestions(name, people, &editor.guests, own);
        if suggested.len() == 1 {
            editor.add_guest(&suggested.remove(0), own);
        } else {
            editor.unresolved.push(Unresolved {
                name: name.clone(),
                suggested,
            });
        }
    }
    if let Some(meet) = described.meet {
        editor.meet = meet;
        editor.meet_described = true;
    }
}

/// The model's answer read; anything not in the fixed form is no event.
pub fn read(answer: &str, today: NaiveDate) -> Reading {
    parse(answer, today).unwrap_or(Reading::NoEvent)
}

fn parse(answer: &str, today: NaiveDate) -> Option<Reading> {
    let answer: Value = serde_json::from_str(answer).ok()?;
    match answer["action"].as_str()? {
        "none" => return Some(Reading::NoEvent),
        "change" => return change(&answer, today),
        "add" => {}
        _ => return None,
    }
    let title = answer["title"].as_str()?.trim().to_string();
    if title.is_empty() {
        return None;
    }
    let (kind, value) = (answer["kind"].as_str()?, answer["day"].as_str()?);
    let (time, length) = time_and_length(&answer)?;
    // No day is asked for, but a part of the day alone is today's: "Trening wieczorem" is this
    // evening (`MOD-10-F27`, `DEC-232`). Told not to, the model may say "same" when adding: the
    // day the description gives, else no day either.
    let part_of_day = (answer["time"].as_str()).is_some_and(|time| time_of_day(time).is_some());
    let date = match kind {
        "same" | "none" => same(value, today).or(part_of_day.then_some(today)),
        kind => Some(day(kind, value, today)?),
    };
    let guests = (answer["guests"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect();
    Some(Reading::Event(Described {
        title,
        date,
        time,
        length,
        guests,
        meet: answer["meet"].as_bool(),
        // An answer without the field, as recorded before it was asked for, names none.
        calendar: (answer["calendar"].as_str())
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string),
    }))
}

/// A change: the words naming the event, and the day, the hour and the length that change.
fn change(answer: &Value, today: NaiveDate) -> Option<Reading> {
    let value = answer["day"].as_str()?;
    let date = match answer["kind"].as_str()? {
        "same" | "none" => same(value, today),
        kind => Some(day(kind, value, today)?),
    };
    let (time, length) = time_and_length(answer)?;
    Some(Reading::Change(Change {
        event: answer["event"].as_str()?.trim().to_string(),
        date,
        time,
        length,
    }))
}

/// The day of an answer of the kind "same": none, the day staying, unless its value names one —
/// a kind ("tomorrow"), a weekday ("fri") or a date ("11-14").
fn same(value: &str, today: NaiveDate) -> Option<NaiveDate> {
    day(value, "", today)
        .or_else(|| day("weekday", value, today))
        .or_else(|| day("date", value, today))
}

/// The hour and the length of an answer, each `None` when it gives none.
fn time_and_length(answer: &Value) -> Option<(Option<NaiveTime>, Option<u32>)> {
    let time = match &answer["time"] {
        Value::Null => None,
        Value::String(time) => Some(time_of(time)?),
        _ => return None,
    };
    let length = match &answer["duration_min"] {
        Value::Null => None,
        length => Some(u32::try_from(length.as_u64()?).ok()?).filter(|&m| m > 0),
    };
    Some((time, length))
}

/// The date the kind of day names, worked out from `today` (`MOD-10-F04`, `DEC-212`): a weekday
/// alone is the nearest one after today; next week's is that weekday of the week after this one,
/// weeks starting on Monday; a date that has passed this year is next year's.
fn day(kind: &str, value: &str, today: NaiveDate) -> Option<NaiveDate> {
    let value = value.trim().to_lowercase();
    let weekday = || WEEKDAYS.iter().position(|&w| w == value).map(|i| i as i64);
    let days = |n: i64| Some(today + Duration::days(n));
    match kind {
        "yesterday" => days(-1),
        "today" => days(0),
        "tomorrow" => days(1),
        "day_after_tomorrow" => days(2),
        "weekday" => {
            let now = today.weekday().num_days_from_monday() as i64;
            days((weekday()? - now - 1).rem_euclid(7) + 1)
        }
        "next_weekday" => {
            let now = today.weekday().num_days_from_monday() as i64;
            days(7 - now + weekday()?)
        }
        "in_days" => days(
            value
                .parse::<i64>()
                .ok()
                .filter(|n| (0..=3660).contains(n))?,
        ),
        "date" => {
            let mut parts = value.rsplit('-');
            let day: u32 = parts.next()?.parse().ok()?;
            let month: u32 = parts.next()?.parse().ok()?;
            let this_year = NaiveDate::from_ymd_opt(today.year(), month, day);
            match this_year {
                Some(date) if date >= today => Some(date),
                _ => NaiveDate::from_ymd_opt(today.year() + 1, month, day),
            }
        }
        _ => None,
    }
}

/// An hour as `HH:MM`, or a part of the day (`time_of_day`).
fn time_of(time: &str) -> Option<NaiveTime> {
    time_of_day(time).or_else(|| {
        let (h, m) = time.trim().split_once(':')?;
        NaiveTime::from_hms_opt(h.parse().ok()?, m.get(..2)?.parse().ok()?, 0)
    })
}

/// A part of the day: 9:00 in the morning, 14:00 in the afternoon, 18:00 in the evening
/// (`MOD-10-F27`, `DEC-209`).
fn time_of_day(time: &str) -> Option<NaiveTime> {
    let hour = |h| NaiveTime::from_hms_opt(h, 0, 0);
    match time.trim().to_lowercase().as_str() {
        "morning" => hour(9),
        "afternoon" => hour(14),
        "evening" => hour(18),
        _ => None,
    }
}

#[cfg(test)]
#[path = "described_tests.rs"]
mod tests;
