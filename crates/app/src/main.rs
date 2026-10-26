mod agenda_view;
mod appearance;
mod calendar_list;
mod checks;
mod connect;
mod editor;
mod grid_view;
mod icons;
mod local_calendars;
mod meet_with;
mod mini_calendar;
mod month_view;
mod network;
mod platform;
#[cfg(test)]
mod preview;
mod remove;
#[cfg(test)]
mod screen;
mod sync;
mod widgets;

use appearance::Desktop;
use calendar_core::accounts::{self, Account, AccountState, Accounts, Check, CheckError};
use calendar_core::agenda::{AgendaViewModel, agenda_view};
use calendar_core::availability;
use calendar_core::background::{self, Signal};
use calendar_core::colors::{self, LocalColors};
use calendar_core::create::{self, QuickEntry, Saving};
use calendar_core::details::event_details;
use calendar_core::drag::{self, Target};
use calendar_core::editor::{Edit, Editor, Save};
use calendar_core::guests::{self, Person, Reach, Response};
use calendar_core::invitations;
use calendar_core::local;
use calendar_core::meet_with::{self as meeting, MeetWith, Overlay};
use calendar_core::model::{Calendar, Event, Timing};
use calendar_core::month::{MonthViewModel, month_view};
use calendar_core::navigation::{Navigation, View, mini_calendar};
use calendar_core::notice;
use calendar_core::notify::Pending;
use calendar_core::offline;
use calendar_core::platform::{Autostart, Browser, Instance, LinkHandler, Tray};
use calendar_core::regional::Regional;
use calendar_core::reminders::{Action, ReminderId, SNOOZES, Silenced};
use calendar_core::series::{Question, Scope};
use calendar_core::settings::{self, Settings};
use calendar_core::shortcuts::{self, Command};
use calendar_core::side_panel::{self, Panel};
use calendar_core::status::{self, State};
use calendar_core::sync::Window;
use calendar_core::texts::{self, NO_CALENDAR, WITHOUT_MEET};
use calendar_core::theme::Theme;
use calendar_core::undo::{LastChange, Undo};
use calendar_core::view_model::GridViewModel;
use calendar_core::webcal;
use checks::Checks;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use connect::Connect;
use eframe::egui;
use icons::Icon;
use network::Online;
use platform::{
    SystemAppearance, SystemBrowser, SystemInstance, SystemNetwork, SystemTray, XdgAutostart,
    XdgLauncher, XdgLinkHandler,
};
use remove::Removal;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Instant;
use sync::{LocalDeletion, Opened, Saved, Synchronization};

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
    /// The event whose details are open, what had the keyboard when it opened, whether its
    /// guest list is expanded (`DEC-63`), and the reminder of the notification it was opened
    /// from, to snooze there when the desktop's notifications have no buttons (`DEC-64`).
    Event(Box<Event>, Option<egui::Id>, bool, Option<ReminderId>),
    /// The settings, where accounts are managed (`MOD-01-U01`).
    Settings,
    /// Which events of a series a change or deletion of one of its occurrences applies to
    /// (`MOD-05-F20`, `MOD-05-F21`), and the editor the change was made in, which Cancel goes
    /// back to.
    Scope(Question, Option<Box<Editor>>),
    /// Whether Google e-mails the guests of the event changed or deleted (`MOD-06-F12`), and the
    /// editor the change was made in, which Back goes back to.
    Notify(Pending, Option<Box<Editor>>),
    /// The invitations awaiting the user's answer, opened from the side panel's counter
    /// (`MOD-06-F10`).
    Invitations,
}

/// Ctrl+Z: undoes the last change to an event (`MOD-05-F28`); not Ctrl+Shift+Z, there is no redo.
fn undo_pressed(input: &mut egui::InputState) -> bool {
    !input.modifiers.shift && input.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)
}

/// Room kept above and below a dialog, besides its title bar.
const DIALOG_MARGIN: f32 = 32.0;
/// About how tall a dialog's title bar is, with the margins under it.
const DIALOG_TITLE: f32 = 96.0;
/// Widest a dialog grows.
const DIALOG_WIDTH: f32 = 560.0;
/// How long each notice along the bottom has been shown, to go on its own.
#[derive(Default)]
struct Shown {
    connected: notice::Shown,
    not_saved: notice::Shown,
    gone: notice::Shown,
}

/// How wide the box of a new event is: room for a title, as Google Calendar's.
const QUICK_ENTRY_WIDTH: f32 = 420.0;

/// How wide the shortcuts' list is.
const SHORTCUTS_WIDTH: f32 = 360.0;

/// How wide the keys' column of the shortcuts' list is.
const SHORTCUT_KEYS: f32 = 72.0;

/// How wide "Go to date" is.
const GO_TO_WIDTH: f32 = 320.0;

/// How wide the settings are: more room than the other dialogs, for its sections and list.
const SETTINGS_WIDTH: f32 = 720.0;

/// Days the agenda shows when opened, and how many more each step of scrolling loads.
const AGENDA_DAYS: i64 = 30;

