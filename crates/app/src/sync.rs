//! Synchronizing with Google off the UI thread (`US-02-01`, `US-02-03`, `NFR-01`): one worker owns
//! the cache and shows what it holds at start. Each connected account's first synchronization of
//! the run starts once Google has confirmed the account's sign-in; from then on its calendars are
//! polled as the `Schedule` says, and synchronized at once on "Refresh", when the window is
//! activated, when an event is opened, after waking from sleep and when the network returns. Each
//! round first brings the accounts' calendar lists up to date (`US-02-04`): new calendars are
//! synchronized, and those gone go with their events, the user being told once. Calendars disabled
//! in the settings stay listed but are never synchronized. A removed account's data is deleted.
//! Days on screen that the cache does not hold are fetched from Google while browsed, and shown
//! without being kept (`US-02-05`). Whenever what the cache holds may have changed, the views get
//! the events to show. The views are told when synchronization runs, when it last got through,
//! and which calendars are not up to date and why; a calendar that fails waits longer before each
//! retry, and the others go on (`US-02-06`). A new event goes to Google first and comes back
//! through synchronizing its calendar (`US-05-01`), and so does a changed one (`US-05-02`), a
//! deleted one and one restored (`US-05-04`); each account's default event length is read with
//! its first synchronization. People to invite are searched for at Google while the user types a
//! guest (`US-06-01`). An event saved with Google Meet that comes back without a call is told of
//! (`US-06-07`). The guests' busy times on the day chosen in the editor are asked of Google
//! (`US-06-08`). The reminders of the events the cache holds are shown as desktop notifications
//! when due, and a click on one opens its event (`US-07-01`). Join on a notification opens the
//! event's Google Meet call in the browser, and Snooze shows it again 5 or 10 minutes later,
//! without telling Google (`US-07-02`).

use std::collections::hash_map::RandomState;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::BuildHasher;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use calendar_core::accounts::Check;
use calendar_core::availability::{Asked, Availability};
use calendar_core::cache::{self, Cache};
use calendar_core::create::{DEFAULT_LENGTH, NewEvent};
use calendar_core::editor::{Edit, Patch};
use calendar_core::guests::{Person, Reach};
use calendar_core::model::{Calendar, Event};
use calendar_core::platform::{Browser, Notifier, SecretStore, Wakeups};
use calendar_core::regional::Regional;
use calendar_core::reminders::{Action, Alarms, EventId, ReminderId, Silenced};
use calendar_core::schedule::Schedule;
use calendar_core::series::{self, Plan, Scope, Split};
use calendar_core::status::{Problem, Status};
use calendar_core::sync::{
    Provider, SyncError, SyncedEvent, Window, fetch_events, shown_events, sync_calendar_list,
    sync_calendars,
};
use calendar_google::{Endpoints, Session};
use chrono::{DateTime, Utc};
use eframe::egui;

use crate::platform::{SystemBrowser, SystemNotifier, SystemSecretStore, SystemWakeups};

enum Job {
    Sync(String, Window),
    /// Google turned the account's sign-in away: it is not polled until it works again.
    Stop(String),
    /// Google could not confirm the sign-in of an account not synchronized yet in this run.
    Unconfirmed(String),
    Forget(String, Window),
    Refresh,
    Activated,
    /// One of the calendar's events is being opened.
    Opening(String),
    Shown(bool),
    /// The days the cache holds moved, as days pass or the time zone changes.
    Window(Window),
    /// The calendars disabled in the settings changed.
    Disabled(Disabled),
    /// The days on screen that the cache does not hold and are to be fetched, if any.
    Browse(Option<Window>),
    /// A new event to create in the calendar, by the number the views know it by.
    Create(u64, Calendar, NewEvent),
    /// A change to an event, by the number the views know it by.
    Change(u64, Box<Edit>),
    /// An event to delete from the calendar, by the number the views know it by, and whether
    /// Google e-mails its guests (`MOD-06-F12`).
    Delete(u64, Calendar, Event, bool),
    /// An event deleted just now to bring back to the calendar, by the number the views know it
    /// by, and whether Google e-mails its guests.
    Restore(u64, Calendar, Event, bool),
    /// A change to an occurrence of a series and the following or all events, by the number the
    /// views know it by (`MOD-05-F20`).
    ChangeSeries(u64, Box<Edit>, Scope),
    /// An occurrence of a series to delete from the calendar with the following or all events,
    /// by the number the views know it by (`MOD-05-F21`), and whether Google e-mails its guests.
    DeleteSeries(u64, Calendar, Event, Scope, bool),
    /// What is typed in the guest field, to search the account's people for where it may look
    /// (`MOD-06-F02`, `F04`).
    Suggest(String, String, Reach),
    /// The guests' busy times the editor's preview asks for (`MOD-06-F19`).
    Busy(Asked),
    /// Reminders are to be shown from now on (`MOD-07-F01`), their times written as the system
    /// writes them.
    Remind(Box<dyn Notifier>, Regional, Acts),
    /// A reminder shown is to be shown again in so many minutes from when Snooze was chosen
    /// (`MOD-07-F10`).
    Snooze(ReminderId, u32, DateTime<Utc>),
}

/// What the user chose on a notification, for the app to do.
#[derive(Debug, PartialEq, Eq)]
enum Chosen {
    /// The views are to open its event (`MOD-07-F08`).
    Open(Opened),
    /// The Google Meet call at the link is to be joined in the browser (`MOD-07-F09`).
    Join(String),
    /// The reminder is to be shown again in so many minutes (`MOD-07-F10`).
    Snooze(ReminderId, u32),
}

/// Does what the user chose on a notification.
type Acts = Arc<dyn Fn(Chosen) + Send + Sync>;

/// The event of a notification clicked, to open (`MOD-07-F08`), with its reminder to snooze from
/// the event's details when the desktop's notifications have no buttons (`DEC-64`).
#[derive(Debug, PartialEq, Eq)]
pub struct Opened {
    pub event: EventId,
    pub snooze: Option<ReminderId>,
}

enum Update {
    /// The calendars listed in the cache, and the events to show.
    Events(Vec<Calendar>, Vec<Event>),
    /// The account's synchronization did not start; it is tried again after its next check.
    Failed(String),
    /// The names of calendars gone from their accounts' lists (`MOD-02-F21`).
    Gone(Vec<String>),
    /// A synchronization is under way.
    Synchronizing,
    /// Where synchronization stands, once the worker is done for now.
    Status(Status),
    /// Google created the new event under this ID.
    Created(u64, String),
    /// Google did not create the new event, and why; never holds the event's contents.
    NotCreated(u64, String),
    /// The new or changed event was saved with Google Meet, but came back without a call.
    WithoutMeet(u64),
    /// Google changed the event; its new version.
    Changed(u64, String),
    /// Google did not change the event, and why; never holds the event's contents.
    NotChanged(u64, String),
    /// Google deleted the event.
    Deleted(u64),
    /// Google did not delete the event, and why; never holds the event's contents.
    NotDeleted(u64, String),
    /// The account's default event length, in minutes (`MOD-05-F09`).
    DefaultLength(String, u32),
    /// The people of the account found for what was last typed in the guest field.
    People(String, Vec<Person>),
    /// The guests' busy times Google gave for what was asked; `None` when it gave none.
    Busy(Asked, Option<Vec<Availability>>),
}

/// Calendars disabled in the settings, by account ID and calendar ID (`MOD-02-F06`).
type Disabled = BTreeSet<(String, String)>;

/// Signs in to an account at Google.
type SignIn = Box<dyn Fn(&str) -> Result<Box<dyn Provider + Send>, SyncError> + Send>;

pub struct Synchronization {
    jobs: Sender<Job>,
    updates: Receiver<Update>,
    /// Accounts whose first synchronization of this run is done or under way.
    synced: HashSet<String>,
    /// Whether the window is shown, as last told to the worker.
    shown: bool,
    /// The days the cache holds, as last told to the worker.
    window: Window,
    /// The names of calendars gone from their accounts' lists, not yet taken.
    gone: Vec<String>,
    /// The days outside the cache to fetch, as last told to the worker.
    browsed: Option<Window>,
    /// Where synchronization stands, as the worker last told.
    status: Status,
    /// The calendars listed in the cache, as the worker last told.
    calendars: Vec<Calendar>,
    /// Each account's default event length, in minutes, once Google told it.
    lengths: HashMap<String, u32>,
    /// What became of new and changed events at Google, not yet taken.
    saved: Vec<Saved>,
    /// The people Google last found to invite, and the account they were found for.
    people: Option<(String, Vec<Person>)>,
    /// The guests' busy times last asked for, if the editor still asks for them.
    asked: Option<Asked>,
    /// The guests' busy times Google last gave, and what they were asked for.
    busy: Option<(Asked, Option<Vec<Availability>>)>,
    /// The events of notifications clicked, not yet taken.
    opens: Sender<Opened>,
    opened: Receiver<Opened>,
    /// The notifications not shown, shared with the worker so a change holds at once.
    silenced: Arc<Mutex<Silenced>>,
}

/// What became of a new, changed or deleted event at Google.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Saved {
    Created(u64, String),
    /// Why it was not created.
    NotCreated(u64, String),
    /// Saved, but without the Google Meet call asked for (MOD-06 *Constraints and edge cases*).
    WithoutMeet(u64),
    /// The event's new version.
    Changed(u64, String),
    /// Why it was not changed.
    NotChanged(u64, String),
    Deleted(u64),
    /// Why it was not deleted.
    NotDeleted(u64, String),
}

impl Synchronization {
    /// Starts the worker on the cache file; without a home directory the cache lives in memory.
    /// Data of accounts not in `account_ids` is deleted, and the disabled calendars' events.
    pub fn start(
        ctx: &egui::Context,
        account_ids: Vec<String>,
        window: Window,
        disabled: Disabled,
        silenced: Silenced,
        regional: Regional,
    ) -> Self {
        let store: Arc<dyn SecretStore> = Arc::new(SystemSecretStore);
        let endpoints = Endpoints::google();
        let client_file = calendar_google::client_file_path();
        let sign_in: SignIn = Box::new(move |id| {
            Session::sign_in(&client_file, &endpoints, store.as_ref(), id)
                .map(|session| Box::new(session) as Box<dyn Provider + Send>)
        });
        let open = || {
            cache::default_path()
                .and_then(|path| Cache::open(&path).ok())
                .unwrap_or_else(Cache::in_memory)
        };
        let sync = Synchronization::with(ctx, open, sign_in, account_ids, window, disabled);
        sync.wake_on(&SystemWakeups);
        sync.set_silenced(silenced);
        sync.remind_with(ctx, Box::new(SystemNotifier::new()), regional);
        sync
    }

    fn with(
        ctx: &egui::Context,
        open: impl FnOnce() -> Cache + Send + 'static,
        sign_in: SignIn,
        account_ids: Vec<String>,
        window: Window,
        disabled: Disabled,
    ) -> Self {
        let (jobs, received) = mpsc::channel();
        let (sender, updates) = mpsc::channel();
        let (opens, opened) = mpsc::channel();
        let silenced = Arc::new(Mutex::new(Silenced::default()));
        let shared = silenced.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let send = |update| {
                let _ = sender.send(update);
                ctx.request_repaint();
            };
            let mut engine = Engine::new(open(), sign_in, window, disabled);
            engine.silenced = shared;
            let _ = engine.cache.retain_accounts(&account_ids);
            engine.clear_disabled();
            send(engine.shown());
            send(Update::Status(engine.status()));
            loop {
                let next = (engine.schedule.next().into_iter())
                    .chain(engine.next_reminder(Instant::now()))
                    .min();
                let job = match next {
                    Some(next) => {
                        match received.recv_timeout(next.saturating_duration_since(Instant::now()))
                        {
                            Ok(job) => Some(job),
                            Err(RecvTimeoutError::Timeout) => None,
                            Err(RecvTimeoutError::Disconnected) => return,
                        }
                    }
                    None => match received.recv() {
                        Ok(job) => Some(job),
                        Err(_) => return,
                    },
                };
                // Reminders due, e.g. on waking, do not wait for the synchronization it starts.
                engine.remind();
                // Woken for a reminder alone, nothing is polled.
                let polled = engine
                    .schedule
                    .next()
                    .is_some_and(|due| due <= Instant::now());
                let mut jobs: Vec<Option<Job>> = match job {
                    None if !polled => Vec::new(),
                    job => vec![job],
                };
                jobs.extend(received.try_iter().map(Some));
                if !jobs.is_empty() {
                    if jobs.iter().any(synchronizes) {
                        send(Update::Synchronizing);
                    }
                    for job in latest_only(jobs) {
                        for update in engine.handle(job, Instant::now()) {
                            send(update);
                        }
                    }
                    send(Update::Status(engine.status()));
                }
                engine.remind();
            }
        });
        Synchronization {
            jobs,
            updates,
            synced: HashSet::new(),
            shown: true,
            window,
            gone: Vec::new(),
            browsed: None,
            status: Status::default(),
            calendars: Vec::new(),
            lengths: HashMap::new(),
            saved: Vec::new(),
            people: None,
            asked: None,
            busy: None,
            opens,
            opened,
            silenced,
        }
    }

    /// Shows the reminders of the cached events through the desktop's notifications from now
    /// on; those whose time passed before are not shown (`MOD-07-F20`).
    fn remind_with(&self, ctx: &egui::Context, notifier: Box<dyn Notifier>, regional: Regional) {
        let (opens, jobs, ctx) = (self.opens.clone(), self.jobs.clone(), ctx.clone());
        let act: Acts = Arc::new(move |chosen| match chosen {
            Chosen::Open(opened) => {
                let _ = opens.send(opened);
                ctx.request_repaint();
            }
            // There is no one to tell if the browser does not start: the notification is gone.
            Chosen::Join(link) => {
                let _ = SystemBrowser.open(&link);
            }
            Chosen::Snooze(reminder, minutes) => {
                let _ = jobs.send(Job::Snooze(reminder, minutes, Utc::now()));
            }
        });
        let _ = self.jobs.send(Job::Remind(notifier, regional, act));
    }

    /// The event of the notification the user last clicked since asked, to open (`MOD-07-F08`).
    pub fn take_opened(&self) -> Option<Opened> {
        self.opened.try_iter().last()
    }

    /// Shows the reminder again in so many minutes; Google is not told (`MOD-07-F10`).
    pub fn snooze(&self, reminder: ReminderId, minutes: u32) {
        let _ = self.jobs.send(Job::Snooze(reminder, minutes, Utc::now()));
    }

    /// The notifications not to show from now on, as the settings say; Google is not told
    /// (`MOD-07-F11`, `F12`).
    pub fn set_silenced(&self, silenced: Silenced) {
        *self.silenced.lock().unwrap() = silenced;
    }

    /// Synchronizes everything at once after waking from sleep and when the network returns.
    fn wake_on(&self, wakeups: &dyn Wakeups) {
        let jobs = self.jobs.clone();
        wakeups.watch(Box::new(move || {
            let _ = jobs.send(Job::Refresh);
        }));
    }

    /// Google confirmed or turned away these accounts' sign-ins: those confirmed and not
    /// synchronized yet in this run are, those turned away are no longer polled. Those Google
    /// could not confirm before their first synchronization count as failed.
    pub fn checked(&mut self, checked: &[(String, Check)], window: Window) {
        for (id, check) in checked {
            match check {
                Check::Works if self.synced.insert(id.clone()) => {
                    let _ = self.jobs.send(Job::Sync(id.clone(), window));
                }
                Check::SignInNeeded => {
                    // Also an account whose first synchronization got through only on a retry.
                    self.synced.remove(id);
                    let _ = self.jobs.send(Job::Stop(id.clone()));
                }
                Check::Failed(_) if !self.synced.contains(id) => {
                    let _ = self.jobs.send(Job::Unconfirmed(id.clone()));
                }
                _ => {}
            }
        }
    }

    /// The account was removed: its calendars and events are deleted (`DEC-85`).
    pub fn forget(&mut self, id: &str, window: Window) {
        self.synced.remove(id);
        let _ = self.jobs.send(Job::Forget(id.to_string(), window));
    }

    /// "Refresh": every calendar at once (`MOD-02-F09`).
    pub fn refresh(&self) {
        let _ = self.jobs.send(Job::Refresh);
    }

    /// The window became active: the calendars not synchronized just now are (`MOD-02-F23`).
    pub fn activated(&self) {
        let _ = self.jobs.send(Job::Activated);
    }

    /// One of the calendar's events is being opened: the calendar is synchronized at once
    /// (`MOD-02-F23`).
    pub fn opening(&self, calendar_id: &str) {
        let _ = self.jobs.send(Job::Opening(calendar_id.to_string()));
    }

    /// Whether the window is shown; while hidden, calendars are polled less often.
    pub fn set_shown(&mut self, shown: bool) {
        if shown != self.shown {
            self.shown = shown;
            let _ = self.jobs.send(Job::Shown(shown));
        }
    }

    /// The days the cache holds, around today. Called every frame.
    pub fn set_window(&mut self, window: Window) {
        if window != self.window {
            self.window = window;
            let _ = self.jobs.send(Job::Window(window));
        }
    }

    /// The calendars disabled in the settings changed: a newly disabled calendar's events are
    /// deleted, and a newly enabled one is synchronized at once (`MOD-02-F06`).
    pub fn set_disabled(&self, disabled: Disabled) {
        let _ = self.jobs.send(Job::Disabled(disabled));
    }

    /// The days on screen that the cache does not hold, to fetch from Google and show without
    /// keeping them (`MOD-02-F12`); `None` when there are none, or offline. Called every frame.
    pub fn browse(&mut self, outside: Option<Window>) {
        if outside != self.browsed {
            self.browsed = outside;
            let _ = self.jobs.send(Job::Browse(outside));
        }
    }

    /// Creates the event in the calendar at Google, then synchronizes the calendar
    /// (`MOD-05-F16`); `poll` tells what became of it by `local`.
    pub fn create(&self, local: u64, calendar: Calendar, event: NewEvent) {
        let _ = self.jobs.send(Job::Create(local, calendar, event));
    }

    /// Changes the event at Google, then synchronizes its calendars (`MOD-05-F16`); `poll` tells
    /// what became of it by `local`.
    pub fn change(&self, local: u64, edit: Box<Edit>) {
        let _ = self.jobs.send(Job::Change(local, edit));
    }

    /// Deletes the event from the calendar at Google, then synchronizes the calendar
    /// (`MOD-05-F12`), e-mailing its guests or not as `notify` says; `poll` tells what became of
    /// it by `local`.
    pub fn delete(&self, local: u64, calendar: Calendar, event: Event, notify: bool) {
        let _ = self.jobs.send(Job::Delete(local, calendar, event, notify));
    }

    /// Brings back the event deleted just now at Google, then synchronizes the calendar
    /// (`MOD-05-F27`), e-mailing its guests or not as `notify` says; `poll` tells what became of
    /// it by `local`, as of a change.
    pub fn restore(&self, local: u64, calendar: Calendar, event: Event, notify: bool) {
        let _ = self.jobs.send(Job::Restore(local, calendar, event, notify));
    }

    /// Changes the occurrence of a series and the following or all events at Google, by the
    /// number the views know it by, then synchronizes its calendar.
    pub fn change_series(&self, local: u64, edit: Box<Edit>, scope: Scope) {
        let _ = self.jobs.send(Job::ChangeSeries(local, edit, scope));
    }

    /// Deletes the occurrence of a series and the following or all events at Google, by the
    /// number the views know it by, e-mailing its guests or not as `notify` says, then
    /// synchronizes its calendar.
    pub fn delete_series(
        &self,
        local: u64,
        calendar: Calendar,
        event: Event,
        scope: Scope,
        notify: bool,
    ) {
        let _ = (self.jobs).send(Job::DeleteSeries(local, calendar, event, scope, notify));
    }

    /// Searches the account's people at Google for what is typed in the guest field, where
    /// `reach` lets it look; `people` gives them once found.
    pub fn suggest(&self, account_id: &str, query: &str, reach: Reach) {
        let job = Job::Suggest(account_id.to_string(), query.to_string(), reach);
        let _ = self.jobs.send(job);
    }

    /// The people Google last found to invite for the account; none until it answers.
    pub fn people(&self, account_id: &str) -> &[Person] {
        match &self.people {
            Some((account, people)) if account == account_id => people,
            _ => &[],
        }
    }

    /// The guests' busy times the editor's preview needs, `None` while it shows none: asked of
    /// Google when they differ from those last asked for (`MOD-06-F19`); `busy` gives them once
    /// answered. Called every frame.
    pub fn ask_busy(&mut self, asked: Option<Asked>) {
        if asked != self.asked {
            self.asked = asked.clone();
            if let Some(asked) = asked {
                let _ = self.jobs.send(Job::Busy(asked));
            }
        }
    }

    /// The guests' busy times Google gave for what is asked; `None` until it answers, and when it
    /// did not (`MOD-06-F20`). The same asked again, its last answer is given until the new one
    /// comes.
    pub fn busy(&self, asked: &Asked) -> Option<&[Availability]> {
        match &self.busy {
            Some((answered, Some(busy))) if answered == asked => Some(busy),
            _ => None,
        }
    }

    /// The length, in minutes, of a new event of the account started without a range: as set in
    /// its Google Calendar settings, or Google's default until known (`MOD-05-F09`).
    pub fn default_length(&self, account_id: &str) -> u32 {
        self.lengths
            .get(account_id)
            .copied()
            .unwrap_or(DEFAULT_LENGTH)
    }

    /// The events to show, when they changed. Called every frame.
    pub fn poll(&mut self) -> Option<Vec<Event>> {
        let mut events = None;
        for update in self.updates.try_iter() {
            match update {
                Update::Events(calendars, shown) => {
                    self.calendars = calendars;
                    events = Some(shown);
                }
                Update::Failed(id) => {
                    self.synced.remove(&id);
                }
                Update::Gone(names) => self.gone.extend(names),
                Update::Synchronizing => self.status.synchronizing = true,
                Update::Status(status) => self.status = status,
                Update::Created(local, id) => self.saved.push(Saved::Created(local, id)),
                Update::NotCreated(local, reason) => {
                    self.saved.push(Saved::NotCreated(local, reason))
                }
                Update::WithoutMeet(local) => self.saved.push(Saved::WithoutMeet(local)),
                Update::Changed(local, etag) => self.saved.push(Saved::Changed(local, etag)),
                Update::NotChanged(local, reason) => {
                    self.saved.push(Saved::NotChanged(local, reason))
                }
                Update::Deleted(local) => self.saved.push(Saved::Deleted(local)),
                Update::NotDeleted(local, reason) => {
                    self.saved.push(Saved::NotDeleted(local, reason))
                }
                Update::DefaultLength(account, minutes) => {
                    self.lengths.insert(account, minutes);
                }
                Update::People(account, people) => self.people = Some((account, people)),
                Update::Busy(asked, busy) => self.busy = Some((asked, busy)),
            }
        }
        events
    }

    /// The calendars of the connected accounts, as last read from the cache.
    pub fn calendars(&self) -> &[Calendar] {
        &self.calendars
    }

    /// Where synchronization stands, for the indicator (`MOD-02-F17`).
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// What became of new, changed and deleted events at Google since last asked, in the order
    /// Google answered; each is given once.
    pub fn take_saved(&mut self) -> Vec<Saved> {
        std::mem::take(&mut self.saved)
    }

    /// The names of calendars gone from their accounts' lists since last asked; each is given
    /// once (`MOD-02-F21`).
    pub fn take_gone(&mut self) -> Vec<String> {
        std::mem::take(&mut self.gone)
    }
}

/// The worker's state: the cache, which calendars are polled when, and the accounts signed in.
struct Engine {
    cache: Cache,
    schedule: Schedule,
    sign_in: SignIn,
    /// The accounts signed in during this run, by ID.
    sessions: HashMap<String, Box<dyn Provider + Send>>,
    /// Accounts whose first synchronization of this run did not get through.
    pending: HashSet<String>,
    /// The days the cache holds, as the views last gave them.
    window: Window,
    disabled: Disabled,
    /// The days on screen outside the cache, and their events as fetched; never cached.
    browsed: Option<Window>,
    outside: Vec<Event>,
    /// The polled calendars whose last synchronization failed, by account ID and calendar ID.
    problems: HashMap<(String, String), Problem>,
    /// Accounts Google turned away, until they work again.
    stopped: HashSet<String>,
    /// When a synchronization last got through, kept in the cache too.
    last_synced: Option<DateTime<Utc>>,
    clock: Box<dyn Fn() -> DateTime<Utc> + Send>,
    /// The reminders to show, once the app gave the desktop's notifications to show them with.
    reminding: Option<Reminding>,
    /// The notifications not shown when their reminders come, as the app last set them.
    silenced: Arc<Mutex<Silenced>>,
}

/// Reminders, and how they are shown.
struct Reminding {
    alarms: Alarms,
    notifier: Box<dyn Notifier>,
    act: Acts,
}

/// The longest wait for a reminder, so a changed system clock is followed.
const REMINDER_WAIT: std::time::Duration = std::time::Duration::from_secs(60);

impl Engine {
    fn new(cache: Cache, sign_in: SignIn, window: Window, disabled: Disabled) -> Self {
        Engine {
            schedule: Schedule::new(),
            sign_in,
            sessions: HashMap::new(),
            pending: HashSet::new(),
            window,
            disabled,
            browsed: None,
            outside: Vec::new(),
            problems: HashMap::new(),
            stopped: HashSet::new(),
            last_synced: cache.last_synced().ok().flatten(),
            clock: Box::new(Utc::now),
            reminding: None,
            silenced: Arc::default(),
            cache,
        }
    }

    /// Does the job, or with none the polling round that is due; returns what the views are to
    /// be told. The reminders follow the events the views are given, as they are now.
    fn handle(&mut self, job: Option<Job>, now: Instant) -> Vec<Update> {
        let updates = self.handle_job(job, now);
        if let Some(reminding) = &mut self.reminding {
            for update in &updates {
                if let Update::Events(calendars, events) = update {
                    let now = (self.clock)();
                    (reminding.alarms).set(calendars, events, self.window.zone, now);
                }
            }
        }
        updates
    }

