//! The local cache (`DEC-23`): every account's calendars and their synchronized events, in
//! SQLite. It is the read model of the views; only synchronization writes to it. Nothing in it is
//! secret, and it can always be rebuilt from Google (`NFR-17`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, ErrorCode, OptionalExtension, Transaction, params};

use crate::colors::EventColor;
use crate::model::{About, Access, Calendar, Kind, Rgb, Show, Timing, Visibility};
use crate::sync::{Start, SyncError, SyncedEvent, Window};

/// The version of `SCHEMA`; a cache made with an earlier one has its events, synchronization
/// states and time of the last synchronization dropped, so they are fetched again in full.
const VERSION: i64 = 4;

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS calendars (
        account_id TEXT NOT NULL,
        id TEXT NOT NULL,
        position INTEGER NOT NULL,
        name TEXT NOT NULL,
        color TEXT NOT NULL,
        access TEXT NOT NULL,
        zone TEXT NOT NULL,
        PRIMARY KEY (account_id, id)
    );
    CREATE TABLE IF NOT EXISTS events (
        account_id TEXT NOT NULL,
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
        show TEXT NOT NULL,
        visibility TEXT NOT NULL,
        etag TEXT NOT NULL,
        organizer_only INTEGER NOT NULL,
        kind TEXT NOT NULL,
        color TEXT,
        PRIMARY KEY (account_id, calendar_id, id)
    );
    CREATE TABLE IF NOT EXISTS sync_tokens (
        account_id TEXT NOT NULL,
        calendar_id TEXT NOT NULL,
        token TEXT NOT NULL,
        window TEXT NOT NULL,
        PRIMARY KEY (account_id, calendar_id)
    );
    CREATE TABLE IF NOT EXISTS last_synced (
        at TEXT NOT NULL
    );
";

/// A cached calendar with its events, series not expanded.
#[derive(Clone, Debug, PartialEq)]
pub struct CachedCalendar {
    pub calendar: Calendar,
    pub events: Vec<SyncedEvent>,
}

pub struct Cache {
    connection: Connection,
}

fn failed(error: rusqlite::Error) -> SyncError {
    SyncError(format!("the local cache could not be used ({error})"))
}

fn unreadable() -> SyncError {
    SyncError("the local cache could not be read".into())
}

