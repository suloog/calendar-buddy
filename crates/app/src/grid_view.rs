//! Draws a `GridViewModel` (the day, 3-day or week view) with egui. Only drawing and input live
//! here; what goes where is decided in `calendar_core::view_model`.

use calendar_core::model::{Event, Rgb, Timing};
use calendar_core::regional::Regional;
use calendar_core::view_model::GridViewModel;
use chrono::Duration;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, ScrollArea, Sense, Stroke, StrokeKind,
    Vec2,
};

const HOUR_HEIGHT: f32 = 44.0;
const GUTTER: f32 = 52.0;
const HEADER_HEIGHT: f32 = 28.0;
const LANE_HEIGHT: f32 = 20.0;
const SNAP_MINUTES: f32 = 15.0;
/// Narrowest a column of overlapping events may get, so the start of its title stays visible.
const MIN_BLOCK_WIDTH: f32 = 18.0;

pub fn color(c: Rgb) -> Color32 {
    Color32::from_rgb(c.0, c.1, c.2)
}

pub fn text_on(c: Rgb) -> Color32 {
    let luminance = 0.299 * c.0 as f32 + 0.587 * c.1 as f32 + 0.114 * c.2 as f32;
    if luminance > 150.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

/// What the details say of an event whose calendar cannot be changed (`MOD-02-U01`).
pub const READ_ONLY: &str = "🔒 Read-only: this event can't be changed";

/// What an event is called in the views: its title, or "Busy" for an event of a calendar shared
/// as free/busy only, after a lock when its calendar cannot be changed (`MOD-02-U01`).
pub fn event_label(title: Option<&str>, read_only: bool) -> String {
    let title = title.unwrap_or("Busy");
    if read_only {
        format!("🔒 {title}")
    } else {
        title.to_string()
    }
}

/// An event drawn in `rect` that the keyboard can reach (`MOD-03-F24`): Tab moves to it, which
/// keeps it in sight and rings it; Enter or a click opens it into `opened`.
pub fn openable(
    ui: &egui::Ui,
    response: &egui::Response,
    rect: Rect,
    event: usize,
    opened: &mut Option<usize>,
) {
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(3),
            ui.visuals().selection.stroke,
            StrokeKind::Outside,
        );
    }
    if response.clicked() {
        *opened = Some(event);
    }
}

/// Horizontal offset and width of a block in column `column` of `columns`, inside a day of
/// `inner` width. Columns share the width while each stays at least `MIN_BLOCK_WIDTH` wide; past
/// that they overlap like a fan, each one's left edge still showing, and never leave the day.
fn column_geometry(column: u32, columns: u32, inner: f32) -> (f32, f32) {
    let natural = inner / columns as f32;
    if natural >= MIN_BLOCK_WIDTH || columns == 1 {
        return (column as f32 * natural, natural);
    }
    let width = MIN_BLOCK_WIDTH.min(inner);
    let step = (inner - width) / (columns - 1) as f32;
    (column as f32 * step, width)
}

#[derive(Default)]
pub struct State {
    scrolled_to_now: bool,
    drag: Option<Drag>,
}

struct Drag {
    block: usize,
    delta: Vec2,
}

/// A dragged event dropped somewhere else: shift by whole days and snapped minutes.
#[derive(Clone, Copy, Debug)]
pub struct Change {
    pub event: usize,
    pub days: i64,
    pub minutes: i64,
}

/// Moves an event in memory. Saving to Google is `MOD-05`; this exists for the feasibility check.
pub fn apply(events: &mut [Event], change: Change) {
    let shift = Duration::days(change.days) + Duration::minutes(change.minutes);
    if let Timing::Timed { start, end, zone } = events[change.event].timing {
        events[change.event].timing = Timing::Timed {
            start: start + shift,
            end: end + shift,
            zone,
        };
    }
}

