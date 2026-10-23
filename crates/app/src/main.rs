mod week_view;

use calendar_core::fixtures;
use calendar_core::model::Event;
use calendar_core::view_model::{WeekViewModel, week_view};
use calendar_core::week::Week;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use eframe::egui;

fn system_zone() -> Tz {
    iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse().ok())
        .unwrap_or(Tz::UTC)
}

struct CalendarApp {
    events: Vec<Event>,
    zone: Tz,
    week: Week,
    view: WeekViewModel,
    built_at: DateTime<Utc>,
    view_state: week_view::State,
}

impl CalendarApp {
    fn new() -> Self {
        let zone = system_zone();
        let today = Utc::now().with_timezone(&zone).date_naive();
        // Test data until synchronization exists (`US-02-01`).
        let events = fixtures::profile(today);
        let week = Week::containing(today);
        let built_at = Utc::now();
        let view = week_view(&events, week, zone, built_at);
        CalendarApp {
            events,
            zone,
            week,
            view,
            built_at,
            view_state: week_view::State::default(),
        }
    }

    fn rebuild(&mut self) {
        self.built_at = Utc::now();
        self.view = week_view(&self.events, self.week, self.zone, self.built_at);
    }
}

impl eframe::App for CalendarApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The now line moves; nothing else changes on its own.
        if Utc::now() - self.built_at >= chrono::Duration::minutes(1) {
            self.rebuild();
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(30));
        egui::Panel::top("week-bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                // Only for the feasibility check; full navigation is `US-03-05`.
                if ui.button("‹").clicked() {
                    self.week = self.week.previous();
                    self.rebuild();
                }
                if ui.button("›").clicked() {
                    self.week = self.week.next();
                    self.rebuild();
                }
                let days = self.week.days();
                ui.label(format!(
                    "{} – {}",
                    days[0].format("%-d %b"),
                    days[6].format("%-d %b %Y")
                ));
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(change) = week_view::show(ui, &self.view, &mut self.view_state) {
                week_view::apply(&mut self.events, change);
                self.rebuild();
            }
        });
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
