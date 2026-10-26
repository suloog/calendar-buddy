//! Local calendars (`US-09-01`, `MOD-09`): calendars and events kept only on the device, in their
//! own SQLite file in the app's data directory, apart from the cache (`MOD-09-NFR-01`). They are
//! the calendars of one more provider behind `Provider` (`NFR-14`), the "Local" pseudo-account, so
//! synchronization brings them into the cache and the views as it does an account's; nothing
//! about them reaches Google (`MOD-09-F01`).

use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

use chrono::NaiveDate;
use chrono_tz::Tz;
use rusqlite::{Connection, Transaction, params};

use crate::accounts::Entry;
use crate::cache::{
    color_text, event_color_key, parse_color, parse_event_color, parse_reminders, parse_start,
    reminders_text, start_text, timing,
};
use crate::colors::{CALENDAR_COLORS, EventColor};
use crate::create::{DEFAULT_LENGTH, NewEvent};
use crate::editor::Patch;
use crate::model::{About, Access, Calendar, Method, Reminder, Reminders, Rgb, Timing};
use crate::sync::{Fetched, Provider, Start, SyncError, SyncedEvent, Window};
use crate::texts;

/// The account the local calendars are listed under; Google's account IDs are numbers, so none
/// is this one.
pub const ACCOUNT_ID: &str = "local";

/// The version of `SCHEMA`, kept in the file from the first release so a later version can
/// migrate it. A file of a later version is neither read nor changed.
const VERSION: i64 = 1;

const SCHEMA: &str = "
    CREATE TABLE calendars (
        id TEXT PRIMARY KEY,
        position INTEGER NOT NULL,
        name TEXT NOT NULL,
        color TEXT NOT NULL,
        zone TEXT NOT NULL,
        reminders TEXT NOT NULL
    );
    CREATE TABLE events (
        calendar_id TEXT NOT NULL,
        id TEXT NOT NULL,
        title TEXT NOT NULL,
        start TEXT,
        end TEXT,
        zone TEXT,
        recurrence TEXT NOT NULL,
        series_id TEXT,
        original_start TEXT,
        description TEXT NOT NULL,
        location TEXT NOT NULL,
        color TEXT,
        reminders TEXT,
        etag TEXT NOT NULL,
        PRIMARY KEY (calendar_id, id)
    );
    CREATE TABLE revision (
        n INTEGER NOT NULL
    );
    INSERT INTO revision VALUES (0);
";

/// The local calendars' file, shared by the synchronization worker's handles to it.
#[derive(Clone)]
pub struct LocalStore {
    connection: Arc<Mutex<Connection>>,
}

fn failed(error: rusqlite::Error) -> SyncError {
    SyncError(texts::local_not_used(&error.to_string()))
}

fn unreadable() -> SyncError {
    SyncError(texts::LOCAL_DAMAGED.to_string())
}

