use chrono::TimeZone;
use chrono_tz::Europe::Warsaw;

use super::*;
use crate::ollama::fake::FakeOllama;
use crate::ollama::{self, DEFAULT_MODEL};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Monday 26 October 2026, 09:00 in Warsaw: the clock of the test set and the test cases.
fn now() -> DateTime<Tz> {
    Warsaw.with_ymd_and_hms(2026, 10, 26, 9, 0, 0).unwrap()
}

fn answer(kind: &str, day: &str, time: Value, length: Value) -> String {
    json!({
        "action": "add", "title": "Kino", "kind": kind, "day": day, "time": time,
        "duration_min": length, "guests": [], "meet": false,
    })
    .to_string()
}

fn event(answer: &str) -> Described {
    match read(answer, now().date_naive()) {
        Reading::Event(event) => event,
        _ => panic!("no event in {answer}"),
    }
}

#[test]
fn a_description_is_cut_at_500_letters_not_bytes() {
    let long = "ż".repeat(600);
    assert_eq!(cut(&long).chars().count(), 500);
    assert_eq!(cut("Kino jutro"), "Kino jutro");
}

#[test]
fn the_message_gives_the_day_and_time_then_the_description() {
    let message = message("Kino jutro o 20", now());
    assert_eq!(
        message,
        "Now is Monday 2026-10-26 09:00, time zone Europe/Warsaw.\n\nKino jutro o 20"
    );
    assert_eq!(description_in(&message), Some("Kino jutro o 20"));
}

#[test]
fn the_request_has_the_fixed_form_no_thinking_temperature_0_and_keeps_the_model_5_minutes() {
    let request = request(&Ollama::default(), "Kino", now());
    assert_eq!(request["model"], DEFAULT_MODEL);
    assert_eq!(request["stream"], false);
    assert_eq!(request["think"], false);
    assert_eq!(request["keep_alive"], "5m");
    assert_eq!(request["options"]["temperature"], 0);
    assert_eq!(
        request["format"]["properties"]["kind"]["enum"][0],
        "yesterday"
    );
    // The rules are the same for every description, so Ollama can reuse them.
    let other = super::request(&Ollama::default(), "Kawa", now() + Duration::hours(3));
    assert_eq!(request["messages"][0], other["messages"][0]);
}

#[test]
fn the_app_works_out_every_kind_of_day_from_today() {
    let none = Value::Null;
    let on = |kind, day| event(&answer(kind, day, json!("10:00"), none.clone())).date;
    assert_eq!(on("yesterday", ""), d(2026, 10, 25));
    assert_eq!(on("today", ""), d(2026, 10, 26));
    assert_eq!(on("tomorrow", ""), d(2026, 10, 27));
    assert_eq!(on("day_after_tomorrow", ""), d(2026, 10, 28));
    assert_eq!(on("weekday", "wed"), d(2026, 10, 28));
    // A weekday alone is the next one: on Monday, "on Monday" is next Monday.
    assert_eq!(on("weekday", "mon"), d(2026, 11, 2));
    assert_eq!(on("weekday", "fri"), d(2026, 10, 30));
    assert_eq!(on("next_weekday", "thu"), d(2026, 11, 5));
    assert_eq!(on("next_weekday", "mon"), d(2026, 11, 2));
    assert_eq!(on("in_days", "7"), d(2026, 11, 2));
    assert_eq!(on("date", "11-14"), d(2026, 11, 14));
    // A date passed this year is next year's.
    assert_eq!(on("date", "01-05"), d(2027, 1, 5));
}

#[test]
fn a_part_of_the_day_is_9_14_or_18() {
    let at = |time: &str| event(&answer("today", "", json!(time), Value::Null)).time;
    let hm = |h, m| NaiveTime::from_hms_opt(h, m, 0);
    assert_eq!(at("morning"), hm(9, 0));
    assert_eq!(at("afternoon"), hm(14, 0));
    assert_eq!(at("evening"), hm(18, 0));
    assert_eq!(at("Evening"), hm(18, 0));
    assert_eq!(at("8:15"), hm(8, 15));
    assert_eq!(
        event(&answer("today", "", Value::Null, Value::Null)).time,
        None
    );
}

