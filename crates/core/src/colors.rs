//! A calendar's local colour (`MOD-02-F04`): chosen in the app, kept in its settings and never
//! sent to Google; without one the calendar keeps the colour Google gives it.

use std::collections::BTreeMap;

use crate::model::{Calendar, Event, Rgb};

/// Local colours by account ID and calendar ID (`DEC-99`).
pub type LocalColors = BTreeMap<(String, String), Rgb>;

/// The colours Google Calendar offers in a calendar's menu, with their names.
pub const CALENDAR_COLORS: [(&str, Rgb); 24] = [
    ("Radicchio", Rgb(0xAD, 0x14, 0x57)),
    ("Tangerine", Rgb(0xF4, 0x51, 0x1E)),
    ("Citron", Rgb(0xE4, 0xC4, 0x41)),
    ("Basil", Rgb(0x0B, 0x80, 0x43)),
    ("Blueberry", Rgb(0x3F, 0x51, 0xB5)),
    ("Grape", Rgb(0x8E, 0x24, 0xAA)),
    ("Cherry blossom", Rgb(0xD8, 0x1B, 0x60)),
    ("Pumpkin", Rgb(0xEF, 0x6C, 0x00)),
    ("Avocado", Rgb(0xC0, 0xCA, 0x33)),
    ("Eucalyptus", Rgb(0x00, 0x96, 0x88)),
    ("Lavender", Rgb(0x79, 0x86, 0xCB)),
    ("Cocoa", Rgb(0x79, 0x55, 0x48)),
    ("Tomato", Rgb(0xD5, 0x00, 0x00)),
    ("Mango", Rgb(0xF0, 0x93, 0x00)),
    ("Pistachio", Rgb(0x7C, 0xB3, 0x42)),
    ("Peacock", Rgb(0x03, 0x9B, 0xE5)),
    ("Wisteria", Rgb(0xB3, 0x9D, 0xDB)),
    ("Graphite", Rgb(0x61, 0x61, 0x61)),
    ("Flamingo", Rgb(0xE6, 0x7C, 0x73)),
    ("Banana", Rgb(0xF6, 0xBF, 0x26)),
    ("Sage", Rgb(0x33, 0xB6, 0x79)),
    ("Cobalt", Rgb(0x42, 0x85, 0xF4)),
    ("Amethyst", Rgb(0x9E, 0x69, 0xAF)),
    ("Birch", Rgb(0xA7, 0x9B, 0x8E)),
];

/// The colour the app shows the calendar in: its local colour, or else Google's.
pub fn calendar_color(calendar: &Calendar, local: &LocalColors) -> Rgb {
    local
        .get(&(calendar.account_id.clone(), calendar.id.clone()))
        .copied()
        .unwrap_or(calendar.color)
}

/// The events, in the same order, each in its calendar's local colour where it has one.
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
    fn the_palette_has_googles_24_calendar_colours_once_each() {
        let mut colors: Vec<Rgb> = CALENDAR_COLORS.iter().map(|(_, rgb)| *rgb).collect();
        colors.sort_by_key(|Rgb(r, g, b)| (*r, *g, *b));
        colors.dedup();
        assert_eq!(colors.len(), 24);
    }
}
