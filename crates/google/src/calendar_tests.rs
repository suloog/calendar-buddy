use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::*;
use calendar_core::cache::CachedCalendar;
use calendar_core::details::event_details;
use calendar_core::fixtures::d;
use calendar_core::model::{Access, Event, Rgb};
use calendar_core::sync::shown_events;
use calendar_core::view_model::grid_view;
use calendar_core::week::Week;
use chrono::{TimeZone, Weekday};
use chrono_tz::Europe::{London, Warsaw};
use serde_json::{Value, json};

use crate::CLIENT_FILE;
use crate::fake::{ACCOUNT_ID, CLIENT_ID, FakeGoogle, FakeStore, OTHER_ID};

/// The fake clock's today: a Wednesday, in the week of 9–15 November 2026.
fn today() -> NaiveDate {
    d(2026, 11, 11)
}

fn window() -> Window {
    Window::around(today(), Warsaw)
}

/// A client file pointing at the test client, in a directory of the test's own.
fn client_file(name: &str) -> (PathBuf, PathBuf) {
    let dir =
        std::env::temp_dir().join(format!("calendar-buddy-sync-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(CLIENT_FILE);
    std::fs::write(
        &path,
        format!(r#"{{"installed":{{"client_id":"{CLIENT_ID}","client_secret":"test-secret"}}}}"#),
    )
    .unwrap();
    (dir, path)
}

/// An account that signed in before: Google and the secret store both know its refresh token.
fn signed_in(google: &FakeGoogle, store: &FakeStore, account: &'static str) {
    let token = google.signed_in(account);
    store
        .saved
        .lock()
        .unwrap()
        .push((account.to_string(), token));
}

fn entry(id: &str, name: &str, color: &str, role: &str) -> Value {
    json!({
        "kind": "calendar#calendarListEntry",
        "id": id,
        "summary": name,
        "timeZone": "Europe/Warsaw",
        "colorId": "14",
        "backgroundColor": color,
        "foregroundColor": "#000000",
        "accessRole": role,
    })
}

/// A timed event in Europe/Warsaw, as Google sends one.
fn timed(id: &str, title: &str, start: &str, end: &str) -> Value {
    json!({
        "kind": "calendar#event",
        "id": id,
        "status": "confirmed",
        "summary": title,
        "start": {"dateTime": start, "timeZone": "Europe/Warsaw"},
        "end": {"dateTime": end, "timeZone": "Europe/Warsaw"},
    })
}

fn sync(
    google: &FakeGoogle,
    store: &FakeStore,
    client_file: &Path,
    account: &str,
    cache: &mut Cache,
) -> Result<(), SyncError> {
    sync_account(
        client_file,
        &google.endpoints,
        store,
        account,
        window(),
        cache,
    )
}

fn local(at: DateTime<Utc>) -> String {
    at.with_timezone(&Warsaw).format("%F %R").to_string()
}

/// The timed events shown in the week containing `date`, in Warsaw: title, start and end.
fn week_of(events: &[Event], date: NaiveDate) -> Vec<(String, String, String)> {
    let days = Week::containing(date, Weekday::Mon).days();
    let now = Utc.with_ymd_and_hms(2026, 11, 11, 12, 0, 0).unwrap();
    let mut blocks: Vec<_> = grid_view(events, &days, Warsaw, now)
        .blocks
        .into_iter()
        .map(|b| (b.title.unwrap(), local(b.start), local(b.end)))
        .collect();
    blocks.sort_by(|a, b| a.1.cmp(&b.1));
    blocks
}

#[test]
fn tc_02_01_calendars_of_two_accounts_are_fetched_and_shown_together_in_their_colours() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("tc-02-01");
    // Account A has two own calendars; account B one own calendar and a subscribed holiday one.
    let holidays = "en.polish#holiday@group.v.calendar.google.com";
    let calendars = [
        (ACCOUNT_ID, "owner@example.com", "Owner", "#9fe1e7", "owner"),
        (ACCOUNT_ID, "team@example.com", "Team", "#16a765", "writer"),
        (OTHER_ID, "other@example.com", "Home", "#f83a22", "owner"),
        (
            OTHER_ID,
            holidays,
            "Holidays in Poland",
            "#7bd148",
            "reader",
        ),
    ];
    for (n, (account, id, name, color, role)) in calendars.into_iter().enumerate() {
        google.add_calendar(account, entry(id, name, color, role));
        let day = 9 + n;
        google.add_event(
            account,
            id,
            timed(
                &format!("event{n}"),
                name,
                &format!("2026-11-{day:02}T10:00:00+01:00"),
                &format!("2026-11-{day:02}T11:00:00+01:00"),
            ),
        );
    }
    signed_in(&google, &store, ACCOUNT_ID);
    signed_in(&google, &store, OTHER_ID);
    let mut cache = Cache::in_memory();

    // 1. The first synchronization finishes: all four calendars are known to the app.
    let a = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    let b = sync(&google, &store, &client_file, OTHER_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!((a, b), (Ok(()), Ok(())));
    let cached = cache.calendars().unwrap();
    let mut known: Vec<(&str, &str, Rgb, Access)> = cached
        .iter()
        .map(|c| {
            (
                c.calendar.account_id.as_str(),
                c.calendar.id.as_str(),
                c.calendar.color,
                c.calendar.access,
            )
        })
        .collect();
    // Each account's calendars in Google's order.
    known.sort_by_key(|(account, ..)| *account != ACCOUNT_ID);
    assert_eq!(
        known,
        [
            (
                ACCOUNT_ID,
                "owner@example.com",
                Rgb(0x9f, 0xe1, 0xe7),
                Access::Owner
            ),
            (
                ACCOUNT_ID,
                "team@example.com",
                Rgb(0x16, 0xa7, 0x65),
                Access::Writer
            ),
            (
                OTHER_ID,
                "other@example.com",
                Rgb(0xf8, 0x3a, 0x22),
                Access::Owner
            ),
            (OTHER_ID, holidays, Rgb(0x7b, 0xd1, 0x48), Access::Reader),
        ]
    );

    // 2. The week's view holds the four events, each in its calendar's Google colour.
    let days = Week::containing(today(), Weekday::Mon).days();
    let now = Utc.with_ymd_and_hms(2026, 11, 11, 12, 0, 0).unwrap();
    let view = grid_view(&shown_events(&cached, window()), &days, Warsaw, now);
    let mut blocks: Vec<(String, Rgb)> = view
        .blocks
        .iter()
        .map(|b| (b.title.clone().unwrap(), b.color))
        .collect();
    blocks.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        blocks,
        [
            ("Holidays in Poland".to_string(), Rgb(0x7b, 0xd1, 0x48)),
            ("Home".to_string(), Rgb(0xf8, 0x3a, 0x22)),
            ("Owner".to_string(), Rgb(0x9f, 0xe1, 0xe7)),
            ("Team".to_string(), Rgb(0x16, 0xa7, 0x65)),
        ]
    );
}

#[test]
fn tc_02_02_other_zones_daylight_saving_and_changed_occurrences_show_at_the_right_local_times() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("tc-02-02");
    let calendar = "owner@example.com";
    google.add_calendar(ACCOUNT_ID, entry(calendar, "Owner", "#9fe1e7", "owner"));
    // Event 1: 15:00–16:00 in Europe/London on a day in June.
    google.add_event(
        ACCOUNT_ID,
        calendar,
        json!({
            "id": "london",
            "status": "confirmed",
            "summary": "London",
            "start": {"dateTime": "2027-06-15T15:00:00+01:00", "timeZone": "Europe/London"},
            "end": {"dateTime": "2027-06-15T16:00:00+01:00", "timeZone": "Europe/London"},
        }),
    );
    // Event 2: 01:30–03:30 Europe/Warsaw on the last Sunday of March, when clocks go forward.
    google.add_event(
        ACCOUNT_ID,
        calendar,
        timed(
            "dst",
            "Night shift",
            "2027-03-28T01:30:00+01:00",
            "2027-03-28T03:30:00+02:00",
        ),
    );
    // The series: weekly on Monday 09:00, 5 occurrences; the 3rd moved to 11:00, the 4th
    // cancelled.
    let mut series = timed(
        "series",
        "Weekly",
        "2026-11-02T09:00:00+01:00",
        "2026-11-02T10:00:00+01:00",
    );
    series["recurrence"] = json!(["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=5"]);
    google.add_event(ACCOUNT_ID, calendar, series);
    let mut moved = timed(
        "series_20261116T080000Z",
        "Weekly",
        "2026-11-16T11:00:00+01:00",
        "2026-11-16T12:00:00+01:00",
    );
    moved["recurringEventId"] = json!("series");
    moved["originalStartTime"] =
        json!({"dateTime": "2026-11-16T09:00:00+01:00", "timeZone": "Europe/Warsaw"});
    google.add_event(ACCOUNT_ID, calendar, moved);
    google.add_event(
        ACCOUNT_ID,
        calendar,
        json!({
            "id": "series_20261123T080000Z",
            "status": "cancelled",
            "recurringEventId": "series",
            "originalStartTime": {"dateTime": "2026-11-23T09:00:00+01:00", "timeZone": "Europe/Warsaw"},
        }),
    );
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();

    // 1. The synchronization finishes; the events of the relevant weeks are read.
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(synced, Ok(()));
    let events = shown_events(&cache.calendars().unwrap(), window());
    let row = |title: &str, start: &str, end: &str| {
        (title.to_string(), start.to_string(), end.to_string())
    };
    assert_eq!(
        week_of(&events, d(2027, 6, 15)),
        [row("London", "2027-06-15 16:00", "2027-06-15 17:00")]
    );
    assert_eq!(
        week_of(&events, d(2027, 3, 28)),
        [row("Night shift", "2027-03-28 01:30", "2027-03-28 03:30")]
    );
    let night = events.iter().find(|e| e.id == "dst").unwrap();
    let Timing::Timed { start, end, .. } = night.timing else {
        panic!("a timed event");
    };
    assert_eq!(end - start, chrono::Duration::hours(1));
    let london = events.iter().find(|e| e.id == "london").unwrap();
    assert!(matches!(london.timing, Timing::Timed { zone, .. } if zone == London));

    // 2. The occurrences of the series: 1, 2 and 5 at 09:00, 3 at 11:00, 4 absent.
    let mondays: Vec<_> = [2, 9, 16, 23, 30]
        .into_iter()
        .flat_map(|day| week_of(&events, d(2026, 11, day)))
        .collect();
    assert_eq!(
        mondays,
        [
            row("Weekly", "2026-11-02 09:00", "2026-11-02 10:00"),
            row("Weekly", "2026-11-09 09:00", "2026-11-09 10:00"),
            row("Weekly", "2026-11-16 11:00", "2026-11-16 12:00"),
            row("Weekly", "2026-11-30 09:00", "2026-11-30 10:00"),
        ]
    );
}

#[test]
fn tc_02_03_read_only_events_are_marked_and_free_busy_ones_are_busy_without_details() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("tc-02-03");
    // An own calendar, a colleague's shared read-only and a room shared as free/busy only.
    let calendars = [
        ("owner@example.com", "Owner", "#9fe1e7", "owner", "Standup"),
        (
            "colleague@example.com",
            "Colleague",
            "#16a765",
            "reader",
            "1:1",
        ),
        (
            "room@example.com",
            "Room",
            "#f83a22",
            "freeBusyReader",
            "Board",
        ),
    ];
    for (n, (id, name, color, role, title)) in calendars.into_iter().enumerate() {
        google.add_calendar(ACCOUNT_ID, entry(id, name, color, role));
        let day = 9 + n;
        google.add_event(
            ACCOUNT_ID,
            id,
            timed(
                &format!("event{n}"),
                title,
                &format!("2026-11-{day:02}T10:00:00+01:00"),
                &format!("2026-11-{day:02}T11:00:00+01:00"),
            ),
        );
    }
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(synced, Ok(()));
    let events = shown_events(&cache.calendars().unwrap(), window());
    let days = Week::containing(today(), Weekday::Mon).days();
    let now = Utc.with_ymd_and_hms(2026, 11, 11, 12, 0, 0).unwrap();
    let mut blocks = grid_view(&events, &days, Warsaw, now).blocks;
    blocks.sort_by_key(|b| b.start);
    let shown: Vec<(Option<&str>, Rgb, bool)> = blocks
        .iter()
        .map(|b| (b.title.as_deref(), b.color, b.read_only))
        .collect();

    // 1. The colleague's event is shown like the others, and marked as not editable.
    // 2. The room's event is a "busy" block, without its title, also when it is opened.
    assert_eq!(
        shown,
        [
            (Some("Standup"), Rgb(0x9f, 0xe1, 0xe7), false),
            (Some("1:1"), Rgb(0x16, 0xa7, 0x65), true),
            (None, Rgb(0xf8, 0x3a, 0x22), true),
        ]
    );
    let opened = |id: &str| event_details(events.iter().find(|e| e.id == id).unwrap(), Warsaw);
    assert!(!opened("event0").read_only);
    assert!(opened("event1").read_only);
    assert_eq!(opened("event1").title.as_deref(), Some("1:1"));
    assert!(opened("event2").read_only);
    assert_eq!(opened("event2").title, None);
}

