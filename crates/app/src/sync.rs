//! Synchronizing with Google off the UI thread (`US-02-01`, `US-02-03`, `NFR-01`): one worker owns
//! the cache and shows what it holds at start. Each connected account's first synchronization of
//! the run starts once Google has confirmed the account's sign-in; from then on its calendars are
//! polled as the `Schedule` says, and synchronized at once on "Refresh", when the window is
//! activated, when an event is opened, after waking from sleep and when the network returns. A
//! removed account's data is deleted. Whenever what the cache holds may have changed, the views
//! get the events to show.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Instant;

use calendar_core::accounts::Check;
use calendar_core::cache::{self, Cache};
use calendar_core::model::{Calendar, Event};
use calendar_core::platform::{SecretStore, Wakeups};
use calendar_core::schedule::Schedule;
use calendar_core::sync::{Provider, SyncError, Window, shown_events, sync_calendars};
use calendar_google::{Endpoints, Session};
use eframe::egui;

use crate::platform::{SystemSecretStore, SystemWakeups};

enum Job {
    Sync(String, Window),
    /// Google turned the account's sign-in away: it is not polled until it works again.
    Stop(String),
    Forget(String, Window),
    Refresh,
    Activated,
    /// One of the calendar's events is being opened.
    Opening(String),
    Shown(bool),
    /// The days the cache holds moved, as days pass or the time zone changes.
    Window(Window),
}

enum Update {
    Events(Vec<Event>),
    /// The account's synchronization did not start; it is tried again after its next check.
    Failed(String),
}

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
}

impl Synchronization {
    /// Starts the worker on the cache file; without a home directory the cache lives in memory.
    /// Data of accounts not in `account_ids` is deleted.
    pub fn start(ctx: &egui::Context, account_ids: Vec<String>, window: Window) -> Self {
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
        let sync = Synchronization::with(ctx, open, sign_in, account_ids, window);
        sync.wake_on(&SystemWakeups);
        sync
    }

