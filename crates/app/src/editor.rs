//! Draws the full event editor (`US-05-02`): the title, start and end, all-day, time zone,
//! location, description, calendar, busy or free and visibility of a new or an existing event,
//! and how it repeats, through Google Calendar's repeat menu and custom recurrence dialog
//! (`US-05-06`, `US-05-11`); a series' rule the dialog cannot express opens in Google Calendar
//! (`US-05-07`); out of office and focus time where the account allows them (`US-05-08`). What
//! the fields mean and what saving does is `calendar_core::editor`'s. The event's own colour is
//! picked from Google's event colours (`US-05-09`). Guests are added by address or from the people
//! suggested while typing, marked optional and removed (`US-06-01`); the "Add Google Meet" switch
//! turns on with the first guest (`US-06-07`). Under the time, the guests' availability that day
//! (`US-06-08`). The event's reminders, starting from its calendar's defaults (`US-07-04`).

use calendar_core::accounts::Entry;
use calendar_core::availability::Preview;
use calendar_core::colors::EventColor;
use calendar_core::editor::{Editor, LONGEST_REMINDER, MOST_REMINDERS, ReminderField, Save, Unit};
use calendar_core::guests::Person;
use calendar_core::model::{Calendar, Kind, Method, Show, Visibility};
use calendar_core::regional::Regional;
use calendar_core::repeat::{self, Ending, Frequency};
use calendar_core::texts::{self, CONFLICT, NO_DATA};
use eframe::egui::{self, CornerRadius, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::calendar_list;
use crate::grid_view::{color, text_on};
use crate::widgets::{close_button, form_row, section, text_field};

const DATE_WIDTH: f32 = 100.0;
const TIME_WIDTH: f32 = 70.0;
const COUNT_WIDTH: f32 = 40.0;
/// The width of the custom recurrence dialog.
const CUSTOM_WIDTH: f32 = 320.0;
const SWATCH: f32 = 20.0;
/// How tall the list of time zones grows before it scrolls.
const ZONES_HEIGHT: f32 = 320.0;
/// Remembers that the user clicked the formatted description and is asked whether to edit it as
/// plain text.
pub const ASKING: &str = "editor-description-asking";
/// The height of a guest's strip of the day in the availability preview.
const STRIP: f32 = 12.0;
/// The room between two guests in the availability preview.
const GUESTS_APART: f32 = 8.0;

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

fn ending_label(ends: Ending) -> &'static str {
    match ends {
        Ending::Never => texts::NEVER,
        Ending::OnDate => texts::ON,
        Ending::AfterCount => texts::AFTER,
    }
}

/// Google Calendar's repeat menu (`MOD-05-F32`): "Does not repeat" for a new event, the presets
/// for the start's day, the rule chosen in words when it is none of them (`MOD-05-F34`), and
/// "Custom…", which opens the custom recurrence dialog (`MOD-05-F33`).
fn repeat(ui: &mut egui::Ui, editor: &mut Editor, regional: &Regional) {
    let choices = editor.repeat_choices(regional);
    let chosen = editor.repeat_label(regional);
    form_row(ui, texts::REPEATS, |ui| {
        egui::ComboBox::from_id_salt("editor-repeat")
            .selected_text(&chosen)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (rule, words) in choices {
                    let is_chosen = words == chosen;
                    if ui.selectable_label(is_chosen, &words).clicked() && !is_chosen {
                        editor.repeat = rule;
                    }
                }
                if ui.selectable_label(false, texts::CUSTOM).clicked() {
                    editor.open_custom(regional);
                }
            });
    });
    custom(ui, editor, regional);
}

