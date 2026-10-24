//! Draws the calendars in the side panel under their accounts (`MOD-03-F10`): each with the quick
//! toggle that hides its events (`MOD-03-F12`), a lock when it cannot be changed (`MOD-02-U01`),
//! its synchronization problem (`DEC-97`) and its menu, where a local colour is picked or reset
//! (`MOD-02-U04`) and its notifications are muted (`MOD-07-U03`). Also the settings' list, where
//! calendars are disabled and enabled again (`MOD-03-F13`).

use calendar_core::colors::{CALENDAR_COLORS, LocalColors};
use calendar_core::model::{Calendar, Rgb};
use calendar_core::side_panel::{Group, Row};
use calendar_core::texts::{self, ENABLED_HINT, READ_ONLY};
use eframe::egui::{self, CornerRadius, Sense, Stroke, StrokeKind, Vec2};

use crate::grid_view::{color, text_on};

const TOGGLE: f32 = 14.0;
const PALETTE_SWATCH: f32 = 20.0;
const PALETTE_COLUMNS: usize = 6;

/// The calendar a new event goes to, picked from `choices`, under their accounts (`MOD-05-F10`);
/// says whether another one was picked.
pub fn picker(
    ui: &mut egui::Ui,
    id_salt: &str,
    choices: &[(String, Vec<Calendar>)],
    picked: &mut Option<Calendar>,
) -> bool {
    let is_picked = |calendar: &Calendar| {
        picked
            .as_ref()
            .is_some_and(|p| p.account_id == calendar.account_id && p.id == calendar.id)
    };
    let mut chosen = None;
    let name = picked
        .as_ref()
        .map_or("", |calendar| calendar.name.as_str());
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(name)
        .show_ui(ui, |ui| {
            for (email, calendars) in choices {
                ui.label(egui::RichText::new(email).strong());
                for calendar in calendars {
                    let selected = is_picked(calendar);
                    if ui.selectable_label(selected, &calendar.name).clicked() && !selected {
                        chosen = Some(calendar.clone());
                    }
                }
            }
        });
    let changed = chosen.is_some();
    if changed {
        *picked = chosen;
    }
    changed
}

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
    /// The calendar's notifications are to be muted, or unmuted (`MOD-07-F12`).
    Muted(Calendar),
}

pub fn show(ui: &mut egui::Ui, groups: &[Group], local: &LocalColors) -> Option<Action> {
    let mut action = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Rows close together, as in Google Calendar's list.
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.spacing_mut().button_padding.y = 2.0;
        ui.spacing_mut().interact_size.y = 26.0;
        for (i, group) in groups.iter().enumerate() {
            // Each account apart from the one above it.
            if i > 0 {
                ui.add_space(10.0);
            }
            let email = egui::RichText::new(&group.email)
                .font(crate::appearance::strong(12.5))
                .color(ui.visuals().weak_text_color());
            ui.add(egui::Label::new(email).truncate());
            ui.add_space(2.0);
            for row in &group.calendars {
                let calendar = &row.calendar;
                ui.push_id((&calendar.account_id, &calendar.id), |ui| {
                    if let Some(a) = show_row(ui, row, local) {
                        action = Some(a);
                    }
                });
            }
        }
    });
    action
}

/// The settings' list of calendars under their accounts, each with a checkbox ticked while it is
/// enabled (`MOD-03-F13`); each account's calendars fold away under it, folded when the
/// settings open. Gives the calendar whose checkbox was used.
pub fn in_settings(
    ui: &mut egui::Ui,
    accounts: &[(String, Vec<(Calendar, bool)>)],
) -> Option<Calendar> {
    let mut used = None;
    for (email, calendars) in accounts {
        egui::CollapsingHeader::new(egui::RichText::new(email).strong())
            .id_salt(("calendars-in-settings", email))
            .default_open(false)
            .show(ui, |ui| {
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
            });
    }
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
    // Its menu's "…" shows while the row is under the pointer.
    let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    let rect = egui::Rect::from_min_size(ui.cursor().min, size);
    let hovered = ui.rect_contains_pointer(rect);
    // The row under the pointer is lit, as the views' events are.
    if hovered {
        let fill = ui.visuals().widgets.hovered.weak_bg_fill;
        ui.painter().rect_filled(
            rect.expand2(egui::vec2(4.0, 0.0)),
            CornerRadius::same(4),
            fill,
        );
    }
    ui.horizontal(|ui| {
        if toggle(ui, row).clicked() {
            action = Some(Action::Toggled(calendar.clone()));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            match menu(ui, row, local, hovered) {
                Some(Chosen::Color(color)) => {
                    action = Some(Action::Picked(Picked {
                        account_id: calendar.account_id.clone(),
                        calendar_id: calendar.id.clone(),
                        color,
                    }))
                }
                Some(Chosen::Mute) => action = Some(Action::Muted(calendar.clone())),
                None => {}
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
        texts::HIDE_ITS_EVENTS
    } else {
        texts::SHOW_ITS_EVENTS
    };
    response.on_hover_text(hint)
}

/// What was chosen in a calendar's menu.
enum Chosen {
    /// A local colour, or `None` to bring back Google's.
    Color(Option<Rgb>),
    /// Muting its notifications, or unmuting them.
    Mute,
}

/// The calendar's menu: Google Calendar's calendar colours and "Reset", then muting its
/// notifications or unmuting them. Gives what was chosen.
fn menu(ui: &mut egui::Ui, row: &Row, local: &LocalColors, shown: bool) -> Option<Chosen> {
    let calendar = &row.calendar;
    let current = local
        .get(&(calendar.account_id.clone(), calendar.id.clone()))
        .copied();
    let mut picked = None;
    // The default fonts have no "⋮".
    crate::widgets::menu_dots(ui, shown, |ui| {
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
                        picked = Some(Chosen::Color(Some(*rgb)));
                        ui.close();
                    }
                    if (i + 1) % PALETTE_COLUMNS == 0 {
                        ui.end_row();
                    }
                }
            });
        // Google's colour comes back; nothing to reset while the calendar has it.
        if ui
            .add_enabled(current.is_some(), egui::Button::new(texts::RESET))
            .clicked()
        {
            picked = Some(Chosen::Color(None));
            ui.close();
        }
        ui.separator();
        // Its events stay in the views; the reminders in Google stay as they are.
        if ui.button(mute_label(row.muted)).clicked() {
            picked = Some(Chosen::Mute);
            ui.close();
        }
    });
    picked
}

/// What the menu's item muting the calendar's notifications says, as they are now.
fn mute_label(muted: bool) -> &'static str {
    if muted { texts::UNMUTE } else { texts::MUTE }
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
    fn a_calendars_menu_mutes_its_notifications_or_unmutes_them() {
        assert_eq!(mute_label(false), texts::MUTE);
        assert_eq!(mute_label(true), texts::UNMUTE);
    }

    #[test]
    fn the_side_panels_marks_are_in_the_apps_fonts() {
        let ctx = egui::Context::default();
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .textures_delta
            .clear();
        let font = egui::FontId::proportional(14.0);
        // The calendars' locks, the toggles' ticks and the menus' dots; the bars' icons are drawn
        // (`icons`).
        for mark in ["🔒", "✔", "…"] {
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, mark)), "{mark}");
        }
    }
}
