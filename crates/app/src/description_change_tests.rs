//! The test cases of `US-10-04` (`TC-10-26`…`TC-10-31`): changing an event by describing the
//! change, driving the whole app against a fake Ollama and a fake Google (`DEC-65`), on the fixed
//! clock of the reading tests. Each step is announced with `step` before it is carried out.
//! `TC-10-18` runs in `calendar-core` (`described::tests`); `TC-10-19` is run by hand. The fake
//! Google's account is `owner@example.com` (`user@example.com` in the test cases).

use calendar_core::model::{Event, Timing};
use calendar_google::fake::ACCOUNT_ID;
use serde_json::Value;

use super::guests_tests::{editor, editor_times, with_guests_day, written};
use super::*;
use crate::description::{Matches, row};

/// The fake model's answer: change the event the words name, or the open one when they are
/// empty, to the kind of day and its value (`same` keeps the day) and the hour.
fn change(event: &str, kind: &str, day: &str, time: Option<&str>) -> String {
    json!({
        "action": "change", "event": event, "title": "", "kind": kind, "day": day,
        "time": time, "duration_min": null, "guests": [], "meet": null,
    })
    .to_string()
}

/// A single event of Google's, from `start` to `end` (RFC 3339, in Warsaw).
fn single(id: &str, title: &str, start: &str, end: &str) -> Value {
    json!({
        "id": id, "status": "confirmed", "summary": title,
        "start": {"dateTime": start, "timeZone": "Europe/Warsaw"},
        "end": {"dateTime": end, "timeZone": "Europe/Warsaw"},
    })
}

/// The app with the fake Google's "Work" holding `events`.
fn world_of(test: &str, events: Vec<Value>) -> World {
    let dir = std::env::temp_dir().join(format!("calendar-buddy-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    world_with(dir, true, false, |google| {
        for event in events {
            google.add_event(ACCOUNT_ID, EMAIL, event);
        }
    })
}

/// The events shown titled `title`, on the day `day` of October or November 2026 (`MM-DD`).
fn on(app: &CalendarApp, title: &str, day: &str) -> Option<usize> {
    let date = NaiveDate::parse_from_str(&format!("2026-{day}"), "%Y-%m-%d").unwrap();
    app.events.iter().position(|e| {
        e.title == title
            && match e.timing {
                Timing::Timed { start, .. } => start.with_timezone(&Warsaw).date_naive() == date,
                Timing::AllDay { start, .. } => start == date,
            }
    })
}

/// Opens the details of the event titled `title` on `day`, once the app has it.
fn open_details(harness: &mut Harness<'static, CalendarApp>, title: &str, day: &str) {
    until(harness, |app| on(app, title, day).is_some());
    let event = on(harness.state(), title, day).unwrap();
    harness.state_mut().open(event, None);
    harness.run_steps(3);
    assert_eq!(details(harness).map(|e| e.title), Some(title.to_string()));
}

/// The event whose details are open.
fn details(harness: &Harness<'static, CalendarApp>) -> Option<Event> {
    match &harness.state().dialog {
        Some(Dialog::Event(event, ..)) => Some((**event).clone()),
        _ => None,
    }
}

/// The start and end of an event shown, on the wall clock in Warsaw.
fn shown_at(event: &Event) -> (String, String) {
    let Timing::Timed { start, end, .. } = event.timing else {
        panic!("a timed event");
    };
    let at = |t: DateTime<Utc>| {
        t.with_timezone(&Warsaw)
            .format("%a %d %b %H:%M")
            .to_string()
    };
    (at(start), at(end))
}

/// `q`, the box's text replaced with `text`, and sent.
fn describe_anew(harness: &mut Harness<'static, CalendarApp>, text: &str) {
    press(harness, 'q');
    let deadline = Instant::now() + Duration::from_secs(10);
    while !description_box(harness).is_some_and(|b| b.is_focused()) {
        assert!(Instant::now() < deadline, "the box never took the keyboard");
        harness.step();
    }
    harness.key_combination_modifiers(egui::Modifiers::COMMAND, &[egui::Key::A]);
    description_box(harness).unwrap().type_text(text);
    harness.run_steps(2);
    send(harness);
}

fn google(world: &World) -> &FakeGoogle {
    &world.google.as_ref().unwrap().fake
}

/// Waits until Google has `n` writes of events since `before`, and gives them.
fn writes(world: &mut World, before: usize, n: usize) -> Vec<String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while written(google(world), before).len() < n {
        assert!(Instant::now() < deadline, "Google never got {n} writes");
        std::thread::sleep(Duration::from_millis(10));
        world.harness.step();
    }
    idle(&mut world.harness, 200);
    written(google(world), before)
}

