use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// When an event happens. Timed events are stored as UTC instants plus the zone they were
/// created in; all-day events are calendar dates, the end being exclusive (as Google stores them).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Timing {
    Timed {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        zone: Tz,
    },
    AllDay {
        start: NaiveDate,
        end_exclusive: NaiveDate,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub id: String,
    pub calendar_id: String,
    pub title: String,
    pub color: Rgb,
    pub timing: Timing,
    /// Event of a calendar shared as free/busy only: no details may be shown.
    pub busy_only: bool,
}
