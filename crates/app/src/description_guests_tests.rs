//! The test cases of `US-10-03` (`TC-10-20`…`TC-10-25`): guests and Google Meet named in a
//! description, driving the whole app against a fake Ollama and a fake Google (`DEC-65`), on the
//! fixed clock of the reading tests. Each step is announced with `step` before it is carried out.
//! The fake Google's account is `owner@example.com` (`user@example.com` and `user@firma.example`
//! in the test cases).

use calendar_core::availability;
use calendar_core::editor::Unresolved;
use calendar_core::guests::Person;
use calendar_core::local;
use calendar_core::model::Timing;
use calendar_google::fake::ACCOUNT_ID;
use serde_json::Value;

use super::*;

/// The fake model's answer: add an event with guests and Meet asked for, refused or not
/// mentioned (`None`).
fn with_guests(
    title: &str,
    kind: &str,
    day: &str,
    time: &str,
    guests: &[&str],
    meet: Option<bool>,
) -> String {
    json!({
        "action": "add", "title": title, "kind": kind, "day": day, "time": time,
        "duration_min": null, "guests": guests, "meet": meet,
    })
    .to_string()
}

/// The full editor, when it is open.
pub(super) fn editor(harness: &Harness<'static, CalendarApp>) -> Editor {
    match &harness.state().dialog {
        Some(Dialog::Editor(editor)) => (**editor).clone(),
        other => panic!("the full editor is not open: {:?}", other.is_some()),
    }
}

/// The editor's start and end on the wall clock in Warsaw.
pub(super) fn editor_times(harness: &Harness<'static, CalendarApp>) -> (String, String) {
    let editor = editor(harness);
    let Ok(Timing::Timed { start, end, .. }) = editor.timing(&harness.state().regional) else {
        panic!("a timed event");
    };
    let at = |t: DateTime<Utc>| {
        t.with_timezone(&Warsaw)
            .format("%a %d %b %H:%M")
            .to_string()
    };
    (at(start), at(end))
}

/// The addresses of the guests the editor lists.
fn invited(editor: &Editor) -> Vec<String> {
    (editor.guests.iter())
        .filter(|guest| !guest.own)
        .map(|guest| guest.email.clone())
        .collect()
}

fn calendar_name(editor: &Editor) -> String {
    editor.calendar.as_ref().unwrap().name.clone()
}

/// What the app asked Google to create, change or delete among events since `before`; and, below,
/// its searches for people.
pub(super) fn written(google: &FakeGoogle, before: usize) -> Vec<String> {
    (google.requests().into_iter().skip(before))
        .filter(|r| !r.starts_with("GET ") && r.contains("/events"))
        .collect()
}

fn searched(google: &FakeGoogle, before: usize) -> Vec<String> {
    (google.requests().into_iter().skip(before))
        .filter(|r| r.contains("/people/v1/"))
        .collect()
}

/// Esc in the editor, nothing saved.
fn close_editor(harness: &mut Harness<'static, CalendarApp>) {
    harness.key_press(egui::Key::Escape);
    harness.run_steps(3);
    assert!(harness.state().dialog.is_none(), "the editor closed");
}

/// The event Google created titled `title` in "Work", once the app has it back.
fn created(world: &mut World, title: &str) -> Value {
    until(&mut world.harness, |app| {
        titled(app, title)
            .iter()
            .any(|e| e.id.starts_with("created-"))
    });
    let id = titled(world.harness.state(), title)[0].id.clone();
    let google = world.google.as_ref().unwrap();
    google.fake.event(ACCOUNT_ID, EMAIL, &id).unwrap()
}

/// Waits until the guests' day Google gives is in the editor, which grows with it, so that a
/// click lands on the editor as it then is.
pub(super) fn with_guests_day(harness: &mut Harness<'static, CalendarApp>) {
    until(harness, |app| {
        matches!(&app.dialog, Some(Dialog::Editor(editor))
            if availability::asked(editor, &app.regional, true)
                .is_some_and(|(asked, _)| app.sync.busy(&asked).is_some()))
    });
}