fn matches<'a>(harness: &'a Harness<'static, CalendarApp>) -> Option<&'a Matches> {
    harness.state().matches.as_ref()
}

#[test]
fn tc_10_26_the_open_event_moves_to_15_its_length_kept_and_only_save_changes_it() {
    const ID: &str = "TC-10-26";
    let mut world = world_of(
        "tc-10-26",
        vec![single(
            "planning",
            "Sprint planning",
            "2026-10-26T10:00:00+01:00",
            "2026-10-26T11:00:00+01:00",
        )],
    );
    world
        .ollama
        .answer("przesuń na 15", &change("", "same", "", Some("15:00")));
    let before = google(&world).requests().len();

    step(
        ID,
        1,
        "Open the details of \"Sprint planning\"; press q, send the description.",
    );
    open_details(&mut world.harness, "Sprint planning", "10-26");
    describe_and_send(&mut world.harness, "przesuń na 15");
    let opened = editor(&world.harness);
    assert_eq!(opened.title, "Sprint planning");
    assert!(
        opened.opened.is_some(),
        "the event's editing, not a new one"
    );
    let expected = ("Mon 26 Oct 15:00".into(), "Mon 26 Oct 16:00".into());
    assert_eq!(editor_times(&world.harness), expected);

    step(ID, 2, "Read the fake Google's record.");
    assert!(written(google(&world), before).is_empty());
    let shown = on(world.harness.state(), "Sprint planning", "10-26").unwrap();
    let expected = ("Mon 26 Oct 10:00".into(), "Mon 26 Oct 11:00".into());
    assert_eq!(shown_at(&world.harness.state().events[shown]), expected);

    step(ID, 3, "Save.");
    world.harness.get_by_label(texts::SAVE).click();
    world.harness.run_steps(2);
    let sent = writes(&mut world, before, 1);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(sent[0].starts_with("PATCH "), "{sent:?}");
    assert!(sent[0].contains("/events/planning"), "{sent:?}");
    let kept = google(&world).event(ACCOUNT_ID, EMAIL, "planning").unwrap();
    assert_eq!(kept["start"]["dateTime"], "2026-10-26T15:00:00+01:00");
    assert_eq!(kept["end"]["dateTime"], "2026-10-26T16:00:00+01:00");
    until(&mut world.harness, |app| {
        on(app, "Sprint planning", "10-26")
            .is_some_and(|i| shown_at(&app.events[i]).0 == "Mon 26 Oct 15:00")
    });
}

#[test]
fn tc_10_27_the_named_event_moves_to_friday_at_the_same_time_and_no_event_is_added() {
    const ID: &str = "TC-10-27";
    let mut world = world_of(
        "tc-10-27",
        vec![single(
            "ania",
            "Spotkanie z Anią",
            "2026-10-28T11:00:00+01:00",
            "2026-10-28T12:00:00+01:00",
        )],
    );
    let description = "przesuń spotkanie z Anią na piątek";
    let answer = change("spotkanie z Anią", "weekday", "fri", None);
    world.ollama.answer(description, &answer);
    until(&mut world.harness, |app| {
        on(app, "Spotkanie z Anią", "10-28").is_some()
    });
    let before = google(&world).requests().len();

    step(ID, 1, "Press q, send the description.");
    assert!(world.harness.state().dialog.is_none(), "no details open");
    describe_and_send(&mut world.harness, description);
    let opened = editor(&world.harness);
    assert_eq!(opened.title, "Spotkanie z Anią");
    assert_eq!(opened.opened.as_ref().unwrap().0.id, "ania");
    let expected = ("Fri 30 Oct 11:00".into(), "Fri 30 Oct 12:00".into());
    assert_eq!(editor_times(&world.harness), expected);

    step(ID, 2, "Save; read the fake Google's record.");
    world.harness.get_by_label(texts::SAVE).click();
    world.harness.run_steps(2);
    let sent = writes(&mut world, before, 1);
    assert_eq!(sent.len(), 1, "one update and nothing created: {sent:?}");
    assert!(sent[0].starts_with("PATCH "), "{sent:?}");
    assert!(sent[0].contains("/events/ania"), "{sent:?}");
    until(&mut world.harness, |app| {
        on(app, "Spotkanie z Anią", "10-30").is_some()
    });
    let app = world.harness.state();
    assert_eq!(titled(app, "Spotkanie z Anią").len(), 1);
    assert!(on(app, "Spotkanie z Anią", "10-28").is_none());
}

