//! The test cases of `US-10-05` (`TC-10-32`…`TC-10-35`): a day or a time a description leaves out
//! is asked for, and a calendar it names is chosen, driving the whole app against a fake Ollama
//! and a fake Google (`DEC-65`), on the fixed clock of the reading tests. Each step is announced
//! with `step` before it is carried out. The fake Google's account is `owner@example.com`
//! (`user@example.com` in the test cases).

use calendar_core::model::Timing;
use calendar_google::fake::ACCOUNT_ID;
use chrono::NaiveTime;
use serde_json::Value;

use super::guests_tests::editor;
use super::*;

/// The fake model's answer: add an event, with the day's kind and value, the time, the guests and
/// the calendar named, as the model gives them.
fn answer(
    title: &str,
    (kind, day): (&str, &str),
    time: Option<&str>,
    guests: &[&str],
    calendar: &str,
) -> String {
    json!({
        "action": "add", "event": "", "title": title, "calendar": calendar, "kind": kind,
        "day": day, "time": time, "duration_min": null, "guests": guests, "meet": null,
    })
    .to_string()
}

fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
}

/// The empty start field saying `hint`, and whether it has the cursor.
fn asked_field_focused(harness: &Harness<'static, CalendarApp>, hint: &str) -> bool {
    let hint = hint.to_string();
    harness
        .get_by(move |node| node.placeholder() == Some(hint.as_str()))
        .is_focused()
}

/// Types `text` into the start field saying `hint`.
fn fill_in(harness: &mut Harness<'static, CalendarApp>, hint: &str, text: &str) {
    let hint = hint.to_string();
    harness
        .get_by(move |node| node.placeholder() == Some(hint.as_str()))
        .type_text(text);
    harness.run_steps(3);
}

fn save_enabled(harness: &Harness<'static, CalendarApp>) -> bool {
    !harness
        .get_by_label(texts::SAVE)
        .accesskit_node()
        .is_disabled()
}

/// The calendar picked in the "New event" window.
fn picked(entry: &QuickEntry) -> String {
    entry.calendar.as_ref().unwrap().name.clone()
}

#[test]
fn tc_10_32_no_time_opens_on_saturday_with_the_time_asked_for_and_save_waits_for_it() {
    const ID: &str = "TC-10-32";
    let mut world = world_with(
        std::env::temp_dir().join(format!("calendar-buddy-tc-10-32-{}", std::process::id())),
        true,
        false,
        |google| google.add_contact(ACCOUNT_ID, "Michał Wójcik", "michal.wojcik@example.com"),
    );
    let spotkanie = answer("Spotkanie", ("weekday", "sat"), None, &["Michał"], "");
    world.ollama.answer_all(&spotkanie);
    let description = "Spotkanie z Michałem w sobotę";

    step(ID, 1, "Press q, send the description.");
    describe_and_send(&mut world.harness, description);
    let harness = &mut world.harness;
    // The cursor is put in the field in the editor's first frames.
    harness.run_steps(2);
    assert!(
        new_event(harness).is_none(),
        "the full editor, a guest named"
    );
    let opened = editor(harness);
    let regional = harness.state().regional;
    assert_eq!(opened.title, "Spotkanie");
    assert_eq!(opened.start_date, regional.numeric_date(date(31)));
    assert_eq!(
        (opened.start_time.as_str(), opened.end_time.as_str()),
        ("", "")
    );
    let guests: Vec<&str> = (opened.guests.iter())
        .filter(|g| !g.own)
        .map(|g| g.name.as_str())
        .collect();
    assert_eq!(guests, ["Michał Wójcik"]);
    assert!(asked_field_focused(harness, texts::TIME_HINT));
    assert!(!save_enabled(harness));

    step(ID, 2, "Type 11:00.");
    let eleven = regional.time(NaiveTime::from_hms_opt(11, 0, 0).unwrap());
    fill_in(harness, texts::TIME_HINT, &eleven);
    let expected = ("Sat 31 Oct 11:00".into(), "Sat 31 Oct 11:30".into());
    assert_eq!(guests_tests::editor_times(harness), expected);
    assert!(save_enabled(harness));

    step(
        ID,
        3,
        "Send the description again; choose all-day instead of typing a time.",
    );
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);
    assert!(harness.state().dialog.is_none());
    describe_and_send(harness, description);
    harness.run_steps(2);
    assert_eq!(editor(harness).start_time, "");
    assert!(!save_enabled(harness));
    harness.get_by_label(texts::ALL_DAY).click();
    harness.run_steps(3);
    assert!(save_enabled(harness));
    assert_eq!(
        editor(harness).timing(&regional),
        Ok(Timing::AllDay {
            start: date(31),
            end_exclusive: NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
        })
    );
}

