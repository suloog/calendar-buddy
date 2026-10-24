//! Running in the background (`US-07-05`, `DEC-44`): closing the window leaves the app in the
//! system tray, where it keeps synchronizing and reminding; clicking the tray icon shows or hides
//! the window, its menu opens the window or quits the app, the app can start at login in the
//! tray, and launching the app again brings the running window forward.

use std::path::Path;

/// What the user did to the app's window, its tray icon or its launcher.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    /// The window was closed.
    Closed,
    /// The tray icon was clicked (`MOD-07-U02`).
    TrayClicked,
    /// "Open" was chosen in the tray menu (`MOD-07-F15`).
    Open,
    /// "Quit" was chosen in the tray menu (`MOD-07-F15`).
    Quit,
    /// The app was launched again while it runs (`MOD-07-F17`).
    Launched,
}

/// What the app does about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    /// Opens the window.
    Show,
    /// Brings the open window forward.
    Raise,
    /// Closes the window and stays in the tray, synchronizing and reminding (`MOD-07-F14`).
    Hide,
    /// Ends the app, and its reminders with it.
    Quit,
}

/// What the app does when `signal` comes while its window is `shown` or not; `tray` is whether
/// the desktop shows the app's tray icon. Without one, closing the window quits the app, as there
/// would be nothing left to open it from.
pub fn respond(signal: Signal, shown: bool, tray: bool) -> Response {
    match signal {
        Signal::Closed if tray => Response::Hide,
        Signal::Closed | Signal::Quit => Response::Quit,
        Signal::TrayClicked if shown => Response::Hide,
        Signal::TrayClicked => Response::Show,
        Signal::Open | Signal::Launched if shown => Response::Raise,
        Signal::Open | Signal::Launched => Response::Show,
    }
}

/// The argument the app is started with at login, to start in the tray (`MOD-07-F16`).
pub const IN_TRAY: &str = "--in-tray";

/// Whether the app was started with `args` at login.
pub fn at_login(args: &[String]) -> bool {
    args.iter().any(|arg| arg == IN_TRAY)
}

/// Whether the app opens its window when started with `args`: always, unless started at login
/// with a tray icon to open it from.
pub fn starts_shown(args: &[String], tray: bool) -> bool {
    !(tray && at_login(args))
}

/// The name of the setting that starts the app at login (`MOD-07-F16`).
pub const START_AT_LOGIN: &str = "Start at login";
pub const START_AT_LOGIN_HINT: &str = "Calendar Buddy starts in the system tray when you log in, \
                                       so reminders arrive without opening it.";

/// The XDG autostart entry that starts the app at `exe` in the tray at login.
pub fn autostart_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Calendar Buddy\nIcon=x-office-calendar\n\
         Exec={} {IN_TRAY}\n",
        exec_argument(&exe.to_string_lossy())
    )
}

/// The argument quoted for an entry's `Exec` key: the characters the desktop entry
/// specification reserves inside quotes take a backslash, which is itself written doubled as the
/// key's value escapes it again; a percent sign is doubled, not to be read as a field code.
fn exec_argument(argument: &str) -> String {
    let mut quoted = String::from("\"");
    for c in argument.chars() {
        match c {
            '"' | '`' | '$' => quoted.extend(['\\', '\\', c]),
            '\\' => quoted.push_str("\\\\\\\\"),
            '%' => quoted.push_str("%%"),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tc_07_06_launching_the_app_again_brings_its_window_forward_or_opens_it_from_the_tray() {
        assert_eq!(respond(Signal::Launched, true, true), Response::Raise);
        assert_eq!(respond(Signal::Launched, false, true), Response::Show);
        assert_eq!(respond(Signal::Launched, false, false), Response::Show);
    }

    #[test]
    fn closing_the_window_leaves_the_app_in_the_tray_or_quits_it_without_one() {
        assert_eq!(respond(Signal::Closed, true, true), Response::Hide);
        assert_eq!(respond(Signal::Closed, true, false), Response::Quit);
    }

    #[test]
    fn clicking_the_tray_icon_shows_or_hides_the_window() {
        assert_eq!(respond(Signal::TrayClicked, true, true), Response::Hide);
        assert_eq!(respond(Signal::TrayClicked, false, true), Response::Show);
    }

    #[test]
    fn the_tray_menu_opens_the_window_and_quits_the_app() {
        assert_eq!(respond(Signal::Open, false, true), Response::Show);
        assert_eq!(respond(Signal::Open, true, true), Response::Raise);
        assert_eq!(respond(Signal::Quit, false, true), Response::Quit);
        assert_eq!(respond(Signal::Quit, true, true), Response::Quit);
    }

    #[test]
    fn the_app_starts_in_the_tray_only_at_login_and_with_a_tray_icon() {
        let login = ["calendar-buddy".to_string(), IN_TRAY.to_string()];
        let launched = ["calendar-buddy".to_string()];
        assert!(at_login(&login));
        assert!(!at_login(&launched));
        assert!(!starts_shown(&login, true));
        assert!(starts_shown(&login, false));
        assert!(starts_shown(&launched, true));
    }

    #[test]
    fn the_autostart_entry_starts_the_app_in_the_tray() {
        let entry = autostart_entry(Path::new("/usr/bin/calendar-buddy"));
        assert_eq!(
            entry,
            "[Desktop Entry]\nType=Application\nName=Calendar Buddy\nIcon=x-office-calendar\n\
             Exec=\"/usr/bin/calendar-buddy\" --in-tray\n"
        );
    }

    #[test]
    fn the_autostart_entry_quotes_the_reserved_characters_of_the_apps_path() {
        let entry = autostart_entry(Path::new("/home/a b/$x\"`\\/100%/calendar-buddy"));
        let exec = entry
            .lines()
            .find(|line| line.starts_with("Exec="))
            .unwrap();
        assert_eq!(
            exec,
            r#"Exec="/home/a b/\\$x\\"\\`\\\\/100%%/calendar-buddy" --in-tray"#
        );
    }
}