#[test]
fn tc_10_28_a_past_event_is_not_found_by_name_but_is_changed_when_its_details_are_open() {
    const ID: &str = "TC-10-28";
    let mut world = world_of(
        "tc-10-28",
        vec![single(
            "ania",
            "Spotkanie z Anią",
            "2026-10-19T11:00:00+02:00",
            "2026-10-19T12:00:00+02:00",
        )],
    );
    let first = "przesuń spotkanie z Anią na piątek";
    let second = "przesuń na piątek";
    let answer = change("spotkanie z Anią", "weekday", "fri", None);
    world.ollama.answer(first, &answer);
    world
        .ollama
        .answer(second, &change("", "weekday", "fri", None));
    until(&mut world.harness, |app| {
        on(app, "Spotkanie z Anią", "10-19").is_some()
    });

    step(ID, 1, "With no details open, send the first description.");
    describe_and_send(&mut world.harness, first);
    let harness = &mut world.harness;
    harness.run_steps(3);
    assert!(harness.state().dialog.is_none(), "no editing opens");
    assert!(harness.state().description.no_such_event);
    assert!(shows(harness, texts::NO_SUCH_EVENT));
    let field = description_box(harness).unwrap();
    assert_eq!(field.value().as_deref(), Some(first));
    assert!(field.is_focused());

    step(
        ID,
        2,
        "Go to the week of 19 October, open the event's details; press q, send the second \
         description.",
    );
    harness.key_press(egui::Key::Escape);
    harness.run_steps(2);
    let monday = NaiveDate::from_ymd_opt(2026, 10, 19).unwrap();
    harness.state_mut().navigation.go_to(monday);
    harness.run_steps(2);
    open_details(harness, "Spotkanie z Anią", "10-19");
    describe_anew(harness, second);
    let opened = editor(harness);
    assert_eq!(opened.opened.as_ref().unwrap().0.id, "ania");
    let expected = ("Fri 30 Oct 11:00".into(), "Fri 30 Oct 12:00".into());
    assert_eq!(editor_times(harness), expected);
}

#[test]
fn tc_10_29_several_standups_are_listed_and_picked_from_the_keyboard() {
    const ID: &str = "TC-10-29";
    let standup = |id: &str, day: &str| {
        single(
            id,
            "Standup",
            &format!("2026-10-{day}T09:30:00+01:00"),
            &format!("2026-10-{day}T09:45:00+01:00"),
        )
    };
    let mut world = world_of(
        "tc-10-29",
        vec![
            standup("tue", "27"),
            standup("wed", "28"),
            standup("thu", "29"),
            standup("past", "23"),
        ],
    );
    let description = "przesuń standup na 10";
    let answer = change("standup", "same", "", Some("10:00"));
    world.ollama.answer(description, &answer);
    until(&mut world.harness, |app| titled(app, "Standup").len() == 4);
    let before = google(&world).requests().len();

    step(ID, 1, "Press q, send the description.");
    describe_and_send(&mut world.harness, description);
    let harness = &mut world.harness;
    harness.run_steps(3);
    assert!(harness.state().dialog.is_none());
    let listed = matches(harness).expect("the list under the box");
    let ids: Vec<&str> = listed.events.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, ["tue", "wed", "thu"], "the 23 October one not listed");
    let app = harness.state();
    let rows: Vec<String> = (listed.events.iter())
        .map(|e| row(e, app.sync.calendars(), &app.regional, app.zone))
        .collect();
    for (row, day) in rows.iter().zip(["27", "28", "29"]) {
        assert!(row.starts_with("Standup · "), "{row}");
        assert!(row.contains(day), "{row}");
        assert!(row.ends_with(" · 09:30 · Work"), "{row}");
        assert!(shows(harness, row), "{row} is shown");
    }
    let field = description_box(harness).unwrap();
    assert!(field.rect().bottom() <= meet_with(harness).rect().top());
    let first = harness.get_by_label(rows[0].as_str()).rect();
    assert!(first.top() >= field.rect().bottom(), "under the box");

    step(ID, 2, "Press Esc.");
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);
    assert!(matches(harness).is_none(), "the list is gone");
    assert!(!shows(harness, &rows[0]));
    let field = description_box(harness).unwrap();
    assert_eq!(field.value().as_deref(), Some(description));
    assert!(field.is_focused());

    step(ID, 3, "Press Enter, then Down twice and Enter.");
    send(harness);
    harness.run_steps(3);
    assert_eq!(matches(harness).map(|m| m.events.len()), Some(3));
    for _ in 0..2 {
        harness.key_press(egui::Key::ArrowDown);
        harness.run_steps(2);
    }
    harness.key_press(egui::Key::Enter);
    harness.run_steps(3);
    let opened = editor(harness);
    assert_eq!(opened.opened.as_ref().unwrap().0.id, "thu");
    let expected = ("Thu 29 Oct 10:00".into(), "Thu 29 Oct 10:15".into());
    assert_eq!(editor_times(harness), expected);
    assert!(matches(harness).is_none());
    assert!(written(google(&world), before).is_empty());
}