impl LocalStore {
    /// Opens the file, made with its schema when missing or empty. One that cannot be read
    /// whole — damaged, not a database, or from a later version — is left as it is, and the
    /// error says why and where it is (MOD-09 *Constraints and edge cases*).
    pub fn open(path: &Path) -> Result<LocalStore, String> {
        let not_read = |reason: String| texts::local_not_read(path, &reason);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|error| not_read(error.to_string()))?;
        }
        let connection = Connection::open(path).map_err(|error| not_read(error.to_string()))?;
        let store = LocalStore::with_schema(connection).map_err(not_read)?;
        store.check().map_err(|error| not_read(error.0))?;
        Ok(store)
    }

    /// Reads the whole file, so that one damaged anywhere is found before anything is written to
    /// it.
    fn check(&self) -> Result<(), SyncError> {
        let checked: String = (self.connection())
            .query_row("PRAGMA quick_check", [], |row| row.get(0))
            .map_err(failed)?;
        if checked != "ok" {
            return Err(unreadable());
        }
        // Every event is read, whatever days the window holds.
        let any = Window::around(NaiveDate::default(), Tz::UTC);
        for calendar in self.calendars()? {
            self.events(&calendar, any)?;
        }
        Ok(())
    }

    fn with_schema(connection: Connection) -> Result<LocalStore, String> {
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|error| error.to_string())?;
        match version {
            0 => connection
                .execute_batch(&format!(
                    "BEGIN; {SCHEMA} PRAGMA user_version = {VERSION}; COMMIT;"
                ))
                .map_err(|error| error.to_string())?,
            VERSION => {}
            _ => return Err(texts::LOCAL_FROM_NEWER.to_string()),
        }
        Ok(LocalStore {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// A store that lives in memory only: without a home directory, and for tests.
    pub fn in_memory() -> LocalStore {
        let connection = Connection::open_in_memory().expect("an in-memory database");
        LocalStore::with_schema(connection).expect("the local schema")
    }

    fn connection(&self) -> MutexGuard<'_, Connection> {
        self.connection.lock().unwrap()
    }

    /// Adds the calendar after the others.
    pub fn add_calendar(&self, calendar: &Calendar) -> Result<(), SyncError> {
        let connection = self.connection();
        let transaction = connection.unchecked_transaction().map_err(failed)?;
        transaction
            .execute(
                "INSERT INTO calendars VALUES \
                 (?1, (SELECT COALESCE(MAX(position) + 1, 0) FROM calendars), ?2, ?3, ?4, ?5)",
                params![
                    calendar.id,
                    calendar.name,
                    color_text(calendar.color),
                    calendar.zone.name(),
                    reminders_text(&calendar.default_reminders),
                ],
            )
            .map_err(failed)?;
        revise(&transaction)?;
        transaction.commit().map_err(failed)
    }

    /// Renames and recolours the calendar as `calendar` has it (`MOD-09-F03`).
    pub fn change_calendar(&self, calendar: &Calendar) -> Result<(), SyncError> {
        let connection = self.connection();
        let transaction = connection.unchecked_transaction().map_err(failed)?;
        transaction
            .execute(
                "UPDATE calendars SET name = ?2, color = ?3 WHERE id = ?1",
                params![calendar.id, calendar.name, color_text(calendar.color)],
            )
            .map_err(failed)?;
        revise(&transaction)?;
        transaction.commit().map_err(failed)
    }

    /// The number of the calendar's events, a series counted once (`MOD-09-U03`).
    pub fn event_count(&self, calendar_id: &str) -> Result<usize, SyncError> {
        let count: i64 = (self.connection())
            .query_row(
                "SELECT COUNT(*) FROM events WHERE calendar_id = ?1 AND series_id IS NULL",
                params![calendar_id],
                |row| row.get(0),
            )
            .map_err(failed)?;
        Ok(count as usize)
    }

    /// Deletes the calendar and its events for good (`MOD-09-F04`).
    pub fn delete_calendar(&self, calendar_id: &str) -> Result<(), SyncError> {
        let connection = self.connection();
        let transaction = connection.unchecked_transaction().map_err(failed)?;
        for table in [
            "DELETE FROM events WHERE calendar_id = ?1",
            "DELETE FROM calendars WHERE id = ?1",
        ] {
            transaction
                .execute(table, params![calendar_id])
                .map_err(failed)?;
        }
        revise(&transaction)?;
        transaction.commit().map_err(failed)
    }

    /// The number of the last change, which an event's version and the state of the calendars
    /// are: what changed since a state is asked for again in full.
    fn revision(&self) -> Result<i64, SyncError> {
        (self.connection())
            .query_row("SELECT n FROM revision", [], |row| row.get(0))
            .map_err(failed)
    }
}

/// An event as its row keeps it: its ID, title, recurrence, description, location and version,
/// then its start, end, time zone, series, original start, colour and own reminders, each when it
/// has one. Anything that does not read back is damage.
fn event(
    [id, title, recurrence, description, location, etag]: [String; 6],
    [start, end, zone, series, original, color, reminders]: [Option<String>; 7],
) -> Result<SyncedEvent, SyncError> {
    let timing = match (start, end) {
        (Some(start), Some(end)) => {
            Some(timing(&start, &end, zone.as_deref()).ok_or_else(unreadable)?)
        }
        _ => None,
    };
    let instance_of = match (series, original) {
        (Some(series), Some(original)) => {
            Some((series, parse_start(&original).ok_or_else(unreadable)?))
        }
        _ => None,
    };
    let color = (color.map(|key| parse_event_color(&key).ok_or_else(unreadable))).transpose()?;
    let reminders = match reminders {
        None => Reminders::Default,
        Some(own) => Reminders::Own(parse_reminders(&own).ok_or_else(unreadable)?),
    };
    Ok(SyncedEvent {
        id,
        title,
        timing,
        recurrence: recurrence.lines().map(str::to_string).collect(),
        instance_of,
        about: About {
            description,
            location,
            color,
            reminders,
            ..About::default()
        },
        etag,
        organizer_only: false,
    })
}