/// The view's model, of the events of calendars neither hidden in the side panel
/// (`MOD-03-F12`) nor disabled in the settings (`MOD-03-F13`), each in its calendar's local
/// colour where it has one, and in the day, 3-day and week views the calendars of the people
/// chosen in "Meet with…" over them (`MOD-06-U05`); with the places in `events` of the events the
/// model's indices point to, the people's events counted after them.
fn build(
    events: &[Event],
    settings: &Settings,
    navigation: Navigation,
    agenda_days: i64,
    zone: Tz,
    now: DateTime<Utc>,
    overlay: &Overlay,
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
        View::Grid(grid) => Built::Grid(meeting::grid(
            events,
            overlay,
            grid,
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

/// Accept, Maybe and Decline (`MOD-06-U03`), the user's own answer marked; the answer clicked.
fn answer_buttons(ui: &mut egui::Ui, own: Response, enabled: bool) -> Option<Response> {
    let mut clicked = None;
    // In the order given, also inside a right-to-left row.
    let mut answers = invitations::ANSWERS.to_vec();
    if ui.layout().prefer_right_to_left() {
        answers.reverse();
    }
    for (response, label) in answers {
        // The answer given stands out in the accent; the others are outlined.
        let line = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let button = if own == response {
            widgets::primary(ui, label)
        } else {
            egui::Button::new(label)
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::new(1.0, line))
        };
        if ui.add_enabled(enabled, button).clicked() {
            clicked = Some(response);
        }
    }
    clicked
}

/// Keeps a calendar's local colour picked in its menu, or forgets it on "Reset"; only in the
/// app's settings, never sent to Google (`MOD-02-F04`). A local calendar's colour is its own,
/// kept in the local calendars' file (`DEC-182`): the calendar is given back recoloured, to keep
/// there.
fn pick(
    colors: &mut LocalColors,
    picked: calendar_list::Picked,
    calendars: &[Calendar],
) -> Option<Calendar> {
    let key = (picked.account_id, picked.calendar_id);
    if key.0 == local::ACCOUNT_ID {
        colors.remove(&key);
        let calendar = (calendars.iter()).find(|c| c.account_id == key.0 && c.id == key.1)?;
        return (picked.color).map(|color| Calendar {
            color,
            ..calendar.clone()
        });
    }
    match picked.color {
        Some(color) => colors.insert(key, color),
        None => colors.remove(&key),
    };
    None
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
    /// The settings' section of local calendars.
    local_calendars: local_calendars::State,
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
    /// When the notices along the bottom were shown, to go on their own.
    shown: Shown,
    /// Whether the computer is online, as the system says (`MOD-02-F14`).
    online: Online,
    /// The desktop's light or dark preference (`MOD-04-F02`).
    desktop: Desktop,
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
    /// The side panel's "Meet with…", never kept across restarts (`MOD-06-F27`).
    meet_with: MeetWith,
    /// The calendars of the people chosen in it, as the built view shows them.
    overlay: Overlay,
    /// The places in `events` of the invitations awaiting the user's answer (`MOD-06-F10`).
    invitations: Vec<usize>,
    built_at: DateTime<Utc>,
    view_state: grid_view::State,
    month_state: month_view::State,
    dialog: Option<Dialog>,
    /// The event of the notification last clicked, to open once the window is shown
    /// (`MOD-07-F08`).
    notified: Option<Opened>,
    /// What the user did to the tray icon or the launcher, and wake-ups of the app while its
    /// window is closed (`None`).
    signals: Receiver<Option<Signal>>,
    /// Whether the desktop shows the app's tray icon (`MOD-07-F14`).
    tray: bool,
    /// Whether "Quit" was chosen in the tray menu (`MOD-07-F15`).
    quitting: bool,
    /// Whether the window closes to open again in front (`MOD-07-F17`).
    reopening: bool,
    /// The window's size and whether it is maximized, as last shown.
    placement: Placement,
    /// Whether the app starts at login (`MOD-07-F16`), as last set, and what turns it on or off
    /// off the UI thread, in the order set.
    start_at_login: bool,
    autostart_writer: Sender<bool>,
}

impl CalendarApp {
    fn new(ctx: egui::Context, signals: Receiver<Option<Signal>>, tray: bool) -> Self {
        appearance::install(&ctx);
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
            Silenced::of(&settings),
            regional,
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
        let overlay = Overlay::default();
        let (built, places) = build(
            &events,
            &settings,
            navigation,
            AGENDA_DAYS,
            zone,
            built_at,
            &overlay,
        );
        let autostart = XdgAutostart::new();
        let start_at_login = autostart.is_on();
        let (autostart_writer, turned) = mpsc::channel::<bool>();
        std::thread::spawn(move || {
            for on in turned {
                // Like the settings, a lost change is acceptable.
                let _ = autostart.set(on);
            }
        });
        CalendarApp {
            accounts,
            accounts_writer,
            checks: Checks::new(),
            problem: None,
            connect: Connect::new(),
            connected: None,
            removal: Removal::new(),
            local_calendars: local_calendars::State::default(),
            sync,
            events,
            cached: Vec::new(),
            saving: Saving::default(),
            last: LastChange::default(),
            not_saved: Vec::new(),
            shown: Shown::default(),
            online: Online::watch(&ctx, &SystemNetwork),
            desktop: Desktop::watch(&ctx, &SystemAppearance),
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
            meet_with: MeetWith::default(),
            overlay,
            invitations: Vec::new(),
            built_at,
            view_state: grid_view::State::default(),
            month_state: month_view::State::default(),
            dialog: None,
            notified: None,
            signals,
            tray,
            quitting: false,
            reopening: false,
            placement: Placement::default(),
            start_at_login,
            autostart_writer,
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
            &self.overlay,
        );
        self.invitations = invitations::awaiting(
            &self.events,
            self.accounts.list(),
            &self.settings,
            self.zone,
            self.built_at,
        );
    }

    /// Opens the details of the event at `event` in `events`, giving the keyboard back to
    /// `focused` when they close.
    fn open(&mut self, event: usize, focused: Option<egui::Id>) {
        // The event is brought up to date first, as the editor opens from its details.
        self.sync.opening(&self.events[event].calendar_id);
        let opened = Box::new(self.events[event].clone());
        self.dialog = Some(Dialog::Event(opened, focused, false, None));
    }

    /// Opens the event of a notification clicked on its day, bringing the window forward
    /// (`MOD-07-F08`), with Snooze when the notification had no buttons (`DEC-64`). Over an event
    /// being edited or saved, the window only comes forward, so nothing typed is lost.
    fn open_notified(&mut self, ctx: &egui::Context, opened: &Opened) {
        let notified = &opened.event;
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        let busy = matches!(
            self.dialog,
            Some(
                Dialog::QuickEntry(..) | Dialog::Editor(_) | Dialog::Scope(..) | Dialog::Notify(..)
            )
        );
        let found = self.events.iter().position(|event| {
            event.account_id == notified.account_id
                && event.calendar_id == notified.calendar_id
                && event.id == notified.id
        });
        let (false, Some(event)) = (busy, found) else {
            return;
        };
        // The agenda starts now whatever the date; the other views move to the event's day.
        self.navigation.selected = match self.events[event].timing {
            Timing::Timed { start, .. } => start.with_timezone(&self.zone).date_naive(),
            Timing::AllDay { start, .. } => start,
        };
        self.open(event, None);
        if let Some(Dialog::Event(.., snooze)) = &mut self.dialog {
            *snooze = opened.snooze.clone();
        }
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
        // The event stays where it was until the user answers what it asks.
        match Question::changing(drag::edit(event, calendar.clone(), timing)) {
            Ok(edit) => {
                // Saved at once, it leaves what is open as it is.
                if let Some(asked) = self.save(Pending::Change(Box::new(edit), Scope::This), None) {
                    self.dialog = Some(asked);
                }
            }
            Err(question) => self.dialog = Some(Dialog::Scope(question, None)),
        }
    }

    /// Saves the change or deletion at once, or first asks whether to e-mail the event's guests
    /// (`MOD-06-F12`): the question to open, over `editor` when it was made there.
    fn save(&mut self, pending: Pending, editor: Option<Box<Editor>>) -> Option<Dialog> {
        if pending.asks().is_some() {
            return Some(Dialog::Notify(pending, editor));
        }
        self.saved(pending, false);
        None
    }

    /// Saves the change to the event: shown changed at once, marked "saving" until Google
    /// confirms it (`MOD-05-U03`).
    fn change(&mut self, edit: Box<Edit>) {
        let local = self.saving.change(&edit.event, edit.changed());
        self.last.changing(local, &edit);
        self.sync.change(local, edit);
        self.show_events(None);
    }

    /// Saves the change to or deletion of the event, or of the events of its series the user
    /// chose (`MOD-05-F20`, `MOD-05-F21`), Google e-mailing the guests as `notify` says
    /// (`MOD-06-F12`). One of the following or all events cannot be undone: as after creating an
    /// event, nothing is left to undo.
    fn saved(&mut self, pending: Pending, notify: bool) {
        match pending {
            Pending::Change(mut edit, scope) => {
                edit.notify = notify;
                if scope == Scope::This {
                    return self.change(edit);
                }
                let local = self.saving.change_series(&edit.event, edit.changed());
                self.last.creating();
                self.sync.change_series(local, edit, scope);
                self.show_events(None);
            }
            Pending::Delete(deleted, Scope::This) => {
                let (calendar, event) = *deleted;
                self.delete(&event, calendar, notify);
            }
            Pending::Delete(deleted, scope) => {
                let (calendar, event) = *deleted;
                let local = self.saving.delete(&event);
                self.last.creating();
                self.sync
                    .delete_series(local, calendar, event, scope, notify);
                self.show_events(None);
            }
            Pending::Respond(edit, scope) => self.respond(edit, scope),
        }
    }

    /// Sends the user's answer to the invitation to Google, to the occurrence alone or to all
    /// events of its series as the user chose (`MOD-06-F11`, `MOD-06-F13`): shown at once, marked
    /// "saving" until Google confirms it. An answer is no change to undo.
    fn respond(&mut self, edit: Box<Edit>, scope: Scope) {
        if scope == Scope::This {
            let local = self.saving.change(&edit.event, edit.changed());
            self.sync.change(local, edit);
        } else {
            let local = self.saving.change_series(&edit.event, edit.changed());
            self.sync.change_series(local, edit, scope);
        }
        self.show_events(None);
    }

    /// Answers the invitation at once, or first asks whether the answer applies to this event or
    /// to all events of its series (`MOD-06-F13`).
    fn answer(&mut self, event: &Event, response: Response) {
        let Some(calendar) = self.sync.calendars().iter().find(|calendar| {
            calendar.account_id == event.account_id && calendar.id == event.calendar_id
        }) else {
            return;
        };
        match invitations::answer(event, calendar.clone(), response) {
            Ok(edit) => self.respond(Box::new(edit), Scope::This),
            Err(question) => self.dialog = Some(Dialog::Scope(question, None)),
        }
    }

    /// Join: opens the event's Google Meet call in the browser (`MOD-06-F18`).
    fn join(&mut self, link: &str) {
        if let Err(error) = SystemBrowser.open(link) {
            (self.not_saved).push(texts::meeting_not_opened(&error.to_string()));
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
            (self.not_saved).push(texts::google_calendar_not_opened(&error.to_string()));
        }
    }

    /// Deletes the event from its calendar, without a question (`MOD-05-F12`, `MOD-05-F27`): gone
    /// at once, and back if Google refuses.
    fn delete(&mut self, event: &Event, calendar: Calendar, notify: bool) {
        let local = self.saving.delete(event);
        self.last.deleting(local, &calendar, event, notify);
        self.sync.delete(local, calendar, event.clone(), notify);
        self.show_events(None);
    }

    /// Ctrl+Z or the notice's Undo: the last change is undone at Google, shown at once, marked
    /// "saving" until Google confirms it (`MOD-05-F28`).
    fn undo(&mut self) {
        let local = match self.last.undo() {
            Some(Undo::Restore(deleted, notify)) => {
                let (calendar, event) = *deleted;
                let local = self.saving.change(&event, event.clone());
                self.sync.restore(local, calendar, event, notify);
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
        let entry = self.new_entry(times);
        self.dialog = Some(Dialog::QuickEntry(entry, false));
    }

    /// A new event in the default calendar, at the times `times` gives for the default length of
    /// its account.
    fn new_entry(&self, times: impl FnOnce(u32) -> (DateTime<Utc>, DateTime<Utc>)) -> QuickEntry {
        let calendar =
            create::default_calendar(self.accounts.list(), self.sync.calendars(), &self.settings);
        let length = calendar
            .as_ref()
            .map_or(create::DEFAULT_LENGTH, |calendar| {
                self.sync.default_length(&calendar.account_id)
            });
        QuickEntry::new(times(length), self.zone, calendar)
    }

    /// The account "Meet with…" searches for people and reads their calendars through: the
    /// default calendar's, or else the first connected (`DEC-175`).
    fn meeting_account(&self) -> Option<&accounts::Entry> {
        let calendar =
            create::default_calendar(self.accounts.list(), self.sync.calendars(), &self.settings);
        let list = self.accounts.list();
        (calendar.and_then(|calendar| list.iter().find(|e| e.account.id == calendar.account_id)))
            .or(list.first())
    }

    /// The calendars of the people chosen in "Meet with…" as the grid views are to show them:
    /// those Google gave for the days shown, nothing offline (`MOD-06-F27`) nor in the month and
    /// agenda views.
    fn overlay(&self, online: bool) -> Overlay {
        let Some(asked) = self.meeting_asked(online) else {
            return Overlay::default();
        };
        let answer = self.sync.their(asked.span).unwrap_or_default();
        let user = self.meeting_account().map(|e| e.account.email.as_str());
        Overlay::of(
            user.unwrap_or_default(),
            &self.meet_with.chosen,
            answer,
            self.zone,
        )
    }

    /// The calendars of the people chosen in "Meet with…" over the days the grid view shows.
    fn meeting_asked(&self, online: bool) -> Option<meeting::Asked> {
        let navigation = self.navigation;
        let dates = match navigation.view {
            View::Grid(grid) => Some(grid.days(navigation.selected, navigation.week_start)),
            _ => None,
        };
        self.meet_with.asked(dates.as_deref(), self.zone, online)
    }

    /// The event at `index` in the built view: the user's, or a chosen person's after them.
    fn in_view(&self, index: usize) -> Option<&Event> {
        match self.places.get(index) {
            Some(&place) => self.events.get(place),
            None => self.overlay.events().nth(index - self.places.len()),
        }
    }

    /// Opens the event at `index` in the built view; a chosen person's read-only, as it is not
    /// on the user's calendars (`MOD-06-F25`).
    fn open_in_view(&mut self, index: usize, focused: Option<egui::Id>) {
        match self.places.get(index) {
            Some(&place) => self.open(place, focused),
            None => {
                if let Some(theirs) = self.in_view(index) {
                    let opened = Box::new(theirs.clone());
                    self.dialog = Some(Dialog::Event(opened, focused, false, None));
                }
            }
        }
    }

    fn today(&self) -> NaiveDate {
        Utc::now().with_timezone(&self.zone).date_naive()
    }

    /// The days the cache holds, around today (`MOD-02-F11`).
    /// The notices in the bottom corner, over the view so nothing moves (`MOD-04-U06`): an account
    /// connected, what could not be saved or changed, calendars gone, each closed by its cross or
    /// going on its own after a while; and the Undo that follows a change, for 5 seconds
    /// (`MOD-05-U04`).
    fn notices(&mut self, ctx: &egui::Context, online: bool) {
        let now = Instant::now();
        let shown = &mut self.shown;
        let lefts = [
            shown
                .connected
                .left(usize::from(self.connected.is_some()), now),
            shown.not_saved.left(self.not_saved.len(), now),
            shown.gone.left(self.gone.len(), now),
        ];
        let [connected, not_saved, gone] = lefts;
        if connected.is_none() {
            self.connected = None;
        }
        if not_saved.is_none() {
            self.not_saved.clear();
        }
        if gone.is_none() {
            self.gone.clear();
        }
        let undo = self.last.notice(now);
        let undoable = self.undoable(online);
        let lefts = lefts.into_iter().chain([undo.map(|(_, left)| left)]);
        let Some(soonest) = lefts.flatten().min() else {
            return;
        };
        ctx.request_repaint_after(soonest);
        let (mut close_connected, mut close_not_saved, mut close_gone) = (false, false, false);
        let mut undone = false;
        egui::Area::new(egui::Id::unique("notices"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(16.0, -16.0))
            .show(ctx, |ui| {
                if let Some(account) = &self.connected {
                    close_connected = widgets::notice(ui, |ui| {
                        ui.label(texts::connected(&account.email));
                        let unavailable = account.unavailable();
                        if !unavailable.is_empty() {
                            ui.label(texts::not_available(&unavailable));
                        }
                    });
                }
                // A new event Google did not create is gone from the views, and one it did not
                // change or delete is shown as it was; the user is told why.
                if !self.not_saved.is_empty() {
                    close_not_saved = widgets::notice(ui, |ui| {
                        for message in &self.not_saved {
                            let warn = ui.visuals().warn_fg_color;
                            ui.colored_label(warn, message);
                        }
                    });
                }
                // Calendars the user no longer has are told of once (`MOD-02-F21`).
                if !self.gone.is_empty() {
                    close_gone = widgets::notice(ui, |ui| {
                        for name in &self.gone {
                            ui.label(texts::calendar_gone(name));
                        }
                    });
                }
                if let Some((says, _)) = undo {
                    widgets::toast(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(says);
                            undone = ui
                                .add_enabled(undoable, egui::Button::new(texts::UNDO))
                                .clicked();
                        });
                    });
                }
            });
        if close_connected {
            self.connected = None;
        }
        if close_not_saved {
            self.not_saved.clear();
        }
        if close_gone {
            self.gone.clear();
        }
        if undone {
            self.undo();
        }
    }

    /// Whether the last change can be undone now: offline, only a local event's
    /// (`MOD-02-F15`, MOD-09 *Constraints and edge cases*).
    fn undoable(&self, online: bool) -> bool {
        (self.last.undoes()).is_none_or(|event| offline::refused_for(online, event).is_none())
    }

    /// Opens "Go to date" on the selected date, from its key or its button (`MOD-03-F23`).
    fn go_to_date(&mut self) {
        let date = self.regional.numeric_date(self.navigation.selected);
        self.dialog = Some(Dialog::GoToDate(date, false));
    }

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
        // Offline the undo waits, as changes need Google (`MOD-02-F15`), but a local event's.
        if self.undoable(self.online.get()) && ctx.input_mut(undo_pressed) {
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
                Command::GoToDate => self.go_to_date(),
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
    /// the event, and the scope and notify questions go back to the editor.
    fn close_dialog(&mut self, ctx: &egui::Context) {
        match self.dialog.take() {
            Some(Dialog::Event(_, Some(id), ..)) => ctx.memory_mut(|m| m.request_focus(id)),
            // The custom recurrence dialog closes alone, back to the editor (`MOD-05-F33`).
            Some(Dialog::Editor(mut editor)) if editor.custom.is_some() => {
                editor.custom = None;
                self.dialog = Some(Dialog::Editor(editor));
            }
            // The scope and notify questions go back to the editor the change was made in.
            Some(Dialog::Scope(_, Some(editor)) | Dialog::Notify(_, Some(editor))) => {
                self.dialog = Some(Dialog::Editor(editor))
            }
            _ => {}
        }
    }

    fn dialog(&mut self, ctx: &egui::Context) {
        // Connecting and removing an account never run at once.
        let can_change = self.connect.idle() && self.removal.idle();
        let can_remove = self.connect.idle();
        // Changes need Google (`MOD-02-F15`).
        let online = self.online.get();
        let refused = offline::refused(online);
        let Some(dialog) = &mut self.dialog else {
            self.sync.ask_busy(None);
            return;
        };
        let mut open = true;
        let mut closed = false;
        let mut go_to = None;
        let mut again = None;
        let mut remove = None;
        let mut toggled = None;
        let mut local_actions = Vec::new();
        let mut save = None;
        let mut edit = None;
        let mut delete = None;
        let mut default_calendar = None;
        let mut notifications = None;
        let mut start_at_login = None;
        let mut theme = None;
        let mut hour_zoom = None;
        let mut scope = None;
        let mut notify = None;
        let mut cancelled = false;
        let mut open_series = None;
        let mut attachment = None;
        let mut join = None;
        let mut snoozed = None;
        let mut search = None;
        let mut busy = None;
        let mut invitation = None;
        let mut answered = None;
        let title = match dialog {
            Dialog::Shortcuts => texts::KEYBOARD_SHORTCUTS,
            Dialog::GoToDate(..) => texts::GO_TO_DATE,
            Dialog::QuickEntry(..) => texts::NEW_EVENT,
            Dialog::Editor(editor) if editor.opened.is_none() => texts::NEW_EVENT,
            Dialog::Editor(_) => texts::EDIT_EVENT,
            Dialog::Event(..) => texts::EVENT,
            Dialog::Settings => texts::SETTINGS,
            Dialog::Scope(Question::Change(_), _) => texts::EDIT_RECURRING_EVENT,
            Dialog::Scope(Question::Delete(_), _) => texts::DELETE_RECURRING_EVENT,
            Dialog::Scope(Question::Respond(_), _) => texts::RESPOND_TO_RECURRING_EVENT,
            Dialog::Notify(..) => texts::NOTIFY_GUESTS,
            Dialog::Invitations => texts::INVITATIONS,
        };
        // A dialog taller than the window scrolls, so all of it can be reached; its title bar
        // takes some of the room.
        let room = ctx.content_rect().height() - 2.0 * DIALOG_MARGIN - DIALOG_TITLE;
        let (min_width, max_width) = match dialog {
            Dialog::Settings => (SETTINGS_WIDTH, SETTINGS_WIDTH),
            Dialog::QuickEntry(..) => (QUICK_ENTRY_WIDTH, QUICK_ENTRY_WIDTH),
            Dialog::GoToDate(..) => (GO_TO_WIDTH, GO_TO_WIDTH),
            Dialog::Shortcuts => (SHORTCUTS_WIDTH, SHORTCUTS_WIDTH),
            _ => (0.0, DIALOG_WIDTH),
        };
        // An event's details carry their own title and buttons, as in Google Calendar.
        let titled = !matches!(dialog, Dialog::Event(..));
        egui::Window::new(title)
            .open(&mut open)
            .title_bar(titled)
            .collapsible(false)
            // Sized by what it holds; the scroll inside keeps it within the window.
            .auto_sized()
            .max_width(max_width)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                // As tall as what it holds, up to the room there is.
                egui::ScrollArea::vertical()
                    .max_height(room)
                    .show(ui, |ui| {
                        ui.set_min_width(min_width);
                        match dialog {
                            Dialog::Shortcuts => {
                                // In groups, each under its title, keys doing the same thing on
                                // one row, each key drawn as a key; the rows close together.
                                ui.spacing_mut().interact_size.y = 24.0;
                                ui.spacing_mut().item_spacing.y = 4.0;
                                for (n, (title, rows)) in
                                    shortcuts::listed().into_iter().enumerate()
                                {
                                    if n > 0 {
                                        widgets::section(ui);
                                    }
                                    widgets::heading(ui, title);
                                    for (keys, description) in rows {
                                        ui.horizontal(|ui| {
                                            let size = egui::vec2(
                                                SHORTCUT_KEYS,
                                                ui.spacing().interact_size.y,
                                            );
                                            let layout =
                                                egui::Layout::left_to_right(egui::Align::Center);
                                            ui.allocate_ui_with_layout(size, layout, |ui| {
                                                ui.set_min_size(size);
                                                ui.spacing_mut().item_spacing.x = 4.0;
                                                for key in keys {
                                                    widgets::keycap(ui, key);
                                                }
                                            });
                                            ui.label(description);
                                        });
                                    }
                                }
                            }
                            Dialog::GoToDate(text, focused) => {
                                // As the editor's fields: the format above the field, outlined,
                                // and Go at the end.
                                widgets::outlined_controls(ui);
                                let format = texts::date_as(
                                    &self.regional.numeric_date(self.navigation.selected),
                                );
                                ui.label(egui::RichText::new(format).small().weak());
                                let field = widgets::text_field(
                                    ui,
                                    egui::TextEdit::singleline(text).desired_width(f32::INFINITY),
                                );
                                let date = self.regional.parse_date(text);
                                let entered = field.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter));
                                // The field takes the keyboard when the dialog opens and after a wrong date.
                                if !*focused || (entered && date.is_none()) {
                                    field.request_focus();
                                    *focused = true;
                                }
                                ui.add_space(6.0);
                                let go = egui::Sides::new()
                                    .show(
                                        ui,
                                        |_| {},
                                        |ui| {
                                            let button = widgets::primary(ui, texts::GO);
                                            ui.add_enabled(date.is_some(), button).clicked()
                                        },
                                    )
                                    .1;
                                if go || entered {
                                    go_to = date;
                                }
                            }
                            Dialog::QuickEntry(entry, focused) => {
                                // As the editor: its title large, its drop-down and buttons
                                // outlined; the time and the calendar on rows with their icons, as
                                // in an event's details.
                                widgets::outlined_controls(ui);
                                let field = widgets::text_field(
                                    ui,
                                    egui::TextEdit::singleline(&mut entry.title)
                                        .hint_text(
                                            egui::RichText::new(texts::ADD_TITLE)
                                                .font(appearance::strong(18.0)),
                                        )
                                        .font(appearance::strong(18.0))
                                        .desired_width(f32::INFINITY),
                                );
                                // The title takes the keyboard when the box opens.
                                if !*focused {
                                    field.request_focus();
                                    *focused = true;
                                }
                                ui.add_space(6.0);
                                widgets::detail_row(ui, "🕒", |ui| {
                                    ui.label(entry.when().describe(&self.regional));
                                });
                                let choices = create::choices(
                                    self.accounts.list(),
                                    self.sync.calendars(),
                                    &self.settings,
                                );
                                widgets::detail_row(ui, "📅", |ui| {
                                    calendar_list::picker(
                                        ui,
                                        "new-event-calendar",
                                        &choices,
                                        &mut entry.calendar,
                                    );
                                });
                                // No network, no calendar: creating is unavailable, and the box says why;
                                // a local calendar needs no network (MOD-09).
                                let refused = offline::refused_in(online, entry.calendar.as_ref());
                                let unavailable =
                                    refused.or(entry.calendar.is_none().then_some(NO_CALENDAR));
                                if let Some(reason) = unavailable {
                                    ui.colored_label(ui.visuals().warn_fg_color, reason);
                                }
                                let entered = field.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter));
                                ui.add_space(6.0);
                                // Save at the end, as in the editor and Google Calendar.
                                let (_, clicked) = egui::Sides::new().show(
                                    ui,
                                    |ui| {
                                        // The full editor, keeping what was typed (`MOD-05-F03`).
                                        if ui.button(texts::MORE_OPTIONS).clicked() {
                                            edit = Some(Editor::from_quick(entry, &self.regional));
                                        }
                                    },
                                    |ui| {
                                        let button = widgets::primary(ui, texts::SAVE);
                                        ui.add_enabled(unavailable.is_none(), button).clicked()
                                    },
                                );
                                if (clicked || entered) && unavailable.is_none() {
                                    save = entry.new_event().map(|(calendar, event)| {
                                        Save::Create(calendar, Box::new(event))
                                    });
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
                                let account =
                                    editor.calendar.as_ref().map(|c| c.account_id.clone());
                                // The people Google found, then the account's recent guests (`MOD-06-F03`).
                                let found: Vec<Person> = match &account {
                                    Some(id) if !typed.is_empty() => {
                                        (self.sync.people(id).iter().cloned())
                                            .chain(guests::recent(&self.cached, id))
                                            .collect()
                                    }
                                    _ => Vec::new(),
                                };
                                let own = editor.own_addresses(accounts);
                                let suggested =
                                    guests::suggestions(&typed, &found, &editor.guests, &own);
                                let regional = &self.regional;
                                // The guests' day, once Google answered.
                                let saved = editor.opened.as_ref().map(|(event, _)| event);
                                let preview = (availability::asked(editor, regional, online))
                                    .and_then(|(asked, chosen)| {
                                        let answer = self.sync.busy(&asked)?;
                                        Some(availability::preview(
                                            &asked,
                                            chosen,
                                            &editor.guests,
                                            saved,
                                            answer,
                                        ))
                                    });
                                let google = editor::FromGoogle {
                                    suggested: &suggested,
                                    availability: preview.as_ref(),
                                };
                                let refused = offline::refused_in(online, editor.calendar.as_ref());
                                match editor::show(
                                    ui, editor, regional, &choices, accounts, refused, google,
                                ) {
                                    Some(editor::Edited::Save(saved)) => save = Some(saved),
                                    Some(editor::Edited::OpenSeries) => {
                                        open_series =
                                            editor.opened.as_ref().map(|(event, _)| event.clone());
                                    }
                                    None => {}
                                }
                                // What is typed is searched for at Google as it changes, and in the account of
                                // another calendar picked (`MOD-06-F02`).
                                let now_typed = editor.guest_field.trim();
                                let now_account =
                                    editor.calendar.as_ref().map(|c| c.account_id.clone());
                                if let Some(id) = now_account
                                    && (now_typed != typed || account.as_ref() != Some(&id))
                                    && !now_typed.is_empty()
                                {
                                    search = Some((id, now_typed.to_string()));
                                }
                                // Asked again as the day, the guests or the calendar change (`MOD-06-F19`).
                                busy = availability::asked(editor, regional, online)
                                    .map(|(asked, _)| asked);
                            }
                            Dialog::Event(event, _, expanded, snooze) => {
                                // Its buttons outlined, as the editor's are.
                                widgets::outlined_controls(ui);
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
                                let details = event_details(event, self.zone);
                                let not_yet = self.saving.holds(event).then_some(texts::SAVING);
                                // A local event is changed offline too (MOD-09 *Constraints and edge
                                // cases*).
                                let refused = offline::refused_for(online, event);
                                // Editing is not offered for an event the user may not change (`MOD-05-F17`),
                                // though a guest still answers it (`MOD-06-F15`).
                                let protected = event.protected();
                                let calendar = self.sync.calendars().iter().find(|calendar| {
                                    calendar.account_id == event.account_id
                                        && calendar.id == event.calendar_id
                                });
                                let changeable = calendar.filter(|_| protected.is_none());
                                let can_edit = refused.is_none() && not_yet.is_none();
                                // Its calendar's colour and its title, with what can be done with it
                                // beside them, as in Google Calendar.
                                egui::Sides::new().shrink_left().wrap().show(
                                    ui,
                                    |ui| {
                                        widgets::color_mark(
                                            ui,
                                            crate::grid_view::color(event.shown_color()),
                                        );
                                        let title = details.title.as_deref().unwrap_or(texts::BUSY);
                                        ui.label(
                                            egui::RichText::new(title)
                                                .font(appearance::strong(20.0))
                                                .color(ui.visuals().strong_text_color()),
                                        );
                                    },
                                    |ui| {
                                        if widgets::close_button(ui, texts::CLOSE).clicked() {
                                            closed = true;
                                        }
                                        if let Some(calendar) = changeable {
                                            // Also Delete or Backspace, as in Google Calendar.
                                            let keyed = ui.input(|i| {
                                                i.key_pressed(egui::Key::Delete)
                                                    || i.key_pressed(egui::Key::Backspace)
                                            });
                                            let clicked = widgets::icon_button_if(
                                                ui,
                                                "🗑",
                                                texts::DELETE,
                                                can_edit,
                                            )
                                            .clicked();
                                            if can_edit && (clicked || keyed) {
                                                delete = Some((event.clone(), calendar.clone()));
                                            }
                                            if widgets::icon_button_if(
                                                ui,
                                                "✏",
                                                texts::EDIT,
                                                can_edit,
                                            )
                                            .clicked()
                                            {
                                                edit = Some(Editor::of(
                                                    event,
                                                    calendar.clone(),
                                                    &self.regional,
                                                ));
                                            }
                                        }
                                    },
                                );
                                ui.add_space(6.0);
                                // An invitation opens with its answers at the top (`MOD-06-U03`).
                                let own_answer = invitations::own_answer(event);
                                if let Some(own) = own_answer {
                                    let enabled = refused.is_none() && not_yet.is_none();
                                    widgets::card(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(texts::GOING).strong());
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if let Some(response) =
                                                        answer_buttons(ui, own, enabled)
                                                    {
                                                        answered = Some((event.clone(), response));
                                                    }
                                                },
                                            );
                                        });
                                    });
                                    ui.add_space(6.0);
                                }
                                widgets::detail_row(ui, "🕒", |ui| {
                                    ui.label(details.when.describe(&self.regional));
                                    if let Some(own) = details.in_own_zone(&self.regional) {
                                        ui.weak(own);
                                    }
                                });
                                // Where it happens and what it says (`MOD-05-F30`).
                                if let Some(location) = &details.location {
                                    widgets::detail_row(ui, "📍", |ui| ui.label(location));
                                }
                                // Join opens its Google Meet call in the browser (`MOD-06-F18`).
                                if let Some(link) = &details.meet {
                                    widgets::detail_row(ui, "📹", |ui| {
                                        let button = widgets::primary(ui, texts::JOIN_WITH_MEET);
                                        if ui.add(button).clicked() {
                                            join = Some(link.clone());
                                        }
                                    });
                                }
                                // The Snooze of a notification without buttons (`DEC-64`).
                                if let Some(reminder) = snooze {
                                    widgets::detail_row(ui, "⏰", |ui| {
                                        ui.horizontal_wrapped(|ui| {
                                            for minutes in SNOOZES {
                                                if ui
                                                    .button(Action::Snooze(minutes).label())
                                                    .clicked()
                                                {
                                                    snoozed = Some((reminder.clone(), minutes));
                                                }
                                            }
                                        });
                                    });
                                }
                                // When and where; what it says; who comes; the rest: each group
                                // set apart by a line.
                                if let Some(description) = &details.description {
                                    widgets::section(ui);
                                    // A link in it opens in the browser, as an attachment does.
                                    widgets::detail_row(ui, "📄", |ui| {
                                        if let Some(link) = widgets::rich_text(ui, description) {
                                            attachment = Some(link);
                                        }
                                    });
                                }
                                // Each guest with their answer as an icon (`MOD-06-F05`, `MOD-06-U01`).
                                if !details.guests.is_empty() {
                                    widgets::section(ui);
                                    widgets::detail_row(ui, "👥", |ui| {
                                        ui.label(
                                            egui::RichText::new(texts::guests(
                                                details.guests.len(),
                                            ))
                                            .strong(),
                                        );
                                        // A guest a line, close together.
                                        ui.spacing_mut().interact_size.y = 20.0;
                                        ui.spacing_mut().item_spacing.y = 4.0;
                                        for guest in details.listed_guests(*expanded) {
                                            let answer = guest.response;
                                            let icon = ui
                                                .horizontal(|ui| {
                                                    let icon = ui
                                                        .weak(answer.icon())
                                                        .on_hover_text(answer.label());
                                                    if !guest.name.is_empty() {
                                                        ui.label(&guest.name);
                                                    }
                                                    ui.weak(&guest.email);
                                                    icon.rect
                                                })
                                                .inner;
                                            // An optional guest says so under their name, as in
                                            // Google Calendar (`MOD-06-F06`).
                                            if guest.optional {
                                                ui.add_space(-4.0);
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().interact_size.y = 0.0;
                                                    let item = ui.spacing().item_spacing.x;
                                                    ui.add_space(icon.width() + item);
                                                    let optional =
                                                        egui::RichText::new(texts::OPTIONAL);
                                                    ui.label(optional.small().weak());
                                                });
                                            }
                                        }
                                        let more = details.more_guests();
                                        if more > 0 {
                                            let toggle = if *expanded {
                                                texts::SHOW_FEWER.to_string()
                                            } else {
                                                texts::show_more(more)
                                            };
                                            if ui.link(toggle).clicked() {
                                                *expanded = !*expanded;
                                            }
                                        }
                                    });
                                }
                                // Each attachment opens in the browser (`MOD-05-F26`).
                                widgets::section(ui);
                                if !details.attachments.is_empty() {
                                    widgets::detail_row(ui, "📎", |ui| {
                                        for listed in &details.attachments {
                                            if ui.link(&listed.title).clicked() {
                                                attachment = Some(listed.url.clone());
                                            }
                                        }
                                    });
                                }
                                if let Some(calendar) = calendar {
                                    widgets::detail_row(ui, "📅", |ui| ui.weak(&calendar.name));
                                }
                                // Why it cannot be changed or answered now, under the rest.
                                let reasons = [
                                    protected.map(|protected| protected.why()),
                                    refused.filter(|_| protected.is_none() || own_answer.is_some()),
                                    // A local event has no "saving" (MOD-09 *Constraints and edge
                                    // cases*).
                                    not_yet.filter(|_| {
                                        changeable.is_some()
                                            && event.account_id != local::ACCOUNT_ID
                                    }),
                                ];
                                if reasons.iter().any(Option::is_some) {
                                    ui.add_space(4.0);
                                }
                                for why in reasons.into_iter().flatten() {
                                    ui.weak(why);
                                }
                            }
                            Dialog::Scope(question, _) => {
                                let refused = offline::refused_for(online, question.event());
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
                                cancelled = ui.button(texts::CANCEL).clicked();
                            }
                            Dialog::Notify(pending, _) => {
                                ui.label(pending.asks().unwrap_or_default());
                                // Changes need Google (`MOD-02-F15`).
                                if let Some(reason) = refused {
                                    ui.label(reason);
                                }
                                ui.horizontal(|ui| {
                                    cancelled = ui.button(texts::BACK).clicked();
                                    for (answer, send) in
                                        [(texts::DONT_SEND, false), (texts::SEND, true)]
                                    {
                                        let button = egui::Button::new(answer);
                                        if ui.add_enabled(refused.is_none(), button).clicked() {
                                            notify = Some(send);
                                        }
                                    }
                                });
                            }
                            Dialog::Invitations => {
                                if self.invitations.is_empty() {
                                    ui.label(texts::NO_INVITATIONS);
                                }
                                // Each is answered here, or opens its details (`MOD-06-F10`).
                                for &i in &self.invitations {
                                    let event = &self.events[i];
                                    let details = event_details(event, self.zone);
                                    let enabled = refused.is_none() && !self.saving.holds(event);
                                    ui.horizontal(|ui| {
                                        if ui
                                            .link(details.title.as_deref().unwrap_or(texts::BUSY))
                                            .clicked()
                                        {
                                            invitation = Some(i);
                                        }
                                        ui.weak(details.when.describe(&self.regional));
                                        if let Some(own) = invitations::own_answer(event)
                                            && let Some(response) = answer_buttons(ui, own, enabled)
                                        {
                                            answered = Some((event.clone(), response));
                                        }
                                    });
                                }
                                if let Some(reason) =
                                    refused.filter(|_| !self.invitations.is_empty())
                                {
                                    ui.label(reason);
                                }
                            }
                            Dialog::Settings => {
                                // Its drop-downs and buttons outlined, as the editor's are.
                                widgets::outlined_controls(ui);
                                widgets::heading(ui, texts::ACCOUNTS);
                                // Each account a row: its address and state, what can be done
                                // with it at the row's end.
                                for entry in self.accounts.list() {
                                    egui::Sides::new().show(
                                        ui,
                                        |ui| {
                                            ui.vertical(|ui| {
                                                ui.spacing_mut().item_spacing.y = 0.0;
                                                ui.label(&entry.account.email);
                                                // An account's problem is shown on the account
                                                // (`MOD-01-F11`).
                                                let state =
                                                    egui::RichText::new(entry.state.label());
                                                let state =
                                                    if entry.state == AccountState::NeedsSignIn {
                                                        state.color(ui.visuals().warn_fg_color)
                                                    } else {
                                                        state.weak()
                                                    };
                                                ui.label(state.small());
                                            });
                                        },
                                        |ui| {
                                            ui.add_enabled_ui(can_change, |ui| {
                                                if widgets::outlined_button(ui, texts::REMOVE)
                                                    .clicked()
                                                {
                                                    remove = Some(entry.account.clone());
                                                }
                                                if entry.state == AccountState::NeedsSignIn
                                                    && ui
                                                        .add(widgets::primary(
                                                            ui,
                                                            texts::SIGN_IN_AGAIN,
                                                        ))
                                                        .clicked()
                                                {
                                                    again = Some(entry.account.email.clone());
                                                }
                                            });
                                        },
                                    );
                                    ui.add_space(4.0);
                                }
                                self.removal.show(ui, can_remove);
                                if self.removal.idle() {
                                    ui.add_space(8.0);
                                    self.connect.show(ui, texts::ADD_ACCOUNT);
                                }
                                widgets::section(ui);
                                // Next to the accounts, the other place calendars come from
                                // (`MOD-09-U01`); unavailable while the file cannot be read.
                                widgets::heading(ui, texts::LOCAL_CALENDARS);
                                match self.sync.local() {
                                    Some(Ok(())) => {
                                        let listed: Vec<Calendar> =
                                            local::of(self.sync.calendars()).cloned().collect();
                                        local_actions = local_calendars::show(
                                            ui,
                                            &mut self.local_calendars,
                                            &listed,
                                        );
                                    }
                                    Some(Err(why)) => {
                                        let warn = ui.visuals().warn_fg_color;
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(why).color(warn),
                                        ));
                                    }
                                    None => {}
                                }
                                widgets::section(ui);
                                // All of them, on or off (`MOD-07-F11`); the reminders in Google stay.
                                widgets::heading(ui, texts::NOTIFICATIONS);
                                let mut on = !self.settings.notifications_off;
                                if ui
                                    .checkbox(&mut on, texts::DESKTOP_NOTIFICATIONS)
                                    .on_hover_text(texts::DESKTOP_NOTIFICATIONS_HINT)
                                    .changed()
                                {
                                    notifications = Some(on);
                                }
                                // Off until the user turns it on (`MOD-07-F16`, `DEC-64`); without a tray
                                // icon there would be nothing to open the app from at login.
                                let mut on = self.start_at_login;
                                if self.tray
                                    && ui
                                        .checkbox(&mut on, texts::START_AT_LOGIN)
                                        .on_hover_text(texts::START_AT_LOGIN_HINT)
                                        .changed()
                                {
                                    start_at_login = Some(on);
                                }
                                widgets::section(ui);
                                // System, Light or Dark (`MOD-04-F03`), where Google Calendar has its
                                // theme; picked as the views are.
                                widgets::heading(ui, texts::APPEARANCE);
                                widgets::form_row(ui, texts::THEME, |ui| {
                                    ui.horizontal(|ui| {
                                        let names = Theme::ALL.map(|choice| choice.label());
                                        let current = Theme::ALL
                                            .iter()
                                            .position(|t| *t == self.settings.theme);
                                        if let Some(i) = widgets::segmented(
                                            ui,
                                            &names,
                                            current.unwrap_or_default(),
                                        ) {
                                            theme = Some(Theme::ALL[i]);
                                        }
                                    });
                                });
                                // Also zoomed in the views (`MOD-03-F28`).
                                widgets::form_row(ui, texts::HOUR_HEIGHT, |ui| {
                                    let mut zoom = self.settings.hour_zoom;
                                    if appearance::hour_height(ui, &mut zoom) {
                                        hour_zoom = Some(zoom);
                                    }
                                });
                                widgets::section(ui);
                                widgets::heading(ui, texts::CALENDARS);
                                // Where new events go (`MOD-05-F10`).
                                widgets::form_row(ui, texts::DEFAULT_CALENDAR, |ui| {
                                    let list = (self.accounts.list(), self.sync.calendars());
                                    let choices = create::choices(list.0, list.1, &self.settings);
                                    let mut picked =
                                        create::default_calendar(list.0, list.1, &self.settings);
                                    if calendar_list::picker(
                                        ui,
                                        "default-calendar",
                                        &choices,
                                        &mut picked,
                                    ) {
                                        default_calendar = picked;
                                    }
                                });
                                ui.add_space(4.0);
                                let calendars = side_panel::in_settings(
                                    self.accounts.list(),
                                    self.sync.calendars(),
                                    &self.settings,
                                );
                                toggled = calendar_list::in_settings(ui, &calendars);
                            }
                        }
                    })
            });
        if closed {
            open = false;
        }
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
        for action in local_actions {
            match action {
                local_calendars::Action::Add(name) => {
                    let calendar = local::new_calendar(&name, self.zone, self.sync.calendars());
                    self.sync.add_local(calendar);
                }
                local_calendars::Action::Change(calendar) => self.change_local(calendar),
                local_calendars::Action::Delete(calendar, confirmed) => {
                    self.sync.delete_local(calendar, confirmed);
                }
            }
        }
        if let Some(on) = notifications {
            self.settings.notifications_off = !on;
            self.save_settings();
            self.sync.set_silenced(Silenced::of(&self.settings));
        }
        if let Some(on) = start_at_login {
            self.start_at_login = on;
            let _ = self.autostart_writer.send(on);
        }
        if let Some(theme) = theme {
            self.settings.theme = theme;
            self.save_settings();
            ctx.request_repaint();
        }
        if let Some(zoom) = hour_zoom {
            self.settings.hour_zoom = zoom;
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
            Some(Save::Change(change)) => {
                let editor = match self.dialog.take() {
                    Some(Dialog::Editor(editor)) => Some(editor),
                    _ => None,
                };
                self.dialog = match Question::changing(*change) {
                    Ok(change) => self.save(Pending::Change(Box::new(change), Scope::This), editor),
                    Err(question) => Some(Dialog::Scope(question, editor)),
                };
            }
            None => {}
        }
        if cancelled {
            open = false;
        }
        if let Some(chosen) = scope
            && let Some(Dialog::Scope(question, editor)) = self.dialog.take()
        {
            self.dialog = self.save(Pending::of(question, chosen), editor);
        }
        if let Some(send) = notify
            && let Some(Dialog::Notify(pending, _)) = self.dialog.take()
        {
            self.saved(pending, send);
        }
        if let Some(event) = open_series {
            self.open_series(&event);
        }
        self.sync.ask_busy(busy);
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
            (self.not_saved).push(texts::attachment_not_opened(&error.to_string()));
        }
        if let Some(link) = join {
            self.join(&link);
        }
        if let Some((reminder, minutes)) = snoozed {
            self.sync.snooze(reminder, minutes);
            open = false;
        }
        if let Some(editor) = edit {
            self.dialog = Some(Dialog::Editor(Box::new(editor)));
        }
        if let Some(event) = invitation {
            self.open(event, None);
        }
        if let Some((event, response)) = answered {
            self.answer(&event, response);
        }
        if let Some((event, calendar)) = delete {
            self.dialog = match Question::deleting(calendar, event) {
                Ok(deleted) => self.save(Pending::Delete(Box::new(deleted), Scope::This), None),
                Err(question) => Some(Dialog::Scope(question, None)),
            };
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

    /// The side panel's "Meet with…" (`US-06-09`): people suggested as guests are, from the
    /// account it reads calendars through (`MOD-06-F23`), chosen and removed; **Create event**
    /// opens the editor of a meeting with them on the selected day (`MOD-06-F26`).
    fn meet_with_panel(&mut self, ui: &mut egui::Ui, online: bool) {
        let entry = self.meeting_account().cloned();
        let account = entry.as_ref().map(|e| e.account.id.clone());
        let own: Vec<String> = (self.accounts.list().iter())
            .map(|e| e.account.email.clone())
            .collect();
        let typed = self.meet_with.field.trim().to_string();
        // The people Google found, then the account's recent guests (`MOD-06-F03`).
        let found: Vec<Person> = match &account {
            Some(id) if !typed.is_empty() => (self.sync.people(id).iter().cloned())
                .chain(guests::recent(&self.cached, id))
                .collect(),
            _ => Vec::new(),
        };
        let suggested = self.meet_with.suggestions(&found, &own);
        let answer = (self.meeting_asked(online))
            .and_then(|asked| self.sync.their(asked.span))
            .unwrap_or_default();
        let seen: Vec<bool> = (self.meet_with.chosen.iter())
            .map(|chosen| !meeting::cannot_be_seen(&chosen.person.email, answer))
            .collect();
        let unavailable = meeting::unavailable(!self.accounts.is_empty(), online);
        let action = meet_with::show(ui, &mut self.meet_with, &suggested, &seen, unavailable);
        let account_id = account.clone().unwrap_or_default();
        match action {
            Some(meet_with::Action::Choose(person)) => {
                self.meet_with.choose(&person, &account_id, &own)
            }
            Some(meet_with::Action::Enter) => self.meet_with.enter(&suggested, &account_id, &own),
            Some(meet_with::Action::Remove(email)) => self.meet_with.remove(&email),
            Some(meet_with::Action::Create) => {
                let (date, zone) = (self.navigation.selected, self.zone);
                let entry = self.new_entry(|length| create::keyed(date, Utc::now(), length, zone));
                let editor = self.meet_with.meeting(&entry, &self.regional);
                self.dialog = Some(Dialog::Editor(Box::new(editor)));
            }
            None => {}
        }
        // What is typed is searched for at Google as it changes (`MOD-06-F23`).
        let now_typed = self.meet_with.field.trim();
        if let Some(entry) = entry
            && now_typed != typed
            && !now_typed.is_empty()
        {
            let reach = Reach::of(&entry.account);
            self.sync.suggest(&entry.account.id, now_typed, reach);
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
            self.sync.set_silenced(Silenced::of(&self.settings));
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
        self.close_on_welcome();
    }

    /// With neither an account nor a local calendar left, the welcome screen comes back
    /// (`MOD-09-F19`): the settings close, so they do not reopen over the calendar after a new
    /// connection.
    fn close_on_welcome(&mut self) {
        if local::welcome(self.accounts.list(), self.sync.calendars()) {
            self.dialog = None;
            self.connect.close();
        }
    }

    /// The local calendar is renamed or recoloured: the side panel, the pickers and its events
    /// show it so at once (`MOD-09-F03`). Its colour is its own: one chosen replaces a local
    /// colour kept for it before (`DEC-182`).
    fn change_local(&mut self, calendar: Calendar) {
        let key = (calendar.account_id.clone(), calendar.id.clone());
        let recoloured = (self.sync.calendars().iter())
            .any(|c| c.account_id == key.0 && c.id == key.1 && c.color != calendar.color);
        if recoloured && self.settings.colors.remove(&key).is_some() {
            self.save_settings();
        }
        for event in (self.cached.iter_mut())
            .filter(|e| e.account_id == calendar.account_id && e.calendar_id == calendar.id)
        {
            event.color = calendar.color;
        }
        self.sync.change_local(calendar);
        self.show_events(None);
    }

    /// The local calendar is gone with its events (`MOD-09-F04`): the settings forget it,
    /// the default calendar too if it was (`MOD-09-F11`); nothing done before can be undone, and
    /// the welcome screen comes back when nothing is left (`MOD-09-F19`).
    fn local_deleted(&mut self, calendar: &Calendar) {
        self.last = LastChange::default();
        let disabled = self.settings.disabled.clone();
        if self
            .settings
            .forget_calendar(&calendar.account_id, &calendar.id)
        {
            self.save_settings();
            if self.settings.disabled != disabled {
                self.sync.set_disabled(self.settings.disabled.clone());
            }
            self.sync.set_silenced(Silenced::of(&self.settings));
            self.rebuild();
        }
        self.close_on_welcome();
    }

    /// Opens the settings and signs in again to an account that needs it.
    fn sign_in_again(&mut self, ctx: &egui::Context, email: &str) {
        self.dialog = Some(Dialog::Settings);
        self.connect.sign_in_again(ctx, email);
    }

    /// "Use without an account": a local calendar, "My calendar", becomes the default calendar,
    /// and the calendar views open (`MOD-09-F17`).
    fn use_without_account(&mut self) {
        let calendar = local::my_calendar(self.zone);
        self.settings.default_calendar = Some((calendar.account_id.clone(), calendar.id.clone()));
        self.save_settings();
        self.sync.add_local(calendar);
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

    /// What the app does whether its window is shown or not: taking in what came from Google,
    /// the sign-ins and the notifications clicked.
    fn background(&mut self, ctx: &egui::Context) {
        if let Some(account) = self.connect.poll(ctx) {
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
        self.checks.tick(ctx, &self.accounts);
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
                        texts::not_undone(reason)
                    } else {
                        texts::not_saved(reason)
                    });
                }
                Saved::NotDeleted(local, reason) => {
                    self.saving.refused(*local);
                    self.last.refused(*local);
                    self.not_saved.push(texts::not_deleted(reason));
                }
                Saved::WithoutMeet(_) => {
                    self.not_saved.push(WITHOUT_MEET.to_string());
                }
            }
        }
        if cached.is_some() || !saved.is_empty() {
            self.show_events(cached);
        }
        self.gone.extend(self.sync.take_gone());
        for deletion in self.sync.take_local_deletions() {
            match deletion {
                LocalDeletion::Ask(calendar, events) => self.local_calendars.ask(calendar, events),
                LocalDeletion::Deleted(calendar) => self.local_deleted(&calendar),
            }
        }
        if let Some(opened) = self.sync.take_opened() {
            self.notified = Some(opened);
        }
        // Events and their reminders move to their new local times when the system's time zone
        // changes.
        if let Some(zone) = self.zone_changes.try_iter().last() {
            self.zone = zone;
            self.rebuild();
        }
        let window = self.window();
        self.sync.set_window(window);
    }

    /// Acts on what the user did to the tray icon or the launcher while the window is shown.
    fn signaled(&mut self, ctx: &egui::Context) {
        for signal in self.signals.try_iter().flatten().collect::<Vec<_>>() {
            match background::respond(signal, true, self.tray) {
                background::Response::Hide => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                background::Response::Quit => {
                    self.quitting = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                // Wayland lets no app bring its own window forward, so a window without the
                // keyboard is closed and opened again, in front.
                background::Response::Show | background::Response::Raise => {
                    if !ctx.input(|i| i.focused) {
                        self.reopening = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        }
    }
}

impl eframe::App for CalendarApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        appearance::apply(ui.ctx(), self.settings.theme, &self.desktop);
        self.signaled(ui.ctx());
        self.background(ui.ctx());
        // Looking at the app brings it up to date; a minimized window is polled less often.
        let (focused, minimized) = ui
            .ctx()
            .input(|i| (i.focused, i.viewport().minimized == Some(true)));
        // The window opens again as it was left.
        ui.ctx().input(|i| {
            let viewport = i.viewport();
            self.placement.size = viewport
                .inner_rect
                .map_or(self.placement.size, |r| r.size());
            self.placement.maximized = viewport.maximized.unwrap_or(self.placement.maximized);
        });
        if focused && !self.focused {
            self.sync.activated();
        }
        self.focused = focused;
        self.sync.set_shown(!minimized);
        // With neither an account nor a local calendar there is only the welcome screen
        // (`MOD-09-F18`); nothing until the local calendars are read.
        if local::welcome(self.accounts.list(), self.sync.calendars()) {
            if let Some(read) = self.sync.local()
                && connect::welcome(ui, &mut self.connect, read.err())
            {
                self.use_without_account();
            }
            return;
        }
        // The now line and today move; nothing else changes on its own.
        if Utc::now() - self.built_at >= chrono::Duration::minutes(1) {
            self.rebuild();
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(30));
        let before = self.navigation;
        self.shortcuts(ui.ctx());
        if let Some(event) = self.notified.take() {
            self.open_notified(ui.ctx(), &event);
        }
        let online = self.online.get();
        let sync_status = self.sync.status().clone();
        self.panel.resized(ui.ctx().content_rect().width());
        let bar =
            egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(12, 10));
        // What the bar shows, worked out first: its two sides only say what was clicked.
        let hint = if self.panel.expanded() {
            texts::HIDE_SIDE_PANEL
        } else {
            texts::SHOW_SIDE_PANEL
        };
        // The agenda always starts now; there is no period to step through.
        let periods = self.navigation.view != View::Agenda;
        let regional = &self.regional;
        let title = match (self.navigation.view, self.navigation.period()) {
            (View::Month, _) => regional.month_year(self.navigation.selected),
            (_, Some((first, last))) if first == last => regional.short_date(first),
            (_, Some((first, last))) => regional.date_range(first, last),
            (_, None) => texts::UPCOMING.to_string(),
        };
        // Accounts that cannot be checked are not synchronized either (`DEC-52`).
        let state = match sync_status.state(online) {
            State::Offline => State::Offline,
            _ if self.problem.is_some() => State::Error,
            state => state,
        };
        // The synchronization indicator (`MOD-02-F17`, `MOD-02-U03`), with the full sentence of
        // the last synchronization on hover.
        let today = self.today();
        let last = sync_status.last_synced;
        let indicator = status::indicator(state, last, self.zone, today, regional);
        let synced = status::last_synced(last, self.zone, today, regional);
        // The widths the indicator keeps, so the view picker stays put as it changes.
        let a_time = regional.time(chrono::NaiveTime::from_hms_opt(22, 22, 0).unwrap_or_default());
        let widest = [
            texts::SYNCHRONIZING.to_string(),
            texts::state_since(texts::UP_TO_DATE, &a_time),
            texts::state_since(texts::OFFLINE, &a_time),
        ];
        let widest: Vec<&str> = widest.iter().map(String::as_str).collect();
        let declined = !self.settings.hide_declined;
        let names: Vec<&str> = View::ALL.iter().map(|view| view.name()).collect();
        let picked = View::ALL
            .iter()
            .position(|view| *view == self.navigation.view)
            .unwrap_or(0);
        let (mut menu, mut to_today, mut previous, mut next) = (false, false, false, false);
        let (mut go_to_date, mut help) = (false, false);
        let (mut settings, mut refresh, mut switched) = (false, false, None);
        let mut declined_switched = false;
        egui::Panel::top("view-bar").frame(bar).show(ui, |ui| {
            // The right side first: in a narrow window the period's title is cut short rather
            // than drawn under the controls.
            egui::containers::Sides::new()
                .shrink_left()
                .truncate()
                .show(
                    ui,
                    |ui| {
                        // Google Calendar's main menu button, at the far left.
                        menu = widgets::menu_button(ui, hint).clicked();
                        ui.add_space(4.0);
                        to_today = widgets::outlined_button(ui, texts::TODAY).clicked();
                        ui.add_enabled_ui(periods, |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            previous =
                                icons::button(ui, Icon::Previous, texts::PREVIOUS_PERIOD, true)
                                    .clicked();
                            next =
                                icons::button(ui, Icon::Next, texts::NEXT_PERIOD, true).clicked();
                        });
                        // "Go to date", also on `g` (`DEC-161`).
                        go_to_date =
                            icons::button(ui, Icon::Calendar, texts::GO_TO_DATE, true).clicked();
                        ui.add_space(4.0);
                        ui.heading(title);
                    },
                    |ui| {
                        settings =
                            icons::button(ui, Icon::Settings, texts::SETTINGS, true).clicked();
                        // The shortcuts' list, also on `?`, where Google Calendar has its help.
                        help = widgets::help_button(ui, texts::KEYBOARD_SHORTCUTS).clicked();
                        // Turns while synchronizing.
                        let turning = state == State::Synchronizing;
                        refresh = widgets::refresh_button(ui, texts::REFRESH, turning).clicked();
                        let shown = egui::RichText::new(indicator).small();
                        let shown = if matches!(state, State::Delayed | State::Error) {
                            shown.color(ui.visuals().warn_fg_color)
                        } else {
                            shown.weak()
                        };
                        widgets::steady_label(ui, shown, &widest).on_hover_text(synced);
                        ui.add_space(8.0);
                        switched = widgets::segmented(ui, &names, picked);
                        // One switch shows or hides declined events, shown by default
                        // (`MOD-06-F22`).
                        let hint = if declined {
                            texts::HIDE_DECLINED
                        } else {
                            texts::SHOW_DECLINED
                        };
                        declined_switched =
                            widgets::icon_switch(ui, Icon::Eye, declined, hint).clicked();
                    },
                );
        });
        if go_to_date {
            self.go_to_date();
        }
        if help {
            self.dialog = Some(Dialog::Shortcuts);
        }
        if menu {
            self.settings.side_panel_collapsed = self.panel.toggle();
            self.save_settings();
        }
        if to_today {
            self.navigation.today(self.today());
            // Brings back a mini calendar browsed away, even when the view stays put.
            self.mini_month = self.navigation.selected;
        }
        if previous {
            self.navigation.previous();
        }
        if next {
            self.navigation.next();
        }
        if settings {
            self.dialog = Some(Dialog::Settings);
        }
        if refresh {
            self.sync.refresh();
        }
        if let Some(i) = switched {
            self.navigation.switch(View::ALL[i]);
        }
        if declined_switched {
            self.settings.hide_declined = declined;
            self.save_settings();
            self.rebuild();
        }
        // Everything cached can still be browsed (`MOD-02-U02`, `MOD-02-F15`).
        if let Some(indicator) = offline::indicator(online) {
            egui::Panel::top("offline").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(ui.visuals().warn_fg_color, indicator);
                    ui.label(texts::NO_CHANGES_OFFLINE);
                });
            });
        }
        // The calendar keeps showing what it has; only the accounts cannot be reached (`DEC-52`).
        if let Some(problem) = &self.problem {
            let mut again = false;
            egui::Panel::top("account-problem").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(ui.visuals().warn_fg_color, problem.message());
                    again = ui.button(texts::TRY_AGAIN).clicked();
                });
            });
            if again {
                self.problem = None;
                self.checks.now();
            }
        }
        // Local calendars that cannot be read are told of, the file left as it is (`MOD-09-NFR-01`).
        if let Some(Err(why)) = self.sync.local() {
            egui::Panel::top("local-problem").show(ui, |ui| {
                ui.colored_label(ui.visuals().warn_fg_color, why);
            });
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
                        ui.colored_label(warn, texts::needs_sign_in(email));
                        let button = egui::Button::new(texts::SIGN_IN_AGAIN);
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
        self.notices(ui.ctx(), online);
        let mut expanded = self.panel.expanded();
        let side = egui::Frame::side_top_panel(ui.style())
            .fill(appearance::side_fill(ui.visuals()))
            .inner_margin(egui::Margin::symmetric(14, 12));
        egui::Panel::left("side-panel")
            .frame(side)
            .resizable(false)
            .show_collapsible(ui, &mut expanded, |ui| {
                let today = self.today();
                // The days with events the views show (`MOD-03-F25`).
                let shown = side_panel::shown(&self.events, &self.settings);
                let mini = mini_calendar(self.mini_month, &self.navigation, today)
                    .with_events(shown.iter().map(|&i| &self.events[i]), self.zone);
                match mini_calendar::show(ui, &mini, &self.regional) {
                    Some(mini_calendar::Action::Browse(month)) => self.mini_month = month,
                    Some(mini_calendar::Action::GoTo(date)) => {
                        self.navigation.go_to(date);
                        self.mini_month = date;
                    }
                    None => {}
                }
                ui.separator();
                self.meet_with_panel(ui, online);
                ui.separator();
                // How many invitations await an answer; the counter opens their list
                // (`MOD-06-F10`).
                if !self.invitations.is_empty() {
                    let counter = texts::invitations_awaiting(self.invitations.len());
                    if ui.link(counter).clicked() {
                        self.dialog = Some(Dialog::Invitations);
                    }
                    ui.separator();
                }
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
                        let calendars = self.sync.calendars();
                        if let Some(calendar) = pick(&mut self.settings.colors, picked, calendars) {
                            self.change_local(calendar);
                        }
                    }
                    Some(calendar_list::Action::Toggled(calendar)) => {
                        side_panel::toggle(&mut self.settings.hidden, &calendar)
                    }
                    // Its events stay in the views (`MOD-07-F12`).
                    Some(calendar_list::Action::Muted(calendar)) => {
                        side_panel::toggle(&mut self.settings.muted, &calendar);
                        self.sync.set_silenced(Silenced::of(&self.settings));
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
            self.local_calendars.close();
        }
        self.navigated(before);
        // The people's calendars over the days shown, asked of Google as they change
        // (`MOD-06-F24`), shown once it answers.
        self.sync.ask_their(self.meeting_asked(online));
        let overlay = self.overlay(online);
        if overlay != self.overlay {
            self.overlay = overlay;
            self.view_state.events_changed();
            self.rebuild();
        }
        // Days on screen the cache does not hold: fetched online, a note offline (`MOD-02-F12`).
        let (first, last) = self.navigation.on_screen(self.today(), self.agenda_days);
        let browsed = offline::browsed(self.window(), first, last, online);
        self.sync.browse(browsed.fetch);
        let mut opened = None;
        let mut join = None;
        let mut started = None;
        let mut dropped = None;
        let mut unmovable = None;
        let zoom = self.settings.hour_zoom;
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(note) = browsed.note {
                ui.colored_label(ui.visuals().warn_fg_color, note);
            }
            if side_panel::nothing_shown(self.sync.calendars(), &self.settings) {
                ui.label(texts::NOTHING_SHOWN);
            }
            match &self.built {
                Built::Grid(view) => {
                    // Offline, only local events can be moved (`MOD-02-F15`), also by a drag under
                    // way.
                    let state = &mut self.view_state;
                    let hours = &mut self.settings.hour_zoom;
                    match grid_view::show(
                        ui,
                        view,
                        &self.regional,
                        state,
                        online,
                        &mut opened,
                        hours,
                    ) {
                        // Offline, only a local event is moved (MOD-09 *Constraints and edge cases*).
                        Some(grid_view::Action::Dropped { event, drop }) => {
                            dropped = (self.places.get(event))
                                .filter(|&&place| {
                                    offline::refused_for(online, &self.events[place]).is_none()
                                })
                                .map(|&place| (place, drop));
                        }
                        Some(grid_view::Action::Unmovable { event }) => {
                            unmovable = Some(event);
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
                    if agenda_view::show(ui, view, &self.regional, &mut opened, &mut join) {
                        self.agenda_days += AGENDA_DAYS;
                        self.rebuild();
                        ui.ctx().request_repaint();
                    }
                }
            }
        });
        // One height for the three views, kept across restarts (`MOD-03-F29`).
        if self.settings.hour_zoom != zoom {
            self.save_settings();
        }
        if let Some((event, drop)) = dropped {
            self.dropped(event, drop);
        }
        if let Some(link) = join {
            self.join(&link);
        }
        if let Some(event) = unmovable
            && let Some(event) = self.in_view(event).cloned()
        {
            tell_unmovable(&mut self.not_saved, &event);
        }
        if let Some((date, pressed, released)) = started {
            let zone = self.zone;
            self.start_event(|length| match released {
                Some(released) => create::selected(date, pressed, released, zone),
                None => create::clicked(date, pressed, length, zone),
            });
        }
        if let Some(event) = opened {
            let focused = ui.ctx().memory(|m| m.focused());
            self.open_in_view(event, focused);
        }
    }
}

