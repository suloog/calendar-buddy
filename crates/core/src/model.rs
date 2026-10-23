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
    /// The account whose calendar list the event's calendar is on.
    pub account_id: String,
    pub calendar_id: String,
    pub title: String,
    pub color: Rgb,
    pub timing: Timing,
    /// Event of a calendar shared as free/busy only: no details may be shown.
    pub busy_only: bool,
    /// Event of a calendar the account cannot change: read-only, subscribed or free/busy
    /// (`MOD-02-U01`).
    pub read_only: bool,
    /// A new event Google has not confirmed yet (`MOD-05-U03`); view state only, never cached.
    pub saving: bool,
}

/// What the account may do with a calendar, as Google says it (`accessRole`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Owner,
    Writer,
    Reader,
    /// Only busy blocks, without event details.
    FreeBusyReader,
}

impl Access {
    /// Whether the account may change the calendar's events.
    pub fn can_edit(self) -> bool {
        matches!(self, Access::Owner | Access::Writer)
    }
}

/// A calendar an account can access: its own, shared with it or subscribed (`MOD-02-F01`).
#[derive(Clone, Debug, PartialEq)]
pub struct Calendar {
    /// The calendar's ID; the same calendar can be on the list of two accounts.
    pub id: String,
    pub account_id: String,
    pub name: String,
    /// The colour Google shows the calendar in (`MOD-02-F03`).
    pub color: Rgb,
    pub access: Access,
    /// The calendar's own time zone, for events stored without one.
    pub zone: Tz,
}