#[test]
fn the_times_use_the_length_given_or_the_default_one_across_the_end_of_summer_time() {
    let given = event(&answer("yesterday", "", json!("12:00"), json!(45)));
    let (start, end) = given.times(30, now().to_utc(), Warsaw);
    // Sunday 25 October is the day summer time ends in Warsaw.
    assert_eq!(
        start,
        Warsaw.with_ymd_and_hms(2026, 10, 25, 12, 0, 0).unwrap()
    );
    assert_eq!(end - start, Duration::minutes(45));
    let default = event(&answer("tomorrow", "", json!("10:00"), Value::Null));
    let (start, end) = default.times(30, now().to_utc(), Warsaw);
    assert_eq!(
        start,
        Warsaw.with_ymd_and_hms(2026, 10, 27, 10, 0, 0).unwrap()
    );
    assert_eq!(end - start, Duration::minutes(30));
}

#[test]
fn an_answer_not_in_the_fixed_form_is_no_event() {
    let today = now().date_naive();
    let not_an_event = r#"{"action":"none","title":"","kind":"today","day":"","time":null,"duration_min":null,"guests":[],"meet":false}"#;
    assert_eq!(read(not_an_event, today), Reading::NoEvent);
    for unusable in [
        "Kino jutro o 20",
        "",
        r#"{"title":"Kino"}"#,
        &answer("someday", "", json!("10:00"), Value::Null),
        &answer("weekday", "friday", json!("10:00"), Value::Null),
        &answer("date", "13-40", json!("10:00"), Value::Null),
        &answer("today", "", json!("noon"), Value::Null),
        &answer("today", "", json!("25:00"), Value::Null),
        &answer("today", "", json!("10:00"), Value::Null).replace("Kino", " "),
    ] {
        assert_eq!(read(unusable, today), Reading::NoEvent, "{unusable}");
    }
}

/// A sentence of the test set, with the event expected, or for a change the event named and what
/// changes, and the answer `gemma4:e4b` gave it (variant D, `DEC-218`; the changing sentences
/// with the rules of `US-10-04`), which the fake model answers with.
struct Case {
    lang: String,
    /// Whether the sentence changes an event rather than adding one.
    change: bool,
    text: String,
    /// The new date of a change; `None` when the day stays.
    date: Option<String>,
    time: Option<String>,
    /// The length of an event added; the new length of a change, `None` when it stays.
    length: Option<u32>,
    guests: Vec<String>,
    meet: bool,
    /// Words one of which the title holds; for a change, the words naming the event, none when
    /// it names none.
    title: Vec<String>,
    answer: String,
}

/// The test set (`MOD-10-NFR-04`): the adding sentences moved from the test of 2026-10-27, and
/// the changing ones of `US-10-04` (`DEC-221`).
fn cases() -> Vec<Case> {
    let cases: Value = serde_json::from_str(include_str!("described_cases.json")).unwrap();
    let strings = |value: &Value| -> Vec<String> {
        (value.as_array().into_iter().flatten())
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    };
    (cases.as_array().unwrap().iter())
        .map(|case| {
            let change = case["action"] == "change";
            Case {
                lang: case["lang"].as_str().unwrap().to_string(),
                change,
                text: case["text"].as_str().unwrap().to_string(),
                date: case["date"].as_str().map(str::to_string),
                time: case["time"].as_str().map(str::to_string),
                length: case["duration_min"].as_u64().map(|m| m as u32),
                guests: strings(&case["guests"]),
                meet: case["meet"].as_bool().unwrap_or(false),
                title: strings(if change {
                    &case["event"]
                } else {
                    &case["title"]
                }),
                answer: case["answer"].as_str().unwrap().to_string(),
            }
        })
        .collect()
}

const FIELDS: [&str; 6] = ["date", "time", "length", "guests", "meet", "title"];
const CHANGE_FIELDS: [&str; 4] = ["date", "time", "length", "event"];