/// The window over the app, which outlives it: closing the window leaves the app in the tray
/// (`MOD-07-F14`).
struct AppWindow<'a>(&'a mut CalendarApp);

impl eframe::App for AppWindow<'_> {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.0.ui(ui, frame);
    }
}

/// How long the app waits, while its window is closed, before looking at its accounts again.
const IN_TRAY_CHECK: std::time::Duration = std::time::Duration::from_secs(30);

/// The window's size and whether it is maximized.
struct Placement {
    size: egui::Vec2,
    maximized: bool,
}

impl Default for Placement {
    fn default() -> Self {
        Placement {
            size: egui::vec2(1200.0, 800.0),
            maximized: false,
        }
    }
}

fn window_options(placement: &Placement) -> eframe::NativeOptions {
    eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title(texts::APP_NAME)
            .with_inner_size(placement.size)
            .with_maximized(placement.maximized)
            .with_min_inner_size([480.0, 360.0])
            // The desktop finds the launcher's entry, and so the icon, by the application ID.
            .with_app_id(calendar_core::background::APP_ID)
            .with_icon(std::sync::Arc::new(
                eframe::icon_data::from_png_bytes(platform::ICON).expect("the app's icon"),
            )),
        // Also makes each new window's renderer get the whole font atlas of the context the
        // windows share; without it a reopened window panics on its first new glyph.
        system_font_fallback: true,
        ..Default::default()
    }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    // A `webcal:` link opened in the system goes to Google Calendar's page for adding it; the
    // calendar comes back through synchronization (`MOD-07-F18`).
    if let Some(link) = webcal::link(&args) {
        let _ = SystemBrowser.open(&calendar_google::subscribe_page(link));
        return Ok(());
    }
    // One context for every time the window opens, so what the app's threads ask of it reaches
    // the window shown.
    let ctx = egui::Context::default();
    let (sender, signals) = mpsc::channel::<Option<Signal>>();
    let signaled = {
        let (sender, ctx) = (sender.clone(), ctx.clone());
        move |signal: Signal| {
            let _ = sender.send(Some(signal));
            ctx.request_repaint();
        }
    };
    // Only one copy runs; a second launch brings the first one's window forward (`MOD-07-F17`).
    let instance = SystemInstance::new();
    let launched = signaled.clone();
    if !instance.claim(Box::new(move || launched(Signal::Launched))) {
        return Ok(());
    }
    // While the window is closed, the app wakes whenever its threads ask for the window to be
    // drawn, e.g. when a notification is clicked.
    ctx.set_repaint_observer(move |info| {
        if info.delay.is_zero() {
            let _ = sender.send(None);
        }
    });
    // Off the UI thread, as the window opens.
    std::thread::spawn(|| XdgLinkHandler::new().register());
    // Losing it only leaves the app without its icon in the desktop's menus and task bar.
    std::thread::spawn(|| XdgLauncher::new().install());
    let tray = SystemTray::new();
    let tray_shown = tray.show(Box::new(signaled), background::at_login(&args));
    let mut shown = background::starts_shown(&args, tray_shown);
    let mut app = CalendarApp::new(ctx.clone(), signals, tray_shown);
    loop {
        if shown {
            eframe::run_native_ext(
                background::APP_ID,
                window_options(&app.placement),
                Some(ctx.clone()),
                Box::new(|_| Ok(Box::new(AppWindow(&mut app)))),
            )?;
            let closed = background::respond(Signal::Closed, true, tray_shown);
            if app.quitting || (closed == background::Response::Quit && !app.reopening) {
                return Ok(());
            }
            shown = std::mem::take(&mut app.reopening);
            if shown {
                continue;
            }
        }
        // In the tray, the app keeps synchronizing, less often, and reminding (`MOD-07-F14`).
        app.sync.set_shown(false);
        match app.signals.recv_timeout(IN_TRAY_CHECK) {
            Ok(Some(signal)) => match background::respond(signal, false, tray_shown) {
                background::Response::Show => shown = true,
                background::Response::Quit => return Ok(()),
                background::Response::Hide | background::Response::Raise => {}
            },
            Ok(None) | Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
        app.background(&ctx);
        // A notification clicked opens its event in the window (`MOD-07-F08`).
        shown |= app.notified.is_some();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calendar_core::guests::Response;

    #[test]
    fn the_guest_answers_icons_are_in_the_apps_fonts() {
        let ctx = egui::Context::default();
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .textures_delta
            .clear();
        let font = egui::FontId::proportional(14.0);
        for answer in [
            Response::Accepted,
            Response::Declined,
            Response::Maybe,
            Response::Awaiting,
        ] {
            let icon = answer.icon();
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, icon)), "{icon}");
        }
    }

    /// The string literals of `source` before its tests, with the lines they start on; comments
    /// and the messages of panics, which the user never reads, are left out.
    fn literals(source: &str) -> Vec<(usize, String)> {
        let code = source.split("#[cfg(test)]\nmod tests").next().unwrap();
        let (mut found, mut line, mut chars) = (Vec::new(), 1, code.char_indices().peekable());
        while let Some((at, c)) = chars.next() {
            match c {
                '\n' => line += 1,
                '/' if chars.peek().is_some_and(|&(_, next)| next == '/') => {
                    while chars.next_if(|&(_, c)| c != '\n').is_some() {}
                }
                '"' => {
                    let (start, mut literal) = (line, String::new());
                    while let Some((_, c)) = chars.next() {
                        match c {
                            '"' => break,
                            '\\' => match chars.next() {
                                // A line continued: the next line's indentation is not in it.
                                Some((_, '\n')) => {
                                    line += 1;
                                    while chars.next_if(|&(_, c)| c == ' ').is_some() {}
                                }
                                Some((_, c)) => literal.push(c),
                                None => {}
                            },
                            c => {
                                line += usize::from(c == '\n');
                                literal.push(c);
                            }
                        }
                    }
                    let before = code[..at].trim_end();
                    let panic = ["unreachable!(", "expect(", "panic!("]
                        .iter()
                        .any(|p| before.ends_with(p));
                    if !panic {
                        found.push((start, literal));
                    }
                }
                _ => {}
            }
        }
        found
    }

    /// Whether a literal reads as words to the user: with its `{…}` arguments left out, a word
    /// of two letters or more that is capitalized or followed by another. A single lowercase word
    /// is not caught: it cannot be told from the IDs the views give egui.
    fn is_text(literal: &str) -> bool {
        let mut bare = String::new();
        let mut depth = 0;
        for c in literal.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                c if depth == 0 => bare.push(c),
                _ => {}
            }
        }
        let words: Vec<&str> = bare
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|w| !w.is_empty())
            .collect();
        let capitalized = bare.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && bare.chars().nth(1).is_some_and(|c| c.is_ascii_lowercase());
        words.iter().any(|w| w.len() >= 2)
            && (capitalized || words.len() >= 2 && bare.contains(' '))
    }

    #[test]
    fn the_views_keep_no_interface_texts_of_their_own() {
        // Every text the user reads comes from one table (`MOD-04-F04`, `DEC-31`).
        assert!(is_text("Save") && is_text("No upcoming events") && is_text("{n} more events"));
        assert!(!is_text("editor-repeat") && !is_text("{} {title}") && !is_text("☰"));
        let sample = "let a = \"x-id\"; // \"Not this\"\nlet b = \"One \\\n    two\";\nx.expect(\"Not it\");";
        assert_eq!(
            literals(sample),
            [(1, "x-id".into()), (2, "One two".into())]
        );
        let views = [
            ("agenda_view.rs", include_str!("agenda_view.rs")),
            ("appearance.rs", include_str!("appearance.rs")),
            ("calendar_list.rs", include_str!("calendar_list.rs")),
            ("connect.rs", include_str!("connect.rs")),
            ("editor.rs", include_str!("editor.rs")),
            ("grid_view.rs", include_str!("grid_view.rs")),
            ("local_calendars.rs", include_str!("local_calendars.rs")),
            ("main.rs", include_str!("main.rs")),
            ("meet_with.rs", include_str!("meet_with.rs")),
            ("mini_calendar.rs", include_str!("mini_calendar.rs")),
            ("month_view.rs", include_str!("month_view.rs")),
            ("remove.rs", include_str!("remove.rs")),
            ("widgets.rs", include_str!("widgets.rs")),
        ];
        for (file, source) in views {
            for (line, literal) in literals(source) {
                assert!(!is_text(&literal), "{file}:{line}: \"{literal}\"");
            }
        }
    }
}