/// Counts one more change and returns its number.
fn revise(transaction: &Transaction) -> Result<i64, SyncError> {
    transaction
        .query_row("UPDATE revision SET n = n + 1 RETURNING n", [], |row| {
            row.get(0)
        })
        .map_err(failed)
}

impl Provider for LocalStore {
    fn calendars(&self) -> Result<Vec<Calendar>, SyncError> {
        let connection = self.connection();
        let mut select = connection
            .prepare("SELECT id, name, color, zone, reminders FROM calendars ORDER BY position")
            .map_err(failed)?;
        let rows = select
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(failed)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(failed)?;
        rows.into_iter()
            .map(|(id, name, color, zone, reminders)| {
                Some(Calendar {
                    id,
                    account_id: ACCOUNT_ID.to_string(),
                    name,
                    color: parse_color(&color)?,
                    access: Access::Owner,
                    zone: zone.parse().ok()?,
                    default_reminders: parse_reminders(&reminders)?,
                })
            })
            .collect::<Option<_>>()
            .ok_or_else(unreadable)
    }

    fn events(&self, calendar: &Calendar, window: Window) -> Result<Fetched, SyncError> {
        let revision = self.revision()?;
        let connection = self.connection();
        let mut select = connection
            .prepare(
                "SELECT id, title, start, end, zone, recurrence, series_id, original_start, \
                 description, location, color, reminders, etag FROM events WHERE calendar_id = ?1",
            )
            .map_err(failed)?;
        let rows = select
            .query_map(params![calendar.id], |row| {
                let text = |i: usize| row.get::<_, String>(i);
                let maybe = |i: usize| row.get::<_, Option<String>>(i);
                Ok(event(
                    [text(0)?, text(1)?, text(5)?, text(8)?, text(9)?, text(12)?],
                    [
                        maybe(2)?,
                        maybe(3)?,
                        maybe(4)?,
                        maybe(6)?,
                        maybe(7)?,
                        maybe(10)?,
                        maybe(11)?,
                    ],
                ))
            })
            .map_err(failed)?;
        let mut events = Vec::new();
        for row in rows {
            let event = row.map_err(failed)??;
            if window.keeps(&event) {
                events.push(event);
            }
        }
        Ok(Fetched {
            events,
            removed: Vec::new(),
            sync_token: Some(revision.to_string()),
        })
    }

    /// Nothing, while the file is as it was at the state; after any change, the calendar is
    /// asked for again in full.
    fn changes(&self, _: &Calendar, sync_token: &str) -> Result<Option<Fetched>, SyncError> {
        let unchanged = self.revision()?.to_string() == sync_token;
        Ok(unchanged.then(|| Fetched {
            sync_token: Some(sync_token.to_string()),
            ..Fetched::default()
        }))
    }

    /// Keeps what a local event has (`MOD-09-F09`); guests and Google Meet are Google's.
    fn create(&self, calendar: &Calendar, event: &NewEvent) -> Result<String, SyncError> {
        let connection = self.connection();
        let transaction = connection.unchecked_transaction().map_err(failed)?;
        let revision = revise(&transaction)?;
        let id = new_id();
        let (start, end, zone) = match event.timing {
            Timing::Timed { start, end, zone } => (
                start_text(Start::At(start)),
                start_text(Start::At(end)),
                Some(zone.name()),
            ),
            Timing::AllDay {
                start,
                end_exclusive,
            } => (
                start_text(Start::On(start)),
                start_text(Start::On(end_exclusive)),
                None,
            ),
        };
        let about = &event.about;
        transaction
            .execute(
                "INSERT INTO events VALUES \
                 (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL, ?8, ?9, ?10, ?11, ?12)",
                params![
                    calendar.id,
                    id,
                    event.title,
                    start,
                    end,
                    zone,
                    event.recurrence.join("\n"),
                    about.description,
                    about.location,
                    about.color.map(event_color_key),
                    match &about.reminders {
                        Reminders::Default => None,
                        Reminders::Own(own) => Some(reminders_text(own)),
                    },
                    revision.to_string(),
                ],
            )
            .map_err(failed)?;
        transaction.commit().map_err(failed)?;
        Ok(id)
    }

