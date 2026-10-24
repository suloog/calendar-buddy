//! The real platform behind `calendar_core::platform`: the Secret Service, the desktop's URL
//! handler, the desktop's notifications, systemd-logind and NetworkManager on the system bus, the
//! system tray, XDG autostart, the handler of `webcal:` links, the desktop's light or dark
//! preference and the app's name on the session bus. Tests never use these; they use fakes, or a
//! private bus and folder of their own.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use calendar_core::accounts::Account;
use calendar_core::background::{self, Signal};
use calendar_core::platform::{
    Appearance, Autostart, Browser, Instance, LinkHandler, Network, Notifier, SecretStore,
    SecretStoreError, Tray, Wakeups,
};
use calendar_core::reminders::{Action, Notification, SNOOZES};
use calendar_core::theme::{self, Scheme};
use calendar_core::webcal;
use secret_service::blocking::SecretService;
use secret_service::{EncryptionType, Error};

/// The Secret Service (KWallet, GNOME Keyring, KeePassXC) on the session bus (`DEC-47`).
pub struct SystemSecretStore;

fn failed(error: Error) -> SecretStoreError {
    SecretStoreError::Failed(match error {
        Error::Prompt => "unlocking it was declined".to_string(),
        error => error.to_string(),
    })
}

impl SecretStore for SystemSecretStore {
    fn check(&self) -> Result<(), SecretStoreError> {
        // Whatever stops a session with the service, there is no store the app can use.
        SecretService::connect(EncryptionType::Dh)
            .map(|_| ())
            .map_err(|_| SecretStoreError::NotRunning)
    }

    fn save(&self, account: &Account, refresh_token: &str) -> Result<(), SecretStoreError> {
        let service =
            SecretService::connect(EncryptionType::Dh).map_err(|_| SecretStoreError::NotRunning)?;
        let collection = service.get_default_collection().map_err(failed)?;
        if collection.is_locked().map_err(failed)? {
            collection.unlock().map_err(failed)?;
        }
        collection
            .create_item(
                &format!("Calendar Buddy: {}", account.email),
                HashMap::from([("application", "calendar-buddy"), ("account", &account.id)]),
                refresh_token.as_bytes(),
                true,
                "text/plain",
            )
            .map_err(failed)?;
        Ok(())
    }

    fn load(&self, account_id: &str) -> Result<Option<String>, SecretStoreError> {
        let service =
            SecretService::connect(EncryptionType::Dh).map_err(|_| SecretStoreError::NotRunning)?;
        let collection = service.get_default_collection().map_err(failed)?;
        if collection.is_locked().map_err(failed)? {
            collection.unlock().map_err(failed)?;
        }
        let items = collection
            .search_items(HashMap::from([
                ("application", "calendar-buddy"),
                ("account", account_id),
            ]))
            .map_err(failed)?;
        let Some(item) = items.first() else {
            return Ok(None);
        };
        let secret = item.get_secret().map_err(failed)?;
        String::from_utf8(secret)
            .map(Some)
            .map_err(|_| SecretStoreError::Failed("the saved sign-in could not be read".into()))
    }

    fn take(&self, account_id: &str) -> Result<Option<String>, SecretStoreError> {
        let service =
            SecretService::connect(EncryptionType::Dh).map_err(|_| SecretStoreError::NotRunning)?;
        let collection = service.get_default_collection().map_err(failed)?;
        if collection.is_locked().map_err(failed)? {
            collection.unlock().map_err(failed)?;
        }
        let items = collection
            .search_items(HashMap::from([
                ("application", "calendar-buddy"),
                ("account", account_id),
            ]))
            .map_err(failed)?;
        let refresh_token = items
            .first()
            .and_then(|item| item.get_secret().ok())
            .and_then(|secret| String::from_utf8(secret).ok());
        for item in items {
            item.delete().map_err(failed)?;
        }
        Ok(refresh_token)
    }
}

/// The default browser, through `xdg-open`.
pub struct SystemBrowser;

impl Browser for SystemBrowser {
    fn open(&self, url: &str) -> std::io::Result<()> {
        let mut child = std::process::Command::new("xdg-open").arg(url).spawn()?;
        // `xdg-open` can stay until the browser closes; the sign-in does not wait for it.
        std::thread::spawn(move || child.wait());
        Ok(())
    }
}

