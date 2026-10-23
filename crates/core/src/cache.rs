//! The local cache (`DEC-23`): every account's calendars and their synchronized events, in
//! SQLite. It is the read model of the views; only synchronization writes to it. Nothing in it is
//! secret, and it can always be rebuilt from Google (`NFR-17`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, ErrorCode, params};

use crate::model::{Access, Calendar, Rgb, Timing};
use crate::sync::{Start, SyncError, SyncedEvent};

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
        PRIMARY KEY (account_id, calendar_id, id)
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
        let connection = Connection::open(path)?;
        connection.execute_batch(SCHEMA)?;
        Ok(Cache { connection })
    }

    /// A cache that lives in memory only, for tests.
    pub fn in_memory() -> Cache {
        let connection = Connection::open_in_memory().expect("an in-memory database");
        connection.execute_batch(SCHEMA).expect("the cache schema");
        Cache { connection }
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
        transaction
            .execute(
                "DELETE FROM events WHERE account_id = ?1 AND calendar_id NOT IN \
                 (SELECT id FROM calendars WHERE account_id = ?1)",
                params![account_id],
            )
            .map_err(failed)?;
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
        {
            let mut insert = transaction
                .prepare(
                    "INSERT OR REPLACE INTO events VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
                    Some((series, original)) => {
                        (Some(series.as_str()), Some(start_text(*original)))
                    }
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
                    ])
                    .map_err(failed)?;
            }
        }
        transaction.commit().map_err(failed)
    }

    /// Deletes everything kept for an account (`MOD-01-F08`, `DEC-85`).
    pub fn forget_account(&mut self, account_id: &str) -> Result<(), SyncError> {
        let transaction = self.connection.transaction().map_err(failed)?;
        for table in ["calendars", "events"] {
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

    /// Every cached calendar with its events, each account's calendars in the order Google lists
    /// them.
    pub fn calendars(&self) -> Result<Vec<CachedCalendar>, SyncError> {
        let unreadable = || SyncError("the local cache could not be read".into());
        let mut calendars: Vec<CachedCalendar> = self
            .connection
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
                Some(CachedCalendar {
                    calendar: Calendar {
                        id,
                        account_id,
                        name,
                        color: parse_color(&color)?,
                        access: parse_access(&access)?,
                        zone: zone.parse().ok()?,
                    },
                    events: Vec::new(),
                })
            })
            .collect::<Option<_>>()
            .ok_or_else(unreadable)?;
        let index: HashMap<(String, String), usize> = calendars
            .iter()
            .enumerate()
            .map(|(i, c)| ((c.calendar.account_id.clone(), c.calendar.id.clone()), i))
            .collect();
        let mut select = self
            .connection
            .prepare(
                "SELECT account_id, calendar_id, id, title, start, end, zone, recurrence, \
                 series_id, original_start FROM events",
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
            };
            if let Some(&index) = index.get(&(account_id, calendar_id)) {
                calendars[index].events.push(event);
            }
        }
        Ok(calendars)
    }
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

fn color_text(Rgb(r, g, b): Rgb) -> String {
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
    fn colours_are_read_as_google_writes_them() {
        assert_eq!(parse_color("#9fe1e7"), Some(Rgb(0x9f, 0xe1, 0xe7)));
        assert_eq!(parse_color("9fe1e7"), None);
        assert_eq!(parse_color("#9fe1e"), None);
    }
}
