mod agenda_view;
mod checks;
mod connect;
mod grid_view;
mod mini_calendar;
mod month_view;
mod platform;
mod remove;
mod sync;

use calendar_core::accounts::{self, Account, AccountState, Accounts, CheckError};
use calendar_core::agenda::{AgendaViewModel, agenda_view};
use calendar_core::details::event_details;
use calendar_core::model::Event;
use calendar_core::month::{MonthViewModel, month_view};
use calendar_core::navigation::{Navigation, View, mini_calendar};
use calendar_core::regional::Regional;
use calendar_core::settings::{self, Settings};
use calendar_core::shortcuts::{self, Command, SHORTCUTS};
use calendar_core::sync::Window;
use calendar_core::view_model::{GridViewModel, grid_view};
use checks::Checks;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use connect::Connect;
use eframe::egui;
use remove::Removal;
use std::sync::mpsc::{self, Receiver, Sender};
use sync::Synchronization;

fn system_zone() -> Tz {
    iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse().ok())
        .unwrap_or(Tz::UTC)
}

/// How often the system's time zone is looked up, to follow a change of it.
const ZONE_CHECK: std::time::Duration = std::time::Duration::from_secs(5);

/// Looks up the system's time zone off the UI thread and sends every change of it.
fn watch_zone(mut zone: Tz, ctx: egui::Context) -> Receiver<Tz> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(ZONE_CHECK);
            let now = system_zone();
            if now != zone {
                zone = now;
                if sender.send(zone).is_err() {
                    return;
                }
                ctx.request_repaint();
            }
        }
    });
    receiver
}

enum Built {
    Grid(GridViewModel),
    Month(MonthViewModel),
    Agenda(AgendaViewModel),
}

/// What is open over the view; like Google Calendar, one thing at a time.
enum Dialog {
    Shortcuts,
    /// The date typed so far, and whether the field has had the keyboard yet.
    GoToDate(String, bool),
    /// The event editor is `MOD-05`; until then this only says where the event would go.
    NewEvent,
    /// The event whose details are open, and what had the keyboard when it opened.
    Event(Event, Option<egui::Id>),
    /// The settings, where accounts are managed (`MOD-01-U01`).
    Settings,
}

/// Days the agenda shows when opened, and how many more each step of scrolling loads.
const AGENDA_DAYS: i64 = 30;

fn build(
    events: &[Event],
    navigation: Navigation,
    agenda_days: i64,
    zone: Tz,
    now: DateTime<Utc>,
) -> Built {
    let selected = navigation.selected;
    let week_start = navigation.week_start;
    match navigation.view {
        View::Grid(grid) => Built::Grid(grid_view(
            events,
            &grid.days(selected, week_start),
            zone,
            now,
        )),
        View::Month => Built::Month(month_view(events, selected, zone, now, week_start)),
        View::Agenda => Built::Agenda(agenda_view(events, zone, now, agenda_days)),
    }
}

struct CalendarApp {
    /// The connected Google accounts, remembered across restarts; none on a first run.
    accounts: Accounts,
    /// Saves the accounts off the UI thread; `None` when there is no home directory.
    accounts_writer: Option<Sender<Accounts>>,
    /// Asking Google whether the accounts still work.
    checks: Checks,
    /// Why the accounts could not be checked, until a check gets through.
    problem: Option<CheckError>,
    /// Connecting an account, from the welcome screen or the settings.
    connect: Connect,
    /// The account just connected, until the user closes the notice about it.
    connected: Option<Account>,
    /// Removing an account, from the settings.
    removal: Removal,
    /// Synchronizing with Google and reading the cache, off the UI thread.
    sync: Synchronization,
    /// What the views show, from the cache; empty until it is read.
    events: Vec<Event>,
    /// Whether the window had the keyboard at the last frame.
    focused: bool,
    /// The system's regional settings, read at start (`DEC-29`).
    regional: Regional,
    /// The system's time zone; followed when it changes.
    zone: Tz,
    zone_changes: Receiver<Tz>,
    navigation: Navigation,
    /// Any day of the month the mini calendar shows; follows the view, browsed on its own.
    mini_month: NaiveDate,
    /// Saves the settings off the UI thread; `None` when there is no home directory.
    settings_writer: Option<Sender<Settings>>,
    /// Days from today the agenda covers; grows as the user scrolls down.
    agenda_days: i64,
    built: Built,
    built_at: DateTime<Utc>,
    view_state: grid_view::State,
    month_state: month_view::State,
    dialog: Option<Dialog>,
}

