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
        Reading::NoEvent => panic!("no event in {answer}"),
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

/// A sentence of the test set, with the event expected and the answer `gemma4:e4b` gave it in the
/// test of 2026-10-27 (variant D), which the fake model answers with.
struct Case {
    lang: String,
    text: String,
    date: String,
    time: Option<String>,
    length: u32,
    guests: Vec<String>,
    meet: bool,
    /// Words one of which the title holds.
    title: Vec<String>,
    answer: String,
}

/// The test set (`MOD-10-NFR-04`), moved from the test of 2026-10-27.
fn cases() -> Vec<Case> {
    let cases: Value = serde_json::from_str(include_str!("described_cases.json")).unwrap();
    let strings = |value: &Value| -> Vec<String> {
        (value.as_array().unwrap().iter())
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    };
    (cases.as_array().unwrap().iter())
        .map(|case| Case {
            lang: case["lang"].as_str().unwrap().to_string(),
            text: case["text"].as_str().unwrap().to_string(),
            date: case["date"].as_str().unwrap().to_string(),
            time: case["time"].as_str().map(str::to_string),
            length: case["duration_min"].as_u64().unwrap() as u32,
            guests: strings(&case["guests"]),
            meet: case["meet"].as_bool().unwrap(),
            title: strings(&case["title"]),
            answer: case["answer"].as_str().unwrap().to_string(),
        })
        .collect()
}

/// What a sentence's reading got right, field by field, as the test of 2026-10-27 scored it.
fn score(case: &Case, reading: &Reading) -> [(&'static str, bool); 6] {
    let Reading::Event(event) = reading else {
        return ["date", "time", "length", "guests", "meet", "title"].map(|field| (field, false));
    };
    let folded = |names: &[String]| {
        let mut names: Vec<String> = names.iter().map(|n| n.trim().to_lowercase()).collect();
        names.sort();
        names
    };
    let title = event.title.to_lowercase();
    [
        ("date", event.date.to_string() == case.date),
        (
            "time",
            event.time.map(|t| t.format("%H:%M").to_string()) == case.time,
        ),
        (
            "length",
            event.length.unwrap_or(create::DEFAULT_LENGTH) == case.length,
        ),
        ("guests", folded(&event.guests) == folded(&case.guests)),
        ("meet", event.meet == case.meet),
        ("title", case.title.iter().any(|word| title.contains(word))),
    ]
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
    let adding = |lang: &str| cases.iter().filter(|c| c.lang == lang).count();
    println!("adding sentences: {} pl, {} en", adding("pl"), adding("en"));
    assert!(cases.len() >= 30);
    assert!(adding("pl") > 0 && adding("en") > 0);
    // The changing sentences join the set with changing events, `US-10-04` (`DEC-221`).

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
