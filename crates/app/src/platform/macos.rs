//! The real platform on macOS behind `calendar_core::platform` (`DEC-217`): tokens in the
//! Keychain, the default browser and notifications through the system's tools, starting at login
//! through a launch agent and the app as a bundle in `~/Applications`. There is no tray yet: the
//! app ends when its window closes. Tests never use the Keychain; they use fakes, or a folder of
//! their own.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use calendar_core::accounts::Account;
use calendar_core::background::{self, Signal};
use calendar_core::platform::{
    Appearance, Autostart, Browser, Instance, LinkHandler, Network, Notifier, SecretStore,
    SecretStoreError, Tray, Wakeups,
};
use calendar_core::reminders::{Action, Notification};
use calendar_core::texts;
use calendar_core::theme::Scheme;
use security_framework::passwords;

use super::{ICON, running_exe};

/// The app's identifier on macOS: its bundle, launch agent and running copy; a debug build is
/// another app, with an identifier of its own (`DEC-164`).
const BUNDLE_ID: &str = if background::DEV {
    "pl.ordergroup.CalendarBuddyDev"
} else {
    "pl.ordergroup.CalendarBuddy"
};

/// The user's home directory; `None` when it is not known.
fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
}

/// Writes `contents` to `path` only when it changed, creating its folder; through a new file put
/// in its place, so a program that runs from it keeps its own copy.
fn write_if_changed(path: &Path, contents: &[u8], executable: bool) -> std::io::Result<()> {
    if std::fs::read(path).is_ok_and(|kept| kept == contents) {
        return Ok(());
    }
    let Some(dir) = path.parent() else {
        return Err(std::io::ErrorKind::NotFound.into());
    };
    std::fs::create_dir_all(dir)?;
    let written = dir.join(format!(
        ".{}.new",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    std::fs::write(&written, contents)?;
    if executable {
        std::fs::set_permissions(&written, std::fs::Permissions::from_mode(0o755))?;
    }
    std::fs::rename(written, path)
}

/// The text for an XML property list's string.
fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// A property list holding the dictionary of `entries`, each a key and its XML value.
fn property_list(entries: &[(&str, String)]) -> String {
    let mut list = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n<dict>\n",
    );
    for (key, value) in entries {
        list.push_str(&format!("\t<key>{key}</key>\n\t{value}\n"));
    }
    list.push_str("</dict>\n</plist>\n");
    list
}

fn string(text: &str) -> String {
    format!("<string>{}</string>", escaped(text))
}

/// The Keychain's status for an item it does not hold.
const ITEM_NOT_FOUND: i32 = -25300;
/// The Keychain's statuses for the user declining to unlock it or to let the app read an item.
const USER_CANCELED: i32 = -128;
const AUTH_FAILED: i32 = -25293;

/// The user's login Keychain, as generic passwords of the app's service, one for each account
/// (`DEC-47`). The Keychain is always there.
pub struct SystemSecretStore;

fn failed(error: security_framework::base::Error) -> SecretStoreError {
    SecretStoreError::Failed(match error.code() {
        USER_CANCELED | AUTH_FAILED => texts::UNLOCKING_DECLINED.to_string(),
        _ => error.to_string(),
    })
}

impl SecretStore for SystemSecretStore {
    fn check(&self) -> Result<(), SecretStoreError> {
        Ok(())
    }

    fn save(&self, account: &Account, refresh_token: &str) -> Result<(), SecretStoreError> {
        passwords::set_generic_password(background::APP_ID, &account.id, refresh_token.as_bytes())
            .map_err(failed)
    }

    fn load(&self, account_id: &str) -> Result<Option<String>, SecretStoreError> {
        match passwords::get_generic_password(background::APP_ID, account_id) {
            Ok(secret) => String::from_utf8(secret)
                .map(Some)
                .map_err(|_| SecretStoreError::Failed(texts::SECRET_NOT_READ.into())),
            Err(error) if error.code() == ITEM_NOT_FOUND => Ok(None),
            Err(error) => Err(failed(error)),
        }
    }

    fn take(&self, account_id: &str) -> Result<Option<String>, SecretStoreError> {
        let refresh_token = passwords::get_generic_password(background::APP_ID, account_id)
            .ok()
            .and_then(|secret| String::from_utf8(secret).ok());
        match passwords::delete_generic_password(background::APP_ID, account_id) {
            Err(error) if error.code() != ITEM_NOT_FOUND => Err(failed(error)),
            _ => Ok(refresh_token),
        }
    }
}

/// The default browser, through `open`.
pub struct SystemBrowser;

impl Browser for SystemBrowser {
    fn open(&self, url: &str) -> std::io::Result<()> {
        let mut child = Command::new("open").arg(url).spawn()?;
        std::thread::spawn(move || child.wait());
        Ok(())
    }
}

/// How often the system's light or dark appearance is read again, to follow its changes.
const APPEARANCE_CHECK: std::time::Duration = std::time::Duration::from_secs(5);

/// The system's light or dark appearance, from the global `AppleInterfaceStyle` default: "Dark"
/// while it is dark, absent while it is light.
pub struct SystemAppearance;

fn system_scheme() -> Scheme {
    let style = Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output();
    match style {
        Ok(output) if String::from_utf8_lossy(&output.stdout).trim() == "Dark" => Scheme::Dark,
        _ => Scheme::Light,
    }
}

impl Appearance for SystemAppearance {
    fn watch(&self, changed: Box<dyn Fn(Option<Scheme>) + Send + Sync>) {
        std::thread::spawn(move || {
            let mut last = None;
            loop {
                let now = system_scheme();
                if last != Some(now) {
                    changed(Some(now));
                    last = Some(now);
                }
                std::thread::sleep(APPEARANCE_CHECK);
            }
        });
    }
}

/// The system's notifications, through AppleScript's `display notification`; they have no
/// buttons and say nothing back when clicked.
pub struct SystemNotifier;

impl SystemNotifier {
    pub fn new() -> Self {
        SystemNotifier
    }
}

impl Notifier for SystemNotifier {
    fn show(&self, notification: &Notification, _chosen: Box<dyn Fn(Action) + Send + Sync>) {
        // The texts go as the script's arguments, so nothing in them is read as AppleScript.
        let shown = Command::new("osascript")
            .args([
                "-e",
                "on run argv",
                "-e",
                "display notification (item 2 of argv) with title (item 1 of argv)",
                "-e",
                "end run",
                notification.title.as_str(),
                &notification.body(),
            ])
            .spawn();
        if let Ok(mut child) = shown {
            std::thread::spawn(move || child.wait());
        }
    }

    fn buttons(&self) -> bool {
        false
    }
}

/// Waking from sleep and the network's return are not watched yet; synchronization keeps its
/// interval.
pub struct SystemWakeups;

impl Wakeups for SystemWakeups {
    fn watch(&self, _woke: Box<dyn Fn() + Send + Sync>) {}
}

/// The network is not watched yet: the app counts as online, and a failed synchronization says
/// so as it does.
pub struct SystemNetwork;

impl Network for SystemNetwork {
    fn watch(&self, _changed: Box<dyn Fn(bool) + Send + Sync>) {}
}

/// No icon in the menu bar yet: the app ends when its window closes.
pub struct SystemTray;

impl SystemTray {
    pub fn new() -> Self {
        SystemTray
    }
}

impl Tray for SystemTray {
    fn show(&self, _chosen: Box<dyn Fn(Signal) + Send + Sync>, _at_login: bool) -> bool {
        false
    }
}

/// The one copy of the app is kept by the system: opening the app while it runs brings its
/// window forward.
pub struct SystemInstance;

impl SystemInstance {
    pub fn new() -> Self {
        SystemInstance
    }
}

impl Instance for SystemInstance {
    fn claim(&self, _launched: Box<dyn Fn() + Send + Sync>) -> bool {
        true
    }
}

/// `webcal:` links are not handled yet.
pub struct SystemLinkHandler;

impl SystemLinkHandler {
    pub fn new() -> Self {
        SystemLinkHandler
    }
}

impl LinkHandler for SystemLinkHandler {
    fn register(&self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Starting at login through a launch agent in `~/Library/LaunchAgents`, which starts the program
/// it was turned on from.
pub struct SystemAutostart {
    /// The agent's file; `None` when there is no home directory.
    agent: Option<PathBuf>,
}

impl SystemAutostart {
    pub fn new() -> Self {
        SystemAutostart::at(home().map(|home| home.join("Library/LaunchAgents")))
    }

    fn at(agents: Option<PathBuf>) -> Self {
        SystemAutostart {
            agent: agents.map(|dir| dir.join(format!("{BUNDLE_ID}.plist"))),
        }
    }
}

/// The launch agent that starts the app at `exe` at login.
fn launch_agent(exe: &Path) -> String {
    property_list(&[
        ("Label", string(BUNDLE_ID)),
        (
            "ProgramArguments",
            format!(
                "<array>{}{}</array>",
                string(&exe.to_string_lossy()),
                string(background::IN_TRAY)
            ),
        ),
        ("RunAtLoad", "<true/>".into()),
    ])
}

impl Autostart for SystemAutostart {
    fn is_on(&self) -> bool {
        self.agent.as_ref().is_some_and(|agent| agent.exists())
    }

    fn set(&self, on: bool) -> std::io::Result<()> {
        let Some(agent) = &self.agent else {
            return Err(std::io::ErrorKind::NotFound.into());
        };
        if on {
            write_if_changed(agent, launch_agent(&running_exe()?).as_bytes(), false)
        } else {
            match std::fs::remove_file(agent) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                removed => removed,
            }
        }
    }
}

/// The bundle's property list.
fn info_plist() -> String {
    property_list(&[
        ("CFBundleIdentifier", string(BUNDLE_ID)),
        ("CFBundleName", string(texts::APP_NAME)),
        ("CFBundleDisplayName", string(texts::APP_NAME)),
        ("CFBundleExecutable", string(background::APP_ID)),
        ("CFBundleIconFile", string("AppIcon")),
        ("CFBundlePackageType", string("APPL")),
        (
            "CFBundleShortVersionString",
            string(env!("CARGO_PKG_VERSION")),
        ),
        ("CFBundleVersion", string(env!("CARGO_PKG_VERSION"))),
        (
            "LSApplicationCategoryType",
            string("public.app-category.productivity"),
        ),
        ("NSHighResolutionCapable", "<true/>".into()),
    ])
}

/// The icon as an Apple icon image: one 256×256 PNG image, the `ic08` kind.
fn icns(png: &[u8]) -> Vec<u8> {
    let entry = 8 + png.len() as u32;
    let mut icns = Vec::with_capacity(8 + entry as usize);
    icns.extend_from_slice(b"icns");
    icns.extend_from_slice(&(8 + entry).to_be_bytes());
    icns.extend_from_slice(b"ic08");
    icns.extend_from_slice(&entry.to_be_bytes());
    icns.extend_from_slice(png);
    icns
}

/// The app in Launchpad, Spotlight and the Dock: a bundle in `~/Applications` holding a copy of
/// the program that runs, with the app's icon.
pub struct SystemLauncher {
    /// The bundle; `None` when there is no home directory.
    bundle: Option<PathBuf>,
}

impl SystemLauncher {
    pub fn new() -> Self {
        SystemLauncher::at(home().map(|home| home.join("Applications")))
    }

    fn at(applications: Option<PathBuf>) -> Self {
        SystemLauncher {
            bundle: applications.map(|dir| dir.join(format!("{}.app", texts::APP_NAME))),
        }
    }

    /// Writes the bundle's files, each only when it changed; run from the bundle, the program is
    /// already its own copy.
    pub fn install(&self) -> std::io::Result<()> {
        let Some(bundle) = &self.bundle else {
            return Err(std::io::ErrorKind::NotFound.into());
        };
        let contents = bundle.join("Contents");
        write_if_changed(&contents.join("Info.plist"), info_plist().as_bytes(), false)?;
        write_if_changed(&contents.join("Resources/AppIcon.icns"), &icns(ICON), false)?;
        let exe = running_exe()?;
        let copy = contents.join("MacOS").join(background::APP_ID);
        if exe != copy {
            write_if_changed(&copy, &std::fs::read(&exe)?, true)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("calendar-buddy-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn starting_at_login_is_off_until_turned_on_and_off_again_when_turned_off() {
        let dir = folder("launch-agent");
        let autostart = SystemAutostart::at(Some(dir.clone()));
        assert!(!autostart.is_on());

        autostart.set(true).unwrap();
        assert!(autostart.is_on());
        let agent = std::fs::read_to_string(dir.join(format!("{BUNDLE_ID}.plist"))).unwrap();
        assert_eq!(agent, launch_agent(&running_exe().unwrap()));
        assert!(agent.contains(&format!("<key>Label</key>\n\t<string>{BUNDLE_ID}</string>")));
        assert!(agent.contains(&format!("<string>{}</string></array>", background::IN_TRAY)));
        assert!(agent.contains("<key>RunAtLoad</key>\n\t<true/>"));

        autostart.set(false).unwrap();
        assert!(!autostart.is_on());
        autostart.set(false).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_bundle_is_installed_with_the_program_and_its_icon_and_kept_up_to_date() {
        let dir = folder("bundle");
        let launcher = SystemLauncher::at(Some(dir.clone()));
        launcher.install().unwrap();

        let contents = dir.join(format!("{}.app/Contents", texts::APP_NAME));
        let info = std::fs::read_to_string(contents.join("Info.plist")).unwrap();
        assert!(info.contains(&format!(
            "<key>CFBundleIdentifier</key>\n\t<string>{BUNDLE_ID}</string>"
        )));
        assert!(info.contains(&format!(
            "<key>CFBundleExecutable</key>\n\t<string>{}</string>",
            background::APP_ID
        )));
        let copy = contents.join("MacOS").join(background::APP_ID);
        assert_eq!(
            std::fs::read(&copy).unwrap(),
            std::fs::read(running_exe().unwrap()).unwrap()
        );
        assert_eq!(
            std::fs::metadata(&copy).unwrap().permissions().mode() & 0o777,
            0o755
        );
        let icon = contents.join("Resources/AppIcon.icns");
        assert_eq!(std::fs::read(&icon).unwrap(), icns(ICON));

        // A file that changed is put back; the others stay as they are.
        std::fs::write(contents.join("Info.plist"), "old").unwrap();
        let icon_written = std::fs::metadata(&icon).unwrap().modified().unwrap();
        launcher.install().unwrap();
        assert_eq!(
            std::fs::read_to_string(contents.join("Info.plist")).unwrap(),
            info
        );
        assert_eq!(
            std::fs::metadata(&icon).unwrap().modified().unwrap(),
            icon_written
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_icon_is_one_png_image_with_the_lengths_apple_reads() {
        let icns = icns(ICON);
        assert_eq!(&icns[..4], b"icns");
        assert_eq!(
            u32::from_be_bytes(icns[4..8].try_into().unwrap()) as usize,
            icns.len()
        );
        assert_eq!(&icns[8..12], b"ic08");
        assert_eq!(
            u32::from_be_bytes(icns[12..16].try_into().unwrap()) as usize,
            icns.len() - 8
        );
        assert_eq!(&icns[16..], ICON);
    }

    #[test]
    fn texts_in_a_property_list_cannot_close_its_tags() {
        assert_eq!(string("a <b> & c"), "<string>a &lt;b&gt; &amp; c</string>");
    }
}
