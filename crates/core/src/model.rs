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
    /// A new or changed event Google has not confirmed yet (`MOD-05-U03`); view state only,
    /// never cached.
    pub saving: bool,
    /// An occurrence of a series, changed or not.
    pub recurring: bool,
    pub about: About,
    /// The version of the event at the provider, which a change is sent against; empty when not
    /// known yet.
    pub etag: String,
}

/// Whether an event shows the user as busy or free (`MOD-05-F07`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Show {
    #[default]
    Busy,
    Free,
}

/// Who may see an event's details (`MOD-05-F08`): as the calendar's sharing says, everyone who
/// can see the calendar, or only those who can change it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Default,
    Public,
    Private,
}

/// What an event says besides its title and times.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct About {
    pub description: String,
    pub location: String,
    pub show: Show,
    pub visibility: Visibility,
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
