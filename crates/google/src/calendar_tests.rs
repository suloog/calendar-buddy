use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::*;
use calendar_core::availability::Availability;
use calendar_core::cache::CachedCalendar;
use calendar_core::colors::EventColor;
use calendar_core::create::NewEvent;
use calendar_core::details::event_details;
use calendar_core::editor::Patch;
use calendar_core::fixtures::d;
use calendar_core::guests::{Guest, Person, Reach, Response};
use calendar_core::model::{
    About, Access, Attachment, Event, Kind, Method, Reminder, Reminders, Rgb, Show, Visibility,
};
use calendar_core::sync::shown_events;
use calendar_core::view_model::grid_view;
use calendar_core::week::Week;
use chrono::{TimeZone, Weekday};
use chrono_tz::Europe::{London, Warsaw};
use serde_json::{Value, json};

use crate::CLIENT_FILE;
use crate::fake::{ACCOUNT_ID, CLIENT_ID, FakeGoogle, FakeStore, OTHER_ID, param};

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

    assert_eq!(synced, [Ok(true)]);
    assert_eq!(shown_titles(&cache), ["Standup"]);
}

#[test]
fn googles_rate_limits_are_told_from_other_errors() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("rate-limited");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap();
    let mut errors = Vec::new();
    for status in ["429 Too Many Requests", "403 Forbidden"] {
        google.limit_calendar("work", status);
        errors.push(calendar_core::sync::sync_calendars(
            &session,
            &work,
            window(),
            &mut cache,
        ));
        google.recover();
    }
    google.fail_calendar("work");
    errors.push(calendar_core::sync::sync_calendars(
        &session,
        &work,
        window(),
        &mut cache,
    ));
    std::fs::remove_dir_all(&dir).unwrap();

    let rate_limited: Vec<bool> = errors
        .iter()
        .map(|synced| synced[0].as_ref().unwrap_err().is_rate_limited())
        .collect();
    assert_eq!(rate_limited, [true, true, false]);
    assert_eq!(shown_titles(&cache), ["Standup", "Review"]);
}

/// "Planning" on Thursday 12 November 2026, 10:00–11:30 in Warsaw.
fn planning() -> NewEvent {
    NewEvent {
        title: "Planning".into(),
        timing: Timing::Timed {
            start: Warsaw
                .with_ymd_and_hms(2026, 11, 12, 10, 0, 0)
                .unwrap()
                .with_timezone(&Utc),
            end: Warsaw
                .with_ymd_and_hms(2026, 11, 12, 11, 30, 0)
                .unwrap()
                .with_timezone(&Utc),
            zone: Warsaw,
        },
        about: About::default(),
        recurrence: Vec::new(),
        meet: false,
    }
}

#[test]
fn a_new_event_is_sent_to_google_and_comes_back_through_synchronization() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("create");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    // An access token Google no longer takes is renewed for a change too.
    google.expire_access_tokens();

    let id = session.create(&work, &planning()).unwrap();

    let sent = google.event(ACCOUNT_ID, "work", &id).unwrap();
    assert_eq!(sent["summary"], "Planning");
    assert_eq!(
        sent["start"],
        json!({"dateTime": "2026-11-12T10:00:00+01:00", "timeZone": "Europe/Warsaw"})
    );
    assert_eq!(
        sent["end"],
        json!({"dateTime": "2026-11-12T11:30:00+01:00", "timeZone": "Europe/Warsaw"})
    );
    assert!(
        google
            .requests()
            .contains(&"POST /calendar/v3/calendars/work/events".to_string())
    );
    // Nothing is written to the cache but by synchronization.
    assert_eq!(shown_titles(&cache), ["Standup", "Review"]);

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let week = week_of(
        &shown_events(&cache.calendars().unwrap(), window()),
        today(),
    );
    assert!(week.contains(&(
        "Planning".to_string(),
        "2026-11-12 10:00".to_string(),
        "2026-11-12 11:30".to_string()
    )));
}

#[test]
fn a_new_event_in_a_calendar_google_does_not_let_the_account_change_is_refused() {
    let (google, store, dir, client_file, cache) = synchronized_once("create-refused");
    google.set_access_role("work", "reader");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);

    let created = session.create(&work, &planning());
    std::fs::remove_dir_all(&dir).unwrap();

    let error = created.unwrap_err();
    assert!(error.0.starts_with("Google refused"), "{}", error.0);
    assert!(!error.0.contains("Planning"));
    assert!(!error.is_rate_limited());
}

#[test]
fn the_default_length_of_new_events_comes_from_the_accounts_google_calendar_settings() {
    let (google, store, dir, client_file, _) = synchronized_once("default-length");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();

    // Google's own default, until the user sets another.
    let before = session.default_length();
    google.set_default_length(ACCOUNT_ID, 30);
    let after = session.default_length();
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!((before, after), (Ok(60), Ok(30)));
    assert!(
        google
            .requests()
            .contains(&"GET /calendar/v3/users/me/settings/defaultEventLength".to_string())
    );
}

/// The cached event of the calendar with the ID.
fn cached(cache: &Cache, calendar: &str, id: &str) -> Option<SyncedEvent> {
    cache
        .calendars()
        .unwrap()
        .into_iter()
        .filter(|cached| cached.calendar.id == calendar)
        .flat_map(|cached| cached.events)
        .find(|event| event.id == id)
}

#[test]
fn events_come_with_their_description_location_busy_or_free_visibility_and_version() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("about");
    let mut review = timed(
        "review",
        "Review",
        "2026-11-11T09:00:00+01:00",
        "2026-11-11T10:00:00+01:00",
    );
    review["description"] = json!("Q4 numbers");
    review["location"] = json!("Room 2");
    review["transparency"] = json!("transparent");
    review["visibility"] = json!("confidential");
    google.add_event(ACCOUNT_ID, "work", review);

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let review = cached(&cache, "work", "review").unwrap();
    assert_eq!(
        review.about,
        About {
            description: "Q4 numbers".into(),
            location: "Room 2".into(),
            show: Show::Free,
            // Google keeps "confidential" for compatibility; it means private.
            visibility: Visibility::Private,
            kind: Kind::Ordinary,
            color: None,
            attachments: Vec::new(),
            guests: Vec::new(),
            meet: None,
            reminders: Reminders::Default,
        }
    );
    assert_eq!(
        review.etag,
        google.event(ACCOUNT_ID, "work", "review").unwrap()["etag"]
    );
    let standup = cached(&cache, "work", "standup").unwrap();
    assert_eq!(standup.about, About::default());
}

#[test]
fn events_come_with_their_google_type() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("types");
    for (id, kind) in [
        ("away", "outOfOffice"),
        ("focus", "focusTime"),
        ("office", "workingLocation"),
        ("anna", "birthday"),
        ("mail", "fromGmail"),
    ] {
        let mut event = timed(
            id,
            id,
            "2026-11-12T09:00:00+01:00",
            "2026-11-12T10:00:00+01:00",
        );
        event["eventType"] = json!(kind);
        google.add_event(ACCOUNT_ID, "work", event);
    }

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let kind = |id| cached(&cache, "work", id).unwrap().about.kind;
    assert_eq!(kind("away"), Kind::OutOfOffice);
    assert_eq!(kind("focus"), Kind::FocusTime);
    assert_eq!(kind("office"), Kind::WorkingLocation);
    assert_eq!(kind("anna"), Kind::Birthday);
    // Other types Google has are shown as ordinary events.
    assert_eq!(kind("mail"), Kind::Ordinary);
    assert_eq!(kind("review"), Kind::Ordinary);
}