/// The desktop's light or dark preference, from the XDG desktop portal's `color-scheme` and its
/// change signal on the session bus (MOD-04 *Technical notes*). Without the portal the desktop
/// states no preference.
pub struct SystemAppearance;

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const PORTAL_SETTINGS: &str = "org.freedesktop.portal.Settings";
const APPEARANCE: &str = "org.freedesktop.appearance";
const COLOR_SCHEME: &str = "color-scheme";

/// The preference a `color-scheme` value states.
fn scheme(value: &zbus::zvariant::Value) -> Option<Scheme> {
    theme::from_color_scheme(value.downcast_ref::<u32>().unwrap_or(0))
}

impl SystemAppearance {
    /// Watches the portal on `bus`, off the UI thread.
    fn watch_on(
        bus: zbus::Result<zbus::blocking::Connection>,
        changed: Box<dyn Fn(Option<Scheme>) + Send + Sync>,
    ) {
        std::thread::spawn(move || -> zbus::Result<()> {
            let bus = bus?;
            let proxy = zbus::blocking::Proxy::new(&bus, PORTAL, PORTAL_PATH, PORTAL_SETTINGS)?;
            // Listening first, so no change is missed between the answer and the signals.
            let signals = proxy.receive_signal("SettingChanged")?;
            let value: zbus::zvariant::OwnedValue =
                proxy.call("ReadOne", &(APPEARANCE, COLOR_SCHEME))?;
            changed(scheme(&value));
            for message in signals {
                let body = message.body();
                let Ok((namespace, key, value)) =
                    body.deserialize::<(&str, &str, zbus::zvariant::Value)>()
                else {
                    continue;
                };
                if namespace == APPEARANCE && key == COLOR_SCHEME {
                    changed(scheme(&value));
                }
            }
            Ok(())
        });
    }
}

impl Appearance for SystemAppearance {
    fn watch(&self, changed: Box<dyn Fn(Option<Scheme>) + Send + Sync>) {
        SystemAppearance::watch_on(zbus::blocking::Connection::session(), changed);
    }
}

/// What choosing on each notification shown and not closed yet does, by its ID.
type Clicks = Arc<Mutex<HashMap<u32, Box<dyn Fn(Action) + Send + Sync>>>>;

/// The desktop's notifications, through `org.freedesktop.Notifications` on the session bus
/// (MOD-07 *Technical notes*). Without a session bus nothing is shown.
pub struct SystemNotifier {
    /// The session bus, once connected; `None` when it could not be.
    bus: OnceLock<Option<zbus::blocking::Connection>>,
    clicks: Clicks,
    /// Whether the desktop's notifications show buttons, once asked.
    buttons: OnceLock<bool>,
}

const NOTIFICATIONS: &str = "org.freedesktop.Notifications";
const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";

impl SystemNotifier {
    pub fn new() -> Self {
        SystemNotifier {
            bus: OnceLock::new(),
            clicks: Arc::default(),
            buttons: OnceLock::new(),
        }
    }

    /// The session bus, connected the first time it is needed.
    fn bus(&self) -> Option<&zbus::blocking::Connection> {
        let bus = self
            .bus
            .get_or_init(|| SystemNotifier::connect(self.clicks.clone()).ok());
        bus.as_ref()
    }

    /// Connects to the session bus and listens for clicks on the notifications and for their
    /// closing, in the order the desktop sends them.
    fn connect(clicks: Clicks) -> zbus::Result<zbus::blocking::Connection> {
        let bus = zbus::blocking::Connection::session()?;
        let rule = format!(
            "type='signal',sender='{NOTIFICATIONS}',interface='{NOTIFICATIONS}',\
             path='{NOTIFICATIONS_PATH}'"
        );
        let messages = zbus::blocking::MessageIterator::for_match_rule(rule.as_str(), &bus, None)?;
        std::thread::spawn(move || {
            for message in messages.flatten() {
                let header = message.header();
                let body = message.body();
                // Clicking a notification's body invokes its "default" action, a button its own.
                let (id, invoked) = match header.member().map(|member| member.as_str()) {
                    Some("ActionInvoked") => match body.deserialize::<(u32, String)>() {
                        Ok((id, key)) => match action(&key) {
                            Some(action) => (id, Some(action)),
                            None => continue,
                        },
                        Err(_) => continue,
                    },
                    Some("NotificationClosed") => match body.deserialize::<(u32, u32)>() {
                        Ok((id, _)) => (id, None),
                        Err(_) => continue,
                    },
                    _ => continue,
                };
                let chosen = clicks.lock().expect("clicks").remove(&id);
                if let (Some(action), Some(chosen)) = (invoked, chosen) {
                    chosen(action);
                }
            }
        });
        Ok(bus)
    }
}

