//! The side panel (`US-03-06`): the connected accounts, each with its calendars (`MOD-03-F10`),
//! the quick toggle that hides a calendar's events but keeps it listed (`MOD-03-F12`), and
//! whether the panel is shown (`MOD-03-F11`, `MOD-03-U03`); and the calendars disabled in the
//! settings, left out of the panel, the calendar pickers and the views (`MOD-03-F13`).

use std::collections::BTreeSet;

use crate::accounts::Entry;
use crate::colors::calendar_color;
use crate::local;
use crate::model::{Calendar, Event, Rgb};
use crate::settings::Settings;
use crate::status::{self, Status};
use crate::texts;

/// The calendars whose events are hidden by the quick toggle, by account ID and calendar ID.
pub type Hidden = BTreeSet<(String, String)>;

/// Below this window width, in points, the side panel collapses on its own (`DEC-100`).
pub const NARROW: f32 = 800.0;

/// One calendar's row in the side panel.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub calendar: Calendar,
    /// Its local colour, or else Google's.
    pub color: Rgb,
    /// Its events are in the views; off when hidden by the quick toggle.
    pub shown: bool,
    /// The account cannot change it: read-only, subscribed or free/busy (`MOD-02-U01`).
    pub read_only: bool,
    /// What keeps it from being up to date (`MOD-02-F18`, `DEC-97`).
    pub problem: Option<&'static str>,
    /// Its notifications are muted from its menu (`MOD-07-F12`, `MOD-07-U03`).
    pub muted: bool,
}

/// A connected account and, under it, its calendars.
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub email: String,
    pub calendars: Vec<Row>,
}

/// The side panel's accounts and local calendars in their order (`MOD-03-F30`), each with its
/// calendars in Google's order; calendars disabled in the settings are left out (`MOD-03-F13`),
/// and the local ones while they are hidden (`MOD-09-F22`).
pub fn groups(
    accounts: &[Entry],
    calendars: &[Calendar],
    settings: &Settings,
    status: &Status,
    online: bool,
) -> Vec<Group> {
    let rows = by_account(accounts, calendars, &settings.order, |calendar| {
        (!settings.is_disabled(&key(calendar))).then(|| Row {
            calendar: calendar.clone(),
            color: calendar_color(calendar, &settings.colors),
            shown: !settings.hidden.contains(&key(calendar)),
            read_only: !calendar.access.can_edit(),
            problem: status.problem(calendar, online).map(status::on_row),
            muted: settings.muted.contains(&key(calendar)),
        })
    });
    rows.into_iter()
        .map(|(email, calendars)| Group { email, calendars })
        .collect()
}

/// The quick toggle: hides the calendar's events, or shows them again. In the settings, the same
/// on the disabled calendars disables the calendar or enables it again (`MOD-03-F13`), and on the
/// muted ones mutes its notifications or unmutes them (`MOD-07-F12`).
pub fn toggle(hidden: &mut Hidden, calendar: &Calendar) {
    let key = key(calendar);
    if !hidden.remove(&key) {
        hidden.insert(key);
    }
}

/// The places in `events` of those whose calendar is neither hidden nor disabled, declined ones
/// left out while the switch hides them (`MOD-06-F22`).
pub fn shown(events: &[Event], settings: &Settings) -> Vec<usize> {
    (0..events.len())
        .filter(|&i| {
            let event = &events[i];
            let key = (event.account_id.clone(), event.calendar_id.clone());
            !settings.hidden.contains(&key)
                && !settings.is_disabled(&key)
                && !(settings.hide_declined && event.about.declined())
        })
        .collect()
}

/// The settings' list of calendars: each connected account's e-mail address with all its
/// calendars in Google's order, each with whether it is enabled (`MOD-03-F13`).
pub fn in_settings(
    accounts: &[Entry],
    calendars: &[Calendar],
    settings: &Settings,
) -> Vec<(String, Vec<(Calendar, bool)>)> {
    by_account(accounts, calendars, &settings.order, |calendar| {
        Some((
            calendar.clone(),
            !settings.disabled.contains(&key(calendar)),
        ))
    })
}