    fn copy(&self, _: &Calendar, _: &str, _: &NewEvent, _: bool) -> Result<String, SyncError> {
        Err(not_yet())
    }

    fn update(
        &self,
        _: &Calendar,
        _: &str,
        _: &str,
        _: &Patch,
        _: bool,
    ) -> Result<String, SyncError> {
        Err(not_yet())
    }

    fn move_to(&self, _: &Calendar, _: &str, _: &Calendar, _: bool) -> Result<String, SyncError> {
        Err(not_yet())
    }

    fn delete(&self, _: &Calendar, _: &str, _: &str, _: bool) -> Result<(), SyncError> {
        Err(not_yet())
    }

    fn restore(&self, _: &Calendar, _: &str, _: bool) -> Result<String, SyncError> {
        Err(not_yet())
    }

    /// Google's default, as there is no Google setting to read (MOD-09 *Constraints and edge
    /// cases*).
    fn default_length(&self) -> Result<u32, SyncError> {
        Ok(DEFAULT_LENGTH)
    }
}

/// Changing, moving and deleting local events come with `US-09-03` (`DEC-181`).
fn not_yet() -> SyncError {
    SyncError(texts::LOCAL_NOT_CHANGED_YET.to_string())
}

/// A new ID for a local calendar or event, unique on the device.
pub fn new_id() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let state = RandomState::new();
    format!(
        "{:016x}{:016x}",
        state.hash_one(now),
        state.hash_one((now, 1))
    )
}

/// The local calendar "Use without an account" makes (`MOD-09-F17`), in the system's time zone:
/// in Google Calendar's Peacock, with one notification 30 minutes before its timed events, as a
/// new Google calendar has (`DEC-178`, `DEC-181`).
pub fn my_calendar(zone: Tz) -> Calendar {
    new_calendar(texts::MY_CALENDAR, zone, &[])
}

/// A new local calendar named `name`, in the system's time zone, with one notification 30
/// minutes before its timed events (`DEC-178`). Its colour is the first of Google Calendar's
/// calendar colours, from Peacock on, that no local calendar among `calendars` has; Peacock
/// once all are taken (`DEC-182`).
pub fn new_calendar(name: &str, zone: Tz, calendars: &[Calendar]) -> Calendar {
    let peacock = EventColor::Peacock.rgb();
    let from = (CALENDAR_COLORS.iter()).position(|(_, rgb)| *rgb == peacock);
    let taken = |rgb: &Rgb| of(calendars).any(|calendar| calendar.color == *rgb);
    let color = (CALENDAR_COLORS
        .iter()
        .cycle()
        .skip(from.unwrap_or_default()))
    .take(CALENDAR_COLORS.len())
    .map(|(_, rgb)| *rgb)
    .find(|rgb| !taken(rgb))
    .unwrap_or(peacock);
    Calendar {
        id: new_id(),
        account_id: ACCOUNT_ID.to_string(),
        name: name.to_string(),
        color,
        access: Access::Owner,
        zone,
        default_reminders: vec![Reminder {
            method: Method::Notification,
            minutes: 30,
        }],
    }
}

/// The local calendars among `calendars`, in their order (`MOD-09-U01`).
pub fn of(calendars: &[Calendar]) -> impl Iterator<Item = &Calendar> {
    (calendars.iter()).filter(|calendar| calendar.account_id == ACCOUNT_ID)
}

/// Whether the welcome screen shows: there is neither a connected account nor a local calendar
/// (`MOD-09-F18`).
pub fn welcome(accounts: &[Entry], calendars: &[Calendar]) -> bool {
    accounts.is_empty() && of(calendars).next().is_none()
}