impl Cache {
    /// Opens the cache file, creating it when missing; a damaged one is discarded and started
    /// anew, as Google holds everything in it.
    pub fn open(path: &Path) -> Result<Cache, SyncError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|error| {
                SyncError(format!("the local cache could not be made ({error})"))
            })?;
        }
        let damaged = |error: &rusqlite::Error| {
            matches!(
                error.sqlite_error_code(),
                Some(ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt)
            )
        };
        match Cache::open_file(path) {
            Err(error) if damaged(&error) => {
                let _ = std::fs::remove_file(path);
                Cache::open_file(path).map_err(failed)
            }
            opened => opened.map_err(failed),
        }
    }

    fn open_file(path: &Path) -> rusqlite::Result<Cache> {
        Cache::with_schema(Connection::open(path)?)
    }

    fn with_schema(connection: Connection) -> rusqlite::Result<Cache> {
        let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version < VERSION {
            connection.execute_batch(
                "DROP TABLE IF EXISTS events; DROP TABLE IF EXISTS sync_tokens; \
                 DROP TABLE IF EXISTS last_synced;",
            )?;
        }
        connection.execute_batch(SCHEMA)?;
        connection.pragma_update(None, "user_version", VERSION)?;
        Ok(Cache { connection })
    }

    /// A cache that lives in memory only, for tests.
    pub fn in_memory() -> Cache {
        let connection = Connection::open_in_memory().expect("an in-memory database");
        Cache::with_schema(connection).expect("the cache schema")
    }

    /// Replaces the account's calendars; the events of calendars no longer listed go with them.
    pub fn set_calendars(
        &mut self,
        account_id: &str,
        calendars: &[Calendar],
    ) -> Result<(), SyncError> {
        let transaction = self.connection.transaction().map_err(failed)?;
        transaction
            .execute(
                "DELETE FROM calendars WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(failed)?;
        for (position, calendar) in calendars.iter().enumerate() {
            transaction
                .execute(
                    "INSERT OR REPLACE INTO calendars VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        account_id,
                        calendar.id,
                        position as i64,
                        calendar.name,
                        color_text(calendar.color),
                        access_key(calendar.access),
                        calendar.zone.name(),
                    ],
                )
                .map_err(failed)?;
        }
        for table in ["events", "sync_tokens"] {
            transaction
                .execute(
                    &format!(
                        "DELETE FROM {table} WHERE account_id = ?1 AND calendar_id NOT IN \
                         (SELECT id FROM calendars WHERE account_id = ?1)"
                    ),
                    params![account_id],
                )
                .map_err(failed)?;
        }
        transaction.commit().map_err(failed)
    }

    /// Replaces the calendar's events.
    pub fn set_events(
        &mut self,
        calendar: &Calendar,
        events: &[SyncedEvent],
    ) -> Result<(), SyncError> {
        let transaction = self.connection.transaction().map_err(failed)?;
        transaction
            .execute(
                "DELETE FROM events WHERE account_id = ?1 AND calendar_id = ?2",
                params![calendar.account_id, calendar.id],
            )
            .map_err(failed)?;
        insert(&transaction, calendar, events)?;
        transaction.commit().map_err(failed)
    }

    /// Applies what changed in the calendar: changed and new events replace those with their IDs,
    /// and removed ones go, a removed series with its changed and cancelled occurrences.
    pub fn change_events(
        &mut self,
        calendar: &Calendar,
        changed: &[SyncedEvent],
        removed: &[String],
    ) -> Result<(), SyncError> {
        let transaction = self.connection.transaction().map_err(failed)?;
        for id in removed {
            transaction
                .execute(
                    "DELETE FROM events WHERE account_id = ?1 AND calendar_id = ?2 \
                     AND (id = ?3 OR series_id = ?3)",
                    params![calendar.account_id, calendar.id, id],
                )
                .map_err(failed)?;
        }
        insert(&transaction, calendar, changed)?;
        transaction.commit().map_err(failed)
    }

    /// The state Google gave for asking what changed in the calendar since its last full
    /// synchronization within this window; `None` before it, or once the window has moved, as
    /// changes do not bring the events that entered the window unchanged.
    pub fn sync_token(
        &self,
        calendar: &Calendar,
        window: Window,
    ) -> Result<Option<String>, SyncError> {
        self.connection
            .query_row(
                "SELECT token FROM sync_tokens WHERE account_id = ?1 AND calendar_id = ?2 \
                 AND window = ?3",
                params![calendar.account_id, calendar.id, window_key(window)],
                |row| row.get(0),
            )
            .optional()
            .map_err(failed)
    }

    /// Keeps the calendar's synchronization state, for the window its events were fetched in;
    /// `None` forgets it.
    pub fn set_sync_token(
        &mut self,
        calendar: &Calendar,
        token: Option<&str>,
        window: Window,
    ) -> Result<(), SyncError> {
        match token {
            Some(token) => self.connection.execute(
                "INSERT OR REPLACE INTO sync_tokens VALUES (?1, ?2, ?3, ?4)",
                params![calendar.account_id, calendar.id, token, window_key(window)],
            ),
            None => self.connection.execute(
                "DELETE FROM sync_tokens WHERE account_id = ?1 AND calendar_id = ?2",
                params![calendar.account_id, calendar.id],
            ),
        }
        .map(|_| ())
        .map_err(failed)
    }

    /// When a synchronization last got through (`MOD-02-U03`); `None` before the first.
    pub fn last_synced(&self) -> Result<Option<DateTime<Utc>>, SyncError> {
        let at: Option<String> = self
            .connection
            .query_row("SELECT at FROM last_synced", [], |row| row.get(0))
            .optional()
            .map_err(failed)?;
        Ok(at.and_then(|at| DateTime::parse_from_rfc3339(&at).ok().map(|at| at.to_utc())))
    }

    pub fn set_last_synced(&mut self, at: DateTime<Utc>) -> Result<(), SyncError> {
        let transaction = self.connection.transaction().map_err(failed)?;
        transaction
            .execute("DELETE FROM last_synced", [])
            .map_err(failed)?;
        transaction
            .execute(
                "INSERT INTO last_synced VALUES (?1)",
                params![at.to_rfc3339()],
            )
            .map_err(failed)?;
        transaction.commit().map_err(failed)
    }

    /// Deletes everything kept for an account (`MOD-01-F08`, `DEC-85`).
    pub fn forget_account(&mut self, account_id: &str) -> Result<(), SyncError> {
        let transaction = self.connection.transaction().map_err(failed)?;
        for table in ["calendars", "events", "sync_tokens"] {
            transaction
                .execute(
                    &format!("DELETE FROM {table} WHERE account_id = ?1"),
                    params![account_id],
                )
                .map_err(failed)?;
        }
        transaction.commit().map_err(failed)
    }

    /// Deletes what is kept for accounts no longer connected, e.g. left by a removal the app did
    /// not finish.
    pub fn retain_accounts(&mut self, account_ids: &[String]) -> Result<(), SyncError> {
        let cached: Vec<String> = self
            .connection
            .prepare("SELECT DISTINCT account_id FROM calendars")
            .and_then(|mut select| select.query_map([], |row| row.get(0))?.collect())
            .map_err(failed)?;
        for gone in cached.iter().filter(|id| !account_ids.contains(id)) {
            self.forget_account(gone)?;
        }
        Ok(())
    }

    /// Deletes the calendar's events and synchronization state; the calendar stays listed, as a
    /// disabled one does (`MOD-02-F06`).
    pub fn clear_calendar(&mut self, account_id: &str, calendar_id: &str) -> Result<(), SyncError> {
        let transaction = self.connection.transaction().map_err(failed)?;
        for table in ["events", "sync_tokens"] {
            transaction
                .execute(
                    &format!("DELETE FROM {table} WHERE account_id = ?1 AND calendar_id = ?2"),
                    params![account_id, calendar_id],
                )
                .map_err(failed)?;
        }
        transaction.commit().map_err(failed)
    }

    /// Every cached calendar, without its events, each account's calendars in the order Google
    /// lists them.
    pub fn calendar_list(&self) -> Result<Vec<Calendar>, SyncError> {
        self.connection
            .prepare(
                "SELECT account_id, id, name, color, access, zone FROM calendars \
                 ORDER BY account_id, position",
            )
            .and_then(|mut select| {
                select
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, String>(5)?,
                        ))
                    })?
                    .collect::<Result<Vec<_>, _>>()
            })
            .map_err(failed)?
            .into_iter()
            .map(|(account_id, id, name, color, access, zone)| {
                Some(Calendar {
                    id,
                    account_id,
                    name,
                    color: parse_color(&color)?,
                    access: parse_access(&access)?,
                    zone: zone.parse().ok()?,
                })
            })
            .collect::<Option<_>>()
            .ok_or_else(unreadable)
    }

    /// Every cached calendar with its events, each account's calendars in the order Google lists
    /// them.
    pub fn calendars(&self) -> Result<Vec<CachedCalendar>, SyncError> {
        let mut calendars: Vec<CachedCalendar> = self
            .calendar_list()?
            .into_iter()
            .map(|calendar| CachedCalendar {
                calendar,
                events: Vec::new(),
            })
            .collect();
        let index: HashMap<(String, String), usize> = calendars
            .iter()
            .enumerate()
            .map(|(i, c)| ((c.calendar.account_id.clone(), c.calendar.id.clone()), i))
            .collect();
        let mut select = self
            .connection
            .prepare(
                "SELECT account_id, calendar_id, id, title, start, end, zone, recurrence, \
                 series_id, original_start, description, location, show, visibility, etag, \
                 organizer_only, kind, color FROM events",
            )
            .map_err(failed)?;
        let rows = select
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    About {
                        description: row.get(10)?,
                        location: row.get(11)?,
                        show: parse_show(&row.get::<_, String>(12)?),
                        visibility: parse_visibility(&row.get::<_, String>(13)?),
                        kind: parse_kind(&row.get::<_, String>(16)?),
                        color: (row.get::<_, Option<String>>(17)?)
                            .and_then(|key| parse_event_color(&key)),
                    },
                    row.get::<_, String>(14)?,
                    row.get::<_, bool>(15)?,
                ))
            })
            .map_err(failed)?;
        for row in rows {
            let (
                account_id,
                calendar_id,
                id,
                title,
                start,
                end,
                zone,
                recurrence,
                series,
                original,
                about,
                etag,
                organizer_only,
            ) = row.map_err(failed)?;
            let timing = match (start, end, zone) {
                (Some(start), Some(end), zone) => {
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
            let event = SyncedEvent {
                id,
                title,
                timing,
                recurrence: recurrence.lines().map(str::to_string).collect(),
                instance_of,
                about,
                etag,
                organizer_only,
            };
            if let Some(&index) = index.get(&(account_id, calendar_id)) {
                calendars[index].events.push(event);
            }
        }
        Ok(calendars)
    }
}