#[test]
fn a_calendar_that_became_read_only_is_marked_so_at_the_next_synchronization() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("access-dropped");
    google.add_calendar(ACCOUNT_ID, entry("team", "Team", "#16a765", "writer"));
    google.add_event(
        ACCOUNT_ID,
        "team",
        timed(
            "planning",
            "Planning",
            "2026-11-10T10:00:00+01:00",
            "2026-11-10T11:00:00+01:00",
        ),
    );
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();
    let read_only = |cache: &Cache| {
        shown_events(&cache.calendars().unwrap(), window())
            .iter()
            .map(|e| e.read_only)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache),
        Ok(())
    );
    assert_eq!(read_only(&cache), [false]);
    google.set_access_role("team", "reader");

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    assert_eq!(read_only(&cache), [true]);
}

#[test]
fn events_are_asked_for_within_the_cache_window_with_series_unexpanded() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("window");
    let holidays = "en.polish#holiday@group.v.calendar.google.com";
    google.add_calendar(
        ACCOUNT_ID,
        entry(holidays, "Holidays in Poland", "#7bd148", "reader"),
    );
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let asked = google
        .requests()
        .into_iter()
        .find(|r| r.contains("/events"))
        .unwrap();
    let url = Url::parse(&format!("http://127.0.0.1{}", &asked["GET ".len()..])).unwrap();
    let param = |name: &str| crate::fake::param(&url, name);
    assert_eq!(
        url.path(),
        "/calendar/v3/calendars/en.polish%23holiday@group.v.calendar.google.com/events"
    );
    // One year back to two years ahead of today, in the system's time zone.
    assert_eq!(param("timeMin").as_deref(), Some("2025-11-10T23:00:00Z"));
    assert_eq!(param("timeMax").as_deref(), Some("2028-11-10T23:00:00Z"));
    assert_eq!(param("singleEvents"), None);
}