#[test]
fn tc_10_33_no_date_opens_at_13_with_the_day_asked_for_and_save_waits_for_it() {
    const ID: &str = "TC-10-33";
    let mut world = world("tc-10-33", true);
    world
        .ollama
        .answer_all(&answer("Lunch", ("none", ""), Some("13:00"), &[], ""));

    step(ID, 1, "Press q, send the description.");
    describe_and_send(&mut world.harness, "Lunch o 13");
    let harness = &mut world.harness;
    let entry = opened(harness);
    let regional = harness.state().regional;
    assert_eq!(entry.title, "Lunch");
    assert_eq!(picked(&entry), "Work");
    assert_eq!(entry.asked.as_ref().unwrap().day.as_deref(), Some(""));
    assert_eq!(entry.asked.as_ref().unwrap().time, None);
    let (thirteen, half_past) = (
        regional.time(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
        regional.time(NaiveTime::from_hms_opt(13, 30, 0).unwrap()),
    );
    assert!(shows(harness, &format!("{thirteen} – {half_past}")));
    assert!(asked_field_focused(harness, texts::DAY_HINT));
    assert!(!save_enabled(harness));

    step(ID, 2, "Choose Wednesday 28 October.");
    fill_in(harness, texts::DAY_HINT, &regional.numeric_date(date(28)));
    assert!(save_enabled(harness));

    step(ID, 3, "Save.");
    harness.get_by_label(texts::SAVE).click();
    harness.run_steps(2);
    until(harness, |app| {
        titled(app, "Lunch")
            .iter()
            .any(|e| e.id.starts_with("created-"))
    });
    let id = titled(harness.state(), "Lunch")[0].id.clone();
    let google = &world.google.as_ref().unwrap().fake;
    let sent: Value = google.event(ACCOUNT_ID, EMAIL, &id).unwrap();
    assert_eq!(sent["summary"], "Lunch");
    let at = |field: &str| {
        DateTime::parse_from_rfc3339(sent[field]["dateTime"].as_str().unwrap())
            .unwrap()
            .with_timezone(&Warsaw)
            .format("%a %d %b %H:%M")
            .to_string()
    };
    assert_eq!(
        (at("start"), at("end")),
        ("Wed 28 Oct 13:00".into(), "Wed 28 Oct 13:30".into())
    );
    assert_eq!(calendar_of(world.harness.state(), "Lunch"), "Work");
}

#[test]
fn tc_10_34_a_named_calendar_is_chosen_in_any_letter_case() {
    const ID: &str = "TC-10-34";
    let mut world = world_with(
        std::env::temp_dir().join(format!("calendar-buddy-tc-10-34-{}", std::process::id())),
        true,
        false,
        |google| {
            google.add_calendar(
                ACCOUNT_ID,
                json!({
                    "id": "praca@group.calendar.google.com", "summary": "Praca",
                    "timeZone": "Europe/Warsaw", "backgroundColor": "#7bd148",
                    "accessRole": "owner",
                }),
            )
        },
    );
    until(&mut world.harness, |app| {
        app.sync.calendars().iter().any(|c| c.name == "Praca")
    });
    let przeglad = "Przegląd auta w piątek o 9 w kalendarzu praca";
    let kino = "Kino jutro o 20 w kalendarzu MY CALENDAR";
    let on = ("weekday", "fri");
    (world.ollama).answer(
        przeglad,
        &answer("Przegląd auta", on, Some("09:00"), &[], "praca"),
    );
    let tomorrow = ("tomorrow", "");
    (world.ollama).answer(
        kino,
        &answer("Kino", tomorrow, Some("20:00"), &[], "MY CALENDAR"),
    );
    let harness = &mut world.harness;
    let note = |named: &str, harness: &Harness<'static, CalendarApp>| {
        shows(harness, &texts::calendar_not_found(named, "Work"))
    };

    step(ID, 1, "Send the first description.");
    describe_and_send(harness, przeglad);
    let entry = opened(harness);
    let expected = ("Fri 30 Oct 09:00".into(), "Fri 30 Oct 09:30".into());
    assert_eq!(times(&entry), expected);
    assert_eq!(picked(&entry), "Praca");
    assert_eq!(entry.not_found, None);
    assert!(!note("praca", harness));

    step(ID, 2, "Close it; send the second description.");
    close(harness);
    describe_and_send(harness, kino);
    let entry = opened(harness);
    assert_eq!(picked(&entry), "My calendar");
    assert_eq!(entry.not_found, None);
    assert!(!note("MY CALENDAR", harness));
}

#[test]
fn tc_10_35_a_calendar_not_found_or_read_only_gives_the_default_with_a_note() {
    const ID: &str = "TC-10-35";
    let mut world = world_with(
        std::env::temp_dir().join(format!("calendar-buddy-tc-10-35-{}", std::process::id())),
        true,
        false,
        |google| {
            google.add_calendar(
                ACCOUNT_ID,
                json!({
                    "id": "pl.polish#holiday@group.v.calendar.google.com",
                    "summary": "Święta w Polsce", "timeZone": "Europe/Warsaw",
                    "backgroundColor": "#16a765", "accessRole": "reader",
                }),
            )
        },
    );
    until(&mut world.harness, |app| {
        app.sync
            .calendars()
            .iter()
            .any(|c| c.name == "Święta w Polsce")
    });
    let on = ("weekday", "fri");
    let descriptions = [
        (
            "Przegląd auta w piątek o 9 w kalendarzu Prywatny",
            "Prywatny",
        ),
        (
            "Przegląd auta w piątek o 9 w kalendarzu Święta w Polsce",
            "Święta w Polsce",
        ),
    ];
    for (text, named) in descriptions {
        let przeglad = answer("Przegląd auta", on, Some("09:00"), &[], named);
        world.ollama.answer(text, &przeglad);
    }
    let harness = &mut world.harness;

    for (n, (text, named)) in descriptions.into_iter().enumerate() {
        if n == 0 {
            step(ID, 1, "Send the first description.");
        } else {
            step(ID, 2, "Close it; send the second description.");
            close(harness);
        }
        describe_and_send(harness, text);
        let entry = opened(harness);
        assert_eq!(picked(&entry), "Work");
        assert_eq!(entry.not_found.as_deref(), Some(named));
        let note = texts::calendar_not_found(named, "Work");
        assert_eq!(note, format!("Calendar \"{named}\" not found — using Work"));
        assert!(shows(harness, &note), "{note}");
    }
}
