//! Draws the full event editor (`US-05-02`): the title, start and end, all-day, time zone,
//! location, description, calendar, busy or free and visibility of a new or an existing event,
//! and how a new one repeats (`US-05-06`); a series' repetition opens in Google Calendar
//! (`US-05-07`); out of office and focus time where the account allows them (`US-05-08`). What
//! the fields mean and what saving does is `calendar_core::editor`'s. The event's own colour is
//! picked from Google's event colours (`US-05-09`). Guests are added by address or from the people
//! suggested while typing, marked optional and removed (`US-06-01`); the "Add Google Meet" switch
//! turns on with the first guest (`US-06-07`). Under the time, the guests' availability that day
//! (`US-06-08`). The event's reminders, starting from its calendar's defaults (`US-07-04`).

use calendar_core::accounts::Entry;
use calendar_core::availability::Preview;
use calendar_core::colors::EventColor;
use calendar_core::editor::{
    Editor, Ending, LONGEST_REMINDER, MOST_REMINDERS, ReminderField, Save, Unit,
};
use calendar_core::guests::Person;
use calendar_core::model::{Calendar, Kind, Method, Show, Visibility};
use calendar_core::regional::Regional;
use calendar_core::repeat::Pattern;
use calendar_core::texts::{self, CONFLICT, NO_DATA};
use eframe::egui::{self, CornerRadius, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::calendar_list;
use crate::grid_view::{color, text_on};

const FIELD_WIDTH: f32 = 320.0;
const DATE_WIDTH: f32 = 100.0;
const TIME_WIDTH: f32 = 70.0;
const COUNT_WIDTH: f32 = 40.0;
const SWATCH: f32 = 20.0;
/// How tall the list of time zones grows before it scrolls.
const ZONES_HEIGHT: f32 = 320.0;
/// The size of a guest's strip of the day in the availability preview.
const STRIP: Vec2 = Vec2::new(240.0, 14.0);

fn show_label(show: Show) -> &'static str {
    match show {
        Show::Busy => texts::SHOW_BUSY,
        Show::Free => texts::FREE,
    }
}

fn visibility_label(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Default => texts::DEFAULT_VISIBILITY,
        Visibility::Public => texts::PUBLIC,
        Visibility::Private => texts::PRIVATE,
    }
}

fn method_label(method: Method) -> &'static str {
    match method {
        Method::Notification => texts::NOTIFICATION,
        Method::Email => texts::EMAIL,
    }
}

fn unit_label(unit: Unit) -> &'static str {
    match unit {
        Unit::Minutes => texts::MINUTES,
        Unit::Hours => texts::HOURS,
        Unit::Days => texts::DAYS,
    }
}

fn repeat_label(repeat: Option<&Pattern>) -> &'static str {
    match repeat {
        None => texts::DOES_NOT_REPEAT,
        Some(Pattern::Daily) => texts::DAILY,
        Some(Pattern::Weekdays) => texts::EVERY_WEEKDAY,
        Some(Pattern::Weekly(_)) => texts::WEEKLY,
        Some(Pattern::Monthly) => texts::MONTHLY,
        Some(Pattern::Yearly) => texts::ANNUALLY,
    }
}

fn ending_label(ends: Ending) -> &'static str {
    match ends {
        Ending::Never => texts::NEVER,
        Ending::OnDate => texts::ON,
        Ending::AfterCount => texts::AFTER,
    }
}

/// How a new event repeats and how its series ends (`MOD-05-F18`, `F19`), as rows of the grid.
fn repeat(ui: &mut egui::Ui, editor: &mut Editor, regional: &Regional) {
    ui.label(texts::REPEATS);
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
        ui.label(texts::ON);
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
    ui.label(texts::SERIES_ENDS);
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
                ui.label(texts::OCCURRENCES);
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
            let name = choice.map_or(texts::CALENDAR_COLOUR, EventColor::name);
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

/// The guest field, the people suggested for what is typed in it, why what was typed was not
/// added, and the guests, each marked optional or not and removable (`MOD-06-F01`…`F07`). Enter
/// or a click adds; the user's own addresses (`own`) are never added.
fn guests(ui: &mut egui::Ui, editor: &mut Editor, suggested: &[Person], own: &[String]) {
    ui.vertical(|ui| {
        let field = ui.add(
            egui::TextEdit::singleline(&mut editor.guest_field)
                .hint_text(texts::ADD_GUESTS)
                .desired_width(FIELD_WIDTH),
        );
        if field.changed() {
            editor.guest_problem = None;
        }
        if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            editor.enter_guest(suggested, own);
            // The field keeps the keyboard for the next guest.
            field.request_focus();
        }
        if let Some(problem) = &editor.guest_problem {
            ui.colored_label(ui.visuals().warn_fg_color, problem);
        }
        for person in suggested {
            if ui.button(person.label()).clicked() {
                editor.add_guest(person, own);
            }
        }
        let mut removed = None;
        for guest in editor.listed_guests() {
            ui.horizontal(|ui| {
                let shown = Person {
                    name: guest.name.clone(),
                    email: guest.email.clone(),
                };
                ui.label(shown.label());
                ui.checkbox(&mut guest.optional, texts::OPTIONAL);
                if ui.button(texts::REMOVE).clicked() {
                    removed = Some(guest.email.clone());
                }
            });
        }
        if let Some(email) = removed {
            editor.remove_guest(&email);
        }
    });
}

