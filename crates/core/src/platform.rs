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
}

/// The system's default browser, opened through the desktop's URL handler.
pub trait Browser: Send + Sync {
    fn open(&self, url: &str) -> std::io::Result<()>;
}