#[test]
fn an_event_crossing_the_window_edge_is_kept_whole() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("edge");
    let calendar = "owner@example.com";
    google.add_calendar(ACCOUNT_ID, entry(calendar, "Owner", "#9fe1e7", "owner"));
    google.add_event(
        ACCOUNT_ID,
        calendar,
        json!({
            "id": "trip",
            "status": "confirmed",
            "summary": "Trip",
            "start": {"date": "2025-11-08"},
            "end": {"date": "2025-11-14"},
        }),
    );
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let events = shown_events(&cache.calendars().unwrap(), window());
    assert_eq!(
        events[0].timing,
        Timing::AllDay {
            start: d(2025, 11, 8),
            end_exclusive: d(2025, 11, 14)
        }
    );
}

#[test]
fn a_series_started_before_the_window_has_its_occurrences_inside_it() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("old-series");
    let calendar = "owner@example.com";
    google.add_calendar(ACCOUNT_ID, entry(calendar, "Owner", "#9fe1e7", "owner"));
    let mut series = timed(
        "standup",
        "Standup",
        "2019-01-07T09:00:00+01:00",
        "2019-01-07T09:15:00+01:00",
    );
    series["recurrence"] = json!(["RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"]);
    google.add_event(ACCOUNT_ID, calendar, series);
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let events = shown_events(&cache.calendars().unwrap(), window());
    let this_week: Vec<String> = week_of(&events, today())
        .into_iter()
        .map(|(_, start, _)| start)
        .collect();
    assert_eq!(
        this_week,
        [
            "2026-11-09 09:00",
            "2026-11-10 09:00",
            "2026-11-11 09:00",
            "2026-11-12 09:00",
            "2026-11-13 09:00"
        ]
    );
    assert!(events.iter().all(|e| window().overlaps(&e.timing)));
}