/// Google Calendar's custom recurrence dialog over the editor, while open (`MOD-05-F33`): repeat
/// every N days, weeks, months or years, on the weekdays for weeks and on a day of the month for
/// months, the other units showing neither (`MOD-05-U05`), and how the series ends. **Done**
/// takes the rule, or says why it cannot; **Cancel**, Esc or a click outside leave the rule as it
/// was.
fn custom(ui: &egui::Ui, editor: &mut Editor, regional: &Regional) {
    let Some(custom) = &mut editor.custom else {
        return;
    };
    let (mut done, mut cancelled) = (false, false);
    let modal = egui::Modal::new(egui::Id::unique("editor-custom")).show(ui.ctx(), |ui| {
        ui.set_width(CUSTOM_WIDTH);
        crate::widgets::heading(ui, texts::CUSTOM_RECURRENCE);
        ui.horizontal(|ui| {
            ui.label(texts::REPEAT_EVERY);
            field(ui, &mut custom.interval, COUNT_WIDTH);
            let mut frequency = custom.frequency();
            egui::ComboBox::from_id_salt("custom-unit")
                .selected_text(frequency.label())
                .show_ui(ui, |ui| {
                    for each in Frequency::EACH {
                        ui.selectable_value(&mut frequency, each, each.label());
                    }
                });
            if frequency != custom.frequency() {
                custom.set_frequency(frequency);
            }
        });
        match custom.frequency() {
            Frequency::Weekly => {
                ui.label(texts::REPEAT_ON);
                ui.horizontal(|ui| {
                    let mut day = regional.week_start;
                    for letter in regional.weekday_letters() {
                        let chosen = custom.days().contains(&day);
                        let clicked = (ui.selectable_label(chosen, letter))
                            .on_hover_text(repeat::day_name(day))
                            .clicked();
                        if clicked {
                            custom.toggle_day(day);
                        }
                        day = day.succ();
                    }
                });
            }
            Frequency::Monthly => {
                let choices = custom.month_days();
                let chosen = (choices.iter())
                    .find(|(month_day, _)| *month_day == custom.month_day)
                    .map(|(_, words)| words.clone())
                    .unwrap_or_default();
                egui::ComboBox::from_id_salt("custom-month-day")
                    .selected_text(chosen)
                    .show_ui(ui, |ui| {
                        for (month_day, words) in choices {
                            ui.selectable_value(&mut custom.month_day, month_day, words);
                        }
                    });
            }
            Frequency::Daily | Frequency::Yearly => {}
        }
        ui.add_space(4.0);
        ui.label(texts::SERIES_ENDS);
        for ends in [Ending::Never, Ending::OnDate, Ending::AfterCount] {
            ui.horizontal(|ui| {
                ui.radio_value(&mut custom.ends, ends, ending_label(ends));
                match ends {
                    Ending::Never => {}
                    Ending::OnDate => {
                        field(ui, &mut custom.last_date, DATE_WIDTH);
                    }
                    Ending::AfterCount => {
                        field(ui, &mut custom.count, COUNT_WIDTH);
                        ui.label(texts::OCCURRENCES);
                    }
                }
            });
        }
        if let Some(problem) = &custom.problem {
            ui.colored_label(ui.visuals().warn_fg_color, problem);
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            cancelled = ui.button(texts::CANCEL).clicked();
            done = ui.add(crate::widgets::primary(ui, texts::DONE)).clicked();
        });
    });
    if done {
        editor.custom_done(regional);
    } else if cancelled || modal.should_close() {
        editor.custom = None;
    }
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
        let field = text_field(
            ui,
            egui::TextEdit::singleline(&mut editor.guest_field)
                .hint_text(texts::ADD_GUESTS)
                .desired_width(f32::INFINITY),
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
        // Names from a description that fit several people or nobody, each with its own
        // suggestions; nobody is invited until picked (`MOD-10-F12`).
        let (mut picked, mut dropped) = (None, None);
        for (index, name) in editor.unresolved.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.colored_label(ui.visuals().warn_fg_color, &name.name);
                ui.weak(texts::PICK_WHO);
                let hint = texts::drop_name(&name.name);
                let remove = close_button(ui, &hint);
                remove.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &hint));
                if remove.clicked() {
                    dropped = Some(index);
                }
            });
            ui.indent(("unresolved", index), |ui| {
                if name.suggested.is_empty() {
                    ui.weak(texts::NOBODY_FITS);
                }
                for person in &name.suggested {
                    if ui.button(person.label()).clicked() {
                        picked = Some((index, person.clone()));
                    }
                }
            });
        }
        if let Some((index, person)) = picked {
            editor.pick(index, &person, own);
        } else if let Some(index) = dropped {
            editor.drop_unresolved(index);
        }
        let mut removed = None;
        // A guest a line, their switches lined up in columns.
        ui.spacing_mut().interact_size.y = 24.0;
        ui.spacing_mut().button_padding.y = 2.0;
        egui::Grid::new("editor-guests")
            .num_columns(3)
            .spacing([12.0, 2.0])
            .show(ui, |ui| {
                for guest in editor.listed_guests() {
                    let shown = Person {
                        name: guest.name.clone(),
                        email: guest.email.clone(),
                    };
                    let name = ui.label(shown.label());
                    // The row under the pointer, from the guest to the row's end.
                    let row = egui::Rect::from_x_y_ranges(
                        name.rect.left()..=ui.max_rect().right(),
                        name.rect.y_range(),
                    );
                    let hovered = ui.rect_contains_pointer(row.expand2(egui::vec2(0.0, 2.0)));
                    optional_switch(ui, &mut guest.optional, hovered);
                    if close_button(ui, texts::REMOVE).clicked() {
                        removed = Some(guest.email.clone());
                    }
                    ui.end_row();
                }
            });
        if let Some(email) = removed {
            editor.remove_guest(&email);
        }
    });
}

