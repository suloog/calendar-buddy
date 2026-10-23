//! Draws the full event editor (`US-05-02`): the title, start and end, all-day, time zone,
//! location, description, calendar, busy or free and visibility of a new or an existing event.
//! What the fields mean and what saving does is `calendar_core::editor`'s.

use calendar_core::editor::{Editor, Save};
use calendar_core::model::{Calendar, Show, Visibility};
use calendar_core::regional::Regional;
use eframe::egui;

use crate::calendar_list;

const FIELD_WIDTH: f32 = 320.0;
const DATE_WIDTH: f32 = 100.0;
const TIME_WIDTH: f32 = 70.0;
/// How tall the list of time zones grows before it scrolls.
const ZONES_HEIGHT: f32 = 320.0;

fn show_label(show: Show) -> &'static str {
    match show {
        Show::Busy => "Busy",
        Show::Free => "Free",
    }
}

fn visibility_label(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Default => "Default visibility",
        Visibility::Public => "Public",
        Visibility::Private => "Private",
    }
}

/// A date or time field of the editor; says whether it changed.
fn field(ui: &mut egui::Ui, text: &mut String, width: f32) -> bool {
    ui.add(egui::TextEdit::singleline(text).desired_width(width))
        .changed()
}

/// Draws the editor with the calendars it can save to, under their accounts, and why changes are
/// refused now, if they are; what saving does once the user saves. Until the event can be saved,
/// the editor says why and **Save** is unavailable.
pub fn show(
    ui: &mut egui::Ui,
    editor: &mut Editor,
    regional: &Regional,
    choices: &[(String, Vec<Calendar>)],
    refused: Option<&str>,
) -> Option<Save> {
    egui::Grid::new("editor")
        .num_columns(2)
        .spacing([8.0, 6.0])
        .show(ui, |ui| {
            ui.label("Title");
            ui.add(
                egui::TextEdit::singleline(&mut editor.title)
                    .hint_text("Add title")
                    .desired_width(FIELD_WIDTH),
            );
            ui.end_row();

            ui.label("Starts");
            let start_changed = ui
                .horizontal(|ui| {
                    let date = field(ui, &mut editor.start_date, DATE_WIDTH);
                    let time = !editor.all_day && field(ui, &mut editor.start_time, TIME_WIDTH);
                    date || time
                })
                .inner;
            if start_changed {
                editor.follow_start(regional);
            }
            ui.end_row();

            ui.label("Ends");
            ui.horizontal(|ui| {
                field(ui, &mut editor.end_date, DATE_WIDTH);
                if !editor.all_day {
                    field(ui, &mut editor.end_time, TIME_WIDTH);
                }
            });
            ui.end_row();

            ui.label("");
            ui.checkbox(&mut editor.all_day, "All day");
            ui.end_row();

            if !editor.all_day {
                ui.label("Time zone");
                egui::ComboBox::from_id_salt("editor-zone")
                    .selected_text(editor.zone.name())
                    .height(ZONES_HEIGHT)
                    .show_ui(ui, |ui| {
                        for zone in chrono_tz::TZ_VARIANTS {
                            ui.selectable_value(&mut editor.zone, zone, zone.name());
                        }
                    });
                ui.end_row();
            }

            ui.label("Location");
            ui.add(
                egui::TextEdit::singleline(&mut editor.location)
                    .hint_text("Add location")
                    .desired_width(FIELD_WIDTH),
            );
            ui.end_row();

            ui.label("Description");
            ui.add(
                egui::TextEdit::multiline(&mut editor.description)
                    .hint_text("Add description")
                    .desired_width(FIELD_WIDTH),
            );
            ui.end_row();

            ui.label("Calendar");
            calendar_list::picker(ui, "editor-calendar", choices, &mut editor.calendar);
            ui.end_row();

            ui.label("Show as");
            egui::ComboBox::from_id_salt("editor-show")
                .selected_text(show_label(editor.show))
                .show_ui(ui, |ui| {
                    for show in [Show::Busy, Show::Free] {
                        ui.selectable_value(&mut editor.show, show, show_label(show));
                    }
                });
            ui.end_row();

            ui.label("Visibility");
            egui::ComboBox::from_id_salt("editor-visibility")
                .selected_text(visibility_label(editor.visibility))
                .show_ui(ui, |ui| {
                    for visibility in [Visibility::Default, Visibility::Public, Visibility::Private]
                    {
                        let label = visibility_label(visibility);
                        ui.selectable_value(&mut editor.visibility, visibility, label);
                    }
                });
            ui.end_row();
        });
    let saved = editor.save(regional);
    let problem = refused
        .map(str::to_string)
        .or_else(|| saved.as_ref().err().cloned());
    if let Some(problem) = &problem {
        ui.colored_label(ui.visuals().warn_fg_color, problem);
    }
    let clicked = ui
        .add_enabled(problem.is_none(), egui::Button::new("Save"))
        .clicked();
    if clicked { saved.ok() } else { None }
}
