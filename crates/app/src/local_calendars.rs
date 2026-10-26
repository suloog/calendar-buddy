//! The settings' "Local calendars" section (`US-09-02`, `MOD-09-U01`): each local calendar with
//! its colour, picked from Google Calendar's calendar colours, its name, renamed where it is
//! shown, its default notifications (`US-09-04`, `MOD-09-F14`), and deleting it, asked first when
//! it has events (`MOD-09-U03`); then adding one by name (`MOD-09-F02`).

use std::collections::HashMap;

use calendar_core::editor::{MOST_REMINDERS, ReminderField};
use calendar_core::model::{Calendar, Method, Reminder, Rgb};
use calendar_core::texts;
use eframe::egui::{self, Vec2};

use crate::calendar_list::palette;
use crate::editor::reminder_rows;
use crate::grid_view::color;
use crate::widgets;

const SWATCH: f32 = 16.0;

/// What the user did in the section.
pub enum Action {
    /// A new local calendar, by its name.
    Add(String),
    /// A local calendar renamed, recoloured or with other default notifications, as it now is.
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
    /// The default notifications while an edit of them is under way, by calendar ID.
    defaults: HashMap<String, Vec<ReminderField>>,
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
    // There is always a local calendar: the last one is not deleted (`MOD-09-F21`).
    let deletable = calendars.len() > 1;
    for calendar in calendars {
        ui.push_id(&calendar.id, |ui| {
            actions.extend(row(ui, state, calendar, deletable));
        });
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

/// One local calendar: its colour, its name and "Delete", unavailable when it is the last one, with
/// the question under it while the user is asked.
fn row(ui: &mut egui::Ui, state: &mut State, calendar: &Calendar, deletable: bool) -> Vec<Action> {
    let mut changed = calendar.clone();
    let ((), deleted) = egui::Sides::new().show(
        ui,
        |ui| {
            if let Some(rgb) = swatch(ui, calendar) {
                changed.color = rgb;
            }
            let typed = (state.names)
                .entry(calendar.id.clone())
                .or_insert_with(|| calendar.name.clone());
            let field = egui::TextEdit::singleline(typed).desired_width(220.0);
            let field = widgets::text_field(ui, field).on_hover_text(texts::RENAME_HINT);
            // Kept once the field is left; an empty name gives way to the one it had.
            if field.lost_focus() {
                let name = typed.trim().to_string();
                if !name.is_empty() {
                    changed.name = name;
                }
                state.names.remove(&calendar.id);
            } else if !field.has_focus() {
                state.names.remove(&calendar.id);
            }
        },
        |ui| {
            ui.add_enabled_ui(deletable, |ui| widgets::outlined_button(ui, texts::DELETE))
                .inner
                .on_disabled_hover_text(texts::LAST_LOCAL_CALENDAR)
                .clicked()
        },
    );
    if let Some(defaults) = defaults(ui, state, calendar) {
        changed.default_reminders = defaults;
    }
    let mut actions = Vec::new();
    if changed != *calendar {
        actions.push(Action::Change(changed));
    }
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

/// The calendar's default notifications, each so many minutes, hours or days before, removable,
/// and "Add notification" up to Google's limit (`MOD-09-F14`); notifications only, as Google
/// Calendar's calendar settings have them. Gives them as changed once an edit is finished.
fn defaults(ui: &mut egui::Ui, state: &mut State, calendar: &Calendar) -> Option<Vec<Reminder>> {
    let mut fields = (state.defaults.remove(&calendar.id)).unwrap_or_else(|| shown(calendar));
    let mut finished = false;
    ui.indent("defaults", |ui| {
        ui.label(texts::EVENT_NOTIFICATIONS)
            .on_hover_text(texts::EVENT_NOTIFICATIONS_HINT);
        finished = reminder_rows(ui, &mut fields, &[Method::Notification]);
        if fields.len() < MOST_REMINDERS && ui.button(texts::ADD_NOTIFICATION).clicked() {
            fields.push(ReminderField::NEW);
            finished = true;
        }
    });
    let (kept, editing) = settle(calendar, fields, finished);
    if let Some(editing) = editing {
        state.defaults.insert(calendar.id.clone(), editing);
    }
    kept
}

/// The calendar's default notifications as the rows show them.
fn shown(calendar: &Calendar) -> Vec<ReminderField> {
    (calendar.default_reminders.iter().copied())
        .map(ReminderField::of)
        .collect()
}

/// What becomes of the rows as the user left them: once an edit is finished, the defaults to
/// keep, if they differ from the calendar's, and the rows show the calendar again, as the file
/// took it; while one is under way (a number typed or dragged), the rows to show on, nothing
/// kept yet, so a passing number never reminds. With no Save to refuse it, a time too early is
/// kept at the limit.
fn settle(
    calendar: &Calendar,
    fields: Vec<ReminderField>,
    finished: bool,
) -> (Option<Vec<Reminder>>, Option<Vec<ReminderField>>) {
    if finished {
        let kept: Vec<Reminder> = (fields.into_iter())
            .map(|field| field.within_limit().reminder())
            .collect();
        ((kept != calendar.default_reminders).then_some(kept), None)
    } else {
        (None, (fields != shown(calendar)).then_some(fields))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use calendar_core::editor::Unit;
    use calendar_core::local;
    use chrono_tz::Europe::Warsaw;

    fn field(amount: u32, unit: Unit) -> ReminderField {
        ReminderField {
            method: Method::Notification,
            amount,
            unit,
        }
    }

    fn minutes(minutes: u32) -> Reminder {
        Reminder {
            method: Method::Notification,
            minutes,
        }
    }

    #[test]
    fn default_notifications_are_kept_once_an_edit_is_finished_never_while_typed() {
        let private = local::new_calendar("Private", Warsaw, &[]);
        let ten = vec![field(10, Unit::Minutes)];

        // Typed or dragged: shown on, nothing kept.
        assert_eq!(
            settle(&private, ten.clone(), false),
            (None, Some(ten.clone()))
        );
        // Left: kept, and the rows show the calendar again.
        assert_eq!(settle(&private, ten, true), (Some(vec![minutes(10)]), None));
        // Added and removed: kept as they are, none at all too.
        let added = vec![field(30, Unit::Minutes), field(1, Unit::Days)];
        assert_eq!(
            settle(&private, added, true),
            (Some(vec![minutes(30), minutes(1440)]), None)
        );
        assert_eq!(settle(&private, Vec::new(), true), (Some(Vec::new()), None));
    }

    #[test]
    fn default_notifications_unchanged_or_too_early_are_not_written_again() {
        let private = local::new_calendar("Private", Warsaw, &[]);
        let as_shown = shown(&private);
        assert_eq!(settle(&private, as_shown.clone(), false), (None, None));
        assert_eq!(settle(&private, as_shown, true), (None, None));

        // 30 minutes switched to days is kept as four weeks.
        let days = vec![field(30, Unit::Days)];
        assert_eq!(
            settle(&private, days, true),
            (Some(vec![minutes(4 * 7 * 24 * 60)]), None)
        );
    }
}
