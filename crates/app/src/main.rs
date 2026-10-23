mod grid_view;
mod month_view;

use calendar_core::fixtures;
use calendar_core::model::Event;
use calendar_core::month::{MonthViewModel, month_view};
use calendar_core::view_model::{GridViewModel, grid_view};
use calendar_core::week::GridView;
use chrono::{DateTime, Duration, Months, NaiveDate, Utc};
use chrono_tz::Tz;
use eframe::egui;

fn system_zone() -> Tz {
    iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse().ok())
        .unwrap_or(Tz::UTC)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Grid(GridView),
    Month,
}

enum Built {
    Grid(GridViewModel),
    Month(MonthViewModel),
}

fn build(events: &[Event], view: View, selected: NaiveDate, zone: Tz, now: DateTime<Utc>) -> Built {
    match view {
        View::Grid(grid) => Built::Grid(grid_view(events, &grid.days(selected), zone, now)),
        View::Month => Built::Month(month_view(events, selected, zone, now)),
    }
}

struct CalendarApp {
    events: Vec<Event>,
    zone: Tz,
    view: View,
    selected: NaiveDate,
    built: Built,
    built_at: DateTime<Utc>,
    view_state: grid_view::State,
    month_state: month_view::State,
}

impl CalendarApp {
    fn new() -> Self {
        let zone = system_zone();
        let today = Utc::now().with_timezone(&zone).date_naive();
        // Test data until synchronization exists (`US-02-01`).
        let events = fixtures::profile(today);
        let view = View::Grid(GridView::Week);
        let built_at = Utc::now();
        let built = build(&events, view, today, zone, built_at);
        CalendarApp {
            events,
            zone,
            view,
            selected: today,
            built,
            built_at,
            view_state: grid_view::State::default(),
            month_state: month_view::State::default(),
        }
    }

    fn rebuild(&mut self) {
        self.built_at = Utc::now();
        self.built = build(
            &self.events,
            self.view,
            self.selected,
            self.zone,
            self.built_at,
        );
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
        egui::Panel::top("view-bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                // A plain picker and period buttons; full navigation and shortcuts are `US-03-05`.
                for (view, name) in [
                    (View::Grid(GridView::Day), "Day"),
                    (View::Grid(GridView::ThreeDays), "3 days"),
                    (View::Grid(GridView::Week), "Week"),
                    (View::Month, "Month"),
                ] {
                    if ui.selectable_label(self.view == view, name).clicked() && self.view != view {
                        self.view = view;
                        self.month_state = month_view::State::default();
                        self.rebuild();
                    }
                }
                ui.separator();
                let previous = ui.button("‹").clicked();
                let next = ui.button("›").clicked();
                match self.view {
                    View::Grid(grid) => {
                        let step = Duration::days(grid.days(self.selected).len() as i64);
                        if previous {
                            self.selected -= step;
                        }
                        if next {
                            self.selected += step;
                        }
                    }
                    View::Month => {
                        if previous {
                            self.selected = self.selected - Months::new(1);
                        }
                        if next {
                            self.selected = self.selected + Months::new(1);
                        }
                    }
                }
                if previous || next {
                    self.month_state = month_view::State::default();
                    self.rebuild();
                }
                ui.label(match self.view {
                    View::Grid(grid) => {
                        let days = grid.days(self.selected);
                        let (first, last) = (days[0], days[days.len() - 1]);
                        if first == last {
                            first.format("%a %-d %b %Y").to_string()
                        } else {
                            format!("{} – {}", first.format("%-d %b"), last.format("%-d %b %Y"))
                        }
                    }
                    View::Month => self.selected.format("%B %Y").to_string(),
                });
            });
        });
        egui::CentralPanel::default().show(ui, |ui| match &self.built {
            Built::Grid(view) => {
                if let Some(change) = grid_view::show(ui, view, &mut self.view_state) {
                    grid_view::apply(&mut self.events, change);
                    self.rebuild();
                }
            }
            Built::Month(view) => month_view::show(ui, view, &mut self.month_state),
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
