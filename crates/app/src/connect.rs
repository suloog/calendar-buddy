//! Connecting a Google account: from the welcome screen of a first run (`US-01-01`) and from the
//! settings (`US-01-02`), and signing in again to one that needs it (`US-01-03`). Checking and
//! signing in run off the UI thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

use calendar_core::accounts::{Account, ConnectError};
use calendar_core::platform::{Browser, SecretStore};
use calendar_core::texts;
use calendar_google::{Client, Endpoints, SIGN_IN_WAIT};
use eframe::egui;

use crate::platform::{SystemBrowser, SystemSecretStore};

enum Stage {
    /// The one action, after a failure with what went wrong.
    Start(Option<ConnectError>),
    /// Checking the client file, the secret store and the network.
    Checking,
    /// The heads-up about Google's "unverified app" screen, before the browser opens.
    HeadsUp(Client),
    /// The browser has the sign-in; set the flag to cancel it.
    Waiting(Arc<AtomicBool>),
}

/// What the user chose on the screen.
enum Action {
    Check,
    SignIn(Client),
    Back,
}

enum Outcome {
    Checked(Result<Client, ConnectError>),
    Connected(Result<Account, ConnectError>),
}

/// Connecting one account at a time, step by step.
pub struct Connect {
    stage: Stage,
    sender: Sender<Outcome>,
    outcomes: Receiver<Outcome>,
    store: Arc<dyn SecretStore>,
    browser: Arc<dyn Browser>,
    /// The e-mail address of the account being signed in to again; `None` adds an account.
    again: Option<String>,
}

impl Connect {
    pub fn new() -> Self {
        let (sender, outcomes) = mpsc::channel();
        Connect {
            stage: Stage::Start(None),
            sender,
            outcomes,
            store: Arc::new(SystemSecretStore),
            browser: Arc::new(SystemBrowser),
            again: None,
        }
    }

    /// Whether nothing is under way, so a new connection can start.
    pub fn idle(&self) -> bool {
        matches!(self.stage, Stage::Start(_))
    }

    /// Starts signing in again to an account that needs it (`MOD-01-F06`): the same steps as
    /// adding one, with Google asked for that account.
    pub fn sign_in_again(&mut self, ctx: &egui::Context, email: &str) {
        self.again = Some(email.to_string());
        self.check(ctx);
    }

    /// Runs `work` off the UI thread and wakes the UI with its outcome.
    fn spawn(&self, ctx: &egui::Context, work: impl FnOnce() -> Outcome + Send + 'static) {
        let sender = self.sender.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = sender.send(work());
            ctx.request_repaint();
        });
    }

    fn check(&mut self, ctx: &egui::Context) {
        let store = self.store.clone();
        self.spawn(ctx, move || {
            let path = calendar_google::client_file_path();
            Outcome::Checked(calendar_google::prepare(
                &path,
                store.as_ref(),
                &Endpoints::google(),
            ))
        });
        self.stage = Stage::Checking;
    }

    fn sign_in(&mut self, ctx: &egui::Context, client: Client) {
        let cancel = Arc::new(AtomicBool::new(false));
        let (store, browser, flag) = (self.store.clone(), self.browser.clone(), cancel.clone());
        let again = self.again.clone();
        self.spawn(ctx, move || {
            Outcome::Connected(calendar_google::connect(
                &client,
                &Endpoints::google(),
                browser.as_ref(),
                store.as_ref(),
                again.as_deref(),
                &flag,
                SIGN_IN_WAIT,
            ))
        });
        self.stage = Stage::Waiting(cancel);
    }

    /// Takes what the work off the UI thread brought; returns the account once one is connected.
    /// Called every frame, so an account connects even with its screen closed.
    pub fn poll(&mut self) -> Option<Account> {
        let mut connected = None;
        for outcome in self.outcomes.try_iter() {
            match outcome {
                Outcome::Checked(Ok(client)) => self.stage = Stage::HeadsUp(client),
                Outcome::Connected(Ok(account)) => {
                    self.stage = Stage::Start(None);
                    self.again = None;
                    connected = Some(account);
                }
                Outcome::Checked(Err(error)) | Outcome::Connected(Err(error)) => {
                    self.stage = Stage::Start(Some(error))
                }
            }
        }
        connected
    }

    /// Back to the start when the screen showing connecting closes, so it opens afresh; a sign-in
    /// the browser has goes on.
    pub fn close(&mut self) {
        if matches!(self.stage, Stage::Start(Some(_)) | Stage::HeadsUp(_)) {
            self.stage = Stage::Start(None);
            self.again = None;
        }
    }

    /// Draws the current step; `start` names the action that begins connecting.
    pub fn show(&mut self, ui: &mut egui::Ui, start: &str) {
        let mut chosen = None;
        if let Some(email) = &self.again
            && !matches!(self.stage, Stage::Start(None))
        {
            ui.label(texts::signing_in_again(email));
        }
        match &self.stage {
            Stage::Start(error) => {
                let label = match error {
                    Some(error) => {
                        ui.colored_label(ui.visuals().warn_fg_color, error.message());
                        ui.add_space(8.0);
                        texts::TRY_AGAIN
                    }
                    None => start,
                };
                if ui.button(label).clicked() {
                    chosen = Some(Action::Check);
                }
            }
            Stage::Checking => {
                ui.spinner();
            }
            Stage::HeadsUp(client) => {
                ui.label(texts::UNVERIFIED_HEADS_UP);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(texts::CONTINUE_IN_BROWSER).clicked() {
                        chosen = Some(Action::SignIn(client.clone()));
                    }
                    if ui.button(texts::CANCEL).clicked() {
                        chosen = Some(Action::Back);
                    }
                });
            }
            Stage::Waiting(cancel) => {
                ui.spinner();
                ui.label(texts::WAITING);
                ui.add_space(8.0);
                if ui.button(texts::CANCEL).clicked() {
                    // The sign-in then ends as cancelled and says so.
                    cancel.store(true, Ordering::Relaxed);
                }
            }
        }
        let ctx = ui.ctx().clone();
        match chosen {
            Some(Action::Check) => self.check(&ctx),
            Some(Action::SignIn(client)) => self.sign_in(&ctx, client),
            Some(Action::Back) => {
                self.stage = Stage::Start(None);
                self.again = None;
            }
            None => {}
        }
    }
}

/// The welcome screen of a first run: its one action connects a Google account (`MOD-01-F12`).
pub fn welcome(ui: &mut egui::Ui, connect: &mut Connect) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.set_max_width(480.0);
            ui.add_space(ui.available_height() / 4.0);
            ui.heading(texts::WELCOME);
            ui.label(texts::WELCOME_HINT);
            ui.add_space(16.0);
            connect.show(ui, texts::CONNECT);
        });
    });
}
