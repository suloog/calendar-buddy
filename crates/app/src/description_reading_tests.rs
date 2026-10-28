//! The test cases of `US-10-02` (`TC-10-08`…`TC-10-17`, and `TC-10-04`, which waited for it,
//! `DEC-215`), driving the whole app against a fake Ollama and, where a case has a Google account,
//! a fake Google (`DEC-65`), on a fixed clock: Monday 26 October 2026, 09:00 in Warsaw. Each step
//! is announced with `step` before it is carried out. `TC-10-18` runs in `calendar-core`
//! (`described::tests`); `TC-10-19` is run by hand.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use calendar_core::accounts::{Account, Accounts, Check};
use calendar_core::cache::Cache;
use calendar_core::create::QuickEntry;
use calendar_core::editor::Editor;
use calendar_core::navigation::View;
use calendar_core::offline;
use calendar_core::ollama::fake::FakeOllama;
use calendar_core::ollama::{Availability, DEFAULT_MODEL};
use calendar_core::platform::Network;
use calendar_core::settings::Settings;
use calendar_core::side_panel::Panel;
use calendar_core::sync::{Provider, Window};
use calendar_core::texts;
use calendar_core::theme::Theme;
use calendar_core::week::GridView;
use calendar_google::fake::{ACCOUNT_ID, CLIENT_ID, EMAIL, FakeGoogle, FakeStore};
use calendar_google::{CLIENT_FILE, Session};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Europe::Warsaw;
use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use serde_json::json;

use super::tests::{description_box, meet_with, press, q_listed, shows, step, until};
use super::{Checker, Reader};
use crate::network::Online;
use crate::sync::Synchronization;
use crate::{CalendarApp, Dialog};

/// Monday 26 October 2026, 09:00 in Warsaw.
fn now() -> DateTime<Utc> {
    Warsaw
        .with_ymd_and_hms(2026, 10, 26, 9, 0, 0)
        .unwrap()
        .to_utc()
}

/// The fake model's answer: add an event with the title, the kind of day and its value, the time
/// and the length.
fn add(title: &str, kind: &str, day: &str, time: Option<&str>, length: Option<u32>) -> String {
    json!({
        "action": "add", "title": title, "kind": kind, "day": day, "time": time,
        "duration_min": length, "guests": [], "meet": false,
    })
    .to_string()
}

fn kino() -> String {
    add("Kino", "tomorrow", "", Some("20:00"), None)
}

/// The system's word on the network, changed by the test.
#[derive(Clone, Default)]
struct Switch(Arc<Mutex<Option<Changed>>>);

type Changed = Box<dyn Fn(bool) + Send + Sync>;

impl Network for Switch {
    fn watch(&self, changed: Changed) {
        changed(true);
        *self.0.lock().unwrap() = Some(changed);
    }
}

impl Switch {
    fn set(&self, online: bool) {
        (self.0.lock().unwrap().as_ref().unwrap())(online);
    }
}

/// Google, faked, with the account `owner@example.com` signed in: its primary calendar "Work" is
/// the default calendar, and new events last 30 minutes in it.
struct Google {
    fake: FakeGoogle,
    store: Arc<FakeStore>,
    client_file: PathBuf,
}

impl Google {
    fn start(dir: &Path, workspace: bool) -> Self {
        let fake = FakeGoogle::start(workspace.then_some("firma.example"), &[]);
        let store = Arc::new(FakeStore::default());
        let token = fake.signed_in(ACCOUNT_ID);
        (store.saved.lock().unwrap()).push((ACCOUNT_ID.to_string(), token));
        let client_file = dir.join(CLIENT_FILE);
        let client = json!({"installed": {"client_id": CLIENT_ID, "client_secret": "test-secret"}});
        std::fs::write(&client_file, client.to_string()).unwrap();
        fake.add_calendar(
            ACCOUNT_ID,
            json!({
                "id": EMAIL, "summary": "Work", "timeZone": "Europe/Warsaw",
                "backgroundColor": "#9fe1e7", "accessRole": "owner", "primary": true,
            }),
        );
        fake.set_default_length(ACCOUNT_ID, 30);
        Google {
            fake,
            store,
            client_file,
        }
    }
}