/// `$XDG_DATA_HOME/calendar-buddy/local.sqlite3`, or under `~/.local/share` when that is not set
/// (`calendar-buddy-dev` in a debug build); never the cache's directory (`MOD-09-NFR-01`).
pub fn default_path() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
        })?;
    Some(data.join(crate::background::APP_ID).join("local.sqlite3"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;
    use crate::model::Rgb;
    use chrono::{Duration, TimeZone, Utc};
    use chrono_tz::Europe::Warsaw;

    fn window() -> Window {
        Window::around(d(2026, 11, 12), Warsaw)
    }

    fn file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "calendar-buddy-local-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("local.sqlite3")
    }

    fn dentist() -> NewEvent {
        let start = Utc.with_ymd_and_hms(2026, 11, 13, 9, 0, 0).unwrap();
        NewEvent {
            title: "Dentist".to_string(),
            timing: Timing::Timed {
                start,
                end: start + Duration::hours(1),
                zone: Warsaw,
            },
            about: About::default(),
            recurrence: Vec::new(),
            meet: false,
        }
    }

    fn gym() -> NewEvent {
        NewEvent {
            title: "Gym".to_string(),
            timing: Timing::AllDay {
                start: d(2026, 11, 13),
                end_exclusive: d(2026, 11, 14),
            },
            about: About {
                description: "Legs".to_string(),
                location: "Fit Club".to_string(),
                ..About::default()
            },
            recurrence: Vec::new(),
            meet: false,
        }
    }

    #[test]
    fn my_calendar_is_a_local_calendar_the_user_owns() {
        let calendar = my_calendar(Warsaw);
        assert_eq!(calendar.account_id, ACCOUNT_ID);
        assert_eq!(calendar.name, "My calendar");
        assert_eq!(calendar.color, Rgb(0x03, 0x9B, 0xE5));
        assert!(calendar.access.can_edit());
        assert_eq!(calendar.id.len(), 32);
        assert_ne!(calendar.id, my_calendar(Warsaw).id);
    }

    #[test]
    fn calendars_and_events_are_kept_in_the_file_across_openings() {
        let path = file("kept");
        let store = LocalStore::open(&path).unwrap();
        let calendar = my_calendar(Warsaw);
        store.add_calendar(&calendar).unwrap();
        let dentist_id = store.create(&calendar, &dentist()).unwrap();
        let gym_id = store.create(&calendar, &gym()).unwrap();
        drop(store);

        let store = LocalStore::open(&path).unwrap();

        assert_eq!(store.calendars().unwrap(), std::slice::from_ref(&calendar));
        let fetched = store.events(&calendar, window()).unwrap();
        let mut events = fetched.events;
        events.sort_by(|a, b| a.title.cmp(&b.title));
        let [dentist_kept, gym_kept] = events.as_slice() else {
            panic!("two events");
        };
        assert_eq!(
            (dentist_kept.id.as_str(), dentist_kept.timing),
            (dentist_id.as_str(), Some(dentist().timing))
        );
        assert_eq!(gym_kept.id, gym_id);
        assert_eq!(gym_kept.timing, Some(gym().timing));
        assert_eq!(gym_kept.about.description, "Legs");
        assert_eq!(gym_kept.about.location, "Fit Club");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn a_new_file_has_its_schema_version() {
        let path = file("version");
        drop(LocalStore::open(&path).unwrap());
        let connection = Connection::open(&path).unwrap();
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, VERSION);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn tc_09_04_a_file_that_is_not_a_database_is_reported_and_left_as_it_is() {
        let path = file("tc-09-04");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = b"These are not the bytes of an SQLite database, whatever they are.".repeat(20);
        std::fs::write(&path, &bytes).unwrap();

        let Err(error) = LocalStore::open(&path) else {
            panic!("the file was read");
        };

        assert!(
            error.starts_with("Local calendars cannot be read from"),
            "{error}"
        );
        assert!(error.contains(&path.display().to_string()), "{error}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn a_file_with_a_damaged_event_is_not_read_nor_changed() {
        let path = file("damaged");
        let store = LocalStore::open(&path).unwrap();
        let calendar = my_calendar(Warsaw);
        store.add_calendar(&calendar).unwrap();
        store.create(&calendar, &dentist()).unwrap();
        drop(store);
        Connection::open(&path)
            .unwrap()
            .execute("UPDATE events SET start = 'not a time'", [])
            .unwrap();
        let bytes = std::fs::read(&path).unwrap();

        let Err(error) = LocalStore::open(&path) else {
            panic!("the file was read");
        };

        assert!(error.contains(texts::LOCAL_DAMAGED), "{error}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn a_file_of_a_later_version_is_not_read_nor_changed() {
        let path = file("later");
        drop(LocalStore::open(&path).unwrap());
        Connection::open(&path)
            .unwrap()
            .pragma_update(None, "user_version", VERSION + 1)
            .unwrap();
        let bytes = std::fs::read(&path).unwrap();

        let Err(error) = LocalStore::open(&path) else {
            panic!("the file was read");
        };

        assert!(error.contains(texts::LOCAL_FROM_NEWER), "{error}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn changes_are_none_until_something_is_written_then_the_calendar_is_fetched_again() {
        let store = LocalStore::in_memory();
        let calendar = my_calendar(Warsaw);
        store.add_calendar(&calendar).unwrap();
        let state = store
            .events(&calendar, window())
            .unwrap()
            .sync_token
            .unwrap();

        let unchanged = store.changes(&calendar, &state).unwrap().unwrap();
        assert!(unchanged.events.is_empty() && unchanged.removed.is_empty());

        store.create(&calendar, &dentist()).unwrap();
        assert_eq!(store.changes(&calendar, &state).unwrap(), None);
    }

    #[test]
    fn a_calendar_is_renamed_recoloured_and_deleted_with_its_events() {
        let store = LocalStore::in_memory();
        let family = new_calendar("Family", Warsaw, &[]);
        let sport = new_calendar("Sport", Warsaw, std::slice::from_ref(&family));
        store.add_calendar(&family).unwrap();
        store.add_calendar(&sport).unwrap();
        store.create(&family, &dentist()).unwrap();
        store.create(&family, &gym()).unwrap();
        let state = store.events(&family, window()).unwrap().sync_token.unwrap();

        let private = Calendar {
            name: "Private".to_string(),
            color: EventColor::Basil.rgb(),
            ..family.clone()
        };
        store.change_calendar(&private).unwrap();
        assert_eq!(store.calendars().unwrap(), [private.clone(), sport.clone()]);
        assert_eq!(store.changes(&family, &state).unwrap(), None);
        assert_eq!(
            (store.event_count(&family.id), store.event_count(&sport.id)),
            (Ok(2), Ok(0))
        );

        store.delete_calendar(&family.id).unwrap();
        assert_eq!(store.calendars().unwrap(), [sport]);
        assert_eq!(store.event_count(&family.id), Ok(0));
        assert!(store.events(&family, window()).unwrap().events.is_empty());
    }

    #[test]
    fn a_new_calendar_takes_the_first_colour_from_peacock_on_no_local_calendar_has() {
        let my = my_calendar(Warsaw);
        assert_eq!(my.color, EventColor::Peacock.rgb());
        let second = new_calendar("Family", Warsaw, std::slice::from_ref(&my));
        let names: Vec<&str> = (CALENDAR_COLORS.iter())
            .filter(|(_, rgb)| *rgb == second.color)
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(names, [texts::WISTERIA]);
        assert_eq!(second.default_reminders, my.default_reminders);
        assert_eq!(second.account_id, ACCOUNT_ID);

        // A Google calendar's colour is not counted; with all 24 taken, Peacock again.
        let google = Calendar {
            account_id: "1".to_string(),
            ..my.clone()
        };
        assert_eq!(new_calendar("A", Warsaw, &[google]).color, my.color);
        let all: Vec<Calendar> = (CALENDAR_COLORS.iter())
            .map(|(_, rgb)| Calendar {
                color: *rgb,
                ..my.clone()
            })
            .collect();
        assert_eq!(new_calendar("B", Warsaw, &all).color, my.color);
        let radicchio = new_calendar("C", Warsaw, &all[1..]).color;
        assert_eq!(radicchio, CALENDAR_COLORS[0].1);
    }

    #[test]
    fn local_events_take_googles_default_length_and_cannot_be_changed_yet() {
        let store = LocalStore::in_memory();
        let calendar = my_calendar(Warsaw);
        assert_eq!(store.default_length(), Ok(60));
        let refused = store.delete(&calendar, "x", "", false).unwrap_err();
        assert_eq!(refused.0, texts::LOCAL_NOT_CHANGED_YET);
    }
}
