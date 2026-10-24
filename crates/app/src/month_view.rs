//! Draws a `MonthViewModel` with egui. Only drawing and input live here; which events a day has
//! and how many of them fit is decided in `calendar_core::month`.

use calendar_core::month::{MonthItem, MonthViewModel};
use calendar_core::regional::Regional;
use calendar_core::texts;
use chrono::NaiveDate;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, ScrollArea, Sense, Vec2,
};

use calendar_core::model::Rgb;

use crate::grid_view::{
    STRIP, awaiting, color, event_label, event_text, fade, filled, grid_line, openable, text_on,
};

const HEADER_HEIGHT: f32 = 24.0;
const DATE_HEIGHT: f32 = 22.0;
const ITEM_HEIGHT: f32 = 18.0;

#[derive(Default)]
pub struct State {
    /// The day whose full list is open, after choosing "+N more".
    open_day: Option<NaiveDate>,
}

fn label(item: &MonthItem, view: &MonthViewModel, regional: &Regional) -> String {
    let title = event_label(item.title.as_deref(), item.read_only, item.saving);
    match item.start {
        Some(start) => format!(
            "{} {title}",
            regional.time(start.with_timezone(&view.zone).time())
        ),
        None => title,
    }
}

/// One event in a row of `rect`: all-day and multi-day events as a filled chip, timed ones as a
/// coloured dot with the time in `ink`, as Google Calendar draws them.
fn paint_item(
    painter: &egui::Painter,
    rect: Rect,
    item: &MonthItem,
    view: &MonthViewModel,
    regional: &Regional,
    ink: Color32,
) {
    let clipped = painter.with_clip_rect(rect);
    let text = label(item, view, regional);
    // An invitation awaiting an answer, a chip or not (`MOD-06-U04`).
    let ink = if item.awaiting {
        text_on(awaiting(painter, rect, item.color))
    } else {
        ink
    };
    if item.start.is_none() {
        let on = if item.awaiting {
            ink
        } else {
            text_on(filled(painter, rect, item.color, item.declined))
        };
        event_text(
            &clipped,
            rect.left_center() + Vec2::new(STRIP + 5.0, 0.0),
            Align2::LEFT_CENTER,
            text,
            crate::appearance::strong(12.0),
            on,
            item.declined,
            rect.width() - STRIP - 8.0,
        );
    } else {
        painter.circle_filled(
            rect.left_center() + Vec2::new(6.0, 0.0),
            3.5,
            color(item.color),
        );
        // A declined event, struck through and faded (`MOD-06-F14`).
        if item.declined {
            fade(painter, rect);
        }
        event_text(
            &clipped,
            rect.left_center() + Vec2::new(14.0, 0.0),
            Align2::LEFT_CENTER,
            text,
            FontId::proportional(12.0),
            ink,
            item.declined,
            rect.width() - 16.0,
        );
    }
}

