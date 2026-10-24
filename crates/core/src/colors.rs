//! A calendar's local colour (`MOD-02-F04`): chosen in the app, kept in its settings and never
//! sent to Google; without one the calendar keeps the colour Google gives it. An event's own
//! colour (`MOD-05-F25`) is one of Google's event colours, and is shown instead of its
//! calendar's.

use std::collections::BTreeMap;

use crate::model::{Calendar, Event, Rgb};
use crate::texts;

/// Local colours by account ID and calendar ID (`DEC-99`).
pub type LocalColors = BTreeMap<(String, String), Rgb>;

/// The colours Google Calendar offers in a calendar's menu, with their names.
pub const CALENDAR_COLORS: [(&str, Rgb); 24] = [
    (texts::RADICCHIO, Rgb(0xAD, 0x14, 0x57)),
    (texts::TANGERINE, Rgb(0xF4, 0x51, 0x1E)),
    (texts::CITRON, Rgb(0xE4, 0xC4, 0x41)),
    (texts::BASIL, Rgb(0x0B, 0x80, 0x43)),
    (texts::BLUEBERRY, Rgb(0x3F, 0x51, 0xB5)),
    (texts::GRAPE, Rgb(0x8E, 0x24, 0xAA)),
    (texts::CHERRY_BLOSSOM, Rgb(0xD8, 0x1B, 0x60)),
    (texts::PUMPKIN, Rgb(0xEF, 0x6C, 0x00)),
    (texts::AVOCADO, Rgb(0xC0, 0xCA, 0x33)),
    (texts::EUCALYPTUS, Rgb(0x00, 0x96, 0x88)),
    (texts::LAVENDER, Rgb(0x79, 0x86, 0xCB)),
    (texts::COCOA, Rgb(0x79, 0x55, 0x48)),
    (texts::TOMATO, Rgb(0xD5, 0x00, 0x00)),
    (texts::MANGO, Rgb(0xF0, 0x93, 0x00)),
    (texts::PISTACHIO, Rgb(0x7C, 0xB3, 0x42)),
    (texts::PEACOCK, Rgb(0x03, 0x9B, 0xE5)),
    (texts::WISTERIA, Rgb(0xB3, 0x9D, 0xDB)),
    (texts::GRAPHITE, Rgb(0x61, 0x61, 0x61)),
    (texts::FLAMINGO, Rgb(0xE6, 0x7C, 0x73)),
    (texts::BANANA, Rgb(0xF6, 0xBF, 0x26)),
    (texts::SAGE, Rgb(0x33, 0xB6, 0x79)),
    (texts::COBALT, Rgb(0x42, 0x85, 0xF4)),
    (texts::AMETHYST, Rgb(0x9E, 0x69, 0xAF)),
    (texts::BIRCH, Rgb(0xA7, 0x9B, 0x8E)),
];

/// Google's event colours, as Google Calendar names and shows them (`DEC-118`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventColor {
    Tomato,
    Flamingo,
    Tangerine,
    Banana,
    Sage,
    Basil,
    Peacock,
    Blueberry,
    Lavender,
    Grape,
    Graphite,
}

impl EventColor {
    /// Every event colour, in the order Google Calendar's event menu offers them.
    pub const ALL: [EventColor; 11] = [
        EventColor::Tomato,
        EventColor::Flamingo,
        EventColor::Tangerine,
        EventColor::Banana,
        EventColor::Sage,
        EventColor::Basil,
        EventColor::Peacock,
        EventColor::Blueberry,
        EventColor::Lavender,
        EventColor::Grape,
        EventColor::Graphite,
    ];

    pub fn name(self) -> &'static str {
        match self {
            EventColor::Tomato => texts::TOMATO,
            EventColor::Flamingo => texts::FLAMINGO,
            EventColor::Tangerine => texts::TANGERINE,
            EventColor::Banana => texts::BANANA,
            EventColor::Sage => texts::SAGE,
            EventColor::Basil => texts::BASIL,
            EventColor::Peacock => texts::PEACOCK,
            EventColor::Blueberry => texts::BLUEBERRY,
            EventColor::Lavender => texts::LAVENDER,
            EventColor::Grape => texts::GRAPE,
            EventColor::Graphite => texts::GRAPHITE,
        }
    }

    /// The colour Google Calendar shows it in.
    pub fn rgb(self) -> Rgb {
        match self {
            EventColor::Tomato => Rgb(0xD5, 0x00, 0x00),
            EventColor::Flamingo => Rgb(0xE6, 0x7C, 0x73),
            EventColor::Tangerine => Rgb(0xF4, 0x51, 0x1E),
            EventColor::Banana => Rgb(0xF6, 0xBF, 0x26),
            EventColor::Sage => Rgb(0x33, 0xB6, 0x79),
            EventColor::Basil => Rgb(0x0B, 0x80, 0x43),
            EventColor::Peacock => Rgb(0x03, 0x9B, 0xE5),
            EventColor::Blueberry => Rgb(0x3F, 0x51, 0xB5),
            EventColor::Lavender => Rgb(0x79, 0x86, 0xCB),
            EventColor::Grape => Rgb(0x8E, 0x24, 0xAA),
            EventColor::Graphite => Rgb(0x61, 0x61, 0x61),
        }
    }
}