/// The event's reminders, each a notification or an e-mail so many minutes, hours or days before,
/// and removable; "Add notification" up to Google's limit, and "Use calendar defaults" for an
/// event with reminders of its own (`MOD-07-F04`, `F05`). Changing the calendar's defaults shown
/// makes them the event's own, as in Google Calendar.
fn reminders(ui: &mut egui::Ui, editor: &mut Editor) {
    let shown = editor.reminder_fields();
    let mut fields = shown.clone();
    ui.vertical(|ui| {
        let mut removed = None;
        for (i, field) in fields.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt(("editor-reminder-method", i))
                    .selected_text(method_label(field.method))
                    .show_ui(ui, |ui| {
                        for method in [Method::Notification, Method::Email] {
                            ui.selectable_value(&mut field.method, method, method_label(method));
                        }
                    });
                // A number too large for the unit picked stays, and Save says why.
                let most = LONGEST_REMINDER / field.unit.minutes();
                ui.add(
                    egui::DragValue::new(&mut field.amount)
                        .range(0..=most)
                        .clamp_existing_to_range(false),
                );
                egui::ComboBox::from_id_salt(("editor-reminder-unit", i))
                    .selected_text(unit_label(field.unit))
                    .show_ui(ui, |ui| {
                        for unit in Unit::ALL {
                            ui.selectable_value(&mut field.unit, unit, unit_label(unit));
                        }
                    });
                if ui.button(texts::REMOVE).clicked() {
                    removed = Some(i);
                }
            });
        }
        if let Some(i) = removed {
            fields.remove(i);
        }
        ui.horizontal(|ui| {
            if fields.len() < MOST_REMINDERS && ui.button(texts::ADD_NOTIFICATION).clicked() {
                fields.push(ReminderField::NEW);
            }
            if editor.reminders.is_some() && ui.button(texts::USE_CALENDAR_DEFAULTS).clicked() {
                editor.reminders = None;
            } else if fields != shown {
                editor.reminders = Some(fields);
            }
        });
    });
}

/// One strip per guest, their busy times on the day filled in, and the chosen time highlighted
/// across all strips (`MOD-06-U02`); beside each, the guest, and whether they are busy then or
/// share no availability (`MOD-06-F19`, `F20`).
fn availability(ui: &mut egui::Ui, preview: &Preview) {
    let strips = ui
        .vertical(|ui| {
            let mut strips = Vec::new();
            for strip in &preview.strips {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(STRIP, Sense::hover());
                    let visuals = ui.visuals();
                    let painter = ui.painter();
                    painter.rect_filled(rect, CornerRadius::same(2), visuals.extreme_bg_color);
                    for &(start, end) in strip.busy.iter().flatten() {
                        let x = |part: f32| rect.left() + part * rect.width();
                        let busy = Rect::from_x_y_ranges(x(start)..=x(end), rect.y_range());
                        painter.rect_filled(busy, 0.0, visuals.widgets.inactive.fg_stroke.color);
                    }
                    strips.push(rect);
                    ui.label(&strip.label);
                    if strip.busy.is_none() {
                        ui.weak(NO_DATA);
                    } else if strip.conflict {
                        ui.colored_label(ui.visuals().warn_fg_color, CONFLICT);
                    }
                });
            }
            strips
        })
        .inner;
    if let (Some(first), Some(last)) = (strips.first(), strips.last()) {
        let x = |part: f32| first.left() + part * first.width();
        let (start, end) = preview.chosen;
        // A moment still shows.
        let chosen = Rect::from_x_y_ranges(
            x(start)..=x(end).max(x(start) + 2.0),
            first.top()..=last.bottom(),
        );
        let selection = ui.visuals().selection;
        let painter = ui.painter();
        painter.rect_filled(chosen, 0.0, selection.bg_fill.gamma_multiply(0.4));
        painter.rect_stroke(
            chosen,
            0.0,
            Stroke::new(2.0, selection.stroke.color),
            StrokeKind::Outside,
        );
    }
}

