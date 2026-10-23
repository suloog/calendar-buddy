//! The app's own settings, kept in a small text file of `key = value` lines. Losing them is
//! acceptable (`DEC-15`): a missing or unreadable file gives the defaults.

use std::path::{Path, PathBuf};

use crate::navigation::View;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// The view used last; the app starts in it (`DEC-55`).
    pub last_view: Option<View>,
}

impl Settings {
    pub fn parse(text: &str) -> Self {
        let mut settings = Settings::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() == "view" {
                settings.last_view = View::from_key(value.trim());
            }
        }
        settings
    }

    pub fn to_text(&self) -> String {
        match self.last_view {
            Some(view) => format!("view = {}\n", view.key()),
            None => String::new(),
        }
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

/// `$XDG_CONFIG_HOME/calendar-buddy/settings`, or under `~/.config` when that is not set.
pub fn default_path() -> Option<PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(config.join("calendar-buddy").join("settings"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;
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
            };
            assert_eq!(Settings::parse(&settings.to_text()), settings);
        }
    }

    #[test]
    fn a_missing_or_unknown_setting_gives_the_default() {
        assert_eq!(Settings::parse(""), Settings::default());
        assert_eq!(Settings::parse("view = year\nnoise"), Settings::default());
        assert_eq!(
            Settings::load(Path::new("/nonexistent/calendar-buddy/settings")),
            Settings::default()
        );
    }
}
