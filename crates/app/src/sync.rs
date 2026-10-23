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
//! its first synchronization.

use std::collections::hash_map::RandomState;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::BuildHasher;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Instant;

use calendar_core::accounts::Check;
use calendar_core::cache::{self, Cache};
use calendar_core::create::{DEFAULT_LENGTH, NewEvent};
use calendar_core::editor::Edit;
use calendar_core::model::{Calendar, Event};
use calendar_core::platform::{SecretStore, Wakeups};
use calendar_core::schedule::Schedule;
use calendar_core::status::{Problem, Status};
use calendar_core::sync::{
    Provider, SyncError, Window, fetch_events, shown_events, sync_calendar_list, sync_calendars,
};
use calendar_google::{Endpoints, Session};
use chrono::{DateTime, Utc};
use eframe::egui;

use crate::platform::{SystemSecretStore, SystemWakeups};

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
    /// An event to delete from the calendar, by the number the views know it by.
    Delete(u64, Calendar, Event),
    /// An event deleted just now to bring back to the calendar, by the number the views know it
    /// by.
    Restore(u64, Calendar, Event),
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
}

/// What became of a new, changed or deleted event at Google.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Saved {
    Created(u64, String),
    /// Why it was not created.
    NotCreated(u64, String),
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
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let send = |update| {
                let _ = sender.send(update);
                ctx.request_repaint();
            };
            let mut engine = Engine::new(open(), sign_in, window, disabled);
            let _ = engine.cache.retain_accounts(&account_ids);
            engine.clear_disabled();
            send(engine.shown());
            send(Update::Status(engine.status()));
            loop {
                let job = match engine.schedule.next() {
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
                let mut jobs = vec![job];
                jobs.extend(received.try_iter().map(Some));
                if jobs.iter().any(synchronizes) {
                    send(Update::Synchronizing);
                }
                for job in latest_browsing(jobs) {
                    for update in engine.handle(job, Instant::now()) {
                        send(update);
                    }
                }
                send(Update::Status(engine.status()));
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
        }
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
    /// (`MOD-05-F12`); `poll` tells what became of it by `local`.
    pub fn delete(&self, local: u64, calendar: Calendar, event: Event) {
        let _ = self.jobs.send(Job::Delete(local, calendar, event));
    }

    /// Brings back the event deleted just now at Google, then synchronizes the calendar
    /// (`MOD-05-F27`); `poll` tells what became of it by `local`, as of a change.
    pub fn restore(&self, local: u64, calendar: Calendar, event: Event) {
        let _ = self.jobs.send(Job::Restore(local, calendar, event));
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
}

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
            cache,
        }
    }

    /// Does the job, or with none the polling round that is due; returns what the views are to
    /// be told.
    fn handle(&mut self, job: Option<Job>, now: Instant) -> Vec<Update> {
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
            Some(Job::Delete(local, calendar, event)) => {
                return self.delete(local, calendar, &event, now);
            }
            Some(Job::Restore(local, calendar, event)) => {
                return self.restore(local, calendar, &event, now);
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
                let mut updates = vec![Update::Created(local, id)];
                updates.extend(self.synchronize(&[calendar], now));
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
            changed = session.update(calendar, id, &edit.event.etag, &edit.patch);
        }
        let mut updates = vec![match (changed, &edit.to) {
            (Err(error), _) => Update::NotChanged(local, refusal(error)),
            (Ok(etag), None) => Update::Changed(local, etag),
            (Ok(_), Some(to)) => match session.move_to(calendar, id, to) {
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
        updates
    }

    /// Deletes the event at Google, against the version shown, then synchronizes its calendar,
    /// so the deletion reaches the cache the way every change does (`MOD-05-F16`).
    fn delete(
        &mut self,
        local: u64,
        calendar: Calendar,
        event: &Event,
        now: Instant,
    ) -> Vec<Update> {
        let Some(session) = self.sessions.get(&calendar.account_id) else {
            return vec![Update::NotDeleted(local, NOT_SIGNED_IN.to_string())];
        };
        let mut updates = vec![match session.delete(&calendar, &event.id, &event.etag) {
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
        now: Instant,
    ) -> Vec<Update> {
        let Some(session) = self.sessions.get(&calendar.account_id) else {
            return vec![Update::NotChanged(local, NOT_SIGNED_IN.to_string())];
        };
        let mut updates = vec![match session.restore(&calendar, &event.id) {
            Ok(etag) => Update::Changed(local, etag),
            Err(error) => Update::NotChanged(local, refusal(error)),
        }];
        updates.extend(self.synchronize(&[calendar], now));
        updates
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
}

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

/// The jobs waiting, with only the last of the browsing among them: days left behind while the
/// worker was busy are not fetched.
fn latest_browsing(jobs: Vec<Option<Job>>) -> Vec<Option<Job>> {
    let last = jobs
        .iter()
        .rposition(|job| matches!(job, Some(Job::Browse(_))));
    jobs.into_iter()
        .enumerate()
        .filter(|(i, job)| !matches!(job, Some(Job::Browse(_))) || Some(*i) == last)
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
        Some(Job::Stop(_) | Job::Unconfirmed(_) | Job::Shown(_) | Job::Window(_))
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
    use calendar_core::colors;
    use calendar_core::create::{self, QuickEntry, Saving};
    use calendar_core::drag::{self, Target};
    use calendar_core::editor::{Editor, Save};
    use calendar_core::fixtures::d;
    use calendar_core::model::{Access, Calendar, Protected, Rgb, Show, Timing, Visibility};
    use calendar_core::navigation::Navigation;
    use calendar_core::offline;
    use calendar_core::platform::Network;
    use calendar_core::regional::Regional;
    use calendar_core::settings::Settings;
    use calendar_core::status::{self, State, label};
    use calendar_core::sync::SyncedEvent;
    use calendar_core::undo::{self, LastChange, Undo};
    use calendar_google::CLIENT_FILE;
    use calendar_google::fake::{ACCOUNT_ID, CLIENT_ID, EMAIL, FakeGoogle, FakeStore};
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
    fn of_the_browsing_waiting_only_the_last_is_done() {
        let week = |day| {
            Some(
                window()
                    .outside(d(2030, 3, day), d(2030, 3, day + 6))
                    .unwrap(),
            )
        };
        let jobs = vec![
            Some(Job::Browse(week(4))),
            None,
            Some(Job::Browse(week(11))),
            Some(Job::Refresh),
            Some(Job::Browse(week(18))),
        ];

        let done = latest_browsing(jobs);

        assert_eq!(done.len(), 3);
        assert!(done[0].is_none());
        assert!(matches!(done[1], Some(Job::Refresh)));
        assert!(matches!(&done[2], Some(Job::Browse(span)) if *span == week(18)));
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
        let updates = engine.handle(Some(Job::Create(1, calendar, event)), start);
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
        last.deleting(local, &calendar, &gym);
        assert!(saving.shown(&now_shown(&engine)).is_empty());
        let updates = engine.handle(Some(Job::Delete(local, calendar, gym.clone())), start);
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
        let Some(Undo::Restore(deleted)) = last.undo() else {
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
        let updates = engine.handle(Some(Job::Restore(local, calendar, event)), start);
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
            Some(Job::Delete(local, calendar.clone(), gym.clone())),
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
        let updates = engine.handle(Some(Job::Delete(2, calendar, gym)), start);
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
}