#[test]
fn a_calendar_google_refuses_keeps_what_it_had_and_the_others_synchronize() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("failing");
    for (id, title) in [("work", "Planning"), ("home", "Dentist")] {
        google.add_calendar(ACCOUNT_ID, entry(id, id, "#9fe1e7", "owner"));
        google.add_event(
            ACCOUNT_ID,
            id,
            timed(
                title,
                title,
                "2026-11-10T10:00:00+01:00",
                "2026-11-10T11:00:00+01:00",
            ),
        );
    }
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();
    assert_eq!(
        sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache),
        Ok(())
    );
    google.add_event(
        ACCOUNT_ID,
        "home",
        timed(
            "new",
            "New",
            "2026-11-12T10:00:00+01:00",
            "2026-11-12T11:00:00+01:00",
        ),
    );
    google.fail_calendar("work");

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(synced.is_err());
    let titles = |cached: &CachedCalendar| {
        cached
            .events
            .iter()
            .map(|e| e.title.clone())
            .collect::<Vec<_>>()
    };
    let cached = cache.calendars().unwrap();
    assert_eq!(titles(&cached[0]), ["Planning"]);
    assert_eq!(titles(&cached[1]), ["Dentist", "New"]);
}

#[test]
fn an_account_google_turned_away_leaves_the_cache_as_it_was() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("revoked");
    google.add_calendar(ACCOUNT_ID, entry("work", "Work", "#9fe1e7", "owner"));
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();
    assert_eq!(
        sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache),
        Ok(())
    );
    google.revoke(ACCOUNT_ID);

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(synced.is_err());
    assert_eq!(cache.calendars().unwrap().len(), 1);
}