#[test]
fn out_of_office_and_focus_time_are_sent_with_their_type_and_googles_defaults() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("create-types");
    // Google takes these types only in a primary calendar.
    google.remove_calendar(ACCOUNT_ID, "work");
    let mut primary = entry("work", "Work", "#9fe1e7", "owner");
    primary["primary"] = json!(true);
    google.add_calendar(ACCOUNT_ID, primary);
    assert_eq!(
        sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache),
        Ok(())
    );
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let new = |kind| NewEvent {
        about: About {
            kind,
            ..About::default()
        },
        ..planning()
    };

    let away = session.create(&work, &new(Kind::OutOfOffice)).unwrap();
    let focus = session.create(&work, &new(Kind::FocusTime)).unwrap();
    let planning = session.create(&work, &planning()).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    let away = google.event(ACCOUNT_ID, "work", &away).unwrap();
    assert_eq!(away["eventType"], "outOfOffice");
    assert_eq!(
        away["outOfOfficeProperties"],
        json!({
            "autoDeclineMode": "declineOnlyNewConflictingInvitations",
            "declineMessage": "Declined because I am out of office",
        })
    );
    assert_eq!(away["transparency"], "opaque");
    let focus = google.event(ACCOUNT_ID, "work", &focus).unwrap();
    assert_eq!(focus["eventType"], "focusTime");
    assert_eq!(
        focus["focusTimeProperties"],
        json!({
            "autoDeclineMode": "declineOnlyNewConflictingInvitations",
            "declineMessage": "Declined because I am in focus time",
            "chatStatus": "doNotDisturb",
        })
    );
    // An ordinary event is sent as before, without a type.
    let planning = google.event(ACCOUNT_ID, "work", &planning).unwrap();
    assert_eq!(planning.get("eventType"), None);
}

#[test]
fn events_come_with_their_reminders_and_calendars_with_their_default_ones() {
    let google = FakeGoogle::start(None, &[]);
    let store = FakeStore::default();
    let (dir, client_file) = client_file("reminders");
    let mut work = entry("work", "Work", "#9fe1e7", "owner");
    work["defaultReminders"] = json!([
        {"method": "popup", "minutes": 10},
        {"method": "email", "minutes": 30},
    ]);
    google.add_calendar(ACCOUNT_ID, work);
    for (id, reminders) in [
        (
            "review",
            json!({"useDefault": false, "overrides": [
                {"method": "popup", "minutes": 10},
                {"method": "email", "minutes": 1440},
            ]}),
        ),
        // Google ignores the overrides of an event that uses the defaults.
        (
            "standup",
            json!({"useDefault": true, "overrides": [{"method": "popup", "minutes": 5}]}),
        ),
        ("silent", json!({"useDefault": false})),
    ] {
        let mut event = timed(
            id,
            id,
            "2026-11-12T09:00:00+01:00",
            "2026-11-12T10:00:00+01:00",
        );
        event["reminders"] = reminders;
        google.add_event(ACCOUNT_ID, "work", event);
    }
    google.add_event(
        ACCOUNT_ID,
        "work",
        timed(
            "plain",
            "Plain",
            "2026-11-12T11:00:00+01:00",
            "2026-11-12T12:00:00+01:00",
        ),
    );
    signed_in(&google, &store, ACCOUNT_ID);
    let mut cache = Cache::in_memory();

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let reminder = |method, minutes| Reminder { method, minutes };
    assert_eq!(
        cache.calendar_list().unwrap()[0].default_reminders,
        [
            reminder(Method::Notification, 10),
            reminder(Method::Email, 30)
        ]
    );
    let reminders = |id| cached(&cache, "work", id).unwrap().about.reminders;
    assert_eq!(
        reminders("review"),
        Reminders::Own(vec![
            reminder(Method::Notification, 10),
            reminder(Method::Email, 1440)
        ])
    );
    assert_eq!(reminders("standup"), Reminders::Default);
    assert_eq!(reminders("silent"), Reminders::Own(Vec::new()));
    // Without a word from Google, as for an event it made, the defaults hold.
    assert_eq!(reminders("plain"), Reminders::Default);
}

#[test]
fn a_guests_copy_the_organizer_does_not_let_guests_change_is_for_the_organizer_only() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("organizer-only");
    let copy = |id: &str, title: &str, guests_can_modify: Option<bool>| {
        let mut event = timed(
            id,
            title,
            "2026-11-12T12:00:00+01:00",
            "2026-11-12T13:00:00+01:00",
        );
        event["organizer"] = json!({"email": "boss@example.com"});
        if let Some(can) = guests_can_modify {
            event["guestsCanModify"] = json!(can);
        }
        event
    };
    google.add_event(ACCOUNT_ID, "work", copy("theirs", "Their meeting", None));
    google.add_event(
        ACCOUNT_ID,
        "work",
        copy("shared", "Open meeting", Some(true)),
    );
    let mut own = timed(
        "own",
        "Own meeting",
        "2026-11-12T14:00:00+01:00",
        "2026-11-12T15:00:00+01:00",
    );
    own["organizer"] = json!({"email": "user@example.com", "self": true});
    google.add_event(ACCOUNT_ID, "work", own);

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let theirs = cached(&cache, "work", "theirs").unwrap();
    let patch = Patch {
        title: Some("Mine now".into()),
        ..Patch::default()
    };
    let refused = session.update(&work, "theirs", &theirs.etag, &patch, false);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let organizer_only = |id| cached(&cache, "work", id).unwrap().organizer_only;
    assert!(organizer_only("theirs"));
    assert!(!organizer_only("shared"));
    assert!(!organizer_only("own"));
    // Without an organizer, as in the events Google keeps for the calendar itself.
    assert!(!organizer_only("standup"));
    // Google refuses the organizer's fields to a guest, and says so.
    let error = refused.unwrap_err();
    assert!(error.0.starts_with("Google refused"), "{}", error.0);
    assert!(error.0.contains("organizer"), "{}", error.0);
    let kept = google.event(ACCOUNT_ID, "work", "theirs").unwrap();
    assert_eq!(kept["summary"], "Their meeting");
}

#[test]
fn a_new_all_day_event_is_sent_with_every_field_set_in_the_editor() {
    let (google, store, dir, client_file, cache) = synchronized_once("create-all-day");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let trip = NewEvent {
        title: "Trip".into(),
        timing: Timing::AllDay {
            start: d(2026, 11, 12),
            end_exclusive: d(2026, 11, 15),
        },
        about: About {
            description: "Flights".into(),
            location: "Lisbon".into(),
            show: Show::Free,
            visibility: Visibility::Public,
            kind: Kind::Ordinary,
            color: Some(EventColor::Tomato),
            attachments: Vec::new(),
            guests: Vec::new(),
            meet: None,
            reminders: Reminders::Default,
        },
        recurrence: Vec::new(),
        meet: false,
    };

    let id = session.create(&work, &trip).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    let sent = google.event(ACCOUNT_ID, "work", &id).unwrap();
    assert_eq!(sent["summary"], "Trip");
    assert_eq!(sent["start"], json!({"date": "2026-11-12"}));
    assert_eq!(sent["end"], json!({"date": "2026-11-15"}));
    assert_eq!(sent["description"], "Flights");
    assert_eq!(sent["location"], "Lisbon");
    assert_eq!(sent["transparency"], "transparent");
    assert_eq!(sent["visibility"], "public");
    assert_eq!(sent["colorId"], "11");
}

#[test]
fn events_come_with_their_own_google_colour() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("colours");
    for id in 1..=12 {
        let mut event = timed(
            &format!("c{id}"),
            "Coloured",
            "2026-11-12T09:00:00+01:00",
            "2026-11-12T10:00:00+01:00",
        );
        event["colorId"] = json!(id.to_string());
        google.add_event(ACCOUNT_ID, "work", event);
    }

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let color = |id: &str| cached(&cache, "work", id).unwrap().about.color;
    let colors: Vec<Option<EventColor>> = (1..=11).map(|id| color(&format!("c{id}"))).collect();
    assert_eq!(
        colors,
        [
            EventColor::Lavender,
            EventColor::Sage,
            EventColor::Grape,
            EventColor::Flamingo,
            EventColor::Banana,
            EventColor::Tangerine,
            EventColor::Peacock,
            EventColor::Graphite,
            EventColor::Blueberry,
            EventColor::Basil,
            EventColor::Tomato,
        ]
        .map(Some)
    );
    // An ID outside Google's event palette, and no colour at all: the calendar's.
    assert_eq!(color("c12"), None);
    assert_eq!(color("standup"), None);
}

