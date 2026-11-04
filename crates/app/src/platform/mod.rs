//! The real platform behind `calendar_core::platform`, one for each system the app runs on:
//! Linux (`DEC-02`) and macOS (`DEC-217`).

use std::path::PathBuf;

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
};

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