/// Whether a guest is optional, as Google Calendar shows it: a person, filled, while the guest's
/// row is under the pointer; outlined, and there all the time, once the guest is optional. Its
/// hint says what a click does. The button keeps its place, for the keyboard and so nothing
/// moves.
fn optional_switch(ui: &mut egui::Ui, optional: &mut bool, row_hovered: bool) {
    let hint = if *optional {
        texts::MARK_REQUIRED
    } else {
        texts::MARK_OPTIONAL
    };
    let response = ui
        .add(
            egui::Button::new("")
                .min_size(egui::vec2(24.0, 20.0))
                .frame_when_inactive(false),
        )
        .on_hover_text(hint);
    let on = *optional;
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::CheckBox, true, on, texts::OPTIONAL)
    });
    if on || row_hovered || response.has_focus() {
        let color = ui.visuals().widgets.style(&response).text_color();
        person(ui.painter(), response.rect.center(), color, !on);
    }
    if response.clicked() {
        *optional = !*optional;
    }
}

/// A person, a head over shoulders, about 14 points across, filled or outlined.
fn person(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32, filled: bool) {
    let head = center - egui::vec2(0.0, 4.0);
    let body = Rect::from_center_size(center + egui::vec2(0.0, 4.5), egui::vec2(13.0, 7.0));
    let shoulders = egui::CornerRadius {
        nw: 6,
        ne: 6,
        sw: 1,
        se: 1,
    };
    if filled {
        painter.circle_filled(head, 3.5, color);
        painter.rect_filled(body, shoulders, color);
    } else {
        let line = Stroke::new(1.4, color);
        painter.circle_stroke(head, 3.0, line);
        painter.rect_stroke(body, shoulders, line, StrokeKind::Inside);
    }
}