#[test]
fn tc_10_30_saving_a_described_change_of_an_occurrence_asks_the_scope_and_about_the_guests() {
    const ID: &str = "TC-10-30";
    let mut world = world_of(
        "tc-10-30",
        vec![json!({
            "id": "sync", "status": "confirmed", "summary": "Weekly sync",
            "start": {"dateTime": "2026-10-07T14:00:00+02:00", "timeZone": "Europe/Warsaw"},
            "end": {"dateTime": "2026-10-07T15:00:00+02:00", "timeZone": "Europe/Warsaw"},
            "recurrence": ["RRULE:FREQ=WEEKLY;BYDAY=WE"],
            "organizer": {"email": EMAIL, "self": true},
            "attendees": [
                {"email": EMAIL, "self": true, "organizer": true, "responseStatus": "accepted"},
                {"email": "ania.nowak@example.com", "displayName": "Ania Nowak",
                 "responseStatus": "accepted"},
            ],
        })],
    );
    let description = "przesuń na 16";
    let answer = change("", "same", "", Some("16:00"));
    world.ollama.answer(description, &answer);
    let before = google(&world).requests().len();

    step(
        ID,
        1,
        "Open the details of the 28 October occurrence; press q, send the description.",
    );
    open_details(&mut world.harness, "Weekly sync", "10-28");
    let occurrence = details(&world.harness).unwrap();
    describe_and_send(&mut world.harness, description);
    let opened = editor(&world.harness);
    assert_eq!(opened.opened.as_ref().unwrap().0.id, occurrence.id);
    let expected = ("Wed 28 Oct 16:00".into(), "Wed 28 Oct 17:00".into());
    assert_eq!(editor_times(&world.harness), expected);

    step(ID, 2, "Save.");
    let harness = &mut world.harness;
    with_guests_day(harness);
    harness.get_by_label(texts::SAVE).click();
    harness.run_steps(3);
    assert!(matches!(harness.state().dialog, Some(Dialog::Scope(..))));
    for scope in [
        texts::THIS_EVENT,
        texts::THIS_AND_FOLLOWING,
        texts::ALL_EVENTS,
    ] {
        assert!(shows(harness, scope), "{scope}");
    }

    step(ID, 3, "Choose \"This event\".");
    harness.get_by_label(texts::THIS_EVENT).click();
    harness.run_steps(3);
    assert!(matches!(harness.state().dialog, Some(Dialog::Notify(..))));
    assert!(shows(harness, texts::SEND_UPDATE));

    step(ID, 4, "Choose not to notify.");
    harness.get_by_label(texts::DONT_SEND).click();
    harness.run_steps(2);
    let sent = writes(&mut world, before, 1);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(
        sent[0].contains(&format!("/events/{}", occurrence.id)),
        "{sent:?}"
    );
    assert!(sent[0].ends_with("sendUpdates=none"), "{sent:?}");
    let (_, patch) = google(&world).patched().last().cloned().unwrap();
    assert_eq!(patch["start"]["dateTime"], "2026-10-28T16:00:00+01:00");
    let series = google(&world).event(ACCOUNT_ID, EMAIL, "sync").unwrap();
    assert_eq!(series["start"]["dateTime"], "2026-10-07T14:00:00+02:00");
}

