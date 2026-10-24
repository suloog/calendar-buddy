//! Pictures of the app on made-up data, for looking at its design: two accounts, their calendars
//! and a week of events, drawn off screen into `target/preview/`. Nothing real is touched: the
//! cache is in memory, Google cannot be reached and no account is checked.
//!
//! `cargo test -p calendar-buddy preview -- --ignored`

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use calendar_core::accounts::{Account, Accounts};
use calendar_core::cache::Cache;
use calendar_core::guests::{Guest, Response};
use calendar_core::model::{About, Access, Calendar, Kind, Rgb, Timing};
use calendar_core::navigation::{Navigation, View};
use calendar_core::platform::{Appearance, Network};
use calendar_core::regional::Regional;
use calendar_core::settings::Settings;
use calendar_core::side_panel::Panel;
use calendar_core::sync::{SyncError, SyncedEvent, Window};
use calendar_core::theme::{Scheme, Theme};
use calendar_core::undo::LastChange;
use calendar_core::week::GridView;
use chrono::{Datelike, Duration as Span, NaiveDate, TimeZone, Utc};
use chrono_tz::{Europe::Warsaw, Tz};
use eframe::egui;
use egui_kittest::Harness;

use crate::appearance::Desktop;
use crate::checks::Checks;
use crate::connect::Connect;
use crate::network::Online;
use crate::remove::Removal;
use crate::sync::Synchronization;
use crate::{AGENDA_DAYS, CalendarApp, Dialog, Placement, build};

struct Online_;

impl Network for Online_ {
    fn watch(&self, changed: Box<dyn Fn(bool) + Send + Sync>) {
        changed(true);
    }
}

struct NoPreference;

impl Appearance for NoPreference {
    fn watch(&self, _changed: Box<dyn Fn(Option<Scheme>) + Send + Sync>) {}
}

fn account(id: &str, email: &str, workspace: bool) -> Account {
    Account {
        id: id.into(),
        email: email.into(),
        workspace,
        contacts: true,
        directory: workspace,
    }
}

fn calendar(account_id: &str, id: &str, name: &str, color: Rgb, access: Access) -> Calendar {
    Calendar {
        id: id.into(),
        account_id: account_id.into(),
        name: name.into(),
        color,
        access,
        zone: Warsaw,
        default_reminders: Vec::new(),
    }
}

/// An event on `day` from `from` to `to`, as hours and minutes.
fn timed(id: &str, title: &str, day: NaiveDate, from: (u32, u32), to: (u32, u32)) -> SyncedEvent {
    let at = |(h, m): (u32, u32)| {
        Warsaw
            .from_local_datetime(&day.and_hms_opt(h, m, 0).unwrap())
            .unwrap()
            .with_timezone(&Utc)
    };
    SyncedEvent {
        id: id.into(),
        title: title.into(),
        timing: Some(Timing::Timed {
            start: at(from),
            end: at(to),
            zone: Warsaw,
        }),
        recurrence: Vec::new(),
        instance_of: None,
        about: About::default(),
        etag: String::new(),
        organizer_only: false,
    }
}

fn all_day(id: &str, title: &str, first: NaiveDate, days: i64) -> SyncedEvent {
    SyncedEvent {
        timing: Some(Timing::AllDay {
            start: first,
            end_exclusive: first + Span::days(days),
        }),
        ..timed(id, title, first, (0, 0), (1, 0))
    }
}

fn kind(mut event: SyncedEvent, kind: Kind) -> SyncedEvent {
    event.about.kind = kind;
    event
}

fn with_guests(mut event: SyncedEvent, own: Response) -> SyncedEvent {
    event.about.guests = vec![
        Guest {
            email: "anna@example.com".into(),
            name: "Anna Nowak".into(),
            optional: false,
            response: Response::Accepted,
            own: false,
        },
        Guest {
            email: "alex@example.com".into(),
            name: String::new(),
            optional: false,
            response: own,
            own: true,
        },
    ];
    event
}