/// The event's reminders, each a notification or an e-mail so many minutes, hours or days before,
/// and removable; "Add notification" up to Google's limit, and "Use calendar defaults" for an
/// event with reminders of its own (`MOD-07-F04`, `F05`). Changing the calendar's defaults shown
/// makes them the event's own, as in Google Calendar.
fn reminders(ui: &mut egui::Ui, editor: &mut Editor) {
    let shown = editor.reminder_fields();
    let mut fields = shown.clone();
    let methods = editor.reminder_methods();
    ui.vertical(|ui| {
        reminder_rows(ui, &mut fields, methods);
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

/// One row per reminder: how it reaches the user, among `methods`, and how many minutes, hours or
/// days before, and "Remove". Also a local calendar's default notifications (`MOD-09-F14`).
/// Whether an edit was finished: a choice picked, a row removed, or a number left or let go of.
pub fn reminder_rows(
    ui: &mut egui::Ui,
    fields: &mut Vec<ReminderField>,
    methods: &[Method],
) -> bool {
    let mut finished = false;
    let mut removed = None;
    for (i, field) in fields.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt(("editor-reminder-method", i))
                .selected_text(method_label(field.method))
                .show_ui(ui, |ui| {
                    for &method in methods {
                        finished |=
                            (ui.selectable_value(&mut field.method, method, method_label(method)))
                                .changed();
                    }
                });
            // A number too large for the unit picked stays, and Save says why.
            let most = LONGEST_REMINDER / field.unit.minutes();
            let amount = ui.add(
                egui::DragValue::new(&mut field.amount)
                    .range(0..=most)
                    .clamp_existing_to_range(false),
            );
            finished |= amount.drag_stopped() || amount.lost_focus();
            egui::ComboBox::from_id_salt(("editor-reminder-unit", i))
                .selected_text(unit_label(field.unit))
                .show_ui(ui, |ui| {
                    for unit in Unit::ALL {
                        finished |= (ui.selectable_value(&mut field.unit, unit, unit_label(unit)))
                            .changed();
                    }
                });
            if ui.button(texts::REMOVE).clicked() {
                removed = Some(i);
            }
        });
    }
    if let Some(i) = removed {
        fields.remove(i);
        finished = true;
    }
    finished
}

/// One strip per guest, their busy times on the day filled in, and the chosen time highlighted
/// on each (`MOD-06-U02`); above each, the guest in full, and whether they are busy then or share
/// no availability (`MOD-06-F19`, `F20`).
fn availability(ui: &mut egui::Ui, preview: &Preview) {
    // The first guest level with the row's label; each close to their strip, the guests apart.
    let text = ui.text_style_height(&egui::TextStyle::Body);
    ui.add_space(((ui.spacing().interact_size.y - text) / 2.0).max(0.0));
    ui.spacing_mut().item_spacing.y = 3.0;
    for (i, strip) in preview.strips.iter().enumerate() {
        if i > 0 {
            ui.add_space(GUESTS_APART);
        }
        // The guest and their state as one text, wrapping close above their strip.
        let state = match (&strip.busy, strip.conflict) {
            (None, _) => Some((NO_DATA, ui.visuals().weak_text_color())),
            (Some(_), true) => Some((CONFLICT, ui.visuals().warn_fg_color)),
            (Some(_), false) => None,
        };
        let mut text = egui::text::LayoutJob::default();
        let style = ui.style();
        let mut add = |rich: egui::RichText| {
            rich.append_to(
                &mut text,
                style,
                egui::FontSelection::Default,
                egui::Align::Center,
            )
        };
        add(egui::RichText::new(&strip.label));
        if let Some((state, color)) = state {
            add(egui::RichText::new(format!("  {state}")).color(color));
        }
        ui.add(egui::Label::new(text).wrap());
        let size = Vec2::new(ui.available_width(), STRIP);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let x = |part: f32| rect.left() + part * rect.width();
        let visuals = ui.visuals();
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(2), visuals.extreme_bg_color);
        for &(start, end) in strip.busy.iter().flatten() {
            let busy = Rect::from_x_y_ranges(x(start)..=x(end), rect.y_range());
            painter.rect_filled(busy, 0.0, visuals.widgets.inactive.fg_stroke.color);
        }
        let (start, end) = preview.chosen;
        // A moment still shows.
        let chosen = Rect::from_x_y_ranges(x(start)..=x(end).max(x(start) + 2.0), rect.y_range());
        let selection = visuals.selection;
        painter.rect_filled(chosen, 0.0, selection.bg_fill.gamma_multiply(0.4));
        painter.rect_stroke(
            chosen,
            0.0,
            Stroke::new(2.0, selection.stroke.color),
            StrokeKind::Outside,
        );
    }
    ui.add_space(GUESTS_APART / 2.0);
}

