//! The real platform behind `calendar_core::platform`: the Secret Service and the desktop's URL
//! handler. Tests never use these; they use fakes.

use std::collections::HashMap;

use calendar_core::accounts::Account;
use calendar_core::platform::{Browser, SecretStore, SecretStoreError};
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
