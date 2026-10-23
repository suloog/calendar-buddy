//! Draws the full event editor (`US-05-02`): the title, start and end, all-day, time zone,
//! location, description, calendar, busy or free and visibility of a new or an existing event,
//! and how a new one repeats (`US-05-06`); a series' repetition opens in Google Calendar
//! (`US-05-07`); out of office and focus time where the account allows them (`US-05-08`). What
//! the fields mean and what saving does is `calendar_core::editor`'s. The event's own colour is
//! picked from Google's event colours (`US-05-09`).

use calendar_core::accounts::Entry;
use calendar_core::colors::EventColor;
use calendar_core::editor::{Editor, Ending, Save};
use calendar_core::model::{Calendar, Kind, Show, Visibility};
use calendar_core::regional::Regional;
use calendar_core::repeat::Pattern;
use eframe::egui::{self, Sense, Vec2};

use crate::calendar_list;
use crate::grid_view::{color, text_on};

const FIELD_WIDTH: f32 = 320.0;
const DATE_WIDTH: f32 = 100.0;
const TIME_WIDTH: f32 = 70.0;
const COUNT_WIDTH: f32 = 40.0;
const SWATCH: f32 = 20.0;
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

fn repeat_label(repeat: Option<&Pattern>) -> &'static str {
    match repeat {
        None => "Does not repeat",
        Some(Pattern::Daily) => "Daily",
        Some(Pattern::Weekdays) => "Every weekday (Monday to Friday)",
        Some(Pattern::Weekly(_)) => "Weekly",
        Some(Pattern::Monthly) => "Monthly",
        Some(Pattern::Yearly) => "Annually",
    }
}

fn ending_label(ends: Ending) -> &'static str {
    match ends {
        Ending::Never => "Never",
        Ending::OnDate => "On",
        Ending::AfterCount => "After",
    }
}

/// How a new event repeats and how its series ends (`MOD-05-F18`, `F19`), as rows of the grid.
fn repeat(ui: &mut egui::Ui, editor: &mut Editor, regional: &Regional) {
    ui.label("Repeats");
    let weekly = editor.weekly();
    let choices = [
        None,
        Some(Pattern::Daily),
        Some(Pattern::Weekdays),
        Some(weekly),
        Some(Pattern::Monthly),
        Some(Pattern::Yearly),
    ];
    egui::ComboBox::from_id_salt("editor-repeat")
        .selected_text(repeat_label(editor.repeat.as_ref()))
        .show_ui(ui, |ui| {
            for choice in choices {
                let label = repeat_label(choice.as_ref());
                let chosen = repeat_label(editor.repeat.as_ref()) == label;
                if ui.selectable_label(chosen, label).clicked() && !chosen {
                    editor.repeat = choice;
                }
            }
        });
    ui.end_row();
    let Some(pattern) = &editor.repeat else {
        return;
    };
    if let Pattern::Weekly(days) = pattern {
        let days = days.clone();
        ui.label("On");
        ui.horizontal(|ui| {
            let mut day = regional.week_start;
            for letter in regional.weekday_letters() {
                if ui.selectable_label(days.contains(&day), letter).clicked() {
                    editor.toggle_day(day);
                }
                day = day.succ();
            }
        });
        ui.end_row();
    }
    ui.label("Series ends");
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("editor-ends")
            .selected_text(ending_label(editor.ends))
            .show_ui(ui, |ui| {
                for ends in [Ending::Never, Ending::OnDate, Ending::AfterCount] {
                    ui.selectable_value(&mut editor.ends, ends, ending_label(ends));
                }
            });
        match editor.ends {
            Ending::Never => {}
            Ending::OnDate => {
                field(ui, &mut editor.last_date, DATE_WIDTH);
            }
            Ending::AfterCount => {
                field(ui, &mut editor.count, COUNT_WIDTH);
                ui.label("occurrences");
            }
        }
    });
    ui.end_row();
}