/// Writes the calendar's events, replacing those with the same IDs.
fn insert(
    transaction: &Transaction,
    calendar: &Calendar,
    events: &[SyncedEvent],
) -> Result<(), SyncError> {
    let mut insert = transaction
        .prepare(
            "INSERT OR REPLACE INTO events VALUES \
             (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        )
        .map_err(failed)?;
    for event in events {
        let (start, end, zone) = match event.timing {
            None => (None, None, None),
            Some(Timing::Timed { start, end, zone }) => (
                Some(start_text(Start::At(start))),
                Some(start_text(Start::At(end))),
                Some(zone.name()),
            ),
            Some(Timing::AllDay {
                start,
                end_exclusive,
            }) => (
                Some(start_text(Start::On(start))),
                Some(start_text(Start::On(end_exclusive))),
                None,
            ),
        };
        let (series_id, original_start) = match &event.instance_of {
            Some((series, original)) => (Some(series.as_str()), Some(start_text(*original))),
            None => (None, None),
        };
        insert
            .execute(params![
                calendar.account_id,
                calendar.id,
                event.id,
                event.title,
                start,
                end,
                zone,
                event.recurrence.join("\n"),
                series_id,
                original_start,
                event.about.description,
                event.about.location,
                show_key(event.about.show),
                visibility_key(event.about.visibility),
                event.etag,
                event.organizer_only,
                kind_key(event.about.kind),
                event.about.color.map(EventColor::name),
            ])
            .map_err(failed)?;
    }
    Ok(())
}

fn timing(start: &str, end: &str, zone: Option<&str>) -> Option<Timing> {
    Some(match (parse_start(start)?, parse_start(end)?, zone) {
        (Start::At(start), Start::At(end), Some(zone)) => Timing::Timed {
            start,
            end,
            zone: zone.parse().ok()?,
        },
        (Start::On(start), Start::On(end_exclusive), None) => Timing::AllDay {
            start,
            end_exclusive,
        },
        _ => return None,
    })
}

/// `$XDG_CACHE_HOME/calendar-buddy/cache.sqlite3`, or under `~/.cache` when that is not set.
pub fn default_path() -> Option<PathBuf> {
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))?;
    Some(cache.join("calendar-buddy").join("cache.sqlite3"))
}