/// The calendars a calendar picker starts from (`MOD-05-F10`): under their accounts, like the side
/// panel (`DEC-28`), those disabled in the settings left out (`MOD-03-F13`). Which of them a
/// picker offers, such as only those the account can change, is the picker's own.
pub fn pickable(
    accounts: &[Entry],
    calendars: &[Calendar],
    settings: &Settings,
) -> Vec<(String, Vec<Calendar>)> {
    by_account(accounts, calendars, &settings.order, |calendar| {
        (!settings.is_disabled(&key(calendar))).then(|| calendar.clone())
    })
}

/// The groups in their order, each with what `item` gives for its calendars: the accounts, and
/// "Local" with the local calendars when `item` gives any of them (`MOD-09-U02`, `MOD-09-F22`).
fn by_account<T>(
    accounts: &[Entry],
    calendars: &[Calendar],
    order: &[String],
    item: impl Fn(&Calendar) -> Option<T>,
) -> Vec<(String, Vec<T>)> {
    let of = |account_id: &str| -> Vec<T> {
        (calendars.iter())
            .filter(|calendar| calendar.account_id == account_id)
            .filter_map(&item)
            .collect()
    };
    let names = |id: &str| -> String {
        (accounts.iter())
            .find(|entry| entry.account.id == id)
            .map_or_else(
                || texts::LOCAL.to_string(),
                |entry| entry.account.email.clone(),
            )
    };
    (ordered(accounts, calendars, order).into_iter())
        .map(|id| (names(&id), of(&id), id))
        .filter(|(_, items, id)| !items.is_empty() || id != local::ACCOUNT_ID)
        .map(|(name, items, _)| (name, items))
        .collect()
}

/// The IDs of the side panel's groups in their order (`MOD-03-F30`): those `order` names as it
/// names them, then the others — the accounts in the order they were connected, then the local
/// calendars (`local::ACCOUNT_ID`) if there are any (`DEC-189`).
pub fn ordered(accounts: &[Entry], calendars: &[Calendar], order: &[String]) -> Vec<String> {
    let local = (calendars.iter())
        .any(|calendar| calendar.account_id == local::ACCOUNT_ID)
        .then(|| local::ACCOUNT_ID.to_string());
    let mut ids: Vec<String> = (accounts.iter())
        .map(|entry| entry.account.id.clone())
        .chain(local)
        .collect();
    ids.sort_by_key(|id| {
        order
            .iter()
            .position(|named| named == id)
            .unwrap_or(usize::MAX)
    });
    ids
}

/// The groups `ids`, in their order, with the one at `at` moved one place up or down
/// (`MOD-03-U09`); the order to keep.
pub fn moved(ids: &[String], at: usize, up: bool) -> Vec<String> {
    let mut ids = ids.to_vec();
    match up {
        true if at > 0 && at < ids.len() => ids.swap(at, at - 1),
        false if at + 1 < ids.len() => ids.swap(at, at + 1),
        _ => {}
    }
    ids
}

/// Every calendar there is, is hidden or disabled: the view is empty and points to the side
/// panel. Before the calendars are known nothing is said.
pub fn nothing_shown(calendars: &[Calendar], settings: &Settings) -> bool {
    !calendars.is_empty()
        && calendars.iter().all(|calendar| {
            let key = key(calendar);
            settings.hidden.contains(&key) || settings.is_disabled(&key)
        })
}

fn key(calendar: &Calendar) -> (String, String) {
    (calendar.account_id.clone(), calendar.id.clone())
}

/// Whether the side panel is shown: the user's choice, remembered across restarts
/// (`MOD-03-F11`), and collapsed on its own while the window is too narrow (`MOD-03-U03`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Panel {
    /// The user's choice, kept in the settings.
    collapsed: bool,
    narrow: bool,
    /// Expanded by the user while the window is narrow.
    opened_narrow: bool,
}

impl Panel {
    pub fn new(collapsed: bool) -> Self {
        Panel {
            collapsed,
            narrow: false,
            opened_narrow: false,
        }
    }

