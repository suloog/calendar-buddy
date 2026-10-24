//! The real platform behind `calendar_core::platform`: the Secret Service, the desktop's URL
//! handler, the desktop's notifications, and systemd-logind and NetworkManager on the system bus.
//! Tests never use these; they use fakes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use calendar_core::accounts::Account;
use calendar_core::platform::{Browser, Network, Notifier, SecretStore, SecretStoreError, Wakeups};
use calendar_core::reminders::Notification;
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

/// What clicking each notification shown and not closed yet does, by its ID.
type Clicks = Arc<Mutex<HashMap<u32, Box<dyn Fn() + Send + Sync>>>>;

/// The desktop's notifications, through `org.freedesktop.Notifications` on the session bus
/// (MOD-07 *Technical notes*). Without a session bus nothing is shown.
pub struct SystemNotifier {
    /// The session bus, once connected; `None` when it could not be.
    bus: OnceLock<Option<zbus::blocking::Connection>>,
    clicks: Clicks,
}

const NOTIFICATIONS: &str = "org.freedesktop.Notifications";
const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";

impl SystemNotifier {
    pub fn new() -> Self {
        SystemNotifier {
            bus: OnceLock::new(),
            clicks: Arc::default(),
        }
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
                // Clicking a notification's body invokes its "default" action.
                let (id, invoked) = match header.member().map(|member| member.as_str()) {
                    Some("ActionInvoked") => match body.deserialize::<(u32, String)>() {
                        Ok((id, action)) if action == "default" => (id, true),
                        _ => continue,
                    },
                    Some("NotificationClosed") => match body.deserialize::<(u32, u32)>() {
                        Ok((id, _)) => (id, false),
                        Err(_) => continue,
                    },
                    _ => continue,
                };
                let clicked = clicks.lock().expect("clicks").remove(&id);
                if invoked && let Some(clicked) = clicked {
                    clicked();
                }
            }
        });
        Ok(bus)
    }
}

/// The notification's text with the characters of the notifications' markup written out.
fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

impl Notifier for SystemNotifier {
    fn show(&self, notification: &Notification, clicked: Box<dyn Fn() + Send + Sync>) {
        let bus = self
            .bus
            .get_or_init(|| SystemNotifier::connect(self.clicks.clone()).ok());
        let Some(bus) = bus else {
            return;
        };
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
                vec!["default", "Open"],
                hints,
                -1i32,
            ),
        );
        if let Ok(id) = shown.and_then(|reply| reply.body().deserialize::<u32>()) {
            self.clicks.lock().expect("clicks").insert(id, clicked);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_full_connectivity_or_an_unknown_state_counts_as_online() {
        // Asleep, disconnected, disconnecting, connecting, local only, site only.
        for state in [10, 20, 30, 40, 50, 60] {
            assert!(!online(state), "{state}");
        }
        assert!(online(CONNECTED_GLOBAL));
        assert!(online(0));
    }
}