fn window_key(window: Window) -> String {
    format!("{}/{}/{}", window.from, window.until, window.zone.name())
}

/// `#rrggbb`, as Google writes colours.
pub fn color_text(Rgb(r, g, b): Rgb) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// `#rrggbb`, as Google writes colours.
pub fn parse_color(text: &str) -> Option<Rgb> {
    let hex = text.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    Some(Rgb(byte(0)?, byte(2)?, byte(4)?))
}

fn show_key(show: Show) -> &'static str {
    match show {
        Show::Busy => "busy",
        Show::Free => "free",
    }
}

fn parse_show(key: &str) -> Show {
    match key {
        "free" => Show::Free,
        _ => Show::Busy,
    }
}

fn visibility_key(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Default => "default",
        Visibility::Public => "public",
        Visibility::Private => "private",
    }
}

fn parse_visibility(key: &str) -> Visibility {
    match key {
        "public" => Visibility::Public,
        "private" => Visibility::Private,
        _ => Visibility::Default,
    }
}

fn kind_key(kind: Kind) -> &'static str {
    match kind {
        Kind::Ordinary => "ordinary",
        Kind::OutOfOffice => "out-of-office",
        Kind::FocusTime => "focus-time",
        Kind::WorkingLocation => "working-location",
        Kind::Birthday => "birthday",
    }
}

fn parse_kind(key: &str) -> Kind {
    match key {
        "out-of-office" => Kind::OutOfOffice,
        "focus-time" => Kind::FocusTime,
        "working-location" => Kind::WorkingLocation,
        "birthday" => Kind::Birthday,
        _ => Kind::Ordinary,
    }
}

fn parse_event_color(name: &str) -> Option<EventColor> {
    EventColor::ALL
        .into_iter()
        .find(|color| color.name() == name)
}

