mod agenda_view;
mod grid_view;
mod mini_calendar;
mod month_view;

use calendar_core::agenda::{AgendaViewModel, agenda_view};
use calendar_core::details::{When, event_details};
use calendar_core::fixtures;
use calendar_core::model::Event;
use calendar_core::month::{MonthViewModel, month_view};
use calendar_core::navigation::{Navigation, View, mini_calendar};
use calendar_core::settings::{self, Settings};
use calendar_core::shortcuts::{self, Command, SHORTCUTS};
use calendar_core::view_model::{GridViewModel, grid_view};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use chrono_tz::Tz;
use eframe::egui;
use std::sync::mpsc::{self, Sender};

fn system_zone() -> Tz {
    iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse().ok())
        .unwrap_or(Tz::UTC)
}

enum Built {
    Grid(GridViewModel),
    Month(MonthViewModel),
    Agenda(AgendaViewModel),
}

/// What is open over the view; like Google Calendar, one thing at a time.
enum Dialog {
    Shortcuts,
    /// The date typed so far, and whether the field has had the keyboard yet.
    GoToDate(String, bool),
    /// The event editor is `MOD-05`; until then this only says where the event would go.
    NewEvent,
    /// The index of the event whose details are open, and what had the keyboard when it opened.
    Event(usize, Option<egui::Id>),
}

/// Days the agenda shows when opened, and how many more each step of scrolling loads.
const AGENDA_DAYS: i64 = 30;

fn build(
    events: &[Event],
    navigation: Navigation,
    agenda_days: i64,
    zone: Tz,
    now: DateTime<Utc>,
) -> Built {
    let selected = navigation.selected;
    match navigation.view {
        View::Grid(grid) => Built::Grid(grid_view(events, &grid.days(selected), zone, now)),
        View::Month => Built::Month(month_view(events, selected, zone, now)),
        View::Agenda => Built::Agenda(agenda_view(events, zone, now, agenda_days)),
    }
}

struct CalendarApp {
    events: Vec<Event>,
    zone: Tz,
    navigation: Navigation,
    /// Any day of the month the mini calendar shows; follows the view, browsed on its own.
    mini_month: NaiveDate,
    /// Saves the settings off the UI thread; `None` when there is no home directory.
    settings_writer: Option<Sender<Settings>>,
    /// Days from today the agenda covers; grows as the user scrolls down.
    agenda_days: i64,
    built: Built,
    built_at: DateTime<Utc>,
    view_state: grid_view::State,
    month_state: month_view::State,
    dialog: Option<Dialog>,
}

impl CalendarApp {
    fn new() -> Self {
        let zone = system_zone();
        let today = Utc::now().with_timezone(&zone).date_naive();
        // Test data until synchronization exists (`US-02-01`).
        let events = fixtures::profile(today);
        let settings_path = settings::default_path();
        let last_view = settings_path
            .as_deref()
            .and_then(|path| Settings::load(path).last_view);
        let settings_writer = settings_path.map(|path| {
            let (sender, receiver) = mpsc::channel::<Settings>();
            // The UI never waits on disk.
            std::thread::spawn(move || {
                for settings in receiver {
                    // Losing the last view is acceptable (`DEC-15`); the app still works.
                    let _ = settings.save(&path);
                }
            });
            sender
        });
        let navigation = Navigation::start(last_view, today);
        let built_at = Utc::now();
        let built = build(&events, navigation, AGENDA_DAYS, zone, built_at);
        CalendarApp {
            events,
            zone,
            navigation,
            mini_month: today,
            settings_writer,
            agenda_days: AGENDA_DAYS,
            built,
            built_at,
            view_state: grid_view::State::default(),
            month_state: month_view::State::default(),
            dialog: None,
        }
    }

    fn rebuild(&mut self) {
        self.built_at = Utc::now();
        self.built = build(
            &self.events,
            self.navigation,
            self.agenda_days,
            self.zone,
            self.built_at,
        );
    }

    fn today(&self) -> NaiveDate {
        Utc::now().with_timezone(&self.zone).date_naive()
    }

