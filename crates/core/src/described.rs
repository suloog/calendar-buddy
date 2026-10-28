//! What a description says (`US-10-02`): the request that asks the model to read it, and the
//! event the app makes of its answer. The model only names the kind of day from a closed list and
//! a short value; the app works the date out itself in the user's time zone (`MOD-10-F04`,
//! variant D of the test of 2026-10-27). Neither the description nor the answer is kept or logged
//! (`MOD-10-NFR-01`).

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use serde_json::{Value, json};

use crate::create;
use crate::editor::{Editor, Unresolved};
use crate::guests::{self, Person};
use crate::ollama::{KEEP_ALIVE, Ollama};

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
const RULES: &str = r#"You turn a request to add a calendar event into JSON. The request is in Polish or English; answer in the language of the request.
The user's message gives the current day and time, then the request after an empty line.
Rules:
- action: "add" when the request describes an event; "none" when it is not an event, such as a question or a greeting.
- title: a short event title in the language of the request, without the date or time. Do not translate it. "Coffee with Ben tomorrow" -> "Coffee with Ben"; "Kawa z Bartkiem jutro" -> "Kawa z Bartkiem".
- guests: first names of the people taking part, as written in the request's language, in their basic (nominative) form. Do not translate names. "with Tom" -> "Tom"; "z Tomkiem" -> "Tomek"; "z Anią" -> "Ania"; "with Anna" -> "Anna". Words for roles or groups are not guests: mom, mama, team, zespół, client, klient, board, zarząd. Empty list if no person is named.
- meet: true when Google Meet or a video call is asked for; false only when it is refused ("no Meet", "bez Meeta"); null when it is not mentioned.
- duration_min: the length in minutes when the request gives one ("45 minut" -> 45, "two hours" -> 120, "godzina i kwadrans" -> 75); null when it gives none.
- kind and day describe the day; do not compute the date:
  kind "yesterday" (wczoraj), "today" (also when no day is given), "tomorrow" (jutro), "day_after_tomorrow" (pojutrze): day "";
  kind "weekday" for a weekday alone ("w piątek", "on Friday"): day "fri" (mon tue wed thu fri sat sun);
  kind "next_weekday" ONLY when the request says next: "next Thursday", "w przyszły czwartek", "next week on Tuesday", "w przyszłym tygodniu we wtorek": day "thu" / "tue";
  kind "in_days" for "in N days" / "za N dni", "in a week" / "za tydzień": day "7";
  kind "date" for a date: day "MM-DD" ("14 listopada" -> "11-14", "3 Nov" -> "11-03").
- time: HH:MM 24-hour when an hour is given; "morning", "afternoon" or "evening" when only a part of the day is given (rano, po południu, wieczorem); null when no time is given.
Answer with compact JSON on one line, no spaces."#;

const KINDS: [&str; 8] = [
    "yesterday",
    "today",
    "tomorrow",
    "day_after_tomorrow",
    "weekday",
    "next_weekday",
    "in_days",
    "date",
];
const WEEKDAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

/// The fixed form of the answer (structured output): variant D, with whether it is an event and a
/// length left empty when none is given (`DEC-218`); Meet empty when not mentioned (`DEC-223`).
fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "action": {"type": "string", "enum": ["add", "none"]},
            "title": {"type": "string"},
            "kind": {"type": "string", "enum": KINDS},
            "day": {"type": "string"},
            "time": {"type": ["string", "null"]},
            "duration_min": {"type": ["integer", "null"]},
            "guests": {"type": "array", "items": {"type": "string"}},
            "meet": {"type": ["boolean", "null"]},
        },
        "required": ["action", "title", "kind", "day", "time", "duration_min", "guests", "meet"],
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
    /// Not an event, or an answer that cannot be used (`MOD-10-F23`).
    NoEvent,
}

/// An event to add, as the description gives it.
#[derive(Clone, Debug, PartialEq)]
pub struct Described {
    pub title: String,
    pub date: NaiveDate,
    /// `None` when the description gives no time.
    pub time: Option<NaiveTime>,
    /// In minutes; `None` when the description gives no length.
    pub length: Option<u32>,
    pub guests: Vec<String>,
    /// Google Meet asked for (`true`), refused (`false`), or not mentioned (`None`, `DEC-223`).
    pub meet: Option<bool>,
}

impl Described {
    /// Whether it opens the full editor: it names guests or asks for Meet, which the "New
    /// event" window has not (`MOD-10-F06`, `DEC-205`).
    pub fn full(&self) -> bool {
        !self.guests.is_empty() || self.meet == Some(true)
    }

    /// The times of the event: from its time on its day in `zone`, its length long, or the
    /// default length of a new event when it gives none (`MOD-10-F05`). With no time it starts as
    /// one started with the `c` key would, at the next full hour, until `US-10-05` asks for it
    /// (`DEC-219`).
    pub fn times(
        &self,
        default_length: u32,
        now: DateTime<Utc>,
        zone: Tz,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        let length = self.length.unwrap_or(default_length);
        match self.time {
            Some(time) => {
                let minute = time.signed_duration_since(NaiveTime::MIN).num_minutes();
                let start = create::wall(self.date, minute as u32, zone);
                (start, start + Duration::minutes(length.into()))
            }
            None => create::keyed(self.date, now, length, zone),
        }
    }
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
        "add" => {}
        _ => return None,
    }
    let title = answer["title"].as_str()?.trim().to_string();
    if title.is_empty() {
        return None;
    }
    let date = day(answer["kind"].as_str()?, answer["day"].as_str()?, today)?;
    let time = match &answer["time"] {
        Value::Null => None,
        Value::String(time) => Some(time_of(time)?),
        _ => return None,
    };
    let length = match &answer["duration_min"] {
        Value::Null => None,
        length => Some(u32::try_from(length.as_u64()?).ok()?).filter(|&m| m > 0),
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
    }))
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

/// An hour as `HH:MM`, or a part of the day: 9:00 in the morning, 14:00 in the afternoon, 18:00
/// in the evening (`MOD-10-F27`, `DEC-209`).
fn time_of(time: &str) -> Option<NaiveTime> {
    let hour = |h| NaiveTime::from_hms_opt(h, 0, 0);
    match time.trim().to_lowercase().as_str() {
        "morning" => hour(9),
        "afternoon" => hour(14),
        "evening" => hour(18),
        time => {
            let (h, m) = time.split_once(':')?;
            NaiveTime::from_hms_opt(h.parse().ok()?, m.get(..2)?.parse().ok()?, 0)
        }
    }
}

#[cfg(test)]
#[path = "described_tests.rs"]
mod tests;