#[test]
fn events_come_with_their_attachments_as_links() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("attachments");
    let mut review = timed(
        "review",
        "Review",
        "2026-11-12T09:00:00+01:00",
        "2026-11-12T10:00:00+01:00",
    );
    review["attachments"] = json!([
        {
            "fileUrl": "https://docs.google.com/document/d/1abc/edit",
            "title": "Q4 numbers",
            "mimeType": "application/vnd.google-apps.document",
            "fileId": "1abc"
        },
        {"fileUrl": "https://drive.google.com/file/d/2def/view", "title": ""},
        // Only web pages are opened: nothing else is handed to the desktop's URL handler.
        {"fileUrl": "file:///etc/passwd", "title": "Local file"},
        {"title": "No link"}
    ]);
    google.add_event(ACCOUNT_ID, "work", review);

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    assert_eq!(
        cached(&cache, "work", "review").unwrap().about.attachments,
        [
            Attachment {
                title: "Q4 numbers".into(),
                url: "https://docs.google.com/document/d/1abc/edit".into(),
            },
            // Without a name, the link itself.
            Attachment {
                title: "https://drive.google.com/file/d/2def/view".into(),
                url: "https://drive.google.com/file/d/2def/view".into(),
            },
        ]
    );
    assert_eq!(
        cached(&cache, "work", "standup").unwrap().about.attachments,
        []
    );
}

#[test]
fn an_events_colour_is_patched_and_taken_away() {
    let (google, store, dir, client_file, cache) = synchronized_once("patch-colour");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let etag = |google: &FakeGoogle| {
        let kept = google.event(ACCOUNT_ID, "work", "standup").unwrap();
        kept["etag"].as_str().unwrap().to_string()
    };
    let recolour = |color| Patch {
        color: Some(color),
        ..Patch::default()
    };

    let painted = session.update(
        &work,
        "standup",
        &etag(&google),
        &recolour(Some(EventColor::Peacock)),
        false,
    );
    let painted_to = google.event(ACCOUNT_ID, "work", "standup").unwrap()["colorId"].clone();
    let cleared = session.update(&work, "standup", &etag(&google), &recolour(None), false);
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(painted.is_ok() && cleared.is_ok());
    assert_eq!(painted_to, "7");
    let kept = google.event(ACCOUNT_ID, "work", "standup").unwrap();
    assert_eq!(kept.get("colorId"), None);
    assert_eq!(kept["summary"], "Standup");
}

#[test]
fn tc_05_07_a_new_series_is_sent_with_its_rule_in_its_own_time_zone() {
    let (google, store, dir, client_file, cache) = synchronized_once("create-series");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let training = NewEvent {
        recurrence: vec!["RRULE:FREQ=WEEKLY;BYDAY=MO,WE;COUNT=10".into()],
        ..planning()
    };

    let id = session.create(&work, &training).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    let sent = google.event(ACCOUNT_ID, "work", &id).unwrap();
    assert_eq!(
        sent["recurrence"],
        json!(["RRULE:FREQ=WEEKLY;BYDAY=MO,WE;COUNT=10"])
    );
    // Google expands a series in the zone of its start.
    assert_eq!(sent["start"]["timeZone"], "Europe/Warsaw");
    // An event that does not repeat is sent without a rule.
    let planning = session.create(&work, &planning()).unwrap();
    let sent = google.event(ACCOUNT_ID, "work", &planning).unwrap();
    assert_eq!(sent.get("recurrence"), None);
}

#[test]
fn a_change_is_sent_as_a_patch_of_what_changed_against_the_version_opened() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("update");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let opened = cached(&cache, "work", "review").unwrap();
    let patch = Patch {
        title: Some("Q4 review".into()),
        location: Some("Room 2".into()),
        show: Some(Show::Free),
        visibility: Some(Visibility::Private),
        // Made an all-day event: Google's times give way to dates.
        timing: Some(Timing::AllDay {
            start: d(2026, 11, 12),
            end_exclusive: d(2026, 11, 13),
        }),
        ..Patch::default()
    };

    let etag = session
        .update(&work, "review", &opened.etag, &patch, false)
        .unwrap();

    let changed = google.event(ACCOUNT_ID, "work", "review").unwrap();
    assert_eq!(changed["etag"], etag);
    assert_ne!(etag, opened.etag);
    assert_eq!(changed["summary"], "Q4 review");
    assert_eq!(changed["location"], "Room 2");
    assert_eq!(changed["transparency"], "transparent");
    assert_eq!(changed["visibility"], "private");
    assert_eq!(changed["start"], json!({"date": "2026-11-12"}));
    assert_eq!(changed["end"], json!({"date": "2026-11-13"}));
    // What was not changed is not sent.
    assert_eq!(
        google.patched().last().unwrap().1,
        json!({
            "summary": "Q4 review",
            "location": "Room 2",
            "transparency": "transparent",
            "visibility": "private",
            "start": {"date": "2026-11-12", "dateTime": null, "timeZone": null},
            "end": {"date": "2026-11-13", "dateTime": null, "timeZone": null},
        })
    );
    assert_eq!(google.patched().last().unwrap().0, opened.etag);
    assert!(
        google.requests().contains(
            &"PATCH /calendar/v3/calendars/work/events/review?sendUpdates=none".to_string()
        )
    );

    // And back to a timed event, in another zone.
    let patch = Patch {
        timing: Some(Timing::Timed {
            start: Utc.with_ymd_and_hms(2026, 11, 12, 14, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 11, 12, 15, 0, 0).unwrap(),
            zone: London,
        }),
        description: Some("Agenda".into()),
        ..Patch::default()
    };
    session
        .update(&work, "review", &etag, &patch, false)
        .unwrap();
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let changed = google.event(ACCOUNT_ID, "work", "review").unwrap();
    assert_eq!(
        changed["start"],
        json!({"dateTime": "2026-11-12T14:00:00Z", "timeZone": "Europe/London"})
    );
    let review = cached(&cache, "work", "review").unwrap();
    assert_eq!(review.title, "Q4 review");
    assert_eq!(review.about.description, "Agenda");
    assert_eq!(review.etag, changed["etag"]);
    assert_eq!(review.timing, patch.timing);
}

#[test]
fn a_change_to_an_event_changed_elsewhere_meanwhile_is_refused_as_such() {
    let (google, store, dir, client_file, cache) = synchronized_once("update-conflict");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let opened = cached(&cache, "work", "review").unwrap();
    google.add_event(
        ACCOUNT_ID,
        "work",
        timed(
            "review",
            "Review (moved)",
            "2026-11-11T12:00:00+01:00",
            "2026-11-11T13:00:00+01:00",
        ),
    );
    let patch = Patch {
        location: Some("Room 2".into()),
        ..Patch::default()
    };

    let refused = session.update(&work, "review", &opened.etag, &patch, false);
    // Nor can the account change a calendar it may only read.
    google.set_access_role("work", "reader");
    let current = google.event(ACCOUNT_ID, "work", "review").unwrap()["etag"]
        .as_str()
        .unwrap()
        .to_string();
    let read_only = session.update(&work, "review", &current, &patch, false);
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(refused.unwrap_err().is_changed_elsewhere());
    let error = read_only.unwrap_err();
    assert!(error.0.starts_with("Google refused"), "{}", error.0);
    let kept = google.event(ACCOUNT_ID, "work", "review").unwrap();
    assert_eq!(kept["summary"], "Review (moved)");
    assert!(kept.get("location").is_none());
}

#[test]
fn an_event_moved_to_another_calendar_of_the_account_leaves_the_first_for_the_other() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("move");
    google.add_calendar(ACCOUNT_ID, entry("team", "Team", "#7bd148", "writer"));
    assert_eq!(
        sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache),
        Ok(())
    );
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let calendars = cache.calendar_list().unwrap();
    let (work, team) = (&calendars[0], &calendars[1]);
    let opened = cached(&cache, "work", "review").unwrap();

    let etag = session.move_to(work, "review", team, false).unwrap();

    assert!(google.requests().contains(
        &"POST /calendar/v3/calendars/work/events/review/move?destination=team&sendUpdates=none".to_string()
    ));
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    assert!(cached(&cache, "work", "review").is_none());
    let moved = cached(&cache, "team", "review").unwrap();
    assert_eq!(moved.title, "Review");
    assert_eq!(moved.etag, etag);
    assert_ne!(etag, opened.etag);
}