/// The key of the action on the bus; "default" is clicking the notification's body.
fn key(action: Action) -> String {
    match action {
        Action::Open => "default".to_string(),
        Action::Join => "join".to_string(),
        Action::Snooze(minutes) => format!("snooze-{minutes}"),
        Action::Dismiss => "dismiss".to_string(),
    }
}

/// The action of the key on the bus; `None` for a key the app did not give.
fn action(key: &str) -> Option<Action> {
    match key {
        "default" => Some(Action::Open),
        "join" => Some(Action::Join),
        "dismiss" => Some(Action::Dismiss),
        _ => key
            .strip_prefix("snooze-")
            .and_then(|minutes| minutes.parse().ok())
            .filter(|minutes| SNOOZES.contains(minutes))
            .map(Action::Snooze),
    }
}

/// The notification's text with the characters of the notifications' markup written out.
fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

impl Notifier for SystemNotifier {
    fn show(&self, notification: &Notification, chosen: Box<dyn Fn(Action) + Send + Sync>) {
        let Some(bus) = self.bus() else {
            return;
        };
        // Pairs of an action's key and what its button says.
        let mut actions = vec![key(Action::Open), Action::Open.label()];
        for action in notification.actions() {
            actions.extend([key(action), action.label()]);
        }
        let hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
        let shown = bus.call_method(
            Some(NOTIFICATIONS),
            NOTIFICATIONS_PATH,
            Some(NOTIFICATIONS),
            "Notify",
            &(
                "Calendar Buddy",
                0u32,
                "x-office-calendar",
                notification.title.as_str(),
                escaped(&notification.body()),
                actions,
                hints,
                -1i32,
            ),
        );
        if let Ok(id) = shown.and_then(|reply| reply.body().deserialize::<u32>()) {
            self.clicks.lock().expect("clicks").insert(id, chosen);
        }
    }

    fn buttons(&self) -> bool {
        *self.buttons.get_or_init(|| {
            let Some(bus) = self.bus() else {
                return false;
            };
            let capabilities = bus.call_method(
                Some(NOTIFICATIONS),
                NOTIFICATIONS_PATH,
                Some(NOTIFICATIONS),
                "GetCapabilities",
                &(),
            );
            capabilities
                .and_then(|reply| reply.body().deserialize::<Vec<String>>())
                .is_ok_and(|capabilities| capabilities.iter().any(|c| c == "actions"))
        })
    }
}

/// Waking from sleep, from systemd-logind, and the network's return, from NetworkManager, on the
/// system bus. Without either, there is nothing to watch.
pub struct SystemWakeups;

/// NetworkManager's state with full connectivity (`NM_STATE_CONNECTED_GLOBAL`).
const CONNECTED_GLOBAL: u32 = 70;

impl Wakeups for SystemWakeups {
    fn watch(&self, woke: Box<dyn Fn() + Send + Sync>) {
        let woke = std::sync::Arc::new(woke);
        let signals = [
            (
                "org.freedesktop.login1",
                "/org/freedesktop/login1",
                "org.freedesktop.login1.Manager",
                "PrepareForSleep",
            ),
            (
                "org.freedesktop.NetworkManager",
                "/org/freedesktop/NetworkManager",
                "org.freedesktop.NetworkManager",
                "StateChanged",
            ),
        ];
        for (service, path, interface, signal) in signals {
            let woke = woke.clone();
            std::thread::spawn(move || -> zbus::Result<()> {
                let bus = zbus::blocking::Connection::system()?;
                let proxy = zbus::blocking::Proxy::new(&bus, service, path, interface)?;
                for message in proxy.receive_signal(signal)? {
                    let body = message.body();
                    // `PrepareForSleep(false)` comes after waking; `StateChanged(70)` when the
                    // network is back.
                    let back = match signal {
                        "PrepareForSleep" => body.deserialize::<bool>().is_ok_and(|going| !going),
                        _ => body.deserialize::<u32>() == Ok(CONNECTED_GLOBAL),
                    };
                    if back {
                        woke();
                    }
                }
                Ok(())
            });
        }
    }
}

/// The network's state, from NetworkManager on the system bus. Without it there is nothing to
/// ask, and the app counts as online.
pub struct SystemNetwork;

/// Whether NetworkManager's state (`NMState`) means the computer reaches the internet: full
/// connectivity, or a state NetworkManager cannot tell (`NM_STATE_UNKNOWN`).
fn online(state: u32) -> bool {
    state == 0 || state >= CONNECTED_GLOBAL
}

