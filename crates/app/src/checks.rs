//! Asking Google, off the UI thread, whether the connected accounts still work: at start, then
//! regularly, so an account whose access Google ended is noticed (`US-01-03`).

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;

use calendar_core::accounts::{self, Accounts, Check, CheckError};
use calendar_core::platform::SecretStore;
use calendar_google::Endpoints;
use eframe::egui;

use crate::platform::SystemSecretStore;

type Checked = Result<Vec<(String, Check)>, CheckError>;

pub struct Checks {
    sender: Sender<Checked>,
    results: Receiver<Checked>,
    store: Arc<dyn SecretStore>,
    /// When the next check is due; `None` waits for "Try again".
    next: Option<Instant>,
    running: bool,
    /// An account was signed in while the check ran, so what it found may be out of date.
    stale: bool,
}

impl Checks {
    /// The first check is due at once.
    pub fn new() -> Self {
        let (sender, results) = mpsc::channel();
        Checks {
            sender,
            results,
            store: Arc::new(SystemSecretStore),
            next: Some(Instant::now()),
            running: false,
            stale: false,
        }
    }

    /// No check is due until "Try again"; for pictures of the app, which must not reach Google.
    #[cfg(test)]
    pub fn paused() -> Self {
        Checks {
            next: None,
            ..Checks::new()
        }
    }

    /// Starts a check of the connected accounts when one is due. Called every frame.
    pub fn tick(&mut self, ctx: &egui::Context, accounts: &Accounts) {
        if self.running || self.next.is_none_or(|next| Instant::now() < next) {
            return;
        }
        let connected = accounts.connected();
        if connected.is_empty() {
            self.next = Some(Instant::now() + accounts::CHECK_EVERY);
            return;
        }
        self.running = true;
        self.stale = false;
        let (sender, store, ctx) = (self.sender.clone(), self.store.clone(), ctx.clone());
        std::thread::spawn(move || {
            let path = calendar_google::client_file_path();
            let _ = sender.send(calendar_google::check(
                &path,
                &Endpoints::google(),
                store.as_ref(),
                &connected,
            ));
            ctx.request_repaint();
        });
    }

    /// "Try again" after a problem: checks at once.
    pub fn now(&mut self) {
        self.next = Some(Instant::now());
    }

    /// An account was just signed in: a check under way may have read its old sign-in, and checks
    /// paused by a secret store problem can go on, as the store has just kept the sign-in.
    pub fn signed_in(&mut self) {
        self.stale = self.running;
        if !self.running {
            self.now();
        }
    }

    /// What the last check found, once it is done.
    pub fn poll(&mut self) -> Option<Checked> {
        let checked = self.results.try_recv().ok()?;
        self.running = false;
        if std::mem::take(&mut self.stale) {
            self.now();
            return None;
        }
        self.next = accounts::next_check(&checked).map(|after| Instant::now() + after);
        Some(checked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_check_that_ran_during_a_sign_in_is_dropped_and_run_again() {
        let mut checks = Checks::new();
        checks.running = true;
        checks.next = Some(Instant::now() + accounts::CHECK_EVERY);
        checks.signed_in();
        let old = Ok(vec![("1".to_string(), Check::SignInNeeded)]);
        checks.sender.send(old).unwrap();

        assert_eq!(checks.poll(), None);
        assert!(!checks.running);
        assert!(checks.next.is_some_and(|next| next <= Instant::now()));
    }

    #[test]
    fn checks_paused_by_the_secret_store_go_on_after_a_sign_in() {
        let mut checks = Checks::new();
        checks.running = true;
        let locked = Err(CheckError::SecretStore("unlocking it was declined".into()));
        checks.sender.send(locked.clone()).unwrap();
        assert_eq!(checks.poll(), Some(locked));
        assert_eq!(checks.next, None);

        checks.signed_in();

        assert!(checks.next.is_some_and(|next| next <= Instant::now()));
    }
}
