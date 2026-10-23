//! Synchronizing with Google off the UI thread (`US-02-01`, `NFR-01`): one worker owns the cache,
//! shows what it holds at start, then runs each connected account's first full synchronization
//! once Google has confirmed the account's sign-in, and deletes a removed account's data. After
//! every step it sends the views the events to show.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

use calendar_core::accounts::Check;
use calendar_core::cache::{self, Cache};
use calendar_core::model::Event;
use calendar_core::platform::SecretStore;
use calendar_core::sync::{Window, shown_events};
use calendar_google::Endpoints;
use eframe::egui;

use crate::platform::SystemSecretStore;

enum Job {
    Sync(String, Window),
    Forget(String, Window),
}

enum Update {
    Events(Vec<Event>),
    /// The account's synchronization did not finish; it is tried again after its next check.
    Failed(String),
}

pub struct Synchronization {
    jobs: Sender<Job>,
    updates: Receiver<Update>,
    /// Accounts whose first full synchronization of this run is done or under way.
    synced: HashSet<String>,
}

/// Where Google and the sign-ins are, for the worker.
struct Google {
    store: Arc<dyn SecretStore>,
    endpoints: Endpoints,
    client_file: PathBuf,
}

impl Synchronization {
    /// Starts the worker on the cache file; without a home directory the cache lives in memory.
    /// Data of accounts not in `account_ids` is deleted.
    pub fn start(ctx: &egui::Context, account_ids: Vec<String>, window: Window) -> Self {
        let google = Google {
            store: Arc::new(SystemSecretStore),
            endpoints: Endpoints::google(),
            client_file: calendar_google::client_file_path(),
        };
        let open = || {
            cache::default_path()
                .and_then(|path| Cache::open(&path).ok())
                .unwrap_or_else(Cache::in_memory)
        };
        Synchronization::with(ctx, open, google, account_ids, window)
    }

    fn with(
        ctx: &egui::Context,
        open: impl FnOnce() -> Cache + Send + 'static,
        google: Google,
        account_ids: Vec<String>,
        window: Window,
    ) -> Self {
        let (jobs, received) = mpsc::channel();
        let (sender, updates) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let mut cache = open();
            let send = |update| {
                let _ = sender.send(update);
                ctx.request_repaint();
            };
            let shown = |cache: &Cache, window| {
                // An unreadable cache shows nothing until synchronization rewrites it.
                Update::Events(
                    cache
                        .calendars()
                        .map(|cached| shown_events(&cached, window))
                        .unwrap_or_default(),
                )
            };
            let _ = cache.retain_accounts(&account_ids);
            send(shown(&cache, window));
            for job in received {
                match job {
                    Job::Sync(id, window) => {
                        let synced = calendar_google::sync_account(
                            &google.client_file,
                            &google.endpoints,
                            google.store.as_ref(),
                            &id,
                            window,
                            &mut cache,
                        );
                        if synced.is_err() {
                            send(Update::Failed(id));
                        }
                        send(shown(&cache, window));
                    }
                    Job::Forget(id, window) => {
                        let _ = cache.forget_account(&id);
                        send(shown(&cache, window));
                    }
                }
            }
        });
        Synchronization {
            jobs,
            updates,
            synced: HashSet::new(),
        }
    }

    /// Google confirmed these accounts' sign-ins: those not synchronized yet in this run are.
    pub fn checked(&mut self, checked: &[(String, Check)], window: Window) {
        for (id, check) in checked {
            if *check == Check::Works && self.synced.insert(id.clone()) {
                let _ = self.jobs.send(Job::Sync(id.clone(), window));
            }
        }
    }

    /// The account was removed: its calendars and events are deleted (`DEC-85`).
    pub fn forget(&mut self, id: &str, window: Window) {
        self.synced.remove(id);
        let _ = self.jobs.send(Job::Forget(id.to_string(), window));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    use calendar_core::accounts::Account;
    use calendar_core::fixtures::d;
    use calendar_core::model::{Access, Calendar, Rgb, Timing};
    use calendar_core::platform::SecretStoreError;
    use calendar_core::sync::SyncedEvent;
    use chrono::{TimeZone, Utc};
    use chrono_tz::Europe::Warsaw;

    /// A secret store with a sign-in for every account.
    struct Store;

    impl SecretStore for Store {
        fn check(&self) -> Result<(), SecretStoreError> {
            Ok(())
        }
        fn save(&self, _: &Account, _: &str) -> Result<(), SecretStoreError> {
            Ok(())
        }
        fn load(&self, _: &str) -> Result<Option<String>, SecretStoreError> {
            Ok(Some("refresh".into()))
        }
        fn take(&self, _: &str) -> Result<Option<String>, SecretStoreError> {
            Ok(None)
        }
    }

    fn window() -> Window {
        Window::around(d(2026, 11, 11), Warsaw)
    }

    /// Google's address, with nothing answering there, and no client file: every
    /// synchronization fails.
    fn unreachable_google() -> Google {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let base = format!("http://127.0.0.1:{port}");
        Google {
            store: Arc::new(Store),
            endpoints: Endpoints {
                auth: format!("{base}/auth"),
                token: format!("{base}/token"),
                userinfo: format!("{base}/userinfo"),
                revoke: format!("{base}/revoke"),
                calendar: format!("{base}/calendar/v3"),
            },
            client_file: std::env::temp_dir().join("calendar-buddy-sync-no-client-file.json"),
        }
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
}