/// The description: a field, or, while it keeps its formatting from Google, the formatted text,
/// which asks before it is turned into plain text to be edited, as Google Calendar's mobile app
/// does (`MOD-05-F31`).
fn description(ui: &mut egui::Ui, editor: &mut Editor) {
    if !editor.keeps_formatting() {
        text_field(
            ui,
            egui::TextEdit::multiline(&mut editor.description)
                .hint_text(texts::ADD_DESCRIPTION)
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );
        return;
    }
    let asking = egui::Id::unique(ASKING);
    let (line, _) = crate::appearance::field_lines(ui.visuals());
    let shown = egui::Frame::new()
        .stroke(Stroke::new(1.0, line))
        .corner_radius(CornerRadius::same(crate::appearance::RADIUS))
        .inner_margin(ui.spacing().button_padding)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Read here, not followed: a click anywhere asks to edit it.
            crate::widgets::rich_text(ui, &calendar_core::rich::lines(&editor.description));
        });
    let clicked = (shown.response.interact(Sense::click()))
        .on_hover_cursor(egui::CursorIcon::Text)
        .on_hover_text(texts::EDIT_FORMATTED_DESCRIPTION)
        .clicked();
    if clicked {
        ui.data_mut(|data| data.insert_temp(asking, true));
    }
    if ui.data(|data| data.get_temp(asking).unwrap_or(false)) {
        crate::widgets::card(ui, |ui| {
            ui.label(texts::FORMATTING_WILL_BE_LOST);
            ui.horizontal(|ui| {
                if ui
                    .add(crate::widgets::primary(ui, texts::EDIT_AS_PLAIN_TEXT))
                    .clicked()
                {
                    editor.edit_description_as_text();
                    ui.data_mut(|data| data.remove::<bool>(asking));
                }
                if ui.button(texts::CANCEL).clicked() {
                    ui.data_mut(|data| data.remove::<bool>(asking));
                }
            });
        });
    }
}

