//! Draws a `MiniCalendar` with egui. Which days it shows and marks is decided in
//! `calendar_core::navigation`.

use calendar_core::model::Rgb;
use calendar_core::navigation::MiniCalendar;
use calendar_core::regional::Regional;
use calendar_core::texts;
use chrono::{Months, NaiveDate};
use eframe::egui::{self, RichText};

use crate::grid_view::text_on;
use crate::icons::{self, Icon};

const DAY_SIZE: [f32; 2] = [34.0, 34.0];
/// How far a day's number sits above the middle of its cell, leaving room for its dot.
const NUMBER_RAISED: f32 = 4.0;
/// How far below the middle of a day's cell its dot sits.
const DOT_BELOW: f32 = 10.0;

/// What the user did in the mini calendar.
pub enum Action {
    /// Browse to the month starting on this date, without moving the view.
    Browse(NaiveDate),
    /// Show this date in the view.
    GoTo(NaiveDate),
}

pub fn show(ui: &mut egui::Ui, mini: &MiniCalendar, regional: &Regional) -> Option<Action> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(regional.month_year(mini.month)).font(crate::appearance::strong(15.0)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if icons::button(ui, Icon::Next, texts::NEXT_MONTH, true).clicked() {
                action = Some(Action::Browse(mini.month + Months::new(1)));
            }
            if icons::button(ui, Icon::Previous, texts::PREVIOUS_MONTH, true).clicked() {
                action = Some(Action::Browse(mini.month - Months::new(1)));
            }
        });
    });
    egui::Grid::new("mini-calendar")
        .spacing([0.0, 0.0])
        .show(ui, |ui| {
            for weekday in regional.weekday_letters() {
                ui.add_sized(
                    [DAY_SIZE[0], 24.0],
                    egui::Label::new(RichText::new(weekday).small().weak()),
                );
            }
            ui.end_row();
            for week in &mini.weeks {
                // The period shown is one rounded band under its days in the week, rather than a
                // box a day, and today a circle in the accent, as in Google Calendar; both are
                // painted under the days, which have no frames of their own.
                let band_slot = ui.painter().add(egui::Shape::Noop);
                let today_slot = ui.painter().add(egui::Shape::Noop);
                let mut cells = Vec::new();
                for day in week {
                    let mut text = RichText::new(day.date.format("%-d").to_string()).size(12.5);
                    // A day of the period shown, or one under the pointer, is drawn on a fill the
                    // accent and weak text do not read on (`MOD-04-NFR-01`); egui hovers a day
                    // also from just outside it.
                    let near = ui.style().interaction.interact_radius;
                    let at = egui::Rect::from_min_size(ui.cursor().min, DAY_SIZE.into());
                    let plain = day.in_period || ui.rect_contains_pointer(at.expand(near));
                    let accent = ui.visuals().selection.stroke.color;
                    let ink = text_on(Rgb(accent.r(), accent.g(), accent.b()));
                    if day.is_today {
                        text = text.strong().color(ink);
                    } else if !day.in_month {
                        // The neighbouring months' days smaller, so the month stands out, and
                        // still readable (`MOD-04-NFR-01`).
                        text = text.size(11.0);
                        if !plain {
                            text = text.weak();
                        }
                    }
                    // Drawn here rather than as a button, so the number sits a little high and
                    // its dot has room under it, inside the band and today's circle.
                    let (rect, response) =
                        ui.allocate_exact_size(DAY_SIZE.into(), egui::Sense::click());
                    let number = text.text().to_string();
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::Button, true, &number)
                    });
                    let visuals = ui.visuals();
                    let center = rect.center();
                    let radius = DAY_SIZE[1] / 2.0 - 2.0;
                    if day.is_today {
                        // Today keeps its circle under the pointer, ringed rather than framed.
                        let mut circle = vec![egui::Shape::circle_filled(center, radius, accent)];
                        if response.hovered() || response.has_focus() {
                            let ring = egui::Stroke::new(1.5, accent);
                            circle.push(egui::Shape::circle_stroke(center, radius + 2.5, ring));
                        }
                        ui.painter().set(today_slot, circle);
                    } else if response.hovered() || response.has_focus() {
                        let hover = rect.shrink2(egui::vec2(1.0, 2.0));
                        let fill = visuals.widgets.hovered.weak_bg_fill;
                        ui.painter().rect_filled(hover, radius, fill);
                    }
                    let color = visuals.text_color();
                    let galley = egui::WidgetText::from(text).into_galley(
                        ui,
                        Some(egui::TextWrapMode::Extend),
                        f32::INFINITY,
                        egui::TextStyle::Body,
                    );
                    let at = center - egui::vec2(0.0, NUMBER_RAISED) - galley.size() / 2.0;
                    ui.painter().galley(at, galley, color);
                    // A dot under a day with events (`MOD-03-F25`).
                    if day.has_events {
                        // In the colour of its number.
                        let color = if day.is_today {
                            ink
                        } else if !day.in_month && !plain {
                            ui.visuals().weak_text_color()
                        } else {
                            ui.visuals().text_color()
                        };
                        let dot = center + egui::vec2(0.0, DOT_BELOW);
                        ui.painter().circle_filled(dot, 1.5, color);
                    }
                    cells.push(response.rect);
                    if response.clicked() {
                        action = Some(Action::GoTo(day.date));
                    }
                }
                // Measured from where the days start, evenly apart, as egui widens some days.
                let first = week.iter().position(|day| day.in_period);
                let last = week.iter().rposition(|day| day.in_period);
                if let (Some(first), Some(last)) = (first, last) {
                    let pitch = cells[1].left() - cells[0].left();
                    let x =
                        |column: usize| cells[0].left() + column as f32 * pitch + DAY_SIZE[0] / 2.0;
                    let band = egui::Rect::from_x_y_ranges(
                        x(first) - pitch / 2.0..=x(last) + pitch / 2.0,
                        cells[first].y_range(),
                    )
                    .shrink2(egui::vec2(1.0, 2.0));
                    let fill = ui.visuals().selection.bg_fill;
                    let shape = egui::Shape::rect_filled(band, band.height() / 2.0, fill);
                    ui.painter().set(band_slot, shape);
                }
                ui.end_row();
            }
        });
    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen;
    use calendar_core::contrast::AA;
    use calendar_core::fixtures::d;
    use calendar_core::navigation::{Navigation, View, mini_calendar};
    use calendar_core::theme::Scheme;
    use calendar_core::week::GridView;
    use chrono::Weekday;

    #[test]
    fn its_days_read_at_4_5_also_under_the_pointer_in_both_themes() {
        // November 2026, today the 11th, the week of the 23rd shown.
        let navigation = Navigation {
            view: View::Grid(GridView::Week),
            selected: d(2026, 11, 23),
            week_start: Weekday::Mon,
        };
        let mini = mini_calendar(d(2026, 11, 1), &navigation, d(2026, 11, 11));
        let regional = Regional::from_env(|_| None);
        let drawn = |scheme, pointer| {
            screen::drawn(egui::vec2(400.0, 400.0), 1.0, scheme, pointer, |ui| {
                show(ui, &mini, &regional);
            })
        };

        for scheme in [Scheme::Light, Scheme::Dark] {
            // Days of October, the first at the left edge, today, a day of the week shown and a
            // plain day.
            for day in ["26", "31", "11", "24", "5"] {
                let is_day = |text: &str| text == day;
                let at = screen::text_where(&drawn(scheme, None), is_day)
                    .visual_bounding_rect()
                    .center();
                // On the day, and just left of it, from where egui hovers it too.
                let outside = at - egui::vec2(DAY_SIZE[0] / 2.0 + 2.0, 0.0);
                for pointer in [None, Some(at), Some(outside)] {
                    let ratio = screen::contrast_where(&drawn(scheme, pointer), is_day);
                    assert!(ratio >= AA, "{scheme:?}, {day}, {pointer:?}: {ratio}");
                }
            }
        }
    }
}