#[test]
fn calendars_renamed_by_the_user_or_free_busy_are_listed_as_google_lists_them() {
    let mut renamed = entry("team@example.com", "Team", "#16a765", "reader");
    renamed["summaryOverride"] = json!("My team");
    let calendar = serde_json::from_value::<CalendarEntry>(renamed)
        .unwrap()
        .into_calendar(ACCOUNT_ID)
        .unwrap();
    assert_eq!(calendar.name, "My team");
    assert_eq!(calendar.zone, Warsaw);
    let free_busy = entry("room@example.com", "Room", "#16a765", "freeBusyReader");
    let calendar = serde_json::from_value::<CalendarEntry>(free_busy)
        .unwrap()
        .into_calendar(ACCOUNT_ID)
        .unwrap();
    assert_eq!(calendar.access, Access::FreeBusyReader);
}

#[test]
fn an_event_without_a_zone_of_its_own_takes_the_calendars() {
    let event: GoogleEvent = serde_json::from_value(json!({
        "id": "noon",
        "summary": "Noon",
        "start": {"dateTime": "2026-11-10T12:00:00Z"},
        "end": {"dateTime": "2026-11-10T13:00:00Z"},
    }))
    .unwrap();

    let synced = event.into_synced(London).unwrap();

    assert_eq!(
        synced.timing,
        Some(Timing::Timed {
            start: Utc.with_ymd_and_hms(2026, 11, 10, 12, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 11, 10, 13, 0, 0).unwrap(),
            zone: London
        })
    );
}

/// The profile of `DEC-14`: 5 accounts, 50 calendars, 20,000 events.
#[test]
fn the_first_full_synchronization_of_the_profile_runs_in_the_background() {
    const ACCOUNTS: [&str; 5] = ["a1", "a2", "a3", "a4", "a5"];
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("profile");
    for account in ACCOUNTS {
        signed_in(&google, &store, account);
        for c in 0..10 {
            let id = format!("{account}-calendar-{c}@example.com");
            google.add_calendar(account, entry(&id, &id, "#9fe1e7", "owner"));
            for n in 0..400 {
                let day = today() - chrono::Duration::days(365) + chrono::Duration::days(n * 2);
                google.add_event(
                    account,
                    &id,
                    timed(
                        &format!("e{n}"),
                        "Event",
                        &format!("{day}T10:00:00+01:00"),
                        &format!("{day}T11:00:00+01:00"),
                    ),
                );
            }
        }
    }
    let endpoints = google.endpoints.clone();
    let started = Instant::now();

    let (sender, done) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let mut cache = Cache::in_memory();
        for account in ACCOUNTS {
            sync_account(
                &client_file,
                &endpoints,
                &store,
                account,
                window(),
                &mut cache,
            )
            .unwrap();
        }
        let shown = shown_events(&cache.calendars().unwrap(), window());
        sender.send(shown.len()).unwrap();
    });
    // The interface's thread is never held: it only looks whether the result has come.
    let mut frames = 0;
    let shown = loop {
        match done.try_recv() {
            Ok(shown) => break shown,
            Err(_) => std::thread::sleep(Duration::from_millis(16)),
        }
        frames += 1;
    };
    worker.join().unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(shown, 20_000);
    assert!(frames > 0);
    eprintln!(
        "first full synchronization of 5 accounts, 50 calendars, 20,000 events: {:?}",
        started.elapsed()
    );
}