/// The colour the app shows the calendar in: its local colour, or else Google's.
pub fn calendar_color(calendar: &Calendar, local: &LocalColors) -> Rgb {
    local
        .get(&(calendar.account_id.clone(), calendar.id.clone()))
        .copied()
        .unwrap_or(calendar.color)
}

/// The events, in the same order, each in its calendar's local colour where it has one; an
/// event's own colour is shown over either (`Event::shown_color`).
pub fn apply(events: &[Event], local: &LocalColors) -> Vec<Event> {
    events
        .iter()
        .map(|event| {
            let key = (event.account_id.clone(), event.calendar_id.clone());
            Event {
                color: local.get(&key).copied().unwrap_or(event.color),
                ..event.clone()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{PALETTE, d, timed};
    use crate::model::Access;

    fn calendar(account: &str, id: &str) -> Calendar {
        Calendar {
            id: id.to_string(),
            account_id: account.to_string(),
            name: id.to_string(),
            color: PALETTE[0],
            access: Access::Owner,
            zone: chrono_tz::Europe::Warsaw,
            default_reminders: Vec::new(),
        }
    }

    #[test]
    fn a_local_colour_replaces_googles_for_its_calendar_on_that_account_only() {
        let green = CALENDAR_COLORS[3].1;
        let local = LocalColors::from([(("1".to_string(), "work".to_string()), green)]);
        let mut other_account = timed("Other account", d(2026, 11, 10), (9, 0), (10, 0));
        other_account.account_id = "2".to_string();
        let mut other_calendar = timed("Other calendar", d(2026, 11, 10), (9, 0), (10, 0));
        other_calendar.calendar_id = "home".to_string();
        let events = vec![
            timed("Standup", d(2026, 11, 10), (9, 0), (10, 0)),
            other_account,
            other_calendar,
        ];

        let shown = apply(&events, &local);

        let colors: Vec<Rgb> = shown.iter().map(|event| event.color).collect();
        assert_eq!(colors, [green, PALETTE[0], PALETTE[0]]);
        assert_eq!(shown[0].title, "Standup");
        assert_eq!(calendar_color(&calendar("1", "work"), &local), green);
        assert_eq!(calendar_color(&calendar("2", "work"), &local), PALETTE[0]);
    }

    #[test]
    fn event_colours_are_googles_eleven_named_as_in_google_calendar() {
        let colors: Vec<(&str, Rgb)> = EventColor::ALL
            .iter()
            .map(|color| (color.name(), color.rgb()))
            .collect();
        assert_eq!(
            colors,
            [
                (texts::TOMATO, Rgb(0xD5, 0x00, 0x00)),
                (texts::FLAMINGO, Rgb(0xE6, 0x7C, 0x73)),
                (texts::TANGERINE, Rgb(0xF4, 0x51, 0x1E)),
                (texts::BANANA, Rgb(0xF6, 0xBF, 0x26)),
                (texts::SAGE, Rgb(0x33, 0xB6, 0x79)),
                (texts::BASIL, Rgb(0x0B, 0x80, 0x43)),
                (texts::PEACOCK, Rgb(0x03, 0x9B, 0xE5)),
                (texts::BLUEBERRY, Rgb(0x3F, 0x51, 0xB5)),
                (texts::LAVENDER, Rgb(0x79, 0x86, 0xCB)),
                (texts::GRAPE, Rgb(0x8E, 0x24, 0xAA)),
                (texts::GRAPHITE, Rgb(0x61, 0x61, 0x61)),
            ]
        );
    }

    #[test]
    fn an_events_own_colour_is_shown_over_its_calendars_local_one() {
        let local = LocalColors::from([(("1".to_string(), "work".to_string()), PALETTE[1])]);
        let mut own = timed("Dentist", d(2026, 11, 12), (16, 0), (17, 0));
        own.about.color = Some(EventColor::Tomato);
        let plain = timed("Standup", d(2026, 11, 12), (9, 0), (10, 0));

        let shown: Vec<Rgb> = apply(&[own, plain], &local)
            .iter()
            .map(Event::shown_color)
            .collect();

        assert_eq!(shown, [EventColor::Tomato.rgb(), PALETTE[1]]);
    }

    #[test]
    fn the_palette_has_googles_24_calendar_colours_once_each() {
        let mut colors: Vec<Rgb> = CALENDAR_COLORS.iter().map(|(_, rgb)| *rgb).collect();
        colors.sort_by_key(|Rgb(r, g, b)| (*r, *g, *b));
        colors.dedup();
        assert_eq!(colors.len(), 24);
    }
}
