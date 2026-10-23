//! Draws a `MiniCalendar` with egui. Which days it shows and marks is decided in
//! `calendar_core::navigation`.

use calendar_core::navigation::MiniCalendar;
use chrono::{Months, NaiveDate};
use eframe::egui::{self, RichText};

const DAY_SIZE: [f32; 2] = [26.0, 22.0];

/// What the user did in the mini calendar.
pub enum Action {
    /// Browse to the month starting on this date, without moving the view.
    Browse(NaiveDate),
    /// Show this date in the view.
    GoTo(NaiveDate),
}

pub fn show(ui: &mut egui::Ui, mini: &MiniCalendar) -> Option<Action> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.strong(mini.month.format("%B %Y").to_string());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("›").clicked() {
                action = Some(Action::Browse(mini.month + Months::new(1)));
            }
            if ui.small_button("‹").clicked() {
                action = Some(Action::Browse(mini.month - Months::new(1)));
            }
        });
    });
    egui::Grid::new("mini-calendar")
        .spacing([0.0, 0.0])
        .show(ui, |ui| {
            for weekday in ["M", "T", "W", "T", "F", "S", "S"] {
                ui.add_sized(
                    DAY_SIZE,
                    egui::Label::new(RichText::new(weekday).small().weak()),
                );
            }
            ui.end_row();
            for week in &mini.weeks {
                for day in week {
                    let mut text = RichText::new(day.date.format("%-d").to_string()).small();
                    if day.is_today {
                        text = text.strong().color(ui.visuals().hyperlink_color);
                    } else if !day.in_month {
                        text = text.weak();
                    }
                    let button = egui::Button::selectable(day.in_period, text);
                    if ui.add_sized(DAY_SIZE, button).clicked() {
                        action = Some(Action::GoTo(day.date));
                    }
                }
                ui.end_row();
            }
        });
    action
}