impl Network for SystemNetwork {
    fn watch(&self, changed: Box<dyn Fn(bool) + Send + Sync>) {
        std::thread::spawn(move || -> zbus::Result<()> {
            let bus = zbus::blocking::Connection::system()?;
            let proxy = zbus::blocking::Proxy::new(
                &bus,
                "org.freedesktop.NetworkManager",
                "/org/freedesktop/NetworkManager",
                "org.freedesktop.NetworkManager",
            )?;
            // Listening first, so no change is missed between the answer and the signals.
            let signals = proxy.receive_signal("StateChanged")?;
            changed(online(proxy.get_property::<u32>("State")?));
            for message in signals {
                if let Ok(state) = message.body().deserialize::<u32>() {
                    changed(online(state));
                }
            }
            Ok(())
        });
    }
}

/// The app's icon in the system tray, as a StatusNotifierItem on the session bus (MOD-07
/// *Technical notes*).
pub struct SystemTray(OnceLock<ksni::blocking::Handle<TrayIcon>>);

pub struct TrayIcon {
    chosen: Box<dyn Fn(Signal) + Send + Sync>,
}

impl SystemTray {
    pub fn new() -> Self {
        SystemTray(OnceLock::new())
    }
}

impl ksni::Tray for TrayIcon {
    fn id(&self) -> String {
        "calendar-buddy".into()
    }

    fn title(&self) -> String {
        "Calendar Buddy".into()
    }

    fn icon_name(&self) -> String {
        "x-office-calendar".into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        (self.chosen)(Signal::TrayClicked);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::StandardItem;
        vec![
            StandardItem {
                label: "Open Calendar Buddy".into(),
                icon_name: "x-office-calendar".into(),
                activate: Box::new(|tray: &mut Self| (tray.chosen)(Signal::Open)),
                ..Default::default()
            }
            .into(),
            ksni::MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|tray: &mut Self| (tray.chosen)(Signal::Quit)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

impl Tray for SystemTray {
    fn show(&self, chosen: Box<dyn Fn(Signal) + Send + Sync>, at_login: bool) -> bool {
        use ksni::blocking::TrayMethods;
        match (TrayIcon { chosen }).assume_sni_available(at_login).spawn() {
            Ok(handle) => self.0.set(handle).is_ok(),
            // No session bus, or no tray on the desktop.
            Err(_) => false,
        }
    }
}

/// Starting at login through an XDG autostart entry (MOD-07 *Technical notes*), in
/// `$XDG_CONFIG_HOME/autostart`, or `~/.config/autostart` when that is not set.
pub struct XdgAutostart {
    /// The entry; `None` when there is no home directory.
    entry: Option<PathBuf>,
}

impl XdgAutostart {
    pub fn new() -> Self {
        let config = calendar_core::settings::config_dir();
        XdgAutostart::at(config.and_then(|dir| Some(dir.parent()?.join("autostart"))))
    }

    fn at(autostart: Option<PathBuf>) -> Self {
        XdgAutostart {
            entry: autostart.map(|dir| dir.join("calendar-buddy.desktop")),
        }
    }
}

impl Autostart for XdgAutostart {
    fn is_on(&self) -> bool {
        self.entry.as_ref().is_some_and(|entry| entry.exists())
    }

    fn set(&self, on: bool) -> std::io::Result<()> {
        let Some(entry) = &self.entry else {
            return Err(std::io::ErrorKind::NotFound.into());
        };
        if on {
            if let Some(dir) = entry.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(entry, background::autostart_entry(&running_exe()?))
        } else {
            match std::fs::remove_file(entry) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                removed => removed,
            }
        }
    }
}

/// The program that runs; one replaced while it runs, as by an upgrade, is named "… (deleted)".
fn running_exe() -> std::io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let exe = exe.to_string_lossy();
    Ok(PathBuf::from(
        exe.strip_suffix(" (deleted)").unwrap_or(&exe),
    ))
}

/// The app as the handler of `webcal:` links (MOD-07 *Technical notes*), through a desktop entry
/// in `$XDG_DATA_HOME/applications`, or `~/.local/share/applications` when that is not set.
pub struct XdgLinkHandler {
    /// The entry; `None` when there is no home directory.
    entry: Option<PathBuf>,
}

impl XdgLinkHandler {
    pub fn new() -> Self {
        let data = std::env::var_os("XDG_DATA_HOME")
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            });
        XdgLinkHandler::at(data.map(|dir| dir.join("applications")))
    }

    fn at(applications: Option<PathBuf>) -> Self {
        XdgLinkHandler {
            entry: applications.map(|dir| dir.join("calendar-buddy-webcal.desktop")),
        }
    }
}