/// What a sentence's reading got right, field by field, as the test of 2026-10-27 scored it; a
/// change scores the new day, hour and length and the event it names, and a reading of the
/// other kind gets nothing right. The date comes first.
fn score(case: &Case, reading: &Reading) -> Vec<(&'static str, bool)> {
    let hhmm = |time: Option<NaiveTime>| time.map(|t| t.format("%H:%M").to_string());
    match (case.change, reading) {
        (false, Reading::Event(event)) => {
            let folded = |names: &[String]| {
                let mut names: Vec<String> =
                    names.iter().map(|n| n.trim().to_lowercase()).collect();
                names.sort();
                names
            };
            let title = event.title.to_lowercase();
            vec![
                ("date", Some(event.date.to_string()) == case.date),
                ("time", hhmm(event.time) == case.time),
                (
                    "length",
                    event.length.unwrap_or(create::DEFAULT_LENGTH) == case.length.unwrap(),
                ),
                ("guests", folded(&event.guests) == folded(&case.guests)),
                // Meet on: asked for; refused and not mentioned both leave it off here (`DEC-223`).
                ("meet", (event.meet == Some(true)) == case.meet),
                ("title", case.title.iter().any(|word| title.contains(word))),
            ]
        }
        (true, Reading::Change(change)) => {
            let named = change.event.to_lowercase();
            vec![
                ("date", change.date.map(|d| d.to_string()) == case.date),
                ("time", hhmm(change.time) == case.time),
                ("length", change.length == case.length),
                (
                    "event",
                    match case.title.as_slice() {
                        [] => named.is_empty(),
                        words => words.iter().any(|word| named.contains(word)),
                    },
                ),
            ]
        }
        (change, _) => (if change {
            &CHANGE_FIELDS[..]
        } else {
            &FIELDS[..]
        })
        .iter()
        .map(|field| (*field, false))
        .collect(),
    }
}

/// Sends every sentence of the set to Ollama as the app does, reads each answer as the app does,
/// prints what each got right and gives the shares with the date right and with every field right.
fn run(ollama: &Ollama) -> (f64, f64) {
    let cases = cases();
    let (mut dates, mut all) = (0, 0);
    for case in &cases {
        let answer = ollama::chat(ollama, &request(ollama, &case.text, now())).unwrap_or_default();
        let score = score(case, &read(&answer, now().date_naive()));
        let wrong: Vec<&str> = (score.iter().filter(|(_, right)| !right))
            .map(|(field, _)| *field)
            .collect();
        dates += usize::from(score[0].1);
        all += usize::from(wrong.is_empty());
        let verdict = if wrong.is_empty() { "right" } else { "wrong" };
        println!("{verdict} {} {:?} {}", case.lang, wrong, case.text);
    }
    let share = |n: usize| 100.0 * n as f64 / cases.len() as f64;
    println!(
        "date right {:.0} %, every field right {:.0} %",
        share(dates),
        share(all)
    );
    (share(dates), share(all))
}

#[test]
fn tc_10_18_the_test_set_passes_its_thresholds_against_the_fake_model() {
    const ID: &str = "TC-10-18";
    println!("og-step {ID} 1 — Count the sentences of the set.");
    let cases = cases();
    let count = |change: bool, lang: &str| {
        (cases.iter())
            .filter(|c| c.change == change && c.lang == lang)
            .count()
    };
    let (adding, changing) = (cases.iter()).partition::<Vec<&Case>, _>(|c| !c.change);
    println!(
        "adding sentences: {} pl, {} en; changing: {} pl, {} en",
        count(false, "pl"),
        count(false, "en"),
        count(true, "pl"),
        count(true, "en"),
    );
    assert!(adding.len() >= 30);
    assert!(changing.len() >= 10);
    for change in [false, true] {
        assert!(count(change, "pl") > 0 && count(change, "en") > 0);
    }

    println!("og-step {ID} 2 — Run the set against the fake model.");
    let fake = FakeOllama::start(&[DEFAULT_MODEL]);
    for case in &cases {
        fake.answer(&case.text, &case.answer);
    }
    let settings = Ollama {
        address: fake.address.clone(),
        ..Ollama::default()
    };
    let (dates, all) = run(&settings);
    assert!(dates >= 95.0, "date right in {dates:.0} %");
    assert!(all >= 80.0, "every field right in {all:.0} %");
    assert_eq!(fake.chats().len(), cases.len());
}