#[test]
fn a_deleted_event_leaves_the_calendar_and_restoring_it_brings_it_back_whole() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("delete");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let opened = cached(&cache, "work", "review").unwrap();

    session
        .delete(&work, "review", &opened.etag, false)
        .unwrap();

    assert!(google.requests().contains(
        &"DELETE /calendar/v3/calendars/work/events/review?sendUpdates=none".to_string()
    ));
    assert_eq!(
        google.event(ACCOUNT_ID, "work", "review").unwrap()["status"],
        "cancelled"
    );
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    assert_eq!(synced, Ok(()));
    assert!(cached(&cache, "work", "review").is_none());

    let etag = session.restore(&work, "review", false).unwrap();

    assert_eq!(
        google.patched().last().unwrap().1,
        json!({"status": "confirmed"})
    );
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let restored = cached(&cache, "work", "review").unwrap();
    assert_eq!(restored.title, opened.title);
    assert_eq!(restored.timing, opened.timing);
    assert_eq!(restored.etag, etag);
}

#[test]
fn a_deletion_of_an_event_changed_elsewhere_meanwhile_is_refused_as_such() {
    let (google, store, dir, client_file, cache) = synchronized_once("delete-conflict");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let opened = cached(&cache, "work", "review").unwrap();
    google.add_event(
        ACCOUNT_ID,
        "work",
        timed(
            "review",
            "Review (moved)",
            "2026-11-11T12:00:00+01:00",
            "2026-11-11T13:00:00+01:00",
        ),
    );

    let refused = session.delete(&work, "review", &opened.etag, false);
    // Nor can the account delete from a calendar it may only read.
    google.set_access_role("work", "reader");
    let current = google.event(ACCOUNT_ID, "work", "review").unwrap()["etag"]
        .as_str()
        .unwrap()
        .to_string();
    let read_only = session.delete(&work, "review", &current, false);
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(refused.unwrap_err().is_changed_elsewhere());
    let error = read_only.unwrap_err();
    assert!(error.0.starts_with("Google refused"), "{}", error.0);
    assert_eq!(
        google.event(ACCOUNT_ID, "work", "review").unwrap()["status"],
        "confirmed"
    );
}

/// A weekly series on Monday 09:00 from 16 November, ten times, beside the calendar's events.
fn with_weekly_series(name: &str) -> (FakeGoogle, FakeStore, PathBuf, PathBuf, Cache) {
    let (google, store, dir, client_file, mut cache) = synchronized_once(name);
    let mut weekly = timed(
        "weekly",
        "Weekly",
        "2026-11-16T09:00:00+01:00",
        "2026-11-16T09:15:00+01:00",
    );
    weekly["recurrence"] = json!(["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=10"]);
    google.add_event(ACCOUNT_ID, "work", weekly);
    assert_eq!(
        sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache),
        Ok(())
    );
    (google, store, dir, client_file, cache)
}

#[test]
fn an_occurrence_is_changed_and_deleted_alone_by_its_id_without_a_version_of_its_own() {
    let (google, store, dir, client_file, mut cache) = with_weekly_series("occurrence");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let patch = Patch {
        title: Some("Weekly (late)".into()),
        ..Patch::default()
    };

    let etag = session
        .update(&work, "weekly_20261123T080000Z", "", &patch, false)
        .unwrap();
    session
        .delete(&work, "weekly_20261130T080000Z", "", false)
        .unwrap();
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    // Sent without `If-Match`: an occurrence Google has not been asked to change has no version.
    assert_eq!(google.patched().last().unwrap().0, "");
    let second = Start::At(Utc.with_ymd_and_hms(2026, 11, 23, 8, 0, 0).unwrap());
    let changed = cached(&cache, "work", "weekly_20261123T080000Z").unwrap();
    assert_eq!(changed.instance_of, Some(("weekly".into(), second)));
    assert_eq!(changed.title, "Weekly (late)");
    assert_eq!(changed.etag, etag);
    let deleted = cached(&cache, "work", "weekly_20261130T080000Z").unwrap();
    assert_eq!(deleted.timing, None);
    let shown = shown_events(&cache.calendars().unwrap(), window());
    assert_eq!(
        week_of(&shown, d(2026, 11, 23))[0],
        (
            "Weekly (late)".into(),
            "2026-11-23 09:00".into(),
            "2026-11-23 09:15".into()
        )
    );
    assert!(week_of(&shown, d(2026, 11, 30)).is_empty());
}

#[test]
fn a_series_rule_is_sent_whole_in_a_patch_against_its_version() {
    let (google, store, dir, client_file, mut cache) = with_weekly_series("rule");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let master = cached(&cache, "work", "weekly").unwrap();
    let ended = vec!["RRULE:FREQ=WEEKLY;BYDAY=MO;UNTIL=20261214T075959Z".to_string()];
    let patch = Patch {
        recurrence: Some(ended.clone()),
        ..Patch::default()
    };

    session
        .update(&work, "weekly", &master.etag, &patch, false)
        .unwrap();
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    assert_eq!(
        google.patched().last().unwrap(),
        &(master.etag, json!({"recurrence": ended}))
    );
    assert_eq!(cached(&cache, "work", "weekly").unwrap().recurrence, ended);
}

#[test]
fn the_series_opens_in_google_calendars_editor_as_the_account() {
    let page = edit_page("work@example.com", "weekly", "owner@example.com");

    let eid = URL_SAFE_NO_PAD.encode("weekly work@example.com");
    assert_eq!(
        page,
        format!(
            "https://calendar.google.com/calendar/r/eventedit/{eid}?authuser=owner%40example.com"
        )
    );
}

#[test]
fn a_webcal_link_opens_google_calendars_page_for_adding_a_calendar_from_its_address() {
    let page = subscribe_page("webcal://example.com/holidays.ics?lang=pl&x=1");

    assert_eq!(
        page,
        "https://calendar.google.com/calendar/r?cid=webcal%3A%2F%2Fexample.com%2Fholidays.ics%3Flang%3Dpl%26x%3D1"
    );
    let page = Url::parse(&page).unwrap();
    let cid: Vec<_> = page.query_pairs().collect();
    assert_eq!(cid.len(), 1);
    assert_eq!(cid[0].0, "cid");
    assert_eq!(cid[0].1, "webcal://example.com/holidays.ics?lang=pl&x=1");
}

#[test]
fn a_series_is_copied_whole_with_its_guests_reminders_and_colour_but_the_new_fields() {
    let (google, store, dir, client_file, cache) = synchronized_once("copy");
    let mut weekly = timed(
        "weekly",
        "Weekly",
        "2026-11-16T09:00:00+01:00",
        "2026-11-16T09:15:00+01:00",
    );
    weekly["recurrence"] = json!(["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=10"]);
    weekly["iCalUID"] = json!("weekly@google.com");
    weekly["attendees"] = json!([{"email": "guest@example.com", "responseStatus": "accepted"}]);
    weekly["reminders"] =
        json!({"useDefault": false, "overrides": [{"method": "popup", "minutes": 5}]});
    weekly["colorId"] = json!("5");
    google.add_event(ACCOUNT_ID, "work", weekly);
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let at = |h| Utc.with_ymd_and_hms(2026, 12, 14, h, 0, 0).unwrap();
    let following = NewEvent {
        title: "Weekly".into(),
        timing: Timing::Timed {
            start: at(10),
            end: at(10) + chrono::Duration::minutes(15),
            zone: Warsaw,
        },
        // The series' colour, guests and reminders, as synchronized.
        about: About {
            color: Some(EventColor::Banana),
            reminders: Reminders::Own(vec![Reminder {
                method: Method::Notification,
                minutes: 5,
            }]),
            // As known from the occurrence, unanswered; Google's own answer of the series stays.
            guests: vec![Guest::invited(&person("", "guest@example.com"))],
            ..About::default()
        },
        recurrence: vec!["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=6".into()],
        meet: false,
    };
    // Without the series' colour, guests or reminders.
    let uncoloured = NewEvent {
        about: About::default(),
        ..following.clone()
    };

    let id = session.copy(&work, "weekly", &following, false).unwrap();
    let without = session.copy(&work, "weekly", &uncoloured, false).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    let copy = google.event(ACCOUNT_ID, "work", &id).unwrap();
    assert_ne!(id, "weekly");
    assert_eq!(copy["attendees"][0]["email"], "guest@example.com");
    assert_eq!(copy["attendees"][0]["responseStatus"], "accepted");
    assert_eq!(copy["reminders"]["overrides"][0]["minutes"], 5);
    assert_eq!(copy["colorId"], "5");
    // A colour taken away for the new series is not copied from the old.
    let without = google.event(ACCOUNT_ID, "work", &without).unwrap();
    assert_eq!(without.get("colorId"), None);
    assert_eq!(without.get("attendees"), None);
    // Nor reminders of its own: it takes its calendar's defaults (`MOD-07-F05`).
    assert_eq!(without["reminders"], json!({"useDefault": true}));
    assert_eq!(copy["start"]["dateTime"], "2026-12-14T11:00:00+01:00");
    assert_eq!(
        copy["recurrence"],
        json!(["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=6"])
    );
    assert!(copy.get("iCalUID").is_none());
    assert!(
        google
            .requests()
            .iter()
            .any(|r| r.starts_with("POST ") && r.contains("conferenceDataVersion=1"))
    );
}