/// The app as a test case starts it, with what it runs against.
struct World {
    harness: Harness<'static, CalendarApp>,
    ollama: FakeOllama,
    google: Option<Google>,
    network: Switch,
    dir: PathBuf,
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Empty config, cache and data directories under `dir`; events from a description on, with a
/// fake Ollama that has the model; with a Google account or "Use without an account".
fn world_in(dir: PathBuf, google: bool) -> World {
    world_with(dir, google, false, |_| {})
}

/// As `world_in`, the Google account a Workspace one when `workspace`, which may search its
/// organization's directory, and `prepare` given the fake Google before the app starts.
fn world_with(
    dir: PathBuf,
    google: bool,
    workspace: bool,
    prepare: impl FnOnce(&FakeGoogle),
) -> World {
    std::fs::create_dir_all(&dir).unwrap();
    let ollama = FakeOllama::start(&[DEFAULT_MODEL]);
    let google = google.then(|| Google::start(&dir, workspace));
    if let Some(google) = &google {
        prepare(&google.fake);
    }
    let network = Switch::default();
    let mut settings = Settings::default();
    settings.ollama.address = ollama.address.clone();
    let mut accounts = Accounts::default();
    let sign_in: Box<dyn Fn(&str) -> _ + Send> = match &google {
        Some(google) => {
            accounts.add(Account {
                id: ACCOUNT_ID.to_string(),
                email: EMAIL.to_string(),
                workspace,
                contacts: true,
                directory: workspace,
            });
            let (client_file, endpoints, store) = (
                google.client_file.clone(),
                google.fake.endpoints.clone(),
                google.store.clone(),
            );
            Box::new(move |id: &str| {
                Session::sign_in(&client_file, &endpoints, store.as_ref(), id)
                    .map(|session| Box::new(session) as Box<dyn Provider + Send>)
            })
        }
        None => Box::new(|_: &str| Err(calendar_core::sync::SyncError("no account".into()))),
    };
    let ids: Vec<String> = (accounts.list().iter())
        .map(|e| e.account.id.clone())
        .collect();
    let window = Window::around(NaiveDate::from_ymd_opt(2026, 10, 26).unwrap(), Warsaw);
    let (paths, switch) = (dir.clone(), network.clone());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_eframe(move |cc| {
            let ctx = &cc.egui_ctx;
            let mut app = CalendarApp::preview(ctx, View::Grid(GridView::Week), Theme::Light);
            let cache = paths.join("cache.sqlite3");
            app.sync = Synchronization::with(
                ctx,
                move || Cache::open(&cache).unwrap(),
                sign_in,
                ids,
                window,
                Default::default(),
            );
            app.sync.use_local(Some(paths.join("local.sqlite3")));
            if !accounts.list().is_empty() {
                app.sync
                    .checked(&[(ACCOUNT_ID.to_string(), Check::Works)], window);
            }
            app.accounts = accounts;
            app.online = Online::watch(ctx, &switch);
            app.settings = settings;
            let (writer, saved) = std::sync::mpsc::channel::<Settings>();
            let file = paths.join("settings");
            std::thread::spawn(move || {
                for settings in saved {
                    settings.save(&file).unwrap();
                }
            });
            app.settings_writer = Some(writer);
            app.checker = Checker::start(ctx);
            app.reader = Reader::start(ctx);
            app.clock = now;
            app.checker.ask(&app.settings.ollama);
            app
        });
    until(&mut harness, |app| app.sync.local().is_some());
    if google.is_some() {
        // The account's calendars, its default length and "My calendar" beside them.
        until(&mut harness, |app| {
            let names: Vec<&str> = app
                .sync
                .calendars()
                .iter()
                .map(|c| c.name.as_str())
                .collect();
            names.contains(&"Work")
                && names.contains(&"My calendar")
                && app.sync.default_length(ACCOUNT_ID) == 30
        });
    } else {
        harness.state_mut().use_without_account();
    }
    until(&mut harness, |app| {
        app.description.availability == Availability::Available
    });
    World {
        harness,
        ollama,
        google,
        network,
        dir,
    }
}

fn world(test: &str, google: bool) -> World {
    let dir = std::env::temp_dir().join(format!("calendar-buddy-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    world_in(dir, google)
}

/// `q`, then the description typed, without sending it.
fn describe(harness: &mut Harness<'static, CalendarApp>, text: &str) {
    press(harness, 'q');
    let deadline = Instant::now() + Duration::from_secs(10);
    while !description_box(harness).is_some_and(|b| b.is_focused()) {
        assert!(Instant::now() < deadline, "the box never took the keyboard");
        harness.step();
    }
    description_box(harness).unwrap().type_text(text);
    harness.run_steps(2);
}

/// Enter in the box, then waits until the model's answer was taken in.
fn send(harness: &mut Harness<'static, CalendarApp>) {
    harness.key_press(egui::Key::Enter);
    harness.run_steps(2);
    until(harness, |app| !app.description.reading);
}

/// `q`, the description typed and sent.
fn describe_and_send(harness: &mut Harness<'static, CalendarApp>, text: &str) {
    describe(harness, text);
    send(harness);
}

/// The "New event" window, when it is open.
fn new_event(harness: &Harness<'static, CalendarApp>) -> Option<QuickEntry> {
    match &harness.state().dialog {
        Some(Dialog::QuickEntry(entry, _)) => Some(entry.clone()),
        _ => None,
    }
}

fn opened(harness: &Harness<'static, CalendarApp>) -> QuickEntry {
    new_event(harness).expect("the \"New event\" window is open")
}

/// The window's start and end on the wall clock in Warsaw.
fn times(entry: &QuickEntry) -> (String, String) {
    let at = |t: DateTime<Utc>| {
        t.with_timezone(&Warsaw)
            .format("%a %d %b %H:%M")
            .to_string()
    };
    (at(entry.start), at(entry.end))
}

fn close(harness: &mut Harness<'static, CalendarApp>) {
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);
    assert!(harness.state().dialog.is_none());
}

/// Clicks the window's or the editor's Save and waits until the event titled `title` is shown.
fn save(harness: &mut Harness<'static, CalendarApp>, title: &str) {
    harness.get_by_label(texts::SAVE).click();
    harness.run_steps(2);
    until(harness, |app| app.events.iter().any(|e| e.title == title));
}

fn titled<'a>(app: &'a CalendarApp, title: &str) -> Vec<&'a calendar_core::model::Event> {
    app.events.iter().filter(|e| e.title == title).collect()
}