    fn handle_job(&mut self, job: Option<Job>, now: Instant) -> Vec<Update> {
        let calendars = match job {
            None => self.schedule.due(now),
            Some(Job::Sync(id, window)) => {
                self.window = window;
                return self.first(&id, now);
            }
            Some(Job::Stop(id)) => {
                self.stop(&id);
                self.stopped.insert(id);
                return Vec::new();
            }
            Some(Job::Unconfirmed(id)) => {
                if !self.sessions.contains_key(&id) {
                    self.pending.insert(id);
                }
                return Vec::new();
            }
            Some(Job::Forget(id, window)) => {
                self.window = window;
                self.stop(&id);
                self.stopped.remove(&id);
                let _ = self.cache.forget_account(&id);
                self.fetch_outside();
                return vec![self.shown()];
            }
            Some(Job::Refresh) => {
                // Accounts not synchronized yet are tried again first.
                let mut updates = Vec::new();
                for id in self.pending.clone() {
                    updates.extend(self.first(&id, now));
                }
                let all = self.schedule.all();
                updates.extend(self.round(&all, now));
                if self.browsed.is_some() {
                    self.fetch_outside();
                    updates.push(self.shown());
                }
                return updates;
            }
            Some(Job::Activated) => self.schedule.stale(now),
            Some(Job::Opening(id)) => self.schedule.calendar(&id),
            Some(Job::Shown(shown)) => {
                self.schedule.set_shown(shown);
                return Vec::new();
            }
            Some(Job::Window(window)) => {
                self.window = window;
                return vec![self.shown()];
            }
            Some(Job::Disabled(disabled)) => {
                self.disabled = disabled;
                self.clear_disabled();
                self.enable(now);
                self.fetch_outside();
                return vec![self.shown()];
            }
            Some(Job::Browse(browsed)) => {
                self.browsed = browsed;
                self.fetch_outside();
                return vec![self.shown()];
            }
            Some(Job::Create(local, calendar, event)) => {
                return self.create(local, calendar, &event, now);
            }
            Some(Job::Change(local, edit)) => return self.change(local, &edit, now),
            Some(Job::Delete(local, calendar, event, notify)) => {
                return self.delete(local, calendar, &event, notify, now);
            }
            Some(Job::Restore(local, calendar, event, notify)) => {
                return self.restore(local, calendar, &event, notify, now);
            }
            Some(Job::ChangeSeries(local, edit, scope)) => {
                return self.change_series(local, &edit, scope, now);
            }
            Some(Job::DeleteSeries(local, calendar, event, scope, notify)) => {
                return self.delete_series(local, calendar, &event, scope, notify, now);
            }
            Some(Job::Suggest(account, query, reach)) => {
                // An account not signed in yet finds nobody.
                let people = (self.sessions.get(&account))
                    .map(|session| session.people(&query, reach))
                    .unwrap_or_default();
                return vec![Update::People(account, people)];
            }
            Some(Job::Busy(asked)) => {
                let busy = (self.sessions.get(&asked.account_id))
                    .and_then(|session| session.busy(&asked.emails, asked.day).ok());
                return vec![Update::Busy(asked, busy)];
            }
            Some(Job::Remind(notifier, regional, act)) => {
                let alarms = Alarms::new((self.clock)(), regional, self.window.zone);
                self.reminding = Some(Reminding {
                    alarms,
                    notifier,
                    act,
                });
                return vec![self.shown()];
            }
            Some(Job::Snooze(reminder, minutes, chosen)) => {
                if let Some(reminding) = &mut self.reminding {
                    (reminding.alarms).snooze(&reminder, minutes, chosen);
                }
                return Vec::new();
            }
        };
        self.round(&calendars, now)
    }

    /// The account's first synchronization of this run: it signs in, fetches the calendar list
    /// and synchronizes every enabled calendar; from then on they are polled. Calendars gone
    /// since the last run, e.g. deleted while the app was offline, are told of.
    fn first(&mut self, id: &str, now: Instant) -> Vec<Update> {
        self.pending.remove(id);
        self.stopped.remove(id);
        let listed = (self.sign_in)(id).and_then(|session| {
            let list = sync_calendar_list(session.as_ref(), id, &mut self.cache)?;
            Ok((session, list))
        });
        let mut updates = Vec::new();
        match listed {
            Ok((session, list)) => {
                // Without it, new events take Google's default length.
                if let Ok(minutes) = session.default_length() {
                    updates.push(Update::DefaultLength(id.to_string(), minutes));
                }
                updates.extend(gone(list.gone));
                let calendars = enabled(&self.disabled, &list.calendars);
                let synced =
                    sync_calendars(session.as_ref(), &calendars, self.window, &mut self.cache);
                self.schedule.set_account(id, &calendars, now);
                self.record(&calendars, &synced);
                self.sessions.insert(id.to_string(), session);
                self.fetch_outside();
            }
            Err(_) => {
                self.pending.insert(id.to_string());
                updates.push(Update::Failed(id.to_string()));
            }
        }
        updates.push(self.shown());
        updates
    }

    /// Synchronizes the calendars, each with its account's session, after bringing those
    /// accounts' calendar lists up to date (`MOD-02-F05`): calendars new on a list are
    /// synchronized too, and those gone from it are told of (`MOD-02-F21`). The views get the
    /// events when what the cache holds may have changed. A calendar that fails does not stop
    /// the others, and is tried again after a growing wait; a list that cannot be fetched stays
    /// as it was.
    fn round(&mut self, calendars: &[Calendar], now: Instant) -> Vec<Update> {
        self.schedule.synced(calendars, now);
        let mut changed = false;
        let mut updates = Vec::new();
        let mut results = Vec::new();
        for (account, session) in &self.sessions {
            let mut of_account: Vec<Calendar> = calendars
                .iter()
                .filter(|calendar| calendar.account_id == *account)
                .cloned()
                .collect();
            if of_account.is_empty() {
                continue;
            }
            if let Ok(list) = sync_calendar_list(session.as_ref(), account, &mut self.cache) {
                changed |= list.changed;
                updates.extend(gone(list.gone));
                let listed = enabled(&self.disabled, &list.calendars);
                let added = self.schedule.update_account(account, &listed, now);
                // Those still listed, as listed now (their access may have changed).
                of_account = listed
                    .into_iter()
                    .filter(|calendar| of_account.iter().any(|due| due.id == calendar.id))
                    .chain(added)
                    .collect();
            }
            let synced =
                sync_calendars(session.as_ref(), &of_account, self.window, &mut self.cache);
            // A failure can come after the calendar's events changed in the cache.
            changed |= synced.iter().any(|result| !matches!(result, Ok(false)));
            results.push((of_account, synced));
        }
        for (calendars, synced) in results {
            self.record(&calendars, &synced);
        }
        if changed {
            updates.push(self.shown());
        }
        updates
    }

    /// Creates the event at Google, then synchronizes its calendar, so the event reaches the cache
    /// the way every change does.
    fn create(
        &mut self,
        local: u64,
        calendar: Calendar,
        event: &NewEvent,
        now: Instant,
    ) -> Vec<Update> {
        let Some(session) = self.sessions.get(&calendar.account_id) else {
            return vec![Update::NotCreated(local, NOT_SIGNED_IN.to_string())];
        };
        match session.create(&calendar, event) {
            Ok(id) => {
                let mut updates = vec![Update::Created(local, id.clone())];
                updates.extend(self.synchronize(std::slice::from_ref(&calendar), now));
                if event.meet && self.without_meet(std::slice::from_ref(&calendar), &id) {
                    updates.push(Update::WithoutMeet(local));
                }
                updates
            }
            Err(error) => vec![Update::NotCreated(local, error.0)],
        }
    }

    /// Changes the event at Google, what changed first and the move after (`MOD-05-F15`), then
    /// synchronizes its calendars, so the change reaches the cache the way every change does. One
    /// changed elsewhere meanwhile is brought up to date with them, for the user to change again.
    fn change(&mut self, local: u64, edit: &Edit, now: Instant) -> Vec<Update> {
        let Some(session) = self.sessions.get(&edit.calendar.account_id) else {
            return vec![Update::NotChanged(local, NOT_SIGNED_IN.to_string())];
        };
        let (calendar, id) = (&edit.calendar, &edit.event.id);
        let mut changed = Ok(edit.event.etag.clone());
        if !edit.patch.is_empty() {
            changed = session.update(calendar, id, &edit.event.etag, &edit.patch, edit.notify);
        }
        // Saved with Google Meet, even when the move after it was not.
        let meet = edit.patch.meet == Some(true) && changed.is_ok();
        let mut updates = vec![match (changed, &edit.to) {
            (Err(error), _) => Update::NotChanged(local, refusal(error)),
            (Ok(etag), None) => Update::Changed(local, etag),
            (Ok(_), Some(to)) => match session.move_to(calendar, id, to, edit.notify) {
                Ok(etag) => Update::Changed(local, etag),
                // What was patched stays; only the move is told of.
                Err(error) if edit.patch.is_empty() => Update::NotChanged(local, error.0),
                Err(error) => Update::NotChanged(
                    local,
                    format!(
                        "it could not be moved to the other calendar ({}); its other changes \
                         were saved",
                        error.0
                    ),
                ),
            },
        }];
        let calendars: Vec<Calendar> = std::iter::once(calendar.clone())
            .chain(edit.to.clone())
            .collect();
        updates.extend(self.synchronize(&calendars, now));
        if meet && self.without_meet(&calendars, id) {
            updates.push(Update::WithoutMeet(local));
        }
        updates
    }

    /// Whether the event, saved with Google Meet in one of the calendars, came back from Google
    /// without a call: Google could not make one for the account (MOD-06 *Constraints and edge
    /// cases*).
    fn without_meet(&self, calendars: &[Calendar], id: &str) -> bool {
        (self.cache.calendars().ok().into_iter().flatten())
            .filter(|cached| {
                calendars.iter().any(|calendar| {
                    cached.calendar.account_id == calendar.account_id
                        && cached.calendar.id == calendar.id
                })
            })
            .flat_map(|cached| cached.events)
            .any(|event| event.id == id && event.about.meet.is_none())
    }

    /// Deletes the event at Google, against the version shown, then synchronizes its calendar,
    /// so the deletion reaches the cache the way every change does (`MOD-05-F16`).
    fn delete(
        &mut self,
        local: u64,
        calendar: Calendar,
        event: &Event,
        notify: bool,
        now: Instant,
    ) -> Vec<Update> {
        let Some(session) = self.sessions.get(&calendar.account_id) else {
            return vec![Update::NotDeleted(local, NOT_SIGNED_IN.to_string())];
        };
        let deleted = session.delete(&calendar, &event.id, &event.etag, notify);
        let mut updates = vec![match deleted {
            Ok(()) => Update::Deleted(local),
            Err(error) => Update::NotDeleted(local, refusal(error)),
        }];
        updates.extend(self.synchronize(&[calendar], now));
        updates
    }

    /// Brings back the event deleted just now at Google, then synchronizes its calendar.
    fn restore(
        &mut self,
        local: u64,
        calendar: Calendar,
        event: &Event,
        notify: bool,
        now: Instant,
    ) -> Vec<Update> {
        let Some(session) = self.sessions.get(&calendar.account_id) else {
            return vec![Update::NotChanged(local, NOT_SIGNED_IN.to_string())];
        };
        let mut updates = vec![match session.restore(&calendar, &event.id, notify) {
            Ok(etag) => Update::Changed(local, etag),
            Err(error) => Update::NotChanged(local, refusal(error)),
        }];
        updates.extend(self.synchronize(&[calendar], now));
        updates
    }

    /// Changes the occurrence of a series and the following or all events at Google, as its
    /// cached series says (`calendar_core::series`), then synchronizes its calendars.
    fn change_series(
        &mut self,
        local: u64,
        edit: &Edit,
        scope: Scope,
        now: Instant,
    ) -> Vec<Update> {
        let plan = self
            .series(&edit.calendar, &edit.event)
            .and_then(|(master, exceptions)| series::change(edit, scope, &master, &exceptions));
        match plan {
            Some(Plan::Change(edit)) => self.change(local, &edit, now),
            Some(Plan::Split(split)) => {
                let mut updates = vec![match self.split(&split, edit.notify) {
                    Ok(etag) => Update::Changed(local, etag),
                    Err(why) => Update::NotChanged(local, why),
                }];
                updates.extend(self.synchronize(&[split.calendar], now));
                updates
            }
            _ => vec![Update::NotChanged(local, NO_SERIES.to_string())],
        }
    }

    /// Deletes the occurrence of a series and the following or all events at Google, as its
    /// cached series says, then synchronizes its calendar.
    fn delete_series(
        &mut self,
        local: u64,
        calendar: Calendar,
        event: &Event,
        scope: Scope,
        notify: bool,
        now: Instant,
    ) -> Vec<Update> {
        let plan = self
            .series(&calendar, event)
            .and_then(|(master, exceptions)| {
                series::delete(&calendar, event, scope, &master, &exceptions)
            });
        match plan {
            Some(Plan::Delete(deleted)) => {
                let (calendar, event) = *deleted;
                self.delete(local, calendar, &event, notify, now)
            }
            Some(Plan::Split(split)) => {
                let mut updates = vec![match self.split(&split, notify) {
                    Ok(_) => Update::Deleted(local),
                    Err(why) => Update::NotDeleted(local, why),
                }];
                updates.extend(self.synchronize(&[split.calendar], now));
                updates
            }
            _ => vec![Update::NotDeleted(local, NO_SERIES.to_string())],
        }
    }

    /// The cached master of the series the occurrence `event` of the calendar belongs to, and its
    /// occurrences changed or deleted on their own.
    fn series(
        &self,
        calendar: &Calendar,
        event: &Event,
    ) -> Option<(SyncedEvent, Vec<SyncedEvent>)> {
        let (id, _) = event.series.as_ref()?;
        let events = (self.cache.calendars().ok()?.into_iter())
            .find(|cached| {
                cached.calendar.account_id == calendar.account_id
                    && cached.calendar.id == calendar.id
            })?
            .events;
        let (exceptions, others): (Vec<_>, Vec<_>) = (events.into_iter())
            .partition(|event| event.instance_of.as_ref().is_some_and(|(of, _)| of == id));
        let master = others.into_iter().find(|event| event.id == *id)?;
        Some((master, exceptions))
    }

    /// Ends the series before the occurrence at Google, against its version, then starts the new
    /// series from the occurrence as a copy of it, if any, and deletes the occurrences from there
    /// on changed on their own, each e-mailing the guests or not as `notify` says; the series'
    /// new version, or why not.
    fn split(&self, split: &Split, notify: bool) -> Result<String, String> {
        let Some(session) = self.sessions.get(&split.calendar.account_id) else {
            return Err(NOT_SIGNED_IN.to_string());
        };
        let ended = Patch {
            recurrence: Some(split.recurrence.clone()),
            ..Patch::default()
        };
        let etag = session
            .update(&split.calendar, &split.id, &split.etag, &ended, notify)
            .map_err(refusal)?;
        if let Some(new) = &split.new {
            session
                .copy(&split.calendar, &split.id, new, notify)
                .map_err(|error| {
                    format!(
                        "the series now ends before this event, but the following events could \
                         not be made a series of their own ({})",
                        error.0
                    )
                })?;
        }
        // The series' end won against its version; Google may have changed these with it.
        let failed = (split.changed.iter())
            .filter_map(|(id, _)| session.delete(&split.calendar, id, "", notify).err())
            .last();
        if let Some(error) = failed {
            return Err(format!(
                "the series now ends before this event, but an event of it changed on its own \
                 could not be deleted ({})",
                error.0
            ));
        }
        Ok(etag)
    }

    /// Synchronizes the calendars after a change to one of their events; an event on browsed
    /// days outside the cache comes back from there.
    fn synchronize(&mut self, calendars: &[Calendar], now: Instant) -> Vec<Update> {
        let mut updates = self.round(calendars, now);
        if self.browsed.is_some() {
            self.fetch_outside();
            updates.push(self.shown());
        }
        updates
    }

    /// Deletes the disabled calendars' events; they stay listed (`MOD-02-F06`).
    fn clear_disabled(&mut self) {
        for (account, calendar) in &self.disabled {
            let _ = self.cache.clear_calendar(account, calendar);
        }
    }

    /// Polls the enabled calendars of every signed-in account; those newly enabled are
    /// synchronized at once, in full, and those disabled are no longer polled (`MOD-02-F06`).
    fn enable(&mut self, now: Instant) {
        // An unreadable cache leaves the calendars polled as they were.
        let Ok(known) = self.cache.calendar_list() else {
            return;
        };
        let mut results = Vec::new();
        for (account, session) in &self.sessions {
            let of_account: Vec<Calendar> = known
                .iter()
                .filter(|calendar| calendar.account_id == *account)
                .cloned()
                .collect();
            let calendars = enabled(&self.disabled, &of_account);
            let added = self.schedule.update_account(account, &calendars, now);
            let synced = sync_calendars(session.as_ref(), &added, self.window, &mut self.cache);
            results.push((added, synced));
        }
        for (calendars, synced) in results {
            self.record(&calendars, &synced);
        }
    }

    /// Takes note of how each calendar's synchronization went (`MOD-02-F16`, `MOD-02-F18`): one
    /// that failed is in error, or delayed when Google asked to slow down, and waits longer
    /// before each retry; one that got through is up to date, and the time is kept.
    fn record(&mut self, calendars: &[Calendar], synced: &[Result<bool, SyncError>]) {
        let mut through = false;
        for (calendar, result) in calendars.iter().zip(synced) {
            let key = (calendar.account_id.clone(), calendar.id.clone());
            match result {
                Ok(_) => {
                    through = true;
                    self.schedule.succeeded(calendar);
                    self.problems.remove(&key);
                }
                Err(error) => {
                    self.schedule.failed(calendar, jitter());
                    let problem = if error.is_rate_limited() {
                        Problem::Delayed
                    } else {
                        Problem::Failed
                    };
                    self.problems.insert(key, problem);
                }
            }
        }
        if through {
            let at = (self.clock)();
            self.last_synced = Some(at);
            // Losing the time only shows an older one at the next start.
            let _ = self.cache.set_last_synced(at);
        }
    }

    /// Where synchronization stands: the enabled cached calendars not up to date, those of
    /// accounts Google turned away not synchronizing and those of accounts whose first
    /// synchronization did not get through in error; those of the other accounts not signed in
    /// yet wait for it.
    fn status(&self) -> Status {
        // An unreadable cache knows no calendars to tell of.
        let known = enabled(
            &self.disabled,
            &self.cache.calendar_list().unwrap_or_default(),
        );
        let waiting = known.iter().any(|calendar| {
            let account = &calendar.account_id;
            !self.sessions.contains_key(account)
                && !self.pending.contains(account)
                && !self.stopped.contains(account)
        });
        let problems = known
            .into_iter()
            .filter_map(|calendar| {
                let problem = if self.stopped.contains(&calendar.account_id) {
                    Some(Problem::NotSynchronizing)
                } else if self.pending.contains(&calendar.account_id) {
                    Some(Problem::Failed)
                } else {
                    let key = (calendar.account_id.clone(), calendar.id.clone());
                    self.problems.get(&key).copied()
                };
                problem.map(|problem| (calendar, problem))
            })
            .collect();
        Status {
            synchronizing: false,
            waiting,
            last_synced: self.last_synced,
            problems,
        }
    }

    /// Fetches the events of the browsed days outside the cache from every signed-in account's
    /// enabled calendars, in place of those fetched before (`MOD-02-F12`).
    fn fetch_outside(&mut self) {
        self.outside.clear();
        let Some(span) = self.browsed else {
            return;
        };
        // An unreadable cache knows no calendars to fetch.
        let known = self.cache.calendar_list().unwrap_or_default();
        for (account, session) in &self.sessions {
            let of_account: Vec<Calendar> = known
                .iter()
                .filter(|calendar| calendar.account_id == *account)
                .cloned()
                .collect();
            let calendars = enabled(&self.disabled, &of_account);
            self.outside
                .extend(fetch_events(session.as_ref(), &calendars, span));
        }
    }

    fn stop(&mut self, id: &str) {
        self.pending.remove(id);
        self.schedule.forget(id);
        self.sessions.remove(id);
        self.problems.retain(|(account, _), _| account != id);
    }

    fn shown(&self) -> Update {
        // An unreadable cache shows nothing until synchronization rewrites it.
        let cached = self.cache.calendars().unwrap_or_default();
        let calendars = cached.iter().map(|c| c.calendar.clone()).collect();
        let mut events = shown_events(&cached, self.window);
        // An event crossing the window's edge is cached already, maybe in a newer version.
        let cached: HashSet<(&str, &str)> = events
            .iter()
            .map(|event| (event.calendar_id.as_str(), event.id.as_str()))
            .collect();
        let outside: Vec<Event> = self
            .outside
            .iter()
            .filter(|event| !cached.contains(&(event.calendar_id.as_str(), event.id.as_str())))
            .cloned()
            .collect();
        events.extend(outside);
        Update::Events(calendars, events)
    }

    /// Shows the reminders due now as desktop notifications, each once (`MOD-07-F07`), and the
    /// snoozed ones due again; what the user chooses on one is done (`MOD-07-F08`–`F10`). Those
    /// silenced in the settings go by unshown (`MOD-07-F11`, `F12`).
    fn remind(&mut self) {
        let Some(reminding) = &mut self.reminding else {
            return;
        };
        let due = reminding.alarms.due((self.clock)());
        let silenced = self.silenced.lock().unwrap().clone();
        for notification in due.into_iter().filter(|n| !silenced.silences(n)) {
            let buttons = reminding.notifier.buttons();
            let (act, shown) = (reminding.act.clone(), notification.clone());
            let chosen = Box::new(move |action| match action {
                Action::Open => act(Chosen::Open(Opened {
                    event: shown.reminder.event.clone(),
                    snooze: (!buttons).then(|| shown.reminder.clone()),
                })),
                Action::Join => {
                    if let Some(link) = &shown.meet {
                        act(Chosen::Join(link.clone()));
                    }
                }
                Action::Snooze(minutes) => act(Chosen::Snooze(shown.reminder.clone(), minutes)),
                Action::Dismiss => {}
            });
            reminding.notifier.show(&notification, chosen);
        }
    }

    /// When the worker is to wake for the next reminder, at most a minute from `now`.
    fn next_reminder(&self, now: Instant) -> Option<Instant> {
        let at = self.reminding.as_ref()?.alarms.next()?;
        let wait = (at - (self.clock)()).to_std().unwrap_or_default();
        Some(now + wait.min(REMINDER_WAIT))
    }
}

/// Why a change to a series was not sent: its series is not in the cache, as for an invitation to
/// one occurrence of someone else's series, or not synchronized yet.
const NO_SERIES: &str =
    "its series is not in the calendar (yet); only this event can be changed or deleted";

/// Why a change was not sent: the event's account has not signed in in this run yet.
const NOT_SIGNED_IN: &str = "its account is not signed in yet; try again in a moment";

/// Why Google did not take a change; one made to an event changed elsewhere meanwhile is shown as
/// it is now, for the user to make again.
fn refusal(error: SyncError) -> String {
    if error.is_changed_elsewhere() {
        format!(
            "{}; it is shown as it is now, make your change again",
            error.0
        )
    } else {
        error.0
    }
}

/// The jobs waiting, with only the last of the browsing, of the searches for guests and of the
/// guests' busy times among them: days left behind, what was typed before and times chosen
/// before while the worker was busy are not fetched.
fn latest_only(jobs: Vec<Option<Job>>) -> Vec<Option<Job>> {
    let browsing = |job: &Option<Job>| matches!(job, Some(Job::Browse(_)));
    let searching = |job: &Option<Job>| matches!(job, Some(Job::Suggest(..)));
    let busy = |job: &Option<Job>| matches!(job, Some(Job::Busy(_)));
    let last_browsing = jobs.iter().rposition(browsing);
    let last_searching = jobs.iter().rposition(searching);
    let last_busy = jobs.iter().rposition(busy);
    jobs.into_iter()
        .enumerate()
        .filter(|(i, job)| {
            (!browsing(job) || Some(*i) == last_browsing)
                && (!searching(job) || Some(*i) == last_searching)
                && (!busy(job) || Some(*i) == last_busy)
        })
        .map(|(_, job)| job)
        .collect()
}

/// A number from 0 to 1, different each time: spreads the retries of calendars that failed
/// together.
fn jitter() -> f64 {
    let random = RandomState::new().hash_one(Instant::now());
    (random >> 11) as f64 / (1u64 << 53) as f64
}

/// Whether the job, or with none the polling round, may ask Google for something.
fn synchronizes(job: &Option<Job>) -> bool {
    !matches!(
        job,
        Some(
            Job::Stop(_)
                | Job::Unconfirmed(_)
                | Job::Shown(_)
                | Job::Window(_)
                | Job::Suggest(..)
                | Job::Busy(_)
                | Job::Remind(..)
                | Job::Snooze(..)
        )
    )
}

/// The calendars not disabled in the settings.
fn enabled(disabled: &Disabled, calendars: &[Calendar]) -> Vec<Calendar> {
    calendars
        .iter()
        .filter(|calendar| !disabled.contains(&(calendar.account_id.clone(), calendar.id.clone())))
        .cloned()
        .collect()
}