pub fn show(
    ui: &mut egui::Ui,
    view: &GridViewModel,
    regional: &Regional,
    state: &mut State,
    movable: bool,
    opened: &mut Option<usize>,
) -> Option<Change> {
    let width = ui.available_width();
    let columns = view.days.len();
    let day_width = ((width - GUTTER) / columns as f32).max(40.0);
    let visuals = ui.visuals().clone();
    let grid_line = visuals.widgets.noninteractive.bg_stroke;
    let weak_text = visuals.weak_text_color();

    // Day header.
    let (header, _) = ui.allocate_exact_size(Vec2::new(width, HEADER_HEIGHT), Sense::hover());
    let painter = ui.painter_at(header);
    for (i, day) in view.days.iter().enumerate() {
        let x = header.left() + GUTTER + i as f32 * day_width;
        let rect = Rect::from_min_size(
            Pos2::new(x, header.top()),
            Vec2::new(day_width, HEADER_HEIGHT),
        );
        if day.is_today {
            painter.rect_filled(
                rect.shrink(2.0),
                CornerRadius::same(4),
                visuals.selection.bg_fill,
            );
        }
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            regional.weekday_day(day.date),
            FontId::proportional(14.0),
            visuals.text_color(),
        );
    }

    // All-day and multi-day strip.
    let strip_height = view.lanes as f32 * LANE_HEIGHT + 4.0;
    let (strip, _) = ui.allocate_exact_size(Vec2::new(width, strip_height), Sense::hover());
    let painter = ui.painter_at(strip);
    for bar in &view.bars {
        let x0 = strip.left() + GUTTER + bar.bar.first_day as f32 * day_width + 2.0;
        let x1 = strip.left() + GUTTER + (bar.bar.last_day + 1) as f32 * day_width - 2.0;
        let y = strip.top() + 2.0 + bar.bar.lane as f32 * LANE_HEIGHT;
        let rect = Rect::from_min_max(Pos2::new(x0, y), Pos2::new(x1, y + LANE_HEIGHT - 2.0));
        let response = ui.interact(
            rect,
            ui.scope_id().with(("bar", bar.bar.event)),
            Sense::click(),
        );
        painter.rect_filled(rect, CornerRadius::same(3), color(bar.color));
        painter.with_clip_rect(rect).text(
            rect.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            event_label(bar.title.as_deref(), bar.read_only),
            FontId::proportional(12.0),
            text_on(bar.color),
        );
        openable(ui, &response, rect, bar.bar.event, opened);
    }
    ui.painter()
        .hline(strip.x_range(), strip.bottom(), grid_line);

    // Hours.
    // The grid follows the clock, also on daylight-saving days (`DEC-67`).
    let grid_height = 24.0 * HOUR_HEIGHT;
    let mut scroll = ScrollArea::vertical()
        .id_salt("grid-hours")
        .auto_shrink([false, false]);
    if !state.scrolled_to_now {
        let target = (view.initial_scroll_minute as f32 / 60.0 - 1.0).max(0.0) * HOUR_HEIGHT;
        scroll = scroll.vertical_scroll_offset(target);
        state.scrolled_to_now = true;
    }

    let mut change = None;
    scroll.show(ui, |ui| {
        let (grid, _) = ui.allocate_exact_size(Vec2::new(width, grid_height), Sense::hover());
        let painter = ui.painter_at(grid);
        let minute_y = |m: f32| grid.top() + m / 60.0 * HOUR_HEIGHT;

        for hour in 0..=24 {
            let y = minute_y(hour as f32 * 60.0);
            painter.hline((grid.left() + GUTTER)..=grid.right(), y, grid_line);
            if hour > 0 && hour < 24 {
                painter.text(
                    Pos2::new(grid.left() + GUTTER - 6.0, y),
                    Align2::RIGHT_CENTER,
                    regional.hour(hour),
                    FontId::proportional(11.0),
                    weak_text,
                );
            }
        }
        for i in 0..=columns {
            let x = grid.left() + GUTTER + i as f32 * day_width;
            painter.vline(x, grid.y_range(), grid_line);
        }

        for (index, block) in view.blocks.iter().enumerate() {
            let p = block.placement;
            let (offset, column_width) = column_geometry(p.column, p.columns, day_width - 4.0);
            let x = grid.left() + GUTTER + p.day as f32 * day_width + 2.0 + offset;
            let mut rect = Rect::from_min_max(
                Pos2::new(x, minute_y(p.start_min as f32) + 1.0),
                Pos2::new(x + column_width - 2.0, minute_y(p.end_min as f32) - 1.0),
            );
            let response = ui.interact(
                rect,
                // Keyed on the event, so focus stays on it when the view is rebuilt.
                ui.scope_id().with(("block", p.event, p.day)),
                Sense::click_and_drag(),
            );
            if response.drag_started() && movable {
                state.drag = Some(Drag {
                    block: index,
                    delta: Vec2::ZERO,
                });
            }
            if let Some(drag) = state.drag.as_mut().filter(|d| d.block == index) {
                // From where the press started: a click-and-drag starts only after a few points.
                if let Some(total) = response.total_drag_delta() {
                    drag.delta = total;
                }
                let days = (drag.delta.x / day_width).round();
                let minutes =
                    (drag.delta.y / HOUR_HEIGHT * 60.0 / SNAP_MINUTES).round() * SNAP_MINUTES;
                rect = rect.translate(Vec2::new(days * day_width, minutes / 60.0 * HOUR_HEIGHT));
                if response.drag_stopped() {
                    change = Some(Change {
                        event: p.event,
                        days: days as i64,
                        minutes: minutes as i64,
                    });
                    state.drag = None;
                }
            }
            painter.rect_filled(rect, CornerRadius::same(3), color(block.color));
            let ink = text_on(block.color);
            let clipped = painter.with_clip_rect(rect.shrink(2.0));
            let time = regional.time(block.start.with_timezone(&view.zone).time());
            let name = event_label(block.title.as_deref(), block.read_only);
            let label = match &block.title {
                Some(_) => format!("{name}\n{time}"),
                None => name,
            };
            clipped.text(
                rect.left_top() + Vec2::new(4.0, 2.0),
                Align2::LEFT_TOP,
                label,
                FontId::proportional(12.0),
                ink,
            );
            openable(ui, &response, rect, p.event, opened);
        }

        if let Some(now) = view.now {
            let x = grid.left() + GUTTER + now.day as f32 * day_width;
            let y = minute_y(now.minute as f32);
            painter.hline(
                x..=(x + day_width),
                y,
                Stroke::new(2.0, Color32::from_rgb(0xEA, 0x43, 0x35)),
            );
            painter.circle_filled(Pos2::new(x, y), 4.0, Color32::from_rgb(0xEA, 0x43, 0x35));
        }
    });
    change
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_of_calendars_that_cannot_be_changed_carry_a_lock() {
        assert_eq!(event_label(Some("Standup"), false), "Standup");
        assert_eq!(event_label(Some("1:1"), true), "🔒 1:1");
        assert_eq!(event_label(None, true), "🔒 Busy");
    }

    #[test]
    fn few_overlapping_events_share_the_day_width() {
        assert_eq!(column_geometry(0, 3, 150.0), (0.0, 50.0));
        assert_eq!(column_geometry(2, 3, 150.0), (100.0, 50.0));
    }

    #[test]
    fn many_overlapping_events_stay_inside_their_day_with_a_visible_start() {
        let inner = 120.0;
        let columns = 20;
        let mut previous_offset = -1.0;
        for column in 0..columns {
            let (offset, width) = column_geometry(column, columns, inner);
            assert!(offset >= 0.0 && offset + width <= inner + 0.001);
            assert!(width >= MIN_BLOCK_WIDTH);
            assert!(
                offset > previous_offset,
                "each block starts right of the previous one"
            );
            previous_offset = offset;
        }
    }
}