fn person(name: &str, email: &str) -> Person {
    Person {
        name: name.to_string(),
        email: email.to_string(),
    }
}

#[test]
fn tc_10_20_two_guests_and_meet_open_the_full_editor_and_nobody_is_invited_before_save() {
    const ID: &str = "TC-10-20";
    let mut world = world_with(
        std::env::temp_dir().join(format!("calendar-buddy-tc-10-20-{}", std::process::id())),
        true,
        false,
        |google| {
            google.add_contact(ACCOUNT_ID, "Ania Nowak", "ania.nowak@example.com");
            google.add_contact(ACCOUNT_ID, "Tomek Wiśniewski", "tomek.w@example.com");
        },
    );
    let answer = with_guests(
        "Spotkanie",
        "tomorrow",
        "",
        "10:00",
        &["Ania", "Tomek"],
        Some(true),
    );
    world.ollama.answer_all(&answer);
    let before = world.google.as_ref().unwrap().fake.requests().len();

    step(ID, 1, "Press q, send the description.");
    let description = "wrzuć mi spotkanie z Anią i Tomkiem jutro o 10 z Meetem";
    describe_and_send(&mut world.harness, description);
    let harness = &world.harness;
    assert!(new_event(harness).is_none(), "not the \"New event\" window");
    let opened = editor(harness);
    assert_eq!(opened.title, "Spotkanie");
    let expected = ("Tue 27 Oct 10:00".into(), "Tue 27 Oct 10:30".into());
    assert_eq!(editor_times(harness), expected);
    assert_eq!(calendar_name(&opened), "Work");
    assert_eq!(
        invited(&opened),
        ["ania.nowak@example.com", "tomek.w@example.com"]
    );
    let names: Vec<&str> = opened.guests.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, ["Ania Nowak", "Tomek Wiśniewski"]);
    assert!(opened.meet);
    assert!(opened.unresolved.is_empty());

    step(ID, 2, "Read the fake Google's record.");
    let google = &world.google.as_ref().unwrap().fake;
    assert!(
        written(google, before).is_empty(),
        "nothing created or sent"
    );
    assert!(google.patched().is_empty());

    step(ID, 3, "Save.");
    with_guests_day(&mut world.harness);
    world.harness.get_by_label(texts::SAVE).click();
    world.harness.run_steps(2);
    let sent = created(&mut world, "Spotkanie");
    let attendees: Vec<&str> = (sent["attendees"].as_array().unwrap().iter())
        .filter(|a| a["self"] != true)
        .map(|a| a["email"].as_str().unwrap())
        .collect();
    assert_eq!(attendees, ["ania.nowak@example.com", "tomek.w@example.com"]);
    assert_eq!(
        sent["conferenceData"]["createRequest"]["conferenceSolutionKey"]["type"],
        "hangoutsMeet"
    );
    let google = &world.google.as_ref().unwrap().fake;
    let posts: Vec<String> = (written(google, before).into_iter())
        .filter(|r| r.starts_with("POST /calendar/v3/"))
        .collect();
    assert_eq!(posts.len(), 1, "{posts:?}");
    assert!(posts[0].contains("sendUpdates=all"), "{posts:?}");
    assert!(posts[0].contains("conferenceDataVersion=1"), "{posts:?}");
}