fn person(name: &str, email: &str) -> Person {
    Person {
        name: name.to_string(),
        email: email.to_string(),
    }
}

#[test]
fn events_come_with_their_guests_their_answers_and_the_accounts_own_entry() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("guests");
    let mut kickoff = timed(
        "kickoff",
        "Kickoff",
        "2026-11-13T10:00:00+01:00",
        "2026-11-13T11:00:00+01:00",
    );
    kickoff["attendees"] = json!([
        {"email": "owner@example.com", "self": true, "organizer": true, "responseStatus": "accepted"},
        {"email": "adam@example.com", "displayName": "Adam Kowalski", "responseStatus": "tentative"},
        {"email": "ewa@example.com", "optional": true, "responseStatus": "declined"},
        {"email": "jan@example.com", "responseStatus": "needsAction"},
        // Without an address, as Google may send one: it cannot be invited again.
        {"displayName": "Someone", "responseStatus": "accepted"}
    ]);
    google.add_event(ACCOUNT_ID, "work", kickoff);

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    assert_eq!(
        cached(&cache, "work", "kickoff").unwrap().about.guests,
        [
            Guest {
                response: Response::Accepted,
                own: true,
                ..Guest::invited(&person("", "owner@example.com"))
            },
            Guest {
                response: Response::Maybe,
                ..Guest::invited(&person("Adam Kowalski", "adam@example.com"))
            },
            Guest {
                optional: true,
                response: Response::Declined,
                ..Guest::invited(&person("", "ewa@example.com"))
            },
            Guest::invited(&person("", "jan@example.com")),
        ]
    );
    assert_eq!(cached(&cache, "work", "standup").unwrap().about.guests, []);
}

#[test]
fn an_invitation_the_account_has_not_answered_awaits_its_answer() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("awaiting");
    let invitation = |id: &str, answer: &str| {
        let mut event = timed(
            id,
            id,
            "2026-11-13T10:00:00+01:00",
            "2026-11-13T11:00:00+01:00",
        );
        event["organizer"] = json!({"email": "boss@example.com"});
        event["attendees"] = json!([
            {"email": "boss@example.com", "organizer": true, "responseStatus": "accepted"},
            {"email": "owner@example.com", "self": true, "responseStatus": answer}
        ]);
        event
    };
    google.add_event(ACCOUNT_ID, "work", invitation("board", "needsAction"));
    google.add_event(ACCOUNT_ID, "work", invitation("lunch", "accepted"));

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    assert!(cached(&cache, "work", "board").unwrap().about.awaiting());
    assert!(!cached(&cache, "work", "lunch").unwrap().about.awaiting());
    assert!(!cached(&cache, "work", "standup").unwrap().about.awaiting());
}

#[test]
fn a_new_event_with_guests_is_created_with_them_and_google_sends_the_invitations() {
    let (google, store, dir, client_file, cache) = synchronized_once("invite");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let mut kickoff = planning();
    kickoff.about.guests = vec![
        Guest::invited(&person("Adam Kowalski", "adam@example.com")),
        Guest {
            optional: true,
            ..Guest::invited(&person("", "ewa@example.com"))
        },
    ];

    let invited = session.create(&work, &kickoff).unwrap();
    let alone = session.create(&work, &planning()).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    let sent = google.event(ACCOUNT_ID, "work", &invited).unwrap();
    assert_eq!(
        sent["attendees"],
        json!([
            {"email": "adam@example.com", "displayName": "Adam Kowalski", "responseStatus": "needsAction"},
            {"email": "ewa@example.com", "optional": true, "responseStatus": "needsAction"}
        ])
    );
    let creations: Vec<String> = (google.requests().into_iter())
        .filter(|r| r.starts_with("POST /calendar/v3/calendars/work/events"))
        .collect();
    assert_eq!(
        creations,
        [
            "POST /calendar/v3/calendars/work/events?sendUpdates=all",
            "POST /calendar/v3/calendars/work/events",
        ]
    );
    let alone = google.event(ACCOUNT_ID, "work", &alone).unwrap();
    assert_eq!(alone.get("attendees"), None);
}

#[test]
fn an_events_guests_are_patched_whole_and_those_google_has_no_entry_for_go_unanswered() {
    let (google, store, dir, client_file, cache) = synchronized_once("patch-guests");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let etag = google.event(ACCOUNT_ID, "work", "standup").unwrap()["etag"]
        .as_str()
        .unwrap()
        .to_string();
    let patch = Patch {
        guests: Some(vec![
            Guest {
                response: Response::Accepted,
                own: true,
                ..Guest::invited(&person("", "owner@example.com"))
            },
            Guest {
                response: Response::Maybe,
                ..Guest::invited(&person("Adam", "adam@example.com"))
            },
        ]),
        ..Patch::default()
    };

    let patched = session.update(&work, "standup", &etag, &patch, false);
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(patched.is_ok());
    let [(_, sent)] = google.patched().try_into().unwrap();
    assert_eq!(
        sent,
        json!({"attendees": [
            {"email": "owner@example.com", "responseStatus": "needsAction"},
            {"email": "adam@example.com", "displayName": "Adam", "responseStatus": "needsAction"}
        ]})
    );
    // The user said not to e-mail the guests (`MOD-06-F12`).
    assert!(
        (google.requests().iter())
            .filter(|r| r.starts_with("PATCH "))
            .all(|r| r.ends_with("?sendUpdates=none"))
    );
}