    fn with(
        ctx: &egui::Context,
        open: impl FnOnce() -> Cache + Send + 'static,
        sign_in: SignIn,
        account_ids: Vec<String>,
        window: Window,
    ) -> Self {
        let (jobs, received) = mpsc::channel();
        let (sender, updates) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let send = |update| {
                let _ = sender.send(update);
                ctx.request_repaint();
            };
            let mut engine = Engine::new(open(), sign_in, window);
            let _ = engine.cache.retain_accounts(&account_ids);
            send(engine.shown());
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
                for update in engine.handle(job, Instant::now()) {
                    send(update);
                }
            }
        });
        Synchronization {
            jobs,
            updates,
            synced: HashSet::new(),
            shown: true,
            window,
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
    /// synchronized yet in this run are, those turned away are no longer polled.
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

    /// The events to show, when they changed. Called every frame.
    pub fn poll(&mut self) -> Option<Vec<Event>> {
        let mut events = None;
        for update in self.updates.try_iter() {
            match update {
                Update::Events(shown) => events = Some(shown),
                Update::Failed(id) => {
                    self.synced.remove(&id);
                }
            }
        }
        events
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
}

impl Engine {
    fn new(cache: Cache, sign_in: SignIn, window: Window) -> Self {
        Engine {
            cache,
            schedule: Schedule::new(),
            sign_in,
            sessions: HashMap::new(),
            pending: HashSet::new(),
            window,
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
                return Vec::new();
            }
            Some(Job::Forget(id, window)) => {
                self.window = window;
                self.stop(&id);
                let _ = self.cache.forget_account(&id);
                return vec![self.shown()];
            }
            Some(Job::Refresh) => {
                // Accounts not synchronized yet are tried again first.
                let mut updates = Vec::new();
                for id in self.pending.clone() {
                    updates.extend(self.first(&id, now));
                }
                let all = self.schedule.all();
                if self.round(&all, now) {
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
        };
        if self.round(&calendars, now) {
            vec![self.shown()]
        } else {
            Vec::new()
        }
    }

    /// The account's first synchronization of this run: it signs in, fetches the calendar list
    /// and synchronizes every calendar; from then on they are polled.
    fn first(&mut self, id: &str, now: Instant) -> Vec<Update> {
        self.pending.remove(id);
        let listed = (self.sign_in)(id).and_then(|session| {
            let calendars = session.calendars()?;
            self.cache.set_calendars(id, &calendars)?;
            Ok((session, calendars))
        });
        let mut updates = Vec::new();
        match listed {
            Ok((session, calendars)) => {
                // A calendar that fails is tried again at its next turn.
                let _ = sync_calendars(session.as_ref(), &calendars, self.window, &mut self.cache);
                self.schedule.set_account(id, &calendars, now);
                self.sessions.insert(id.to_string(), session);
            }
            Err(_) => {
                self.pending.insert(id.to_string());
                updates.push(Update::Failed(id.to_string()));
            }
        }
        updates.push(self.shown());
        updates
    }

    /// Synchronizes the calendars, each with its account's session; says whether what the cache
    /// holds may have changed. A calendar that fails is tried again at its next turn.
    fn round(&mut self, calendars: &[Calendar], now: Instant) -> bool {
        self.schedule.synced(calendars, now);
        let mut changed = false;
        for (account, session) in &self.sessions {
            let of_account: Vec<Calendar> = calendars
                .iter()
                .filter(|calendar| calendar.account_id == *account)
                .cloned()
                .collect();
            if !of_account.is_empty() {
                // A failure can come after other calendars changed.
                changed |=
                    sync_calendars(session.as_ref(), &of_account, self.window, &mut self.cache)
                        .unwrap_or(true);
            }
        }
        changed
    }

    fn stop(&mut self, id: &str) {
        self.pending.remove(id);
        self.schedule.forget(id);
        self.sessions.remove(id);
    }

    fn shown(&self) -> Update {
        // An unreadable cache shows nothing until synchronization rewrites it.
        Update::Events(
            self.cache
                .calendars()
                .map(|cached| shown_events(&cached, self.window))
                .unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::Duration;

    use calendar_core::fixtures::d;
    use calendar_core::model::{Access, Calendar, Rgb, Timing};
    use calendar_core::sync::SyncedEvent;
    use calendar_google::CLIENT_FILE;
    use calendar_google::fake::{ACCOUNT_ID, CLIENT_ID, FakeGoogle, FakeStore};
    use chrono::{TimeZone, Utc};
    use chrono_tz::Europe::Warsaw;
    use serde_json::{Value, json};

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

        let mut sync = Synchronization::with(&ctx, open, unreachable_google(), ids, window());

        let mut shown = next_events(&mut sync);
        shown.sort();
        assert_eq!(shown, ["1", "2"]);
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
        );
        next_events(&mut sync);

        sync.forget("1", window());

        assert_eq!(next_events(&mut sync), ["2"]);
    }

    #[test]
    fn an_account_is_synchronized_once_per_run_and_again_after_a_failure() {
        let ctx = egui::Context::default();
        let ids = vec!["1".to_string()];
        let mut sync =
            Synchronization::with(&ctx, cache_of(&["1"]), unreachable_google(), ids, window());
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
            if let Update::Events(events) = update {
                let mut shown: Vec<String> = events.into_iter().map(|e| e.title).collect();
                shown.sort();
                titles = Some(shown);
            }
        }
        titles
    }

    /// The account's first synchronization of the run, done at `start`.
    fn first_sync(google: &Google, start: Instant) -> Engine {
        let mut engine = Engine::new(Cache::in_memory(), google.sign_in(), window());
        let updates = engine.handle(Some(Job::Sync(ACCOUNT_ID.into(), window())), start);
        assert!(told(updates).is_some());
        engine
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
        let mut engine = Engine::new(Cache::in_memory(), google.sign_in(), window());
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
        let mut sync = Synchronization::with(&ctx, open, google.sign_in(), ids, window());
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
}
