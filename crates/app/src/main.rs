mod grid_view;

use calendar_core::fixtures;
use calendar_core::model::Event;
use calendar_core::view_model::{GridViewModel, grid_view};
use calendar_core::week::GridView;
use chrono::{DateTime, Duration, NaiveDate, Utc};
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
    grid: GridView,
    selected: NaiveDate,
    view: GridViewModel,
    built_at: DateTime<Utc>,
    view_state: grid_view::State,
}

impl CalendarApp {
    fn new() -> Self {
        let zone = system_zone();
        let today = Utc::now().with_timezone(&zone).date_naive();
        // Test data until synchronization exists (`US-02-01`).
        let events = fixtures::profile(today);
        let grid = GridView::Week;
        let built_at = Utc::now();
        let view = grid_view(&events, &grid.days(today), zone, built_at);
        CalendarApp {
            events,
            zone,
            grid,
            selected: today,
            view,
            built_at,
            view_state: grid_view::State::default(),
        }
    }

    fn rebuild(&mut self) {
        self.built_at = Utc::now();
        let days = self.grid.days(self.selected);
        self.view = grid_view(&self.events, &days, self.zone, self.built_at);
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
        egui::Panel::top("view-bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                // A plain picker and period buttons; full navigation and shortcuts are `US-03-05`.
                for (grid, name) in [
                    (GridView::Day, "Day"),
                    (GridView::ThreeDays, "3 days"),
                    (GridView::Week, "Week"),
                ] {
                    if ui.selectable_label(self.grid == grid, name).clicked() && self.grid != grid {
                        self.grid = grid;
                        self.rebuild();
                    }
                }
                ui.separator();
                let step = Duration::days(self.grid.days(self.selected).len() as i64);
                if ui.button("‹").clicked() {
                    self.selected -= step;
                    self.rebuild();
                }
                if ui.button("›").clicked() {
                    self.selected += step;
                    self.rebuild();
                }
                let days = self.grid.days(self.selected);
                let (first, last) = (days[0], days[days.len() - 1]);
                ui.label(if first == last {
                    first.format("%a %-d %b %Y").to_string()
                } else {
                    format!("{} – {}", first.format("%-d %b"), last.format("%-d %b %Y"))
                });
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(change) = grid_view::show(ui, &self.view, &mut self.view_state) {
                grid_view::apply(&mut self.events, change);
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