/// The name of the calendar of the event shown titled `title`.
fn calendar_of(app: &CalendarApp, title: &str) -> String {
    let event = titled(app, title)[0];
    let calendar = (app.sync.calendars().iter())
        .find(|c| c.id == event.calendar_id && c.account_id == event.account_id)
        .unwrap();
    calendar.name.clone()
}

/// Steps the app for `millis`, as time passes with nothing done.
fn idle(harness: &mut Harness<'static, CalendarApp>, millis: u64) {
    let until = Instant::now() + Duration::from_millis(millis);
    while Instant::now() < until {
        std::thread::sleep(Duration::from_millis(10));
        harness.step();
    }
}

#[test]
fn tc_10_04_a_description_reaches_only_the_ollama_set_and_google_receives_nothing_of_it() {
    const ID: &str = "TC-10-04";
    let mut world = world("tc-10-04", true);
    let kawa = add("Kawa", "tomorrow", "", Some("10:00"), None);
    world.ollama.answer_all(&kawa);
    let google = world.google.as_ref().unwrap();
    let before = google.fake.requests().len();

    step(ID, 1, "Press q, type the description and press Enter.");
    describe_and_send(&mut world.harness, "Kawa jutro o 10");
    let entry = opened(&world.harness);
    assert_eq!(entry.title, "Kawa");
    let expected = ("Tue 27 Oct 10:00".into(), "Tue 27 Oct 10:30".into());
    assert_eq!(times(&entry), expected);

    step(
        ID,
        2,
        "Close the window without saving; read the record of the app's requests.",
    );
    close(&mut world.harness);
    let settings = &world.harness.state().settings.ollama;
    assert!(calendar_core::ollama::is_local(&settings.address));
    assert_eq!(settings.address, world.ollama.address);
    assert_eq!(world.ollama.descriptions(), ["Kawa jutro o 10"]);

    step(ID, 3, "Read the fake Google's record.");
    let google = world.google.as_ref().unwrap();
    let since: Vec<String> = google.fake.requests()[before..].to_vec();
    for request in &since {
        assert!(request.starts_with("GET "), "{request}");
        assert!(!request.contains("Kawa"), "{request}");
    }
    assert!(google.fake.patched().is_empty());
}