#[test]
fn tc_10_21_guests_are_found_in_contacts_directory_and_recent_addresses_as_when_typed() {
    const ID: &str = "TC-10-21";
    let mut world = world_with(
        std::env::temp_dir().join(format!("calendar-buddy-tc-10-21-{}", std::process::id())),
        true,
        true,
        |google| {
            google.add_contact(ACCOUNT_ID, "Ania Nowak", "ania.nowak@example.com");
            google.add_colleague(
                ACCOUNT_ID,
                "Marek Zieliński",
                "marek.zielinski@firma.example",
            );
            // Ola was a guest of an earlier event, and is in neither the contacts nor the
            // directory.
            google.add_event(
                ACCOUNT_ID,
                EMAIL,
                json!({
                    "id": "earlier", "status": "confirmed", "summary": "Planowanie",
                    "start": {"dateTime": "2026-10-20T10:00:00+02:00"},
                    "end": {"dateTime": "2026-10-20T11:00:00+02:00"},
                    "attendees": [
                        {"email": EMAIL, "self": true, "organizer": true, "responseStatus": "accepted"},
                        {"email": "ola.k@example.org", "displayName": "Ola Kowalska", "responseStatus": "accepted"}
                    ],
                }),
            );
        },
    );
    until(&mut world.harness, |app| {
        app.cached.iter().any(|e| e.title == "Planowanie")
    });
    let answer = with_guests(
        "Spotkanie",
        "tomorrow",
        "",
        "11:00",
        &["Ania", "Marek", "Ola"],
        None,
    );
    world.ollama.answer_all(&answer);

    step(ID, 1, "Press q, send the description.");
    describe_and_send(
        &mut world.harness,
        "Spotkanie z Anią, Markiem i Olą jutro o 11",
    );
    let opened = editor(&world.harness);
    let found = [
        person("Ania Nowak", "ania.nowak@example.com"),
        person("Marek Zieliński", "marek.zielinski@firma.example"),
        person("Ola Kowalska", "ola.k@example.org"),
    ];
    let guests: Vec<Person> = (opened.guests.iter())
        .map(|g| person(&g.name, &g.email))
        .collect();
    assert_eq!(guests, found);
    assert!(opened.unresolved.is_empty());
    close_editor(&mut world.harness);

    step(
        ID,
        2,
        "In a new event's editor, type \"Marek\" and then \"Ola\" in the guest field.",
    );
    let harness = &mut world.harness;
    press(harness, 'c');
    until(harness, |app| {
        matches!(app.dialog, Some(Dialog::QuickEntry(..)))
    });
    harness.get_by_label(texts::MORE_OPTIONS).click();
    until(harness, |app| matches!(app.dialog, Some(Dialog::Editor(_))));
    harness
        .get_by(|n| n.placeholder() == Some(texts::ADD_GUESTS))
        .focus();
    harness.run_steps(2);
    for (typed, person) in [("Marek", &found[1]), ("Ola", &found[2])] {
        // What was typed before is replaced, the cursor still in the field.
        harness.key_combination_modifiers(egui::Modifiers::COMMAND, &[egui::Key::A]);
        harness.event(egui::Event::Text(typed.to_string()));
        harness.run_steps(2);
        assert_eq!(editor(harness).guest_field, typed);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !shows(harness, &person.label()) {
            assert!(Instant::now() < deadline, "{typed}: never suggested");
            std::thread::sleep(Duration::from_millis(10));
            harness.step();
        }
        let suggested: Vec<String> = harness
            .query_all_by_role(egui::accesskit::Role::Button)
            .filter_map(|n| n.accesskit_node().label())
            .filter(|label| label.contains('@'))
            .collect();
        assert_eq!(suggested, [person.label()], "{typed}");
    }
}

