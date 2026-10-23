//! Draws the calendars in the side panel under their accounts (`MOD-03-F10`): each with the quick
//! toggle that hides its events (`MOD-03-F12`), a lock when it cannot be changed (`MOD-02-U01`),
//! its synchronization problem (`DEC-97`) and its menu, where a local colour is picked or reset
//! (`MOD-02-U04`). Also the settings' list, where calendars are disabled and enabled again
//! (`MOD-03-F13`).

use calendar_core::colors::{CALENDAR_COLORS, LocalColors};
use calendar_core::model::{Calendar, Rgb};
use calendar_core::side_panel::{Group, Row};
use eframe::egui::{self, CornerRadius, Sense, Stroke, StrokeKind, Vec2};

use crate::grid_view::{color, text_on};

const TOGGLE: f32 = 14.0;
const PALETTE_SWATCH: f32 = 20.0;
const PALETTE_COLUMNS: usize = 6;

/// How tall the settings' list of calendars grows before it scrolls.
const SETTINGS_LIST_HEIGHT: f32 = 320.0;

/// What a calendar's checkbox in the settings says on hover.
const ENABLED_HINT: &str =
    "A disabled calendar is not shown in the side panel or the views, and is not synchronized";

/// What a calendar's lock says on hover.
pub const READ_ONLY: &str = "Read-only: this calendar can't be changed";

/// A local colour picked for a calendar, or `None` to bring back Google's, by account ID and
/// calendar ID.
pub struct Picked {
    pub account_id: String,
    pub calendar_id: String,
    pub color: Option<Rgb>,
}

pub enum Action {
    Picked(Picked),
    /// The calendar's quick toggle was used.
    Toggled(Calendar),
}

pub fn show(ui: &mut egui::Ui, groups: &[Group], local: &LocalColors) -> Option<Action> {
    let mut action = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for group in groups {
            ui.add(egui::Label::new(egui::RichText::new(&group.email).strong()).truncate());
            for row in &group.calendars {
                let calendar = &row.calendar;
                ui.push_id((&calendar.account_id, &calendar.id), |ui| {
                    if let Some(a) = show_row(ui, row, local) {
                        action = Some(a);
                    }
                });
            }
            ui.add_space(6.0);
        }
    });
    action
}

/// The settings' list of calendars under their accounts, each with a checkbox ticked while it is
/// enabled (`MOD-03-F13`). Gives the calendar whose checkbox was used.
pub fn in_settings(
    ui: &mut egui::Ui,
    accounts: &[(String, Vec<(Calendar, bool)>)],
) -> Option<Calendar> {
    let mut used = None;
    egui::ScrollArea::vertical()
        .id_salt("calendars-in-settings")
        .max_height(SETTINGS_LIST_HEIGHT)
        .show(ui, |ui| {
            for (email, calendars) in accounts {
                ui.label(egui::RichText::new(email).strong());
                for (calendar, enabled) in calendars {
                    let mut ticked = *enabled;
                    ui.push_id((&calendar.account_id, &calendar.id), |ui| {
                        if ui
                            .checkbox(&mut ticked, &calendar.name)
                            .on_hover_text(ENABLED_HINT)
                            .clicked()
                        {
                            used = Some(calendar.clone());
                        }
                    });
                }
                ui.add_space(6.0);
            }
        });
    used
}

/// The name with a lock before it when the calendar cannot be changed, as on its events.
pub fn row_label(name: &str, read_only: bool) -> String {
    if read_only {
        format!("🔒 {name}")
    } else {
        name.to_string()
    }
}

fn show_row(ui: &mut egui::Ui, row: &Row, local: &LocalColors) -> Option<Action> {
    let calendar = &row.calendar;
    let mut action = None;
    ui.horizontal(|ui| {
        if toggle(ui, row).clicked() {
            action = Some(Action::Toggled(calendar.clone()));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(color) = menu(ui, calendar, local) {
                action = Some(Action::Picked(Picked {
                    account_id: calendar.account_id.clone(),
                    calendar_id: calendar.id.clone(),
                    color,
                }));
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let label = row_label(&calendar.name, row.read_only);
                let label = ui.add(egui::Label::new(label).truncate());
                if row.read_only {
                    label.on_hover_text(READ_ONLY);
                }
            });
        });
    });
    if let Some(problem) = row.problem {
        ui.add(
            egui::Label::new(
                egui::RichText::new(problem)
                    .small()
                    .color(ui.visuals().warn_fg_color),
            )
            .wrap(),
        );
    }
    action
}

/// Google Calendar's checkbox in the calendar's colour: filled and ticked while its events are
/// shown, an empty box while they are hidden. Reached and used from the keyboard as any button.
fn toggle(ui: &mut egui::Ui, row: &Row) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(TOGGLE), Sense::click());
    let name = &row.calendar.name;
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::Role::CheckBox, true, row.shown, name));
    let painter = ui.painter();
    let fill = color(row.color);
    if row.shown {
        painter.rect_filled(rect, CornerRadius::same(2), fill);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "✔",
            egui::FontId::proportional(11.0),
            text_on(row.color),
        );
    } else {
        let stroke = Stroke::new(2.0, fill);
        painter.rect_stroke(
            rect.shrink(1.0),
            CornerRadius::same(2),
            stroke,
            StrokeKind::Inside,
        );
    }
    if response.has_focus() {
        let focus = ui.visuals().selection.stroke;
        painter.rect_stroke(
            rect.expand(2.0),
            CornerRadius::same(3),
            focus,
            StrokeKind::Outside,
        );
    }
    let hint = if row.shown {
        "Hide its events"
    } else {
        "Show its events"
    };
    response.on_hover_text(hint)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendars_that_cannot_be_changed_carry_a_lock() {
        assert_eq!(row_label("Work", false), "Work");
        assert_eq!(
            row_label("Holidays in Poland", true),
            "🔒 Holidays in Poland"
        );
    }

    #[test]
    fn the_side_panels_marks_are_in_the_apps_fonts() {
        let ctx = egui::Context::default();
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .textures_delta
            .clear();
        let font = egui::FontId::proportional(14.0);
        // The top bar's side panel button, the calendars' locks and the toggles' ticks.
        for mark in ["☰", "🔒", "✔", "…"] {
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, mark)), "{mark}");
        }
    }
}