#[test]
fn tc_10_08_the_box_its_keys_and_its_shortcut() {
    const ID: &str = "TC-10-08";
    let mut world = world("tc-10-08", false);
    world.ollama.answer_all(&kino());
    let harness = &mut world.harness;
    harness.state_mut().panel = Panel::new(true);
    harness.run_steps(3);
    assert!(
        description_box(harness).is_none(),
        "the side panel is collapsed"
    );

    step(ID, 1, "In the week view press q.");
    press(harness, 'q');
    until(harness, |app| app.panel.expanded());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !description_box(harness).is_some_and(|b| b.is_focused()) {
        assert!(Instant::now() < deadline, "the box never took the keyboard");
        harness.step();
    }
    let field = description_box(harness).unwrap();
    assert_eq!(
        field.accesskit_node().placeholder(),
        Some(texts::DESCRIBE_EVENT)
    );
    assert!(field.rect().bottom() <= meet_with(harness).rect().top());
    let one_line = field.rect().height();

    step(
        ID,
        2,
        "Type \"Kino jutro\", press Shift+Enter, type \"o 20\".",
    );
    description_box(harness).unwrap().type_text("Kino jutro");
    harness.run_steps(1);
    harness.key_combination_modifiers(egui::Modifiers::SHIFT, &[egui::Key::Enter]);
    harness.run_steps(1);
    description_box(harness).unwrap().type_text("o 20");
    harness.run_steps(3);
    let field = description_box(harness).unwrap();
    assert_eq!(field.value().as_deref(), Some("Kino jutro\no 20"));
    assert!(
        field.rect().height() > one_line,
        "the field grew to two lines"
    );
    assert!(world.ollama.descriptions().is_empty());
    let harness = &mut world.harness;

    step(ID, 3, "Press Esc, then the key for the next week.");
    let week = harness.state().navigation.selected;
    harness.key_press(egui::Key::Escape);
    harness.run_steps(2);
    assert!(!description_box(harness).unwrap().is_focused());
    press(harness, 'n');
    assert_eq!(
        harness.state().navigation.selected,
        week + chrono::Duration::days(7)
    );

    step(ID, 4, "Press q, then Enter.");
    press(harness, 'q');
    let deadline = Instant::now() + Duration::from_secs(10);
    while !description_box(harness).is_some_and(|b| b.is_focused()) {
        assert!(Instant::now() < deadline, "the box never took the keyboard");
        harness.step();
    }
    send(harness);
    assert_eq!(world.ollama.descriptions(), ["Kino jutro\no 20"]);
    let harness = &mut world.harness;
    assert_eq!(opened(harness).title, "Kino");

    step(
        ID,
        5,
        "Close the window; open the list of keyboard shortcuts.",
    );
    close(harness);
    assert!(q_listed(harness));
}

#[test]
fn tc_10_09_focusing_the_box_loads_the_model_for_5_minutes_and_nothing_at_start() {
    const ID: &str = "TC-10-09";
    let mut world = world("tc-10-09", false);
    world.ollama.answer_all(&kino());

    step(ID, 1, "Start the app; read the fake Ollama's record.");
    let requests = world.ollama.requests();
    assert!(!requests.is_empty());
    assert!(
        (requests.iter()).all(|r| r == "GET /api/version" || r == "GET /api/tags"),
        "{requests:?}"
    );

    step(ID, 2, "Press q.");
    press(&mut world.harness, 'q');
    let deadline = Instant::now() + Duration::from_secs(10);
    while world.ollama.chats().is_empty() {
        assert!(Instant::now() < deadline, "the model was never loaded");
        world.harness.step();
    }
    idle(&mut world.harness, 200);
    let chats = world.ollama.chats();
    assert_eq!(chats.len(), 1, "{chats:?}");
    assert_eq!(chats[0]["model"], DEFAULT_MODEL);
    assert_eq!(chats[0]["messages"], json!([]));
    assert_eq!(chats[0]["keep_alive"], "5m");

    step(ID, 3, "Type the description and press Enter.");
    description_box(&world.harness)
        .unwrap()
        .type_text("Kino jutro o 20");
    world.harness.run_steps(2);
    send(&mut world.harness);
    let chats = world.ollama.chats();
    assert_eq!(chats.len(), 2, "{chats:?}");
    assert_eq!(chats[1]["keep_alive"], "5m");
    assert_eq!(world.ollama.descriptions(), ["Kino jutro o 20"]);
    assert!(new_event(&world.harness).is_some());
}

#[test]
fn tc_10_10_reading_shows_and_does_not_block_and_esc_drops_the_late_answer() {
    const ID: &str = "TC-10-10";
    let mut world = world("tc-10-10", false);
    world.ollama.answer_all(&kino());
    world.ollama.hold();
    let harness = &mut world.harness;
    let events = harness.state().events.len();

    step(ID, 1, "Press q, type the description and press Enter.");
    describe(harness, "Kino jutro o 20");
    harness.key_press(egui::Key::Enter);
    harness.run_steps(3);
    assert!(harness.state().description.reading);
    assert!(shows(harness, texts::READING));
    assert!(harness.state().dialog.is_none());

    step(
        ID,
        2,
        "Press the key for the next week, then the key for today.",
    );
    let week = harness.state().navigation.selected;
    press(harness, 'n');
    assert_eq!(
        harness.state().navigation.selected,
        week + chrono::Duration::days(7)
    );
    assert!(shows(harness, texts::READING));
    press(harness, 't');
    assert_eq!(harness.state().navigation.selected, week);
    assert!(shows(harness, texts::READING));

    step(ID, 3, "Press q, then Esc.");
    press(harness, 'q');
    let deadline = Instant::now() + Duration::from_secs(10);
    while !description_box(harness).is_some_and(|b| b.is_focused()) {
        assert!(Instant::now() < deadline, "the box never took the keyboard");
        harness.step();
    }
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);
    assert!(!harness.state().description.reading);
    assert!(!shows(harness, texts::READING));
    assert!(harness.state().dialog.is_none());

    step(ID, 4, "Release the fake Ollama's answer.");
    let chats = world.ollama.chats().len();
    world.ollama.release();
    // Long enough for the answer to come back and be dropped.
    idle(&mut world.harness, 500);
    let harness = &world.harness;
    assert_eq!(world.ollama.chats().len(), chats);
    assert!(harness.state().dialog.is_none());
    assert_eq!(harness.state().events.len(), events);
    assert!(titled(harness.state(), "Kino").is_empty());
}

