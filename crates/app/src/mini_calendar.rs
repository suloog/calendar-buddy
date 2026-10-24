//! Draws a `MiniCalendar` with egui. Which days it shows and marks is decided in
//! `calendar_core::navigation`.

use calendar_core::navigation::MiniCalendar;
use calendar_core::regional::Regional;
use calendar_core::texts;
use chrono::{Months, NaiveDate};
use eframe::egui::{self, RichText};

const DAY_SIZE: [f32; 2] = [34.0, 28.0];

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
            if crate::widgets::icon_button(ui, "›", texts::NEXT_MONTH).clicked() {
                action = Some(Action::Browse(mini.month + Months::new(1)));
            }
            if crate::widgets::icon_button(ui, "‹", texts::PREVIOUS_MONTH).clicked() {
                action = Some(Action::Browse(mini.month - Months::new(1)));
            }
        });
    });
    egui::Grid::new("mini-calendar")
        .spacing([0.0, 0.0])
        .show(ui, |ui| {
            for weekday in regional.weekday_letters() {
                ui.add_sized(
                    DAY_SIZE,
                    egui::Label::new(RichText::new(weekday).small().weak()),
                );
            }
            ui.end_row();
            for week in &mini.weeks {
                for day in week {
                    let mut text = RichText::new(day.date.format("%-d").to_string()).size(12.5);
                    // A day of the period shown, or one under the pointer, is drawn on a fill the
                    // accent and weak text do not read on (`MOD-04-NFR-01`); egui hovers a day
                    // also from just outside it.
                    let near = ui.style().interaction.interact_radius;
                    let at = egui::Rect::from_min_size(ui.cursor().min, DAY_SIZE.into());
                    let plain = day.in_period || ui.rect_contains_pointer(at.expand(near));
                    if day.is_today {
                        text = text.strong();
                        if !plain {
                            text = text.color(ui.visuals().hyperlink_color);
                        }
                    } else if !day.in_month && !plain {
                        text = text.weak();
                    }
                    let button = egui::Button::selectable(day.in_period, text);
                    if ui.add_sized(DAY_SIZE, button).clicked() {
                        action = Some(Action::GoTo(day.date));
                    }
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