#[test]
fn tc_10_22_names_not_certain_stay_with_suggestions_and_nobody_is_a_guest_until_picked() {
    const ID: &str = "TC-10-22";
    let mut world = world_with(
        std::env::temp_dir().join(format!("calendar-buddy-tc-10-22-{}", std::process::id())),
        true,
        false,
        |google| {
            google.add_contact(ACCOUNT_ID, "Kasia Nowak", "kasia.nowak@example.com");
            google.add_contact(ACCOUNT_ID, "Kasia Lis", "kasia.lis@example.com");
        },
    );
    let answer = with_guests(
        "Lunch",
        "weekday",
        "fri",
        "13:00",
        &["Kasia", "Zenek"],
        None,
    );
    world.ollama.answer_all(&answer);
    let nowak = person("Kasia Nowak", "kasia.nowak@example.com");
    let lis = person("Kasia Lis", "kasia.lis@example.com");

    step(ID, 1, "Press q, send the description.");
    describe_and_send(&mut world.harness, "lunch z Kasią i Zenkiem w piątek o 13");
    let harness = &mut world.harness;
    let opened = editor(harness);
    assert_eq!(editor_times(harness).0, "Fri 30 Oct 13:00");
    assert!(invited(&opened).is_empty(), "no guests yet");
    assert_eq!(
        opened.unresolved,
        [
            Unresolved {
                name: "Kasia".into(),
                suggested: vec![nowak.clone(), lis.clone()],
            },
            Unresolved {
                name: "Zenek".into(),
                suggested: Vec::new(),
            },
        ]
    );
    for shown in [
        "Kasia",
        "Zenek",
        &nowak.label(),
        &lis.label(),
        texts::NOBODY_FITS,
    ] {
        assert!(shows(harness, shown), "{shown}");
    }

    step(ID, 2, "Pick Kasia Lis.");
    harness.get_by_label(&lis.label()).click();
    harness.run_steps(3);
    let picked = editor(harness);
    assert_eq!(invited(&picked), ["kasia.lis@example.com"]);
    let waiting: Vec<&str> = picked.unresolved.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(waiting, ["Zenek"]);
    assert!(!shows(harness, &nowak.label()));

    step(ID, 3, "Remove \"Zenek\" and Save.");
    harness.get_by_label(&texts::drop_name("Zenek")).click();
    harness.run_steps(3);
    assert!(editor(harness).unresolved.is_empty());
    with_guests_day(harness);
    harness.get_by_label(texts::SAVE).click();
    harness.run_steps(2);
    let sent = created(&mut world, "Lunch");
    let attendees: Vec<&str> = (sent["attendees"].as_array().unwrap().iter())
        .filter(|a| a["self"] != true)
        .map(|a| a["email"].as_str().unwrap())
        .collect();
    assert_eq!(attendees, ["kasia.lis@example.com"]);
}

/// Meet refused in the description stays off when guests are added later, picked for a waiting
/// name or typed in the guest field, and a name still waiting at Save invites nobody
/// (`MOD-10-F07`, `MOD-10-F12`, `DEC-227`).
#[test]
fn a_refused_meet_stays_off_when_guests_are_added_and_a_waiting_name_invites_nobody() {
    let mut world = world_with(
        std::env::temp_dir().join(format!(
            "calendar-buddy-refused-meet-{}",
            std::process::id()
        )),
        true,
        false,
        |google| {
            google.add_contact(ACCOUNT_ID, "Kasia Nowak", "kasia.nowak@example.com");
            google.add_contact(ACCOUNT_ID, "Kasia Lis", "kasia.lis@example.com");
        },
    );
    let answer = with_guests(
        "Lunch",
        "tomorrow",
        "",
        "13:00",
        &["Kasia", "Zenek", "mama"],
        Some(false),
    );
    world.ollama.answer_all(&answer);
    describe_and_send(
        &mut world.harness,
        "Lunch z Kasią, Zenkiem i mamą jutro o 13 bez Meeta",
    );
    let harness = &mut world.harness;
    assert!(!editor(harness).meet);

    // Kasia Lis picked for "Kasia".
    let lis = person("Kasia Lis", "kasia.lis@example.com");
    harness.get_by_label(&lis.label()).click();
    harness.run_steps(3);
    let picked = editor(harness);
    assert_eq!(invited(&picked), ["kasia.lis@example.com"]);
    assert!(!picked.meet, "Meet was refused");

    // Zenek's address typed in the guest field and Enter; "Zenek" dropped, "mama" still waits.
    harness.get_by_label(&texts::drop_name("Zenek")).click();
    harness.run_steps(3);
    harness
        .get_by(|n| n.placeholder() == Some(texts::ADD_GUESTS))
        .focus();
    harness.run_steps(2);
    harness.event(egui::Event::Text("zenek@example.com".to_string()));
    harness.run_steps(2);
    harness.key_press(egui::Key::Enter);
    harness.run_steps(3);
    let typed = editor(harness);
    assert_eq!(
        invited(&typed),
        ["kasia.lis@example.com", "zenek@example.com"]
    );
    assert!(!typed.meet, "Meet was refused");
    let waiting: Vec<&str> = typed.unresolved.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(waiting, ["mama"]);

    with_guests_day(harness);
    harness.get_by_label(texts::SAVE).click();
    harness.run_steps(2);
    let sent = created(&mut world, "Lunch");
    let attendees: Vec<&str> = (sent["attendees"].as_array().unwrap().iter())
        .filter(|a| a["self"] != true)
        .map(|a| a["email"].as_str().unwrap())
        .collect();
    assert_eq!(attendees, ["kasia.lis@example.com", "zenek@example.com"]);
    assert!(sent["conferenceData"].is_null(), "{sent}");
}