#[test]
fn tc_10_11_trening_wieczorem_opens_new_event_today_at_18_as_a_new_event() {
    const ID: &str = "TC-10-11";
    let mut world = world("tc-10-11", false);
    let trening = add("Trening", "today", "", Some("evening"), None);
    world.ollama.answer_all(&trening);
    let harness = &mut world.harness;

    step(ID, 1, "Press q, type the description and press Enter.");
    describe_and_send(harness, "Trening wieczorem");
    let entry = opened(harness);
    assert_eq!(entry.title, "Trening");
    let expected = ("Mon 26 Oct 18:00".into(), "Mon 26 Oct 19:00".into());
    assert_eq!(times(&entry), expected);
    assert_eq!(entry.calendar.as_ref().unwrap().name, "My calendar");

    step(ID, 2, "Open the full editor from the window.");
    // The same event made by hand: the "New event" window at the same times, the title typed.
    let mut by_hand = harness.state().new_entry(|_| (entry.start, entry.end));
    by_hand.title = "Trening".into();
    let by_hand = Editor::from_quick(&by_hand, &harness.state().regional);
    harness.get_by_label(texts::MORE_OPTIONS).click();
    harness.run_steps(3);
    let Some(Dialog::Editor(editor)) = &harness.state().dialog else {
        panic!("the full editor is open");
    };
    assert_eq!(**editor, by_hand);
    assert!(editor.location.is_empty() && editor.description.is_empty());
    assert_eq!(editor.repeat, None);
    assert_eq!(editor.color, None);
    assert_eq!(editor.reminders, by_hand.reminders);

    step(ID, 3, "Before saving, read the local calendar.");
    assert!(titled(harness.state(), "Trening").is_empty());

    step(ID, 4, "Save.");
    save(harness, "Trening");
    let app = harness.state();
    let event = titled(app, "Trening")[0];
    let start = Warsaw.with_ymd_and_hms(2026, 10, 26, 18, 0, 0).unwrap();
    assert_eq!(
        event.timing,
        calendar_core::model::Timing::Timed {
            start: start.to_utc(),
            end: (start + chrono::Duration::hours(1)).to_utc(),
            zone: Warsaw,
        }
    );
    assert_eq!(calendar_of(app, "Trening"), "My calendar");
}

#[test]
fn tc_10_12_a_length_given_is_used_and_without_one_the_default_length() {
    const ID: &str = "TC-10-12";
    let mut world = world("tc-10-12", true);
    let given = [
        ("Spotkanie zespołu jutro o 10, 45 minut", 45, "10:45"),
        ("Code review tomorrow at 10 for two hours", 120, "12:00"),
        ("Warsztat jutro o 10, godzina i kwadrans", 75, "11:15"),
    ];
    for (text, length, _) in given {
        let answer = add("Spotkanie", "tomorrow", "", Some("10:00"), Some(length));
        world.ollama.answer(text, &answer);
    }
    let kawa = add("Kawa", "tomorrow", "", Some("10:00"), None);
    world.ollama.answer("Kawa jutro o 10", &kawa);
    let harness = &mut world.harness;

    step(
        ID,
        1,
        "Send each of the first three descriptions in turn, closing the window after each.",
    );
    for (text, _, end) in given {
        describe_and_send(harness, text);
        let (start, until) = times(&opened(harness));
        assert_eq!(start, "Tue 27 Oct 10:00", "{text}");
        assert_eq!(until, format!("Tue 27 Oct {end}"), "{text}");
        close(harness);
    }

    step(ID, 2, "Send \"Kawa jutro o 10\".");
    describe_and_send(harness, "Kawa jutro o 10");
    let entry = opened(harness);
    assert_eq!(entry.calendar.as_ref().unwrap().name, "Work");
    let expected = ("Tue 27 Oct 10:00".into(), "Tue 27 Oct 10:30".into());
    assert_eq!(times(&entry), expected);
}