impl LinkHandler for XdgLinkHandler {
    fn register(&self) -> std::io::Result<()> {
        let Some(entry) = &self.entry else {
            return Err(std::io::ErrorKind::NotFound.into());
        };
        let wanted = webcal::handler_entry(&running_exe()?);
        // Written only when it changes, as the desktop rereads its entries after every write.
        if std::fs::read_to_string(entry).is_ok_and(|kept| kept == wanted) {
            return Ok(());
        }
        if let Some(dir) = entry.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(entry, wanted)
    }
}

/// The app's well-known name on the session bus, held by the copy that runs (MOD-07
/// *Technical notes*).
const INSTANCE: &str = "pl.ordergroup.CalendarBuddy";
const INSTANCE_PATH: &str = "/pl/ordergroup/CalendarBuddy";

/// The one copy of the app, through its name on the session bus, where a launch tells the running
/// copy through the desktop's standard `org.freedesktop.Application` interface. Without a session
/// bus every launch runs.
pub struct SystemInstance(OnceLock<zbus::blocking::Connection>);

impl SystemInstance {
    pub fn new() -> Self {
        SystemInstance(OnceLock::new())
    }

    /// Takes the app's name on `bus`, or tells the copy that holds it that it was launched
    /// again; true when this copy took it.
    fn claim_on(
        &self,
        bus: zbus::blocking::Connection,
        launched: Box<dyn Fn() + Send + Sync>,
    ) -> zbus::Result<bool> {
        bus.object_server()
            .at(INSTANCE_PATH, Application { launched })?;
        let flags = zbus::fdo::RequestNameFlags::DoNotQueue.into();
        match bus.request_name_with_flags(INSTANCE, flags) {
            Ok(_) => {
                // The name stays the app's while the connection lives.
                let _ = self.0.set(bus);
                Ok(true)
            }
            Err(zbus::Error::NameTaken) => {
                let platform_data: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
                bus.call_method(
                    Some(INSTANCE),
                    INSTANCE_PATH,
                    Some("org.freedesktop.Application"),
                    "Activate",
                    &(platform_data,),
                )?;
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }
}

impl Instance for SystemInstance {
    fn claim(&self, launched: Box<dyn Fn() + Send + Sync>) -> bool {
        zbus::blocking::Connection::session()
            .and_then(|bus| self.claim_on(bus, launched))
            .unwrap_or(true)
    }
}

/// What a launch of the app tells the copy that runs.
struct Application {
    launched: Box<dyn Fn() + Send + Sync>,
}

#[zbus::interface(name = "org.freedesktop.Application")]
impl Application {
    fn activate(&self, _platform_data: HashMap<String, zbus::zvariant::OwnedValue>) {
        (self.launched)();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_action_of_a_notification_comes_back_from_its_key_on_the_bus() {
        let actions = [
            Action::Open,
            Action::Join,
            Action::Snooze(5),
            Action::Snooze(10),
            Action::Dismiss,
        ];
        for chosen in actions {
            assert_eq!(action(&key(chosen)), Some(chosen));
        }
        assert_eq!(key(Action::Open), "default");
        assert_eq!(action("snooze-later"), None);
        assert_eq!(action("snooze-7"), None);
        assert_eq!(action("other"), None);
    }

    #[test]
    fn only_full_connectivity_or_an_unknown_state_counts_as_online() {
        // Asleep, disconnected, disconnecting, connecting, local only, site only.
        for state in [10, 20, 30, 40, 50, 60] {
            assert!(!online(state), "{state}");
        }
        assert!(online(CONNECTED_GLOBAL));
        assert!(online(0));
    }

    #[test]
    fn starting_at_login_is_off_until_turned_on_and_off_again_when_turned_off() {
        let dir =
            std::env::temp_dir().join(format!("calendar-buddy-autostart-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let autostart = XdgAutostart::at(Some(dir.join("autostart")));
        assert!(!autostart.is_on());

        autostart.set(true).unwrap();
        assert!(autostart.is_on());
        let entry = std::fs::read_to_string(dir.join("autostart/calendar-buddy.desktop")).unwrap();
        let exe = std::env::current_exe().unwrap();
        assert_eq!(entry, background::autostart_entry(&exe));

        autostart.set(false).unwrap();
        assert!(!autostart.is_on());
        autostart.set(false).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_app_registers_as_the_handler_of_webcal_links_and_keeps_the_entry_current() {
        let dir =
            std::env::temp_dir().join(format!("calendar-buddy-webcal-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let handler = XdgLinkHandler::at(Some(dir.join("applications")));
        let path = dir.join("applications/calendar-buddy-webcal.desktop");

        handler.register().unwrap();
        let exe = std::env::current_exe().unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            webcal::handler_entry(&exe)
        );

        std::fs::write(&path, "[Desktop Entry]\nExec=/old/calendar-buddy %u\n").unwrap();
        handler.register().unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            webcal::handler_entry(&exe)
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A session bus of the test's own, gone when dropped.
    struct PrivateBus(std::process::Child, String);

    impl PrivateBus {
        fn start() -> Self {
            use std::io::BufRead;
            let mut daemon = std::process::Command::new("dbus-daemon")
                .args([
                    "--session",
                    "--nofork",
                    "--print-address",
                    "--address=unix:tmpdir=/tmp",
                ])
                .stdout(std::process::Stdio::piped())
                .spawn()
                .expect("dbus-daemon");
            let mut address = String::new();
            std::io::BufReader::new(daemon.stdout.take().unwrap())
                .read_line(&mut address)
                .unwrap();
            PrivateBus(daemon, address.trim().to_string())
        }

        fn connect(&self) -> zbus::blocking::Connection {
            zbus::blocking::connection::Builder::address(self.1.as_str())
                .unwrap()
                .build()
                .unwrap()
        }
    }

    impl Drop for PrivateBus {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    /// A desktop portal whose `color-scheme` the test sets.
    struct FakePortal(Arc<Mutex<u32>>);

    #[zbus::interface(name = "org.freedesktop.portal.Settings")]
    impl FakePortal {
        fn read_one(
            &self,
            namespace: &str,
            key: &str,
        ) -> zbus::fdo::Result<zbus::zvariant::OwnedValue> {
            if (namespace, key) != (APPEARANCE, COLOR_SCHEME) {
                return Err(zbus::fdo::Error::Failed("not found".into()));
            }
            Ok(zbus::zvariant::OwnedValue::from(*self.0.lock().unwrap()))
        }
    }

    #[test]
    fn the_desktop_preference_is_read_from_the_portal_and_followed_as_it_changes() {
        let bus = PrivateBus::start();
        let desktop = bus.connect();
        let color_scheme = Arc::new(Mutex::new(1));
        desktop
            .object_server()
            .at(PORTAL_PATH, FakePortal(color_scheme.clone()))
            .unwrap();
        desktop.request_name(PORTAL).unwrap();

        let (told, preferences) = std::sync::mpsc::channel();
        let told = Mutex::new(told);
        SystemAppearance::watch_on(
            Ok(bus.connect()),
            Box::new(move |preference| told.lock().unwrap().send(preference).unwrap()),
        );
        let next = || preferences.recv_timeout(std::time::Duration::from_secs(5));
        assert_eq!(next(), Ok(Some(Scheme::Dark)));

        let changed = |namespace: &str, value: u32| {
            *color_scheme.lock().unwrap() = value;
            let value = zbus::zvariant::Value::from(value);
            desktop
                .emit_signal(
                    None::<()>,
                    PORTAL_PATH,
                    PORTAL_SETTINGS,
                    "SettingChanged",
                    &(namespace, COLOR_SCHEME, value),
                )
                .unwrap();
        };
        changed("org.kde.kdeglobals.General", 2);
        changed(APPEARANCE, 2);
        assert_eq!(next(), Ok(Some(Scheme::Light)));
        changed(APPEARANCE, 0);
        assert_eq!(next(), Ok(None));
    }

    #[test]
    fn tc_07_06_a_second_launch_tells_the_running_copy_and_does_not_run() {
        let bus = PrivateBus::start();
        let (told, launches) = std::sync::mpsc::channel();
        let running = SystemInstance::new();
        let first = running.claim_on(bus.connect(), Box::new(move || told.send(()).unwrap()));
        assert!(first.unwrap());

        let second = SystemInstance::new().claim_on(bus.connect(), Box::new(|| {}));
        assert!(!second.unwrap());
        launches
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("the running copy is told");

        let third = SystemInstance::new().claim_on(bus.connect(), Box::new(|| {}));
        assert!(!third.unwrap());
        launches
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("told again");
    }
}