    /// Google Calendar's shortcuts (`DEC-55`), unless a text field is taking the keys. Focus on a
    /// button or an event does not stop them.
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.close_dialog(ctx);
        }
        // Google ignores its keys pressed with a modifier; egui already drops Ctrl's text.
        if ctx.text_edit_focused() || ctx.input(|i| i.modifiers.alt) {
            return;
        }
        let typed: Vec<char> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::Text(text) => Some(text.chars()),
                    _ => None,
                })
                .flatten()
                .collect()
        });
        for command in typed.into_iter().filter_map(shortcuts::command) {
            match command {
                Command::GoToDate => {
                    let date = self.navigation.selected.format("%Y-%m-%d").to_string();
                    self.dialog = Some(Dialog::GoToDate(date, false));
                }
                Command::NewEvent => self.dialog = Some(Dialog::NewEvent),
                Command::ShowShortcuts => self.dialog = Some(Dialog::Shortcuts),
                command => {
                    // Like Google Calendar, moving the view closes what is open over it.
                    self.dialog = None;
                    self.navigation.run(command, self.today());
                    if command == Command::Today {
                        // As the Today button: brings back a browsed mini calendar.
                        self.mini_month = self.navigation.selected;
                    }
                }
            }
        }
    }

    /// Closes the open dialog; like Google Calendar, an event's details give the keyboard back to
    /// the event.
    fn close_dialog(&mut self, ctx: &egui::Context) {
        if let Some(Dialog::Event(_, Some(id))) = self.dialog.take() {
            ctx.memory_mut(|m| m.request_focus(id));
        }
    }

    fn dialog(&mut self, ctx: &egui::Context) {
        let Some(dialog) = &mut self.dialog else {
            return;
        };
        let mut open = true;
        let mut go_to = None;
        let title = match dialog {
            Dialog::Shortcuts => "Keyboard shortcuts",
            Dialog::GoToDate(..) => "Go to date",
            Dialog::NewEvent => "New event",
            Dialog::Event(..) => "Event",
        };
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| match dialog {
                Dialog::Shortcuts => {
                    egui::Grid::new("shortcuts").show(ui, |ui| {
                        for (key, _, description) in SHORTCUTS {
                            ui.monospace(key.to_string());
                            ui.label(description);
                            ui.end_row();
                        }
                    });
                }
                Dialog::GoToDate(text, focused) => {
                    ui.label("Date (YYYY-MM-DD)");
                    let field = ui.text_edit_singleline(text);
                    let date = shortcuts::parse_date(text);
                    let entered =
                        field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    // The field takes the keyboard when the dialog opens and after a wrong date.
                    if !*focused || (entered && date.is_none()) {
                        field.request_focus();
                        *focused = true;
                    }
                    if ui
                        .add_enabled(date.is_some(), egui::Button::new("Go"))
                        .clicked()
                        || entered
                    {
                        go_to = date;
                    }
                }
                Dialog::NewEvent => {
                    ui.label(self.navigation.selected.format("%A %-d %B %Y").to_string());
                    ui.label("Creating events is not available yet.");
                }
                Dialog::Event(index, _) => {
                    let details = event_details(&self.events[*index], self.zone);
                    ui.heading(details.title.as_deref().unwrap_or("Busy"));
                    ui.label(match details.when {
                        // An event ending at midnight stays on its day: 23:00 – 00:00.
                        When::Timed { start, end }
                            if start.date_naive()
                                == (end - chrono::Duration::seconds(1)).max(start).date_naive() =>
                        {
                            format!(
                                "{} – {}",
                                start.format("%A %-d %B %Y, %H:%M"),
                                end.format("%H:%M")
                            )
                        }
                        When::Timed { start, end } => format!(
                            "{} – {}",
                            start.format("%-d %B %Y, %H:%M"),
                            end.format("%-d %B %Y, %H:%M")
                        ),
                        When::AllDay { first, last } if first == last => {
                            first.format("%A %-d %B %Y").to_string()
                        }
                        When::AllDay { first, last } => {
                            let from = if first.year() == last.year() {
                                "%-d %B"
                            } else {
                                "%-d %B %Y"
                            };
                            format!("{} – {}", first.format(from), last.format("%-d %B %Y"))
                        }
                    });
                }
            });
        if let Some(date) = go_to {
            self.navigation.go_to(date);
            self.mini_month = date;
            open = false;
        }
        if !open {
            self.close_dialog(ctx);
        }
    }

    /// Shows what `navigation` now points at: the mini calendar follows it, and a new view is
    /// remembered for the next start.
    fn navigated(&mut self, before: Navigation) {
        if self.navigation == before {
            return;
        }
        if self.navigation.view != before.view {
            self.agenda_days = AGENDA_DAYS;
            if let Some(writer) = &self.settings_writer {
                let _ = writer.send(Settings {
                    last_view: Some(self.navigation.view),
                });
            }
        }
        self.mini_month = self.navigation.selected;
        self.month_state = month_view::State::default();
        self.rebuild();
    }
}

