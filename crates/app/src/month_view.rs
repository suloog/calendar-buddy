//! Draws a `MonthViewModel` with egui. Only drawing and input live here; which events a day has
//! and how many of them fit is decided in `calendar_core::month`.

use calendar_core::month::{MonthItem, MonthViewModel};
use calendar_core::regional::Regional;
use chrono::NaiveDate;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, ScrollArea, Sense, Vec2,
};

use crate::grid_view::{color, openable, text_on};

const HEADER_HEIGHT: f32 = 24.0;
const DATE_HEIGHT: f32 = 22.0;
const ITEM_HEIGHT: f32 = 18.0;

#[derive(Default)]
pub struct State {
    /// The day whose full list is open, after choosing "+N more".
    open_day: Option<NaiveDate>,
}

fn label(item: &MonthItem, view: &MonthViewModel, regional: &Regional) -> String {
    let title = item.title.as_deref().unwrap_or("Busy");
    match item.start {
        Some(start) => format!(
            "{} {title}",
            regional.time(start.with_timezone(&view.zone).time())
        ),
        None => title.to_string(),
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
    if item.start.is_none() {
        painter.rect_filled(rect, CornerRadius::same(3), color(item.color));
        clipped.text(
            rect.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            text,
            FontId::proportional(12.0),
            text_on(item.color),
        );
    } else {
        painter.circle_filled(
            rect.left_center() + Vec2::new(6.0, 0.0),
            3.5,
            color(item.color),
        );
        clipped.text(
            rect.left_center() + Vec2::new(14.0, 0.0),
            Align2::LEFT_CENTER,
            text,
            FontId::proportional(12.0),
            ink,
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
    let grid_line = visuals.widgets.noninteractive.bg_stroke;
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
            regional.weekday(day.date),
            FontId::proportional(12.0),
            visuals.weak_text_color(),
        );
    }

    for (w, week) in view.weeks.iter().enumerate() {
        for (i, day) in week.iter().enumerate() {
            let cell = Rect::from_min_size(
                Pos2::new(
                    area.left() + i as f32 * day_width,
                    area.top() + HEADER_HEIGHT + w as f32 * week_height,
                ),
                Vec2::new(day_width, week_height),
            );
            painter.rect_stroke(
                cell,
                CornerRadius::ZERO,
                grid_line,
                egui::StrokeKind::Inside,
            );

            let date_center = Pos2::new(cell.center().x, cell.top() + DATE_HEIGHT / 2.0);
            let date_color = if day.is_today {
                painter.circle_filled(date_center, 10.0, visuals.selection.bg_fill);
                visuals.strong_text_color()
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

            let row = |n: usize| {
                Rect::from_min_size(
                    Pos2::new(
                        cell.left() + 2.0,
                        cell.top() + DATE_HEIGHT + n as f32 * ITEM_HEIGHT,
                    ),
                    Vec2::new(day_width - 4.0, ITEM_HEIGHT - 2.0),
                )
            };
            let (shown, more) = day.fit(slots);
            for (n, item) in day.items[..shown].iter().enumerate() {
                let rect = row(n);
                let id = ui.scope_id().with(("item", day.date, n));
                let response = ui.interact(rect, id, Sense::click());
                paint_item(&painter, rect, item, view, regional, visuals.text_color());
                openable(ui, &response, rect, item.event, opened);
            }
            // With no row free in a very short window, there is nowhere to put "+N more".
            if more > 0 && slots > 0 {
                let rect = row(shown);
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
                    format!("+{more} more"),
                    FontId::proportional(12.0),
                    visuals.text_color(),
                );
                if response.clicked() {
                    state.open_day = Some(day.date);
                }
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