/// `TC-10-19`, run by hand against a live Ollama with `gemma4:e4b` pulled:
/// `cargo test -p calendar-core the_test_set_against_a_live_ollama -- --ignored --nocapture`.
#[test]
#[ignore = "needs a live Ollama with gemma4:e4b; run by hand (TC-10-19)"]
fn the_test_set_against_a_live_ollama() {
    let (dates, all) = run(&Ollama::default());
    assert!(dates >= 95.0, "date right in {dates:.0} %");
    assert!(all >= 80.0, "every field right in {all:.0} %");
}

/// `US-10-03`: what the editor gets from a description with guests and Meet.
mod guests_and_meet {
    use super::*;
    use crate::create::QuickEntry;
    use crate::editor::{Editor, Unresolved};
    use crate::guests::Person;
    use crate::local;
    use crate::model::{Access, Calendar, Rgb};
    use crate::regional::Regional;
    use pure_rust_locales::Locale;

    fn person(name: &str, email: &str) -> Person {
        Person {
            name: name.to_string(),
            email: email.to_string(),
        }
    }

    fn described(guests: &[&str], meet: Option<bool>) -> Described {
        Described {
            title: "Spotkanie".to_string(),
            date: d(2026, 10, 27),
            time: NaiveTime::from_hms_opt(10, 0, 0),
            length: None,
            guests: guests.iter().map(|g| g.to_string()).collect(),
            meet,
        }
    }

    fn calendar(account_id: &str) -> Calendar {
        Calendar {
            id: "owner@example.com".to_string(),
            account_id: account_id.to_string(),
            name: "Work".to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
            default_reminders: Vec::new(),
        }
    }

    fn editor(account_id: &str) -> Editor {
        let start = Warsaw
            .with_ymd_and_hms(2026, 10, 27, 10, 0, 0)
            .unwrap()
            .to_utc();
        let entry = QuickEntry::new(
            (start, start + Duration::minutes(30)),
            Warsaw,
            Some(calendar(account_id)),
        );
        Editor::from_quick(&entry, &Regional::new(Locale::pl_PL))
    }

    fn people() -> Vec<Person> {
        vec![
            person("Ania Nowak", "ania.nowak@example.com"),
            person("Tomek Wiśniewski", "tomek.w@example.com"),
            person("Kasia Nowak", "kasia.nowak@example.com"),
            person("Kasia Lis", "kasia.lis@example.com"),
            person("", "owner@example.com"),
        ]
    }

    /// Each name with everyone found, as Google and the recent guests give them.
    fn found(names: &[&str]) -> Vec<(String, Vec<Person>)> {
        names.iter().map(|n| (n.to_string(), people())).collect()
    }