fn access_key(access: Access) -> &'static str {
    match access {
        Access::Owner => "owner",
        Access::Writer => "writer",
        Access::Reader => "reader",
        Access::FreeBusyReader => "freeBusyReader",
    }
}

/// Google's `accessRole` values.
pub fn parse_access(key: &str) -> Option<Access> {
    Some(match key {
        "owner" => Access::Owner,
        "writer" => Access::Writer,
        "reader" => Access::Reader,
        "freeBusyReader" => Access::FreeBusyReader,
        _ => return None,
    })
}

fn start_text(start: Start) -> String {
    match start {
        Start::At(at) => at.to_rfc3339(),
        Start::On(date) => date.to_string(),
    }
}

fn parse_start(text: &str) -> Option<Start> {
    match DateTime::parse_from_rfc3339(text) {
        Ok(at) => Some(Start::At(at.with_timezone(&Utc))),
        Err(_) => text.parse::<NaiveDate>().ok().map(Start::On),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;
    use chrono::TimeZone;
    use chrono_tz::Europe::{London, Warsaw};

    fn window() -> Window {
        Window::around(d(2026, 11, 11), Warsaw)
    }

    fn calendar(account_id: &str, id: &str) -> Calendar {
        Calendar {
            id: id.into(),
            account_id: account_id.into(),
            name: format!("Calendar {id}"),
            color: Rgb(0x0b, 0x80, 0x43),
            access: Access::Reader,
            zone: London,
        }
    }

    fn events() -> Vec<SyncedEvent> {
        let start = Utc.with_ymd_and_hms(2026, 11, 2, 8, 0, 0).unwrap();
        vec![
            SyncedEvent {
                id: "series".into(),
                title: "Standup".into(),
                timing: Some(Timing::Timed {
                    start,
                    end: start + chrono::Duration::minutes(15),
                    zone: Warsaw,
                }),
                recurrence: vec![
                    "RRULE:FREQ=WEEKLY".into(),
                    "EXDATE;TZID=Europe/Warsaw:20261109T090000".into(),
                ],
                instance_of: None,
                about: About::default(),
                etag: "\"1\"".into(),
                organizer_only: false,
            },
            SyncedEvent {
                id: "series_20261116T080000Z".into(),
                title: "Standup".into(),
                timing: None,
                recurrence: Vec::new(),
                instance_of: Some((
                    "series".into(),
                    Start::At(Utc.with_ymd_and_hms(2026, 11, 16, 8, 0, 0).unwrap()),
                )),
                about: About::default(),
                etag: "\"2\"".into(),
                organizer_only: false,
            },
            SyncedEvent {
                id: "trip".into(),
                title: "Trip".into(),
                timing: Some(Timing::AllDay {
                    start: d(2026, 11, 12),
                    end_exclusive: d(2026, 11, 15),
                }),
                recurrence: Vec::new(),
                instance_of: None,
                about: About {
                    description: "Flights\nHotel".into(),
                    location: "Lisbon".into(),
                    show: Show::Free,
                    visibility: Visibility::Private,
                    kind: Kind::OutOfOffice,
                    color: Some(EventColor::Grape),
                },
                etag: "\"3\"".into(),
                organizer_only: true,
            },
        ]
    }

    #[test]
    fn calendars_and_events_come_back_as_they_were_written() {
        let mut cache = Cache::in_memory();
        let work = calendar("1", "work");
        cache
            .set_calendars("1", std::slice::from_ref(&work))
            .unwrap();
        cache.set_events(&work, &events()).unwrap();

        assert_eq!(
            cache.calendars().unwrap(),
            [CachedCalendar {
                calendar: work,
                events: events()
            }]
        );
    }

    #[test]
    fn the_same_calendar_on_two_accounts_is_kept_twice() {
        let mut cache = Cache::in_memory();
        let (a, b) = (calendar("1", "holidays"), calendar("2", "holidays"));
        cache.set_calendars("1", std::slice::from_ref(&a)).unwrap();
        cache.set_calendars("2", std::slice::from_ref(&b)).unwrap();
        cache.set_events(&a, &events()).unwrap();
        cache.set_events(&b, &events()[..1]).unwrap();

        let cached = cache.calendars().unwrap();

        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].events.len(), 3);
        assert_eq!(cached[1].events.len(), 1);
    }

    #[test]
    fn a_calendar_no_longer_listed_goes_with_its_events() {
        let mut cache = Cache::in_memory();
        let (work, home) = (calendar("1", "work"), calendar("1", "home"));
        cache
            .set_calendars("1", &[work.clone(), home.clone()])
            .unwrap();
        cache.set_events(&work, &events()).unwrap();
        cache.set_events(&home, &events()).unwrap();

        cache
            .set_calendars("1", std::slice::from_ref(&home))
            .unwrap();

        let cached = cache.calendars().unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].calendar, home);
        let left: i64 = cache
            .connection
            .query_row("SELECT count(*) FROM events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(left, 3);
    }

    #[test]
    fn forgetting_an_account_deletes_its_calendars_and_events_only() {
        let mut cache = Cache::in_memory();
        let (a, b) = (calendar("1", "work"), calendar("2", "home"));
        cache.set_calendars("1", std::slice::from_ref(&a)).unwrap();
        cache.set_calendars("2", std::slice::from_ref(&b)).unwrap();
        cache.set_events(&a, &events()).unwrap();
        cache.set_events(&b, &events()).unwrap();

        cache.forget_account("1").unwrap();

        let cached = cache.calendars().unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].calendar, b);
        let left: i64 = cache
            .connection
            .query_row(
                "SELECT count(*) FROM events WHERE account_id = '1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(left, 0);
    }

    #[test]
    fn accounts_no_longer_connected_are_dropped() {
        let mut cache = Cache::in_memory();
        cache.set_calendars("1", &[calendar("1", "work")]).unwrap();
        cache.set_calendars("2", &[calendar("2", "home")]).unwrap();

        cache.retain_accounts(&["2".to_string()]).unwrap();

        let cached = cache.calendars().unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].calendar.account_id, "2");
    }

    #[test]
    fn the_time_of_the_last_synchronization_is_kept() {
        let mut cache = Cache::in_memory();
        assert_eq!(cache.last_synced().unwrap(), None);
        let at = Utc.with_ymd_and_hms(2026, 11, 11, 13, 5, 0).unwrap();

        cache
            .set_last_synced(at - chrono::Duration::hours(1))
            .unwrap();
        cache.set_last_synced(at).unwrap();

        assert_eq!(cache.last_synced().unwrap(), Some(at));
    }

    #[test]
    fn a_damaged_cache_file_is_started_anew() {
        let dir = std::env::temp_dir().join(format!("calendar-buddy-cache-{}", std::process::id()));
        let path = dir.join("cache.sqlite3");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, b"not a database, just some damaged bytes here").unwrap();

        let mut cache = Cache::open(&path).unwrap();
        cache.set_calendars("1", &[calendar("1", "work")]).unwrap();
        drop(cache);

        assert_eq!(Cache::open(&path).unwrap().calendars().unwrap().len(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_cache_from_an_earlier_version_drops_its_events_and_states_to_fetch_them_again() {
        let dir = std::env::temp_dir().join(format!(
            "calendar-buddy-cache-version-{}",
            std::process::id()
        ));
        let path = dir.join("cache.sqlite3");
        std::fs::create_dir_all(&dir).unwrap();
        // The events table before descriptions, locations and versions were kept.
        let old = Connection::open(&path).unwrap();
        old.execute_batch(
            "CREATE TABLE events (account_id TEXT NOT NULL, calendar_id TEXT NOT NULL, \
             id TEXT NOT NULL, title TEXT NOT NULL, start TEXT, end TEXT, zone TEXT, \
             recurrence TEXT NOT NULL, series_id TEXT, original_start TEXT, \
             PRIMARY KEY (account_id, calendar_id, id)); \
             INSERT INTO events VALUES ('1', 'work', 'old', 'Old', NULL, NULL, NULL, '', NULL, \
             NULL); \
             CREATE TABLE last_synced (at TEXT NOT NULL); \
             INSERT INTO last_synced VALUES ('2026-10-22T10:00:00+00:00');",
        )
        .unwrap();
        drop(old);

        let mut cache = Cache::open(&path).unwrap();
        // Nothing synchronized is left to claim it is up to date.
        assert_eq!(cache.last_synced().unwrap(), None);
        let work = calendar("1", "work");
        cache
            .set_calendars("1", std::slice::from_ref(&work))
            .unwrap();
        cache.set_events(&work, &events()).unwrap();
        drop(cache);
        let cached = Cache::open(&path).unwrap().calendars().unwrap();
        std::fs::remove_dir_all(dir).unwrap();

        assert_eq!(cached[0].events, events());
    }

    #[test]
    fn changes_replace_add_and_remove_events_a_removed_series_with_its_occurrences() {
        let mut cache = Cache::in_memory();
        let work = calendar("1", "work");
        cache
            .set_calendars("1", std::slice::from_ref(&work))
            .unwrap();
        cache.set_events(&work, &events()).unwrap();
        let mut trip = events()[2].clone();
        trip.title = "Trip to Kraków".into();
        let mut dinner = events()[2].clone();
        dinner.id = "dinner".into();
        dinner.title = "Dinner".into();

        cache
            .change_events(&work, &[trip.clone(), dinner.clone()], &["series".into()])
            .unwrap();

        let mut left = cache.calendars().unwrap().remove(0).events;
        left.sort_by(|a, b| a.id.cmp(&b.id));
        assert_eq!(left, [dinner, trip]);
    }

    #[test]
    fn a_calendars_sync_token_is_kept_until_it_or_its_account_goes() {
        let mut cache = Cache::in_memory();
        let (work, home) = (calendar("1", "work"), calendar("1", "home"));
        cache
            .set_calendars("1", &[work.clone(), home.clone()])
            .unwrap();
        assert_eq!(cache.sync_token(&work, window()).unwrap(), None);

        cache
            .set_sync_token(&work, Some("token-1"), window())
            .unwrap();
        cache
            .set_sync_token(&home, Some("token-2"), window())
            .unwrap();
        cache
            .set_sync_token(&work, Some("token-3"), window())
            .unwrap();
        assert_eq!(
            cache.sync_token(&work, window()).unwrap().as_deref(),
            Some("token-3")
        );
        assert_eq!(
            cache.sync_token(&calendar("2", "work"), window()).unwrap(),
            None
        );

        // A calendar gone from the list loses its state, as does a forgotten account.
        cache
            .set_calendars("1", std::slice::from_ref(&work))
            .unwrap();
        assert_eq!(cache.sync_token(&home, window()).unwrap(), None);
        cache.set_sync_token(&work, None, window()).unwrap();
        assert_eq!(cache.sync_token(&work, window()).unwrap(), None);
        cache
            .set_sync_token(&work, Some("token-4"), window())
            .unwrap();
        cache.forget_account("1").unwrap();
        assert_eq!(cache.sync_token(&work, window()).unwrap(), None);
    }

    #[test]
    fn a_sync_token_is_given_only_for_the_window_it_was_kept_for() {
        let mut cache = Cache::in_memory();
        let work = calendar("1", "work");
        cache
            .set_calendars("1", std::slice::from_ref(&work))
            .unwrap();
        cache
            .set_sync_token(&work, Some("token"), window())
            .unwrap();

        let next_day = Window::around(d(2026, 11, 12), Warsaw);

        assert_eq!(cache.sync_token(&work, next_day).unwrap(), None);
        assert_eq!(
            cache.sync_token(&work, window()).unwrap().as_deref(),
            Some("token")
        );
    }

    #[test]
    fn a_cleared_calendar_stays_listed_without_events_or_sync_token() {
        let mut cache = Cache::in_memory();
        let (work, home) = (calendar("1", "work"), calendar("1", "home"));
        cache
            .set_calendars("1", &[work.clone(), home.clone()])
            .unwrap();
        for calendar in [&work, &home] {
            cache.set_events(calendar, &events()).unwrap();
            cache
                .set_sync_token(calendar, Some("token"), window())
                .unwrap();
        }

        cache.clear_calendar("1", "home").unwrap();

        assert_eq!(cache.calendar_list().unwrap(), [work.clone(), home.clone()]);
        let cached = cache.calendars().unwrap();
        assert_eq!(cached[0].events.len(), 3);
        assert!(cached[1].events.is_empty());
        assert_eq!(cache.sync_token(&home, window()).unwrap(), None);
        assert!(cache.sync_token(&work, window()).unwrap().is_some());
    }

    #[test]
    fn colours_are_read_as_google_writes_them() {
        assert_eq!(parse_color("#9fe1e7"), Some(Rgb(0x9f, 0xe1, 0xe7)));
        assert_eq!(parse_color("9fe1e7"), None);
        assert_eq!(parse_color("#9fe1e"), None);
    }
}
