mod agenda_view;
mod calendar_list;
mod checks;
mod connect;
mod editor;
mod grid_view;
mod mini_calendar;
mod month_view;
mod network;
mod platform;
mod remove;
mod sync;

use calendar_core::accounts::{self, Account, AccountState, Accounts, Check, CheckError};
use calendar_core::agenda::{AgendaViewModel, agenda_view};
use calendar_core::colors::{self, LocalColors};
use calendar_core::create::{self, NO_CALENDAR, QuickEntry, Saving};
use calendar_core::details::event_details;
use calendar_core::drag::{self, Target};
use calendar_core::editor::{Edit, Editor, Save};
use calendar_core::guests::{self, Person, Reach};
use calendar_core::model::{Calendar, Event};
use calendar_core::month::{MonthViewModel, month_view};
use calendar_core::navigation::{Navigation, View, mini_calendar};
use calendar_core::offline;
use calendar_core::platform::Browser;
use calendar_core::regional::Regional;
use calendar_core::series::{Question, Scope};
use calendar_core::settings::{self, Settings};
use calendar_core::shortcuts::{self, Command, SHORTCUTS};
use calendar_core::side_panel::{self, Panel};
use calendar_core::status::{self, State};
use calendar_core::sync::Window;
use calendar_core::undo::{LastChange, Undo};
use calendar_core::view_model::{GridViewModel, grid_view};
use checks::Checks;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use connect::Connect;
use eframe::egui;
use network::Online;
use platform::{SystemBrowser, SystemNetwork};
use remove::Removal;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;
use sync::{Saved, Synchronization};

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
    /// The quick-entry box of a new event (`MOD-05-F02`), and whether its title has had the
    /// keyboard yet.
    QuickEntry(QuickEntry, bool),
    /// The full editor of a new or an existing event (`MOD-05-F04`).
    Editor(Box<Editor>),
    /// The event whose details are open, and what had the keyboard when it opened.
    Event(Event, Option<egui::Id>),
    /// The settings, where accounts are managed (`MOD-01-U01`).
    Settings,
    /// Which events of a series a change or deletion of one of its occurrences applies to
    /// (`MOD-05-F20`, `MOD-05-F21`), and the editor the change was made in, which Cancel goes
    /// back to.
    Scope(Question, Option<Box<Editor>>),
}

/// Ctrl+Z: undoes the last change to an event (`MOD-05-F28`); not Ctrl+Shift+Z, there is no redo.
fn undo_pressed(input: &mut egui::InputState) -> bool {
    !input.modifiers.shift && input.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)
}

/// Days the agenda shows when opened, and how many more each step of scrolling loads.
const AGENDA_DAYS: i64 = 30;

/// The view's model, of the events of calendars neither hidden in the side panel
/// (`MOD-03-F12`) nor disabled in the settings (`MOD-03-F13`), each in its calendar's local
/// colour where it has one; with the places in `events` of the events the model's indices point
/// to.
fn build(
    events: &[Event],
    settings: &Settings,
    navigation: Navigation,
    agenda_days: i64,
    zone: Tz,
    now: DateTime<Utc>,
) -> (Built, Vec<usize>) {
    let places = side_panel::shown(events, settings);
    let events: &Vec<Event> = &colors::apply(events, &settings.colors)
        .into_iter()
        .enumerate()
        .filter_map(|(i, event)| places.binary_search(&i).is_ok().then_some(event))
        .collect();
    let selected = navigation.selected;
    let week_start = navigation.week_start;
    let built = match navigation.view {
        View::Grid(grid) => Built::Grid(grid_view(
            events,
            &grid.days(selected, week_start),
            zone,
            now,
        )),
        View::Month => Built::Month(month_view(events, selected, zone, now, week_start)),
        View::Agenda => Built::Agenda(agenda_view(events, zone, now, agenda_days)),
    };
    (built, places)
}

/// A drag left `event` where it was: when the user may not change it, the notices say why, once
/// however often it is tried (`MOD-05-F17`).
fn tell_unmovable(notices: &mut Vec<String>, event: &Event) {
    if let Some(protected) = event.protected() {
        let why = protected.why().to_string();
        if !notices.contains(&why) {
            notices.push(why);
        }
    }
}

