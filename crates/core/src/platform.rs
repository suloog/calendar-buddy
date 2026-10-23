//! What the app needs from the system, behind interfaces (`NFR-13`): the real ones live in the
//! app, tests use fakes.

use crate::accounts::Account;

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