impl CalendarApp {
    fn new(ctx: egui::Context) -> Self {
        let regional = Regional::from_env(|key| std::env::var(key).ok());
        let zone = system_zone();
        let zone_changes = watch_zone(zone, ctx.clone());
        let today = Utc::now().with_timezone(&zone).date_naive();
        let settings_path = settings::default_path();
        let last_view = settings_path
            .as_deref()
            .and_then(|path| Settings::load(path).last_view);
        let settings_writer = settings_path.map(|path| {
            let (sender, receiver) = mpsc::channel::<Settings>();
            // The UI never waits on disk.
            std::thread::spawn(move || {
                for settings in receiver {
                    // Losing the last view is acceptable (`DEC-15`); the app still works.
                    let _ = settings.save(&path);
                }
            });
            sender
        });
        let accounts_path = accounts::default_path();
        let accounts = accounts_path
            .as_deref()
            .map(Accounts::load)
            .unwrap_or_default();
        let account_ids = accounts
            .list()
            .iter()
            .map(|e| e.account.id.clone())
            .collect();
        let sync = Synchronization::start(&ctx, account_ids, Window::around(today, zone));
        let accounts_writer = accounts_path.map(|path| {
            let (sender, receiver) = mpsc::channel::<Accounts>();
            std::thread::spawn(move || {
                for accounts in receiver {
                    // A lost file only means connecting the accounts again.
                    let _ = accounts.save(&path);
                }
            });
            sender
        });
        let navigation = Navigation::start(last_view, today, regional.week_start);
        let built_at = Utc::now();
        let events = Vec::new();
        let built = build(&events, navigation, AGENDA_DAYS, zone, built_at);
        CalendarApp {
            accounts,
            accounts_writer,
            checks: Checks::new(),
            problem: None,
            connect: Connect::new(),
            connected: None,
            removal: Removal::new(),
            sync,
            events,
            focused: false,
            regional,
            zone,
            zone_changes,
            navigation,
            mini_month: today,
            settings_writer,
            agenda_days: AGENDA_DAYS,
            built,
            built_at,
            view_state: grid_view::State::default(),
            month_state: month_view::State::default(),
            dialog: None,
        }
    }

    fn rebuild(&mut self) {
        self.built_at = Utc::now();
        self.built = build(
            &self.events,
            self.navigation,
            self.agenda_days,
            self.zone,
            self.built_at,
        );
    }

    fn today(&self) -> NaiveDate {
        Utc::now().with_timezone(&self.zone).date_naive()
    }

    /// The days the cache holds, around today (`MOD-02-F11`).
    fn window(&self) -> Window {
        Window::around(self.today(), self.zone)
    }