/// Two accounts with their calendars, and the week around `today` filled the way a working week
/// is: meetings, overlaps, focus time, an all-day event and a trip over several days.
fn cache(today: NaiveDate) -> Cache {
    let monday = today - Span::days(today.weekday().num_days_from_monday() as i64);
    let day = |n: i64| monday + Span::days(n);
    let mut cache = Cache::in_memory();
    let work = [
        calendar(
            "w",
            "alex@example.com",
            "Alex Morgan",
            Rgb(0x03, 0x9B, 0xE5),
            Access::Owner,
        ),
        calendar("w", "team", "Team", Rgb(0x33, 0xB6, 0x79), Access::Writer),
        calendar(
            "w",
            "holidays",
            "Holidays in Poland",
            Rgb(0x0B, 0x80, 0x43),
            Access::Reader,
        ),
    ];
    let home = [
        calendar(
            "p",
            "me@gmail.com",
            "Personal",
            Rgb(0x8E, 0x24, 0xAA),
            Access::Owner,
        ),
        calendar(
            "p",
            "family",
            "Family",
            Rgb(0xF4, 0x51, 0x1E),
            Access::Writer,
        ),
        calendar("p", "gym", "Gym", Rgb(0xE6, 0x7C, 0x73), Access::Owner),
    ];
    cache.set_calendars("w", &work).unwrap();
    cache.set_calendars("p", &home).unwrap();
    let mut mine = Vec::new();
    for n in 0..5 {
        mine.push(timed(
            &format!("standup{n}"),
            "Daily standup",
            day(n),
            (9, 30),
            (9, 45),
        ));
    }
    mine.extend([
        timed("plan", "Sprint planning", day(0), (10, 0), (11, 30)),
        kind(
            timed("focus1", "Deep work", day(0), (13, 0), (15, 0)),
            Kind::FocusTime,
        ),
        timed("1on1", "1:1 with Anna", day(1), (11, 0), (11, 30)),
        timed(
            "client",
            "Client call — Calendar Buddy",
            day(1),
            (14, 0),
            (15, 0),
        ),
        timed("review", "Design review", day(1), (14, 30), (15, 30)),
        with_guests(
            timed("arch", "Architecture sync", day(2), (10, 0), (11, 0)),
            Response::Awaiting,
        ),
        kind(
            timed("focus2", "Focus time", day(2), (13, 0), (16, 0)),
            Kind::FocusTime,
        ),
        timed("lunch", "Lunch with Piotr", day(3), (12, 30), (13, 30)),
        with_guests(
            timed("retro", "Retrospective", day(4), (15, 0), (16, 0)),
            Response::Declined,
        ),
        timed("demo", "Sprint demo", day(4), (11, 0), (12, 0)),
        kind(
            all_day("ooo", "Out of office", day(4), 1),
            Kind::OutOfOffice,
        ),
    ]);
    cache.set_events(&work[0], &mine).unwrap();
    cache
        .set_events(
            &work[1],
            &[
                timed("lunch-team", "Team lunch", day(2), (12, 0), (13, 0)),
                timed("release", "Release 1.4", day(3), (16, 0), (17, 0)),
                all_day("conf", "Rust conference", day(2), 2),
            ],
        )
        .unwrap();
    cache
        .set_events(
            &work[2],
            &[all_day("holiday", "All Saints' Day", day(6), 1)],
        )
        .unwrap();
    cache
        .set_events(
            &home[0],
            &[
                timed("dentist", "Dentist", day(3), (8, 0), (9, 0)),
                timed("cinema", "Cinema", day(5), (19, 0), (21, 30)),
            ],
        )
        .unwrap();
    cache
        .set_events(
            &home[1],
            &[
                timed("dinner", "Family dinner", day(6), (14, 0), (16, 0)),
                all_day("trip", "Trip to Kraków", day(5), 2),
            ],
        )
        .unwrap();
    cache
        .set_events(
            &home[2],
            &[
                timed("gym1", "Gym", day(0), (18, 0), (19, 0)),
                timed("gym2", "Gym", day(2), (18, 0), (19, 0)),
                timed("gym3", "Gym", day(4), (7, 0), (8, 0)),
            ],
        )
        .unwrap();
    cache
}