    /// The window's width, in points, as it is now; a window made wide again brings the panel
    /// back as the user left it.
    pub fn resized(&mut self, width: f32) {
        self.narrow = width < NARROW;
        if !self.narrow {
            self.opened_narrow = false;
        }
    }

    pub fn expanded(&self) -> bool {
        !self.collapsed && (!self.narrow || self.opened_narrow)
    }

    /// The user collapses or expands the panel; gives the choice to remember.
    pub fn toggle(&mut self) -> bool {
        if self.expanded() {
            self.collapsed = true;
            self.opened_narrow = false;
        } else {
            self.collapsed = false;
            self.opened_narrow = self.narrow;
        }
        self.collapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{Account, AccountState};
    use crate::fixtures::{PALETTE, d, timed};
    use crate::model::Access;
    use crate::status::Problem;
    use crate::view_model::grid_view;
    use crate::week::GridView;
    use chrono::{TimeZone, Utc, Weekday};
    use chrono_tz::Europe::Warsaw;

    fn entry(id: &str, email: &str) -> Entry {
        Entry {
            account: Account {
                id: id.to_string(),
                email: email.to_string(),
                workspace: false,
                contacts: false,
                directory: false,
            },
            state: AccountState::Connected,
        }
    }

    fn calendar(account: &str, name: &str, access: Access) -> Calendar {
        Calendar {
            id: name.to_lowercase(),
            account_id: account.to_string(),
            name: name.to_string(),
            color: PALETTE[0],
            access,
            zone: Warsaw,
            default_reminders: Vec::new(),
        }
    }

    fn event(account: &str, calendar: &str, day: u32) -> Event {
        let mut event = timed(calendar, d(2026, 11, day), (9, 0), (10, 0));
        event.account_id = account.to_string();
        event.calendar_id = calendar.to_lowercase();
        event
    }

    fn names(groups: &[Group]) -> Vec<(String, Vec<(String, bool)>)> {
        groups
            .iter()
            .map(|group| {
                let rows = group.calendars.iter();
                let rows = rows.map(|row| (row.calendar.name.clone(), row.shown));
                (group.email.clone(), rows.collect())
            })
            .collect()
    }

    #[test]
    fn tc_03_09_calendars_are_grouped_by_account_and_the_quick_toggle_hides_only_the_events() {
        let accounts = [entry("a", "a@example.com"), entry("b", "b@example.com")];
        let calendars = [
            calendar("a", "Work", Access::Owner),
            calendar("b", "Home", Access::Owner),
            calendar("a", "Team", Access::Writer),
        ];
        let events = [
            event("a", "Work", 10),
            event("a", "Team", 11),
            event("b", "Home", 12),
        ];
        let mut settings = Settings::default();
        // The week view of Wednesday 11 November 2026, 10:00 in Warsaw.
        let week = |settings: &Settings| {
            let shown: Vec<Event> = shown(&events, settings)
                .into_iter()
                .map(|i| events[i].clone())
                .collect();
            let days = GridView::Week.days(d(2026, 11, 11), Weekday::Mon);
            let now = Utc.with_ymd_and_hms(2026, 11, 11, 9, 0, 0).unwrap();
            let view = grid_view(&shown, &days, Warsaw, now);
            let mut titles: Vec<String> = view
                .blocks
                .iter()
                .map(|block| shown[block.placement.event].title.clone())
                .collect();
            titles.sort();
            titles
        };
        let panel = |settings: &Settings| {
            names(&groups(
                &accounts,
                &calendars,
                settings,
                &Status::default(),
                true,
            ))
        };
        let listed = |team: bool| {
            vec![
                (
                    "a@example.com".to_string(),
                    vec![("Work".to_string(), true), ("Team".to_string(), team)],
                ),
                (
                    "b@example.com".to_string(),
                    vec![("Home".to_string(), true)],
                ),
            ]
        };

        // 1. Account A with Work and Team under it, account B with Home.
        assert_eq!(panel(&settings), listed(true));
        assert_eq!(week(&settings), ["Home", "Team", "Work"]);

        // 2. The quick toggle of Team: it stays listed, its event leaves the week.
        toggle(&mut settings.hidden, &calendars[2]);
        assert_eq!(panel(&settings), listed(false));
        assert_eq!(week(&settings), ["Home", "Work"]);

        // 3. Toggled again, the event is back.
        toggle(&mut settings.hidden, &calendars[2]);
        assert_eq!(panel(&settings), listed(true));
        assert_eq!(week(&settings), ["Home", "Team", "Work"]);
    }

    #[test]
    fn tc_03_10_a_disabled_calendar_leaves_the_panel_the_pickers_and_the_views_until_enabled() {
        let accounts = [entry("a", "a@example.com")];
        let calendars = [
            calendar("a", "Work", Access::Owner),
            calendar("a", "Birthdays", Access::Reader),
        ];
        let events = [event("a", "Work", 10), event("a", "Birthdays", 12)];
        let mut settings = Settings::default();
        // The week view of Wednesday 11 November 2026, 10:00 in Warsaw.
        let week = |settings: &Settings| {
            let shown: Vec<Event> = shown(&events, settings)
                .into_iter()
                .map(|i| events[i].clone())
                .collect();
            let days = GridView::Week.days(d(2026, 11, 11), Weekday::Mon);
            let now = Utc.with_ymd_and_hms(2026, 11, 11, 9, 0, 0).unwrap();
            let view = grid_view(&shown, &days, Warsaw, now);
            let mut titles: Vec<String> = view
                .blocks
                .iter()
                .map(|block| shown[block.placement.event].title.clone())
                .collect();
            titles.sort();
            titles
        };
        let panel = |settings: &Settings| {
            let groups = groups(&accounts, &calendars, settings, &Status::default(), true);
            names(&groups)[0]
                .1
                .iter()
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>()
        };
        let picker = |settings: &Settings| {
            pickable(&accounts, &calendars, settings)[0]
                .1
                .iter()
                .map(|calendar| calendar.name.clone())
                .collect::<Vec<_>>()
        };
        let in_settings = |settings: &Settings| {
            in_settings(&accounts, &calendars, settings)
                .into_iter()
                .map(|(email, rows)| {
                    let rows = rows.into_iter().map(|(c, enabled)| (c.name, enabled));
                    (email, rows.collect::<Vec<_>>())
                })
                .collect::<Vec<_>>()
        };
        let listed = |birthdays: bool| {
            vec![(
                "a@example.com".to_string(),
                vec![
                    ("Work".to_string(), true),
                    ("Birthdays".to_string(), birthdays),
                ],
            )]
        };
        assert_eq!(in_settings(&settings), listed(true));
        assert_eq!(panel(&settings), ["Work", "Birthdays"]);
        assert_eq!(picker(&settings), ["Work", "Birthdays"]);
        assert_eq!(week(&settings), ["Birthdays", "Work"]);

        // 1. Disabled in the settings: still in the settings' list, gone from everything else.
        toggle(&mut settings.disabled, &calendars[1]);
        assert_eq!(in_settings(&settings), listed(false));
        assert_eq!(panel(&settings), ["Work"]);
        assert_eq!(picker(&settings), ["Work"]);
        assert_eq!(week(&settings), ["Work"]);

        // 2. Enabled again: back in the side panel, the picker and the week.
        toggle(&mut settings.disabled, &calendars[1]);
        assert_eq!(in_settings(&settings), listed(true));
        assert_eq!(panel(&settings), ["Work", "Birthdays"]);
        assert_eq!(picker(&settings), ["Work", "Birthdays"]);
        assert_eq!(week(&settings), ["Birthdays", "Work"]);
    }

    #[test]
    fn calendars_the_account_cannot_change_are_marked_and_disabled_ones_left_out() {
        let accounts = [entry("a", "a@example.com")];
        let calendars = [
            calendar("a", "Work", Access::Owner),
            calendar("a", "Team", Access::Writer),
            calendar("a", "Holidays", Access::Reader),
            calendar("a", "Room", Access::FreeBusyReader),
            calendar("a", "Old", Access::Owner),
        ];
        let mut settings = Settings::default();
        settings.disabled.insert(("a".into(), "old".into()));

        let groups = groups(&accounts, &calendars, &settings, &Status::default(), true);

        let read_only: Vec<(&str, bool)> = groups[0]
            .calendars
            .iter()
            .map(|row| (row.calendar.name.as_str(), row.read_only))
            .collect();
        assert_eq!(
            read_only,
            [
                ("Work", false),
                ("Team", false),
                ("Holidays", true),
                ("Room", true)
            ]
        );
    }

    #[test]
    fn a_calendars_problem_and_local_colour_are_on_its_row() {
        let accounts = [entry("a", "a@example.com")];
        let calendars = [
            calendar("a", "Work", Access::Owner),
            calendar("a", "Home", Access::Owner),
        ];
        let green = Rgb(0x0b, 0x80, 0x43);
        let mut settings = Settings::default();
        settings.colors.insert(("a".into(), "home".into()), green);
        let status = Status {
            problems: vec![
                (calendars[0].clone(), Problem::Failed),
                (calendars[1].clone(), Problem::NotSynchronizing),
            ],
            ..Status::default()
        };

        let row = |online| {
            groups(&accounts, &calendars, &settings, &status, online)[0]
                .calendars
                .iter()
                .map(|row| (row.color, row.problem))
                .collect::<Vec<_>>()
        };

        let not_synchronizing = Some("Not synchronizing: its account needs sign-in.");
        assert_eq!(
            row(true),
            [
                (PALETTE[0], Some("Could not be synchronized; trying again.")),
                (green, not_synchronizing),
            ]
        );
        assert_eq!(row(false), [(PALETTE[0], None), (green, not_synchronizing)]);
    }

    #[test]
    fn an_empty_view_points_to_the_side_panel_once_every_calendar_is_hidden_or_disabled() {
        let calendars = [
            calendar("a", "Work", Access::Owner),
            calendar("b", "Home", Access::Owner),
        ];
        let mut settings = Settings::default();
        assert!(!nothing_shown(&[], &settings));
        assert!(!nothing_shown(&calendars, &settings));

        toggle(&mut settings.hidden, &calendars[0]);
        assert!(!nothing_shown(&calendars, &settings));
        settings.disabled.insert(("b".into(), "home".into()));
        assert!(nothing_shown(&calendars, &settings));
        toggle(&mut settings.hidden, &calendars[0]);
        assert!(!nothing_shown(&calendars, &settings));
    }

    fn three_groups() -> ([Entry; 2], [Calendar; 3]) {
        let accounts = [entry("a", "a@example.com"), entry("b", "b@example.com")];
        let calendars = [
            calendar("a", "a@example.com", Access::Owner),
            calendar("b", "Home", Access::Owner),
            calendar(local::ACCOUNT_ID, "Mine", Access::Owner),
        ];
        (accounts, calendars)
    }

    fn emails(accounts: &[Entry], calendars: &[Calendar], settings: &Settings) -> Vec<String> {
        let groups = groups(accounts, calendars, settings, &Status::default(), true);
        groups.into_iter().map(|group| group.email).collect()
    }

    #[test]
    fn tc_03_15_the_groups_are_moved_up_and_down_and_follow_the_order_everywhere() {
        let (accounts, calendars) = three_groups();
        // 1. As connected, "Local" last; the first does not move up, the last not down.
        let mut settings = Settings::default();
        let ids = ordered(&accounts, &calendars, &settings.order);
        assert_eq!(ids, ["a", "b", "local"]);
        assert_eq!(moved(&ids, 0, true), ids);
        assert_eq!(moved(&ids, 2, false), ids);
        // 2. "Local" moved up twice: first in the panel, the pickers and the settings.
        settings.order = moved(&moved(&ids, 2, true), 1, true);
        assert_eq!(settings.order, ["local", "a", "b"]);
        let local_a_b = ["Local", "a@example.com", "b@example.com"];
        assert_eq!(emails(&accounts, &calendars, &settings), local_a_b);
        let picked = pickable(&accounts, &calendars, &settings);
        assert_eq!(
            picked.iter().map(|(e, _)| e.as_str()).collect::<Vec<_>>(),
            local_a_b
        );
        let listed = in_settings(&accounts, &calendars, &settings);
        assert_eq!(
            listed.iter().map(|(e, _)| e.as_str()).collect::<Vec<_>>(),
            local_a_b
        );
        // 3. "b" moved up: "Local", "b", "a".
        settings.order = moved(&settings.order, 2, true);
        assert_eq!(
            emails(&accounts, &calendars, &settings),
            ["Local", "b@example.com", "a@example.com"]
        );
        // 4. Kept across a restart.
        let restarted = Settings::parse(&settings.to_text());
        assert_eq!(
            emails(&accounts, &calendars, &restarted),
            emails(&accounts, &calendars, &settings)
        );
    }

    #[test]
    fn tc_03_16_new_and_removed_accounts_fit_the_order_and_the_default_does_not_follow_it() {
        let (accounts, calendars) = three_groups();
        let mut settings = Settings::default();
        // 1–2. "b" first; a new event still starts in "a"'s primary calendar.
        let ids = ordered(&accounts, &calendars, &settings.order);
        settings.order = moved(&ids, 1, true);
        assert_eq!(settings.order, ["b", "a", "local"]);
        let default = crate::create::default_calendar(&accounts, &calendars, &settings);
        assert_eq!(default.map(|c| c.id), Some("a@example.com".to_string()));
        // 3. "c" connected: last.
        let more = [
            accounts[0].clone(),
            accounts[1].clone(),
            entry("c", "c@example.com"),
        ];
        assert_eq!(
            ordered(&more, &calendars, &settings.order),
            ["b", "a", "local", "c"]
        );
        // 4. "a" removed: "b", "Local", "c".
        assert!(settings.forget_account("a"));
        let left = [more[1].clone(), more[2].clone()];
        assert_eq!(
            ordered(&left, &calendars, &settings.order),
            ["b", "local", "c"]
        );
    }

    #[test]
    fn hidden_local_calendars_are_out_of_the_panel_the_pickers_and_the_views() {
        let accounts = [entry("a", "a@example.com")];
        let mine = calendar(local::ACCOUNT_ID, "Mine", Access::Owner);
        let calendars = [calendar("a", "Work", Access::Owner), mine.clone()];
        let events = [event("a", "Work", 12), event(local::ACCOUNT_ID, "Mine", 12)];
        let settings = Settings {
            local_hidden: true,
            ..Settings::default()
        };
        let groups = groups(&accounts, &calendars, &settings, &Status::default(), true);
        assert_eq!(
            names(&groups),
            [(
                "a@example.com".to_string(),
                vec![("Work".to_string(), true)]
            )]
        );
        assert_eq!(pickable(&accounts, &calendars, &settings).len(), 1);
        assert_eq!(shown(&events, &settings), [0]);
        // Still listed in the settings, as they were.
        let listed = in_settings(&accounts, &calendars, &settings);
        assert_eq!(listed[1], ("Local".to_string(), vec![(mine, true)]));
        // Shown again: back as they were.
        assert_eq!(shown(&events, &Settings::default()), [0, 1]);
    }

    #[test]
    fn the_panel_collapses_while_the_window_is_narrow_and_comes_back_as_the_user_left_it() {
        let mut panel = Panel::new(false);
        panel.resized(1200.0);
        assert!(panel.expanded());

        panel.resized(NARROW - 1.0);
        assert!(!panel.expanded());
        panel.resized(NARROW);
        assert!(panel.expanded());

        // Opened by hand while narrow, it stays open; the choice is to keep it expanded.
        panel.resized(600.0);
        assert!(!panel.toggle());
        assert!(panel.expanded());
        panel.resized(1200.0);
        panel.resized(600.0);
        assert!(!panel.expanded());

        // Collapsed by the user, it stays collapsed when the window widens.
        panel.resized(1200.0);
        assert!(panel.toggle());
        assert!(!panel.expanded());
        panel.resized(600.0);
        panel.resized(1200.0);
        assert!(!panel.expanded());
        assert!(!panel.toggle());
        assert!(panel.expanded());
    }

    #[test]
    fn a_panel_collapsed_by_the_user_starts_collapsed() {
        let mut panel = Panel::new(true);
        panel.resized(1200.0);
        assert!(!panel.expanded());
    }
}