    #[test]
    fn meet_is_asked_for_refused_or_not_mentioned() {
        let with = |meet: Value| {
            let answer = answer("tomorrow", "", json!("10:00"), Value::Null)
                .replace(r#""meet":false"#, &format!(r#""meet":{meet}"#));
            event(&answer).meet
        };
        assert_eq!(with(json!(true)), Some(true));
        assert_eq!(with(json!(false)), Some(false));
        assert_eq!(with(Value::Null), None);
        let schema = &request(&Ollama::default(), "Kino", now())["format"];
        assert_eq!(
            schema["properties"]["meet"]["type"],
            json!(["boolean", "null"])
        );
    }

    #[test]
    fn guests_or_meet_asked_for_open_the_full_editor_and_neither_the_new_event_window() {
        assert!(described(&["Ania"], None).full());
        assert!(described(&[], Some(true)).full());
        assert!(described(&["Ania"], Some(false)).full());
        assert!(!described(&[], None).full());
        // Refusing Meet asks for nothing the "New event" window lacks.
        assert!(!described(&[], Some(false)).full());
    }

    #[test]
    fn a_name_that_fits_one_person_invites_them_and_the_first_guest_turns_meet_on() {
        let mut editor = editor("1");
        let event = described(&["Ania", "Tomek"], None);
        fill(&mut editor, &event, Some(&found(&["Ania", "Tomek"])), &[]);
        let invited: Vec<&str> = editor.guests.iter().map(|g| g.email.as_str()).collect();
        assert_eq!(invited, ["ania.nowak@example.com", "tomek.w@example.com"]);
        assert!(editor.unresolved.is_empty());
        assert!(editor.meet);
    }

    #[test]
    fn a_name_that_fits_several_or_nobody_waits_with_its_suggestions_and_invites_nobody() {
        let mut editor = editor("1");
        let event = described(&["Kasia", "Zenek", "mama"], None);
        fill(
            &mut editor,
            &event,
            Some(&found(&["Kasia", "Zenek", "mama"])),
            &[],
        );
        assert!(editor.guests.is_empty());
        assert!(!editor.meet);
        assert_eq!(
            editor.unresolved,
            [
                Unresolved {
                    name: "Kasia".to_string(),
                    suggested: vec![
                        person("Kasia Nowak", "kasia.nowak@example.com"),
                        person("Kasia Lis", "kasia.lis@example.com"),
                    ],
                },
                Unresolved {
                    name: "Zenek".to_string(),
                    suggested: Vec::new(),
                },
                Unresolved {
                    name: "mama".to_string(),
                    suggested: Vec::new(),
                },
            ]
        );
        // Picking one invites them; dropping the other invites nobody.
        let lis = editor.unresolved[0].suggested[1].clone();
        editor.pick(0, &lis, &[]);
        let invited: Vec<&str> = editor.guests.iter().map(|g| g.email.as_str()).collect();
        assert_eq!(invited, ["kasia.lis@example.com"]);
        assert!(
            editor.meet,
            "the first guest turns Meet on, as in the editor"
        );
        assert_eq!(editor.unresolved.len(), 2);
        editor.drop_unresolved(0);
        assert_eq!(editor.unresolved[0].name, "mama");
    }

    #[test]
    fn guests_added_later_leave_meet_as_the_description_said() {
        // "Lunch z Kasią i Zenkiem jutro o 13 bez Meeta": two people named Kasia, nobody Zenek.
        for (said, on) in [(Some(false), false), (Some(true), true), (None, true)] {
            // Kasia Lis picked for "Kasia".
            let mut picked = editor("1");
            fill(
                &mut picked,
                &described(&["Kasia", "Zenek"], said),
                Some(&found(&["Kasia", "Zenek"])),
                &[],
            );
            let lis = picked.unresolved[0].suggested[1].clone();
            picked.pick(0, &lis, &[]);
            assert_eq!(picked.guests[0].email, "kasia.lis@example.com");
            assert_eq!(picked.meet, on, "picked, {said:?}");

            // Zenek's address typed in the guest field and Enter.
            let mut typed = editor("1");
            fill(
                &mut typed,
                &described(&["Kasia", "Zenek"], said),
                Some(&found(&["Kasia", "Zenek"])),
                &[],
            );
            typed.guest_field = "zenek@example.com".to_string();
            typed.enter_guest(&[], &[]);
            assert_eq!(typed.guests[0].email, "zenek@example.com");
            assert_eq!(typed.meet, on, "typed, {said:?}");
        }
    }

    #[test]
    fn the_users_own_address_is_never_a_guest() {
        let mut editor = editor("1");
        let own = ["owner@example.com".to_string()];
        fill(
            &mut editor,
            &described(&["owner"], None),
            Some(&found(&["owner"])),
            &own,
        );
        assert!(editor.guests.is_empty());
        assert_eq!(editor.unresolved[0].suggested, Vec::new());
    }

    #[test]
    fn meet_follows_the_description_and_the_editors_rule_when_it_says_nothing() {
        let meet = |guests: &[&str], meet| {
            let mut editor = editor("1");
            fill(
                &mut editor,
                &described(guests, meet),
                Some(&found(guests)),
                &[],
            );
            editor.meet
        };
        assert!(meet(&[], Some(true)));
        assert!(meet(&["Ania"], Some(true)));
        assert!(!meet(&["Ania"], Some(false)));
        assert!(meet(&["Ania"], None));
        assert!(!meet(&[], None));
    }

    #[test]
    fn a_local_calendar_takes_neither_guests_nor_meet() {
        let mut editor = editor(local::ACCOUNT_ID);
        let event = described(&["Ania"], Some(true));
        fill(&mut editor, &event, Some(&found(&["Ania"])), &[]);
        assert!(editor.guests.is_empty());
        assert!(editor.unresolved.is_empty());
        assert!(!editor.meet);
    }

    #[test]
    fn with_nothing_looked_up_the_names_are_left_out_and_meet_still_follows() {
        let mut editor = editor("1");
        fill(&mut editor, &described(&["Ania"], Some(true)), None, &[]);
        assert!(editor.guests.is_empty());
        assert!(editor.unresolved.is_empty());
        assert!(editor.meet);
    }
}

/// `US-10-04`: what the app makes of a described change.
mod changes {
    use super::*;
    use crate::fixtures::{all_day, timed};
    use crate::model::{Access, Calendar, Rgb};
    use pure_rust_locales::Locale;