#[test]
fn the_users_answer_patches_their_own_entry_alone_also_in_a_copy_only_its_organizer_may_change() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("answer");
    let mut board = timed(
        "board",
        "Board",
        "2026-11-12T14:00:00+01:00",
        "2026-11-12T15:00:00+01:00",
    );
    board["organizer"] = json!({"email": "boss@example.com"});
    board["attendees"] = json!([
        {"email": "boss@example.com", "organizer": true, "responseStatus": "accepted"},
        {"email": "owner@example.com", "self": true, "responseStatus": "needsAction"},
        {"email": "adam@example.com", "displayName": "Adam", "responseStatus": "tentative",
         "comment": "Late"}
    ]);
    google.add_event(ACCOUNT_ID, "work", board);
    assert_eq!(
        sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache),
        Ok(())
    );
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let etag = cached(&cache, "work", "board").unwrap().etag;
    let answer = |response| Patch {
        response: Some(response),
        ..Patch::default()
    };

    let accepted = session.update(&work, "board", &etag, &answer(Response::Accepted), true);
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    let maybe = cached(&cache, "work", "board").unwrap().etag;
    let tentative = session.update(&work, "board", &maybe, &answer(Response::Maybe), true);
    let declined = session.update(&work, "board", "", &answer(Response::Declined), true);
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(accepted.is_ok() && tentative.is_ok() && declined.is_ok());
    assert_eq!(synced, Ok(()));
    let sent: Vec<Value> = google.patched().into_iter().map(|(_, sent)| sent).collect();
    // Everyone else's entry goes back as Google keeps it.
    assert_eq!(
        sent[0],
        json!({"attendees": [
            {"email": "boss@example.com", "organizer": true, "responseStatus": "accepted"},
            {"email": "owner@example.com", "self": true, "responseStatus": "accepted"},
            {"email": "adam@example.com", "displayName": "Adam", "responseStatus": "tentative",
             "comment": "Late"}
        ]})
    );
    let own = |sent: &Value| sent["attendees"][1]["responseStatus"].clone();
    assert_eq!(own(&sent[1]), "tentative");
    assert_eq!(own(&sent[2]), "declined");
    // Back through synchronization, as the user's own answer.
    let guests = cached(&cache, "work", "board").unwrap().about.guests;
    let answers: Vec<(bool, Response)> = guests.iter().map(|g| (g.own, g.response)).collect();
    assert_eq!(
        answers,
        [
            (false, Response::Accepted),
            (true, Response::Accepted),
            (false, Response::Maybe)
        ]
    );
    assert!(
        (google.requests().iter())
            .filter(|r| r.starts_with("PATCH "))
            .all(|r| r.ends_with("?sendUpdates=all"))
    );
}

#[test]
fn the_organizers_own_answer_emails_nobody_and_a_list_google_left_guests_out_of_stays_so() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("answer-own");
    let mut own = timed(
        "own",
        "Own",
        "2026-11-12T14:00:00+01:00",
        "2026-11-12T15:00:00+01:00",
    );
    own["organizer"] = json!({"email": "owner@example.com", "self": true});
    own["attendees"] = json!([
        {"email": "owner@example.com", "self": true, "organizer": true, "responseStatus": "accepted"},
        {"email": "adam@example.com", "responseStatus": "needsAction"}
    ]);
    google.add_event(ACCOUNT_ID, "work", own);
    let mut few = timed(
        "few",
        "Few",
        "2026-11-13T14:00:00+01:00",
        "2026-11-13T15:00:00+01:00",
    );
    few["organizer"] = json!({"email": "boss@example.com"});
    few["guestsCanModify"] = json!(true);
    few["attendeesOmitted"] = json!(true);
    few["attendees"] = json!([
        {"email": "owner@example.com", "self": true, "responseStatus": "needsAction"}
    ]);
    google.add_event(ACCOUNT_ID, "work", few);
    assert_eq!(
        sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache),
        Ok(())
    );
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let answer = Patch {
        response: Some(Response::Declined),
        ..Patch::default()
    };

    let declined = session.update(&work, "own", "", &answer, true);
    let accepted = session.update(
        &work,
        "few",
        "",
        &Patch {
            response: Some(Response::Accepted),
            ..Patch::default()
        },
        true,
    );
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(declined.is_ok() && accepted.is_ok());
    let patches: Vec<String> = (google.requests().into_iter())
        .filter(|r| r.starts_with("PATCH "))
        .collect();
    assert!(
        patches[0].ends_with("/events/own?sendUpdates=none"),
        "{patches:?}"
    );
    assert!(
        patches[1].ends_with("/events/few?sendUpdates=all"),
        "{patches:?}"
    );
    let sent: Vec<Value> = google.patched().into_iter().map(|(_, sent)| sent).collect();
    assert_eq!(sent[0]["attendees"][0]["responseStatus"], "declined");
    assert_eq!(sent[0].get("attendeesOmitted"), None);
    assert_eq!(
        sent[1],
        json!({"attendeesOmitted": true, "attendees": [
            {"email": "owner@example.com", "self": true, "responseStatus": "accepted"}
        ]})
    );
}

#[test]
fn whether_google_emails_the_guests_is_sent_with_every_change_and_deletion() {
    let (google, store, dir, client_file, cache) = synchronized_once("send-updates");
    google.add_calendar(ACCOUNT_ID, entry("team", "Team", "#7bd148", "writer"));
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let team = Calendar {
        id: "team".into(),
        ..work.clone()
    };
    let retitled = Patch {
        title: Some("Daily".into()),
        ..Patch::default()
    };
    let sent = |google: &FakeGoogle| google.requests().pop().unwrap();

    session
        .update(&work, "standup", "", &retitled, true)
        .unwrap();
    let all = sent(&google);
    session
        .update(&work, "standup", "", &retitled, false)
        .unwrap();
    let none = sent(&google);
    assert_eq!(
        [all, none],
        [
            "PATCH /calendar/v3/calendars/work/events/standup?sendUpdates=all",
            "PATCH /calendar/v3/calendars/work/events/standup?sendUpdates=none",
        ]
    );

    session.delete(&work, "review", "", true).unwrap();
    let deleted = sent(&google);
    session.restore(&work, "review", false).unwrap();
    let restored = sent(&google);
    session.delete(&work, "review", "", false).unwrap();
    assert_eq!(
        [deleted, restored, sent(&google)],
        [
            "DELETE /calendar/v3/calendars/work/events/review?sendUpdates=all",
            "PATCH /calendar/v3/calendars/work/events/review?sendUpdates=none",
            "DELETE /calendar/v3/calendars/work/events/review?sendUpdates=none",
        ]
    );

    session.move_to(&work, "standup", &team, true).unwrap();
    assert_eq!(
        sent(&google),
        "POST /calendar/v3/calendars/work/events/standup/move?destination=team&sendUpdates=all"
    );

    session.copy(&team, "standup", &planning(), false).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(sent(&google).ends_with("&sendUpdates=none"));
}

#[test]
fn people_are_searched_in_contacts_and_other_contacts_after_a_warm_up_and_in_the_directory_if_reached()
 {
    let (google, store, dir, client_file, _cache) = synchronized_once("people");
    google.add_contact(ACCOUNT_ID, "Adam Kowalski", "adam@example.com");
    google.add_contact(ACCOUNT_ID, "Ewa Nowak", "ewa@example.com");
    google.add_other_contact(ACCOUNT_ID, "", "adam.k@mail.example.com");
    google.add_colleague(ACCOUNT_ID, "Adam Nowak", "adam.nowak@example.com");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let contacts = Reach {
        contacts: true,
        directory: false,
    };
    let everywhere = Reach {
        contacts: true,
        directory: true,
    };

    let personal = session.people("ada", contacts);
    let workspace = session.people("ADA", everywhere);
    let nowhere = session.people("ada", Reach::default());
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(
        personal,
        [
            person("Adam Kowalski", "adam@example.com"),
            person("", "adam.k@mail.example.com"),
        ]
    );
    assert_eq!(
        workspace,
        [
            person("Adam Kowalski", "adam@example.com"),
            person("", "adam.k@mail.example.com"),
            person("Adam Nowak", "adam.nowak@example.com"),
        ]
    );
    assert_eq!(nowhere, []);
    let searches: Vec<String> = (google.requests().into_iter())
        .filter(|r| r.starts_with("GET /people/v1/"))
        .map(|r| {
            let url = Url::parse(&format!("http://127.0.0.1{}", &r["GET ".len()..])).unwrap();
            format!("{} {}", url.path(), param(&url, "query").unwrap())
        })
        .collect();
    // One warm-up of each contact search, the first time only; the directory needs none.
    assert_eq!(
        searches,
        [
            "/people/v1/people:searchContacts ",
            "/people/v1/otherContacts:search ",
            "/people/v1/people:searchContacts ada",
            "/people/v1/otherContacts:search ada",
            "/people/v1/people:searchContacts ADA",
            "/people/v1/otherContacts:search ADA",
            "/people/v1/people:searchDirectoryPeople ADA",
        ]
    );
}

#[test]
fn a_search_google_does_not_answer_suggests_nobody_from_there() {
    let (google, store, dir, client_file, _cache) = synchronized_once("people-offline");
    google.add_contact(ACCOUNT_ID, "Adam Kowalski", "adam@example.com");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    google.disconnect();

    let found = session.people(
        "ada",
        Reach {
            contacts: true,
            directory: true,
        },
    );
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(found, []);
}