/// A date or time field of the editor; says whether it changed.
fn field(ui: &mut egui::Ui, text: &mut String, width: f32) -> bool {
    ui.add(egui::TextEdit::singleline(text).desired_width(width))
        .changed()
}

/// What Google gave the editor: the people suggested as guests, and the guests' availability on
/// the day chosen, once known.
pub struct FromGoogle<'a> {
    pub suggested: &'a [Person],
    pub availability: Option<&'a Preview>,
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
/// the app does not (`DEC-111`). The people suggested are offered as guests, and the guests'
/// availability is shown under the time.
pub fn show(
    ui: &mut egui::Ui,
    editor: &mut Editor,
    regional: &Regional,
    choices: &[(String, Vec<Calendar>)],
    accounts: &[Entry],
    refused: Option<&str>,
    google: FromGoogle,
) -> Option<Edited> {
    let own = editor.own_addresses(accounts);
    let mut open_series = false;
    let kinds = editor.kinds(accounts);
    egui::Grid::new("editor")
        .num_columns(2)
        .spacing([8.0, 6.0])
        .show(ui, |ui| {
            // As Google Calendar, where the calendar allows them; an existing event keeps its own.
            if !kinds.is_empty() {
                ui.label(texts::TYPE);
                ui.horizontal(|ui| {
                    for kind in kinds {
                        ui.selectable_value(&mut editor.kind, kind, kind.label());
                    }
                });
                ui.end_row();
            } else if editor.kind != Kind::Ordinary {
                ui.label(texts::TYPE);
                ui.label(editor.kind.label());
                ui.end_row();
            }

            ui.label(texts::TITLE);
            ui.add(
                egui::TextEdit::singleline(&mut editor.title)
                    .hint_text(texts::ADD_TITLE)
                    .desired_width(FIELD_WIDTH),
            );
            ui.end_row();

            ui.label(texts::STARTS);
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

            ui.label(texts::ENDS);
            ui.horizontal(|ui| {
                field(ui, &mut editor.end_date, DATE_WIDTH);
                if !editor.all_day {
                    field(ui, &mut editor.end_time, TIME_WIDTH);
                }
            });
            ui.end_row();

            ui.label("");
            ui.checkbox(&mut editor.all_day, texts::ALL_DAY);
            ui.end_row();

            if !editor.all_day {
                ui.label(texts::TIME_ZONE);
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

            if let Some(preview) = google.availability {
                ui.label(texts::AVAILABILITY);
                availability(ui, preview);
                ui.end_row();
            }

            // An existing event's repetition is not changed here (`DEC-111`).
            match &editor.opened {
                None => repeat(ui, editor, regional),
                Some((event, _)) if event.series.is_some() => {
                    ui.label(texts::REPEATS);
                    ui.horizontal(|ui| {
                        ui.label(texts::CHANGED_IN_GOOGLE);
                        open_series = ui.button(texts::OPEN_IN_GOOGLE).clicked();
                    });
                    ui.end_row();
                }
                Some(_) => {}
            }

            ui.label(texts::LOCATION);
            ui.add(
                egui::TextEdit::singleline(&mut editor.location)
                    .hint_text(texts::ADD_LOCATION)
                    .desired_width(FIELD_WIDTH),
            );
            ui.end_row();

            // Google takes no guests for out of office and focus time, nor a Meet call.
            if !editor.kind.timed_only() {
                ui.label(texts::GUESTS);
                guests(ui, editor, google.suggested, &own);
                ui.end_row();

                ui.label("");
                ui.checkbox(&mut editor.meet, texts::ADD_GOOGLE_MEET);
                ui.end_row();

                ui.label(texts::NOTIFICATIONS);
                reminders(ui, editor);
                ui.end_row();
            }

            ui.label(texts::DESCRIPTION);
            ui.add(
                egui::TextEdit::multiline(&mut editor.description)
                    .hint_text(texts::ADD_DESCRIPTION)
                    .desired_width(FIELD_WIDTH),
            );
            ui.end_row();

            ui.label(texts::CALENDAR);
            calendar_list::picker(ui, "editor-calendar", choices, &mut editor.calendar);
            ui.end_row();

            ui.label(texts::COLOUR);
            event_color(ui, &mut editor.color);
            ui.end_row();

            // Out of office and focus time are always busy.
            if !editor.kind.timed_only() {
                ui.label(texts::SHOW_AS);
                egui::ComboBox::from_id_salt("editor-show")
                    .selected_text(show_label(editor.show))
                    .show_ui(ui, |ui| {
                        for show in [Show::Busy, Show::Free] {
                            ui.selectable_value(&mut editor.show, show, show_label(show));
                        }
                    });
                ui.end_row();
            }

            ui.label(texts::VISIBILITY);
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
        .add_enabled(problem.is_none(), crate::widgets::primary(ui, texts::SAVE))
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