/// The titles shown in the week of `today()`, in order.
fn shown_titles(cache: &Cache) -> Vec<String> {
    week_of(
        &shown_events(&cache.calendars().unwrap(), window()),
        today(),
    )
    .into_iter()
    .map(|(title, ..)| title)
    .collect()
}

/// The query of each request for the calendar's events.
fn event_queries(google: &FakeGoogle) -> Vec<Url> {
    google
        .requests()
        .into_iter()
        .filter(|r| r.contains("/events"))
        .map(|r| Url::parse(&format!("http://127.0.0.1{}", &r["GET ".len()..])).unwrap())
        .collect()
}

/// An editable calendar with "Standup" on Tuesday and "Review" on Wednesday, synchronized once.
fn synchronized_once(name: &str) -> (FakeGoogle, FakeStore, PathBuf, PathBuf, Cache) {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file(name);
    google.add_calendar(ACCOUNT_ID, entry("work", "Work", "#9fe1e7", "owner"));
    for (id, title, day) in [("standup", "Standup", 10), ("review", "Review", 11)] {
        google.add_event(
            ACCOUNT_ID,
            "work",
            timed(
                id,
                title,
                &format!("2026-11-{day}T09:00:00+01:00"),
                &format!("2026-11-{day}T10:00:00+01:00"),
            ),
        );
    }
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();
    assert_eq!(
        sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache),
        Ok(())
    );
    assert_eq!(shown_titles(&cache), ["Standup", "Review"]);
    (google, store, dir, client_file, cache)
}

#[test]
fn after_the_first_full_synchronization_only_changes_are_asked_for() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("incremental");
    google.add_event(
        ACCOUNT_ID,
        "work",
        timed(
            "standup",
            "Standup moved",
            "2026-11-12T09:00:00+01:00",
            "2026-11-12T10:00:00+01:00",
        ),
    );
    google.delete_event(ACCOUNT_ID, "work", "review");
    google.add_event(
        ACCOUNT_ID,
        "work",
        timed(
            "retro",
            "Retro",
            "2026-11-13T09:00:00+01:00",
            "2026-11-13T10:00:00+01:00",
        ),
    );

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    assert_eq!(shown_titles(&cache), ["Standup moved", "Retro"]);
    let asked = event_queries(&google);
    assert_eq!(asked.len(), 2);
    let param = |name: &str| crate::fake::param(&asked[1], name);
    assert_eq!(param("syncToken").as_deref(), Some("state-2"));
    assert_eq!((param("timeMin"), param("timeMax")), (None, None));
}

#[test]
fn a_calendar_whose_sync_token_google_rejects_is_synchronized_in_full_again() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("gone");
    google.delete_event(ACCOUNT_ID, "work", "review");
    google.expire_sync_tokens();

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    assert_eq!(shown_titles(&cache), ["Standup"]);
    let asked = event_queries(&google);
    let param = |n: usize, name: &str| crate::fake::param(&asked[n], name);
    assert_eq!(asked.len(), 3);
    assert_eq!(param(1, "syncToken").as_deref(), Some("state-2"));
    assert_eq!(param(2, "syncToken"), None);
    assert!(param(2, "timeMin").is_some());
}

#[test]
fn an_access_token_google_no_longer_takes_is_renewed_without_failing() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("expired-access");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    google.delete_event(ACCOUNT_ID, "work", "review");
    google.expire_access_tokens();
    let calendars: Vec<Calendar> = cache
        .calendars()
        .unwrap()
        .into_iter()
        .map(|c| c.calendar)
        .collect();

    let synced = calendar_core::sync::sync_calendars(&session, &calendars, window(), &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(true));
    assert_eq!(shown_titles(&cache), ["Standup"]);
}