    fn answer(event: &str, kind: &str, day: &str, time: Value, length: Value) -> String {
        json!({
            "action": "change", "event": event, "title": "", "kind": kind, "day": day,
            "time": time, "duration_min": length, "guests": [], "meet": null,
        })
        .to_string()
    }

    fn change(answer: &str) -> Change {
        match read(answer, now().date_naive()) {
            Reading::Change(change) => change,
            other => panic!("no change in {answer}: {other:?}"),
        }
    }

    fn regional() -> Regional {
        Regional::new(Locale::pl_PL)
    }

    fn calendar() -> Calendar {
        Calendar {
            id: "work".to_string(),
            account_id: "1".to_string(),
            name: "Work".to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
            default_reminders: Vec::new(),
        }
    }

    /// The editor's fields saying when the event happens, after the change.
    fn changed(event: &Event, change: &Change) -> [String; 4] {
        let mut editor = Editor::of(event, calendar(), &regional());
        change.apply(&mut editor, &regional(), Warsaw);
        [
            editor.start_date,
            editor.start_time,
            editor.end_date,
            editor.end_time,
        ]
    }

    #[test]
    fn a_change_gives_the_words_naming_the_event_and_only_what_changes() {
        let moved = change(&answer(
            "spotkanie z Anią",
            "weekday",
            "fri",
            Value::Null,
            Value::Null,
        ));
        assert_eq!(
            moved,
            Change {
                event: "spotkanie z Anią".to_string(),
                date: Some(d(2026, 10, 30)),
                time: None,
                length: None,
            }
        );
        let open = change(&answer("", "same", "", json!("15:00"), Value::Null));
        assert_eq!(open.event, "");
        assert_eq!(open.date, None);
        assert_eq!(open.time, NaiveTime::from_hms_opt(15, 0, 0));
        let longer = change(&answer("retro", "same", "", Value::Null, json!(120)));
        assert_eq!(
            (longer.date, longer.time, longer.length),
            (None, None, Some(120))
        );
        // A day the model put beside "same" is the new day.
        let tomorrow = change(&answer(
            "standup",
            "same",
            "tomorrow",
            json!("10:00"),
            Value::Null,
        ));
        assert_eq!(tomorrow.date, Some(d(2026, 10, 27)));
        let friday = change(&answer("standup", "same", "fri", Value::Null, Value::Null));
        assert_eq!(friday.date, Some(d(2026, 10, 30)));
        let date = change(&answer(
            "urodziny",
            "same",
            "11-14",
            Value::Null,
            Value::Null,
        ));
        assert_eq!(date.date, Some(d(2026, 11, 14)));
        // An adding answer saying "same", as gemma4:e4b gives "Trening wieczorem", is of today,
        // or of the day its value names.
        let add = |day: &str| {
            let answer = format!(
                r#"{{"action":"add","event":"","title":"Trening","kind":"same","day":"{day}","time":"evening","duration_min":null,"guests":[],"meet":null}}"#
            );
            event(&answer).date
        };
        assert_eq!(add(""), d(2026, 10, 26));
        assert_eq!(add("tomorrow"), d(2026, 10, 27));
        let schema = &request(&Ollama::default(), "przesuń na 15", now())["format"];
        assert_eq!(
            schema["properties"]["action"]["enum"],
            json!(["add", "change", "none"])
        );
    }