#[test]
fn tc_10_23_meet_follows_the_description_and_turns_on_with_a_guest_when_it_says_nothing() {
    const ID: &str = "TC-10-23";
    let mut world = world_with(
        std::env::temp_dir().join(format!("calendar-buddy-tc-10-23-{}", std::process::id())),
        true,
        false,
        |google| google.add_contact(ACCOUNT_ID, "Ania Nowak", "ania.nowak@example.com"),
    );

    step(
        ID,
        1,
        "Send each description in turn, closing the editor without saving after each.",
    );
    // The description, the title, the guests and Meet the model reads, and whether Meet is on.
    type Row<'a> = (&'a str, &'a str, &'a [&'a str], Option<bool>, bool);
    let rows: [Row; 4] = [
        (
            "Rozmowa jutro o 12 z Meetem",
            "Rozmowa",
            &[],
            Some(true),
            true,
        ),
        (
            "Call tomorrow at 12 with Meet",
            "Call",
            &[],
            Some(true),
            true,
        ),
        (
            "Spotkanie z Anią jutro o 12 bez Meeta",
            "Spotkanie",
            &["Ania"],
            Some(false),
            false,
        ),
        (
            "Spotkanie z Anią jutro o 12",
            "Spotkanie",
            &["Ania"],
            None,
            true,
        ),
    ];
    for (description, title, guests, meet, meet_on) in rows {
        let answer = with_guests(title, "tomorrow", "", "12:00", guests, meet);
        world.ollama.answer_all(&answer);
        describe_and_send(&mut world.harness, description);
        let opened = editor(&world.harness);
        let expected: Vec<&str> = if guests.is_empty() {
            vec![]
        } else {
            vec!["ania.nowak@example.com"]
        };
        assert_eq!(invited(&opened), expected, "{description}");
        assert_eq!(opened.meet, meet_on, "{description}");
        assert!(
            (world.harness.query_all_by_label(texts::ADD_GOOGLE_MEET)).any(|n| {
                n.accesskit_node().toggled()
                    == Some(if meet_on {
                        egui::accesskit::Toggled::True
                    } else {
                        egui::accesskit::Toggled::False
                    })
            }),
            "{description}: the switch shows it"
        );
        close_editor(&mut world.harness);
    }
}

#[test]
fn tc_10_24_a_local_calendar_takes_the_event_without_guests_or_meet() {
    const ID: &str = "TC-10-24";
    let mut world = world_with(
        std::env::temp_dir().join(format!("calendar-buddy-tc-10-24-{}", std::process::id())),
        true,
        false,
        |google| google.add_contact(ACCOUNT_ID, "Ania Nowak", "ania.nowak@example.com"),
    );
    let mine = (world.harness.state().sync.calendars().iter())
        .find(|c| c.name == "My calendar")
        .unwrap()
        .id
        .clone();
    world.harness.state_mut().settings.default_calendar = Some((local::ACCOUNT_ID.into(), mine));
    let answer = with_guests("Spotkanie", "tomorrow", "", "10:00", &["Ania"], Some(true));
    world.ollama.answer_all(&answer);
    let before = world.google.as_ref().unwrap().fake.requests().len();

    step(ID, 1, "Press q, send the description.");
    describe_and_send(&mut world.harness, "Spotkanie z Anią jutro o 10 z Meetem");
    let harness = &world.harness;
    let opened = editor(harness);
    assert_eq!(calendar_name(&opened), "My calendar");
    assert!(!opened.google());
    assert_eq!(opened.title, "Spotkanie");
    assert_eq!(editor_times(harness).0, "Tue 27 Oct 10:00");
    assert!(opened.guests.is_empty() && opened.unresolved.is_empty());
    assert!(!opened.meet);
    assert!(!shows(harness, texts::ADD_GOOGLE_MEET));
    let guest_field = harness.query_by(|n| n.placeholder() == Some(texts::ADD_GUESTS));
    assert!(guest_field.is_none(), "no guest field");

    step(ID, 2, "Save; read the fake Google's record.");
    save(&mut world.harness, "Spotkanie");
    let app = world.harness.state();
    assert_eq!(calendar_of(app, "Spotkanie"), "My calendar");
    assert!(titled(app, "Spotkanie")[0].about.guests.is_empty());
    let google = &world.google.as_ref().unwrap().fake;
    assert!(written(google, before).is_empty(), "nothing sent to Google");
    assert!(searched(google, before).is_empty(), "nobody looked up");
}

#[test]
fn tc_10_25_offline_or_without_an_account_no_guest_is_looked_up() {
    const ID: &str = "TC-10-25";
    let mut case_a = world_with(
        std::env::temp_dir().join(format!("calendar-buddy-tc-10-25-a-{}", std::process::id())),
        true,
        false,
        |google| google.add_contact(ACCOUNT_ID, "Ania Nowak", "ania.nowak@example.com"),
    );
    let answer = with_guests("Spotkanie", "tomorrow", "", "10:00", &["Ania"], Some(true));
    case_a.ollama.answer_all(&answer);
    case_a.network.set(false);
    let google = case_a.google.as_ref().unwrap();
    google.fake.disconnect();
    let before = google.fake.requests().len();

    step(
        ID,
        1,
        "Case A: send the description; read the fake Google's record.",
    );
    describe_and_send(&mut case_a.harness, "Spotkanie z Anią jutro o 10 z Meetem");
    let harness = &case_a.harness;
    let opened = editor(harness);
    let work = opened.calendar.clone().unwrap();
    assert_eq!(work.name, "Work");
    let refused = offline::refused_in(false, Some(&work)).expect("offline, Work takes nothing");
    assert!(shows(harness, refused));
    assert!(opened.guests.is_empty() && opened.unresolved.is_empty());
    assert!(!shows(harness, "Ania"));
    let google = &case_a.google.as_ref().unwrap().fake;
    assert!(
        searched(google, before).is_empty(),
        "no contacts or directory search"
    );
    drop(case_a);

    step(ID, 2, "Case B: send the description.");
    let mut case_b = world("tc-10-25-b", false);
    case_b.ollama.answer_all(&answer);
    describe_and_send(&mut case_b.harness, "Spotkanie z Anią jutro o 10 z Meetem");
    let opened = editor(&case_b.harness);
    assert_eq!(calendar_name(&opened), "My calendar");
    assert!(!opened.google());
    assert!(opened.guests.is_empty() && opened.unresolved.is_empty());
    assert!(!opened.meet);
}