/// The notice of calendars gone from a list, if any.
fn gone(calendars: Vec<Calendar>) -> Option<Update> {
    (!calendars.is_empty()).then(|| {
        Update::Gone(
            calendars
                .into_iter()
                .map(|calendar| calendar.name)
                .collect(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::Duration;

    use crate::calendar_list::Picked;
    use crate::{Built, build, grid_view, pick, tell_unmovable};
    use calendar_core::availability::{self, Preview};
    use calendar_core::colors;
    use calendar_core::create::{self, QuickEntry, Saving};
    use calendar_core::details::event_details;
    use calendar_core::drag::{self, Target};
    use calendar_core::editor::{Editor, ReminderField, Save, Unit};
    use calendar_core::fixtures::d;
    use calendar_core::guests::{self, Person};
    use calendar_core::invitations;
    use calendar_core::model::{
        Access, Calendar, Kind, Method, Protected, Reminder, Reminders, Rgb, Show, Timing,
        Visibility,
    };
    use calendar_core::navigation::Navigation;
    use calendar_core::notify::{self, Pending};
    use calendar_core::offline;
    use calendar_core::platform::Network;
    use calendar_core::regional::Regional;
    use calendar_core::reminders::Notification;
    use calendar_core::series::{Question, Scope};
    use calendar_core::settings::Settings;
    use calendar_core::side_panel;
    use calendar_core::status::{self, State, label};
    use calendar_core::sync::SyncedEvent;
    use calendar_core::undo::{self, LastChange, Undo};
    use calendar_google::CLIENT_FILE;
    use calendar_google::fake::{
        ACCOUNT_ID, CLIENT_ID, EMAIL, FakeGoogle, FakeStore, OTHER_EMAIL, OTHER_ID,
    };
    use chrono::{TimeZone, Utc};
    use chrono_tz::Europe::Warsaw;
    use serde_json::{Value, json};

    use crate::network::Online;

    fn window() -> Window {
        Window::around(d(2026, 11, 11), Warsaw)
    }

    /// Google cannot be reached: every synchronization fails.
    fn unreachable_google() -> SignIn {
        Box::new(|_| Err(SyncError("Google could not be reached".into())))
    }

    /// A cache holding one event for each of the accounts.
    fn cache_of(accounts: &[&'static str]) -> impl FnOnce() -> Cache + Send + 'static {
        let accounts = accounts.to_vec();
        move || {
            let mut cache = Cache::in_memory();
            for account in accounts {
                let calendar = Calendar {
                    id: format!("{account}@example.com"),
                    account_id: account.to_string(),
                    name: account.to_string(),
                    color: Rgb(0x9f, 0xe1, 0xe7),
                    access: Access::Owner,
                    zone: Warsaw,
                    default_reminders: Vec::new(),
                };
                let start = Utc.with_ymd_and_hms(2026, 11, 10, 9, 0, 0).unwrap();
                let event = SyncedEvent {
                    id: format!("{account}-event"),
                    title: account.to_string(),
                    timing: Some(Timing::Timed {
                        start,
                        end: start + chrono::Duration::hours(1),
                        zone: Warsaw,
                    }),
                    recurrence: Vec::new(),
                    instance_of: None,
                    about: Default::default(),
                    etag: String::new(),
                    organizer_only: false,
                };
                cache
                    .set_calendars(account, std::slice::from_ref(&calendar))
                    .unwrap();
                cache.set_events(&calendar, &[event]).unwrap();
            }
            cache
        }
    }

    /// Waits for the worker's next events.
    fn next_events(sync: &mut Synchronization) -> Vec<String> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(events) = sync.poll() {
                return events.into_iter().map(|e| e.title).collect();
            }
            assert!(Instant::now() < deadline, "no events came");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn the_cached_events_of_connected_accounts_are_shown_at_start() {
        let ctx = egui::Context::default();
        let open = cache_of(&["1", "2", "gone"]);
        let ids = vec!["1".to_string(), "2".to_string()];

        let mut sync = Synchronization::with(
            &ctx,
            open,
            unreachable_google(),
            ids,
            window(),
            Disabled::new(),
        );

        let mut shown = next_events(&mut sync);
        shown.sort();
        assert_eq!(shown, ["1", "2"]);
    }

    /// The system's network, faked: offline from the start.
    struct NoNetwork;

    impl Network for NoNetwork {
        fn watch(&self, changed: Box<dyn Fn(bool) + Send + Sync>) {
            changed(false);
        }
    }

    #[test]
    fn tc_02_07_offline_the_cached_calendar_is_shown_at_once_with_the_indicator_and_changes_refused()
     {
        let ctx = egui::Context::default();
        let ids = vec!["1".to_string()];

        // 1. The app starts without network: the cached events come before any synchronization
        //    (none is asked for: no account check has got through), and the indicator is on.
        let online = Online::watch(&ctx, &NoNetwork);
        let mut sync = Synchronization::with(
            &ctx,
            cache_of(&["1"]),
            unreachable_google(),
            ids,
            window(),
            Disabled::new(),
        );

        assert_eq!(next_events(&mut sync), ["1"]);
        assert_eq!(offline::indicator(online.get()), Some(offline::OFFLINE));

        // 2. Creating, editing and deleting an event and answering an invitation are refused,
        //    the reason saying the app is offline.
        let reason = offline::refused(online.get()).unwrap();
        assert!(reason.contains("Creating, editing and deleting events and answering invitations"));
        assert!(reason.contains("not available offline"));
    }

    #[test]
    fn a_removed_accounts_events_go() {
        let ctx = egui::Context::default();
        let ids = vec!["1".to_string(), "2".to_string()];
        let mut sync = Synchronization::with(
            &ctx,
            cache_of(&["1", "2"]),
            unreachable_google(),
            ids,
            window(),
            Disabled::new(),
        );
        next_events(&mut sync);

        sync.forget("1", window());

        assert_eq!(next_events(&mut sync), ["2"]);
    }

    #[test]
    fn an_account_is_synchronized_once_per_run_and_again_after_a_failure() {
        let ctx = egui::Context::default();
        let ids = vec!["1".to_string()];
        let mut sync = Synchronization::with(
            &ctx,
            cache_of(&["1"]),
            unreachable_google(),
            ids,
            window(),
            Disabled::new(),
        );
        next_events(&mut sync);
        let works = [("1".to_string(), Check::Works)];

        sync.checked(&works, window());
        sync.checked(&works, window());
        sync.checked(&[("1".to_string(), Check::SignInNeeded)], window());

        // The synchronization did not finish: the cached events stay, and the next check tries
        // again. One synchronization ran, so nothing else comes.
        assert_eq!(next_events(&mut sync), ["1"]);
        assert!(!sync.synced.contains("1"));
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(sync.poll(), None);
    }

    /// Google, faked, with the account signed in and a client file for it.
    struct Google {
        fake: FakeGoogle,
        store: Arc<FakeStore>,
        dir: PathBuf,
        client_file: PathBuf,
    }

    impl Google {
        fn start(name: &str) -> Self {
            let fake = FakeGoogle::start(None, &[]);
            let store = Arc::new(FakeStore::default());
            let token = fake.signed_in(ACCOUNT_ID);
            store
                .saved
                .lock()
                .unwrap()
                .push((ACCOUNT_ID.to_string(), token));
            let dir = std::env::temp_dir().join(format!(
                "calendar-buddy-app-sync-{name}-{}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            let client_file = dir.join(CLIENT_FILE);
            std::fs::write(
                &client_file,
                format!(
                    r#"{{"installed":{{"client_id":"{CLIENT_ID}","client_secret":"test-secret"}}}}"#
                ),
            )
            .unwrap();
            Google {
                fake,
                store,
                dir,
                client_file,
            }
        }

        fn sign_in(&self) -> SignIn {
            let (client_file, endpoints, store) = (
                self.client_file.clone(),
                self.fake.endpoints.clone(),
                self.store.clone(),
            );
            Box::new(move |id| {
                Session::sign_in(&client_file, &endpoints, store.as_ref(), id)
                    .map(|session| Box::new(session) as Box<dyn Provider + Send>)
            })
        }

        /// A calendar of the account, with the access Google gives it.
        fn calendar(&self, id: &str, role: &str) {
            self.fake.add_calendar(
                ACCOUNT_ID,
                json!({
                    "id": id,
                    "summary": id,
                    "timeZone": "Europe/Warsaw",
                    "backgroundColor": "#9fe1e7",
                    "accessRole": role,
                }),
            );
        }

        /// The event, added or changed in Google, on Tuesday 10 November 2026 at 9:00.
        fn event(&self, calendar: &str, id: &str, title: &str) {
            self.fake.add_event(ACCOUNT_ID, calendar, event(id, title));
        }
    }

    impl Drop for Google {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn event(id: &str, title: &str) -> Value {
        json!({
            "id": id,
            "status": "confirmed",
            "summary": title,
            "start": {"dateTime": "2026-11-10T09:00:00+01:00", "timeZone": "Europe/Warsaw"},
            "end": {"dateTime": "2026-11-10T10:00:00+01:00", "timeZone": "Europe/Warsaw"},
        })
    }

    /// The titles the views are told to show, sorted; `None` when they are told nothing.
    fn told(updates: Vec<Update>) -> Option<Vec<String>> {
        let mut titles = None;
        for update in updates {
            if let Update::Events(_, events) = update {
                let mut shown: Vec<String> = events.into_iter().map(|e| e.title).collect();
                shown.sort();
                titles = Some(shown);
            }
        }
        titles
    }

    /// The account's first synchronization of the run, done at `start`.
    fn first_sync(google: &Google, start: Instant) -> Engine {
        first_sync_with(google, Cache::in_memory(), Disabled::new(), start)
    }

    /// The account's first synchronization of the run, done at `start` on the cache, with the
    /// calendars disabled in the settings.
    fn first_sync_with(
        google: &Google,
        cache: Cache,
        disabled: Disabled,
        start: Instant,
    ) -> Engine {
        let mut engine = Engine::new(cache, google.sign_in(), window(), disabled);
        engine.clear_disabled();
        let updates = engine.handle(Some(Job::Sync(ACCOUNT_ID.into(), window())), start);
        assert!(told(updates).is_some());
        engine
    }

    /// The names of the calendars the user is told are gone; empty when none.
    fn gone_told(updates: &[Update]) -> Vec<String> {
        updates
            .iter()
            .flat_map(|update| match update {
                Update::Gone(names) => names.clone(),
                _ => Vec::new(),
            })
            .collect()
    }

    /// The account's calendars in the cache, by ID.
    fn listed(engine: &Engine) -> Vec<String> {
        engine
            .cache
            .calendar_list()
            .unwrap()
            .into_iter()
            .map(|calendar| calendar.id)
            .collect()
    }

    /// Whether the app asked Google for the calendar's events.
    fn fetched(google: &Google, calendar: &str) -> bool {
        google
            .fake
            .requests()
            .iter()
            .any(|request| request.contains(&format!("/calendars/{calendar}/events")))
    }

    #[test]
    fn tc_02_06_an_unshared_calendar_goes_with_its_events_and_the_user_is_told_once() {
        let google = Google::start("tc-02-06");
        google.calendar("work", "owner");
        google.calendar("Colleague", "reader");
        google.event("Colleague", "review", "Review");
        google.event("Colleague", "lunch", "Lunch");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        assert_eq!(told(vec![engine.shown()]).unwrap(), ["Lunch", "Review"]);

        // 1. The colleague stops sharing it; the next round lists the account's calendars.
        google.fake.remove_calendar(ACCOUNT_ID, "Colleague");
        let updates = engine.handle(None, start + secs(60));

        assert_eq!(gone_told(&updates), ["Colleague"]);
        assert_eq!(told(updates).unwrap(), Vec::<String>::new());
        assert_eq!(listed(&engine), ["work"]);
        assert!(engine.cache.calendars().unwrap()[0].events.is_empty());

        // 2. One more round: no second notice.
        let updates = engine.handle(None, start + secs(120));
        assert_eq!(gone_told(&updates), Vec::<String>::new());
        let updates = engine.handle(Some(Job::Refresh), start + secs(120));
        assert_eq!(gone_told(&updates), Vec::<String>::new());
    }

    #[test]
    fn a_calendar_created_in_google_appears_at_the_next_round_and_one_deleted_disappears() {
        let google = Google::start("created");
        google.calendar("work", "owner");
        google.event("work", "standup", "Standup");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);

        google.calendar("team", "writer");
        google.event("team", "offsite", "Offsite");
        assert_eq!(
            told(engine.handle(None, start + secs(60))).unwrap(),
            ["Offsite", "Standup"]
        );
        assert_eq!(listed(&engine), ["work", "team"]);
        // From then on it is polled as an editable calendar.
        google.event("team", "offsite", "Offsite 2");
        assert_eq!(
            told(engine.handle(None, start + secs(120))).unwrap(),
            ["Offsite 2", "Standup"]
        );

        google.fake.remove_calendar(ACCOUNT_ID, "team");
        let updates = engine.handle(None, start + secs(180));

        assert_eq!(gone_told(&updates), ["team"]);
        assert_eq!(told(updates).unwrap(), ["Standup"]);
        assert_eq!(listed(&engine), ["work"]);
        assert_eq!(engine.schedule.all().len(), 1);
    }

    #[test]
    fn a_calendar_subscribed_from_a_webcal_link_in_google_appears_at_the_next_round() {
        let google = Google::start("subscribed");
        google.calendar("work", "owner");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);

        // Added in Google Calendar from its address (`MOD-07-F18`): a calendar the account reads.
        google.calendar("holidays", "reader");
        google.event("holidays", "new-year", "New Year");
        assert_eq!(
            told(engine.handle(None, start + secs(60))).unwrap(),
            ["New Year"]
        );
        assert_eq!(listed(&engine), ["work", "holidays"]);
    }

    #[test]
    fn a_calendar_list_google_does_not_send_leaves_the_calendars_as_they_were() {
        let google = Google::start("list-failing");
        google.calendar("work", "owner");
        google.calendar("team", "writer");
        google.event("work", "standup", "Standup");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);

        google.fake.fail_calendar_list();
        google.event("work", "standup", "Standup 2");
        let updates = engine.handle(None, start + secs(60));

        assert_eq!(gone_told(&updates), Vec::<String>::new());
        assert_eq!(told(updates).unwrap(), ["Standup 2"]);
        assert_eq!(listed(&engine), ["work", "team"]);
        assert_eq!(engine.schedule.all().len(), 2);
    }

    #[test]
    fn a_calendar_deleted_while_the_app_was_closed_goes_with_a_notice_at_the_next_run() {
        let google = Google::start("offline");
        google.calendar("work", "owner");
        google.calendar("trips", "owner");
        google.event("trips", "rome", "Rome");
        let engine = first_sync(&google, Instant::now());
        let cache = engine.cache;

        google.fake.remove_calendar(ACCOUNT_ID, "trips");
        let mut engine = Engine::new(cache, google.sign_in(), window(), Disabled::new());
        assert_eq!(told(vec![engine.shown()]).unwrap(), ["Rome"]);
        let updates = engine.handle(Some(Job::Sync(ACCOUNT_ID.into(), window())), Instant::now());

        assert_eq!(gone_told(&updates), ["trips"]);
        assert_eq!(told(updates).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn a_disabled_calendar_stays_listed_but_its_events_are_neither_fetched_nor_cached() {
        let google = Google::start("disabled");
        google.calendar("work", "owner");
        google.calendar("holidays", "reader");
        google.event("work", "standup", "Standup");
        google.event("holidays", "day-off", "Day off");
        let start = Instant::now();
        let disabled = Disabled::from([(ACCOUNT_ID.to_string(), "holidays".to_string())]);

        let mut engine = first_sync_with(&google, Cache::in_memory(), disabled, start);
        engine.handle(Some(Job::Refresh), start + secs(60));
        engine.handle(None, start + secs(30 * 60));

        assert_eq!(told(vec![engine.shown()]).unwrap(), ["Standup"]);
        assert_eq!(listed(&engine), ["work", "holidays"]);
        assert!(!fetched(&google, "holidays"));
        assert_eq!(engine.schedule.all().len(), 1);
    }

    #[test]
    fn enabling_a_disabled_calendar_synchronizes_it_at_once_and_disabling_drops_its_events() {
        let google = Google::start("enabled");
        google.calendar("work", "owner");
        google.calendar("holidays", "reader");
        google.event("work", "standup", "Standup");
        google.event("holidays", "day-off", "Day off");
        let start = Instant::now();
        let holidays = (ACCOUNT_ID.to_string(), "holidays".to_string());
        let mut engine = first_sync_with(
            &google,
            Cache::in_memory(),
            Disabled::from([holidays.clone()]),
            start,
        );

        let updates = engine.handle(Some(Job::Disabled(Disabled::new())), start + secs(5));

        assert_eq!(told(updates).unwrap(), ["Day off", "Standup"]);
        assert!(fetched(&google, "holidays"));
        assert_eq!(engine.schedule.all().len(), 2);
        assert_eq!(engine.schedule.next(), Some(start + secs(60)));

        let updates = engine.handle(
            Some(Job::Disabled(Disabled::from([holidays]))),
            start + secs(10),
        );

        assert_eq!(told(updates).unwrap(), ["Standup"]);
        assert_eq!(listed(&engine), ["work", "holidays"]);
        assert!(engine.cache.calendars().unwrap()[1].events.is_empty());
        assert_eq!(engine.schedule.all().len(), 1);
    }

    #[test]
    fn events_cached_for_a_calendar_disabled_since_are_deleted_at_start() {
        let ctx = egui::Context::default();
        let disabled = Disabled::from([("1".to_string(), "1@example.com".to_string())]);
        let ids = vec!["1".to_string(), "2".to_string()];

        let mut sync = Synchronization::with(
            &ctx,
            cache_of(&["1", "2"]),
            unreachable_google(),
            ids,
            window(),
            disabled,
        );

        assert_eq!(next_events(&mut sync), ["2"]);
    }

    #[test]
    fn the_app_takes_each_gone_calendar_once() {
        let google = Google::start("take-gone");
        google.calendar("work", "owner");
        google.calendar("trips", "owner");
        let engine = first_sync(&google, Instant::now());
        let cache = std::sync::Mutex::new(Some(engine.cache));
        google.fake.remove_calendar(ACCOUNT_ID, "trips");
        let ctx = egui::Context::default();
        let ids = vec![ACCOUNT_ID.to_string()];
        let open = move || cache.lock().unwrap().take().unwrap();
        let mut sync =
            Synchronization::with(&ctx, open, google.sign_in(), ids, window(), Disabled::new());
        next_events(&mut sync);

        sync.checked(&[(ACCOUNT_ID.to_string(), Check::Works)], window());
        next_events(&mut sync);

        assert_eq!(sync.take_gone(), ["trips"]);
        assert_eq!(sync.take_gone(), Vec::<String>::new());
    }

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    #[test]
    fn tc_02_04_a_change_in_google_appears_after_the_next_round_and_at_once_on_refresh() {
        let google = Google::start("tc-02-04");
        google.calendar("work", "owner");
        google.event("work", "standup", "Standup");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        assert_eq!(told(vec![engine.shown()]).unwrap(), ["Standup"]);

        // 1. Renamed in Google; a minute later the background round brings it.
        google.event("work", "standup", "Standup 2");
        assert_eq!(told(engine.handle(None, start + secs(59))), None);
        assert_eq!(engine.schedule.next(), Some(start + secs(60)));
        assert_eq!(
            told(engine.handle(None, start + secs(60))).unwrap(),
            ["Standup 2"]
        );

        // 2. Renamed again; "Refresh" brings it without the clock moving.
        google.event("work", "standup", "Standup 3");
        assert_eq!(told(engine.handle(None, start + secs(60))), None);
        assert_eq!(
            told(engine.handle(Some(Job::Refresh), start + secs(60))).unwrap(),
            ["Standup 3"]
        );
    }

    #[test]
    fn a_read_only_calendar_is_refreshed_after_15_minutes() {
        let google = Google::start("read-only");
        google.calendar("work", "writer");
        google.calendar("holidays", "reader");
        google.event("holidays", "day-off", "Day off");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);

        google.event("holidays", "day-off", "Day off moved");
        for minute in 1..15 {
            assert_eq!(told(engine.handle(None, start + secs(minute * 60))), None);
        }

        assert_eq!(
            told(engine.handle(None, start + secs(15 * 60))).unwrap(),
            ["Day off moved"]
        );
    }

    #[test]
    fn with_the_window_hidden_an_editable_calendar_is_refreshed_within_5_minutes() {
        let google = Google::start("hidden");
        google.calendar("work", "owner");
        google.event("work", "standup", "Standup");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        engine.handle(Some(Job::Shown(false)), start);

        google.event("work", "standup", "Standup 2");
        assert_eq!(engine.schedule.next(), Some(start + secs(5 * 60)));
        assert_eq!(told(engine.handle(None, start + secs(60))), None);

        assert_eq!(
            told(engine.handle(None, start + secs(5 * 60))).unwrap(),
            ["Standup 2"]
        );
    }

    #[test]
    fn activating_the_window_and_opening_an_event_synchronize_at_once() {
        let google = Google::start("activated");
        google.calendar("work", "owner");
        google.calendar("holidays", "reader");
        google.event("work", "standup", "Standup");
        google.event("holidays", "day-off", "Day off");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);

        google.event("holidays", "day-off", "Day off 2");
        assert_eq!(
            told(engine.handle(Some(Job::Activated), start + secs(20))).unwrap(),
            ["Day off 2", "Standup"]
        );

        // Opening an event synchronizes its calendar alone.
        google.event("work", "standup", "Standup 2");
        google.event("holidays", "day-off", "Day off 3");
        assert_eq!(
            told(engine.handle(Some(Job::Opening("work".into())), start + secs(25))).unwrap(),
            ["Day off 2", "Standup 2"]
        );
    }

    #[test]
    fn refresh_tries_again_an_account_whose_first_synchronization_did_not_get_through() {
        let google = Google::start("pending");
        google.calendar("work", "owner");
        google.event("work", "standup", "Standup");
        let token = google.store.saved.lock().unwrap().remove(0);
        let start = Instant::now();
        let mut engine = Engine::new(
            Cache::in_memory(),
            google.sign_in(),
            window(),
            Disabled::new(),
        );
        let updates = engine.handle(Some(Job::Sync(ACCOUNT_ID.into(), window())), start);
        assert!(matches!(updates[0], Update::Failed(_)));

        google.store.saved.lock().unwrap().push(token);

        assert_eq!(
            told(engine.handle(Some(Job::Refresh), start)).unwrap(),
            ["Standup"]
        );
        assert_eq!(engine.schedule.next(), Some(start + secs(60)));
    }

    #[test]
    fn the_views_are_shown_the_events_of_a_moved_window() {
        let google = Google::start("moved");
        google.calendar("work", "owner");
        google.event("work", "standup", "Standup");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        // A year and a half later, Tuesday 10 November 2026 is out of the window.
        let later = Window::around(d(2028, 5, 11), Warsaw);

        assert_eq!(
            told(engine.handle(Some(Job::Window(later)), start)).unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(engine.window, later);
    }

    #[test]
    fn an_account_google_turned_away_is_no_longer_polled() {
        let google = Google::start("stopped");
        google.calendar("work", "owner");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);

        engine.handle(Some(Job::Stop(ACCOUNT_ID.into())), start);

        assert_eq!(engine.schedule.next(), None);
        assert!(engine.sessions.is_empty());
    }

    /// The event, in Google, on Sunday 10 March 2030 at 9:00: past the cache window.
    fn far_event(google: &Google, calendar: &str, id: &str, title: &str) {
        google.fake.add_event(
            ACCOUNT_ID,
            calendar,
            json!({
                "id": id,
                "status": "confirmed",
                "summary": title,
                "start": {"dateTime": "2030-03-10T09:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2030-03-10T10:00:00+01:00", "timeZone": "Europe/Warsaw"},
            }),
        );
    }

    /// March 2030, past the cache window.
    fn march_2030() -> Window {
        window().outside(d(2030, 3, 1), d(2030, 3, 31)).unwrap()
    }

    #[test]
    fn online_a_period_outside_the_cache_window_is_fetched_when_browsed_and_not_kept() {
        let google = Google::start("outside");
        google.calendar("work", "owner");
        google.calendar("holidays", "reader");
        google.event("work", "standup", "Standup");
        far_event(&google, "work", "trip", "Trip");
        far_event(&google, "holidays", "spring", "Spring day");
        let start = Instant::now();
        let disabled = Disabled::from([(ACCOUNT_ID.to_string(), "holidays".to_string())]);
        let mut engine = first_sync_with(&google, Cache::in_memory(), disabled, start);
        assert_eq!(told(vec![engine.shown()]).unwrap(), ["Standup"]);

        let updates = engine.handle(Some(Job::Browse(Some(march_2030()))), start);

        // A disabled calendar is not fetched for it either.
        assert_eq!(told(updates).unwrap(), ["Standup", "Trip"]);
        let cached = engine.cache.calendars().unwrap();
        assert!(
            cached
                .iter()
                .all(|c| c.events.iter().all(|e| e.id != "trip"))
        );
        assert!(!fetched(&google, "holidays"));

        // Browsing away, it is no longer shown.
        let updates = engine.handle(Some(Job::Browse(None)), start);
        assert_eq!(told(updates).unwrap(), ["Standup"]);
    }

    #[test]
    fn a_period_browsed_before_the_first_synchronization_is_fetched_once_signed_in() {
        let google = Google::start("outside-first");
        google.calendar("work", "owner");
        far_event(&google, "work", "trip", "Trip");
        let mut engine = Engine::new(
            Cache::in_memory(),
            google.sign_in(),
            window(),
            Disabled::new(),
        );
        let start = Instant::now();

        assert_eq!(
            told(engine.handle(Some(Job::Browse(Some(march_2030()))), start)).unwrap(),
            Vec::<String>::new()
        );
        let updates = engine.handle(Some(Job::Sync(ACCOUNT_ID.into(), window())), start);

        assert_eq!(told(updates).unwrap(), ["Trip"]);
    }

    #[test]
    fn refresh_fetches_the_browsed_period_again() {
        let google = Google::start("outside-refresh");
        google.calendar("work", "owner");
        far_event(&google, "work", "trip", "Trip");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        engine.handle(Some(Job::Browse(Some(march_2030()))), start);

        far_event(&google, "work", "trip", "Trip moved");

        assert_eq!(
            told(engine.handle(Some(Job::Refresh), start)).unwrap(),
            ["Trip moved"]
        );
    }

    #[test]
    fn an_occurrence_moved_out_of_the_browsed_days_is_not_shown_at_its_old_time() {
        let google = Google::start("outside-moved");
        google.calendar("work", "owner");
        // Every Tuesday at 9:00; the one of 12 March 2030 moved to Monday 18 March.
        google.fake.add_event(
            ACCOUNT_ID,
            "work",
            json!({
                "id": "weekly",
                "status": "confirmed",
                "summary": "Weekly",
                "start": {"dateTime": "2030-03-05T09:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2030-03-05T10:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "recurrence": ["RRULE:FREQ=WEEKLY;BYDAY=TU;COUNT=4"],
            }),
        );
        google.fake.add_event(
            ACCOUNT_ID,
            "work",
            json!({
                "id": "weekly_20300312T080000Z",
                "status": "confirmed",
                "summary": "Weekly",
                "recurringEventId": "weekly",
                "originalStartTime": {"dateTime": "2030-03-12T09:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "start": {"dateTime": "2030-03-18T09:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2030-03-18T10:00:00+01:00", "timeZone": "Europe/Warsaw"},
            }),
        );
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let week = window().outside(d(2030, 3, 11), d(2030, 3, 17)).unwrap();

        assert_eq!(
            told(engine.handle(Some(Job::Browse(Some(week))), start)).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn of_the_browsing_and_the_guest_searches_waiting_only_the_last_of_each_is_done() {
        let week = |day| {
            Some(
                window()
                    .outside(d(2030, 3, day), d(2030, 3, day + 6))
                    .unwrap(),
            )
        };
        let typed = |query: &str| Some(Job::Suggest("1".into(), query.into(), Reach::default()));
        let jobs = vec![
            Some(Job::Browse(week(4))),
            typed("a"),
            None,
            Some(Job::Browse(week(11))),
            typed("ad"),
            Some(Job::Refresh),
            Some(Job::Browse(week(18))),
            typed("ada"),
        ];

        let done = latest_only(jobs);

        assert_eq!(done.len(), 4);
        assert!(done[0].is_none());
        assert!(matches!(done[1], Some(Job::Refresh)));
        assert!(matches!(&done[2], Some(Job::Browse(span)) if *span == week(18)));
        assert!(matches!(&done[3], Some(Job::Suggest(_, query, _)) if query == "ada"));
    }

    #[test]
    fn of_the_guests_busy_times_waiting_only_the_last_is_asked() {
        let day = calendar_core::week::day_span(d(2026, 11, 13), Warsaw);
        let asked = |email: &str| {
            Some(Job::Busy(Asked {
                account_id: "1".into(),
                day,
                emails: vec![email.into()],
            }))
        };
        let jobs = vec![
            asked("anna@example.com"),
            Some(Job::Refresh),
            asked("adam@example.com"),
        ];

        let done = latest_only(jobs);

        assert_eq!(done.len(), 2);
        assert!(matches!(done[0], Some(Job::Refresh)));
        assert!(matches!(&done[1], Some(Job::Busy(asked)) if asked.emails == ["adam@example.com"]));
    }

    #[test]
    fn an_event_crossing_the_window_edge_is_shown_once() {
        let google = Google::start("outside-edge");
        google.calendar("work", "owner");
        google.fake.add_event(
            ACCOUNT_ID,
            "work",
            json!({
                "id": "holiday",
                "status": "confirmed",
                "summary": "Holiday",
                "start": {"date": "2028-11-09"},
                "end": {"date": "2028-11-14"},
            }),
        );
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let edge = window().outside(d(2028, 11, 6), d(2028, 11, 12)).unwrap();

        assert_eq!(
            told(engine.handle(Some(Job::Browse(Some(edge))), start)).unwrap(),
            ["Holiday"]
        );
    }

    /// 14:05 in Warsaw on Wednesday 11 November 2026, plus `minutes`.
    fn clock_at(minutes: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 11, 13, 5, 0).unwrap() + chrono::Duration::minutes(minutes)
    }

    /// The calendars not up to date, by ID, and why.
    fn problems(engine: &Engine) -> Vec<(String, Problem)> {
        engine
            .status()
            .problems
            .into_iter()
            .map(|(calendar, problem)| (calendar.id, problem))
            .collect()
    }

    #[test]
    fn tc_02_08_an_error_on_one_calendar_is_shown_on_it_while_the_others_stay_up_to_date() {
        let google = Google::start("tc-02-08");
        google.calendar("Work", "owner");
        google.calendar("Home", "owner");
        google.event("Work", "standup", "Standup");
        google.event("Home", "dinner", "Dinner");
        let start = Instant::now();
        let mut engine = Engine::new(
            Cache::in_memory(),
            google.sign_in(),
            window(),
            Disabled::new(),
        );
        engine.clock = Box::new(|| clock_at(0));
        engine.handle(Some(Job::Sync(ACCOUNT_ID.into(), window())), start);
        assert_eq!(engine.status().last_synced, Some(clock_at(0)));
        assert_eq!(engine.status().state(true), State::UpToDate);

        // 1. Every request for "Work" fails, "Home" changes, and the next round runs.
        google.fake.fail_calendar("Work");
        google.event("Home", "dinner", "Dinner moved");
        engine.clock = Box::new(|| clock_at(1));
        let updates = engine.handle(None, start + secs(60));

        assert_eq!(told(updates).unwrap(), ["Dinner moved", "Standup"]);
        assert_eq!(problems(&engine), [("Work".to_string(), Problem::Failed)]);
        let status = engine.status();
        assert_eq!(status.state(true), State::Error);
        assert_eq!(
            status
                .problem(&status.problems[0].0, true)
                .map(status::on_row),
            Some("Could not be synchronized; trying again.")
        );
        assert_eq!(status.last_synced, Some(clock_at(1)));
        let regional = Regional::from_env(|var| (var == "LANG").then(|| "en_GB.UTF-8".into()));
        assert_eq!(
            status::last_synced(status.last_synced, Warsaw, d(2026, 11, 11), &regional),
            "Last synchronized at 14:06"
        );
        // The time is kept for the next start.
        assert_eq!(engine.cache.last_synced().unwrap(), Some(clock_at(1)));
    }

    #[test]
    fn after_a_rate_limit_the_calendar_is_delayed_then_resumes_with_growing_waits_and_nothing_lost()
    {
        let google = Google::start("rate-limit");
        google.calendar("work", "owner");
        google.calendar("home", "owner");
        google.event("work", "standup", "Standup");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);

        google.fake.limit_calendar("work", "429 Too Many Requests");
        google.event("work", "standup", "Standup moved");
        google.event("home", "dinner", "Dinner");

        // The calendar Google did not limit goes on.
        let updates = engine.handle(None, start + secs(60));
        assert_eq!(told(updates).unwrap(), ["Dinner", "Standup"]);
        let mut now = start + secs(60);
        let mut waits = Vec::new();
        for _ in 0..3 {
            assert_eq!(problems(&engine), [("work".to_string(), Problem::Delayed)]);
            assert_eq!(engine.status().state(true), State::Delayed);
            assert!(label(engine.status().state(true)).contains("delayed"));
            let mut retry = now;
            while engine.schedule.due(retry).iter().all(|c| c.id != "work") {
                retry += secs(1);
            }
            waits.push((retry - now).as_secs());
            now = retry;
            engine.handle(None, now);
        }
        // The cached event stays meanwhile.
        assert_eq!(told(vec![engine.shown()]).unwrap(), ["Dinner", "Standup"]);
        assert!(
            waits[0] >= 60 && waits[1] >= 120 && waits[2] >= 240,
            "{waits:?}"
        );
        assert!(waits[0] < waits[1] && waits[1] < waits[2], "{waits:?}");

        google.fake.recover();
        while engine.schedule.due(now).iter().all(|c| c.id != "work") {
            now += secs(1);
        }
        let updates = engine.handle(None, now);

        assert_eq!(told(updates).unwrap(), ["Dinner", "Standup moved"]);
        assert_eq!(problems(&engine), []);
        assert_eq!(engine.status().state(true), State::UpToDate);
        assert_eq!(engine.schedule.next(), Some(now + secs(60)));
    }

    #[test]
    fn after_a_lost_connection_synchronization_resumes_on_its_own_and_nothing_is_lost() {
        let google = Google::start("lost-connection");
        google.calendar("work", "owner");
        google.event("work", "standup", "Standup");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);

        google.fake.disconnect();
        google.event("work", "standup", "Standup moved");
        google.event("work", "retro", "Retro");
        engine.handle(None, start + secs(60));

        assert_eq!(problems(&engine), [("work".to_string(), Problem::Failed)]);
        assert_eq!(told(vec![engine.shown()]).unwrap(), ["Standup"]);

        google.fake.recover();
        let retry = engine.schedule.next().unwrap();
        assert!(retry >= start + secs(120));
        let updates = engine.handle(None, retry);

        assert_eq!(told(updates).unwrap(), ["Retro", "Standup moved"]);
        assert_eq!(problems(&engine), []);
    }

    #[test]
    fn an_account_google_turned_away_keeps_its_calendars_shown_and_marked_not_synchronizing() {
        let google = Google::start("not-synchronizing");
        google.calendar("work", "owner");
        google.event("work", "standup", "Standup");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);

        engine.handle(Some(Job::Stop(ACCOUNT_ID.into())), start);

        assert_eq!(told(vec![engine.shown()]).unwrap(), ["Standup"]);
        assert_eq!(
            problems(&engine),
            [("work".to_string(), Problem::NotSynchronizing)]
        );
        let status = engine.status();
        assert_eq!(
            status
                .problem(&status.problems[0].0, false)
                .map(status::on_row),
            Some("Not synchronizing: its account needs sign-in.")
        );

        // Signed in again, it synchronizes as before.
        engine.handle(Some(Job::Sync(ACCOUNT_ID.into(), window())), start);
        assert_eq!(problems(&engine), []);
    }

    #[test]
    fn the_calendars_of_an_account_whose_first_synchronization_failed_are_in_error() {
        let mut engine = Engine::new(
            cache_of(&["1"])(),
            unreachable_google(),
            window(),
            Disabled::new(),
        );

        engine.handle(Some(Job::Sync("1".into(), window())), Instant::now());

        assert_eq!(
            problems(&engine),
            [("1@example.com".to_string(), Problem::Failed)]
        );
        assert_eq!(engine.status().state(true), State::Error);
        assert_eq!(engine.status().state(false), State::Offline);
        let status = engine.status();
        assert_eq!(status.problem(&status.problems[0].0, false), None);
    }

    #[test]
    fn until_an_account_first_synchronizes_its_calendars_are_not_up_to_date() {
        let mut engine = Engine::new(
            cache_of(&["1"])(),
            unreachable_google(),
            window(),
            Disabled::new(),
        );

        // Waiting for Google to confirm the account's sign-in.
        assert_eq!(engine.status().state(true), State::Synchronizing);
        assert_eq!(problems(&engine), []);

        // Google could not be asked.
        engine.handle(Some(Job::Unconfirmed("1".into())), Instant::now());

        assert_eq!(engine.status().state(true), State::Error);
        assert_eq!(
            problems(&engine),
            [("1@example.com".to_string(), Problem::Failed)]
        );
    }

    #[test]
    fn a_check_that_did_not_get_through_marks_only_accounts_not_synchronized_yet() {
        let ctx = egui::Context::default();
        let mut sync = Synchronization::with(
            &ctx,
            cache_of(&["1"]),
            unreachable_google(),
            vec!["1".to_string()],
            window(),
            Disabled::new(),
        );
        next_events(&mut sync);
        sync.synced.insert("2".to_string());

        sync.checked(
            &[
                ("1".to_string(), Check::Failed("timeout".into())),
                ("2".to_string(), Check::Failed("timeout".into())),
            ],
            window(),
        );

        let deadline = Instant::now() + Duration::from_secs(10);
        while sync.status().problems.is_empty() {
            assert!(Instant::now() < deadline, "no problem was told");
            std::thread::sleep(Duration::from_millis(5));
            sync.poll();
        }
        assert_eq!(sync.status().problems[0].1, Problem::Failed);
    }

    #[test]
    fn an_account_turned_away_in_an_earlier_run_is_marked_not_synchronizing_at_start() {
        let ctx = egui::Context::default();
        let mut sync = Synchronization::with(
            &ctx,
            cache_of(&["1"]),
            unreachable_google(),
            vec!["1".to_string()],
            window(),
            Disabled::new(),
        );

        // What the app does at start for an account saved as needing sign-in.
        sync.checked(&[("1".to_string(), Check::SignInNeeded)], window());

        assert_eq!(next_events(&mut sync), ["1"]);
        let deadline = Instant::now() + Duration::from_secs(10);
        while sync.status().problems.is_empty() || sync.status().synchronizing {
            assert!(Instant::now() < deadline, "no problem was told");
            std::thread::sleep(Duration::from_millis(5));
            sync.poll();
        }
        assert_eq!(sync.status().problems[0].1, Problem::NotSynchronizing);
        assert!(!sync.status().waiting);
        assert_eq!(sync.status().state(true), State::Error);
    }

    #[test]
    fn the_time_of_the_last_synchronization_is_shown_from_the_cache_at_start() {
        let mut cache = cache_of(&["1"])();
        cache.set_last_synced(clock_at(0)).unwrap();
        let ctx = egui::Context::default();

        let mut sync = Synchronization::with(
            &ctx,
            move || cache,
            unreachable_google(),
            vec!["1".to_string()],
            window(),
            Disabled::new(),
        );

        next_events(&mut sync);
        let deadline = Instant::now() + Duration::from_secs(10);
        while sync.status().last_synced.is_none() {
            assert!(Instant::now() < deadline, "no status came");
            std::thread::sleep(Duration::from_millis(5));
            sync.poll();
        }
        assert_eq!(sync.status().last_synced, Some(clock_at(0)));
        assert_eq!(sync.status().state(false), State::Offline);
    }

    #[test]
    fn the_views_keep_working_while_a_synchronization_runs() {
        let ctx = egui::Context::default();
        let (release, released) = mpsc::channel::<()>();
        let released = Mutex::new(released);
        // Signing in waits until the test lets it go on.
        let sign_in: SignIn = Box::new(move |_| {
            let _ = released.lock().unwrap().recv();
            Err(SyncError("Google could not be reached".into()))
        });
        let mut sync = Synchronization::with(
            &ctx,
            cache_of(&["1"]),
            sign_in,
            vec!["1".to_string()],
            window(),
            Disabled::new(),
        );
        assert_eq!(next_events(&mut sync), ["1"]);

        sync.checked(&[("1".to_string(), Check::Works)], window());
        let deadline = Instant::now() + Duration::from_secs(10);
        while !sync.status().synchronizing {
            assert!(
                Instant::now() < deadline,
                "the synchronization did not start"
            );
            std::thread::sleep(Duration::from_millis(5));
            // Each frame's calls come back at once while the worker waits for Google.
            let frame = Instant::now();
            sync.poll();
            sync.browse(None);
            sync.set_window(window());
            assert!(frame.elapsed() < Duration::from_millis(50));
        }
        assert_eq!(sync.status().state(true), State::Synchronizing);

        release.send(()).unwrap();
        while sync.status().synchronizing {
            assert!(Instant::now() < deadline, "the synchronization did not end");
            std::thread::sleep(Duration::from_millis(5));
            sync.poll();
        }
        assert_eq!(sync.status().state(true), State::Error);
    }

    /// The system's wake-ups, faked: the test says when the computer woke.
    #[derive(Default)]
    struct Woke(Mutex<Vec<Box<dyn Fn() + Send + Sync>>>);

    impl Wakeups for Woke {
        fn watch(&self, woke: Box<dyn Fn() + Send + Sync>) {
            self.0.lock().unwrap().push(woke);
        }
    }

    #[test]
    fn waking_from_sleep_or_the_network_returning_synchronizes_at_once() {
        let google = Google::start("woke");
        google.calendar("work", "owner");
        google.event("work", "standup", "Standup");
        let ctx = egui::Context::default();
        let ids = vec![ACCOUNT_ID.to_string()];
        let open = || Cache::in_memory();
        let mut sync =
            Synchronization::with(&ctx, open, google.sign_in(), ids, window(), Disabled::new());
        let woke = Woke::default();
        sync.wake_on(&woke);
        assert_eq!(next_events(&mut sync), Vec::<String>::new());
        sync.checked(&[(ACCOUNT_ID.to_string(), Check::Works)], window());
        assert_eq!(next_events(&mut sync), ["Standup"]);

        google.event("work", "standup", "Standup 2");
        for woke in woke.0.lock().unwrap().iter() {
            woke();
        }

        assert_eq!(next_events(&mut sync), ["Standup 2"]);
    }

    #[test]
    fn the_views_of_a_hidden_calendar_point_at_the_events_still_shown() {
        let mut team =
            calendar_core::fixtures::timed("Team sync", d(2026, 11, 10), (9, 0), (10, 0));
        team.calendar_id = "team".to_string();
        let events = vec![
            team,
            calendar_core::fixtures::timed("Standup", d(2026, 11, 11), (9, 0), (10, 0)),
        ];
        let mut settings = Settings::default();
        settings
            .hidden
            .insert(("1".to_string(), "team".to_string()));
        let navigation = Navigation::start(None, d(2026, 11, 10), chrono::Weekday::Mon);
        let now = Utc.with_ymd_and_hms(2026, 11, 10, 8, 0, 0).unwrap();

        let (Built::Grid(week), places) = build(&events, &settings, navigation, 30, Warsaw, now)
        else {
            panic!("the week view is built");
        };

        // Opening or dragging a block reaches the event it shows.
        let opened: Vec<&str> = week
            .blocks
            .iter()
            .map(|block| events[places[block.placement.event]].title.as_str())
            .collect();
        assert_eq!(opened, ["Standup"]);
    }

    #[test]
    fn tc_02_09_a_local_colour_is_shown_without_reaching_google_and_reset_brings_googles_back() {
        let google = Google::start("tc-02-09");
        let blue = Rgb(0x03, 0x9b, 0xe5);
        google.fake.add_calendar(
            ACCOUNT_ID,
            json!({
                "id": "Work",
                "summary": "Work",
                "timeZone": "Europe/Warsaw",
                "backgroundColor": "#039be5",
                "accessRole": "owner",
            }),
        );
        google.event("Work", "standup", "Standup");
        let engine = first_sync(&google, Instant::now());
        let Update::Events(calendars, events) = engine.shown() else {
            panic!("the views are told what to show");
        };
        // The calendar in the side panel and its event in the week of 10 November.
        let shown = |settings: &Settings| {
            let navigation = Navigation::start(None, d(2026, 11, 10), chrono::Weekday::Mon);
            let now = Utc.with_ymd_and_hms(2026, 11, 10, 8, 0, 0).unwrap();
            let (Built::Grid(week), _) = build(&events, settings, navigation, 30, Warsaw, now)
            else {
                panic!("the week view is built");
            };
            let mut colors = vec![colors::calendar_color(&calendars[0], &settings.colors)];
            colors.extend(week.blocks.iter().map(|block| block.color));
            colors
        };
        let picked = |color| Picked {
            account_id: ACCOUNT_ID.to_string(),
            calendar_id: "Work".to_string(),
            color,
        };
        let mut settings = Settings::default();
        assert_eq!(shown(&settings), [blue, blue]);

        // 1. A local colour of green, picked from the calendar's menu, kept in the settings.
        let (_, green) = colors::CALENDAR_COLORS[3];
        pick(&mut settings.colors, picked(Some(green)));
        settings = Settings::parse(&settings.to_text());
        assert_eq!(shown(&settings), [green, green]);

        // 2. Google was only ever asked to read; the calendar's colour was not changed there.
        let requests = google.fake.requests();
        let calendar_api: Vec<&String> = requests
            .iter()
            .filter(|request| request.contains(" /calendar/v3/"))
            .collect();
        assert!(!calendar_api.is_empty());
        assert!(
            calendar_api
                .iter()
                .all(|request| request.starts_with("GET "))
        );

        // 3. Reset.
        pick(&mut settings.colors, picked(None));
        assert_eq!(shown(&settings), [blue, blue]);
    }

    /// The account's primary calendar, "Personal", which new events go to until the user
    /// chooses another one.
    fn personal(google: &Google) {
        google.fake.add_calendar(
            ACCOUNT_ID,
            json!({
                "id": EMAIL,
                "summary": "Personal",
                "timeZone": "Europe/Warsaw",
                "backgroundColor": "#9fe1e7",
                "accessRole": "owner",
                "primary": true,
            }),
        );
    }

    fn accounts() -> Vec<calendar_core::accounts::Entry> {
        vec![calendar_core::accounts::Entry {
            account: calendar_core::accounts::Account {
                id: ACCOUNT_ID.to_string(),
                email: EMAIL.to_string(),
                workspace: false,
                contacts: true,
                directory: false,
            },
            state: calendar_core::accounts::AccountState::Connected,
        }]
    }

    /// What Google answered about new events, in order.
    fn saved(updates: &[Update]) -> Vec<Saved> {
        updates
            .iter()
            .filter_map(|update| match update {
                Update::Created(local, id) => Some(Saved::Created(*local, id.clone())),
                Update::NotCreated(local, reason) => {
                    Some(Saved::NotCreated(*local, reason.clone()))
                }
                Update::WithoutMeet(local) => Some(Saved::WithoutMeet(*local)),
                _ => None,
            })
            .collect()
    }

    /// The events last told to the views.
    fn shown(updates: &[Update]) -> Option<Vec<Event>> {
        updates.iter().rev().find_map(|update| match update {
            Update::Events(_, events) => Some(events.clone()),
            _ => None,
        })
    }

    fn local(at: DateTime<Utc>) -> String {
        at.with_timezone(&Warsaw).format("%F %R").to_string()
    }

    #[test]
    fn tc_05_01_selecting_10_to_11_30_and_typing_a_title_creates_the_event_in_the_default_calendar()
    {
        let google = Google::start("tc-05-01");
        personal(&google);
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let calendars = engine.cache.calendar_list().unwrap();

        // 1. Thursday 12 November 10:00–11:30 is selected in the week view of 9–15 November:
        //    the quick-entry box has those times and the calendar "Personal".
        let selected = create::selected(d(2026, 11, 12), 600, 690, Warsaw);
        let default = create::default_calendar(&accounts(), &calendars, &Settings::default());
        let mut entry = QuickEntry::new(selected, Warsaw, default);
        assert_eq!(
            (local(entry.start), local(entry.end)),
            (
                "2026-11-12 10:00".to_string(),
                "2026-11-12 11:30".to_string()
            )
        );
        assert_eq!(entry.calendar.as_ref().unwrap().name, "Personal");

        // 2. "Planning" is typed and Enter pressed: it is shown at once, marked saving; the fake
        //    Google receives it in "Personal", and the app shows it as synchronized back.
        entry.title = "Planning".to_string();
        let (calendar, event) = entry.new_event().unwrap();
        let mut saving = Saving::default();
        let number = saving.add(&calendar, &event);
        assert_eq!(
            saving.events().map(|e| e.saving).collect::<Vec<_>>(),
            [true]
        );
        let updates = engine.handle(Some(Job::Create(number, calendar, event)), start);

        let saved = saved(&updates);
        let [Saved::Created(confirmed, id)] = saved.as_slice() else {
            panic!("Google did not create the event");
        };
        assert_eq!(*confirmed, number);
        let sent = google.fake.event(ACCOUNT_ID, EMAIL, id).unwrap();
        assert_eq!(sent["summary"], "Planning");
        assert_eq!(sent["start"]["dateTime"], "2026-11-12T10:00:00+01:00");
        assert_eq!(sent["end"]["dateTime"], "2026-11-12T11:30:00+01:00");
        saving.confirmed(number, id);
        let events = shown(&updates).unwrap();
        saving.synced(&events);
        assert!(saving.is_empty());
        let planning = events.iter().find(|e| e.title == "Planning").unwrap();
        assert_eq!(planning.calendar_id, EMAIL);
        assert!(!planning.saving);
        let Timing::Timed { start, end, .. } = planning.timing else {
            panic!("a timed event");
        };
        assert_eq!(
            (local(start), local(end)),
            (
                "2026-11-12 10:00".to_string(),
                "2026-11-12 11:30".to_string()
            )
        );
    }

    #[test]
    fn a_new_event_google_refuses_is_not_created_and_the_reason_comes_back() {
        let google = Google::start("create-refused");
        personal(&google);
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        // Google stops letting the account change it before the app learns of it.
        google.fake.set_access_role(EMAIL, "reader");
        let (_, event) = QuickEntry {
            title: "Planning".to_string(),
            ..QuickEntry::new(
                create::selected(d(2026, 11, 12), 600, 690, Warsaw),
                Warsaw,
                Some(calendar.clone()),
            )
        }
        .new_event()
        .unwrap();

        let updates = engine.handle(Some(Job::Create(7, calendar.clone(), event.clone())), start);

        let refused = saved(&updates);
        let [Saved::NotCreated(7, reason)] = refused.as_slice() else {
            panic!("the event was not refused");
        };
        assert!(reason.starts_with("Google refused"), "{reason}");
        assert!(!reason.contains("Planning"));
        assert!(google.fake.event(ACCOUNT_ID, EMAIL, "created-1").is_none());

        // An account not signed in in this run creates nothing either.
        engine.stop(ACCOUNT_ID);
        let updates = engine.handle(Some(Job::Create(8, calendar, event)), start);
        assert!(matches!(
            saved(&updates).as_slice(),
            [Saved::NotCreated(8, _)]
        ));
    }

    #[test]
    fn a_new_event_on_browsed_days_outside_the_cache_comes_back_from_google() {
        let google = Google::start("create-outside");
        personal(&google);
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        engine.handle(Some(Job::Browse(Some(march_2030()))), start);
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let (_, event) = QuickEntry {
            title: "Trip".to_string(),
            ..QuickEntry::new(
                create::selected(d(2030, 3, 10), 600, 690, Warsaw),
                Warsaw,
                Some(calendar.clone()),
            )
        }
        .new_event()
        .unwrap();

        let updates = engine.handle(Some(Job::Create(1, calendar, event)), start);

        assert!(matches!(saved(&updates).as_slice(), [Saved::Created(1, _)]));
        let events = shown(&updates).unwrap();
        assert!(events.iter().any(|e| e.title == "Trip"));
    }

    #[test]
    fn the_default_length_of_new_events_is_read_from_google_with_the_first_synchronization() {
        let google = Google::start("default-length");
        personal(&google);
        google.fake.set_default_length(ACCOUNT_ID, 30);
        let ctx = egui::Context::default();
        let mut sync = Synchronization::with(
            &ctx,
            Cache::in_memory,
            google.sign_in(),
            vec![ACCOUNT_ID.to_string()],
            window(),
            Disabled::new(),
        );
        // Google's default until the account's own is known.
        assert_eq!(sync.default_length(ACCOUNT_ID), 60);

        sync.checked(&[(ACCOUNT_ID.to_string(), Check::Works)], window());

        let deadline = Instant::now() + Duration::from_secs(10);
        while sync.default_length(ACCOUNT_ID) != 30 {
            assert!(Instant::now() < deadline, "the default length did not come");
            std::thread::sleep(Duration::from_millis(5));
            sync.poll();
        }
        let (start, end) = create::clicked(d(2026, 11, 12), 600, 30, Warsaw);
        assert_eq!(end - start, chrono::Duration::minutes(30));
    }

    fn polish() -> Regional {
        Regional::from_env(|var| (var == "LANG").then(|| "pl_PL.UTF-8".into()))
    }

    /// The events the engine shows now.
    fn now_shown(engine: &Engine) -> Vec<Event> {
        let Update::Events(_, events) = engine.shown() else {
            panic!("events");
        };
        events
    }

    #[test]
    fn tc_05_02_the_full_editor_saves_title_description_times_location_free_and_private_to_google()
    {
        let google = Google::start("tc-05-02");
        personal(&google);
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let calendars = engine.cache.calendar_list().unwrap();
        let regional = polish();

        // 1. The full editor of a new event, opened from the quick-entry box, filled in.
        let default = create::default_calendar(&accounts(), &calendars, &Settings::default());
        let entry = QuickEntry::new(
            create::clicked(d(2026, 11, 13), 600, 60, Warsaw),
            Warsaw,
            default,
        );
        let mut editor = Editor::from_quick(&entry, &regional);
        editor.title = "Review".to_string();
        editor.description = "Q4 numbers".to_string();
        editor.start_date = "13.11.2026".to_string();
        editor.start_time = "14:00".to_string();
        editor.follow_start(&regional);
        editor.location = "Room 2".to_string();
        editor.show = Show::Free;
        editor.visibility = Visibility::Private;
        assert_eq!(
            (
                editor.title.as_str(),
                editor.description.as_str(),
                editor.start_date.as_str(),
                editor.start_time.as_str(),
                editor.end_date.as_str(),
                editor.end_time.as_str(),
                editor.location.as_str(),
            ),
            (
                "Review",
                "Q4 numbers",
                "13.11.2026",
                "14:00",
                "13.11.2026",
                "15:00",
                "Room 2"
            )
        );
        assert_eq!(editor.zone, Warsaw);
        assert_eq!(editor.calendar.as_ref().unwrap().name, "Personal");
        // The calendar list is grouped by account.
        let choices = editor.choices(create::choices(
            &accounts(),
            &calendars,
            &Settings::default(),
        ));
        let grouped: Vec<(String, Vec<String>)> = choices
            .into_iter()
            .map(|(email, calendars)| (email, calendars.into_iter().map(|c| c.name).collect()))
            .collect();
        assert_eq!(grouped, [(EMAIL.to_string(), vec!["Personal".to_string()])]);

        // 2. Saved: the fake Google receives the event with exactly these values.
        let Ok(Save::Create(calendar, event)) = editor.save(&regional) else {
            panic!("a new event");
        };
        let updates = engine.handle(Some(Job::Create(1, calendar, *event)), start);
        let saved = saved(&updates);
        let [Saved::Created(1, id)] = saved.as_slice() else {
            panic!("Google did not create the event");
        };
        let mut sent = google.fake.event(ACCOUNT_ID, EMAIL, id).unwrap();
        for given in ["kind", "id", "status", "etag"] {
            sent.as_object_mut().unwrap().remove(given);
        }
        assert_eq!(
            sent,
            json!({
                "summary": "Review",
                "description": "Q4 numbers",
                "start": {"dateTime": "2026-11-13T14:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2026-11-13T15:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "location": "Room 2",
                "transparency": "transparent",
                "visibility": "private",
            })
        );
        let review = shown(&updates)
            .unwrap()
            .into_iter()
            .find(|e| e.id == *id)
            .unwrap();
        assert_eq!(review.about.location, "Room 2");
    }

    #[test]
    fn tc_05_09_out_of_office_is_created_on_a_workspace_primary_calendar_and_not_offered_on_a_personal_account()
     {
        // Account B (Workspace), whose primary calendar is "B"; account A (personal) with
        // "Personal", the default calendar, connected first.
        let google = Google::start("tc-05-09");
        google.fake.add_calendar(
            ACCOUNT_ID,
            json!({
                "id": EMAIL,
                "summary": "B",
                "timeZone": "Europe/Warsaw",
                "backgroundColor": "#9fe1e7",
                "accessRole": "owner",
                "primary": true,
            }),
        );
        google.fake.add_calendar(
            OTHER_ID,
            json!({
                "id": OTHER_EMAIL,
                "summary": "Personal",
                "timeZone": "Europe/Warsaw",
                "backgroundColor": "#7986cb",
                "accessRole": "owner",
                "primary": true,
            }),
        );
        let token = google.fake.signed_in(OTHER_ID);
        (google.store.saved.lock().unwrap()).push((OTHER_ID.to_string(), token));
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        engine.handle(Some(Job::Sync(OTHER_ID.into(), window())), start);
        let calendars = engine.cache.calendar_list().unwrap();
        let entry = |id: &str, email: &str, workspace| calendar_core::accounts::Entry {
            account: calendar_core::accounts::Account {
                id: id.to_string(),
                email: email.to_string(),
                workspace,
                contacts: true,
                directory: workspace,
            },
            state: calendar_core::accounts::AccountState::Connected,
        };
        let accounts = [
            entry(OTHER_ID, OTHER_EMAIL, false),
            entry(ACCOUNT_ID, EMAIL, true),
        ];
        let regional = polish();
        let calendar = |name: &str| calendars.iter().find(|c| c.name == name).cloned();
        let new_event = |calendar| {
            let entry = QuickEntry::new(
                create::clicked(d(2026, 11, 13), 600, 60, Warsaw),
                Warsaw,
                calendar,
            );
            Editor::from_quick(&entry, &regional)
        };

        // 1. A new event in "B": out of office and focus time are offered.
        let mut editor = new_event(calendar("B"));
        assert_eq!(
            editor.kinds(&accounts),
            [Kind::Ordinary, Kind::OutOfOffice, Kind::FocusTime]
        );

        // 2. Out of office on 13 November, all day: the fake Google receives an out-of-office
        //    event in "B", and it comes back as one.
        editor.kind = Kind::OutOfOffice;
        editor.title = "Out of office".to_string();
        editor.all_day = true;
        let Ok(Save::Create(to, event)) = editor.save(&regional) else {
            panic!("a new event");
        };
        assert_eq!(to.name, "B");
        let updates = engine.handle(Some(Job::Create(1, to, *event)), start);
        let saved = saved(&updates);
        let [Saved::Created(1, id)] = saved.as_slice() else {
            panic!("Google did not create the event: {saved:?}");
        };
        let sent = google.fake.event(ACCOUNT_ID, EMAIL, id).unwrap();
        assert_eq!(sent["eventType"], "outOfOffice");
        assert_eq!(sent["start"]["dateTime"], "2026-11-13T00:00:00+01:00");
        assert_eq!(sent["end"]["dateTime"], "2026-11-14T00:00:00+01:00");
        let away = shown(&updates)
            .unwrap()
            .into_iter()
            .find(|e| e.id == *id)
            .unwrap();
        assert_eq!(away.about.kind, Kind::OutOfOffice);

        // 3. A new event in "Personal" (account A), the default calendar: neither is offered.
        let default = create::default_calendar(&accounts, &calendars, &Settings::default());
        assert_eq!(default.as_ref().unwrap().name, "Personal");
        let mut editor = new_event(default);
        assert_eq!(editor.kinds(&accounts), []);
    }

    #[test]
    fn tc_05_07_a_weekly_series_on_monday_and_wednesday_ending_after_10_is_created_with_its_rule() {
        let google = Google::start("tc-05-07");
        personal(&google);
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let calendars = engine.cache.calendar_list().unwrap();
        let regional = polish();

        // 1. "Training" on Monday 16 November 18:00–19:00, weekly on Monday and Wednesday,
        //    ending after 10 occurrences: the fake Google receives a series with that rule.
        let default = create::default_calendar(&accounts(), &calendars, &Settings::default());
        let entry = QuickEntry::new(
            create::clicked(d(2026, 11, 16), 18 * 60, 60, Warsaw),
            Warsaw,
            default,
        );
        let mut editor = Editor::from_quick(&entry, &regional);
        editor.title = "Training".to_string();
        editor.repeat = Some(editor.weekly());
        editor.toggle_day(chrono::Weekday::Wed);
        editor.ends = calendar_core::editor::Ending::AfterCount;
        editor.count = "10".to_string();
        let Ok(Save::Create(calendar, event)) = editor.save(&regional) else {
            panic!("a new event");
        };
        let mut saving = Saving::default();
        let number = saving.add(&calendar, &event);
        let updates = engine.handle(Some(Job::Create(number, calendar, *event)), start);
        let saved = saved(&updates);
        let [Saved::Created(1, id)] = saved.as_slice() else {
            panic!("Google did not create the series");
        };
        let sent = google.fake.event(ACCOUNT_ID, EMAIL, id).unwrap();
        assert_eq!(
            sent["recurrence"],
            json!(["RRULE:FREQ=WEEKLY;BYDAY=MO,WE;COUNT=10"])
        );
        assert_eq!(sent["start"]["dateTime"], "2026-11-16T18:00:00+01:00");

        // 2. The app lists 10 occurrences, from 16 November to 16 December, on Mondays and
        //    Wednesdays at 18:00; the series is no longer shown as saving.
        saving.confirmed(number, id);
        let events = shown(&updates).unwrap();
        saving.synced(&events);
        assert!(saving.is_empty());
        let mut starts: Vec<DateTime<Utc>> = events
            .iter()
            .filter(|e| e.title == "Training")
            .map(|e| match e.timing {
                Timing::Timed { start, .. } => start,
                Timing::AllDay { .. } => panic!("a timed occurrence"),
            })
            .collect();
        starts.sort();
        let occurrences: Vec<String> = starts
            .iter()
            .map(|start| start.with_timezone(&Warsaw).format("%a %F %R").to_string())
            .collect();
        assert_eq!(
            occurrences,
            [
                "Mon 2026-11-16 18:00",
                "Wed 2026-11-18 18:00",
                "Mon 2026-11-23 18:00",
                "Wed 2026-11-25 18:00",
                "Mon 2026-11-30 18:00",
                "Wed 2026-12-02 18:00",
                "Mon 2026-12-07 18:00",
                "Wed 2026-12-09 18:00",
                "Mon 2026-12-14 18:00",
                "Wed 2026-12-16 18:00",
            ]
        );
        assert!(
            events
                .iter()
                .filter(|e| e.title == "Training")
                .all(|e| e.recurring)
        );
    }

    /// "Standup" in "Personal" and the calendar "Team" of the same account, synchronized once.
    fn standup_and_team(name: &str) -> (Google, Engine, Instant) {
        let google = Google::start(name);
        personal(&google);
        google.calendar("team", "writer");
        google.event(EMAIL, "standup", "Standup");
        let start = Instant::now();
        let engine = first_sync(&google, start);
        (google, engine, start)
    }

    fn opened(engine: &Engine, regional: &Regional) -> (Event, Editor) {
        let standup = now_shown(engine)
            .into_iter()
            .find(|e| e.id == "standup")
            .unwrap();
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let editor = Editor::of(&standup, calendar, regional);
        (standup, editor)
    }

    fn edit(editor: &Editor, regional: &Regional) -> Box<Edit> {
        match editor.save(regional) {
            Ok(Save::Change(edit)) => edit,
            other => panic!("not a change: {other:?}"),
        }
    }

    #[test]
    fn a_field_changed_later_in_the_editor_reaches_google_and_comes_back_by_synchronization() {
        let (google, mut engine, start) = standup_and_team("change");
        let regional = polish();
        let (standup, mut editor) = opened(&engine, &regional);
        assert_eq!(
            (editor.start_time.as_str(), editor.end_time.as_str()),
            ("09:00", "10:00")
        );
        editor.title = "Daily".to_string();
        editor.description = "Status".to_string();
        editor.all_day = true;
        editor.end_date = "11.11.2026".to_string();
        let edit = edit(&editor, &regional);
        let mut saving = Saving::default();
        let local = saving.change(&standup, edit.changed());
        let shown_now: Vec<(String, bool)> = saving
            .shown(&now_shown(&engine))
            .into_iter()
            .map(|e| (e.title, e.saving))
            .collect();
        assert_eq!(shown_now, [("Daily".to_string(), true)]);

        let updates = engine.handle(Some(Job::Change(local, edit)), start);

        let changes = saved_changes(&updates);
        let [Saved::Changed(confirmed, etag)] = changes.as_slice() else {
            panic!("Google did not change the event");
        };
        assert_eq!(*confirmed, local);
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "standup").unwrap();
        assert_eq!(kept["etag"], etag.as_str());
        assert_eq!(kept["summary"], "Daily");
        assert_eq!(kept["description"], "Status");
        assert_eq!(kept["start"], json!({"date": "2026-11-10"}));
        assert_eq!(kept["end"], json!({"date": "2026-11-12"}));
        // Only what changed was sent, against the version opened.
        let (sent_against, patch) = google.fake.patched().pop().unwrap();
        assert_eq!(sent_against, standup.etag);
        assert!(patch.get("location").is_none());
        saving.changed(local, etag);
        let events = shown(&updates).unwrap();
        saving.synced(&events);
        assert!(saving.is_empty());
        let daily = events.iter().find(|e| e.id == "standup").unwrap();
        assert_eq!(daily.title, "Daily");
        assert_eq!(
            daily.timing,
            Timing::AllDay {
                start: d(2026, 11, 10),
                end_exclusive: d(2026, 11, 12),
            }
        );
    }

    /// "Dentist" on 12 November 16:00–17:00 in "Personal", in the colour Google gives it, if any.
    fn dentist(google: &Google, color_id: Option<&str>) {
        let mut dentist = json!({
            "id": "dentist",
            "status": "confirmed",
            "summary": "Dentist",
            "start": {"dateTime": "2026-11-12T16:00:00+01:00", "timeZone": "Europe/Warsaw"},
            "end": {"dateTime": "2026-11-12T17:00:00+01:00", "timeZone": "Europe/Warsaw"},
        });
        if let Some(id) = color_id {
            dentist["colorId"] = json!(id);
        }
        google.fake.add_event(ACCOUNT_ID, EMAIL, dentist);
    }

    /// The colours of the blocks of the week of 12 November the events are shown in.
    fn week_colors(events: &[Event]) -> Vec<Rgb> {
        let navigation = Navigation::start(None, d(2026, 11, 12), chrono::Weekday::Mon);
        let now = Utc.with_ymd_and_hms(2026, 11, 11, 9, 0, 0).unwrap();
        let (Built::Grid(week), _) =
            build(events, &Settings::default(), navigation, 30, Warsaw, now)
        else {
            panic!("the week view is built");
        };
        week.blocks.iter().map(|block| block.color).collect()
    }

    #[test]
    fn an_event_with_its_own_colour_in_google_is_shown_in_that_colour() {
        let google = Google::start("own-colour");
        personal(&google);
        // Flamingo, in a calendar of another colour.
        dentist(&google, Some("4"));
        let engine = first_sync(&google, Instant::now());

        assert_eq!(
            week_colors(&now_shown(&engine)),
            [colors::EventColor::Flamingo.rgb()]
        );
    }

    #[test]
    fn tc_05_10_changing_dentists_colour_in_the_editor_sends_it_to_google_and_shows_it() {
        let google = Google::start("tc-05-10");
        personal(&google);
        dentist(&google, None);
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let regional = polish();
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let shown_before = now_shown(&engine);
        assert_eq!(week_colors(&shown_before), [calendar.color]);

        // 1. "Dentist" opened in the editor, given Tomato and saved.
        let dentist = shown_before
            .into_iter()
            .find(|e| e.id == "dentist")
            .unwrap();
        let mut editor = Editor::of(&dentist, calendar, &regional);
        assert_eq!(editor.color, None);
        editor.color = Some(colors::EventColor::Tomato);
        let updates = engine.handle(Some(Job::Change(1, edit(&editor, &regional))), start);

        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "dentist").unwrap();
        assert_eq!(kept["colorId"], "11");
        assert_eq!(
            week_colors(&shown(&updates).unwrap()),
            [colors::EventColor::Tomato.rgb()]
        );
    }

    #[test]
    fn choosing_another_calendar_of_the_account_moves_the_event_there() {
        let (google, mut engine, start) = standup_and_team("move");
        let regional = polish();
        let (_, mut editor) = opened(&engine, &regional);
        let calendars = engine.cache.calendar_list().unwrap();
        let choices = editor.choices(create::choices(
            &accounts(),
            &calendars,
            &Settings::default(),
        ));
        let team = choices[0].1.iter().find(|c| c.id == "team").unwrap();
        editor.calendar = Some(team.clone());
        editor.location = "Room 1".to_string();

        let updates = engine.handle(Some(Job::Change(1, edit(&editor, &regional))), start);

        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        assert!(google.fake.event(ACCOUNT_ID, EMAIL, "standup").unwrap()["status"] == "cancelled");
        assert_eq!(
            google.fake.event(ACCOUNT_ID, "team", "standup").unwrap()["location"],
            "Room 1"
        );
        let events = shown(&updates).unwrap();
        let moved: Vec<&str> = events
            .iter()
            .filter(|e| e.id == "standup")
            .map(|e| e.calendar_id.as_str())
            .collect();
        assert_eq!(moved, ["team"]);
    }

    #[test]
    fn a_move_google_refuses_after_the_other_changes_were_saved_says_so() {
        let (google, mut engine, start) = standup_and_team("move-refused");
        let regional = polish();
        let (_, mut editor) = opened(&engine, &regional);
        let team = engine
            .cache
            .calendar_list()
            .unwrap()
            .into_iter()
            .find(|c| c.id == "team")
            .unwrap();
        editor.calendar = Some(team);
        editor.location = "Room 1".to_string();
        // Google stops letting the account change "team" before the app learns of it.
        google.fake.set_access_role("team", "reader");

        let updates = engine.handle(Some(Job::Change(1, edit(&editor, &regional))), start);

        let changes = saved_changes(&updates);
        let [Saved::NotChanged(1, reason)] = changes.as_slice() else {
            panic!("the move was not refused");
        };
        assert!(
            reason.starts_with("it could not be moved to the other calendar (Google refused"),
            "{reason}"
        );
        assert!(reason.ends_with("its other changes were saved"), "{reason}");
        let standup = shown(&updates)
            .unwrap()
            .into_iter()
            .find(|e| e.id == "standup")
            .unwrap();
        assert_eq!(
            (
                standup.calendar_id.as_str(),
                standup.about.location.as_str()
            ),
            (EMAIL, "Room 1")
        );
    }

    #[test]
    fn a_change_to_an_event_changed_elsewhere_meanwhile_is_refused_and_the_event_reloaded() {
        let (google, mut engine, start) = standup_and_team("conflict");
        let regional = polish();
        let (_, mut editor) = opened(&engine, &regional);
        google.event(EMAIL, "standup", "Standup (moved)");
        editor.location = "Room 1".to_string();

        let updates = engine.handle(Some(Job::Change(1, edit(&editor, &regional))), start);

        let changes = saved_changes(&updates);
        let [Saved::NotChanged(1, reason)] = changes.as_slice() else {
            panic!("the change was not refused");
        };
        assert!(reason.contains("changed elsewhere"), "{reason}");
        assert!(reason.contains("make your change again"), "{reason}");
        let events = shown(&updates).unwrap();
        assert!(events.iter().any(|e| e.title == "Standup (moved)"));
        // An account not signed in in this run changes nothing either.
        engine.stop(ACCOUNT_ID);
        let updates = engine.handle(Some(Job::Change(2, edit(&editor, &regional))), start);
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::NotChanged(2, _)]
        ));
    }

    #[test]
    fn tc_05_03_dragging_call_down_by_40_minutes_lands_at_10_45_and_is_saved_to_google() {
        let google = Google::start("tc-05-03");
        personal(&google);
        google.fake.add_event(
            ACCOUNT_ID,
            EMAIL,
            json!({
                "id": "call",
                "status": "confirmed",
                "summary": "Call",
                "start": {"dateTime": "2026-11-12T10:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2026-11-12T11:00:00+01:00", "timeZone": "Europe/Warsaw"},
            }),
        );
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let calendars = engine.cache.calendar_list().unwrap();
        // Wednesday 2026-11-11 10:00 in Warsaw, the week view.
        let now = Utc.with_ymd_and_hms(2026, 11, 11, 9, 0, 0).unwrap();
        let navigation = Navigation::start(None, d(2026, 11, 11), chrono::Weekday::Mon);
        let events = now_shown(&engine);
        let (Built::Grid(week), places) =
            build(&events, &Settings::default(), navigation, 30, Warsaw, now)
        else {
            panic!("the week view is built");
        };
        let block = &week.blocks[0];
        assert!(block.movable);

        // 1. "Call" dragged down by 40 minutes snaps to 45 and keeps its length.
        let down = grid_view::HOUR_HEIGHT * 40.0 / 60.0;
        let drop = grid_view::moved(egui::Vec2::new(0.0, down), 100.0, block.placement.day, 7);
        assert_eq!(
            drop,
            Target::Moved {
                days: 0,
                minutes: 45
            }
        );
        let call = &events[places[block.placement.event]];
        let timing = drag::dropped(call, drop, Warsaw, 60).unwrap();
        let edit = drag::edit(call, calendars[0].clone(), timing);
        let mut saving = Saving::default();
        let local = saving.change(call, edit.changed());
        let dropped: Vec<(String, String, String, bool)> = saving
            .shown(&events)
            .into_iter()
            .map(|e| {
                let Timing::Timed { start, end, .. } = e.timing else {
                    panic!("timed");
                };
                (e.title, self::local(start), self::local(end), e.saving)
            })
            .collect();
        assert_eq!(
            dropped,
            [(
                "Call".to_string(),
                "2026-11-12 10:45".to_string(),
                "2026-11-12 11:45".to_string(),
                true
            )]
        );

        // 2. The fake Google received the new time.
        let updates = engine.handle(Some(Job::Change(local, Box::new(edit))), start);
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "call").unwrap();
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T10:45:00+01:00");
        assert_eq!(kept["end"]["dateTime"], "2026-11-12T11:45:00+01:00");
        let (sent_against, patch) = google.fake.patched().pop().unwrap();
        assert_eq!(sent_against, call.etag);
        assert!(patch.get("summary").is_none());
    }

    #[test]
    fn an_all_day_event_dragged_into_the_hours_is_saved_timed_and_back_all_day() {
        let (google, mut engine, start) = standup_and_team("drag-convert");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let standup = now_shown(&engine).remove(0);
        let date = d(2026, 11, 12);
        let timing = drag::dropped(&standup, Target::AllDay { date }, Warsaw, 60).unwrap();
        let edit = drag::edit(&standup, calendar.clone(), timing);
        engine.handle(Some(Job::Change(1, Box::new(edit))), start);
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "standup").unwrap();
        assert_eq!(
            (&kept["start"], &kept["end"]),
            (
                &json!({"date": "2026-11-12"}),
                &json!({"date": "2026-11-13"})
            )
        );

        let trip = now_shown(&engine).remove(0);
        let drop = Target::Timed {
            date,
            minute: 14 * 60,
        };
        let timing = drag::dropped(&trip, drop, Warsaw, 30).unwrap();
        let edit = drag::edit(&trip, calendar, timing);
        let updates = engine.handle(Some(Job::Change(2, Box::new(edit))), start);
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(2, _)]
        ));
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "standup").unwrap();
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T14:00:00+01:00");
        assert_eq!(kept["end"]["dateTime"], "2026-11-12T14:30:00+01:00");
    }

    /// "Personal" with one event, 12 November in Warsaw from `from` to `until`, synchronized once.
    fn personal_with(
        name: &str,
        id: &str,
        title: &str,
        from: &str,
        until: &str,
    ) -> (Google, Engine, Instant) {
        let google = Google::start(name);
        personal(&google);
        google.fake.add_event(
            ACCOUNT_ID,
            EMAIL,
            json!({
                "id": id,
                "status": "confirmed",
                "summary": title,
                "start": {"dateTime": format!("2026-11-12T{from}:00+01:00"), "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": format!("2026-11-12T{until}:00+01:00"), "timeZone": "Europe/Warsaw"},
            }),
        );
        let start = Instant::now();
        let engine = first_sync(&google, start);
        (google, engine, start)
    }

    /// The events shown, by title, start and end in Warsaw.
    fn times(events: Vec<Event>) -> Vec<(String, String, String)> {
        events
            .into_iter()
            .map(|e| {
                let Timing::Timed { start, end, .. } = e.timing else {
                    panic!("timed");
                };
                (e.title, self::local(start), self::local(end))
            })
            .collect()
    }

    #[test]
    fn tc_05_04_deleting_gym_asks_nothing_and_undo_brings_it_back_also_in_google() {
        let (google, mut engine, start) = personal_with("tc-05-04", "gym", "Gym", "18:00", "19:00");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let gym = now_shown(&engine).remove(0);
        let mut saving = Saving::default();
        let mut last = LastChange::default();

        // 1. "Gym" is deleted with no question: it is gone, and the Undo notice stays 5 seconds.
        let local = saving.delete(&gym);
        last.deleting(local, &calendar, &gym, false);
        assert!(saving.shown(&now_shown(&engine)).is_empty());
        let updates = engine.handle(
            Some(Job::Delete(local, calendar, gym.clone(), false)),
            start,
        );
        assert_eq!(saved_changes(&updates), [Saved::Deleted(local)]);
        let deleted_at = Instant::now();
        saving.deleted(local);
        last.deleted(local, deleted_at);
        let events = shown(&updates).unwrap();
        saving.synced(&events);
        assert!(saving.is_empty());
        assert!(saving.shown(&events).is_empty());
        assert_eq!(
            google.fake.event(ACCOUNT_ID, EMAIL, "gym").unwrap()["status"],
            "cancelled"
        );
        assert_eq!(last.notice(deleted_at), Some((undo::DELETED, undo::NOTICE)));
        assert_eq!(last.notice(deleted_at + undo::NOTICE), None);

        // 2. Undo within the 5 seconds: "Gym" is back at 18:00–19:00, also in Google.
        assert!(last.notice(deleted_at + Duration::from_secs(3)).is_some());
        let Some(Undo::Restore(deleted, notify)) = last.undo() else {
            panic!("the deletion is undone");
        };
        let (calendar, event) = *deleted;
        let local = saving.change(&event, event.clone());
        last.undoing(local);
        assert_eq!(
            times(saving.shown(&events)),
            [(
                "Gym".to_string(),
                "2026-11-12 18:00".to_string(),
                "2026-11-12 19:00".to_string()
            )]
        );
        let updates = engine.handle(Some(Job::Restore(local, calendar, event, notify)), start);
        let changes = saved_changes(&updates);
        let [Saved::Changed(confirmed, etag)] = changes.as_slice() else {
            panic!("Google did not restore the event");
        };
        assert_eq!(*confirmed, local);
        saving.changed(local, etag);
        last.changed(local, etag, Instant::now());
        let events = shown(&updates).unwrap();
        saving.synced(&events);
        assert!(saving.is_empty());
        assert_eq!(
            times(saving.shown(&events)),
            [(
                "Gym".to_string(),
                "2026-11-12 18:00".to_string(),
                "2026-11-12 19:00".to_string()
            )]
        );
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "gym").unwrap();
        assert_eq!(kept["status"], "confirmed");
        assert_eq!(kept["summary"], "Gym");
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T18:00:00+01:00");
        // The undo itself is not a change to undo.
        assert_eq!(last.undo(), None);
    }

    #[test]
    fn tc_05_05_ctrl_z_after_dragging_call_restores_its_time_also_in_google() {
        let (google, mut engine, start) =
            personal_with("tc-05-05", "call", "Call", "10:00", "11:00");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let call = now_shown(&engine).remove(0);
        let mut saving = Saving::default();
        let mut last = LastChange::default();

        // 1. "Call" dragged to 14:00–15:00: Google holds 14:00–15:00.
        let drop = Target::Moved {
            days: 0,
            minutes: 4 * 60,
        };
        let timing = drag::dropped(&call, drop, Warsaw, 60).unwrap();
        let edit = drag::edit(&call, calendar, timing);
        let local = saving.change(&edit.event, edit.changed());
        last.changing(local, &edit);
        let updates = engine.handle(Some(Job::Change(local, Box::new(edit))), start);
        let [Saved::Changed(_, dragged)] = <[Saved; 1]>::try_from(saved_changes(&updates)).unwrap()
        else {
            panic!("Google did not change the event");
        };
        saving.changed(local, &dragged);
        last.changed(local, &dragged, Instant::now());
        saving.synced(&shown(&updates).unwrap());
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "call").unwrap();
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T14:00:00+01:00");
        assert_eq!(kept["end"]["dateTime"], "2026-11-12T15:00:00+01:00");

        // 2. Ctrl+Z: the app and Google hold 10:00–11:00 again.
        assert!(crate::undo_pressed(&mut ctrl_z()));
        let Some(Undo::Revert(undo)) = last.undo() else {
            panic!("the change is undone");
        };
        let local = saving.change(&undo.event, undo.changed());
        last.undoing(local);
        let updates = engine.handle(Some(Job::Change(local, undo)), start);
        let [Saved::Changed(_, etag)] = <[Saved; 1]>::try_from(saved_changes(&updates)).unwrap()
        else {
            panic!("Google did not undo the change");
        };
        saving.changed(local, &etag);
        let events = shown(&updates).unwrap();
        saving.synced(&events);
        assert!(saving.is_empty());
        assert_eq!(
            times(saving.shown(&events)),
            [(
                "Call".to_string(),
                "2026-11-12 10:00".to_string(),
                "2026-11-12 11:00".to_string()
            )]
        );
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "call").unwrap();
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T10:00:00+01:00");
        assert_eq!(kept["end"]["dateTime"], "2026-11-12T11:00:00+01:00");
        // Patched back against the version the drag gave it.
        let (sent_against, patch) = google.fake.patched().pop().unwrap();
        assert_eq!(sent_against, dragged);
        assert!(patch.get("summary").is_none());
    }

    /// Ctrl+Z pressed, as egui reads it.
    fn ctrl_z() -> egui::InputState {
        let raw = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Z,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            }],
            ..Default::default()
        };
        egui::InputState::default().begin_pass(raw, false, 1.0, egui::InputOptions::default())
    }

    #[test]
    fn an_undo_of_an_event_changed_elsewhere_meanwhile_is_refused_and_the_event_reloaded() {
        let (google, mut engine, start) =
            personal_with("undo-conflict", "call", "Call", "10:00", "11:00");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let call = now_shown(&engine).remove(0);
        let mut last = LastChange::default();
        let timing = drag::dropped(
            &call,
            Target::Moved {
                days: 0,
                minutes: 60,
            },
            Warsaw,
            60,
        );
        let edit = drag::edit(&call, calendar, timing.unwrap());
        last.changing(1, &edit);
        let updates = engine.handle(Some(Job::Change(1, Box::new(edit))), start);
        let [Saved::Changed(_, etag)] = <[Saved; 1]>::try_from(saved_changes(&updates)).unwrap()
        else {
            panic!("Google did not change the event");
        };
        last.changed(1, &etag, Instant::now());
        // Someone renames it in Google Calendar.
        google.fake.add_event(
            ACCOUNT_ID,
            EMAIL,
            json!({
                "id": "call",
                "status": "confirmed",
                "summary": "Call (renamed)",
                "start": {"dateTime": "2026-11-12T11:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2026-11-12T12:00:00+01:00", "timeZone": "Europe/Warsaw"},
            }),
        );

        let Some(Undo::Revert(undo)) = last.undo() else {
            panic!("the change is undone");
        };
        last.undoing(2);
        let updates = engine.handle(Some(Job::Change(2, undo)), start);

        let changes = saved_changes(&updates);
        let [Saved::NotChanged(2, reason)] = changes.as_slice() else {
            panic!("the undo was not refused");
        };
        assert!(last.is_undo(2));
        assert!(reason.contains("changed elsewhere"), "{reason}");
        assert_eq!(
            times(shown(&updates).unwrap()),
            [(
                "Call (renamed)".to_string(),
                "2026-11-12 11:00".to_string(),
                "2026-11-12 12:00".to_string()
            )]
        );
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "call").unwrap();
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T11:00:00+01:00");
    }

    #[test]
    fn a_deletion_of_an_event_changed_elsewhere_meanwhile_is_refused_and_the_event_shown_again() {
        let (google, mut engine, start) =
            personal_with("delete-conflict", "gym", "Gym", "18:00", "19:00");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let gym = now_shown(&engine).remove(0);
        google.event(EMAIL, "gym", "Gym (moved)");
        let mut saving = Saving::default();
        let local = saving.delete(&gym);

        let updates = engine.handle(
            Some(Job::Delete(local, calendar.clone(), gym.clone(), false)),
            start,
        );

        let changes = saved_changes(&updates);
        let [Saved::NotDeleted(_, reason)] = changes.as_slice() else {
            panic!("the deletion was not refused");
        };
        assert!(reason.contains("changed elsewhere"), "{reason}");
        saving.refused(local);
        let events = shown(&updates).unwrap();
        let titles: Vec<String> = saving.shown(&events).into_iter().map(|e| e.title).collect();
        assert_eq!(titles, ["Gym (moved)"]);
        assert_eq!(
            google.fake.event(ACCOUNT_ID, EMAIL, "gym").unwrap()["status"],
            "confirmed"
        );
        // An account not signed in in this run deletes nothing either.
        engine.stop(ACCOUNT_ID);
        let updates = engine.handle(Some(Job::Delete(2, calendar, gym, false)), start);
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::NotDeleted(2, _)]
        ));
    }

    /// Presses the mouse on the week view's only hour-long event and drags it down by `dy`, frame
    /// by frame, as egui gets it from the window; returns what the view reported.
    fn mouse_drag(
        view: &calendar_core::view_model::GridViewModel,
        dy: f32,
    ) -> Vec<grid_view::Action> {
        let ctx = egui::Context::default();
        let regional = polish();
        let mut state = grid_view::State::default();
        let mut actions = Vec::new();
        let mut frame = |events: Vec<egui::Event>| {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200.0, 1000.0),
                )),
                events,
                ..Default::default()
            };
            ctx.run_ui(input, |ui| {
                let mut opened = None;
                actions.extend(grid_view::show(
                    ui,
                    view,
                    &regional,
                    &mut state,
                    true,
                    &mut opened,
                ));
            })
            .textures_delta
            .clear();
        };
        frame(Vec::new());
        frame(Vec::new());
        let block = ctx.viewport(|viewport| {
            viewport
                .prev_pass
                .widgets
                .layers()
                .flat_map(|(_, widgets)| widgets)
                .find(|w| {
                    w.sense.senses_drag()
                        && (w.rect.height() - (grid_view::HOUR_HEIGHT - 2.0)).abs() < 0.5
                })
                .map(|w| w.rect)
        });
        let from = block.expect("the event is drawn").center();
        let button = |pressed| egui::Event::PointerButton {
            pos: from + egui::vec2(0.0, if pressed { 0.0 } else { dy }),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(vec![egui::Event::PointerMoved(from)]);
        frame(vec![button(true)]);
        frame(vec![egui::Event::PointerMoved(
            from + egui::vec2(0.0, 10.0),
        )]);
        frame(vec![egui::Event::PointerMoved(from + egui::vec2(0.0, dy))]);
        frame(vec![button(false)]);
        actions
    }

    #[test]
    fn tc_05_06_an_event_of_a_read_only_calendar_cannot_be_dragged_or_edited_and_says_why() {
        let google = Google::start("tc-05-06");
        personal(&google);
        google.calendar("colleague", "reader");
        google.fake.add_event(
            ACCOUNT_ID,
            "colleague",
            json!({
                "id": "theirs",
                "status": "confirmed",
                "summary": "Their meeting",
                "start": {"dateTime": "2026-11-12T12:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2026-11-12T13:00:00+01:00", "timeZone": "Europe/Warsaw"},
            }),
        );
        let start = Instant::now();
        let engine = first_sync(&google, start);
        // Wednesday 2026-11-11 10:00 in Warsaw, the week view.
        let now = Utc.with_ymd_and_hms(2026, 11, 11, 9, 0, 0).unwrap();
        let navigation = Navigation::start(None, d(2026, 11, 11), chrono::Weekday::Mon);
        let events = now_shown(&engine);
        let (Built::Grid(week), places) =
            build(&events, &Settings::default(), navigation, 30, Warsaw, now)
        else {
            panic!("the week view is built");
        };

        // 1. Dragged to 14:00, the event stays at 12:00–13:00, and the app says why.
        let actions = mouse_drag(&week, grid_view::HOUR_HEIGHT * 2.0);
        let [grid_view::Action::Unmovable { event }] = actions.as_slice() else {
            panic!("the drag was not refused: {actions:?}");
        };
        let theirs = &events[places[*event]];
        assert_eq!(theirs.title, "Their meeting");
        let mut notices = Vec::new();
        tell_unmovable(&mut notices, theirs);
        tell_unmovable(&mut notices, theirs);
        assert_eq!(
            notices,
            ["🔒 This event can't be changed: its calendar is read-only."]
        );
        assert_eq!(
            times(now_shown(&engine)),
            [(
                "Their meeting".to_string(),
                "2026-11-12 12:00".to_string(),
                "2026-11-12 13:00".to_string()
            )]
        );

        // 2. Editing is not offered, with the same reason; Google received no change.
        assert_eq!(
            theirs.protected().map(Protected::why),
            Some(notices[0].as_str())
        );
        assert!(google.fake.patched().is_empty());
        let kept = google
            .fake
            .event(ACCOUNT_ID, "colleague", "theirs")
            .unwrap();
        assert_eq!(kept["summary"], "Their meeting");
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T12:00:00+01:00");
    }

    #[test]
    fn a_guests_copy_only_its_organizer_may_change_cannot_be_dragged_and_says_why() {
        let google = Google::start("organizer-only");
        personal(&google);
        google.fake.add_event(
            ACCOUNT_ID,
            EMAIL,
            json!({
                "id": "invited",
                "status": "confirmed",
                "summary": "Board meeting",
                "organizer": {"email": "boss@example.com"},
                "start": {"dateTime": "2026-11-12T12:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2026-11-12T13:00:00+01:00", "timeZone": "Europe/Warsaw"},
            }),
        );
        let start = Instant::now();
        let engine = first_sync(&google, start);
        let now = Utc.with_ymd_and_hms(2026, 11, 11, 9, 0, 0).unwrap();
        let navigation = Navigation::start(None, d(2026, 11, 11), chrono::Weekday::Mon);
        let events = now_shown(&engine);
        let (Built::Grid(week), places) =
            build(&events, &Settings::default(), navigation, 30, Warsaw, now)
        else {
            panic!("the week view is built");
        };

        let actions = mouse_drag(&week, grid_view::HOUR_HEIGHT * 2.0);

        let [grid_view::Action::Unmovable { event }] = actions.as_slice() else {
            panic!("the drag was not refused: {actions:?}");
        };
        let invited = &events[places[*event]];
        // Its calendar is the user's own, so it carries no lock.
        assert!(!invited.read_only);
        let mut notices = Vec::new();
        tell_unmovable(&mut notices, invited);
        assert_eq!(
            notices,
            ["This event can't be changed: only its organizer can change it."]
        );
        assert!(invited.protected().is_some());
        assert!(google.fake.patched().is_empty());
    }

    #[test]
    fn a_change_google_refuses_is_told_clearly_and_the_event_returns_to_how_it_was() {
        let (google, mut engine, start) =
            personal_with("change-refused", "call", "Call", "10:00", "11:00");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let call = now_shown(&engine).remove(0);
        let drop = Target::Moved {
            days: 0,
            minutes: 60,
        };
        let timing = drag::dropped(&call, drop, Warsaw, 60).unwrap();
        let edit = drag::edit(&call, calendar, timing);
        let mut saving = Saving::default();
        let local = saving.change(&call, edit.changed());
        // Its owner stops letting the account change the calendar before the app learns of it.
        google.fake.set_access_role(EMAIL, "reader");

        let updates = engine.handle(Some(Job::Change(local, Box::new(edit))), start);

        let changes = saved_changes(&updates);
        let [Saved::NotChanged(1, reason)] = changes.as_slice() else {
            panic!("the change was not refused");
        };
        assert_eq!(
            reason,
            "Google refused (You need to have writer access to this calendar.)"
        );
        assert!(!reason.contains("Call"));
        saving.refused(local);
        let shown = shown(&updates).unwrap();
        assert_eq!(
            times(saving.shown(&shown)),
            [(
                "Call".to_string(),
                "2026-11-12 10:00".to_string(),
                "2026-11-12 11:00".to_string()
            )]
        );
        // And from then on it is protected, with the reason why.
        assert!(saving.shown(&shown)[0].protected().is_some());
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "call").unwrap();
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T10:00:00+01:00");
    }

    /// `TC-05-08`'s calendar: "Personal", the default, with "Standup" weekly on Monday
    /// 09:00–09:15, 10 occurrences from 16 November.
    fn standup_series(name: &str) -> (Google, Engine, Instant) {
        let google = Google::start(name);
        personal(&google);
        google.fake.add_event(
            ACCOUNT_ID,
            EMAIL,
            json!({
                "id": "standup",
                "status": "confirmed",
                "summary": "Standup",
                "start": {"dateTime": "2026-11-16T09:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2026-11-16T09:15:00+01:00", "timeZone": "Europe/Warsaw"},
                "recurrence": ["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=10"],
            }),
        );
        let start = Instant::now();
        let engine = first_sync(&google, start);
        (google, engine, start)
    }

    /// The local starts of the occurrences of "Standup" among the events, in order.
    fn standups(events: &[Event]) -> Vec<String> {
        let mut starts: Vec<DateTime<Utc>> = events
            .iter()
            .filter(|e| e.title == "Standup")
            .map(|e| match e.timing {
                Timing::Timed { start, .. } => start,
                Timing::AllDay { .. } => panic!("a timed occurrence"),
            })
            .collect();
        starts.sort();
        starts.into_iter().map(local).collect()
    }

    /// The occurrence of "Standup" starting on the day, as the engine shows it now.
    fn standup_on(engine: &Engine, date: chrono::NaiveDate) -> Event {
        now_shown(engine)
            .into_iter()
            .find(|e| {
                e.title == "Standup"
                    && matches!(e.timing, Timing::Timed { start, .. }
                        if start.with_timezone(&Warsaw).date_naive() == date)
            })
            .unwrap()
    }

    /// The occurrences of "Standup" as Google keeps them, fetched afresh from the fake Google.
    fn standups_in_google(google: &Google, engine: &Engine) -> Vec<String> {
        let session = (google.sign_in())(ACCOUNT_ID).unwrap();
        let calendars = engine.cache.calendar_list().unwrap();
        let span = Window {
            from: d(2026, 11, 1),
            until: d(2027, 2, 1),
            zone: Warsaw,
        };
        standups(&calendar_core::sync::fetch_events(
            session.as_ref(),
            &calendars,
            span,
        ))
    }

    #[test]
    fn tc_05_08_this_and_following_from_the_fifth_of_ten_leaves_one_to_four_and_moves_five_to_ten()
    {
        let (google, mut engine, start) = standup_series("tc-05-08");
        let regional = polish();
        let calendar = engine.cache.calendar_list().unwrap().remove(0);

        // 1. The 5th occurrence (14 December) opened, its start changed to 11:00: the app asks
        //    for the scope before saving, and "this and following events" is chosen.
        let fifth = standup_on(&engine, d(2026, 12, 14));
        let mut editor = Editor::of(&fifth, calendar, &regional);
        editor.start_time = "11:00".to_string();
        editor.follow_start(&regional);
        let Err(question) = Question::changing(*edit(&editor, &regional)) else {
            panic!("the scope is asked first");
        };
        assert_eq!(
            question.scopes(),
            [
                (Scope::This, None),
                (Scope::Following, None),
                (Scope::All, None)
            ]
        );
        let Question::Change(change) = question else {
            panic!("a change");
        };
        let mut saving = Saving::default();
        let local = saving.change_series(&change.event, change.changed());
        let updates = engine.handle(
            Some(Job::ChangeSeries(local, change, Scope::Following)),
            start,
        );
        assert!(matches!(
            saved_changes(&updates)[..],
            [Saved::Changed(1, _)]
        ));

        // 2. Occurrences 1–4 are at 09:00 and 5–10 at 11:00, in the app and in the fake Google.
        let at_nine = ["2026-11-16", "2026-11-23", "2026-11-30", "2026-12-07"];
        let at_eleven = [
            "2026-12-14",
            "2026-12-21",
            "2026-12-28",
            "2027-01-04",
            "2027-01-11",
            "2027-01-18",
        ];
        let expected: Vec<String> = (at_nine.iter().map(|day| format!("{day} 09:00")))
            .chain(at_eleven.iter().map(|day| format!("{day} 11:00")))
            .collect();
        let events = shown(&updates).unwrap();
        saving.changed(local, "");
        saving.synced(&events);
        assert!(saving.is_empty());
        assert_eq!(standups(&events), expected);
        assert_eq!(standups_in_google(&google, &engine), expected);
    }

    #[test]
    fn all_events_move_the_series_times_and_this_event_only_the_occurrence_also_in_google() {
        let (google, mut engine, start) = standup_series("series-all");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let fifth = standup_on(&engine, d(2026, 12, 14));
        let to = |(h, m): (u32, u32)| {
            let at = |m| {
                Warsaw
                    .with_ymd_and_hms(2026, 12, 14, h, m, 0)
                    .unwrap()
                    .with_timezone(&Utc)
            };
            Timing::Timed {
                start: at(m),
                end: at(m + 15),
                zone: Warsaw,
            }
        };

        let all = drag::edit(&fifth, calendar.clone(), to((10, 0)));
        let updates = engine.handle(Some(Job::ChangeSeries(1, Box::new(all), Scope::All)), start);
        assert!(matches!(
            saved_changes(&updates)[..],
            [Saved::Changed(1, _)]
        ));
        let moved = standup_on(&engine, d(2026, 12, 14));
        let this = drag::edit(&moved, calendar, to((12, 0)));
        let updates = engine.handle(Some(Job::Change(2, Box::new(this))), start);

        assert!(matches!(
            saved_changes(&updates)[..],
            [Saved::Changed(2, _)]
        ));
        let mut expected: Vec<String> = (0..10)
            .map(|week| {
                let day = d(2026, 11, 16) + chrono::Duration::weeks(week);
                let hour = if week == 4 { 12 } else { 10 };
                format!("{day} {hour}:00")
            })
            .collect();
        expected.sort();
        assert_eq!(standups(&now_shown(&engine)), expected);
        assert_eq!(standups_in_google(&google, &engine), expected);
    }

    #[test]
    fn deleting_this_following_or_all_events_matches_in_google() {
        let (google, mut engine, start) = standup_series("series-delete");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let deleting = |engine: &Engine, date| {
            let Err(Question::Delete(deleted)) =
                Question::deleting(calendar.clone(), standup_on(engine, date))
            else {
                panic!("the scope is asked first");
            };
            let (calendar, event) = *deleted;
            (calendar, event)
        };

        // This event: the 2nd occurrence alone, without a version of its own.
        let (calendar, second) = deleting(&engine, d(2026, 11, 23));
        let updates = engine.handle(Some(Job::Delete(1, calendar, second, false)), start);
        assert_eq!(saved_changes(&updates), [Saved::Deleted(1)]);
        // This and following: from the 8th on.
        let (calendar, eighth) = deleting(&engine, d(2027, 1, 4));
        let updates = engine.handle(
            Some(Job::DeleteSeries(
                2,
                calendar,
                eighth,
                Scope::Following,
                false,
            )),
            start,
        );
        assert_eq!(saved_changes(&updates), [Saved::Deleted(2)]);
        let left = [
            "2026-11-16 09:00",
            "2026-11-30 09:00",
            "2026-12-07 09:00",
            "2026-12-14 09:00",
            "2026-12-21 09:00",
            "2026-12-28 09:00",
        ];
        assert_eq!(standups(&now_shown(&engine)), left);
        assert_eq!(standups_in_google(&google, &engine), left);

        // All events: the whole series.
        let (calendar, third) = deleting(&engine, d(2026, 11, 30));
        let updates = engine.handle(
            Some(Job::DeleteSeries(3, calendar, third, Scope::All, false)),
            start,
        );
        assert_eq!(saved_changes(&updates), [Saved::Deleted(3)]);
        assert!(standups(&now_shown(&engine)).is_empty());
        assert!(standups_in_google(&google, &engine).is_empty());
        assert_eq!(
            google.fake.event(ACCOUNT_ID, EMAIL, "standup").unwrap()["status"],
            "cancelled"
        );
    }

    /// The 5th occurrence of "Standup" (14 December) moved to 15:00 on its own, in the app and
    /// in Google.
    fn fifth_moved_alone(google: &Google, engine: &mut Engine, start: Instant) -> Event {
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let fifth = standup_on(engine, d(2026, 12, 14));
        let at = |h| Warsaw.with_ymd_and_hms(2026, 12, 14, h, 0, 0).unwrap();
        let timing = Timing::Timed {
            start: at(15).with_timezone(&Utc),
            end: (at(15) + chrono::Duration::minutes(15)).with_timezone(&Utc),
            zone: Warsaw,
        };
        let alone = drag::edit(&fifth, calendar, timing);
        let updates = engine.handle(Some(Job::Change(1, Box::new(alone))), start);
        assert!(matches!(
            saved_changes(&updates)[..],
            [Saved::Changed(1, _)]
        ));
        assert!(standups_in_google(google, engine).contains(&"2026-12-14 15:00".to_string()));
        standup_on(engine, d(2026, 12, 14))
    }

    #[test]
    fn this_and_following_on_an_occurrence_changed_on_its_own_takes_it_along_also_in_google() {
        let (google, mut engine, start) = standup_series("series-exception-change");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let moved = fifth_moved_alone(&google, &mut engine, start);
        let at = |h| Warsaw.with_ymd_and_hms(2026, 12, 14, h, 0, 0).unwrap();
        let timing = Timing::Timed {
            start: at(11).with_timezone(&Utc),
            end: (at(11) + chrono::Duration::minutes(15)).with_timezone(&Utc),
            zone: Warsaw,
        };

        let following = drag::edit(&moved, calendar, timing);
        let updates = engine.handle(
            Some(Job::ChangeSeries(2, Box::new(following), Scope::Following)),
            start,
        );

        assert!(matches!(
            saved_changes(&updates)[..],
            [Saved::Changed(2, _)]
        ));
        let on_the_14th = |starts: Vec<String>| -> Vec<String> {
            starts
                .into_iter()
                .filter(|start| start.starts_with("2026-12-14"))
                .collect()
        };
        assert_eq!(
            on_the_14th(standups(&now_shown(&engine))),
            ["2026-12-14 11:00"]
        );
        assert_eq!(
            on_the_14th(standups_in_google(&google, &engine)),
            ["2026-12-14 11:00"]
        );
    }

    #[test]
    fn deleting_this_and_following_on_an_occurrence_changed_on_its_own_deletes_it_too() {
        let (google, mut engine, start) = standup_series("series-exception-delete");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let moved = fifth_moved_alone(&google, &mut engine, start);

        let updates = engine.handle(
            Some(Job::DeleteSeries(
                2,
                calendar,
                moved,
                Scope::Following,
                false,
            )),
            start,
        );

        assert_eq!(saved_changes(&updates), [Saved::Deleted(2)]);
        let left = [
            "2026-11-16 09:00",
            "2026-11-23 09:00",
            "2026-11-30 09:00",
            "2026-12-07 09:00",
        ];
        assert_eq!(standups(&now_shown(&engine)), left);
        assert_eq!(standups_in_google(&google, &engine), left);
    }

    /// `MOD-05-F29`: a series of every second Tuesday of the month, made in Google.
    #[test]
    fn a_series_with_a_rule_the_presets_cannot_express_keeps_it_when_its_title_changes() {
        let google = Google::start("series-complex");
        personal(&google);
        let rule = "RRULE:FREQ=MONTHLY;BYDAY=2TU";
        google.fake.add_event(
            ACCOUNT_ID,
            EMAIL,
            json!({
                "id": "board",
                "status": "confirmed",
                "summary": "Board",
                "start": {"dateTime": "2026-11-10T18:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2026-11-10T19:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "recurrence": [rule],
            }),
        );
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let regional = polish();
        let second = now_shown(&engine)
            .into_iter()
            .find(|e| e.id == "board_20261208T170000Z")
            .unwrap();
        let mut editor = Editor::of(&second, calendar, &regional);
        editor.title = "Board meeting".to_string();

        let updates = engine.handle(
            Some(Job::ChangeSeries(1, edit(&editor, &regional), Scope::All)),
            start,
        );

        assert!(matches!(
            saved_changes(&updates)[..],
            [Saved::Changed(1, _)]
        ));
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "board").unwrap();
        assert_eq!(kept["summary"], "Board meeting");
        assert_eq!(kept["recurrence"], json!([rule]));
    }

    /// What Google answered about changed events, in order.
    fn saved_changes(updates: &[Update]) -> Vec<Saved> {
        updates
            .iter()
            .filter_map(|update| match update {
                Update::Changed(local, etag) => Some(Saved::Changed(*local, etag.clone())),
                Update::NotChanged(local, reason) => {
                    Some(Saved::NotChanged(*local, reason.clone()))
                }
                Update::Deleted(local) => Some(Saved::Deleted(*local)),
                Update::NotDeleted(local, reason) => {
                    Some(Saved::NotDeleted(*local, reason.clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// The people the engine finds for what is typed in the guest field, with the account's
    /// recent guests after them, as the editor is offered them.
    fn suggested(engine: &mut Engine, editor: &Editor, reach: Reach) -> Vec<Person> {
        let updates = engine.handle(
            Some(Job::Suggest(
                ACCOUNT_ID.into(),
                editor.guest_field.clone(),
                reach,
            )),
            Instant::now(),
        );
        let [Update::People(account, found)] = updates.as_slice() else {
            panic!("the people found");
        };
        assert_eq!(account, ACCOUNT_ID);
        let found: Vec<Person> = (found.iter().cloned())
            .chain(guests::recent(&now_shown(engine), ACCOUNT_ID))
            .collect();
        guests::suggestions(&editor.guest_field, &found, &editor.guests, &[])
    }

    fn named(people: &[Person]) -> Vec<String> {
        people.iter().map(Person::label).collect()
    }

    #[test]
    fn tc_06_01_typing_part_of_a_name_suggests_contacts_and_recent_guests_and_saving_invites_them()
    {
        let google = Google::start("tc-06-01");
        personal(&google);
        google
            .fake
            .add_contact(ACCOUNT_ID, "Adam Kowalski", "adam@example.com");
        // "ewa@example.com" was invited to an earlier event.
        let mut earlier = event("earlier", "Planning");
        earlier["attendees"] = json!([
            {"email": EMAIL, "self": true, "organizer": true, "responseStatus": "accepted"},
            {"email": "ewa@example.com", "responseStatus": "accepted"}
        ]);
        google.fake.add_event(ACCOUNT_ID, EMAIL, earlier);
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let regional = polish();
        let calendars = engine.cache.calendar_list().unwrap();
        let reach = Reach::of(&accounts()[0].account);
        // A new event "Kickoff" on 13 November 10:00–11:00, open in the editor.
        let default = create::default_calendar(&accounts(), &calendars, &Settings::default());
        let entry = QuickEntry {
            title: "Kickoff".to_string(),
            ..QuickEntry::new(
                create::clicked(d(2026, 11, 13), 600, 60, Warsaw),
                Warsaw,
                default,
            )
        };
        let mut editor = Editor::from_quick(&entry, &regional);
        let own = editor.own_addresses(&accounts());

        // 1. "Ada" typed: "Adam Kowalski" is suggested.
        editor.guest_field = "Ada".to_string();
        let found = suggested(&mut engine, &editor, reach);
        assert_eq!(named(&found), ["Adam Kowalski <adam@example.com>"]);

        // 2. He is chosen, then "ewa" typed and "ewa@example.com" chosen: both are guests.
        editor.add_guest(&found[0], &own);
        editor.guest_field = "ewa".to_string();
        let found = suggested(&mut engine, &editor, reach);
        assert_eq!(named(&found), ["ewa@example.com"]);
        editor.add_guest(&found[0], &own);
        let listed: Vec<String> = editor.listed_guests().map(|g| g.email.clone()).collect();
        assert_eq!(listed, ["adam@example.com", "ewa@example.com"]);

        // 3. Saved: Google receives "Kickoff" with both guests, and is asked to invite them.
        let Ok(Save::Create(calendar, event)) = editor.save(&regional) else {
            panic!("a new event");
        };
        let updates = engine.handle(Some(Job::Create(1, calendar, *event)), start);
        let saved = saved(&updates);
        let [Saved::Created(1, id)] = saved.as_slice() else {
            panic!("Google did not create the event");
        };
        let sent = google.fake.event(ACCOUNT_ID, EMAIL, id).unwrap();
        assert_eq!(sent["summary"], "Kickoff");
        assert_eq!(
            sent["attendees"],
            json!([
                {"email": "adam@example.com", "displayName": "Adam Kowalski", "responseStatus": "needsAction"},
                {"email": "ewa@example.com", "responseStatus": "needsAction"}
            ])
        );
        let created: Vec<String> = (google.fake.requests().into_iter())
            .filter(|r| r.starts_with("POST /calendar/v3/"))
            .collect();
        assert_eq!(created.len(), 1);
        assert!(
            created[0].contains("/events?sendUpdates=all"),
            "{created:?}"
        );
        // Back through synchronization, with its guests.
        let kickoff = (shown(&updates).unwrap().into_iter())
            .find(|e| e.title == "Kickoff")
            .unwrap();
        let guests: Vec<String> = kickoff.about.guests.into_iter().map(|g| g.email).collect();
        assert_eq!(guests, ["adam@example.com", "ewa@example.com"]);
    }

    #[test]
    fn tc_06_03_a_guests_answer_is_shown_as_an_icon_and_changes_after_the_next_synchronization() {
        let google = Google::start("tc-06-03");
        personal(&google);
        let mut kickoff = event("kickoff", "Kickoff");
        kickoff["organizer"] = json!({"email": EMAIL, "self": true});
        kickoff["attendees"] = json!([
            {"email": EMAIL, "self": true, "organizer": true, "responseStatus": "accepted"},
            {"email": "adam@example.com", "displayName": "Adam Kowalski", "responseStatus": "accepted"},
            {"email": "ewa@example.com", "responseStatus": "needsAction"}
        ]);
        google.fake.add_event(ACCOUNT_ID, EMAIL, kickoff.clone());
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        // Each guest as Kickoff's details list them: name, address and the answer's icon.
        let listed = |events: Vec<Event>| -> Vec<(String, String, &'static str)> {
            let kickoff = events.into_iter().find(|e| e.id == "kickoff").unwrap();
            let details = event_details(&kickoff, Warsaw);
            (details.listed_guests(false).iter())
                .filter(|g| !g.own)
                .map(|g| (g.name.clone(), g.email.clone(), g.response.icon()))
                .collect()
        };

        // 1. Adam accepted, Ewa awaiting.
        assert_eq!(
            listed(now_shown(&engine)),
            [
                ("Adam Kowalski".into(), "adam@example.com".into(), "✔"),
                (String::new(), "ewa@example.com".into(), "⏳"),
            ]
        );

        // 2. Ewa declines in Google; the next round shows it.
        kickoff["attendees"][2]["responseStatus"] = json!("declined");
        google.fake.add_event(ACCOUNT_ID, EMAIL, kickoff);
        let updates = engine.handle(None, start + secs(60));
        assert_eq!(
            listed(shown(&updates).unwrap()),
            [
                ("Adam Kowalski".into(), "adam@example.com".into(), "✔"),
                (String::new(), "ewa@example.com".into(), "✖"),
            ]
        );
    }

    #[test]
    fn tc_06_02_on_a_workspace_account_a_colleague_from_the_directory_is_suggested() {
        let google = Google::start("tc-06-02");
        personal(&google);
        // "Anna Nowak" is in the organization's directory, not in the contacts.
        google
            .fake
            .add_colleague(ACCOUNT_ID, "Anna Nowak", "anna.nowak@example.com");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let regional = polish();
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let workspace = calendar_core::accounts::Account {
            workspace: true,
            directory: true,
            ..accounts()[0].account.clone()
        };
        let entry = QuickEntry::new(
            create::clicked(d(2026, 11, 13), 600, 60, Warsaw),
            Warsaw,
            Some(calendar),
        );
        let mut editor = Editor::from_quick(&entry, &regional);

        // 1. "Anna" typed: "Anna Nowak" is suggested, from the directory.
        editor.guest_field = "Anna".to_string();
        let found = suggested(&mut engine, &editor, Reach::of(&workspace));

        assert_eq!(named(&found), ["Anna Nowak <anna.nowak@example.com>"]);
        assert!(
            (google.fake.requests().iter())
                .any(|r| r.starts_with("GET /people/v1/people:searchDirectoryPeople?"))
        );
    }

    #[test]
    fn without_the_directory_granted_at_sign_in_no_colleague_is_suggested_or_asked_for() {
        let google = Google::start("no-directory");
        personal(&google);
        google
            .fake
            .add_colleague(ACCOUNT_ID, "Anna Nowak", "anna.nowak@example.com");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        // A Workspace account that did not grant the directory (`DEC-52`).
        let without = calendar_core::accounts::Account {
            workspace: true,
            directory: false,
            ..accounts()[0].account.clone()
        };
        let entry = QuickEntry::new(
            create::clicked(d(2026, 11, 13), 600, 60, Warsaw),
            Warsaw,
            Some(calendar),
        );
        let mut editor = Editor::from_quick(&entry, &polish());
        editor.guest_field = "Anna".to_string();

        let found = suggested(&mut engine, &editor, Reach::of(&without));

        assert!(found.is_empty());
        assert!((google.fake.requests().iter()).all(|r| !r.contains("searchDirectoryPeople")));
    }

    #[test]
    fn guests_are_added_to_and_removed_from_an_existing_event_at_google() {
        let google = Google::start("edit-guests");
        personal(&google);
        let mut review = event("review", "Review");
        review["organizer"] = json!({"email": EMAIL, "self": true});
        review["attendees"] = json!([
            {"email": EMAIL, "self": true, "organizer": true, "responseStatus": "accepted"},
            {"email": "ewa@example.com", "responseStatus": "declined"}
        ]);
        google.fake.add_event(ACCOUNT_ID, EMAIL, review);
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let regional = polish();
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let review = (now_shown(&engine).into_iter())
            .find(|e| e.id == "review")
            .unwrap();
        let mut editor = Editor::of(&review, calendar, &regional);

        // Ewa removed, Adam added as an optional guest.
        editor.remove_guest("ewa@example.com");
        editor.guest_field = "adam@example.com".to_string();
        editor.add_typed_guest(&editor.own_addresses(&accounts()));
        editor.listed_guests().next().unwrap().optional = true;
        let updates = engine.handle(Some(Job::Change(1, edit(&editor, &regional))), start);

        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "review").unwrap();
        assert_eq!(
            kept["attendees"],
            json!([
                {"email": EMAIL, "self": true, "organizer": true, "responseStatus": "accepted"},
                {"email": "adam@example.com", "optional": true, "responseStatus": "needsAction"}
            ])
        );
        let review = (shown(&updates).unwrap().into_iter())
            .find(|e| e.id == "review")
            .unwrap();
        let guests: Vec<(String, bool)> = (review.about.guests.into_iter())
            .filter(|g| !g.own)
            .map(|g| (g.email, g.optional))
            .collect();
        assert_eq!(guests, [("adam@example.com".to_string(), true)]);
    }

    /// "Sync" on 12 November 10:00–10:30, organized by the user, with guest Adam
    /// (`TC-06-04`), and the engine after the first synchronization.
    fn sync_with_adam(name: &str) -> (Google, Engine, Instant) {
        let google = Google::start(name);
        personal(&google);
        google.fake.add_event(
            ACCOUNT_ID,
            EMAIL,
            json!({
                "id": "sync",
                "status": "confirmed",
                "summary": "Sync",
                "start": {"dateTime": "2026-11-12T10:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2026-11-12T10:30:00+01:00", "timeZone": "Europe/Warsaw"},
                "organizer": {"email": EMAIL, "self": true},
                "attendees": [
                    {"email": EMAIL, "self": true, "organizer": true, "responseStatus": "accepted"},
                    {"email": "adam@example.com", "displayName": "Adam Kowalski", "responseStatus": "needsAction"}
                ],
            }),
        );
        let start = Instant::now();
        let engine = first_sync(&google, start);
        (google, engine, start)
    }

    /// The writes the fake Google received, in order.
    fn writes(google: &Google) -> Vec<String> {
        (google.fake.requests().into_iter())
            .filter(|r| !r.starts_with("GET ") && r.contains("/calendar/v3/calendars/"))
            .collect()
    }

    /// "Sync" dropped at 11:00, the change the drop asks to save.
    fn sync_dropped_at_11(engine: &Engine) -> Pending {
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let sync = (now_shown(engine).into_iter())
            .find(|e| e.id == "sync")
            .unwrap();
        let drop = Target::Moved {
            days: 0,
            minutes: 60,
        };
        let timing = drag::dropped(&sync, drop, Warsaw, 60).unwrap();
        // Not an occurrence of a series: no scope is asked first.
        let Ok(edit) = Question::changing(drag::edit(&sync, calendar, timing)) else {
            panic!("no scope question");
        };
        Pending::Change(Box::new(edit), Scope::This)
    }

    #[test]
    fn tc_06_04_dragging_an_event_with_guests_asks_once_whether_to_notify_them_and_no_sends_no_update()
     {
        let (google, mut engine, start) = sync_with_adam("tc-06-04");

        // 1. "Sync" dragged to 11:00: the app asks once whether to send the guests an update.
        let pending = sync_dropped_at_11(&engine);
        assert_eq!(pending.asks(), Some(notify::UPDATE));

        // 2. "No": Google receives the new time with guest notifications turned off.
        let Pending::Change(mut edit, Scope::This) = pending else {
            panic!("a change of this event");
        };
        edit.notify = false;
        let updates = engine.handle(Some(Job::Change(1, edit)), start);

        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "sync").unwrap();
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T11:00:00+01:00");
        assert_eq!(kept["end"]["dateTime"], "2026-11-12T11:30:00+01:00");
        let [patched] = writes(&google).try_into().unwrap();
        assert!(patched.starts_with("PATCH "), "{patched}");
        assert!(
            patched.ends_with("/events/sync?sendUpdates=none"),
            "{patched}"
        );
    }

    #[test]
    fn answering_yes_to_a_dragged_meeting_has_google_notify_its_guests() {
        let (google, mut engine, start) = sync_with_adam("notify-yes");
        let Pending::Change(mut edit, _) = sync_dropped_at_11(&engine) else {
            panic!("a change");
        };

        edit.notify = true;
        engine.handle(Some(Job::Change(1, edit)), start);

        let [patched] = writes(&google).try_into().unwrap();
        assert!(
            patched.ends_with("/events/sync?sendUpdates=all"),
            "{patched}"
        );
    }

    #[test]
    fn saving_a_change_in_the_editor_and_deleting_the_event_ask_the_same_question() {
        let (google, mut engine, start) = sync_with_adam("notify-editor-delete");
        let regional = polish();
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let sync = (now_shown(&engine).into_iter())
            .find(|e| e.id == "sync")
            .unwrap();

        // A new title saved in the editor asks, and "Yes" notifies the guests.
        let mut editor = Editor::of(&sync, calendar.clone(), &regional);
        editor.title = "Weekly sync".to_string();
        let Ok(mut edit) = Question::changing(*edit(&editor, &regional)) else {
            panic!("no scope question");
        };
        let asked = Pending::Change(Box::new(edit.clone()), Scope::This).asks();
        assert_eq!(asked, Some(notify::UPDATE));
        edit.notify = true;
        engine.handle(Some(Job::Change(1, Box::new(edit))), start);
        assert_eq!(
            google.fake.event(ACCOUNT_ID, EMAIL, "sync").unwrap()["summary"],
            "Weekly sync"
        );

        // Deleting it asks too, and "No" deletes it without notifying them.
        let sync = (now_shown(&engine).into_iter())
            .find(|e| e.id == "sync")
            .unwrap();
        let Ok(deleted) = Question::deleting(calendar, sync) else {
            panic!("no scope question");
        };
        let asked = Pending::Delete(Box::new(deleted.clone()), Scope::This).asks();
        assert_eq!(asked, Some(notify::CANCELLATION));
        let (calendar, sync) = deleted;
        let updates = engine.handle(
            Some(Job::Delete(2, calendar.clone(), sync.clone(), false)),
            start,
        );
        assert_eq!(saved_changes(&updates), [Saved::Deleted(2)]);

        // Its undo tells the guests as the deletion did.
        let updates = engine.handle(Some(Job::Restore(3, calendar, sync, false)), start);
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(3, _)]
        ));
        let writes: Vec<String> = (writes(&google).into_iter())
            .map(|r| r.replace(&format!("/calendar/v3/calendars/{EMAIL}"), ""))
            .collect();
        assert_eq!(
            writes,
            [
                "PATCH /events/sync?sendUpdates=all",
                "DELETE /events/sync?sendUpdates=none",
                "PATCH /events/sync?sendUpdates=none",
            ]
        );
    }

    #[test]
    fn a_change_to_the_following_events_of_a_series_with_guests_tells_google_with_each_request() {
        let (google, mut engine, start) = standup_series("notify-series");
        let mut standup = google.fake.event(ACCOUNT_ID, EMAIL, "standup").unwrap();
        standup["attendees"] = json!([{"email": "adam@example.com", "responseStatus": "accepted"}]);
        google.fake.add_event(ACCOUNT_ID, EMAIL, standup);
        engine.handle(Some(Job::Refresh), start);
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let fifth = standup_on(&engine, d(2026, 12, 14));
        let retitled = Edit {
            patch: Patch {
                title: Some("Daily".to_string()),
                ..Patch::default()
            },
            ..drag::edit(&fifth, calendar, fifth.timing)
        };
        let Err(question) = Question::changing(retitled) else {
            panic!("the scope is asked first");
        };
        let pending = Pending::of(question, Scope::Following);
        assert_eq!(pending.asks(), Some(notify::UPDATE));
        let Pending::Change(mut edit, scope) = pending else {
            panic!("a change");
        };

        edit.notify = true;
        let updates = engine.handle(Some(Job::ChangeSeries(1, edit, scope)), start);

        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        let writes = writes(&google);
        assert_eq!(writes.len(), 2, "{writes:?}");
        assert!(
            writes.iter().all(|r| r.ends_with("sendUpdates=all")),
            "{writes:?}"
        );
    }

    /// The connected account as the views know it, for the invitations it is listed under.
    fn connected() -> Vec<calendar_core::accounts::Entry> {
        vec![calendar_core::accounts::Entry {
            account: calendar_core::accounts::Account {
                id: ACCOUNT_ID.to_string(),
                email: EMAIL.to_string(),
                workspace: false,
                contacts: false,
                directory: false,
            },
            state: calendar_core::accounts::AccountState::Connected,
        }]
    }

    /// Thursday 2026-11-12 09:00 in Warsaw (`TC-06-06`…`TC-06-08`).
    fn thursday_9() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 12, 8, 0, 0).unwrap()
    }

    /// An invitation from boss@example.com, who does not let guests change it, awaiting the
    /// user's answer.
    fn invitation(id: &str, title: &str, start: &str, end: &str) -> Value {
        json!({
            "id": id,
            "status": "confirmed",
            "summary": title,
            "start": {"dateTime": start, "timeZone": "Europe/Warsaw"},
            "end": {"dateTime": end, "timeZone": "Europe/Warsaw"},
            "organizer": {"email": "boss@example.com"},
            "attendees": [
                {"email": "boss@example.com", "organizer": true, "responseStatus": "accepted"},
                {"email": EMAIL, "self": true, "responseStatus": "needsAction"}
            ],
        })
    }

    /// "Board" on Thursday 12 November 14:00–15:00, awaiting the user's answer, and the engine
    /// after the first synchronization.
    fn board(name: &str) -> (Google, Engine, Instant) {
        let google = Google::start(name);
        personal(&google);
        let board = invitation(
            "board",
            "Board",
            "2026-11-12T14:00:00+01:00",
            "2026-11-12T15:00:00+01:00",
        );
        google.fake.add_event(ACCOUNT_ID, EMAIL, board);
        let start = Instant::now();
        let engine = first_sync(&google, start);
        (google, engine, start)
    }

    /// The titles of the invitations awaiting the user's answer among the events.
    fn awaiting(events: &[Event]) -> Vec<String> {
        let settings = Settings::default();
        (invitations::awaiting(events, &connected(), &settings, Warsaw, thursday_9()).into_iter())
            .map(|i| events[i].title.clone())
            .collect()
    }

    /// The user's own answer as Google keeps it in the event.
    fn own_status(event: &Value) -> Value {
        (event["attendees"].as_array().unwrap().iter())
            .find(|attendee| attendee["email"] == EMAIL)
            .unwrap()["responseStatus"]
            .clone()
    }

    #[test]
    fn tc_06_06_accepting_board_from_the_invitations_list_sends_the_answer_to_google() {
        let (google, mut engine, start) = board("tc-06-06");
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let cached = now_shown(&engine);
        assert_eq!(awaiting(&cached), ["Board"]);
        let board = cached.iter().find(|e| e.title == "Board").unwrap();

        // 1. Accept, Maybe and Decline are offered; Accept takes it off the list at once.
        let offered: Vec<&str> = invitations::ANSWERS.iter().map(|(_, a)| *a).collect();
        assert_eq!(offered, ["Accept", "Maybe", "Decline"]);
        assert_eq!(
            invitations::own_answer(board),
            Some(guests::Response::Awaiting)
        );
        let Ok(edit) = invitations::answer(board, calendar, guests::Response::Accepted) else {
            panic!("a single event asks no scope");
        };
        let mut saving = Saving::default();
        let local = saving.change(board, edit.changed());
        assert!(awaiting(&saving.shown(&cached)).is_empty());
        let updates = engine.handle(Some(Job::Change(local, Box::new(edit))), start);

        // 2. Google received the answer, and lets the organizer know (`DEC-128`).
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "board").unwrap();
        assert_eq!(own_status(&kept), "accepted");
        assert_eq!(kept["attendees"][0]["responseStatus"], "accepted");
        let [patched] = writes(&google).try_into().unwrap();
        assert!(
            patched.ends_with("/events/board?sendUpdates=all"),
            "{patched}"
        );
        // Back through synchronization, answered.
        let synced = shown(&updates).unwrap();
        saving.changed(local, "");
        saving.synced(&synced);
        assert!(saving.is_empty());
        assert!(awaiting(&synced).is_empty());
        let board = synced.iter().find(|e| e.title == "Board").unwrap();
        assert_eq!(
            invitations::own_answer(board),
            Some(guests::Response::Accepted)
        );
    }

    /// "Team call" weekly on Mondays 10:00–10:30, 4 occurrences from 16 November, awaiting the
    /// user's answer (`TC-06-07`).
    fn team_call(name: &str) -> (Google, Engine, Instant) {
        let google = Google::start(name);
        personal(&google);
        let mut call = invitation(
            "teamcall",
            "Team call",
            "2026-11-16T10:00:00+01:00",
            "2026-11-16T10:30:00+01:00",
        );
        call["recurrence"] = json!(["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=4"]);
        google.fake.add_event(ACCOUNT_ID, EMAIL, call);
        let start = Instant::now();
        let engine = first_sync(&google, start);
        (google, engine, start)
    }

    /// The user's answer to each occurrence of "Team call", in order.
    fn team_call_answers(events: &[Event]) -> Vec<(String, guests::Response)> {
        let mut calls: Vec<(String, guests::Response)> = (events.iter())
            .filter(|e| e.title == "Team call")
            .map(|e| {
                let Timing::Timed { start, .. } = e.timing else {
                    panic!("a timed occurrence");
                };
                (local(start), invitations::own_answer(e).unwrap())
            })
            .collect();
        calls.sort_by(|a, b| a.0.cmp(&b.0));
        calls
    }

    /// The question answering the occurrence of "Team call" on the day asks.
    fn answering_team_call(
        engine: &Engine,
        date: chrono::NaiveDate,
        response: guests::Response,
    ) -> Question {
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let call = (now_shown(engine).into_iter())
            .find(|e| {
                e.title == "Team call"
                    && matches!(e.timing, Timing::Timed { start, .. }
                        if start.with_timezone(&Warsaw).date_naive() == date)
            })
            .unwrap();
        let Err(question) = invitations::answer(&call, calendar, response) else {
            panic!("an occurrence asks the scope first");
        };
        question
    }

    #[test]
    fn tc_06_07_declining_the_second_team_call_for_this_event_leaves_the_others_unanswered() {
        use guests::Response::{Awaiting, Declined};
        let (google, mut engine, start) = team_call("tc-06-07");

        // 1. Decline on the 2nd occurrence asks "this event or all events"; "this event".
        let question = answering_team_call(&engine, d(2026, 11, 23), Declined);
        let asked: Vec<&str> = (question.scopes().into_iter())
            .map(|(scope, _)| scope.label())
            .collect();
        assert_eq!(asked, ["This event", "All events"]);
        let pending = Pending::of(question, Scope::This);
        assert_eq!(pending.asks(), None);
        let Pending::Respond(edit, Scope::This) = pending else {
            panic!("an answer to this event");
        };
        let updates = engine.handle(Some(Job::Change(1, edit)), start);

        // 2. The 2nd is declined, in the app and in Google; the others still await an answer.
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        assert_eq!(
            team_call_answers(&shown(&updates).unwrap()),
            [
                ("2026-11-16 10:00".to_string(), Awaiting),
                ("2026-11-23 10:00".to_string(), Declined),
                ("2026-11-30 10:00".to_string(), Awaiting),
                ("2026-12-07 10:00".to_string(), Awaiting),
            ]
        );
        let second = (google.fake)
            .event(ACCOUNT_ID, EMAIL, "teamcall_20261123T090000Z")
            .unwrap();
        assert_eq!(own_status(&second), "declined");
        let series = google.fake.event(ACCOUNT_ID, EMAIL, "teamcall").unwrap();
        assert_eq!(own_status(&series), "needsAction");
    }

    #[test]
    fn accepting_an_occurrence_for_all_events_answers_the_series() {
        use guests::Response::Accepted;
        let (google, mut engine, start) = team_call("answer-all-events");

        let question = answering_team_call(&engine, d(2026, 11, 23), Accepted);
        let Pending::Respond(edit, scope) = Pending::of(question, Scope::All) else {
            panic!("an answer");
        };
        let updates = engine.handle(Some(Job::ChangeSeries(1, edit, scope)), start);

        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        let answers = team_call_answers(&shown(&updates).unwrap());
        assert_eq!(answers.len(), 4);
        assert!(answers.iter().all(|(_, answer)| *answer == Accepted));
        let [patched] = writes(&google).try_into().unwrap();
        assert!(
            patched.ends_with("/events/teamcall?sendUpdates=all"),
            "{patched}"
        );
    }

    #[test]
    fn tc_06_08_as_a_guest_board_cannot_be_dragged_or_edited_but_can_be_answered() {
        let (google, mut engine, start) = board("tc-06-08");
        let navigation = Navigation::start(None, d(2026, 11, 12), chrono::Weekday::Mon);
        let events = now_shown(&engine);
        let (Built::Grid(week), places) = build(
            &events,
            &Settings::default(),
            navigation,
            30,
            Warsaw,
            thursday_9(),
        ) else {
            panic!("the week view is built");
        };

        // 1. Dragged to another time, "Board" stays, and the app says only the organizer can
        // change it.
        let actions = mouse_drag(&week, grid_view::HOUR_HEIGHT * 2.0);
        let [grid_view::Action::Unmovable { event }] = actions.as_slice() else {
            panic!("the drag was not refused: {actions:?}");
        };
        let board = &events[places[*event]];
        assert_eq!(board.title, "Board");
        let mut notices = Vec::new();
        tell_unmovable(&mut notices, board);
        assert_eq!(
            notices,
            ["This event can't be changed: only its organizer can change it."]
        );
        assert!(google.fake.patched().is_empty());

        // 2. Opened, it offers Accept, Maybe and Decline, but no editing of its time, details or
        // guests.
        assert_eq!(board.protected(), Some(Protected::OrganizerOnly));
        assert_eq!(
            invitations::own_answer(board),
            Some(guests::Response::Awaiting)
        );
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let Ok(edit) = invitations::answer(board, calendar, guests::Response::Maybe) else {
            panic!("a single event asks no scope");
        };
        let updates = engine.handle(Some(Job::Change(1, Box::new(edit))), start);

        // Google takes the answer alone: the event itself stays as the organizer made it.
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(1, _)]
        ));
        let [(_, sent)] = google.fake.patched().try_into().unwrap();
        assert_eq!(sent.as_object().unwrap().len(), 1, "{sent}");
        let kept = google.fake.event(ACCOUNT_ID, EMAIL, "board").unwrap();
        assert_eq!(own_status(&kept), "tentative");
        assert_eq!(kept["summary"], "Board");
        assert_eq!(kept["start"]["dateTime"], "2026-11-12T14:00:00+01:00");
    }

    #[test]
    fn tc_06_09_declined_lunch_is_struck_through_and_the_switch_hides_and_shows_it() {
        let google = Google::start("tc-06-09");
        personal(&google);
        let mut lunch = invitation(
            "lunch",
            "Lunch",
            "2026-11-13T12:00:00+01:00",
            "2026-11-13T13:00:00+01:00",
        );
        lunch["attendees"][1]["responseStatus"] = json!("declined");
        google.fake.add_event(ACCOUNT_ID, EMAIL, lunch);
        let engine = first_sync(&google, Instant::now());
        let events = now_shown(&engine);
        let week = |settings: &Settings| {
            let navigation = Navigation::start(None, d(2026, 11, 12), chrono::Weekday::Mon);
            let (Built::Grid(week), _) =
                build(&events, settings, navigation, 30, Warsaw, thursday_9())
            else {
                panic!("the week view is built");
            };
            (week.blocks.iter())
                .map(|block| (block.title.clone().unwrap(), block.declined))
                .collect::<Vec<_>>()
        };

        // 1. In the week view, "Lunch" is shown struck through and faded.
        let mut settings = Settings::default();
        assert_eq!(week(&settings), [("Lunch".to_string(), true)]);

        // 2. The switch off, "Lunch" disappears; on again, it is shown again.
        settings.hide_declined = true;
        assert!(week(&settings).is_empty());
        settings.hide_declined = false;
        assert_eq!(week(&settings), [("Lunch".to_string(), true)]);
    }

    #[test]
    fn tc_06_10_adding_the_first_guest_turns_google_meet_on_and_the_saved_event_has_a_meet_link() {
        let google = Google::start("tc-06-10");
        personal(&google);
        google
            .fake
            .add_contact(ACCOUNT_ID, "Adam Kowalski", "adam@example.com");
        // "ewa@example.com" was invited to an earlier event.
        let mut earlier = event("earlier", "Planning");
        earlier["attendees"] = json!([
            {"email": EMAIL, "self": true, "organizer": true, "responseStatus": "accepted"},
            {"email": "ewa@example.com", "responseStatus": "accepted"}
        ]);
        google.fake.add_event(ACCOUNT_ID, EMAIL, earlier);
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let regional = polish();
        let calendars = engine.cache.calendar_list().unwrap();
        // A new event "Design review", from the next full hour, open in the editor.
        let default = create::default_calendar(&accounts(), &calendars, &Settings::default());
        let entry = QuickEntry {
            title: "Design review".to_string(),
            ..QuickEntry::new(
                create::keyed(d(2026, 11, 12), thursday_9(), 60, Warsaw),
                Warsaw,
                default,
            )
        };
        let mut editor = Editor::from_quick(&entry, &regional);
        let own = editor.own_addresses(&accounts());
        assert!(!editor.meet);

        // 1. Adam added: the "Add Google Meet" switch turns on.
        editor.guest_field = "Adam".to_string();
        let found = suggested(&mut engine, &editor, Reach::of(&accounts()[0].account));
        editor.add_guest(&found[0], &own);
        assert!(editor.meet);

        // 2. Saved: Google is asked to make a Meet call, and the event shows its link.
        let Ok(Save::Create(calendar, event)) = editor.save(&regional) else {
            panic!("a new event");
        };
        let updates = engine.handle(Some(Job::Create(1, calendar, *event)), start);
        let saved = saved(&updates);
        let [Saved::Created(1, id)] = saved.as_slice() else {
            panic!("Google did not create the event with its call: {saved:?}");
        };
        let sent = google.fake.event(ACCOUNT_ID, EMAIL, id).unwrap();
        assert_eq!(
            sent["conferenceData"]["createRequest"]["conferenceSolutionKey"]["type"],
            "hangoutsMeet"
        );
        assert!(
            (google.fake.requests().iter())
                .any(|r| r.starts_with("POST ") && r.contains("conferenceDataVersion=1"))
        );
        let review = (shown(&updates).unwrap().into_iter())
            .find(|e| e.title == "Design review")
            .unwrap();
        let link = event_details(&review, Warsaw).meet.unwrap();
        assert!(link.starts_with("https://meet.google.com/"), "{link}");
    }

    #[test]
    fn an_event_google_cannot_make_a_meet_call_for_is_saved_without_one_and_the_user_is_told() {
        let google = Google::start("no-meet");
        google.fake.add_calendar(
            ACCOUNT_ID,
            json!({
                "id": EMAIL,
                "summary": "Personal",
                "timeZone": "Europe/Warsaw",
                "backgroundColor": "#9fe1e7",
                "accessRole": "owner",
                "primary": true,
                "conferenceProperties": {"allowedConferenceSolutionTypes": ["eventHangout"]},
            }),
        );
        google.event(EMAIL, "standup", "Standup");
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let regional = polish();
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let entry = QuickEntry {
            title: "Design review".to_string(),
            ..QuickEntry::new(
                create::keyed(d(2026, 11, 12), thursday_9(), 60, Warsaw),
                Warsaw,
                Some(calendar.clone()),
            )
        };
        let mut editor = Editor::from_quick(&entry, &regional);
        editor.meet = true;
        let Ok(Save::Create(to, event)) = editor.save(&regional) else {
            panic!("a new event");
        };

        let created = engine.handle(Some(Job::Create(1, to, *event)), start);
        let standup = (now_shown(&engine).into_iter())
            .find(|e| e.id == "standup")
            .unwrap();
        let mut editor = Editor::of(&standup, calendar, &regional);
        editor.meet = true;
        let changed = engine.handle(Some(Job::Change(2, edit(&editor, &regional))), start);

        let saved = saved(&created);
        let [Saved::Created(1, id), Saved::WithoutMeet(1)] = saved.as_slice() else {
            panic!("saved without the call, and told: {saved:?}");
        };
        let review = google.fake.event(ACCOUNT_ID, EMAIL, id).unwrap();
        assert_eq!(review["summary"], "Design review");
        assert_eq!(review.get("conferenceData"), None);
        assert!(
            changed
                .iter()
                .any(|update| matches!(update, Update::WithoutMeet(2)))
        );
        assert!(matches!(
            saved_changes(&changed)[..],
            [Saved::Changed(2, _)]
        ));
    }

    /// A new event on Friday 13 November 2026, 14:30–15:00 in Warsaw, open in the editor with
    /// the people invited.
    fn availability_editor(engine: &Engine, people: &[(&str, &str)]) -> Editor {
        let calendar = engine.cache.calendar_list().unwrap().remove(0);
        let entry = QuickEntry::new(
            create::selected(d(2026, 11, 13), 870, 900, Warsaw),
            Warsaw,
            Some(calendar),
        );
        let mut editor = Editor::from_quick(&entry, &polish());
        for (name, email) in people {
            let person = Person {
                name: name.to_string(),
                email: email.to_string(),
            };
            editor.add_guest(&person, &[]);
        }
        editor
    }

    /// The preview the editor shows, once the engine has asked Google for the guests' day.
    fn previewed(engine: &mut Engine, editor: &Editor) -> Option<Preview> {
        let (asked, chosen) = availability::asked(editor, &polish(), true).unwrap();
        let updates = engine.handle(Some(Job::Busy(asked.clone())), Instant::now());
        let [Update::Busy(answered, answer)] = updates.as_slice() else {
            panic!("the guests' busy times");
        };
        assert_eq!(*answered, asked);
        let answer = answer.as_ref()?;
        Some(availability::preview(
            &asked,
            chosen,
            &editor.guests,
            None,
            answer,
        ))
    }

    #[test]
    fn tc_06_12_the_editor_shows_each_guests_busy_blocks_for_the_day_and_marks_a_conflict() {
        let google = Google::start("tc-06-12");
        personal(&google);
        // Anna is busy on 13 November 14:00–15:00; Adam has no busy time that day.
        google.fake.share_busy(
            "anna.nowak@example.com",
            &[("2026-11-13T14:00:00+01:00", "2026-11-13T15:00:00+01:00")],
        );
        google.fake.share_busy("adam@example.com", &[]);
        let mut engine = first_sync(&google, Instant::now());

        // 1. A new event on 13 November 14:30–15:00 with Anna and Adam: a strip per guest with
        //    the day's busy blocks, 14:00–15:00 for Anna and none for Adam; the chosen time
        //    highlighted.
        let editor = availability_editor(
            &engine,
            &[
                ("Anna Nowak", "anna.nowak@example.com"),
                ("Adam", "adam@example.com"),
            ],
        );
        let preview = previewed(&mut engine, &editor).unwrap();

        let [anna, adam] = preview.strips.as_slice() else {
            panic!("a strip per guest");
        };
        assert_eq!(anna.label, "Anna Nowak");
        assert_eq!(anna.busy, Some(vec![(14.0 / 24.0, 15.0 / 24.0)]));
        assert_eq!(adam.label, "Adam");
        assert_eq!(adam.busy, Some(vec![]));
        assert_eq!(preview.chosen, (14.5 / 24.0, 15.0 / 24.0));
        let [query] = google.fake.busy_queries().try_into().unwrap();
        assert_eq!(query["timeMin"], "2026-11-12T23:00:00Z");
        assert_eq!(query["timeMax"], "2026-11-13T23:00:00Z");

        // 2. Anna is marked as a conflict; Adam is not.
        assert!(anna.conflict);
        assert!(!adam.conflict);
    }

    #[test]
    fn a_guest_who_does_not_share_their_availability_shows_no_data() {
        let google = Google::start("no-data");
        personal(&google);
        let mut engine = first_sync(&google, Instant::now());
        let editor = availability_editor(&engine, &[("", "private@example.com")]);

        let preview = previewed(&mut engine, &editor).unwrap();

        let [strip] = preview.strips.as_slice() else {
            panic!("a strip");
        };
        assert_eq!(strip.busy, None);
        assert!(!strip.conflict);
    }

    #[test]
    fn without_network_the_preview_is_not_shown() {
        let google = Google::start("busy-offline");
        personal(&google);
        let mut engine = first_sync(&google, Instant::now());
        let editor = availability_editor(&engine, &[("", "adam@example.com")]);

        // Offline, nothing is asked; with Google out of reach, nothing comes to show.
        assert_eq!(availability::asked(&editor, &polish(), false), None);
        google.fake.disconnect();
        assert_eq!(previewed(&mut engine, &editor), None);
    }

    #[test]
    fn the_guests_day_is_asked_once_and_again_when_the_editor_asks_anew() {
        let google = Google::start("busy-asked");
        personal(&google);
        google.fake.share_busy("adam@example.com", &[]);
        let ctx = egui::Context::default();
        let ids = vec![ACCOUNT_ID.to_string()];
        let mut sync = Synchronization::with(
            &ctx,
            Cache::in_memory,
            google.sign_in(),
            ids,
            window(),
            Disabled::new(),
        );
        next_events(&mut sync);
        sync.checked(&[(ACCOUNT_ID.to_string(), Check::Works)], window());
        next_events(&mut sync);
        let calendar = sync.calendars()[0].clone();
        let entry = QuickEntry::new(
            create::selected(d(2026, 11, 13), 870, 900, Warsaw),
            Warsaw,
            Some(calendar),
        );
        let mut editor = Editor::from_quick(&entry, &polish());
        let adam = Person {
            name: String::new(),
            email: "adam@example.com".to_string(),
        };
        editor.add_guest(&adam, &[]);
        let (asked, _) = availability::asked(&editor, &polish(), true).unwrap();
        let answered = |sync: &mut Synchronization| {
            let deadline = Instant::now() + Duration::from_secs(10);
            while sync.busy(&asked).is_none() {
                assert!(Instant::now() < deadline, "no answer came");
                std::thread::sleep(Duration::from_millis(5));
                sync.poll();
            }
        };

        // Asked every frame: once.
        sync.ask_busy(Some(asked.clone()));
        sync.ask_busy(Some(asked.clone()));
        answered(&mut sync);
        assert_eq!(google.fake.busy_queries().len(), 1);

        // The editor closed and opened again: asked again, the last answer shown meanwhile.
        sync.ask_busy(None);
        sync.ask_busy(Some(asked.clone()));
        assert!(sync.busy(&asked).is_some());
        let deadline = Instant::now() + Duration::from_secs(10);
        while google.fake.busy_queries().len() < 2 {
            assert!(Instant::now() < deadline, "not asked again");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// The desktop's notifications, faked: what was shown, and what choosing on each does; with
    /// buttons unless said otherwise.
    #[derive(Clone, Default)]
    struct FakeNotifier {
        shown: Arc<Mutex<Vec<Shown>>>,
        without_buttons: bool,
    }

    type Shown = (Notification, Box<dyn Fn(Action) + Send + Sync>);

    impl Notifier for FakeNotifier {
        fn show(&self, notification: &Notification, chosen: Box<dyn Fn(Action) + Send + Sync>) {
            let shown = (notification.clone(), chosen);
            self.shown.lock().unwrap().push(shown);
        }

        fn buttons(&self) -> bool {
            !self.without_buttons
        }
    }

    impl FakeNotifier {
        /// Title, start time and location of each notification shown, in order.
        fn shown(&self) -> Vec<(String, String, String)> {
            (self.shown.lock().unwrap().iter())
                .map(|(n, _)| (n.title.clone(), n.when.clone(), n.location.clone()))
                .collect()
        }

        /// The buttons of the notification shown at `index`.
        fn actions(&self, index: usize) -> Vec<Action> {
            self.shown.lock().unwrap()[index].0.actions()
        }

        fn choose(&self, index: usize, action: Action) {
            (self.shown.lock().unwrap()[index].1)(action);
        }
    }

    /// Thursday 12 November 2026 at the time in Warsaw.
    fn thursday(h: u32, m: u32) -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(2026, 11, 12, h, m, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    /// The account's calendar "Work", as Google lists it.
    fn work_calendar(google: &Google, id: &str, name: &str) {
        google.fake.add_calendar(
            ACCOUNT_ID,
            json!({
                "id": id,
                "summary": name,
                "timeZone": "Europe/Warsaw",
                "backgroundColor": "#9fe1e7",
                "accessRole": "owner",
            }),
        );
    }

    /// An event on Thursday 12 November from 10:00 to 11:00, with its reminders.
    fn at_ten(id: &str, title: &str, overrides: Value) -> Value {
        json!({
            "id": id,
            "status": "confirmed",
            "summary": title,
            "start": {"dateTime": "2026-11-12T10:00:00+01:00", "timeZone": "Europe/Warsaw"},
            "end": {"dateTime": "2026-11-12T11:00:00+01:00", "timeZone": "Europe/Warsaw"},
            "reminders": {"useDefault": false, "overrides": overrides},
        })
    }

    /// The app started at 9:00 on Thursday 12 November with the clock that the test moves,
    /// reminders shown through the fake notifications, and the account's first synchronization
    /// done; with what the user chose on the notifications.
    fn reminding(
        google: &Google,
        clock: &Arc<Mutex<DateTime<Utc>>>,
        notifier: &FakeNotifier,
        disabled: Disabled,
        start: Instant,
    ) -> (Engine, Receiver<Chosen>) {
        *clock.lock().unwrap() = thursday(9, 0);
        let mut engine = Engine::new(Cache::in_memory(), google.sign_in(), window(), disabled);
        let time = clock.clone();
        engine.clock = Box::new(move || *time.lock().unwrap());
        engine.clear_disabled();
        let (acts, chosen) = mpsc::channel();
        let act: Acts = Arc::new(move |choice| acts.send(choice).unwrap());
        let regional = Regional::from_env(|var| (var == "LANG").then(|| "en_GB.UTF-8".into()));
        let job = Job::Remind(Box::new(notifier.clone()), regional, act);
        engine.handle(Some(job), start);
        let updates = engine.handle(Some(Job::Sync(ACCOUNT_ID.into(), window())), start);
        assert!(told(updates).is_some());
        (engine, chosen)
    }

    #[test]
    fn tc_07_01_a_10_minute_reminder_gives_exactly_one_notification_across_synchronizations() {
        let google = Google::start("tc-07-01");
        work_calendar(&google, "work", "Work");
        let mut review = at_ten(
            "review",
            "Review",
            json!([{"method": "popup", "minutes": 10}]),
        );
        review["location"] = json!("Room 2");
        google.fake.add_event(ACCOUNT_ID, "work", review.clone());
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let start = Instant::now();
        let (mut engine, _) = reminding(&google, &clock, &notifier, Disabled::new(), start);
        engine.remind();
        assert_eq!(notifier.shown(), []);

        // 1. At 9:50 one notification is shown, with "Review", 10:00 and "Room 2".
        *clock.lock().unwrap() = thursday(9, 50);
        engine.remind();
        let review_shown = ("Review".into(), "10:00".into(), "Room 2".into());
        assert_eq!(notifier.shown(), std::slice::from_ref(&review_shown));

        // 2. Three more synchronizations, one bringing the event changed in Google, and 9:55: no
        //    further notification.
        review["description"] = json!("Q4 numbers");
        google.fake.add_event(ACCOUNT_ID, "work", review);
        for round in 1..=3 {
            let updates = engine.handle(Some(Job::Refresh), start + secs(60 * round));
            if round == 1 {
                assert!(shown(&updates).is_some());
            }
            engine.remind();
        }
        *clock.lock().unwrap() = thursday(9, 55);
        engine.remind();
        assert_eq!(notifier.shown(), [review_shown]);
    }

    #[test]
    fn tc_07_05_a_new_event_gets_its_calendars_default_reminders_an_added_one_reaches_google_and_use_calendar_defaults_resets()
     {
        let google = Google::start("tc-07-05");
        google.fake.add_calendar(
            ACCOUNT_ID,
            json!({
                "id": "work",
                "summary": "Work",
                "timeZone": "Europe/Warsaw",
                "backgroundColor": "#9fe1e7",
                "accessRole": "owner",
                "defaultReminders": [{"method": "popup", "minutes": 15}],
            }),
        );
        let start = Instant::now();
        let mut engine = first_sync(&google, start);
        let regional = polish();
        let work = engine.cache.calendar_list().unwrap().remove(0);
        let fifteen = Reminder {
            method: Method::Notification,
            minutes: 15,
        };
        assert_eq!(work.default_reminders, [fifteen]);

        // 1. "Plan" created in "Work" uses the calendar's defaults: a notification 15 minutes
        //    before.
        let entry = QuickEntry::new(
            create::clicked(d(2026, 11, 12), 600, 60, Warsaw),
            Warsaw,
            Some(work.clone()),
        );
        let mut editor = Editor::from_quick(&entry, &regional);
        editor.title = "Plan".to_string();
        assert_eq!(editor.reminder_fields(), [ReminderField::of(fifteen)]);
        let Ok(Save::Create(calendar, event)) = editor.save(&regional) else {
            panic!("a new event");
        };
        let updates = engine.handle(Some(Job::Create(1, calendar, *event)), start);
        let saved = saved(&updates);
        let [Saved::Created(1, id)] = saved.as_slice() else {
            panic!("Google did not create the event");
        };
        let id = id.clone();
        let kept = google.fake.event(ACCOUNT_ID, "work", &id).unwrap();
        assert_eq!(kept.get("reminders"), None);
        let plan = (shown(&updates).unwrap().into_iter())
            .find(|e| e.id == id)
            .unwrap();
        assert_eq!(plan.about.reminders, Reminders::Default);

        // 2. An e-mail reminder 1 day before added in the editor and saved.
        let mut editor = Editor::of(&plan, work.clone(), &regional);
        let mut fields = editor.reminder_fields();
        fields.push(ReminderField {
            method: Method::Email,
            amount: 1,
            unit: Unit::Days,
        });
        editor.reminders = Some(fields);
        let updates = engine.handle(Some(Job::Change(2, edit(&editor, &regional))), start);
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(2, _)]
        ));
        let kept = google.fake.event(ACCOUNT_ID, "work", &id).unwrap();
        assert_eq!(
            kept["reminders"],
            json!({"useDefault": false, "overrides": [
                {"method": "popup", "minutes": 15},
                {"method": "email", "minutes": 1440},
            ]})
        );

        // 3. "Use calendar defaults" and saved.
        let plan = (shown(&updates).unwrap().into_iter())
            .find(|e| e.id == id)
            .unwrap();
        let mut editor = Editor::of(&plan, work, &regional);
        assert!(editor.reminders.is_some());
        editor.reminders = None;
        let updates = engine.handle(Some(Job::Change(3, edit(&editor, &regional))), start);
        assert!(matches!(
            saved_changes(&updates).as_slice(),
            [Saved::Changed(3, _)]
        ));
        let kept = google.fake.event(ACCOUNT_ID, "work", &id).unwrap();
        assert_eq!(kept["reminders"]["useDefault"], true);
        assert_eq!(kept["reminders"]["overrides"], json!([]));
        let plan = (shown(&updates).unwrap().into_iter())
            .find(|e| e.id == id)
            .unwrap();
        assert_eq!(plan.about.reminders, Reminders::Default);
    }

    #[test]
    fn tc_07_02_declined_events_and_e_mail_reminders_give_no_desktop_notification() {
        let google = Google::start("tc-07-02");
        work_calendar(&google, "work", "Work");
        let mut offsite = at_ten(
            "offsite",
            "Offsite",
            json!([{"method": "popup", "minutes": 10}]),
        );
        offsite["organizer"] = json!({"email": "adam@example.com"});
        offsite["attendees"] = json!([
            {"email": "adam@example.com", "organizer": true, "responseStatus": "accepted"},
            {"email": EMAIL, "self": true, "responseStatus": "declined"}
        ]);
        google.fake.add_event(ACCOUNT_ID, "work", offsite);
        let report = at_ten(
            "report",
            "Report",
            json!([{"method": "email", "minutes": 10}]),
        );
        google.fake.add_event(ACCOUNT_ID, "work", report);
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let (mut engine, _) =
            reminding(&google, &clock, &notifier, Disabled::new(), Instant::now());

        // 1. At 9:55 nothing was shown for "Offsite" or "Report".
        for minute in 50..=55 {
            *clock.lock().unwrap() = thursday(9, minute);
            engine.remind();
        }
        assert_eq!(notifier.shown(), []);
    }

    #[test]
    fn events_of_calendars_disabled_in_the_settings_give_no_notifications() {
        let google = Google::start("reminders-disabled");
        work_calendar(&google, "work", "Work");
        work_calendar(&google, "team", "Team");
        work_calendar(&google, "side", "Side project");
        let reminder = json!([{"method": "popup", "minutes": 10}]);
        for (calendar, title) in [("work", "Review"), ("team", "Retro"), ("side", "Pairing")] {
            let event = at_ten(&title.to_lowercase(), title, reminder.clone());
            google.fake.add_event(ACCOUNT_ID, calendar, event);
        }
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let start = Instant::now();
        let disabled = Disabled::from([(ACCOUNT_ID.to_string(), "team".to_string())]);
        let (mut engine, _) = reminding(&google, &clock, &notifier, disabled.clone(), start);

        // "Side project" is disabled too while the app runs.
        let mut more = disabled;
        more.insert((ACCOUNT_ID.to_string(), "side".to_string()));
        engine.handle(Some(Job::Disabled(more)), start);
        *clock.lock().unwrap() = thursday(9, 50);
        engine.remind();

        let titles: Vec<String> = notifier.shown().into_iter().map(|(t, _, _)| t).collect();
        assert_eq!(titles, ["Review"]);
    }

    /// Google's calendars were only ever read, and the event's reminders there are as they were.
    fn reminders_unchanged_in_google(google: &Google, calendar: &str, event: &Value) {
        let requests = google.fake.requests();
        let calendar_api: Vec<&String> = (requests.iter())
            .filter(|request| request.contains(" /calendar/v3/"))
            .collect();
        assert!(!calendar_api.is_empty());
        assert!(
            calendar_api
                .iter()
                .all(|request| request.starts_with("GET "))
        );
        let id = event["id"].as_str().unwrap();
        let now = google.fake.event(ACCOUNT_ID, calendar, id).unwrap();
        assert_eq!(now["reminders"], event["reminders"]);
    }

    #[test]
    fn tc_07_04_a_muted_calendar_still_shows_its_events_but_gives_no_notifications() {
        let google = Google::start("tc-07-04");
        work_calendar(&google, "work", "Work");
        work_calendar(&google, "team", "Team");
        let reminder = json!([{"method": "popup", "minutes": 10}]);
        let review = at_ten("review", "Review", reminder.clone());
        let sync = at_ten("sync", "Sync", reminder);
        google.fake.add_event(ACCOUNT_ID, "work", review);
        google.fake.add_event(ACCOUNT_ID, "team", sync.clone());
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let start = Instant::now();
        let (mut engine, _) = reminding(&google, &clock, &notifier, Disabled::new(), start);
        let Update::Events(calendars, events) = engine.shown() else {
            panic!("the views are told what to show");
        };
        let team = (calendars.iter())
            .find(|calendar| calendar.id == "team")
            .unwrap();

        // 1. "Team" muted from its side-panel menu: "Sync" is still in the week view.
        let mut settings = Settings::default();
        side_panel::toggle(&mut settings.muted, team);
        let settings = Settings::parse(&settings.to_text());
        *engine.silenced.lock().unwrap() = Silenced::of(&settings);
        let navigation = Navigation::start(None, d(2026, 11, 12), chrono::Weekday::Mon);
        let (Built::Grid(week), _) =
            build(&events, &settings, navigation, 30, Warsaw, thursday(9, 0))
        else {
            panic!("the week view is built");
        };
        let mut titles: Vec<String> = (week.blocks.iter())
            .filter_map(|block| block.title.clone())
            .collect();
        titles.sort();
        assert_eq!(titles, ["Review", "Sync"]);

        // 2. Up to 9:55, no notification of "Sync"; "Work"'s "Review" is still shown.
        for minute in 50..=55 {
            *clock.lock().unwrap() = thursday(9, minute);
            engine.remind();
        }
        let shown: Vec<String> = notifier.shown().into_iter().map(|(t, _, _)| t).collect();
        assert_eq!(shown, ["Review"]);
        reminders_unchanged_in_google(&google, "team", &sync);
    }

    #[test]
    fn notifications_turned_off_show_none_and_turned_on_again_those_still_to_come() {
        let google = Google::start("reminders-off");
        work_calendar(&google, "work", "Work");
        let review = at_ten(
            "review",
            "Review",
            json!([{"method": "popup", "minutes": 10}]),
        );
        let retro = at_ten("retro", "Retro", json!([{"method": "popup", "minutes": 5}]));
        google.fake.add_event(ACCOUNT_ID, "work", review.clone());
        google.fake.add_event(ACCOUNT_ID, "work", retro);
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let start = Instant::now();
        let (mut engine, _) = reminding(&google, &clock, &notifier, Disabled::new(), start);

        // Off in the settings: "Review"'s reminder at 9:50 shows nothing, even after a
        // synchronization.
        let mut settings = Settings {
            notifications_off: true,
            ..Settings::default()
        };
        *engine.silenced.lock().unwrap() = Silenced::of(&settings);
        *clock.lock().unwrap() = thursday(9, 50);
        engine.remind();
        engine.handle(Some(Job::Refresh), start + secs(60));
        engine.remind();
        assert_eq!(notifier.shown(), []);

        // On again at 9:52: "Retro"'s reminder at 9:55 is shown, "Review"'s past one is not.
        settings.notifications_off = false;
        *engine.silenced.lock().unwrap() = Silenced::of(&settings);
        for minute in 52..=55 {
            *clock.lock().unwrap() = thursday(9, minute);
            engine.remind();
        }
        let shown: Vec<String> = notifier.shown().into_iter().map(|(t, _, _)| t).collect();
        assert_eq!(shown, ["Retro"]);
        reminders_unchanged_in_google(&google, "work", &review);
    }

    #[test]
    fn clicking_a_notification_opens_its_event() {
        let google = Google::start("reminders-click");
        work_calendar(&google, "work", "Work");
        let review = at_ten(
            "review",
            "Review",
            json!([{"method": "popup", "minutes": 10}]),
        );
        google.fake.add_event(ACCOUNT_ID, "work", review);
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let (mut engine, chosen) =
            reminding(&google, &clock, &notifier, Disabled::new(), Instant::now());
        *clock.lock().unwrap() = thursday(9, 50);
        engine.remind();

        notifier.choose(0, Action::Open);

        assert_eq!(
            chosen.try_recv(),
            Ok(Chosen::Open(Opened {
                event: EventId {
                    account_id: ACCOUNT_ID.to_string(),
                    calendar_id: "work".to_string(),
                    id: "review".to_string(),
                },
                snooze: None,
            }))
        );
    }

    /// The "Review" event at 10:00 with a 10-minute reminder, shown at 9:50 through the fake
    /// notifications; with the engine, what the user chose, and when it started.
    fn review_shown(
        google: &Google,
        clock: &Arc<Mutex<DateTime<Utc>>>,
        notifier: &FakeNotifier,
        meet: bool,
    ) -> (Engine, Receiver<Chosen>, Instant) {
        work_calendar(google, "work", "Work");
        let mut review = at_ten(
            "review",
            "Review",
            json!([{"method": "popup", "minutes": 10}]),
        );
        if meet {
            review["conferenceData"] = json!({
                "conferenceSolution": {"key": {"type": "hangoutsMeet"}},
                "entryPoints": [
                    {"entryPointType": "video", "uri": "https://meet.google.com/abc-defg-hij"}
                ],
            });
        }
        google.fake.add_event(ACCOUNT_ID, "work", review);
        let start = Instant::now();
        let (mut engine, chosen) = reminding(google, clock, notifier, Disabled::new(), start);
        *clock.lock().unwrap() = thursday(9, 50);
        engine.remind();
        assert_eq!(notifier.shown().len(), 1);
        (engine, chosen, start)
    }

    #[test]
    fn tc_07_03_snoozing_for_5_minutes_shows_the_notification_again_without_changing_google() {
        let google = Google::start("tc-07-03");
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let (mut engine, chosen, start) = review_shown(&google, &clock, &notifier, false);
        let before = google.fake.requests().len();

        // 1. Dismiss, Snooze 5 minutes and Snooze 10 minutes are offered.
        assert_eq!(
            notifier.actions(0),
            [Action::Snooze(5), Action::Snooze(10), Action::Dismiss]
        );

        // 2. Snooze 5 minutes, and the clock at 9:55: "Review" is shown again.
        notifier.choose(0, Action::Snooze(5));
        let Ok(Chosen::Snooze(reminder, 5)) = chosen.try_recv() else {
            panic!("Snooze 5 minutes was not chosen");
        };
        assert!(
            engine
                .handle(Some(Job::Snooze(reminder, 5, thursday(9, 50))), start)
                .is_empty()
        );
        assert_eq!(engine.next_reminder(start), Some(start + secs(60)));
        *clock.lock().unwrap() = thursday(9, 54) + chrono::Duration::seconds(30);
        assert_eq!(engine.next_reminder(start), Some(start + secs(30)));
        engine.remind();
        assert_eq!(notifier.shown().len(), 1);
        *clock.lock().unwrap() = thursday(9, 55);
        engine.remind();
        let review = ("Review".to_string(), "10:00".to_string(), String::new());
        assert_eq!(notifier.shown(), [review.clone(), review]);

        // 3. Google was told nothing.
        assert_eq!(google.fake.requests().len(), before);
        assert!(google.fake.patched().is_empty());
    }

    #[test]
    fn a_snoozed_notification_of_a_calendar_muted_meanwhile_does_not_come_back() {
        let google = Google::start("reminders-snooze-muted");
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let (mut engine, chosen, start) = review_shown(&google, &clock, &notifier, false);
        notifier.choose(0, Action::Snooze(5));
        let Ok(Chosen::Snooze(reminder, 5)) = chosen.try_recv() else {
            panic!("Snooze 5 minutes was not chosen");
        };
        engine.handle(Some(Job::Snooze(reminder, 5, thursday(9, 50))), start);

        // "Work" muted before the snooze is due at 9:55, and unmuted after it: it is gone.
        let muted = Silenced {
            all: false,
            muted: [(ACCOUNT_ID.to_string(), "work".to_string())].into(),
        };
        *engine.silenced.lock().unwrap() = muted;
        *clock.lock().unwrap() = thursday(9, 55);
        engine.remind();
        *engine.silenced.lock().unwrap() = Silenced::default();
        *clock.lock().unwrap() = thursday(10, 0);
        engine.remind();
        assert_eq!(notifier.shown().len(), 1);
    }

    #[test]
    fn dismissing_a_notification_closes_it_for_good_and_tells_google_nothing() {
        let google = Google::start("reminders-dismiss");
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let (mut engine, chosen, _) = review_shown(&google, &clock, &notifier, false);
        let before = google.fake.requests().len();

        notifier.choose(0, Action::Dismiss);

        assert!(chosen.try_recv().is_err());
        *clock.lock().unwrap() = thursday(10, 5);
        engine.remind();
        assert_eq!(notifier.shown().len(), 1);
        assert_eq!(google.fake.requests().len(), before);
    }

    #[test]
    fn snoozing_for_10_minutes_shows_the_notification_again_after_10_minutes() {
        let google = Google::start("reminders-snooze-10");
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let (mut engine, chosen, start) = review_shown(&google, &clock, &notifier, false);
        let before = google.fake.requests().len();

        notifier.choose(0, Action::Snooze(10));
        let Ok(Chosen::Snooze(reminder, 10)) = chosen.try_recv() else {
            panic!("Snooze 10 minutes was not chosen");
        };
        // Counted from the choice at 9:59, though the worker got to it later.
        *clock.lock().unwrap() = thursday(10, 1);
        engine.handle(Some(Job::Snooze(reminder, 10, thursday(9, 59))), start);
        *clock.lock().unwrap() = thursday(10, 8);
        engine.remind();
        assert_eq!(notifier.shown().len(), 1);
        *clock.lock().unwrap() = thursday(10, 9);
        engine.remind();
        assert_eq!(notifier.shown().len(), 2);
        assert_eq!(google.fake.requests().len(), before);
    }

    #[test]
    fn a_notification_of_an_event_with_google_meet_joins_the_meeting_from_its_join_button() {
        let google = Google::start("reminders-join");
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let (_engine, chosen, _) = review_shown(&google, &clock, &notifier, true);

        assert_eq!(
            notifier.actions(0),
            [
                Action::Join,
                Action::Snooze(5),
                Action::Snooze(10),
                Action::Dismiss
            ]
        );
        notifier.choose(0, Action::Join);

        let link = "https://meet.google.com/abc-defg-hij".to_string();
        assert_eq!(chosen.try_recv(), Ok(Chosen::Join(link)));
    }

    #[test]
    fn without_buttons_clicking_a_notification_opens_its_event_with_snooze() {
        let google = Google::start("reminders-no-buttons");
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier {
            without_buttons: true,
            ..FakeNotifier::default()
        };
        let (_engine, chosen, _) = review_shown(&google, &clock, &notifier, true);

        notifier.choose(0, Action::Open);

        let Ok(Chosen::Open(opened)) = chosen.try_recv() else {
            panic!("the event was not opened");
        };
        assert_eq!(opened.event.id, "review");
        let shown = notifier.shown.lock().unwrap()[0].0.reminder.clone();
        assert_eq!(opened.snooze, Some(shown));
    }

    #[test]
    fn the_worker_wakes_for_the_next_reminder_or_within_a_minute_to_follow_the_clock() {
        let google = Google::start("reminders-wake");
        work_calendar(&google, "work", "Work");
        let review = at_ten(
            "review",
            "Review",
            json!([{"method": "popup", "minutes": 10}]),
        );
        google.fake.add_event(ACCOUNT_ID, "work", review);
        let clock = Arc::new(Mutex::new(thursday(9, 0)));
        let notifier = FakeNotifier::default();
        let now = Instant::now();
        let (engine, _) = reminding(&google, &clock, &notifier, Disabled::new(), now);

        assert_eq!(engine.next_reminder(now), Some(now + secs(60)));
        *clock.lock().unwrap() = thursday(9, 49) + chrono::Duration::seconds(30);
        assert_eq!(engine.next_reminder(now), Some(now + secs(30)));
    }
}