/// Keeps a calendar's local colour picked in its menu, or forgets it on "Reset"; only in the
/// app's settings, never sent to Google (`MOD-02-F04`).
fn pick(colors: &mut LocalColors, picked: calendar_list::Picked) {
    let key = (picked.account_id, picked.calendar_id);
    match picked.color {
        Some(color) => colors.insert(key, color),
        None => colors.remove(&key),
    };
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
    /// What the views show: the events from the cache, empty until it is read, but those changed
    /// and not synchronized back yet, followed by the new and changed events not synchronized
    /// yet.
    events: Vec<Event>,
    /// The events from the cache.
    cached: Vec<Event>,
    /// New, changed and deleted events sent to Google and not synchronized back yet
    /// (`MOD-05-U03`).
    saving: Saving,
    /// The last change to an event, and what undoes it (`MOD-05-F28`).
    last: LastChange,
    /// What could not be saved, deleted, undone or changed, and why, until the user closes the
    /// notice.
    not_saved: Vec<String>,
    /// Whether the computer is online, as the system says (`MOD-02-F14`).
    online: Online,
    /// Whether the window had the keyboard at the last frame.
    focused: bool,
    /// The system's regional settings, read at start (`DEC-29`).
    regional: Regional,
    /// The system's time zone; followed when it changes.
    zone: Tz,
    zone_changes: Receiver<Tz>,
    navigation: Navigation,
    /// Whether the side panel is shown.
    panel: Panel,
    /// Any day of the month the mini calendar shows; follows the view, browsed on its own.
    mini_month: NaiveDate,
    /// The app's settings, as last saved.
    settings: Settings,
    /// Saves the settings off the UI thread; `None` when there is no home directory.
    settings_writer: Option<Sender<Settings>>,
    /// The names of calendars no longer available, until the user closes the notice
    /// (`MOD-02-F21`).
    gone: Vec<String>,
    /// Days from today the agenda covers; grows as the user scrolls down.
    agenda_days: i64,
    built: Built,
    /// The places in `events` of the events the built view's indices point to.
    places: Vec<usize>,
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
        let settings = settings_path
            .as_deref()
            .map(Settings::load)
            .unwrap_or_default();
        let settings_writer = settings_path.map(|path| {
            let (sender, receiver) = mpsc::channel::<Settings>();
            // The UI never waits on disk.
            std::thread::spawn(move || {
                for settings in receiver {
                    // Losing the settings is acceptable (`DEC-15`); the app still works.
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
        let mut sync = Synchronization::start(
            &ctx,
            account_ids,
            Window::around(today, zone),
            settings.disabled.clone(),
        );
        // Accounts Google turned away in an earlier run are not checked, nor synchronized: their
        // calendars are marked as not synchronizing.
        let turned_away: Vec<(String, Check)> = accounts
            .list()
            .iter()
            .filter(|entry| entry.state == AccountState::NeedsSignIn)
            .map(|entry| (entry.account.id.clone(), Check::SignInNeeded))
            .collect();
        sync.checked(&turned_away, Window::around(today, zone));
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
        let navigation = Navigation::start(settings.last_view, today, regional.week_start);
        let built_at = Utc::now();
        let events = Vec::new();
        let (built, places) = build(&events, &settings, navigation, AGENDA_DAYS, zone, built_at);
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
            cached: Vec::new(),
            saving: Saving::default(),
            last: LastChange::default(),
            not_saved: Vec::new(),
            online: Online::watch(&ctx, &SystemNetwork),
            focused: false,
            regional,
            zone,
            zone_changes,
            navigation,
            panel: Panel::new(settings.side_panel_collapsed),
            mini_month: today,
            settings,
            settings_writer,
            gone: Vec::new(),
            agenda_days: AGENDA_DAYS,
            built,
            places,
            built_at,
            view_state: grid_view::State::default(),
            month_state: month_view::State::default(),
            dialog: None,
        }
    }

    fn rebuild(&mut self) {
        self.built_at = Utc::now();
        (self.built, self.places) = build(
            &self.events,
            &self.settings,
            self.navigation,
            self.agenda_days,
            self.zone,
            self.built_at,
        );
    }

    /// The cached events but those changed meanwhile, followed by the new and changed events not
    /// synchronized yet; `cached` when the cache changed.
    fn show_events(&mut self, cached: Option<Vec<Event>>) {
        if let Some(events) = cached {
            self.saving.synced(&events);
            self.cached = events;
        }
        self.events = self.saving.shown(&self.cached);
        self.view_state.events_changed();
        self.rebuild();
    }

    /// Saves the event at `event` in `events`, dragged and dropped (`MOD-05-F13`, `MOD-05-F14`):
    /// shown at once where it was dropped, marked "saving" until Google confirms it
    /// (`MOD-05-U03`).
    fn dropped(&mut self, event: usize, drop: Target) {
        let event = &self.events[event];
        let calendar = self.sync.calendars().iter().find(|calendar| {
            calendar.account_id == event.account_id && calendar.id == event.calendar_id
        });
        let length = self.sync.default_length(&event.account_id);
        let (Some(calendar), Some(timing)) =
            (calendar, drag::dropped(event, drop, self.zone, length))
        else {
            return;
        };
        match Question::changing(drag::edit(event, calendar.clone(), timing)) {
            Ok(edit) => self.change(Box::new(edit)),
            // An occurrence stays where it was until the user says which events move.
            Err(question) => self.dialog = Some(Dialog::Scope(question, None)),
        }
    }

    /// Saves the change to the event: shown changed at once, marked "saving" until Google
    /// confirms it (`MOD-05-U03`).
    fn change(&mut self, edit: Box<Edit>) {
        let local = self.saving.change(&edit.event, edit.changed());
        self.last.changing(local, &edit);
        self.sync.change(local, edit);
        self.show_events(None);
    }

    /// Saves the change to or deletion of an occurrence for the events the user chose
    /// (`MOD-05-F20`, `MOD-05-F21`). One of the following or all events cannot be undone: as
    /// after creating an event, nothing is left to undo.
    fn answered(&mut self, question: Question, scope: Scope) {
        match (question, scope) {
            (Question::Change(edit), Scope::This) => self.change(edit),
            (Question::Delete(deleted), Scope::This) => {
                let (calendar, event) = *deleted;
                self.delete(&event, calendar);
            }
            (Question::Change(edit), scope) => {
                let local = self.saving.change_series(&edit.event, edit.changed());
                self.last.creating();
                self.sync.change_series(local, edit, scope);
                self.show_events(None);
            }
            (Question::Delete(deleted), scope) => {
                let (calendar, event) = *deleted;
                let local = self.saving.delete(&event);
                self.last.creating();
                self.sync.delete_series(local, calendar, event, scope);
                self.show_events(None);
            }
        }
    }

    /// Opens the series of the occurrence in Google Calendar's editor in the browser, as its
    /// account, where its repetition is changed (`MOD-05-F29`, `DEC-111`).
    fn open_series(&mut self, event: &Event) {
        let (Some((series, _)), Some(entry)) = (
            &event.series,
            (self.accounts.list().iter()).find(|entry| entry.account.id == event.account_id),
        ) else {
            return;
        };
        let page = calendar_google::edit_page(&event.calendar_id, series, &entry.account.email);
        if let Err(error) = SystemBrowser.open(&page) {
            (self.not_saved).push(format!("Google Calendar could not be opened: {error}."));
        }
    }

    /// Deletes the event from its calendar, without a question (`MOD-05-F12`, `MOD-05-F27`): gone
    /// at once, and back if Google refuses.
    fn delete(&mut self, event: &Event, calendar: Calendar) {
        let local = self.saving.delete(event);
        self.last.deleting(local, &calendar, event);
        self.sync.delete(local, calendar, event.clone());
        self.show_events(None);
    }

    /// Ctrl+Z or the notice's Undo: the last change is undone at Google, shown at once, marked
    /// "saving" until Google confirms it (`MOD-05-F28`).
    fn undo(&mut self) {
        let local = match self.last.undo() {
            Some(Undo::Restore(deleted)) => {
                let (calendar, event) = *deleted;
                let local = self.saving.change(&event, event.clone());
                self.sync.restore(local, calendar, event);
                local
            }
            Some(Undo::Revert(edit)) => {
                let local = self.saving.change(&edit.event, edit.changed());
                self.sync.change(local, edit);
                local
            }
            None => return,
        };
        self.last.undoing(local);
        self.show_events(None);
    }

    /// Opens the quick-entry box at the times `times` gives for the default length of the
    /// default calendar's account (`MOD-05-F09`, `MOD-05-F10`).
    fn start_event(&mut self, times: impl FnOnce(u32) -> (DateTime<Utc>, DateTime<Utc>)) {
        let calendar =
            create::default_calendar(self.accounts.list(), self.sync.calendars(), &self.settings);
        let length = calendar
            .as_ref()
            .map_or(create::DEFAULT_LENGTH, |calendar| {
                self.sync.default_length(&calendar.account_id)
            });
        let entry = QuickEntry::new(times(length), self.zone, calendar);
        self.dialog = Some(Dialog::QuickEntry(entry, false));
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
        // Offline the undo waits, as changes need Google (`MOD-02-F15`).
        if self.online.get() && ctx.input_mut(undo_pressed) {
            self.undo();
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
                Command::NewEvent => {
                    let (date, zone) = (self.navigation.selected, self.zone);
                    self.start_event(|length| create::keyed(date, Utc::now(), length, zone));
                }
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
    /// the event, and the scope question goes back to the editor.
    fn close_dialog(&mut self, ctx: &egui::Context) {
        match self.dialog.take() {
            Some(Dialog::Event(_, Some(id))) => ctx.memory_mut(|m| m.request_focus(id)),
            // The scope question goes back to the editor the change was made in.
            Some(Dialog::Scope(_, Some(editor))) => self.dialog = Some(Dialog::Editor(editor)),
            _ => {}
        }
    }

    fn dialog(&mut self, ctx: &egui::Context) {
        // Connecting and removing an account never run at once.
        let can_change = self.connect.idle() && self.removal.idle();
        let can_remove = self.connect.idle();
        // Changes need Google (`MOD-02-F15`).
        let refused = offline::refused(self.online.get());
        let Some(dialog) = &mut self.dialog else {
            return;
        };
        let mut open = true;
        let mut go_to = None;
        let mut again = None;
        let mut remove = None;
        let mut toggled = None;
        let mut save = None;
        let mut edit = None;
        let mut delete = None;
        let mut default_calendar = None;
        let mut scope = None;
        let mut cancelled = false;
        let mut open_series = None;
        let mut attachment = None;
        let mut search = None;
        let title = match dialog {
            Dialog::Shortcuts => "Keyboard shortcuts",
            Dialog::GoToDate(..) => "Go to date",
            Dialog::QuickEntry(..) => "New event",
            Dialog::Editor(editor) if editor.opened.is_none() => "New event",
            Dialog::Editor(_) => "Edit event",
            Dialog::Event(..) => "Event",
            Dialog::Settings => "Settings",
            Dialog::Scope(Question::Change(_), _) => "Edit recurring event",
            Dialog::Scope(Question::Delete(_), _) => "Delete recurring event",
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
                Dialog::QuickEntry(entry, focused) => {
                    let field =
                        ui.add(egui::TextEdit::singleline(&mut entry.title).hint_text("Add title"));
                    // The title takes the keyboard when the box opens.
                    if !*focused {
                        field.request_focus();
                        *focused = true;
                    }
                    ui.label(entry.when().describe(&self.regional));
                    let choices = create::choices(
                        self.accounts.list(),
                        self.sync.calendars(),
                        &self.settings,
                    );
                    calendar_list::picker(ui, "new-event-calendar", &choices, &mut entry.calendar);
                    // No network, no calendar: creating is unavailable, and the box says why.
                    let unavailable = refused.or(entry.calendar.is_none().then_some(NO_CALENDAR));
                    if let Some(reason) = unavailable {
                        ui.label(reason);
                    }
                    let entered =
                        field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let clicked = ui
                        .horizontal(|ui| {
                            // The full editor, keeping what was typed (`MOD-05-F03`).
                            if ui.button("More options").clicked() {
                                edit = Some(Editor::from_quick(entry, &self.regional));
                            }
                            let button = egui::Button::new("Save");
                            ui.add_enabled(unavailable.is_none(), button).clicked()
                        })
                        .inner;
                    if (clicked || entered) && unavailable.is_none() {
                        save = entry
                            .new_event()
                            .map(|(calendar, event)| Save::Create(calendar, Box::new(event)));
                    }
                }
                Dialog::Editor(editor) => {
                    let choices = editor.choices(create::choices(
                        self.accounts.list(),
                        self.sync.calendars(),
                        &self.settings,
                    ));
                    let accounts = self.accounts.list();
                    let typed = editor.guest_field.trim().to_string();
                    let account = editor.calendar.as_ref().map(|c| c.account_id.clone());
                    // The people Google found, then the account's recent guests (`MOD-06-F03`).
                    let found: Vec<Person> = match &account {
                        Some(id) if !typed.is_empty() => (self.sync.people(id).iter().cloned())
                            .chain(guests::recent(&self.cached, id))
                            .collect(),
                        _ => Vec::new(),
                    };
                    let own = editor.own_addresses(accounts);
                    let suggested = guests::suggestions(&typed, &found, &editor.guests, &own);
                    let regional = &self.regional;
                    match editor::show(
                        ui, editor, regional, &choices, accounts, refused, &suggested,
                    ) {
                        Some(editor::Edited::Save(saved)) => save = Some(saved),
                        Some(editor::Edited::OpenSeries) => {
                            open_series = editor.opened.as_ref().map(|(event, _)| event.clone());
                        }
                        None => {}
                    }
                    // What is typed is searched for at Google as it changes, and in the account of
                    // another calendar picked (`MOD-06-F02`).
                    let now_typed = editor.guest_field.trim();
                    let now_account = editor.calendar.as_ref().map(|c| c.account_id.clone());
                    if let Some(id) = now_account
                        && (now_typed != typed || account.as_ref() != Some(&id))
                        && !now_typed.is_empty()
                    {
                        search = Some((id, now_typed.to_string()));
                    }
                }
                Dialog::Event(event, _) => {
                    let details = event_details(event, self.zone);
                    ui.heading(details.title.as_deref().unwrap_or("Busy"));
                    ui.label(details.when.describe(&self.regional));
                    if let Some(own) = details.in_own_zone(&self.regional) {
                        ui.label(own);
                    }
                    // Each attachment opens in the browser (`MOD-05-F26`).
                    for listed in &details.attachments {
                        if ui.link(format!("📎 {}", listed.title)).clicked() {
                            attachment = Some(listed.url.clone());
                        }
                    }
                    // The event as last synchronized, which may have come since it was opened.
                    let event = self
                        .events
                        .iter()
                        .find(|e| {
                            e.id == event.id
                                && e.calendar_id == event.calendar_id
                                && e.account_id == event.account_id
                        })
                        .unwrap_or(event);
                    // Editing is not offered for an event the user may not change (`MOD-05-F17`).
                    let protected = event.protected();
                    if let Some(protected) = protected {
                        ui.label(protected.why());
                    } else if let Some(reason) = refused {
                        ui.label(reason);
                    }
                    let calendar = self.sync.calendars().iter().find(|calendar| {
                        calendar.account_id == event.account_id && calendar.id == event.calendar_id
                    });
                    let not_yet = self.saving.holds(event).then_some(create::SAVING);
                    if let (None, Some(calendar)) = (protected, calendar) {
                        let can_edit = refused.is_none() && not_yet.is_none();
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(can_edit, egui::Button::new("Edit"))
                                .clicked()
                            {
                                edit = Some(Editor::of(event, calendar.clone(), &self.regional));
                            }
                            // Also Delete or Backspace, as in Google Calendar.
                            let keyed = ui.input(|i| {
                                i.key_pressed(egui::Key::Delete)
                                    || i.key_pressed(egui::Key::Backspace)
                            });
                            let clicked = ui
                                .add_enabled(can_edit, egui::Button::new("Delete"))
                                .clicked();
                            if can_edit && (clicked || keyed) {
                                delete = Some((event.clone(), calendar.clone()));
                            }
                        });
                        if let Some(why) = not_yet {
                            ui.label(why);
                        }
                    }
                }
                Dialog::Scope(question, _) => {
                    let scopes = question.scopes();
                    for (choice, why) in &scopes {
                        let button = egui::Button::new(choice.label());
                        if ui
                            .add_enabled(why.is_none() && refused.is_none(), button)
                            .clicked()
                        {
                            scope = Some(*choice);
                        }
                    }
                    let mut reasons: Vec<&str> =
                        scopes.iter().filter_map(|(_, why)| *why).collect();
                    reasons.dedup();
                    for why in reasons.into_iter().chain(refused) {
                        ui.label(why);
                    }
                    cancelled = ui.button("Cancel").clicked();
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
                    ui.add_space(8.0);
                    ui.heading("Calendars");
                    let calendars = side_panel::in_settings(
                        self.accounts.list(),
                        self.sync.calendars(),
                        &self.settings,
                    );
                    toggled = calendar_list::in_settings(ui, &calendars);
                    ui.add_space(8.0);
                    // Where new events go (`MOD-05-F10`).
                    ui.horizontal(|ui| {
                        ui.label("Default calendar");
                        let list = (self.accounts.list(), self.sync.calendars());
                        let choices = create::choices(list.0, list.1, &self.settings);
                        let mut picked = create::default_calendar(list.0, list.1, &self.settings);
                        if calendar_list::picker(ui, "default-calendar", &choices, &mut picked) {
                            default_calendar = picked;
                        }
                    });
                }
            });
        if let Some(email) = again {
            self.sign_in_again(ctx, &email);
        }
        if let Some(account) = remove {
            self.removal.ask(account);
        }
        if let Some(calendar) = default_calendar {
            self.settings.default_calendar = Some((calendar.account_id, calendar.id));
            self.save_settings();
        }
        match save {
            Some(Save::Create(calendar, event)) => {
                // Shown at once, marked "saving" until Google confirms it (`MOD-05-U03`).
                let local = self.saving.add(&calendar, &event);
                self.last.creating();
                self.sync.create(local, calendar, *event);
                self.show_events(None);
                open = false;
            }
            Some(Save::Change(change)) if change.is_empty() => open = false,
            Some(Save::Change(change)) => match Question::changing(*change) {
                Ok(change) => {
                    self.change(Box::new(change));
                    open = false;
                }
                Err(question) => {
                    let editor = match self.dialog.take() {
                        Some(Dialog::Editor(editor)) => Some(editor),
                        _ => None,
                    };
                    self.dialog = Some(Dialog::Scope(question, editor));
                }
            },
            None => {}
        }
        if cancelled {
            open = false;
        }
        if let Some(chosen) = scope
            && let Some(Dialog::Scope(question, _)) = self.dialog.take()
        {
            self.answered(question, chosen);
            open = false;
        }
        if let Some(event) = open_series {
            self.open_series(&event);
        }
        if let Some((id, query)) = search {
            // As far as the account allowed at sign-in (`DEC-52`).
            let reach = (self.accounts.list().iter())
                .find(|entry| entry.account.id == id)
                .map(|entry| Reach::of(&entry.account))
                .unwrap_or_default();
            self.sync.suggest(&id, &query, reach);
        }
        if let Some(url) = attachment
            && let Err(error) = SystemBrowser.open(&url)
        {
            (self.not_saved).push(format!("The attachment could not be opened: {error}."));
        }
        if let Some(editor) = edit {
            self.dialog = Some(Dialog::Editor(Box::new(editor)));
        }
        if let Some((event, calendar)) = delete {
            match Question::deleting(calendar, event) {
                Ok((calendar, event)) => {
                    self.delete(&event, calendar);
                    open = false;
                }
                Err(question) => self.dialog = Some(Dialog::Scope(question, None)),
            }
        }
        if let Some(calendar) = toggled {
            // Disabled, it is no longer synchronized; enabled, it is synchronized at once (`DEC-94`).
            side_panel::toggle(&mut self.settings.disabled, &calendar);
            self.save_settings();
            self.sync.set_disabled(self.settings.disabled.clone());
            self.rebuild();
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

    fn save_settings(&self) {
        if let Some(writer) = &self.settings_writer {
            let _ = writer.send(self.settings.clone());
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
        self.saving.forget_account(id);
        // Its events can no longer be changed back.
        self.last = LastChange::default();
        self.show_events(None);
        let disabled = self.settings.disabled.clone();
        if self.settings.forget_account(id) {
            self.save_settings();
            if self.settings.disabled != disabled {
                self.sync.set_disabled(self.settings.disabled.clone());
            }
        }
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
            self.settings.last_view = Some(self.navigation.view);
            self.save_settings();
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
        let cached = self.sync.poll();
        let saved = self.sync.take_saved();
        let now = Instant::now();
        for saved in &saved {
            match saved {
                Saved::Created(local, id) => self.saving.confirmed(*local, id),
                Saved::Changed(local, etag) => {
                    self.saving.changed(*local, etag);
                    self.last.changed(*local, etag, now);
                }
                Saved::Deleted(local) => {
                    self.saving.deleted(*local);
                    self.last.deleted(*local, now);
                }
                Saved::NotCreated(local, reason) | Saved::NotChanged(local, reason) => {
                    self.saving.refused(*local);
                    self.last.refused(*local);
                    self.not_saved.push(if self.last.is_undo(*local) {
                        format!("The change could not be undone: {reason}.")
                    } else {
                        format!("The event could not be saved: {reason}.")
                    });
                }
                Saved::NotDeleted(local, reason) => {
                    self.saving.refused(*local);
                    self.last.refused(*local);
                    self.not_saved
                        .push(format!("The event could not be deleted: {reason}."));
                }
            }
        }
        if cached.is_some() || !saved.is_empty() {
            self.show_events(cached);
        }
        self.gone.extend(self.sync.take_gone());
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
        let online = self.online.get();
        let sync_status = self.sync.status().clone();
        self.panel.resized(ui.ctx().content_rect().width());
        egui::Panel::top("view-bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                // Google Calendar's main menu button, at the far left.
                let hint = if self.panel.expanded() {
                    "Hide the side panel"
                } else {
                    "Show the side panel"
                };
                if ui.button("☰").on_hover_text(hint).clicked() {
                    self.settings.side_panel_collapsed = self.panel.toggle();
                    self.save_settings();
                }
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
                    // The synchronization indicator (`MOD-02-F17`, `MOD-02-U03`).
                    let today = self.today();
                    ui.label(status::last_synced(
                        sync_status.last_synced,
                        self.zone,
                        today,
                        &self.regional,
                    ));
                    // Accounts that cannot be checked are not synchronized either (`DEC-52`).
                    let state = match sync_status.state(online) {
                        State::Offline => State::Offline,
                        _ if self.problem.is_some() => State::Error,
                        state => state,
                    };
                    let label = status::label(state);
                    if matches!(state, State::Delayed | State::Error) {
                        ui.colored_label(ui.visuals().warn_fg_color, label);
                    } else {
                        ui.label(label);
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
        // Everything cached can still be browsed (`MOD-02-U02`, `MOD-02-F15`).
        if let Some(indicator) = offline::indicator(online) {
            egui::Panel::top("offline").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(ui.visuals().warn_fg_color, indicator);
                    ui.label(offline::NO_CHANGES);
                });
            });
        }
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
        // The notice that follows a change, with its Undo, for 5 seconds (`MOD-05-U04`).
        if let Some((says, left)) = self.last.notice(Instant::now()) {
            let mut undo = false;
            egui::Panel::top("undo").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(says);
                    undo = ui.add_enabled(online, egui::Button::new("Undo")).clicked();
                });
            });
            ui.ctx().request_repaint_after(left);
            if undo {
                self.undo();
            }
        }
        // A new event Google did not create is gone from the views, and one it did not change or
        // delete is shown as it was; the user is told why.
        if !self.not_saved.is_empty() {
            let mut close = false;
            egui::Panel::top("not-saved").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for message in &self.not_saved {
                        let warn = ui.visuals().warn_fg_color;
                        ui.colored_label(warn, message);
                    }
                    close = ui.small_button("✕").clicked();
                });
            });
            if close {
                self.not_saved.clear();
            }
        }
        // Calendars the user no longer has are told of once (`MOD-02-F21`).
        if !self.gone.is_empty() {
            let mut close = false;
            egui::Panel::top("calendars-gone").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for name in &self.gone {
                        ui.label(format!("The calendar “{name}” is no longer available."));
                    }
                    close = ui.small_button("✕").clicked();
                });
            });
            if close {
                self.gone.clear();
            }
        }
        let mut expanded = self.panel.expanded();
        egui::Panel::left("side-panel")
            .resizable(false)
            .show_collapsible(ui, &mut expanded, |ui| {
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
                ui.separator();
                // A problem with one calendar is told on that calendar (`MOD-02-F18`).
                let groups = side_panel::groups(
                    self.accounts.list(),
                    self.sync.calendars(),
                    &self.settings,
                    &sync_status,
                    online,
                );
                match calendar_list::show(ui, &groups, &self.settings.colors) {
                    Some(calendar_list::Action::Picked(picked)) => {
                        pick(&mut self.settings.colors, picked)
                    }
                    Some(calendar_list::Action::Toggled(calendar)) => {
                        side_panel::toggle(&mut self.settings.hidden, &calendar)
                    }
                    None => return,
                }
                self.save_settings();
                self.rebuild();
            });
        self.dialog(ui.ctx());
        if !matches!(self.dialog, Some(Dialog::Settings)) {
            self.connect.close();
            self.removal.close();
        }
        self.navigated(before);
        // Days on screen the cache does not hold: fetched online, a note offline (`MOD-02-F12`).
        let (first, last) = self.navigation.on_screen(self.today(), self.agenda_days);
        let browsed = offline::browsed(self.window(), first, last, online);
        self.sync.browse(browsed.fetch);
        let mut opened = None;
        let mut started = None;
        let mut dropped = None;
        let mut unmovable = None;
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(note) = browsed.note {
                ui.colored_label(ui.visuals().warn_fg_color, note);
            }
            if side_panel::nothing_shown(self.sync.calendars(), &self.settings) {
                ui.label(side_panel::NOTHING_SHOWN);
            }
            match &self.built {
                Built::Grid(view) => {
                    // Offline, events cannot be moved (`MOD-02-F15`), also by a drag under way.
                    let state = &mut self.view_state;
                    match grid_view::show(ui, view, &self.regional, state, online, &mut opened) {
                        Some(grid_view::Action::Dropped { event, drop }) if online => {
                            dropped = Some((self.places[event], drop));
                        }
                        Some(grid_view::Action::Unmovable { event }) => {
                            unmovable = Some(self.places[event]);
                        }
                        // A click or a selection in the hours starts a new event (`MOD-05-F01`).
                        Some(grid_view::Action::Clicked { day, minute }) => {
                            started = view.days.get(day).map(|d| (d.date, minute, None));
                        }
                        Some(grid_view::Action::Selected {
                            day,
                            pressed,
                            released,
                        }) => {
                            started = view
                                .days
                                .get(day)
                                .map(|d| (d.date, pressed, Some(released)))
                        }
                        _ => {}
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
            }
        });
        if let Some((event, drop)) = dropped {
            self.dropped(event, drop);
        }
        if let Some(event) = unmovable {
            tell_unmovable(&mut self.not_saved, &self.events[event]);
        }
        if let Some((date, pressed, released)) = started {
            let zone = self.zone;
            self.start_event(|length| match released {
                Some(released) => create::selected(date, pressed, released, zone),
                None => create::clicked(date, pressed, length, zone),
            });
        }
        if let Some(event) = opened {
            let event = self.places[event];
            // The event is brought up to date first, as the editor opens from its details.
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
