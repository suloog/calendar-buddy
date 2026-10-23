//! Removing an account (`US-01-04`): asked first, then its credentials deleted from the secret
//! store off the UI thread, and only after that the app's access withdrawn at Google, which the
//! removal never waits for (`DEC-52`).

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

use calendar_core::accounts::{self, Account};
use calendar_core::platform::{SecretStore, SecretStoreError};
use calendar_google::Endpoints;
use eframe::egui;

use crate::platform::SystemSecretStore;

enum Stage {
    Idle,
    /// Asking the user to confirm (`MOD-01-U02`).
    Confirm(Account),
    /// Deleting the account's credentials from the secret store.
    Removing(Account),
    /// The secret store did not delete them, so the account stays.
    Failed(Account, SecretStoreError),
}

/// Removing one account at a time.
pub struct Removal {
    stage: Stage,
    sender: Sender<Result<(), SecretStoreError>>,
    results: Receiver<Result<(), SecretStoreError>>,
    store: Arc<dyn SecretStore>,
    endpoints: Endpoints,
}

impl Removal {
    pub fn new() -> Self {
        Removal::with(Arc::new(SystemSecretStore), Endpoints::google())
    }

    fn with(store: Arc<dyn SecretStore>, endpoints: Endpoints) -> Self {
        let (sender, results) = mpsc::channel();
        Removal {
            stage: Stage::Idle,
            sender,
            results,
            store,
            endpoints,
        }
    }

    /// Whether another account's removal can be asked for.
    pub fn idle(&self) -> bool {
        matches!(self.stage, Stage::Idle | Stage::Failed(..))
    }

    /// "Remove" on an account: asks first.
    pub fn ask(&mut self, account: Account) {
        self.stage = Stage::Confirm(account);
    }

    fn start(&mut self, ctx: &egui::Context, account: Account) {
        let (sender, store, endpoints) = (
            self.sender.clone(),
            self.store.clone(),
            self.endpoints.clone(),
        );
        let (ctx, id) = (ctx.clone(), account.id.clone());
        std::thread::spawn(move || {
            let refresh_token = match store.take(&id) {
                Ok(refresh_token) => {
                    let _ = sender.send(Ok(()));
                    refresh_token
                }
                Err(error) => {
                    let _ = sender.send(Err(error));
                    None
                }
            };
            ctx.request_repaint();
            // The credentials are gone from this computer; Google is told afterwards.
            if let Some(refresh_token) = refresh_token {
                calendar_google::revoke(&endpoints, &refresh_token);
            }
        });
        self.stage = Stage::Removing(account);
    }

    /// The ID of the account whose credentials are gone, so the app forgets it. Called every
    /// frame, so a removal finishes even with the settings closed.
    pub fn poll(&mut self) -> Option<String> {
        let result = self.results.try_recv().ok()?;
        let Stage::Removing(account) = std::mem::replace(&mut self.stage, Stage::Idle) else {
            return None;
        };
        match result {
            Ok(()) => Some(account.id),
            Err(error) => {
                self.stage = Stage::Failed(account, error);
                None
            }
        }
    }

    /// The settings closed: the question goes; a removal under way goes on, and why one stopped
    /// is still told when the settings open again.
    pub fn close(&mut self) {
        if matches!(self.stage, Stage::Confirm(_)) {
            self.stage = Stage::Idle;
        }
    }

    /// Draws the question, the removal under way or why it stopped; `can_start` is false while an
    /// account is being connected, so the two never run at once.
    pub fn show(&mut self, ui: &mut egui::Ui, can_start: bool) {
        let mut remove = None;
        match &self.stage {
            Stage::Idle => return,
            Stage::Confirm(account) => {
                ui.label(accounts::confirm_removal(&account.email));
                remove = buttons(ui, accounts::REMOVE, can_start);
            }
            Stage::Removing(account) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("Removing {}…", account.email));
                });
            }
            Stage::Failed(account, error) => {
                let warn = ui.visuals().warn_fg_color;
                ui.colored_label(warn, accounts::not_removed(&account.email, error));
                remove = buttons(ui, "Try again", can_start);
            }
        }
        match remove {
            Some(true) => {
                if let Stage::Confirm(account) | Stage::Failed(account, _) =
                    std::mem::replace(&mut self.stage, Stage::Idle)
                {
                    self.start(ui.ctx(), account);
                }
            }
            Some(false) => self.stage = Stage::Idle,
            None => {}
        }
    }
}

