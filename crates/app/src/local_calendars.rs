//! The settings' "Local calendars" section (`US-09-02`, `MOD-09-U01`): each local calendar with
//! its colour, picked from Google Calendar's calendar colours, its name, renamed where it is
//! shown, and deleting it, asked first when it has events (`MOD-09-U03`); then adding one by
//! name (`MOD-09-F02`).

use std::collections::HashMap;

use calendar_core::model::{Calendar, Rgb};
use calendar_core::texts;
use eframe::egui::{self, Vec2};

use crate::calendar_list::palette;
use crate::grid_view::color;
use crate::widgets;

const SWATCH: f32 = 16.0;

/// What the user did in the section.
pub enum Action {
    /// A new local calendar, by its name.
    Add(String),
    /// A local calendar renamed or recoloured, as it now is.
    Change(Calendar),
    /// A local calendar to delete, and whether the user confirmed it.
    Delete(Calendar, bool),
}

/// What the section holds while the settings are open.
#[derive(Default)]
pub struct State {
    /// The name typed for a new calendar.
    new_name: String,
    /// The names being typed, by calendar ID.
    names: HashMap<String, String>,
    /// The calendar to delete and the number of its events, while the user is asked.
    asking: Option<(Calendar, usize)>,
}

impl State {
    /// The calendar has events: the user is asked before it is deleted (`MOD-09-U03`).
    pub fn ask(&mut self, calendar: Calendar, events: usize) {
        self.asking = Some((calendar, events));
    }

    /// The settings closed: what was typed and the question go.
    pub fn close(&mut self) {
        *self = State::default();
    }
}

/// Draws the local calendars, in their order, and the field adding one; gives what was done, in
/// the order it was done: a name left by clicking elsewhere is kept before that click's action.
pub fn show(ui: &mut egui::Ui, state: &mut State, calendars: &[Calendar]) -> Vec<Action> {
    let mut actions = Vec::new();
    for calendar in calendars {
        ui.push_id(&calendar.id, |ui| actions.extend(row(ui, state, calendar)));
    }
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let field = egui::TextEdit::singleline(&mut state.new_name)
            .hint_text(texts::NEW_CALENDAR_NAME)
            .desired_width(220.0);
        let entered = widgets::text_field(ui, field).lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let name = state.new_name.trim().to_string();
        let clicked = ui
            .add_enabled(!name.is_empty(), egui::Button::new(texts::ADD_CALENDAR))
            .clicked();
        // A calendar has a name, as in Google Calendar; it need not be unique.
        if (entered || clicked) && !name.is_empty() {
            state.new_name.clear();
            actions.push(Action::Add(name));
        }
    });
    actions
}

/// One local calendar: its colour, its name and "Delete", with the question under it while the
/// user is asked.
fn row(ui: &mut egui::Ui, state: &mut State, calendar: &Calendar) -> Vec<Action> {
    let (changed, deleted) = egui::Sides::new().show(
        ui,
        |ui| {
            let mut changed = None;
            if let Some(rgb) = swatch(ui, calendar)
                && rgb != calendar.color
            {
                changed = Some(Calendar {
                    color: rgb,
                    ..calendar.clone()
                });
            }
            let typed = (state.names)
                .entry(calendar.id.clone())
                .or_insert_with(|| calendar.name.clone());
            let field = egui::TextEdit::singleline(typed).desired_width(220.0);
            let field = widgets::text_field(ui, field).on_hover_text(texts::RENAME_HINT);
            // Kept once the field is left; an empty name gives way to the one it had.
            if field.lost_focus() {
                let name = typed.trim().to_string();
                if !name.is_empty() && name != calendar.name {
                    changed = Some(Calendar {
                        name,
                        ..calendar.clone()
                    });
                }
                state.names.remove(&calendar.id);
            } else if !field.has_focus() {
                state.names.remove(&calendar.id);
            }
            changed
        },
        |ui| widgets::outlined_button(ui, texts::DELETE).clicked(),
    );
    let mut actions: Vec<Action> = changed.into_iter().map(Action::Change).collect();
    if deleted {
        actions.push(Action::Delete(calendar.clone(), false));
    }
    if let Some((asked, events)) = &state.asking
        && asked.id == calendar.id
    {
        ui.label(texts::confirm_local_deletion(&calendar.name, *events));
        ui.horizontal(|ui| {
            if ui.add(widgets::primary(ui, texts::DELETE)).clicked() {
                actions.push(Action::Delete(calendar.clone(), true));
                state.asking = None;
            }
            if ui.button(texts::CANCEL).clicked() {
                state.asking = None;
            }
        });
    }
    ui.add_space(4.0);
    actions
}

/// The calendar's colour, opening Google Calendar's calendar colours, the calendar's ticked;
/// gives the one picked.
fn swatch(ui: &mut egui::Ui, calendar: &Calendar) -> Option<Rgb> {
    let mut picked = None;
    // An empty button the colour is drawn over, so it is reached from the keyboard as any.
    let button = egui::Button::new("").min_size(Vec2::splat(SWATCH + 8.0));
    let (response, _) = egui::containers::menu::MenuButton::from_button(button).ui(ui, |ui| {
        picked = palette(ui, Some(calendar.color));
    });
    let response = response.on_hover_text(texts::COLOR_HINT);
    let center = response.rect.center();
    (ui.painter()).circle_filled(center, SWATCH / 2.0, color(calendar.color));
    picked
}