    /// Google Calendar's shortcuts (`DEC-55`), unless a text field is taking the keys. Focus on a
    /// button or an event does not stop them.
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.close_dialog(ctx);
        }
        // Google ignores its keys pressed with a modifier; egui already drops Ctrl's text.
        if ctx.text_edit_focused() || ctx.input(|i| i.modifiers.alt) {
            return;
        }
        let typed: Vec<char> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::Text(text) => Some(text.chars()),
                    _ => None,
                })
                .flatten()
                .collect()
        });
        for command in typed.into_iter().filter_map(shortcuts::command) {
            match command {
                Command::GoToDate => {
                    let date = self.regional.numeric_date(self.navigation.selected);
                    self.dialog = Some(Dialog::GoToDate(date, false));
                }
                Command::NewEvent => self.dialog = Some(Dialog::NewEvent),
                Command::ShowShortcuts => self.dialog = Some(Dialog::Shortcuts),
                command => {
                    // Like Google Calendar, moving the view closes what is open over it.
                    self.dialog = None;
                    self.navigation.run(command, self.today());
                    if command == Command::Today {
                        // As the Today button: brings back a browsed mini calendar.
                        self.mini_month = self.navigation.selected;
                    }
                }
            }
        }
    }

    /// Closes the open dialog; like Google Calendar, an event's details give the keyboard back to
    /// the event.
    fn close_dialog(&mut self, ctx: &egui::Context) {
        if let Some(Dialog::Event(_, Some(id))) = self.dialog.take() {
            ctx.memory_mut(|m| m.request_focus(id));
        }
    }

    fn dialog(&mut self, ctx: &egui::Context) {
        // Connecting and removing an account never run at once.
        let can_change = self.connect.idle() && self.removal.idle();
        let can_remove = self.connect.idle();
        let Some(dialog) = &mut self.dialog else {
            return;
        };
        let mut open = true;
        let mut go_to = None;
        let mut again = None;
        let mut remove = None;
        let title = match dialog {
            Dialog::Shortcuts => "Keyboard shortcuts",
            Dialog::GoToDate(..) => "Go to date",
            Dialog::NewEvent => "New event",
            Dialog::Event(..) => "Event",
            Dialog::Settings => "Settings",
        };
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| match dialog {
                Dialog::Shortcuts => {
                    egui::Grid::new("shortcuts").show(ui, |ui| {
                        for (key, _, description) in SHORTCUTS {
                            ui.monospace(key.to_string());
                            ui.label(description);
                            ui.end_row();
                        }
                    });
                }
                Dialog::GoToDate(text, focused) => {
                    ui.label(format!(
                        "Date (as {})",
                        self.regional.numeric_date(self.navigation.selected)
                    ));
                    let field = ui.text_edit_singleline(text);
                    let date = self.regional.parse_date(text);
                    let entered =
                        field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    // The field takes the keyboard when the dialog opens and after a wrong date.
                    if !*focused || (entered && date.is_none()) {
                        field.request_focus();
                        *focused = true;
                    }
                    if ui
                        .add_enabled(date.is_some(), egui::Button::new("Go"))
                        .clicked()
                        || entered
                    {
                        go_to = date;
                    }
                }
                Dialog::NewEvent => {
                    ui.label(self.regional.long_date(self.navigation.selected));
                    ui.label("Creating events is not available yet.");
                }
                Dialog::Event(event, _) => {
                    let details = event_details(event, self.zone);
                    ui.heading(details.title.as_deref().unwrap_or("Busy"));
                    ui.label(details.when.describe(&self.regional));
                    if let Some(own) = details.in_own_zone(&self.regional) {
                        ui.label(own);
                    }
                    if details.read_only {
                        ui.label(grid_view::READ_ONLY);
                    }
                }
                Dialog::Settings => {
                    ui.heading("Accounts");
                    egui::Grid::new("accounts").show(ui, |ui| {
                        for entry in self.accounts.list() {
                            ui.label(&entry.account.email);
                            // An account's problem is shown on the account (`MOD-01-F11`).
                            ui.label(entry.state.label());
                            if entry.state == AccountState::NeedsSignIn {
                                if ui
                                    .add_enabled(
                                        can_change,
                                        egui::Button::new(accounts::SIGN_IN_AGAIN),
                                    )
                                    .clicked()
                                {
                                    again = Some(entry.account.email.clone());
                                }
                            } else {
                                ui.label("");
                            }
                            if ui
                                .add_enabled(can_change, egui::Button::new(accounts::REMOVE))
                                .clicked()
                            {
                                remove = Some(entry.account.clone());
                            }
                            ui.end_row();
                        }
                    });
                    self.removal.show(ui, can_remove);
                    if self.removal.idle() {
                        ui.add_space(8.0);
                        self.connect.show(ui, "Add account");
                    }
                }
            });
        if let Some(email) = again {
            self.sign_in_again(ctx, &email);
        }
        if let Some(account) = remove {
            self.removal.ask(account);
        }
        if let Some(date) = go_to {
            self.navigation.go_to(date);
            self.mini_month = date;
            open = false;
        }
        if !open {
            self.close_dialog(ctx);
        }
    }

    fn save_accounts(&self) {
        if let Some(writer) = &self.accounts_writer {
            let _ = writer.send(self.accounts.clone());
        }
    }

    /// The removed account's credentials are gone: the app forgets it, and with the last one gone
    /// the welcome screen comes back (`MOD-01-F14`).
    fn removed(&mut self, id: &str) {
        self.accounts.remove(id);
        self.save_accounts();
        let window = self.window();
        self.sync.forget(id, window);
        if self
            .connected
            .as_ref()
            .is_some_and(|account| account.id == id)
        {
            self.connected = None;
        }
        if self.accounts.is_empty() {
            // The settings close, so they do not reopen over the calendar after a new connection.
            self.dialog = None;
            self.connect.close();
        }
    }

    /// Opens the settings and signs in again to an account that needs it.
    fn sign_in_again(&mut self, ctx: &egui::Context, email: &str) {
        self.dialog = Some(Dialog::Settings);
        self.connect.sign_in_again(ctx, email);
    }

    /// Shows what `navigation` now points at: the mini calendar follows it, and a new view is
    /// remembered for the next start.
    fn navigated(&mut self, before: Navigation) {
        if self.navigation == before {
            return;
        }
        if self.navigation.view != before.view {
            self.agenda_days = AGENDA_DAYS;
            if let Some(writer) = &self.settings_writer {
                let _ = writer.send(Settings {
                    last_view: Some(self.navigation.view),
                });
            }
        }
        self.mini_month = self.navigation.selected;
        self.month_state = month_view::State::default();
        self.rebuild();
    }
}