/// The event's own colour, one of Google's event colours, or none for its calendar's, as Google
/// Calendar's event colour menu offers them (`MOD-05-F25`). Reached and picked from the keyboard
/// as any button.
fn event_color(ui: &mut egui::Ui, picked: &mut Option<EventColor>) {
    ui.horizontal(|ui| {
        for choice in std::iter::once(None).chain(EventColor::ALL.map(Some)) {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(SWATCH), Sense::click());
            let name = choice.map_or("Calendar colour", EventColor::name);
            let chosen = *picked == choice;
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::Role::RadioButton, true, chosen, name)
            });
            let (center, radius) = (rect.center(), SWATCH / 2.0 - 1.0);
            let painter = ui.painter();
            let ink = match choice {
                Some(own) => {
                    painter.circle_filled(center, radius, color(own.rgb()));
                    text_on(own.rgb())
                }
                None => {
                    let stroke = ui.visuals().widgets.inactive.fg_stroke;
                    painter.circle_stroke(center, radius, stroke);
                    ui.visuals().text_color()
                }
            };
            if chosen {
                let font = egui::FontId::proportional(12.0);
                painter.text(center, egui::Align2::CENTER_CENTER, "✔", font, ink);
            }
            if response.has_focus() {
                let focus = ui.visuals().selection.stroke;
                painter.circle_stroke(center, radius + 2.0, focus);
            }
            if response.on_hover_text(name).clicked() {
                *picked = choice;
            }
        }
    });
}

/// A date or time field of the editor; says whether it changed.
fn field(ui: &mut egui::Ui, text: &mut String, width: f32) -> bool {
    ui.add(egui::TextEdit::singleline(text).desired_width(width))
        .changed()
}

/// What the user did in the editor.
pub enum Edited {
    Save(Save),
    /// Asked to change the series' repetition in Google Calendar (`MOD-05-F29`).
    OpenSeries,
}

/// Draws the editor with the calendars it can save to, under their accounts, the types the
/// accounts allow (`MOD-05-F24`), and why changes are refused now, if they are; what saving
/// does once the user saves. Until the event can be saved, the editor says why and **Save** is
/// unavailable. An occurrence of a series offers to change its repetition in Google Calendar, as
/// the app does not (`DEC-111`).
pub fn show(
    ui: &mut egui::Ui,
    editor: &mut Editor,
    regional: &Regional,
    choices: &[(String, Vec<Calendar>)],
    accounts: &[Entry],
    refused: Option<&str>,
) -> Option<Edited> {
    let mut open_series = false;
    let kinds = editor.kinds(accounts);
    egui::Grid::new("editor")
        .num_columns(2)
        .spacing([8.0, 6.0])
        .show(ui, |ui| {
            // As Google Calendar, where the calendar allows them; an existing event keeps its own.
            if !kinds.is_empty() {
                ui.label("Type");
                ui.horizontal(|ui| {
                    for kind in kinds {
                        ui.selectable_value(&mut editor.kind, kind, kind.label());
                    }
                });
                ui.end_row();
            } else if editor.kind != Kind::Ordinary {
                ui.label("Type");
                ui.label(editor.kind.label());
                ui.end_row();
            }

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

            // An existing event's repetition is not changed here (`DEC-111`).
            match &editor.opened {
                None => repeat(ui, editor, regional),
                Some((event, _)) if event.series.is_some() => {
                    ui.label("Repeats");
                    ui.horizontal(|ui| {
                        ui.label("Changed in Google Calendar");
                        open_series = ui.button("Open in Google Calendar").clicked();
                    });
                    ui.end_row();
                }
                Some(_) => {}
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

            ui.label("Colour");
            event_color(ui, &mut editor.color);
            ui.end_row();

            // Out of office and focus time are always busy.
            if !editor.kind.timed_only() {
                ui.label("Show as");
                egui::ComboBox::from_id_salt("editor-show")
                    .selected_text(show_label(editor.show))
                    .show_ui(ui, |ui| {
                        for show in [Show::Busy, Show::Free] {
                            ui.selectable_value(&mut editor.show, show, show_label(show));
                        }
                    });
                ui.end_row();
            }

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
    // The calendar picked may no longer allow the type picked.
    editor.kinds(accounts);
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
    if open_series {
        return Some(Edited::OpenSeries);
    }
    if clicked {
        saved.ok().map(Edited::Save)
    } else {
        None
    }
}