pub fn show(
    ui: &mut egui::Ui,
    view: &MonthViewModel,
    regional: &Regional,
    state: &mut State,
    opened: &mut Option<usize>,
) {
    let visuals = ui.visuals().clone();
    let grid_line = grid_line(&visuals);
    let (area, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let painter = ui.painter_at(area);
    let day_width = area.width() / 7.0;
    let week_height = (area.height() - HEADER_HEIGHT) / view.weeks.len() as f32;
    let slots = ((week_height - DATE_HEIGHT) / ITEM_HEIGHT).max(0.0) as usize;

    // The days of the week, from the locale's first day, as the weeks below run.
    for (i, day) in view.weeks[0].iter().enumerate() {
        painter.text(
            Pos2::new(
                area.left() + (i as f32 + 0.5) * day_width,
                area.top() + HEADER_HEIGHT / 2.0,
            ),
            Align2::CENTER_CENTER,
            // As above the days of the week view.
            regional.weekday(day.date).to_uppercase(),
            FontId::proportional(11.0),
            visuals.weak_text_color(),
        );
    }

    // One line between two days, not a frame of each.
    let weeks = area.top() + HEADER_HEIGHT..=area.bottom();
    for i in 0..=7 {
        let x = (area.left() + i as f32 * day_width).clamp(area.left() + 0.5, area.right() - 0.5);
        painter.vline(x, weeks.clone(), grid_line);
    }
    for w in 0..=view.weeks.len() {
        let y = area.top() + HEADER_HEIGHT + w as f32 * week_height;
        painter.hline(area.x_range(), y.min(area.bottom() - 0.5), grid_line);
    }

    for (w, week) in view.weeks.iter().enumerate() {
        let cell = |i: usize| {
            Rect::from_min_size(
                Pos2::new(
                    area.left() + i as f32 * day_width,
                    area.top() + HEADER_HEIGHT + w as f32 * week_height,
                ),
                Vec2::new(day_width, week_height),
            )
        };
        // The row `n` under a day's date, from its day `first` to its day `last`.
        let row = |first: usize, last: usize, n: usize| {
            Rect::from_min_max(
                Pos2::new(
                    cell(first).left() + 2.0,
                    cell(first).top() + DATE_HEIGHT + n as f32 * ITEM_HEIGHT,
                ),
                Pos2::new(
                    cell(last).right() - 2.0,
                    cell(first).top() + DATE_HEIGHT + (n + 1) as f32 * ITEM_HEIGHT - 2.0,
                ),
            )
        };
        let fits: Vec<_> = week.iter().map(|day| day.fit(slots)).collect();
        for (i, day) in week.iter().enumerate() {
            let cell = cell(i);
            let date_center = Pos2::new(cell.center().x, cell.top() + DATE_HEIGHT / 2.0);
            // Today on a disc of the accent, as in the week view and the mini calendar.
            let date_color = if day.is_today {
                let accent = visuals.selection.stroke.color;
                painter.circle_filled(date_center, 10.0, accent);
                text_on(Rgb(accent.r(), accent.g(), accent.b()))
            } else if day.in_month {
                visuals.text_color()
            } else {
                visuals.weak_text_color()
            };
            painter.text(
                date_center,
                Align2::CENTER_CENTER,
                day.date.format("%-d").to_string(),
                FontId::proportional(12.0),
                date_color,
            );

            // The timed events under the rows of the week's bars.
            let fit = fits[i];
            let timed = day.items.iter().filter(|item| item.lane.is_none());
            for (n, item) in timed.take(fit.timed).enumerate() {
                let rect = row(i, i, fit.lanes + n);
                let id = ui.scope_id().with(("item", day.date, n));
                let response = ui.interact(rect, id, Sense::click());
                // Under the pointer, a timed event's row is lit, as a bar is by its fill.
                if response.hovered() {
                    painter.rect_filled(
                        rect,
                        CornerRadius::same(crate::grid_view::EVENT_RADIUS),
                        visuals.widgets.hovered.weak_bg_fill,
                    );
                }
                paint_item(&painter, rect, item, view, regional, visuals.text_color());
                openable(ui, &response, rect, item.event, opened);
            }
            // With no row free in a very short window, there is nowhere to put "+N more".
            if fit.more > 0 && slots > 0 {
                let rect = row(i, i, fit.lanes + fit.timed);
                let response =
                    ui.interact(rect, ui.scope_id().with(("more", day.date)), Sense::click());
                if response.hovered() || response.has_focus() {
                    painter.rect_filled(
                        rect,
                        CornerRadius::same(3),
                        visuals.widgets.hovered.bg_fill,
                    );
                }
                painter.with_clip_rect(rect).text(
                    rect.left_center() + Vec2::new(4.0, 0.0),
                    Align2::LEFT_CENTER,
                    texts::more(fit.more),
                    FontId::proportional(12.0),
                    visuals.text_color(),
                );
                if response.clicked() {
                    state.open_day = Some(day.date);
                }
            }
        }

        // All-day and multi-day events as one bar across their days in the week, as in Google
        // Calendar (`MOD-03-F26`); broken off on a day whose rows its bar no longer fits in.
        for bar in &view.bars[w] {
            let lane = bar.item.lane.unwrap_or_default() as usize;
            let mut day = bar.first_day;
            while day <= bar.last_day {
                if lane >= fits[day].lanes {
                    day += 1;
                    continue;
                }
                let first = day;
                while day < bar.last_day && lane < fits[day + 1].lanes {
                    day += 1;
                }
                let rect = row(first, day, lane);
                let id = ui.scope_id().with(("bar", bar.item.event, w, first));
                let response = ui.interact(rect, id, Sense::click());
                paint_item(
                    &painter,
                    rect,
                    &bar.item,
                    view,
                    regional,
                    visuals.text_color(),
                );
                openable(ui, &response, rect, bar.item.event, opened);
                day += 1;
            }
        }
    }

    // The full list of the chosen day.
    let Some(date) = state.open_day else {
        return;
    };
    let Some(day) = view.day(date) else {
        state.open_day = None;
        return;
    };
    let mut open = true;
    egui::Window::new(regional.weekday_day_month(date))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_pos(area.center())
        .show(ui.ctx(), |ui| {
            ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for item in &day.items {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(220.0, ITEM_HEIGHT), Sense::click());
                    let rect = rect.shrink2(Vec2::new(0.0, 1.0));
                    paint_item(
                        ui.painter(),
                        rect,
                        item,
                        view,
                        regional,
                        ui.visuals().text_color(),
                    );
                    openable(ui, &response, rect, item.event, opened);
                }
            });
        });
    // Like Google Calendar, opening one of its events or Escape closes the list.
    if !open || opened.is_some() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.open_day = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::{self, VERY_DARK, VERY_LIGHT, answered, contrast_of};
    use calendar_core::contrast::AA;
    use calendar_core::fixtures::{all_day, d, timed};
    use calendar_core::guests::Response;
    use calendar_core::month::month_view;
    use calendar_core::theme::Scheme;
    use chrono::{TimeZone, Utc, Weekday};
    use chrono_tz::Europe::Warsaw;

    #[test]
    fn text_of_invitations_and_declined_events_reads_at_4_5_in_both_themes() {
        let at = |title: &str, day| timed(title, d(2026, 11, day), (9, 0), (10, 0));
        let events = [
            answered(at("Invited light", 9), VERY_LIGHT, Response::Awaiting),
            answered(at("Invited dark", 10), VERY_DARK, Response::Awaiting),
            answered(at("Declined light", 11), VERY_LIGHT, Response::Declined),
            answered(at("Declined dark", 12), VERY_DARK, Response::Declined),
            answered(
                all_day("All day light", d(2026, 11, 17), 1),
                VERY_LIGHT,
                Response::Declined,
            ),
            answered(
                all_day("All day dark", d(2026, 11, 18), 1),
                VERY_DARK,
                Response::Declined,
            ),
        ];
        let now = Utc.with_ymd_and_hms(2026, 11, 11, 12, 0, 0).unwrap();
        let view = month_view(&events, d(2026, 11, 11), Warsaw, now, Weekday::Mon);
        let regional = Regional::from_env(|_| None);

        for scheme in [Scheme::Light, Scheme::Dark] {
            let shapes = screen::drawn(Vec2::new(1600.0, 900.0), 1.0, scheme, None, |ui| {
                show(ui, &view, &regional, &mut State::default(), &mut None);
            });
            for event in &events {
                let ratio = contrast_of(&shapes, &event.title);
                assert!(ratio >= AA, "{scheme:?}, {}: {ratio}", event.title);
            }
        }
    }
}
