//! What the app needs from the system, behind interfaces (`NFR-13`): the real ones live in the
//! app, tests use fakes.

use crate::accounts::Account;
use crate::background::Signal;
use crate::reminders::{Action, Notification};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SecretStoreError {
    /// No secret store runs on the system (`DEC-47`).
    NotRunning,
    /// The store runs but did not do what was asked, e.g. it stayed locked; never holds a secret.
    Failed(String),
}

/// The system's secret store (Secret Service on Linux), the only place tokens are kept (`NFR-04`).
pub trait SecretStore: Send + Sync {
    /// Whether a secret store runs, without unlocking it.
    fn check(&self) -> Result<(), SecretStoreError>;
    /// Keeps the account's refresh token, replacing an earlier one.
    fn save(&self, account: &Account, refresh_token: &str) -> Result<(), SecretStoreError>;
    /// The account's refresh token, unlocking the store if needed; `None` when it holds none.
    fn load(&self, account_id: &str) -> Result<Option<String>, SecretStoreError>;
    /// Deletes everything kept for the account, unlocking the store if needed, and returns its
    /// refresh token; `None` when it held none or the token could not be read, which does not
    /// stop the deletion.
    fn take(&self, account_id: &str) -> Result<Option<String>, SecretStoreError>;
}

/// The system's default browser, opened through the desktop's URL handler.
pub trait Browser: Send + Sync {
    fn open(&self, url: &str) -> std::io::Result<()>;
}

/// The system's word that the computer woke from sleep or the network came back (`MOD-02-F08`).
pub trait Wakeups {
    /// Calls `woke` after every wake from sleep and every return of the network, from then on.
    fn watch(&self, woke: Box<dyn Fn() + Send + Sync>);
}

/// Whether the computer has a network connection, as the system says (`MOD-02-F14`).
pub trait Network {
    /// Calls `changed` with whether the computer is online, once the system answers and after
    /// every change from then on.
    fn watch(&self, changed: Box<dyn Fn(bool) + Send + Sync>);
}

/// The desktop's notification service (`MOD-07-F01`); the desktop decides what is shown, e.g.
/// in "do not disturb" mode.
pub trait Notifier: Send {
    /// Shows the notification with its buttons; `chosen` is called with what the user chooses
    /// on it: clicking it (`MOD-07-F08`) or one of its buttons (`MOD-07-F09`, `F10`).
    fn show(&self, notification: &Notification, chosen: Box<dyn Fn(Action) + Send + Sync>);
    /// Whether the desktop's notifications show buttons; without them, Join and Snooze are
    /// reached by clicking the notification (`DEC-64`).
    fn buttons(&self) -> bool;
}

/// The desktop's system tray (`MOD-07-F14`, StatusNotifierItem on KDE).
pub trait Tray {
    /// Shows the app's icon in the tray, with a menu that opens the window and quits the app;
    /// `chosen` is called with what the user does on it: `TrayClicked`, `Open` or `Quit`. False
    /// when the desktop has no tray to show it in. At login the desktop's tray may not be there
    /// yet: the icon is taken to be shown, and appears once the tray does.
    fn show(&self, chosen: Box<dyn Fn(Signal) + Send + Sync>, at_login: bool) -> bool;
}

/// Starting the app at login, in the tray (`MOD-07-F16`).
pub trait Autostart: Send + Sync {
    /// Whether the app starts at login.
    fn is_on(&self) -> bool;
    fn set(&self, on: bool) -> std::io::Result<()>;
}

/// The one copy of the app that runs (`MOD-07-F17`).
pub trait Instance {
    /// Makes this copy the one that runs, and calls `launched` whenever the app is launched again
    /// from then on. False when another copy already runs: it was told it was launched again, and
    /// this copy is to end.
    fn claim(&self, launched: Box<dyn Fn() + Send + Sync>) -> bool;
}

/// The desktop's handlers of links: `webcal:` links opened anywhere go to the app
/// (`MOD-07-F18`).
pub trait LinkHandler {
    /// Registers the app as the handler of `webcal:` links.
    fn register(&self) -> std::io::Result<()>;
}