impl eframe::App for CalendarApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The now line and today move; nothing else changes on its own.
        if Utc::now() - self.built_at >= chrono::Duration::minutes(1) {
            self.rebuild();
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(30));
        let before = self.navigation;
        self.shortcuts(ui.ctx());
        egui::Panel::top("view-bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Today").clicked() {
                    self.navigation.today(self.today());
                    // Brings back a mini calendar browsed away, even when the view stays put.
                    self.mini_month = self.navigation.selected;
                }
                // The agenda always starts now; there is no period to step through.
                let periods = self.navigation.view != View::Agenda;
                if ui.add_enabled(periods, egui::Button::new("‹")).clicked() {
                    self.navigation.previous();
                }
                if ui.add_enabled(periods, egui::Button::new("›")).clicked() {
                    self.navigation.next();
                }
                ui.label(match (self.navigation.view, self.navigation.period()) {
                    (View::Month, _) => self.navigation.selected.format("%B %Y").to_string(),
                    (_, Some((first, last))) if first == last => {
                        first.format("%a %-d %b %Y").to_string()
                    }
                    (_, Some((first, last))) => {
                        format!("{} – {}", first.format("%-d %b"), last.format("%-d %b %Y"))
                    }
                    (_, None) => "Upcoming".to_string(),
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for view in View::ALL.into_iter().rev() {
                        if ui
                            .selectable_label(self.navigation.view == view, view.name())
                            .clicked()
                        {
                            self.navigation.switch(view);
                        }
                    }
                });
            });
        });
        egui::Panel::left("side-panel")
            .resizable(false)
            .show(ui, |ui| {
                let today = self.today();
                let mini = mini_calendar(self.mini_month, &self.navigation, today);
                match mini_calendar::show(ui, &mini) {
                    Some(mini_calendar::Action::Browse(month)) => self.mini_month = month,
                    Some(mini_calendar::Action::GoTo(date)) => {
                        self.navigation.go_to(date);
                        self.mini_month = date;
                    }
                    None => {}
                }
            });
        self.dialog(ui.ctx());
        self.navigated(before);
        let mut opened = None;
        egui::CentralPanel::default().show(ui, |ui| match &self.built {
            Built::Grid(view) => {
                if let Some(change) = grid_view::show(ui, view, &mut self.view_state, &mut opened) {
                    grid_view::apply(&mut self.events, change);
                    self.rebuild();
                }
            }
            Built::Month(view) => month_view::show(ui, view, &mut self.month_state, &mut opened),
            Built::Agenda(view) => {
                if agenda_view::show(ui, view, &mut opened) {
                    self.agenda_days += AGENDA_DAYS;
                    self.rebuild();
                    ui.ctx().request_repaint();
                }
            }
        });
        if let Some(event) = opened {
            let focused = ui.ctx().memory(|m| m.focused());
            self.dialog = Some(Dialog::Event(event, focused));
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("Calendar Buddy")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([480.0, 360.0]),
        ..Default::default()
    };
    eframe::run_native(
        "calendar-buddy",
        options,
        Box::new(|_cc| Ok(Box::new(CalendarApp::new()))),
    )
}