#[test]
fn tc_10_31_a_described_change_of_an_event_the_user_may_not_change_opens_it_locked() {
    const ID: &str = "TC-10-31";
    const HOLIDAYS: &str = "pl.polish#holiday@group.v.calendar.google.com";
    let dir = std::env::temp_dir().join(format!("calendar-buddy-tc-10-31-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut world = world_with(dir, true, false, |google| {
        google.add_event(
            ACCOUNT_ID,
            EMAIL,
            json!({
                "id": "review", "status": "confirmed", "summary": "Przegląd kwartalny",
                "start": {"dateTime": "2026-10-29T10:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "end": {"dateTime": "2026-10-29T11:00:00+01:00", "timeZone": "Europe/Warsaw"},
                "organizer": {"email": "boss@example.com"},
                "attendees": [
                    {"email": "boss@example.com", "organizer": true, "responseStatus": "accepted"},
                    {"email": EMAIL, "self": true, "responseStatus": "accepted"},
                ],
            }),
        );
        google.add_calendar(
            ACCOUNT_ID,
            json!({
                "id": HOLIDAYS, "summary": "Święta w Polsce", "timeZone": "Europe/Warsaw",
                "backgroundColor": "#16a765", "accessRole": "reader",
            }),
        );
        google.add_event(
            ACCOUNT_ID,
            HOLIDAYS,
            json!({
                "id": "saints", "status": "confirmed", "summary": "Wszystkich Świętych",
                "start": {"date": "2026-11-01"}, "end": {"date": "2026-11-02"},
            }),
        );
    });
    let description = "przesuń na 12";
    let answer = change("", "same", "", Some("12:00"));
    world.ollama.answer(description, &answer);
    let before = google(&world).requests().len();

    step(
        ID,
        1,
        "Open the details of \"Przegląd kwartalny\"; press q, send the description.",
    );
    open_details(&mut world.harness, "Przegląd kwartalny", "10-29");
    describe_and_send(&mut world.harness, description);
    let harness = &mut world.harness;
    harness.run_steps(3);
    let open = details(harness).expect("its details, as without a description");
    assert_eq!(open.id, "review");
    let expected = ("Thu 29 Oct 10:00".into(), "Thu 29 Oct 11:00".into());
    assert_eq!(shown_at(&open), expected, "the time locked");
    assert!(shows(harness, texts::ORGANIZER_ONLY));
    assert!(harness.query_by_label(texts::EDIT).is_none());

    step(
        ID,
        2,
        "Open the details of \"Wszystkich Świętych\"; send the description again.",
    );
    harness.key_press(egui::Key::Escape);
    harness.run_steps(2);
    open_details(harness, "Wszystkich Świętych", "11-01");
    describe_anew(harness, description);
    harness.run_steps(3);
    let open = details(harness).expect("its details, as a read-only event opens today");
    assert_eq!(open.id, "saints");
    assert!(shows(harness, texts::READ_ONLY_EVENT));
    assert!(harness.query_by_label(texts::EDIT).is_none());

    step(ID, 3, "Read the fake Google's record.");
    idle(&mut world.harness, 200);
    assert!(written(google(&world), before).is_empty());
    assert!(google(&world).patched().is_empty());
}

#[test]
fn a_change_sent_after_the_details_closed_is_not_of_the_event_they_showed() {
    let mut world = world_of(
        "closed-details",
        vec![single(
            "planning",
            "Sprint planning",
            "2026-10-26T10:00:00+01:00",
            "2026-10-26T11:00:00+01:00",
        )],
    );
    let text = "przesuń na 15";
    world
        .ollama
        .answer(text, &change("", "same", "", Some("15:00")));
    let harness = &mut world.harness;
    open_details(harness, "Sprint planning", "10-26");
    press(harness, 'q');
    until(harness, |_| true);
    harness.state_mut().dialog = None;
    harness.run_steps(2);
    description_box(harness).unwrap().type_text(text);
    harness.run_steps(2);
    send(harness);
    until(harness, |app| app.description.no_such_event);
    assert!(harness.state().dialog.is_none(), "no editing opens");
}

#[test]
fn an_event_of_a_hidden_calendar_is_not_found_by_name() {
    let mut world = world_of(
        "hidden-calendar",
        vec![single(
            "standup",
            "Standup",
            "2026-10-27T09:00:00+01:00",
            "2026-10-27T09:15:00+01:00",
        )],
    );
    let text = "przełóż standup na 10";
    world
        .ollama
        .answer(text, &change("standup", "same", "", Some("10:00")));
    let harness = &mut world.harness;
    until(harness, |app| on(app, "Standup", "10-27").is_some());
    let event = &harness.state().events[on(harness.state(), "Standup", "10-27").unwrap()];
    let key = (event.account_id.clone(), event.calendar_id.clone());
    harness.state_mut().settings.hidden.insert(key);
    harness.run_steps(2);
    describe_and_send(harness, text);
    until(harness, |app| app.description.no_such_event);
    assert!(harness.state().dialog.is_none(), "no editing opens");
}
