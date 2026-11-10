//! The real platform behind `calendar_core::platform`, one for each system the app runs on:
//! Linux (`DEC-02`) and macOS (`DEC-217`).

use std::path::PathBuf;

use calendar_core::guests::Response;
use calendar_core::reminders::{Action, SNOOZES};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{
    SystemAppearance, SystemBrowser, SystemInstance, SystemNetwork, SystemNotifier,
    SystemSecretStore, SystemTray, SystemWakeups, XdgAutostart as SystemAutostart,
    XdgLauncher as SystemLauncher, XdgLinkHandler as SystemLinkHandler,
};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{
    SystemAppearance, SystemAutostart, SystemBrowser, SystemInstance, SystemLauncher,
    SystemLinkHandler, SystemNetwork, SystemNotifier, SystemSecretStore, SystemTray, SystemWakeups,
    in_dock,
};

/// Whether closing the window hides it while the app stays in the menu bar, rather than ending
/// it: on macOS the system's event loop has to keep running for the menu bar icon and the
/// notifications' buttons (`DEC-218`). On Linux the window ends and opens again.
pub const WINDOW_HIDES: bool = cfg!(target_os = "macos");

/// Whether the app is in the Dock, as it is while its window is shown (`DEC-218`); nothing on
/// Linux.
#[cfg(target_os = "linux")]
pub fn in_dock(_shown: bool) {}

/// The app's icon, as the window and the desktop's icon theme take it.
pub const ICON: &[u8] = include_bytes!("../../assets/icon/calendar-buddy.png");

/// The program that runs; one replaced while it runs, as by an upgrade, is named "… (deleted)".
fn running_exe() -> std::io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let exe = exe.to_string_lossy();
    Ok(PathBuf::from(
        exe.strip_suffix(" (deleted)").unwrap_or(&exe),
    ))
}

/// The key of a notification's action, the same on both systems; "default" is clicking the
/// notification's body.
fn key(action: Action) -> String {
    match action {
        Action::Open => "default".to_string(),
        Action::Join => "join".to_string(),
        Action::Snooze(minutes) => format!("snooze-{minutes}"),
        Action::Dismiss => "dismiss".to_string(),
        Action::Answer(Response::Accepted) => "accept".to_string(),
        Action::Answer(Response::Maybe) => "maybe".to_string(),
        Action::Answer(Response::Declined) => "decline".to_string(),
        Action::Answer(Response::Awaiting) => "awaiting".to_string(),
    }
}

/// The action of the key; `None` for a key the app did not give.
fn action(key: &str) -> Option<Action> {
    match key {
        "default" => Some(Action::Open),
        "join" => Some(Action::Join),
        "dismiss" => Some(Action::Dismiss),
        "accept" => Some(Action::Answer(Response::Accepted)),
        "maybe" => Some(Action::Answer(Response::Maybe)),
        "decline" => Some(Action::Answer(Response::Declined)),
        _ => key
            .strip_prefix("snooze-")
            .and_then(|minutes| minutes.parse().ok())
            .filter(|minutes| SNOOZES.contains(minutes))
            .map(Action::Snooze),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_action_of_a_notification_comes_back_from_its_key() {
        let actions = [
            Action::Open,
            Action::Join,
            Action::Snooze(5),
            Action::Snooze(10),
            Action::Dismiss,
            Action::Answer(Response::Accepted),
            Action::Answer(Response::Maybe),
            Action::Answer(Response::Declined),
        ];
        for chosen in actions {
            assert_eq!(action(&key(chosen)), Some(chosen));
        }
        assert_eq!(key(Action::Open), "default");
        assert_eq!(action("snooze-later"), None);
        assert_eq!(action("snooze-7"), None);
        assert_eq!(action("other"), None);
    }
}