#[test]
fn a_guest_change_keeps_what_google_keeps_of_each_guest_but_optional_and_their_name() {
    let (google, store, dir, client_file, cache) = synchronized_once("merge-guests");
    let mut kept = google.event(ACCOUNT_ID, "work", "standup").unwrap();
    kept["attendees"] = json!([
        {"email": "owner@example.com", "self": true, "organizer": true, "responseStatus": "accepted"},
        {"email": "Ewa@example.com", "responseStatus": "declined", "comment": "On leave",
            "additionalGuests": 1, "optional": true},
        {"email": "room@resource.calendar.google.com", "resource": true, "responseStatus": "accepted"}
    ]);
    google.add_event(ACCOUNT_ID, "work", kept);
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let etag = google.event(ACCOUNT_ID, "work", "standup").unwrap()["etag"]
        .as_str()
        .unwrap()
        .to_string();
    // As an occurrence knows them: Ewa's answer there differs from the series'.
    let patch = Patch {
        guests: Some(vec![
            Guest {
                own: true,
                ..Guest::invited(&person("", "owner@example.com"))
            },
            Guest {
                response: Response::Accepted,
                ..Guest::invited(&person("Ewa", "ewa@example.com"))
            },
            Guest::invited(&person("", "room@resource.calendar.google.com")),
            Guest::invited(&person("", "adam@example.com")),
        ]),
        ..Patch::default()
    };

    let patched = session.update(&work, "standup", &etag, &patch, false);
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(patched.is_ok());
    let [(_, sent)] = google.patched().try_into().unwrap();
    assert_eq!(
        sent["attendees"],
        json!([
            {"email": "owner@example.com", "self": true, "organizer": true, "responseStatus": "accepted"},
            {"email": "Ewa@example.com", "displayName": "Ewa", "responseStatus": "declined",
                "comment": "On leave", "additionalGuests": 1},
            {"email": "room@resource.calendar.google.com", "resource": true, "responseStatus": "accepted"},
            {"email": "adam@example.com", "responseStatus": "needsAction"}
        ])
    );
}

#[test]
fn a_contact_warm_up_that_did_not_get_through_is_tried_again_at_the_next_search() {
    let (google, store, dir, client_file, _cache) = synchronized_once("warm-up-again");
    google.add_contact(ACCOUNT_ID, "Adam Kowalski", "adam@example.com");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let contacts = Reach {
        contacts: true,
        directory: false,
    };
    google.disconnect();
    assert_eq!(session.people("ada", contacts), []);
    google.recover();

    let found = session.people("ada", contacts);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(found, [person("Adam Kowalski", "adam@example.com")]);
}

/// A Google Meet call of an event, as Google sends it: its link, a phone number and a PIN.
fn meet_call(code: &str) -> Value {
    json!({
        "createRequest": {
            "requestId": "request-1",
            "conferenceSolutionKey": {"type": "hangoutsMeet"},
            "status": {"statusCode": "success"}
        },
        "entryPoints": [
            {"entryPointType": "video", "uri": format!("https://meet.google.com/{code}"),
             "label": format!("meet.google.com/{code}")},
            {"entryPointType": "phone", "uri": "tel:+48-22-000-0000", "pin": "123456789"}
        ],
        "conferenceSolution": {"key": {"type": "hangoutsMeet"}, "name": "Google Meet"},
        "conferenceId": code
    })
}

#[test]
fn events_come_with_the_link_of_their_google_meet_call() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("meet-link");
    let mut review = timed(
        "review",
        "Review",
        "2026-11-12T09:00:00+01:00",
        "2026-11-12T10:00:00+01:00",
    );
    review["conferenceData"] = meet_call("abc-defg-hij");
    google.add_event(ACCOUNT_ID, "work", review);
    // Another conference than Google Meet, e.g. from an add-on, has no Join.
    let mut other = timed(
        "other",
        "Other call",
        "2026-11-12T11:00:00+01:00",
        "2026-11-12T12:00:00+01:00",
    );
    other["conferenceData"] = json!({
        "entryPoints": [{"entryPointType": "video", "uri": "https://example.com/call/1"}],
        "conferenceSolution": {"key": {"type": "addOn"}, "name": "Other"}
    });
    google.add_event(ACCOUNT_ID, "work", other);
    // A call Google could not make has no link.
    let mut failed = timed(
        "failed",
        "Failed call",
        "2026-11-12T13:00:00+01:00",
        "2026-11-12T14:00:00+01:00",
    );
    failed["conferenceData"] = json!({
        "createRequest": {
            "requestId": "request-2",
            "conferenceSolutionKey": {"type": "hangoutsMeet"},
            "status": {"statusCode": "failure"}
        }
    });
    google.add_event(ACCOUNT_ID, "work", failed);

    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    let meet = |id| cached(&cache, "work", id).unwrap().about.meet;
    assert_eq!(
        meet("review").as_deref(),
        Some("https://meet.google.com/abc-defg-hij")
    );
    assert_eq!(meet("other"), None);
    assert_eq!(meet("failed"), None);
    assert_eq!(meet("standup"), None);
}

#[test]
fn a_new_event_with_google_meet_asks_google_to_make_its_call() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("create-meet");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let with_meet = NewEvent {
        meet: true,
        ..planning()
    };

    let id = session.create(&work, &with_meet).unwrap();
    let without = session.create(&work, &planning()).unwrap();
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    let sent = google.event(ACCOUNT_ID, "work", &id).unwrap();
    let asked = &sent["conferenceData"]["createRequest"];
    assert_eq!(asked["conferenceSolutionKey"]["type"], "hangoutsMeet");
    assert!(!asked["requestId"].as_str().unwrap().is_empty());
    let created: Vec<String> = (google.requests().into_iter())
        .filter(|r| r.starts_with("POST /calendar/v3/"))
        .collect();
    assert_eq!(
        created,
        [
            "POST /calendar/v3/calendars/work/events?conferenceDataVersion=1",
            "POST /calendar/v3/calendars/work/events"
        ]
    );
    assert_eq!(synced, Ok(()));
    let link = cached(&cache, "work", &id).unwrap().about.meet.unwrap();
    assert!(link.starts_with("https://meet.google.com/"), "{link}");
    assert_eq!(cached(&cache, "work", &without).unwrap().about.meet, None);
}

#[test]
fn an_event_google_cannot_make_a_meet_call_for_is_saved_without_one() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("create-no-meet");
    let mut work_entry = entry("work", "Work", "#9fe1e7", "owner");
    work_entry["conferenceProperties"] =
        json!({"allowedConferenceSolutionTypes": ["eventHangout"]});
    google.remove_calendar(ACCOUNT_ID, "work");
    google.add_calendar(ACCOUNT_ID, work_entry);
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);

    let created = session.create(
        &work,
        &NewEvent {
            meet: true,
            ..planning()
        },
    );
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    std::fs::remove_dir_all(&dir).unwrap();

    let id = created.unwrap();
    assert_eq!(synced, Ok(()));
    let saved = cached(&cache, "work", &id).unwrap();
    assert_eq!(saved.title, "Planning");
    assert_eq!(saved.about.meet, None);
}

#[test]
fn google_meet_is_added_to_an_existing_event_and_taken_away() {
    let (google, store, dir, client_file, cache) = synchronized_once("patch-meet");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let etag = |google: &FakeGoogle| {
        let kept = google.event(ACCOUNT_ID, "work", "standup").unwrap();
        kept["etag"].as_str().unwrap().to_string()
    };
    let meet = |on| Patch {
        meet: Some(on),
        ..Patch::default()
    };

    let added = session.update(&work, "standup", &etag(&google), &meet(true), false);
    let call = google.event(ACCOUNT_ID, "work", "standup").unwrap()["conferenceData"].clone();
    let removed = session.update(&work, "standup", &etag(&google), &meet(false), false);
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(added.is_ok() && removed.is_ok());
    assert_eq!(call["conferenceSolution"]["key"]["type"], "hangoutsMeet");
    let kept = google.event(ACCOUNT_ID, "work", "standup").unwrap();
    assert_eq!(kept.get("conferenceData"), None);
    assert_eq!(kept["summary"], "Standup");
    let patches: Vec<String> = (google.requests().into_iter())
        .filter(|r| r.starts_with("PATCH "))
        .collect();
    assert!(
        patches
            .iter()
            .all(|r| r.contains("conferenceDataVersion=1")),
        "{patches:?}"
    );
}

