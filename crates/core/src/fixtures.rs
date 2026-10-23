//! Deterministic test data: small scenarios for tests.

use chrono::{Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::{Europe::Warsaw, Tz};

use crate::model::{Event, Rgb, Timing};

pub const PALETTE: [Rgb; 10] = [
    Rgb(0x79, 0x86, 0xCB),
    Rgb(0x33, 0xB6, 0x79),
    Rgb(0x8E, 0x24, 0xAA),
    Rgb(0xE6, 0x7C, 0x73),
    Rgb(0xF6, 0xBF, 0x26),
    Rgb(0xF4, 0x51, 0x1E),
    Rgb(0x03, 0x9B, 0xE5),
    Rgb(0x61, 0x61, 0x61),
    Rgb(0x3F, 0x51, 0xB5),
    Rgb(0x0B, 0x80, 0x43),
];

pub fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// A timed event in Europe/Warsaw; hours of 24 and more fall on the following days.
pub fn timed(id: &str, date: NaiveDate, start: (u32, u32), end: (u32, u32)) -> Event {
    timed_in(id, date, start, end, Warsaw)
}

pub fn timed_in(id: &str, date: NaiveDate, start: (u32, u32), end: (u32, u32), zone: Tz) -> Event {
    let at = |(h, m): (u32, u32)| {
        let local = date.and_time(NaiveTime::MIN) + Duration::minutes((h * 60 + m) as i64);
        zone.from_local_datetime(&local)
            .earliest()
            .expect("fixture time exists")
            .with_timezone(&Utc)
    };
    Event {
        id: id.to_string(),
        calendar_id: "work".to_string(),
        title: id.to_string(),
        color: PALETTE[0],
        timing: Timing::Timed {
            start: at(start),
            end: at(end),
            zone,
        },
        busy_only: false,
    }
}

pub fn all_day(id: &str, start: NaiveDate, days: i64) -> Event {
    Event {
        id: id.to_string(),
        calendar_id: "work".to_string(),
        title: id.to_string(),
        color: PALETTE[1],
        timing: Timing::AllDay {
            start,
            end_exclusive: start + Duration::days(days),
        },
        busy_only: false,
    }
}

pub fn all_day_pair() -> Vec<Event> {
    vec![
        all_day("trip", d(2026, 11, 12), 3),
        all_day("holiday", d(2026, 11, 13), 1),
    ]
}

/// The week of 9–15 November 2026 from `TC-03-01`.
pub fn tc_03_01_week() -> Vec<Event> {
    let mut busy = timed("Room booking", d(2026, 11, 13), (13, 0), (14, 0));
    busy.busy_only = true;
    busy.calendar_id = "room".to_string();
    vec![
        timed("Standup", d(2026, 11, 9), (9, 0), (10, 0)),
        all_day("Conference day", d(2026, 11, 10), 1),
        all_day("Offsite", d(2026, 11, 12), 3),
        busy,
    ]
}
