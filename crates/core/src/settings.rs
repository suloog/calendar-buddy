//! The app's own settings, kept in a small text file of `key = value` lines. Losing them is
//! acceptable (`DEC-15`): a missing or unreadable file gives the defaults.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::cache::{color_text, parse_color};
use crate::colors::LocalColors;
use crate::navigation::View;
use crate::side_panel::Hidden;
use crate::theme::Theme;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// The view used last; the app starts in it (`DEC-55`).
    pub last_view: Option<View>,
    /// The calendars disabled in the settings, by account ID and calendar ID: listed, but never
    /// synchronized (`MOD-02-F06`, `MOD-03-F13`).
    pub disabled: BTreeSet<(String, String)>,
    /// The calendars' local colours, never sent to Google (`MOD-02-F04`).
    pub colors: LocalColors,
    /// The calendars whose events the side panel's quick toggle hides (`MOD-03-F12`).
    pub hidden: Hidden,
    /// The side panel was collapsed by the user (`MOD-03-F11`).
    pub side_panel_collapsed: bool,
    /// The calendar new events go to, by account ID and calendar ID, once the user chose one
    /// (`MOD-05-F10`).
    pub default_calendar: Option<(String, String)>,
    /// The switch in the views hides declined events; they are shown by default (`MOD-06-F22`).
    pub hide_declined: bool,
    /// The switch in the settings turned all notifications off; they are on by default
    /// (`MOD-07-F11`).
    pub notifications_off: bool,
    /// The calendars whose notifications are muted from the side panel, by account ID and
    /// calendar ID; their events are still shown (`MOD-07-F12`).
    pub muted: BTreeSet<(String, String)>,
    /// The theme chosen in the settings; by default the desktop's (`MOD-04-F03`).
    pub theme: Theme,
}

impl Settings {
    pub fn parse(text: &str) -> Self {
        let mut settings = Settings::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "view" => settings.last_view = View::from_key(value.trim()),
                "disabled" => {
                    if let Some(calendar) = calendar(value) {
                        settings.disabled.insert(calendar);
                    }
                }
                "hidden" => {
                    if let Some(calendar) = calendar(value) {
                        settings.hidden.insert(calendar);
                    }
                }
                "side_panel" => settings.side_panel_collapsed = value.trim() == "collapsed",
                "default_calendar" => settings.default_calendar = calendar(value),
                "declined" => settings.hide_declined = value.trim() == "hidden",
                "notifications" => settings.notifications_off = value.trim() == "off",
                "muted" => {
                    if let Some(calendar) = calendar(value) {
                        settings.muted.insert(calendar);
                    }
                }
                "theme" => settings.theme = Theme::from_key(value.trim()),
                "color" => {
                    let value = value.trim();
                    let local = value.split_once(' ').and_then(|(account, rest)| {
                        let (calendar, color) = rest.trim().rsplit_once(' ')?;
                        Some((account, calendar.trim(), parse_color(color)?))
                    });
                    if let Some((account, calendar, color)) = local {
                        settings
                            .colors
                            .insert((account.to_string(), calendar.to_string()), color);
                    }
                }
                _ => {}
            }
        }
        settings
    }

    pub fn to_text(&self) -> String {
        let mut text = match self.last_view {
            Some(view) => format!("view = {}\n", view.key()),
            None => String::new(),
        };
        for (account, calendar) in &self.disabled {
            text.push_str(&format!("disabled = {account} {calendar}\n"));
        }
        for ((account, calendar), color) in &self.colors {
            let color = color_text(*color);
            text.push_str(&format!("color = {account} {calendar} {color}\n"));
        }
        for (account, calendar) in &self.hidden {
            text.push_str(&format!("hidden = {account} {calendar}\n"));
        }
        if self.side_panel_collapsed {
            text.push_str("side_panel = collapsed\n");
        }
        if let Some((account, calendar)) = &self.default_calendar {
            text.push_str(&format!("default_calendar = {account} {calendar}\n"));
        }
        if self.hide_declined {
            text.push_str("declined = hidden\n");
        }
        if self.notifications_off {
            text.push_str("notifications = off\n");
        }
        for (account, calendar) in &self.muted {
            text.push_str(&format!("muted = {account} {calendar}\n"));
        }
        if let Some(theme) = self.theme.key() {
            text.push_str(&format!("theme = {theme}\n"));
        }
        text
    }

    /// Forgets what the settings hold for a removed account (`DEC-85`); says whether they held
    /// anything.
    pub fn forget_account(&mut self, account_id: &str) -> bool {
        let before = self.clone();
        self.disabled.retain(|(account, _)| account != account_id);
        self.colors.retain(|(account, _), _| account != account_id);
        self.hidden.retain(|(account, _)| account != account_id);
        self.muted.retain(|(account, _)| account != account_id);
        if self
            .default_calendar
            .as_ref()
            .is_some_and(|(account, _)| account == account_id)
        {
            self.default_calendar = None;
        }
        *self != before
    }

    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .map(|text| Settings::parse(&text))
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, self.to_text())
    }
}

