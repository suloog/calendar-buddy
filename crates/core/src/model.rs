use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use crate::colors::EventColor;
use crate::sync::Start;

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
    /// Its calendar's colour; the views show `shown_color`.
    pub color: Rgb,
    pub timing: Timing,
    /// Event of a calendar shared as free/busy only: no details may be shown.
    pub busy_only: bool,
    /// Event of a calendar the account cannot change: read-only, subscribed or free/busy
    /// (`MOD-02-U01`).
    pub read_only: bool,
    /// A guest's copy of someone else's event, who does not let guests change it (`MOD-05-F17`).
    pub organizer_only: bool,
    /// A new or changed event Google has not confirmed yet (`MOD-05-U03`); view state only,
    /// never cached.
    pub saving: bool,
    /// An occurrence of a series, changed or not.
    pub recurring: bool,
    /// For an occurrence of a series, changed or not: the series' ID and where the occurrence
    /// starts by the series' rule (`MOD-05-F20`).
    pub series: Option<(String, Start)>,
    pub about: About,
    /// The version of the event at the provider, which a change is sent against; empty when not
    /// known yet.
    pub etag: String,
}

/// Why the user may not change an event (`MOD-05-F17`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protected {
    /// Its calendar is read-only, subscribed or free/busy (`DEC-22`).
    ReadOnlyCalendar,
    /// A guest's copy of someone else's event, who does not let guests change it.
    OrganizerOnly,
    /// A working location or a birthday, which the app only shows (`DEC-34`).
    DisplayOnly,
}

impl Protected {
    /// What the app says when the user tries to change the event.
    pub fn why(self) -> &'static str {
        match self {
            Protected::ReadOnlyCalendar => {
                "🔒 This event can't be changed: its calendar is read-only."
            }
            Protected::OrganizerOnly => {
                "This event can't be changed: only its organizer can change it."
            }
            Protected::DisplayOnly => {
                "This event can't be changed here: working locations and birthdays are changed in \
                 Google."
            }
        }
    }
}

impl Event {
    /// The colour the views show the event in: its own (`MOD-05-F25`), or else its calendar's.
    pub fn shown_color(&self) -> Rgb {
        self.about.color.map_or(self.color, EventColor::rgb)
    }

    /// Why the user may not drag, resize, edit or delete the event; `None` when they may.
    pub fn protected(&self) -> Option<Protected> {
        if self.read_only {
            Some(Protected::ReadOnlyCalendar)
        } else if self.organizer_only {
            Some(Protected::OrganizerOnly)
        } else if self.about.kind.display_only() {
            Some(Protected::DisplayOnly)
        } else {
            None
        }
    }
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

/// Google's special event types (`MOD-05-F23`), or an ordinary event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Ordinary,
    OutOfOffice,
    FocusTime,
    WorkingLocation,
    Birthday,
}

impl Kind {
    /// What the type is called where the user picks it.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Ordinary => "Event",
            Kind::OutOfOffice => "Out of office",
            Kind::FocusTime => "Focus time",
            Kind::WorkingLocation => "Working location",
            Kind::Birthday => "Birthday",
        }
    }

    /// The mark the views put before the title of an event of the type (`MOD-05-U01`).
    pub fn icon(self) -> Option<&'static str> {
        match self {
            Kind::Ordinary => None,
            Kind::OutOfOffice => Some("🌴"),
            Kind::FocusTime => Some("🎧"),
            Kind::WorkingLocation => Some("📍"),
            Kind::Birthday => Some("🎂"),
        }
    }

    /// A type the app only shows: working locations and birthdays are made in Google (`DEC-34`).
    pub fn display_only(self) -> bool {
        matches!(self, Kind::WorkingLocation | Kind::Birthday)
    }

    /// A type Google keeps only as timed events, and always busy: out of office and focus time.
    pub fn timed_only(self) -> bool {
        matches!(self, Kind::OutOfOffice | Kind::FocusTime)
    }
}

/// What an event says besides its title and times.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct About {
    pub description: String,
    pub location: String,
    pub show: Show,
    pub visibility: Visibility,
    /// Its type; set when it is created, never changed afterwards (as in Google).
    pub kind: Kind,
    /// Its own colour, shown instead of its calendar's (`MOD-05-F25`); `None` when it has none.
    pub color: Option<EventColor>,
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