#[test]
fn tc_10_13_days_and_times_are_worked_out_by_the_app() {
    const ID: &str = "TC-10-13";
    let mut world = world("tc-10-13", false);
    // Polish, English, the model's answer and the day and time expected.
    let rows = [
        (
            "Dentysta w środę o 8:15",
            "Dentist on Wednesday at 8:15",
            ("weekday", "wed", "08:15"),
            "Wed 28 Oct 08:15",
        ),
        (
            "przegląd w przyszły czwartek o 15",
            "review next Thursday at 3 pm",
            ("next_weekday", "thu", "15:00"),
            "Thu 05 Nov 15:00",
        ),
        (
            "standup za tydzień o 9",
            "standup in a week at 9",
            ("in_days", "7", "09:00"),
            "Mon 02 Nov 09:00",
        ),
        (
            "rozmowa pojutrze o 14:30",
            "call the day after tomorrow at 2:30 pm",
            ("day_after_tomorrow", "", "14:30"),
            "Wed 28 Oct 14:30",
        ),
        (
            "urodziny 14 listopada o 18",
            "birthday on 14 November at 6 pm",
            ("date", "11-14", "18:00"),
            "Sat 14 Nov 18:00",
        ),
        (
            "kawa jutro rano",
            "coffee tomorrow morning",
            ("tomorrow", "", "morning"),
            "Tue 27 Oct 09:00",
        ),
        (
            "retro w piątek po południu",
            "retro on Friday afternoon",
            ("weekday", "fri", "afternoon"),
            "Fri 30 Oct 14:00",
        ),
    ];
    for (pl, en, (kind, day, time), _) in rows {
        let answer = add("Wydarzenie", kind, day, Some(time), None);
        world.ollama.answer(pl, &answer);
        world.ollama.answer(en, &answer);
    }
    let serwis = add("Serwis auta", "yesterday", "", Some("12:00"), None);
    world.ollama.answer("Serwis auta wczoraj o 12", &serwis);
    let harness = &mut world.harness;
    let check = |harness: &mut Harness<'static, CalendarApp>, text: &str, expected: &str| {
        describe_and_send(harness, text);
        assert_eq!(times(&opened(harness)).0, expected, "{text}");
        close(harness);
    };

    step(
        ID,
        1,
        "Send each Polish description of the first seven rows, closing the window after each.",
    );
    for (pl, _, _, expected) in rows {
        check(harness, pl, expected);
    }

    step(ID, 2, "Send each English description of the same rows.");
    for (_, en, _, expected) in rows {
        check(harness, en, expected);
    }

    step(ID, 3, "Send \"Serwis auta wczoraj o 12\".");
    check(harness, "Serwis auta wczoraj o 12", "Sun 25 Oct 12:00");
}

#[test]
fn tc_10_14_no_event_opens_no_window_says_so_and_keeps_the_text() {
    const ID: &str = "TC-10-14";
    let mut world = world("tc-10-14", false);
    let none = json!({
        "action": "none", "title": "", "kind": "today", "day": "", "time": null,
        "duration_min": null, "guests": [], "meet": false,
    });
    world.ollama.answer("jaka jest pogoda?", &none.to_string());
    world
        .ollama
        .answer("Kino jutro o 20", "Kino jutro o 20, a chętnie!");
    let harness = &mut world.harness;
    let events = harness.state().events.len();

    step(
        ID,
        1,
        "Press q, type \"jaka jest pogoda?\" and press Enter.",
    );
    describe_and_send(harness, "jaka jest pogoda?");
    harness.run_steps(3);
    assert!(harness.state().dialog.is_none());
    assert!(shows(harness, texts::NO_EVENT_FOUND));
    let field = description_box(harness).unwrap();
    assert_eq!(field.value().as_deref(), Some("jaka jest pogoda?"));
    assert!(field.is_focused());

    step(
        ID,
        2,
        "Replace the text with \"Kino jutro o 20\" and press Enter.",
    );
    harness.key_combination_modifiers(egui::Modifiers::COMMAND, &[egui::Key::A]);
    description_box(harness)
        .unwrap()
        .type_text("Kino jutro o 20");
    harness.run_steps(2);
    assert!(!shows(harness, texts::NO_EVENT_FOUND));
    send(harness);
    harness.run_steps(3);
    assert!(harness.state().dialog.is_none());
    assert!(shows(harness, texts::NO_EVENT_FOUND));
    let field = description_box(harness).unwrap();
    assert_eq!(field.value().as_deref(), Some("Kino jutro o 20"));
    assert_eq!(
        world.ollama.descriptions(),
        ["jaka jest pogoda?", "Kino jutro o 20"]
    );

    step(ID, 3, "Read the local calendar.");
    idle(&mut world.harness, 200);
    assert_eq!(world.harness.state().events.len(), events);
}

/// Where the child half of `tc_10_15` keeps the app's directories, and what it does.
const CHILD_DIR: &str = "CALENDAR_BUDDY_TC_10_15_DIR";
const CHILD_SAVES: &str = "CALENDAR_BUDDY_TC_10_15_SAVES";
const MARKER: &str = "Zebra-7731";
/// The description sent, and a piece of the model's answer only it holds.
const DESCRIPTION: &str = "Spotkanie Zebra-7731 jutro o 10";
const ANSWERED: &str = r#""title":"Spotkanie Zebra-7731""#;