/// `<account ID> <calendar ID>`.
fn calendar(value: &str) -> Option<(String, String)> {
    let (account, calendar) = value.trim().split_once(' ')?;
    Some((account.to_string(), calendar.trim().to_string()))
}

/// `$XDG_CONFIG_HOME/calendar-buddy/settings`, or under `~/.config` when that is not set.
pub fn default_path() -> Option<PathBuf> {
    Some(config_dir()?.join("settings"))
}

/// `$XDG_CONFIG_HOME/calendar-buddy`, or `~/.config/calendar-buddy` when that is not set.
pub fn config_dir() -> Option<PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(config.join("calendar-buddy"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;
    use crate::model::Rgb;
    use crate::navigation::Navigation;
    use crate::week::GridView;

    #[test]
    fn tc_03_08_the_last_view_survives_a_restart() {
        let dir =
            std::env::temp_dir().join(format!("calendar-buddy-tc-03-08-{}", std::process::id()));
        let path = dir.join("settings");
        let mut nav = Navigation::start(
            Settings::load(&path).last_view,
            d(2026, 11, 11),
            chrono::Weekday::Mon,
        );
        nav.switch(View::Grid(GridView::ThreeDays));
        nav.next();
        nav.next();
        Settings {
            last_view: Some(nav.view),
            ..Settings::default()
        }
        .save(&path)
        .unwrap();

        let restarted = Navigation::start(
            Settings::load(&path).last_view,
            d(2026, 11, 11),
            chrono::Weekday::Mon,
        );
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(restarted.view, View::Grid(GridView::ThreeDays));
        assert_eq!(restarted.selected, d(2026, 11, 11));
    }

    #[test]
    fn every_view_is_saved_and_read_back() {
        for view in View::ALL {
            let settings = Settings {
                last_view: Some(view),
                ..Settings::default()
            };
            assert_eq!(Settings::parse(&settings.to_text()), settings);
        }
    }

    #[test]
    fn disabled_calendars_are_saved_read_back_and_forgotten_with_their_account() {
        let mut settings = Settings::default();
        for (account, calendar) in [
            ("1", "en.polish#holiday@group.v.calendar.google.com"),
            ("1", "team@example.com"),
            ("2", "team@example.com"),
        ] {
            settings
                .disabled
                .insert((account.to_string(), calendar.to_string()));
        }

        assert_eq!(Settings::parse(&settings.to_text()), settings);

        assert!(settings.forget_account("1"));
        assert!(!settings.forget_account("1"));
        assert_eq!(
            settings.disabled.into_iter().collect::<Vec<_>>(),
            [("2".to_string(), "team@example.com".to_string())]
        );
    }

    #[test]
    fn local_colours_are_saved_read_back_and_forgotten_with_their_account() {
        let mut settings = Settings::default();
        for (account, calendar, color) in [
            (
                "1",
                "en.polish#holiday@group.v.calendar.google.com",
                Rgb(0x0b, 0x80, 0x43),
            ),
            ("1", "team@example.com", Rgb(0xad, 0x14, 0x57)),
            ("2", "team@example.com", Rgb(0x03, 0x9b, 0xe5)),
        ] {
            settings
                .colors
                .insert((account.to_string(), calendar.to_string()), color);
        }

        assert!(
            settings
                .to_text()
                .contains("color = 1 en.polish#holiday@group.v.calendar.google.com #0b8043\n")
        );
        assert_eq!(Settings::parse(&settings.to_text()), settings);

        assert!(settings.forget_account("1"));
        assert!(!settings.forget_account("1"));
        assert_eq!(
            settings.colors.into_iter().collect::<Vec<_>>(),
            [(
                ("2".to_string(), "team@example.com".to_string()),
                Rgb(0x03, 0x9b, 0xe5)
            )]
        );
    }

    #[test]
    fn the_default_calendar_is_saved_read_back_and_forgotten_with_its_account() {
        let mut settings = Settings {
            default_calendar: Some(("1".to_string(), "team@example.com".to_string())),
            ..Settings::default()
        };

        assert!(
            settings
                .to_text()
                .contains("default_calendar = 1 team@example.com\n")
        );
        assert_eq!(Settings::parse(&settings.to_text()), settings);

        assert!(!settings.forget_account("2"));
        assert!(settings.forget_account("1"));
        assert_eq!(settings.default_calendar, None);
    }

    #[test]
    fn hidden_calendars_and_the_collapsed_side_panel_survive_a_restart() {
        let dir =
            std::env::temp_dir().join(format!("calendar-buddy-us-03-06-{}", std::process::id()));
        let path = dir.join("settings");
        let mut settings = Settings::default();
        for (account, calendar) in [("1", "team@example.com"), ("2", "team@example.com")] {
            settings
                .hidden
                .insert((account.to_string(), calendar.to_string()));
        }
        settings.side_panel_collapsed = true;
        settings.save(&path).unwrap();

        let restarted = Settings::load(&path);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(restarted, settings);
        assert!(settings.to_text().contains("hidden = 1 team@example.com\n"));
        assert!(settings.to_text().contains("side_panel = collapsed\n"));

        assert!(settings.forget_account("1"));
        assert!(!settings.forget_account("1"));
        assert_eq!(
            settings.hidden.into_iter().collect::<Vec<_>>(),
            [("2".to_string(), "team@example.com".to_string())]
        );
        assert!(!Settings::parse("side_panel = expanded").side_panel_collapsed);
    }

    #[test]
    fn declined_events_are_shown_by_default_and_hiding_them_survives_a_restart() {
        assert!(!Settings::default().hide_declined);
        assert!(!Settings::default().to_text().contains("declined"));
        let settings = Settings {
            hide_declined: true,
            ..Settings::default()
        };

        assert!(settings.to_text().contains("declined = hidden\n"));
        assert_eq!(Settings::parse(&settings.to_text()), settings);
        assert!(!Settings::parse("declined = shown").hide_declined);
    }

    #[test]
    fn notifications_off_and_muted_calendars_survive_a_restart_and_go_with_their_account() {
        assert!(!Settings::default().notifications_off);
        let mut settings = Settings {
            notifications_off: true,
            ..Settings::default()
        };
        for (account, calendar) in [("1", "team@example.com"), ("2", "team@example.com")] {
            settings
                .muted
                .insert((account.to_string(), calendar.to_string()));
        }

        assert!(settings.to_text().contains("notifications = off\n"));
        assert!(settings.to_text().contains("muted = 1 team@example.com\n"));
        assert_eq!(Settings::parse(&settings.to_text()), settings);
        assert!(!Settings::parse("notifications = on").notifications_off);

        assert!(settings.forget_account("1"));
        assert!(!settings.forget_account("1"));
        assert_eq!(
            settings.muted.into_iter().collect::<Vec<_>>(),
            [("2".to_string(), "team@example.com".to_string())]
        );
    }

    #[test]
    fn a_missing_or_unknown_setting_gives_the_default() {
        assert_eq!(Settings::parse(""), Settings::default());
        assert_eq!(
            Settings::parse(
                "view = year\nnoise\ndisabled = 1\nhidden = 1\nmuted = 1\ncolor = 1 work\ncolor = 1 work green"
            ),
            Settings::default()
        );
        assert_eq!(
            Settings::load(Path::new("/nonexistent/calendar-buddy/settings")),
            Settings::default()
        );
    }
}
