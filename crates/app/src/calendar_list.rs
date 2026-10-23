//! Draws the calendars in the side panel, each with its menu, where a local colour is picked or
//! reset (`MOD-02-U04`). Grouping by account and hiding calendars come with the side panel of
//! `US-03-06`.

use calendar_core::colors::{CALENDAR_COLORS, LocalColors, calendar_color};
use calendar_core::model::{Calendar, Rgb};
use eframe::egui::{self, CornerRadius, Sense, Vec2};

use crate::grid_view::{color, text_on};

const SWATCH: f32 = 12.0;
const PALETTE_SWATCH: f32 = 20.0;
const PALETTE_COLUMNS: usize = 6;

/// A local colour picked for a calendar, or `None` to bring back Google's, by account ID and
/// calendar ID.
pub struct Picked {
    pub account_id: String,
    pub calendar_id: String,
    pub color: Option<Rgb>,
}

pub fn show(ui: &mut egui::Ui, calendars: &[&Calendar], local: &LocalColors) -> Option<Picked> {
    let mut picked = None;
    ui.strong("Calendars");
    egui::ScrollArea::vertical().show(ui, |ui| {
        for calendar in calendars {
            ui.push_id((&calendar.account_id, &calendar.id), |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(SWATCH), Sense::hover());
                    let shown = calendar_color(calendar, local);
                    ui.painter()
                        .rect_filled(rect, CornerRadius::same(2), color(shown));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(color) = menu(ui, calendar, local) {
                            picked = Some(Picked {
                                account_id: calendar.account_id.clone(),
                                calendar_id: calendar.id.clone(),
                                color,
                            });
                        }
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add(egui::Label::new(&calendar.name).truncate());
                        });
                    });
                });
            });
        }
    });
    picked
}

/// The calendar's menu: Google Calendar's calendar colours and "Reset". Gives what was picked.
fn menu(ui: &mut egui::Ui, calendar: &Calendar, local: &LocalColors) -> Option<Option<Rgb>> {
    let current = local
        .get(&(calendar.account_id.clone(), calendar.id.clone()))
        .copied();
    let mut picked = None;
    // The default fonts have no "⋮".
    ui.menu_button("…", |ui| {
        egui::Grid::new("palette")
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for (i, (name, rgb)) in CALENDAR_COLORS.iter().enumerate() {
                    let size = Vec2::splat(PALETTE_SWATCH);
                    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                    let painter = ui.painter();
                    painter.circle_filled(rect.center(), PALETTE_SWATCH / 2.0, color(*rgb));
                    if current == Some(*rgb) {
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "✔",
                            egui::FontId::proportional(12.0),
                            text_on(*rgb),
                        );
                    }
                    if response.on_hover_text(*name).clicked() {
                        picked = Some(Some(*rgb));
                        ui.close();
                    }
                    if (i + 1) % PALETTE_COLUMNS == 0 {
                        ui.end_row();
                    }
                }
            });
        // Google's colour comes back; nothing to reset while the calendar has it.
        if ui
            .add_enabled(current.is_some(), egui::Button::new("Reset"))
            .clicked()
        {
            picked = Some(None);
            ui.close();
        }
    });
    picked
}