/// The child half of `tc_10_15`: the app on the directories it is given sends the description and
/// closes the window, or saves the event. Does nothing when run on its own.
#[test]
fn described_with_the_marker() {
    let Some(dir) = std::env::var_os(CHILD_DIR) else {
        return;
    };
    let saves = std::env::var_os(CHILD_SAVES).is_some();
    let mut world = world_in(PathBuf::from(dir), false);
    let title = format!("Spotkanie {MARKER}");
    world
        .ollama
        .answer_all(&add(&title, "tomorrow", "", Some("10:00"), None));
    let harness = &mut world.harness;
    describe_and_send(harness, DESCRIPTION);
    assert_eq!(opened(harness).title, title);
    if saves {
        save(harness, &title);
        // Written to the local calendar's file before the app goes.
        idle(harness, 500);
    } else {
        close(harness);
    }
    // The directories stay for the parent to search.
    std::mem::forget(world);
}

/// Runs the child half with the most detailed logging, giving all it printed.
fn run_child(dir: &Path, saves: bool) -> String {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap());
    child.args([
        "--exact",
        "description::reading_tests::described_with_the_marker",
        "--nocapture",
        "--test-threads=1",
    ]);
    child.env(CHILD_DIR, dir).env("RUST_LOG", "trace");
    if saves {
        child.env(CHILD_SAVES, "1");
    }
    let output = child.output().unwrap();
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{printed}");
    assert!(printed.contains("1 passed"), "{printed}");
    printed
}

/// The files under `dir` holding `marker`.
fn files_with_marker(dir: &Path, marker: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_with_marker(&path, marker));
        } else if (std::fs::read(&path).unwrap())
            .windows(marker.len())
            .any(|w| w == marker.as_bytes())
        {
            found.push(path);
        }
    }
    found
}