/// A date or time field of the editor; says whether it changed.
fn field(ui: &mut egui::Ui, text: &mut String, width: f32) -> bool {
    text_field(ui, egui::TextEdit::singleline(text).desired_width(width)).changed()
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
/// unavailable. An occurrence of a series whose rule the custom dialog cannot express offers to
/// change it in Google Calendar (`MOD-05-F29`). The people suggested are offered as guests, and the guests'
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
    // Its drop-downs and buttons outlined as its fields are.
    crate::widgets::outlined_controls(ui);
    let own = editor.own_addresses(accounts);
    let mut open_series = false;
    let kinds = editor.kinds(accounts);
    // The event as it stands when this frame starts; the fields below change it for the next.
    let saved = editor.save(regional);
    let problem = refused
        .map(str::to_string)
        .or_else(|| saved.as_ref().err().cloned());
    // The title first and large, without a label, and Save beside it, always in sight at the top,
    // as in Google Calendar.
    let (_, clicked) = egui::Sides::new().shrink_left().show(
        ui,
        |ui| {
            text_field(
                ui,
                egui::TextEdit::singleline(&mut editor.title)
                    .hint_text(texts::ADD_TITLE)
                    .font(crate::appearance::strong(18.0))
                    .desired_width(f32::INFINITY),
            );
        },
        |ui| {
            let save = crate::widgets::primary(ui, texts::SAVE);
            ui.add_enabled(problem.is_none(), save).clicked()
        },
    );
    // Why it cannot be saved yet, under the title.
    if let Some(problem) = &problem {
        ui.colored_label(ui.visuals().warn_fg_color, problem);
    }
    // As Google Calendar, where the calendar allows them; an existing event keeps its own.
    if !kinds.is_empty() {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            for kind in kinds {
                ui.selectable_value(&mut editor.kind, kind, kind.label());
            }
        });
    } else if editor.kind != Kind::Ordinary {
        form_row(ui, texts::TYPE, |ui| ui.label(editor.kind.label()));
    }
    section(ui);

    // When.
    form_row(ui, texts::STARTS, |ui| {
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
    });
    form_row(ui, texts::ENDS, |ui| {
        ui.horizontal(|ui| {
            field(ui, &mut editor.end_date, DATE_WIDTH);
            if !editor.all_day {
                field(ui, &mut editor.end_time, TIME_WIDTH);
            }
        });
    });
    form_row(ui, "", |ui| {
        ui.checkbox(&mut editor.all_day, texts::ALL_DAY)
    });
    if !editor.all_day {
        form_row(ui, texts::TIME_ZONE, |ui| {
            egui::ComboBox::from_id_salt("editor-zone")
                .selected_text(editor.zone.name())
                .height(ZONES_HEIGHT)
                .show_ui(ui, |ui| {
                    for zone in chrono_tz::TZ_VARIANTS {
                        ui.selectable_value(&mut editor.zone, zone, zone.name());
                    }
                });
        });
    }
    if let Some(preview) = google.availability {
        form_row(ui, texts::AVAILABILITY, |ui| availability(ui, preview));
    }
    // An existing event not of a series is not made one here (`DEC-111`); a series' rule the
    // custom dialog cannot express is changed in Google Calendar (`MOD-05-F29`).
    let series = matches!(&editor.opened, Some((event, _)) if event.series.is_some());
    if editor.rule_kept() {
        form_row(ui, texts::REPEATS, |ui| {
            ui.horizontal(|ui| {
                ui.label(texts::CHANGED_IN_GOOGLE);
                open_series = ui.button(texts::OPEN_IN_GOOGLE).clicked();
            });
        });
    } else if editor.opened.is_none() || series {
        repeat(ui, editor, regional);
    }
    section(ui);

    // Where and with whom.
    form_row(ui, texts::LOCATION, |ui| {
        text_field(
            ui,
            egui::TextEdit::singleline(&mut editor.location)
                .hint_text(texts::ADD_LOCATION)
                .desired_width(f32::INFINITY),
        );
    });
    // Google takes no guests for out of office and focus time, nor a Meet call; a local
    // calendar has neither (`MOD-09-F10`).
    if !editor.kind.timed_only() {
        if editor.google() {
            form_row(ui, texts::GUESTS, |ui| {
                guests(ui, editor, google.suggested, &own)
            });
            form_row(ui, "", |ui| {
                ui.checkbox(&mut editor.meet, texts::ADD_GOOGLE_MEET)
            });
        }
        form_row(ui, texts::NOTIFICATIONS, |ui| reminders(ui, editor));
    }
    form_row(ui, texts::DESCRIPTION, |ui| description(ui, editor));
    section(ui);

    // How it shows.
    form_row(ui, texts::CALENDAR, |ui| {
        calendar_list::picker(ui, "editor-calendar", choices, &mut editor.calendar);
    });
    form_row(ui, texts::COLOUR, |ui| event_color(ui, &mut editor.color));
    // Out of office and focus time are always busy; a local event has neither busy or free nor
    // visibility (`MOD-09-F10`).
    if editor.google() {
        if !editor.kind.timed_only() {
            form_row(ui, texts::SHOW_AS, |ui| {
                egui::ComboBox::from_id_salt("editor-show")
                    .selected_text(show_label(editor.show))
                    .show_ui(ui, |ui| {
                        for show in [Show::Busy, Show::Free] {
                            ui.selectable_value(&mut editor.show, show, show_label(show));
                        }
                    });
            });
        }
        form_row(ui, texts::VISIBILITY, |ui| {
            egui::ComboBox::from_id_salt("editor-visibility")
                .selected_text(visibility_label(editor.visibility))
                .show_ui(ui, |ui| {
                    for visibility in [Visibility::Default, Visibility::Public, Visibility::Private]
                    {
                        let label = visibility_label(visibility);
                        ui.selectable_value(&mut editor.visibility, visibility, label);
                    }
                });
        });
    }
    if open_series {
        return Some(Edited::OpenSeries);
    }
    if clicked {
        saved.ok().map(Edited::Save)
    } else {
        None
    }
}