impl eframe::App for CalendarApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(account) = self.connect.poll() {
            self.connected = Some(account.clone());
            self.accounts.add(account);
            self.checks.signed_in();
            // The secret store just kept a sign-in; the next check says whether a problem stays.
            self.problem = None;
            self.save_accounts();
        }
        if let Some(id) = self.removal.poll() {
            self.removed(&id);
        }
        match self.checks.poll() {
            Some(Ok(checked)) => {
                self.problem = None;
                if self.accounts.apply(&checked) {
                    self.save_accounts();
                }
                let window = self.window();
                self.sync.checked(&checked, window);
            }
            Some(Err(problem)) => self.problem = Some(problem),
            None => {}
        }
        self.checks.tick(ui.ctx(), &self.accounts);
        if let Some(events) = self.sync.poll() {
            self.events = events;
            self.rebuild();
        }
        // Looking at the app brings it up to date; a minimized window is polled less often.
        let (focused, minimized) = ui
            .ctx()
            .input(|i| (i.focused, i.viewport().minimized == Some(true)));
        if focused && !self.focused {
            self.sync.activated();
        }
        self.focused = focused;
        self.sync.set_shown(!minimized);
        let window = self.window();
        self.sync.set_window(window);
        // With no account connected there is only the welcome screen (`MOD-01-F12`).
        if self.accounts.is_empty() {
            connect::welcome(ui, &mut self.connect);
            return;
        }
        // Events move to their new local times when the system's time zone changes.
        if let Some(zone) = self.zone_changes.try_iter().last() {
            self.zone = zone;
            self.rebuild();
        }
        // The now line and today move; nothing else changes on its own.
        if Utc::now() - self.built_at >= chrono::Duration::minutes(1) {
            self.rebuild();
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(30));
        let before = self.navigation;
        self.shortcuts(ui.ctx());
        egui::Panel::top("view-bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Today").clicked() {
                    self.navigation.today(self.today());
                    // Brings back a mini calendar browsed away, even when the view stays put.
                    self.mini_month = self.navigation.selected;
                }
                // The agenda always starts now; there is no period to step through.
                let periods = self.navigation.view != View::Agenda;
                if ui.add_enabled(periods, egui::Button::new("‹")).clicked() {
                    self.navigation.previous();
                }
                if ui.add_enabled(periods, egui::Button::new("›")).clicked() {
                    self.navigation.next();
                }
                let regional = &self.regional;
                ui.label(match (self.navigation.view, self.navigation.period()) {
                    (View::Month, _) => regional.month_year(self.navigation.selected),
                    (_, Some((first, last))) if first == last => regional.short_date(first),
                    (_, Some((first, last))) => regional.date_range(first, last),
                    (_, None) => "Upcoming".to_string(),
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Settings").clicked() {
                        self.dialog = Some(Dialog::Settings);
                    }
                    if ui.button("Refresh").clicked() {
                        self.sync.refresh();
                    }
                    for view in View::ALL.into_iter().rev() {
                        if ui
                            .selectable_label(self.navigation.view == view, view.name())
                            .clicked()
                        {
                            self.navigation.switch(view);
                        }
                    }
                });
            });
        });
        // The calendar keeps showing what it has; only the accounts cannot be reached (`DEC-52`).
        if let Some(problem) = &self.problem {
            let mut again = false;
            egui::Panel::top("account-problem").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(ui.visuals().warn_fg_color, problem.message());
                    again = ui.button("Try again").clicked();
                });
            });
            if again {
                self.problem = None;
                self.checks.now();
            }
        }
        // The user is told about every account Google turned away (`MOD-01-F05`).
        let mut again = None;
        let turned_away: Vec<&str> = self
            .accounts
            .list()
            .iter()
            .filter(|entry| entry.state == AccountState::NeedsSignIn)
            .map(|entry| entry.account.email.as_str())
            .collect();
        if !turned_away.is_empty() {
            egui::Panel::top("needs-sign-in").show(ui, |ui| {
                for email in turned_away {
                    ui.horizontal_wrapped(|ui| {
                        let warn = ui.visuals().warn_fg_color;
                        ui.colored_label(warn, accounts::needs_sign_in(email));
                        let button = egui::Button::new(accounts::SIGN_IN_AGAIN);
                        let can_change = self.connect.idle() && self.removal.idle();
                        if ui.add_enabled(can_change, button).clicked() {
                            again = Some(email.to_string());
                        }
                    });
                }
            });
        }
        if let Some(email) = again {
            self.sign_in_again(ui.ctx(), &email);
        }
        if let Some(account) = &self.connected {
            let mut close = false;
            egui::Panel::top("connected-account").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("Connected {}.", account.email));
                    let unavailable = account.unavailable();
                    if !unavailable.is_empty() {
                        ui.label(format!(
                            "Not available for this account: {}.",
                            unavailable.join("; ")
                        ));
                    }
                    close = ui.small_button("✕").clicked();
                });
            });
            if close {
                self.connected = None;
            }
        }
        egui::Panel::left("side-panel")
            .resizable(false)
            .show(ui, |ui| {
                let today = self.today();
                let mini = mini_calendar(self.mini_month, &self.navigation, today);
                match mini_calendar::show(ui, &mini, &self.regional) {
                    Some(mini_calendar::Action::Browse(month)) => self.mini_month = month,
                    Some(mini_calendar::Action::GoTo(date)) => {
                        self.navigation.go_to(date);
                        self.mini_month = date;
                    }
                    None => {}
                }
            });
        self.dialog(ui.ctx());
        if !matches!(self.dialog, Some(Dialog::Settings)) {
            self.connect.close();
            self.removal.close();
        }
        self.navigated(before);
        let mut opened = None;
        egui::CentralPanel::default().show(ui, |ui| match &self.built {
            Built::Grid(view) => {
                if let Some(change) =
                    grid_view::show(ui, view, &self.regional, &mut self.view_state, &mut opened)
                {
                    grid_view::apply(&mut self.events, change);
                    self.rebuild();
                }
            }
            Built::Month(view) => {
                month_view::show(ui, view, &self.regional, &mut self.month_state, &mut opened)
            }
            Built::Agenda(view) => {
                if agenda_view::show(ui, view, &self.regional, &mut opened) {
                    self.agenda_days += AGENDA_DAYS;
                    self.rebuild();
                    ui.ctx().request_repaint();
                }
            }
        });
        if let Some(event) = opened {
            // The editor opens from here in `MOD-05`; the event is brought up to date first.
            self.sync.opening(&self.events[event].calendar_id);
            let focused = ui.ctx().memory(|m| m.focused());
            self.dialog = Some(Dialog::Event(self.events[event].clone(), focused));
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("Calendar Buddy")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([480.0, 360.0]),
        ..Default::default()
    };
    eframe::run_native(
        "calendar-buddy",
        options,
        Box::new(|cc| Ok(Box::new(CalendarApp::new(cc.egui_ctx.clone())))),
    )
}