#[test]
fn tc_10_15_descriptions_and_answers_appear_in_no_log_and_no_file() {
    const ID: &str = "TC-10-15";
    let dir = std::env::temp_dir().join(format!("calendar-buddy-tc-10-15-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    step(
        ID,
        1,
        "Press q, send the description; close the window without saving.",
    );
    let printed = run_child(&dir, false);

    step(
        ID,
        2,
        "Search the log files, the captured output and the config, cache and data directories.",
    );
    assert!(!printed.contains(MARKER), "{printed}");
    assert!(dir.join("settings").exists() || dir.join("local.sqlite3").exists());
    assert_eq!(files_with_marker(&dir, MARKER), Vec::<PathBuf>::new());

    step(ID, 3, "Send the description again and Save; search again.");
    let printed = run_child(&dir, true);
    assert!(!printed.contains(MARKER), "{printed}");
    let found = files_with_marker(&dir, MARKER);
    let local = |file: &PathBuf| {
        file.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("local.sqlite3")
    };
    assert!(
        found.iter().any(local),
        "the saved event is in the local calendar"
    );
    // The views read every event from the cache, the local calendar's too (`MOD-09`): the saved
    // event's copy is there, as its title; the description and the answer are nowhere (`DEC-222`).
    for file in &found {
        let name = file.file_name().unwrap().to_string_lossy();
        let cache = name.starts_with("cache.sqlite3");
        assert!(local(file) || cache, "{}", file.display());
    }
    for trace in [DESCRIPTION, ANSWERED] {
        assert_eq!(
            files_with_marker(&dir, trace),
            Vec::<PathBuf>::new(),
            "{trace}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tc_10_16_a_description_of_600_characters_is_taken_whole_and_cut_at_500() {
    const ID: &str = "TC-10-16";
    let mut world = world("tc-10-16", false);
    world.ollama.answer_all(&kino());
    let mut text: String = format!("Kino jutro o 20. {}", "żółć ".repeat(120))
        .chars()
        .take(594)
        .collect();
    text.push_str("KONIEC");
    assert_eq!(text.chars().count(), 600);
    let harness = &mut world.harness;

    step(ID, 1, "Press q and paste the description.");
    describe(harness, &text);
    assert_eq!(harness.state().description.field, text);
    let field = description_box(harness).unwrap();
    assert!(field.value().unwrap().ends_with("KONIEC"));

    step(ID, 2, "Press Enter; read the fake Ollama's record.");
    send(harness);
    let sent = world.ollama.descriptions();
    let first: String = text.chars().take(500).collect();
    assert_eq!(sent, [first]);
    assert!(!sent[0].contains("KONIEC"));
    assert_eq!(opened(&world.harness).title, "Kino");
}

#[test]
fn tc_10_17_offline_only_a_local_calendar_takes_it_and_without_an_account_my_calendar() {
    const ID: &str = "TC-10-17";
    let mut case_a = world("tc-10-17-a", true);
    case_a.ollama.answer_all(&kino());
    case_a.network.set(false);
    let google = case_a.google.as_ref().unwrap();
    google.fake.disconnect();
    let before = google.fake.requests().len();
    let harness = &mut case_a.harness;

    step(ID, 1, "Case A: send the description.");
    describe_and_send(harness, "Kino jutro o 20");
    let entry = opened(harness);
    assert_eq!(times(&entry).0, "Tue 27 Oct 20:00");
    let work = entry.calendar.clone().unwrap();
    assert_eq!(work.name, "Work");
    let refused = offline::refused_in(false, Some(&work)).expect("offline, Work takes nothing");
    assert!(shows(harness, refused));
    let save_button = harness.get_by_label(texts::SAVE);
    assert!(save_button.accesskit_node().is_disabled());

    step(ID, 2, "Choose \"My calendar\" in the picker and Save.");
    let mine = (harness.state().sync.calendars().iter())
        .find(|c| c.name == "My calendar")
        .cloned();
    if let Some(Dialog::QuickEntry(entry, _)) = &mut harness.state_mut().dialog {
        entry.calendar = mine;
    }
    harness.run_steps(2);
    save(harness, "Kino");
    assert_eq!(calendar_of(harness.state(), "Kino"), "My calendar");
    let google = case_a.google.as_ref().unwrap();
    assert!(google.fake.requests().len() == before);
    drop(case_a);

    step(ID, 3, "Case B: send the description and Save.");
    let mut world = world("tc-10-17-b", false);
    world.ollama.answer_all(&kino());
    let harness = &mut world.harness;
    describe_and_send(harness, "Kino jutro o 20");
    let entry = opened(harness);
    assert_eq!(entry.calendar.as_ref().unwrap().name, "My calendar");
    save(harness, "Kino");
    assert_eq!(calendar_of(harness.state(), "Kino"), "My calendar");
}

/// Review B1: an answer that comes while a window is open waits, "Reading…" still shown, and is
/// taken in once the window closes; what the user typed in the window is not lost.
#[test]
fn an_answer_waits_while_a_window_is_open() {
    let mut world = world("answer-waits", false);
    world.ollama.answer_all(&kino());
    world.ollama.hold();
    let harness = &mut world.harness;
    describe(harness, "Kino jutro o 20");
    harness.key_press(egui::Key::Enter);
    harness.run_steps(3);
    assert!(harness.state().description.reading);

    press(harness, 'c');
    until(harness, |app| {
        matches!(app.dialog, Some(Dialog::QuickEntry(..)))
    });
    harness.run_steps(2);
    harness.event(egui::Event::Text("Dentist".into()));
    harness.run_steps(2);
    assert_eq!(opened(harness).title, "Dentist");

    world.ollama.release();
    idle(&mut world.harness, 500);
    let harness = &mut world.harness;
    assert_eq!(opened(harness).title, "Dentist");
    assert!(harness.state().description.reading);
    assert!(shows(harness, texts::READING));

    // Once the window is closed, the answer is taken in.
    harness.key_press(egui::Key::Escape);
    until(harness, |app| !app.description.reading);
    assert_eq!(opened(harness).title, "Kino");
}

/// Review B2: Ollama there with the model, but reading fails (a 500, as when the model runs out
/// of memory): the box says it found no event and keeps the description.
#[test]
fn a_failed_reading_says_no_event_was_found() {
    let mut world = world("failed-reading", false);
    world.ollama.fail();
    let harness = &mut world.harness;
    describe_and_send(harness, "Kino jutro o 20");
    idle(harness, 300);
    assert!(harness.state().dialog.is_none());
    assert_eq!(
        harness.state().description.availability,
        Availability::Available
    );
    assert!(shows(harness, texts::NO_EVENT_FOUND));
    let field = description_box(harness).unwrap();
    assert_eq!(field.value().as_deref(), Some("Kino jutro o 20"));
}

/// Review N10: an answer that comes after events from a description were turned off is dropped.
#[test]
fn an_answer_after_turning_the_feature_off_is_dropped() {
    let mut world = world("turned-off", false);
    world.ollama.answer_all(&kino());
    world.ollama.hold();
    let harness = &mut world.harness;
    describe(harness, "Kino jutro o 20");
    harness.key_press(egui::Key::Enter);
    harness.run_steps(3);
    harness.state_mut().settings.ollama.on = false;
    world.ollama.release();
    let harness = &mut world.harness;
    until(harness, |app| !app.description.reading);
    idle(harness, 200);
    assert!(harness.state().dialog.is_none());
}

#[path = "description_guests_tests.rs"]
mod guests_tests;