impl CalendarApp {
    /// The app on the made-up data, in `view` and `theme`.
    fn preview(ctx: &egui::Context, view: View, theme: Theme) -> Self {
        crate::appearance::install(ctx);
        let zone: Tz = Warsaw;
        let regional = Regional::from_env(|key| match key {
            "LC_TIME" => Some("en_GB.UTF-8".into()),
            _ => None,
        });
        let today = Utc::now().with_timezone(&zone).date_naive();
        let mut accounts = Accounts::default();
        accounts.add(account("w", "alex@example.com", true));
        accounts.add(account("p", "alex.morgan@example.org", false));
        let ids = vec!["w".to_string(), "p".to_string()];
        let window = Window::around(today, zone);
        let sync = Synchronization::with(
            ctx,
            move || cache(today),
            Box::new(|_| Err(SyncError("Google is not reached in previews".into()))),
            ids,
            window,
            Default::default(),
        );
        let settings = Settings {
            last_view: Some(view),
            theme,
            ..Settings::default()
        };
        let navigation = Navigation::start(settings.last_view, today, regional.week_start);
        let built_at = Utc::now();
        let (built, places) = build(&[], &settings, navigation, AGENDA_DAYS, zone, built_at);
        let (autostart_writer, _) = mpsc::channel();
        let (_, signals) = mpsc::channel();
        CalendarApp {
            accounts,
            accounts_writer: None,
            checks: Checks::paused(),
            problem: None,
            connect: Connect::new(),
            connected: None,
            removal: Removal::new(),
            sync,
            events: Vec::new(),
            cached: Vec::new(),
            saving: Default::default(),
            last: LastChange::default(),
            not_saved: Vec::new(),
            online: Online::watch(ctx, &Online_),
            desktop: Desktop::watch(ctx, &NoPreference),
            focused: false,
            regional,
            zone,
            zone_changes: mpsc::channel().1,
            navigation,
            panel: Panel::new(false),
            mini_month: today,
            settings,
            settings_writer: None,
            gone: Vec::new(),
            agenda_days: AGENDA_DAYS,
            built,
            places,
            invitations: Vec::new(),
            built_at,
            view_state: Default::default(),
            month_state: Default::default(),
            dialog: None,
            notified: None,
            signals,
            tray: false,
            quitting: false,
            reopening: false,
            placement: Placement::default(),
            start_at_login: false,
            autostart_writer,
        }
    }
}

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/preview");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Draws the app until the cached events are shown, does `then` to it, then saves the picture
/// as `name`.
fn picture(name: &str, view: View, theme: Theme, then: impl FnOnce(&mut CalendarApp)) {
    picture_sized(name, egui::vec2(1440.0, 900.0), view, theme, then);
}

/// `picture` in a window of `size`.
fn picture_sized(
    name: &str,
    size: egui::Vec2,
    view: View,
    theme: Theme,
    then: impl FnOnce(&mut CalendarApp),
) {
    let egui_theme = match theme {
        Theme::Dark => egui::Theme::Dark,
        _ => egui::Theme::Light,
    };
    let mut harness = Harness::builder()
        .with_size(size)
        .with_pixels_per_point(1.0)
        .with_theme(egui_theme)
        .wgpu()
        .build_eframe(|cc| CalendarApp::preview(&cc.egui_ctx, view, theme));
    let deadline = Instant::now() + Duration::from_secs(10);
    while harness.state().events.is_empty() {
        assert!(Instant::now() < deadline, "the cached events never came");
        std::thread::sleep(Duration::from_millis(20));
        harness.step();
    }
    then(harness.state_mut());
    harness.run_steps(4);
    let image = harness.render().expect("a picture");
    image.save(out_dir().join(format!("{name}.png"))).unwrap();
}

/// The place in `app.events` of the event titled `title`.
fn titled(app: &CalendarApp, title: &str) -> usize {
    let place = app.events.iter().position(|e| e.title == title);
    place.expect("the event is in the made-up data")
}

#[test]
#[ignore = "draws pictures for looking at the design; run on demand"]
fn preview() {
    let week = View::Grid(GridView::Week);
    let nothing = |_: &mut CalendarApp| {};
    picture("week-light", week, Theme::Light, nothing);
    picture("week-dark", week, Theme::Dark, nothing);
    let narrow = egui::vec2(960.0, 700.0);
    picture_sized("week-narrow-light", narrow, week, Theme::Light, nothing);
    picture(
        "day-light",
        View::Grid(GridView::Day),
        Theme::Light,
        nothing,
    );
    picture("month-light", View::Month, Theme::Light, nothing);
    picture("month-dark", View::Month, Theme::Dark, nothing);
    picture("agenda-light", View::Agenda, Theme::Light, nothing);
    picture("settings-light", week, Theme::Light, |app| {
        app.dialog = Some(Dialog::Settings)
    });
    picture_sized(
        "settings-short-light",
        egui::vec2(1100.0, 560.0),
        week,
        Theme::Light,
        |app| app.dialog = Some(Dialog::Settings),
    );
    picture("details-light", week, Theme::Light, |app| {
        app.open(titled(app, "Architecture sync"), None)
    });
    picture("details-dark", week, Theme::Dark, |app| {
        app.open(titled(app, "Client call — Calendar Buddy"), None)
    });
    picture("new-event-light", week, Theme::Light, |app| {
        let start = app.events[titled(app, "Sprint demo")].timing;
        let Timing::Timed { start, .. } = start else {
            unreachable!("a timed event")
        };
        let start = start + Span::hours(3);
        app.start_event(|length| (start, start + Span::minutes(length.into())));
    });
    picture("welcome-light", week, Theme::Light, |app| {
        app.accounts = Accounts::default()
    });
}