/// `go` and "Cancel"; says which was clicked.
fn buttons(ui: &mut egui::Ui, go: &str, enabled: bool) -> Option<bool> {
    ui.horizontal(|ui| {
        let go = ui.add_enabled(enabled, egui::Button::new(go)).clicked();
        let cancel = ui.button("Cancel").clicked();
        if go {
            Some(true)
        } else if cancel {
            Some(false)
        } else {
            None
        }
    })
    .inner
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    use super::*;

    /// The secret store, in memory.
    #[derive(Default)]
    struct FakeStore {
        locked: bool,
        saved: Mutex<Vec<(String, String)>>,
    }

    impl SecretStore for FakeStore {
        fn check(&self) -> Result<(), SecretStoreError> {
            Ok(())
        }

        fn save(&self, account: &Account, refresh_token: &str) -> Result<(), SecretStoreError> {
            let mut saved = self.saved.lock().unwrap();
            saved.push((account.id.clone(), refresh_token.to_string()));
            Ok(())
        }

        fn load(&self, account_id: &str) -> Result<Option<String>, SecretStoreError> {
            if self.locked {
                return Err(SecretStoreError::Failed("unlocking it was declined".into()));
            }
            let saved = self.saved.lock().unwrap();
            Ok(saved
                .iter()
                .find(|(id, _)| id == account_id)
                .map(|(_, token)| token.clone()))
        }

        fn take(&self, account_id: &str) -> Result<Option<String>, SecretStoreError> {
            let refresh_token = self.load(account_id)?;
            self.saved
                .lock()
                .unwrap()
                .retain(|(id, _)| id != account_id);
            Ok(refresh_token)
        }
    }

    fn account(id: &str) -> Account {
        Account {
            id: id.into(),
            email: format!("{id}@example.com"),
            workspace: false,
            contacts: true,
            directory: false,
        }
    }

    fn store_with(ids: &[&str], locked: bool) -> Arc<FakeStore> {
        let store = FakeStore {
            locked,
            ..FakeStore::default()
        };
        for id in ids {
            store.save(&account(id), &format!("refresh-{id}")).unwrap();
        }
        Arc::new(store)
    }

    /// Google takes the request and never answers, as on a network that hangs.
    fn silent_google(listener: &TcpListener) -> Endpoints {
        let revoke = format!("http://{}/revoke", listener.local_addr().unwrap());
        Endpoints {
            revoke,
            ..Endpoints::google()
        }
    }

    /// Polls until the removal stops being under way.
    fn finish(removal: &mut Removal) -> Option<String> {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Some(id) = removal.poll() {
                return Some(id);
            }
            if !matches!(removal.stage, Stage::Removing(_)) {
                return None;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("the removal waited for Google");
    }

    #[test]
    fn the_credentials_are_deleted_without_waiting_for_google() {
        let store = store_with(&["1", "2"], false);
        let google = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut removal = Removal::with(store.clone(), silent_google(&google));

        removal.ask(account("1"));
        assert!(!removal.idle());
        let Stage::Confirm(asked) = std::mem::replace(&mut removal.stage, Stage::Idle) else {
            panic!("not asked first");
        };
        removal.start(&egui::Context::default(), asked);

        assert_eq!(finish(&mut removal), Some("1".to_string()));
        assert!(removal.idle());
        let saved = store.saved.lock().unwrap();
        assert_eq!(
            saved.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            ["2"]
        );
    }

    #[test]
    fn a_secret_store_that_does_not_delete_keeps_the_account() {
        let store = store_with(&["1"], true);
        let google = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut removal = Removal::with(store.clone(), silent_google(&google));

        removal.start(&egui::Context::default(), account("1"));

        assert_eq!(finish(&mut removal), None);
        assert!(matches!(
            &removal.stage,
            Stage::Failed(account, SecretStoreError::Failed(_)) if account.id == "1"
        ));
        assert!(removal.idle());
        assert_eq!(store.saved.lock().unwrap().len(), 1);
        // Closing the settings meanwhile does not hide why the account is still there.
        removal.close();
        assert!(matches!(removal.stage, Stage::Failed(..)));
    }

    #[test]
    fn closing_the_settings_drops_the_question_and_deletes_nothing() {
        let store = store_with(&["1"], false);
        let google = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut removal = Removal::with(store.clone(), silent_google(&google));

        removal.ask(account("1"));
        removal.close();

        assert!(matches!(removal.stage, Stage::Idle));
        assert_eq!(removal.poll(), None);
        assert_eq!(store.saved.lock().unwrap().len(), 1);
    }
}
