//! The light and dark themes (`MOD-04-F01`): the one the settings choose, or the desktop's
//! (`MOD-04-F02`, `MOD-04-F03`, `DEC-57`).

use crate::texts;

/// The theme chosen in the settings; by default the desktop's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

/// How the app looks: light or dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    Light,
    Dark,
}

impl Theme {
    /// The choices in the settings, in their order.
    pub const ALL: [Theme; 3] = [Theme::System, Theme::Light, Theme::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Theme::System => texts::SYSTEM,
            Theme::Light => texts::LIGHT,
            Theme::Dark => texts::DARK,
        }
    }

    /// How the choice is kept in the settings file; the default is not kept.
    pub fn key(self) -> Option<&'static str> {
        match self {
            Theme::System => None,
            Theme::Light => Some("light"),
            Theme::Dark => Some("dark"),
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        }
    }

    /// The scheme shown, given the desktop's preference (`None` when it states none): with
    /// System the desktop's, and light when it has none (`DEC-58`).
    pub fn shown(self, desktop: Option<Scheme>) -> Scheme {
        match self {
            Theme::System => desktop.unwrap_or(Scheme::Light),
            Theme::Light => Scheme::Light,
            Theme::Dark => Scheme::Dark,
        }
    }
}

/// The desktop's preference from the XDG desktop portal's `color-scheme`: 1 prefers dark, 2
/// prefers light, and anything else, 0 included, states no preference.
pub fn from_color_scheme(value: u32) -> Option<Scheme> {
    match value {
        1 => Some(Scheme::Dark),
        2 => Some(Scheme::Light),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn with_system_the_app_shows_what_the_desktop_prefers() {
        assert_eq!(Theme::default(), Theme::System);
        assert_eq!(Theme::System.shown(Some(Scheme::Dark)), Scheme::Dark);
        assert_eq!(Theme::System.shown(Some(Scheme::Light)), Scheme::Light);
    }

    #[test]
    fn a_desktop_without_a_preference_gets_the_light_theme() {
        assert_eq!(from_color_scheme(0), None);
        assert_eq!(from_color_scheme(7), None);
        assert_eq!(Theme::System.shown(None), Scheme::Light);
    }

    #[test]
    fn the_portals_color_scheme_says_dark_or_light() {
        assert_eq!(from_color_scheme(1), Some(Scheme::Dark));
        assert_eq!(from_color_scheme(2), Some(Scheme::Light));
    }

    #[test]
    fn tc_04_02_light_or_dark_overrides_the_desktop_and_survives_a_restart() {
        let labels: Vec<_> = Theme::ALL.iter().map(|theme| theme.label()).collect();
        assert_eq!(labels, ["System", "Light", "Dark"]);

        let dir =
            std::env::temp_dir().join(format!("calendar-buddy-tc-04-02-{}", std::process::id()));
        let path = dir.join("settings");
        for (chosen, shown) in [(Theme::Light, Scheme::Light), (Theme::Dark, Scheme::Dark)] {
            let desktop = match shown {
                Scheme::Light => Scheme::Dark,
                Scheme::Dark => Scheme::Light,
            };
            assert_eq!(chosen.shown(Some(desktop)), shown);
            Settings {
                theme: chosen,
                ..Settings::default()
            }
            .save(&path)
            .unwrap();

            let restarted = Settings::load(&path);
            assert_eq!(restarted.theme, chosen);
            assert_eq!(restarted.theme.shown(Some(desktop)), shown);
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn system_is_not_written_and_an_unknown_theme_is_system() {
        assert!(!Settings::default().to_text().contains("theme"));
        assert_eq!(Settings::parse("theme = sepia").theme, Theme::System);
        for theme in Theme::ALL {
            let settings = Settings {
                theme,
                ..Settings::default()
            };
            assert_eq!(Settings::parse(&settings.to_text()), settings);
        }
    }
}