    #[test]
    fn a_new_hour_or_day_moves_the_start_and_keeps_the_length_and_a_new_length_sets_the_end() {
        // "Sprint planning", Monday 26 October 10:00–11:00.
        let planning = timed("planning", d(2026, 10, 26), (10, 0), (11, 0));
        let at_15 = change(&answer("", "same", "", json!("15:00"), Value::Null));
        assert_eq!(
            changed(&planning, &at_15),
            ["26.10.2026", "15:00", "26.10.2026", "16:00"]
        );
        let friday = change(&answer("", "weekday", "fri", Value::Null, Value::Null));
        assert_eq!(
            changed(&planning, &friday),
            ["30.10.2026", "10:00", "30.10.2026", "11:00"]
        );
        let tomorrow_late = change(&answer("", "tomorrow", "", json!("23:30"), Value::Null));
        assert_eq!(
            changed(&planning, &tomorrow_late),
            ["27.10.2026", "23:30", "28.10.2026", "00:30"]
        );
        let longer = change(&answer("", "same", "", Value::Null, json!(90)));
        assert_eq!(
            changed(&planning, &longer),
            ["26.10.2026", "10:00", "26.10.2026", "11:30"]
        );
    }

    #[test]
    fn the_new_day_and_hour_are_the_users_for_an_event_of_another_zone() {
        use crate::fixtures::timed_in;
        use chrono_tz::America::New_York;
        // 11:00–12:00 in New York on Monday 26 October, shown at 16:00 in Warsaw.
        let call = timed_in("call", d(2026, 10, 26), (11, 0), (12, 0), New_York);
        let at_15 = change(&answer("", "same", "", json!("15:00"), Value::Null));
        // 15:00 in Warsaw, 10:00 in New York, where the editor shows it.
        assert_eq!(
            changed(&call, &at_15),
            ["26.10.2026", "10:00", "26.10.2026", "11:00"]
        );
        // 20:00 in New York on Monday is 01:00 on Tuesday in Warsaw: Wednesday is the user's
        // Wednesday, Tuesday evening in New York.
        let late = timed_in("late", d(2026, 10, 26), (20, 0), (21, 0), New_York);
        let wednesday = change(&answer("", "weekday", "wed", Value::Null, json!(90)));
        assert_eq!(
            changed(&late, &wednesday),
            ["27.10.2026", "20:00", "27.10.2026", "21:30"]
        );
    }

    #[test]
    fn an_all_day_event_moves_to_the_new_day_and_stays_all_day() {
        let trip = all_day("trip", d(2026, 11, 2), 2);
        let change = change(&answer("", "weekday", "fri", json!("15:00"), json!(30)));
        let mut editor = Editor::of(&trip, calendar(), &regional());
        change.apply(&mut editor, &regional(), Warsaw);
        assert!(editor.all_day);
        assert_eq!(
            (editor.start_date.as_str(), editor.end_date.as_str()),
            ("30.10.2026", "31.10.2026")
        );
    }

    #[test]
    fn names_fit_events_from_today_on_in_any_grammatical_case_in_the_order_they_start() {
        let today = now().date_naive();
        let titled = |title: &str, day: u32, hour: u32| {
            let mut event = timed(title, d(2026, 10, day), (hour, 0), (hour + 1, 0));
            event.title = title.to_string();
            event
        };
        let events = vec![
            titled("Standup", 29, 9),
            titled("Spotkanie z Anią", 19, 11),
            titled("Standup", 27, 9),
            titled("Dentysta", 28, 8),
            titled("Spotkanie z Anią Nowak", 28, 11),
            titled("Kawa", 30, 9),
            titled("Standup", 23, 9),
            titled("Dentist", 27, 8),
            titled("Kasia", 27, 12),
        ];
        let fit = |name: &str| matching(&events, name, today, Warsaw);
        // Past standups are not matched; the others in the order they start.
        assert_eq!(fit("standup"), [2, 0]);
        assert_eq!(fit("Spotkanie z Anią"), [4]);
        assert_eq!(fit("dentystę"), [3]);
        // Each word of the title "Kawa" is in the name.
        assert_eq!(fit("kawę z Bartkiem"), [5]);
        assert_eq!(fit("kawę"), [5]);
        assert_eq!(fit("dentist appointment"), [7]);
        assert_eq!(fit("Kate"), Vec::<usize>::new());
        assert_eq!(fit(""), Vec::<usize>::new());
        // An event of today counts, whatever its hour.
        let early = vec![titled("Standup", 26, 7)];
        assert_eq!(matching(&early, "standup", today, Warsaw), [0]);
    }
}