#[test]
fn a_series_copy_keeps_its_meet_call_unless_switched_off_and_gets_one_when_switched_on() {
    let (google, store, dir, client_file, cache) = synchronized_once("copy-meet");
    let mut weekly = timed(
        "weekly",
        "Weekly",
        "2026-11-16T09:00:00+01:00",
        "2026-11-16T09:15:00+01:00",
    );
    weekly["recurrence"] = json!(["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=10"]);
    weekly["conferenceData"] = meet_call("abc-defg-hij");
    google.add_event(ACCOUNT_ID, "work", weekly);
    let mut plain = timed(
        "plain",
        "Plain",
        "2026-11-17T09:00:00+01:00",
        "2026-11-17T09:15:00+01:00",
    );
    plain["recurrence"] = json!(["RRULE:FREQ=WEEKLY;BYDAY=TU;COUNT=10"]);
    google.add_event(ACCOUNT_ID, "work", plain);
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let new = |meet| NewEvent { meet, ..planning() };

    let kept = session.copy(&work, "weekly", &new(true), false).unwrap();
    let off = session.copy(&work, "weekly", &new(false), false).unwrap();
    let on = session.copy(&work, "plain", &new(true), false).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    let call = |id: &str| google.event(ACCOUNT_ID, "work", id).unwrap()["conferenceData"].clone();
    assert_eq!(call(&kept)["conferenceId"], "abc-defg-hij");
    assert_eq!(call(&off), Value::Null);
    assert_eq!(
        call(&on)["conferenceSolution"]["key"]["type"],
        "hangoutsMeet"
    );
    assert_ne!(call(&on)["conferenceId"], "abc-defg-hij");
}

#[test]
fn a_new_event_with_google_meet_google_fails_to_save_is_not_sent_again_without_its_call() {
    let (google, store, dir, client_file, cache) = synchronized_once("create-meet-failing");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    google.fail_calendar("work");

    let created = session.create(
        &work,
        &NewEvent {
            meet: true,
            ..planning()
        },
    );
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(created.unwrap_err().0.starts_with("Google refused"));
    let posted = (google.requests().into_iter())
        .filter(|r| r.starts_with("POST /calendar/v3/"))
        .count();
    assert_eq!(posted, 1);
}

#[test]
fn guests_busy_times_are_asked_with_free_busy_for_the_day_and_one_not_shared_has_none() {
    let (google, store, dir, client_file, _cache) = synchronized_once("free-busy");
    google.share_busy(
        "anna@example.com",
        &[("2026-11-13T14:00:00+01:00", "2026-11-13T15:00:00+01:00")],
    );
    google.share_busy("adam@example.com", &[]);
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let day = calendar_core::week::day_span(d(2026, 11, 13), Warsaw);
    let emails = [
        "anna@example.com",
        "adam@example.com",
        "private@example.com",
    ]
    .map(String::from);

    let busy = session.busy(&emails, day);
    std::fs::remove_dir_all(&dir).unwrap();

    let at = |hour| Utc.with_ymd_and_hms(2026, 11, 13, hour, 0, 0).unwrap();
    assert_eq!(
        busy,
        Ok(vec![
            Availability {
                email: "anna@example.com".into(),
                busy: Some(vec![(at(13), at(14))]),
            },
            Availability {
                email: "adam@example.com".into(),
                busy: Some(vec![]),
            },
            Availability {
                email: "private@example.com".into(),
                busy: None,
            },
        ])
    );
    assert_eq!(
        google.busy_queries(),
        [json!({
            "timeMin": "2026-11-12T23:00:00Z",
            "timeMax": "2026-11-13T23:00:00Z",
            "items": [
                {"id": "anna@example.com"},
                {"id": "adam@example.com"},
                {"id": "private@example.com"}
            ]
        })]
    );
}

#[test]
fn busy_times_google_does_not_answer_are_an_error() {
    let (google, store, dir, client_file, _cache) = synchronized_once("free-busy-offline");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    google.disconnect();

    let busy = session.busy(
        &["anna@example.com".to_string()],
        calendar_core::week::day_span(d(2026, 11, 13), Warsaw),
    );
    std::fs::remove_dir_all(&dir).unwrap();

    assert!(busy.is_err());
}

#[test]
fn a_new_event_is_sent_with_its_own_reminders_or_left_to_its_calendars_defaults() {
    let (google, store, dir, client_file, cache) = synchronized_once("create-reminders");
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let at = |h| Utc.with_ymd_and_hms(2026, 11, 12, h, 0, 0).unwrap();
    let plain = NewEvent {
        title: "Plan".into(),
        timing: Timing::Timed {
            start: at(9),
            end: at(10),
            zone: Warsaw,
        },
        about: About::default(),
        recurrence: Vec::new(),
        meet: false,
    };
    let reminded = NewEvent {
        about: About {
            reminders: Reminders::Own(vec![
                Reminder {
                    method: Method::Notification,
                    minutes: 15,
                },
                Reminder {
                    method: Method::Email,
                    minutes: 1440,
                },
            ]),
            ..About::default()
        },
        ..plain.clone()
    };

    let plain = session.create(&work, &plain).unwrap();
    let reminded = session.create(&work, &reminded).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    // Google gives an event sent without reminders its calendar's defaults (`MOD-07-F03`).
    let plain = google.event(ACCOUNT_ID, "work", &plain).unwrap();
    assert_eq!(plain.get("reminders"), None);
    let reminded = google.event(ACCOUNT_ID, "work", &reminded).unwrap();
    assert_eq!(
        reminded["reminders"],
        json!({"useDefault": false, "overrides": [
            {"method": "popup", "minutes": 15},
            {"method": "email", "minutes": 1440},
        ]})
    );
}

#[test]
fn an_events_reminders_are_changed_and_set_back_to_its_calendars_defaults() {
    let (google, store, dir, client_file, mut cache) = synchronized_once("update-reminders");
    let mut review = timed(
        "review",
        "Review",
        "2026-11-12T10:00:00+01:00",
        "2026-11-12T11:00:00+01:00",
    );
    review["reminders"] =
        json!({"useDefault": false, "overrides": [{"method": "popup", "minutes": 10}]});
    google.add_event(ACCOUNT_ID, "work", review);
    let synced = sync(&google, &store, &client_file, ACCOUNT_ID, &mut cache);
    let session = Session::sign_in(&client_file, &google.endpoints, &store, ACCOUNT_ID).unwrap();
    let work = cache.calendar_list().unwrap().remove(0);
    let etag = cached(&cache, "work", "review").unwrap().etag;
    let own = Patch {
        reminders: Some(Reminders::Own(vec![Reminder {
            method: Method::Email,
            minutes: 60,
        }])),
        ..Patch::default()
    };
    let defaults = Patch {
        reminders: Some(Reminders::Default),
        ..Patch::default()
    };

    let etag = session.update(&work, "review", &etag, &own, false).unwrap();
    let changed = google.event(ACCOUNT_ID, "work", "review").unwrap();
    let reset = session.update(&work, "review", &etag, &defaults, false);
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(synced, Ok(()));
    assert_eq!(
        changed["reminders"],
        json!({"useDefault": false, "overrides": [{"method": "email", "minutes": 60}]})
    );
    // The overrides go with the defaults, as Google takes both together from no one.
    assert!(reset.is_ok(), "{reset:?}");
    let reset = google.event(ACCOUNT_ID, "work", "review").unwrap();
    assert_eq!(reset["reminders"]["useDefault"], true);
    assert!(
        (reset["reminders"]["overrides"].as_array()).is_none_or(Vec::is_empty),
        "{reset}"
    );
}
