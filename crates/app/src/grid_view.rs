//! Draws a `GridViewModel` (the day, 3-day or week view) with egui. Only drawing and input live
//! here; what goes where is decided in `calendar_core::view_model`.

use calendar_core::create::{SAVING, STEP, snapped};
use calendar_core::drag::{self, Target};
use calendar_core::model::Rgb;
use calendar_core::regional::Regional;
use calendar_core::view_model::GridViewModel;
use chrono::{DateTime, Duration, Utc};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, FontId, LayerId, Order, PointerButton, Pos2,
    Rect, ScrollArea, Sense, Stroke, StrokeKind, Vec2,
};

pub const HOUR_HEIGHT: f32 = 44.0;
const GUTTER: f32 = 52.0;
const HEADER_HEIGHT: f32 = 28.0;
const LANE_HEIGHT: f32 = 20.0;
/// How far up from a block's bottom edge its end can be grabbed.
const END_HANDLE: f32 = 5.0;
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

/// What an event is called in the views: its title, or "Busy" for an event of a calendar shared
/// as free/busy only, after a lock when its calendar cannot be changed (`MOD-02-U01`), and marked
/// while Google has not confirmed a new event (`MOD-05-U03`).
pub fn event_label(title: Option<&str>, read_only: bool, saving: bool) -> String {
    let title = title.unwrap_or("Busy");
    match (read_only, saving) {
        (true, _) => format!("🔒 {title}"),
        (false, true) => format!("{title} · {SAVING}"),
        (false, false) => title.to_string(),
    }
}

/// How far apart the stripes of a hatched event are.
const HATCH_SPACING: f32 = 8.0;

/// Diagonal stripes over an out-of-office event (`MOD-05-U01`), in a faint shade of its text's
/// colour `ink`.
pub fn hatch(painter: &egui::Painter, rect: Rect, ink: Color32) {
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let stroke = Stroke::new(1.5, ink.gamma_multiply(0.35));
    let mut x = rect.left() - rect.height();
    while x < rect.right() {
        painter.line_segment(
            [
                Pos2::new(x, rect.bottom()),
                Pos2::new(x + rect.height(), rect.top()),
            ],
            stroke,
        );
        x += HATCH_SPACING;
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
    /// The day and minute where a selection of a time range started.
    selecting: Option<(usize, u32)>,
}

impl State {
    /// The events shown changed: a drag under way would point to another event, so it ends.
    pub fn events_changed(&mut self) {
        self.drag = None;
    }
}

struct Drag {
    grabbed: Grabbed,
    /// From where the press started.
    delta: Vec2,
}

/// What is dragged, by its place in the view's blocks or bars.
#[derive(Clone, Copy, PartialEq)]
enum Grabbed {
    Block(usize),
    /// The end of a block.
    End(usize),
    Bar(usize),
}

/// What the user did in the hours.
#[derive(Clone, Copy, Debug)]
pub enum Action {
    /// An event dragged and dropped (`MOD-05-F13`, `MOD-05-F14`).
    Dropped { event: usize, drop: Target },
    /// A drag started on an event that cannot be moved, which stays where it is (`MOD-05-F17`).
    Unmovable { event: usize },
    /// A click on a time where there is no event: the day's index and the minute of the day.
    Clicked { day: usize, minute: u32 },
    /// A range selected in one day, from the minute pressed to the minute released.
    Selected {
        day: usize,
        pressed: u32,
        released: u32,
    },
}

/// The day and minute of the day under a point `at` from the hours' top left corner; `None` in
/// the hour labels or past the last day.
fn slot(at: Vec2, day_width: f32, days: usize) -> Option<(usize, u32)> {
    let x = at.x - GUTTER;
    let day = (x / day_width).floor();
    if x < 0.0 || day >= days as f32 {
        return None;
    }
    let minute = (at.y / HOUR_HEIGHT * 60.0).clamp(0.0, 24.0 * 60.0);
    Some((day as usize, minute as u32))
}

/// The minutes on the clock `dy` points of the hours stand for, in whole steps (`MOD-05-U02`).
pub fn minutes(dy: f32) -> i64 {
    drag::snap(dy / HOUR_HEIGHT * 60.0)
}

/// Where an event shown on `day` of `days` lands when dragged by `delta`: whole days, kept among
/// those shown, and minutes in whole steps.
pub fn moved(delta: Vec2, day_width: f32, day: usize, days: usize) -> Target {
    let shift = (delta.x / day_width).round() as i64;
    Target::Moved {
        days: shift.clamp(-(day as i64), (days - 1 - day) as i64),
        minutes: minutes(delta.y),
    }
}

/// Whether the widget is being dragged, or was released this frame.
fn held(response: &egui::Response) -> bool {
    response.dragged() || response.drag_stopped()
}

pub fn show(
    ui: &mut egui::Ui,
    view: &GridViewModel,
    regional: &Regional,
    state: &mut State,
    movable: bool,
    opened: &mut Option<usize>,
) -> Option<Action> {
    let width = ui.available_width();
    let columns = view.days.len();
    let day_width = ((width - GUTTER) / columns as f32).max(40.0);
    let visuals = ui.visuals().clone();
    let grid_line = visuals.widgets.noninteractive.bg_stroke;
    let weak_text = visuals.weak_text_color();
    // Offline, events cannot be moved (`MOD-02-F15`), also by a drag under way; one whose event
    // left the view is dropped.
    let grabbed = state.drag.as_ref().map(|drag| drag.grabbed);
    let gone = match grabbed {
        Some(Grabbed::Block(i) | Grabbed::End(i)) => i >= view.blocks.len(),
        Some(Grabbed::Bar(i)) => i >= view.bars.len(),
        None => false,
    };
    if !movable || gone {
        state.drag = None;
    }
    let pointer = ui.input(|i| i.pointer.latest_pos());
    let day_at = |x: f32, left: f32| slot(Vec2::new(x - left, 0.0), day_width, columns);
    let mut action = None;

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
    // Above the hours, a dragged event is over the strip.
    let over_strip = pointer.is_some_and(|p| p.y < strip.bottom());
    let painter = ui.painter_at(strip);
    let mut bar_into_hours = None;
    for (index, bar) in view.bars.iter().enumerate() {
        let x0 = strip.left() + GUTTER + bar.bar.first_day as f32 * day_width + 2.0;
        let x1 = strip.left() + GUTTER + (bar.bar.last_day + 1) as f32 * day_width - 2.0;
        let y = strip.top() + 2.0 + bar.bar.lane as f32 * LANE_HEIGHT;
        let mut rect = Rect::from_min_max(Pos2::new(x0, y), Pos2::new(x1, y + LANE_HEIGHT - 2.0));
        let response = ui.interact(
            rect,
            ui.scope_id().with(("bar", bar.bar.event)),
            Sense::click_and_drag(),
        );
        if response.drag_started_by(PointerButton::Primary) && movable {
            if bar.movable {
                state.drag = Some(Drag {
                    grabbed: Grabbed::Bar(index),
                    delta: Vec2::ZERO,
                });
            } else {
                action = Some(Action::Unmovable {
                    event: bar.bar.event,
                });
            }
        }
        if state.drag.as_ref().map(|d| d.grabbed) == Some(Grabbed::Bar(index)) && !held(&response) {
            state.drag = None;
        }
        if let Some(drag) = state
            .drag
            .as_mut()
            .filter(|d| d.grabbed == Grabbed::Bar(index))
        {
            if let Some(total) = response.total_drag_delta() {
                drag.delta = total;
            }
            // An all-day event goes into the hours where it is dropped there; otherwise it moves
            // by whole days in the strip.
            if bar.all_day && !over_strip {
                if response.drag_stopped() {
                    bar_into_hours = Some(index);
                }
            } else {
                // Kept in sight by a day at least, also when it starts before the first day.
                let shift = (drag.delta.x / day_width).round() as i64;
                let last = bar.bar.last_day as i64;
                let days = shift.clamp(-last, (columns - 1 - bar.bar.first_day) as i64);
                rect = rect.translate(Vec2::new(days as f32 * day_width, 0.0));
                let drop = Target::Moved { days, minutes: 0 };
                if response.drag_stopped() {
                    action = Some(Action::Dropped {
                        event: bar.bar.event,
                        drop,
                    });
                }
            }
        }
        // Working locations and birthdays are labels, outlined in their colour (`MOD-05-U01`).
        let ink = if bar.label {
            let outline = Stroke::new(1.0, color(bar.color));
            painter.rect_stroke(rect, CornerRadius::same(3), outline, StrokeKind::Inside);
            visuals.text_color()
        } else {
            painter.rect_filled(rect, CornerRadius::same(3), color(bar.color));
            text_on(bar.color)
        };
        if bar.hatched {
            hatch(&painter, rect, ink);
        }
        painter.with_clip_rect(rect).text(
            rect.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            event_label(bar.title.as_deref(), bar.read_only, bar.saving),
            FontId::proportional(12.0),
            ink,
        );
        openable(ui, &response, rect, bar.bar.event, opened);
    }
    // An event of the hours dragged over the strip, in the day under the pointer: drawn over the
    // strip's last lane, so the layout under the pointer does not move.
    if let (Some(Grabbed::Block(index)), true, Some(p)) = (grabbed, over_strip, pointer)
        && let Some((day, _)) = day_at(p.x, strip.left())
        && let Some(block) = view.blocks.get(index)
    {
        let x = strip.left() + GUTTER + day as f32 * day_width;
        let y = strip.bottom() - LANE_HEIGHT;
        let rect = Rect::from_min_max(
            Pos2::new(x + 2.0, y),
            Pos2::new(x + day_width - 2.0, y + LANE_HEIGHT - 2.0),
        );
        let painter = ui.ctx().layer_painter(LayerId::new(
            Order::Foreground,
            ui.scope_id().with("drop-lane"),
        ));
        painter.rect_filled(rect, CornerRadius::same(3), color(block.color));
        painter.with_clip_rect(rect).text(
            rect.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            event_label(block.title.as_deref(), false, false),
            FontId::proportional(12.0),
            text_on(block.color),
        );
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

    scroll.show(ui, |ui| {
        // Clicking or selecting where there is no event starts one (`MOD-05-F01`); the events,
        // drawn over it, take their own clicks. Not a Tab stop: `c` starts one from the keyboard.
        let (grid, hours) =
            ui.allocate_exact_size(Vec2::new(width, grid_height), Sense::CLICK | Sense::DRAG);
        let painter = ui.painter_at(grid);
        let minute_y = |m: f32| grid.top() + m / 60.0 * HOUR_HEIGHT;
        let at = |pos: Option<Pos2>| slot(pos? - grid.left_top(), day_width, columns);
        if hours.clicked()
            && let Some((day, minute)) = at(hours.interact_pointer_pos())
        {
            action = Some(Action::Clicked { day, minute });
        }
        if hours.drag_started_by(PointerButton::Primary) {
            state.selecting = at(ui.input(|i| i.pointer.press_origin()));
        }
        // A selection left by a drag that ended elsewhere, e.g. in another view, is dropped.
        if !hours.dragged_by(PointerButton::Primary)
            && !hours.drag_stopped_by(PointerButton::Primary)
            || state.selecting.is_some_and(|(day, _)| day >= columns)
        {
            state.selecting = None;
        }

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

        // The range being selected, in the day where the press started, as Google Calendar does.
        if let Some((day, pressed)) = state.selecting {
            let pointer = hours.interact_pointer_pos().unwrap_or_default() - grid.left_top();
            let released = slot(Vec2::new(GUTTER, pointer.y), day_width, columns)
                .map_or(pressed, |(_, minute)| minute);
            let (from, until) = snapped(pressed, released);
            let x = grid.left() + GUTTER + day as f32 * day_width;
            let rect = Rect::from_min_max(
                Pos2::new(x + 2.0, minute_y(from as f32) + 1.0),
                Pos2::new(x + day_width - 2.0, minute_y(until as f32) - 1.0),
            );
            painter.rect_filled(rect, CornerRadius::same(3), visuals.selection.bg_fill);
            if hours.drag_stopped_by(PointerButton::Primary) {
                action = Some(Action::Selected {
                    day,
                    pressed,
                    released,
                });
                state.selecting = None;
            }
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
            // The end is grabbed where the event ends, not where it goes on past midnight.
            let next_day = view.days[p.day]
                .date
                .succ_opt()
                .unwrap_or(view.days[p.day].date);
            let ends_here = block.end.with_timezone(&view.zone).naive_local()
                <= next_day.and_time(chrono::NaiveTime::MIN);
            let end = (movable && block.movable && ends_here).then(|| {
                let handle = Rect::from_min_max(
                    Pos2::new(rect.left(), rect.bottom() - END_HANDLE),
                    rect.right_bottom(),
                );
                ui.interact(
                    handle,
                    ui.scope_id().with(("end", p.event, p.day)),
                    Sense::drag(),
                )
                .on_hover_cursor(CursorIcon::ResizeVertical)
            });
            if response.drag_started_by(PointerButton::Primary) && movable {
                if block.movable {
                    state.drag = Some(Drag {
                        grabbed: Grabbed::Block(index),
                        delta: Vec2::ZERO,
                    });
                } else {
                    action = Some(Action::Unmovable { event: p.event });
                }
            }
            if let Some(end) = &end
                && end.drag_started_by(PointerButton::Primary)
            {
                state.drag = Some(Drag {
                    grabbed: Grabbed::End(index),
                    delta: Vec2::ZERO,
                });
            }
            // A drag goes with the widget it started on, not with a block that took its place
            // when the view was rebuilt.
            let lost = match state.drag.as_ref().map(|d| d.grabbed) {
                Some(Grabbed::Block(i)) if i == index => !held(&response),
                Some(Grabbed::End(i)) if i == index => !end.as_ref().is_some_and(held),
                _ => false,
            };
            if lost {
                state.drag = None;
            }
            // The times it would have if dropped now, shown while it is dragged (`MOD-05-U02`).
            let mut times = None;
            match state.drag.as_mut() {
                Some(drag) if drag.grabbed == Grabbed::Block(index) => {
                    // From where the press started: a click-and-drag starts only after a few points.
                    if let Some(total) = response.total_drag_delta() {
                        drag.delta = total;
                    }
                    if over_strip {
                        if response.drag_stopped()
                            && let Some((day, _)) = pointer.and_then(|p| day_at(p.x, grid.left()))
                        {
                            let date = view.days[day].date;
                            let drop = Target::AllDay { date };
                            action = Some(Action::Dropped {
                                event: p.event,
                                drop,
                            });
                        }
                    } else if let drop @ Target::Moved { days, minutes } =
                        moved(drag.delta, day_width, p.day, columns)
                    {
                        let shift =
                            Vec2::new(days as f32 * day_width, minutes as f32 / 60.0 * HOUR_HEIGHT);
                        rect = rect.translate(shift);
                        let (start, end) =
                            drag::moved((block.start, block.end), days, minutes, view.zone);
                        times = Some((start, end));
                        if response.drag_stopped() {
                            action = Some(Action::Dropped {
                                event: p.event,
                                drop,
                            });
                        }
                    }
                }
                Some(drag) if drag.grabbed == Grabbed::End(index) => {
                    let Some(end) = &end else {
                        unreachable!("a drag of an end that is not there is lost")
                    };
                    if let Some(total) = end.total_drag_delta() {
                        drag.delta = total;
                    }
                    // A step long at least, and ending by midnight.
                    let step = i64::from(STEP);
                    let shortest = (p.start_min as i64 + step - p.end_min as i64).min(0);
                    let by = minutes(drag.delta.y).clamp(shortest, 24 * 60 - p.end_min as i64);
                    rect.max.y += by as f32 / 60.0 * HOUR_HEIGHT;
                    let until = drag::stretched((block.start, block.end), by, view.zone);
                    times = Some((block.start, until));
                    if end.drag_stopped() {
                        action = Some(Action::Dropped {
                            event: p.event,
                            drop: Target::Resized { minutes: by },
                        });
                    }
                }
                _ => {}
            }
            painter.rect_filled(rect, CornerRadius::same(3), color(block.color));
            let ink = text_on(block.color);
            if block.hatched {
                hatch(&painter, rect, ink);
            }
            let clipped = painter.with_clip_rect(rect.shrink(2.0));
            let name = event_label(block.title.as_deref(), block.read_only, block.saving);
            let time = match times {
                Some((start, end)) => {
                    let time =
                        |at: DateTime<Utc>| regional.time(at.with_timezone(&view.zone).time());
                    format!("{} – {}", time(start), time(end))
                }
                None => regional.time(block.start.with_timezone(&view.zone).time()),
            };
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

        // An all-day event dragged into the hours, from the step under the pointer.
        if let (Some(Grabbed::Bar(index)), false) = (grabbed, over_strip)
            && let Some(bar) = view.bars.get(index).filter(|bar| bar.all_day)
            && let Some((day, minute)) = at(pointer)
        {
            let from = (minute / STEP * STEP).min(24 * 60 - STEP);
            let x = grid.left() + GUTTER + day as f32 * day_width;
            let rect = Rect::from_min_max(
                Pos2::new(x + 2.0, minute_y(from as f32) + 1.0),
                Pos2::new(x + day_width - 2.0, minute_y((from + STEP) as f32) - 1.0),
            );
            painter.rect_filled(rect, CornerRadius::same(3), color(bar.color));
            let time = regional.time(chrono::NaiveTime::MIN + Duration::minutes(from.into()));
            painter.with_clip_rect(rect).text(
                rect.left_center() + Vec2::new(4.0, 0.0),
                Align2::LEFT_CENTER,
                format!("{} {time}", event_label(bar.title.as_deref(), false, false)),
                FontId::proportional(12.0),
                text_on(bar.color),
            );
            if bar_into_hours == Some(index) {
                let date = view.days[day].date;
                action = Some(Action::Dropped {
                    event: bar.bar.event,
                    drop: Target::Timed { date, minute: from },
                });
            }
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
    // A drag ends with the button released, wherever that was.
    if !ui.input(|i| i.pointer.primary_down()) {
        state.drag = None;
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use calendar_core::model::Kind;

    #[test]
    fn the_marks_of_special_event_types_are_in_the_apps_fonts() {
        let ctx = egui::Context::default();
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .textures_delta
            .clear();
        let font = FontId::proportional(12.0);
        for kind in [
            Kind::OutOfOffice,
            Kind::FocusTime,
            Kind::WorkingLocation,
            Kind::Birthday,
        ] {
            let mark = kind.icon().unwrap();
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, mark)), "{mark}");
        }
    }

    #[test]
    fn events_of_calendars_that_cannot_be_changed_carry_a_lock() {
        assert_eq!(event_label(Some("Standup"), false, false), "Standup");
        assert_eq!(event_label(Some("1:1"), true, false), "🔒 1:1");
        assert_eq!(event_label(None, true, false), "🔒 Busy");
    }

    #[test]
    fn a_new_event_google_has_not_confirmed_is_marked_saving() {
        assert_eq!(
            event_label(Some("Planning"), false, true),
            "Planning · Saving…"
        );
    }

    #[test]
    fn a_point_in_the_hours_is_the_day_and_minute_under_it() {
        let day_width = 100.0;
        // Thursday, the fourth day of the week, at 10:00 and at 11:30.
        let at = |day: f32, minute: f32| {
            Vec2::new(GUTTER + day * day_width + 10.0, minute / 60.0 * HOUR_HEIGHT)
        };
        assert_eq!(slot(at(3.0, 600.0), day_width, 7), Some((3, 600)));
        assert_eq!(slot(at(3.0, 690.0), day_width, 7), Some((3, 690)));
        assert_eq!(
            slot(at(0.0, 24.0 * 60.0 + 30.0), day_width, 7),
            Some((0, 1440))
        );
        assert_eq!(slot(Vec2::new(GUTTER - 1.0, 0.0), day_width, 7), None);
        assert_eq!(slot(at(7.0, 600.0), day_width, 7), None);
    }

    #[test]
    fn a_dragged_event_moves_by_whole_days_kept_in_view_and_15_minute_steps() {
        let day_width = 100.0;
        let hour = HOUR_HEIGHT;
        let to = |dx: f32, dy: f32, day| moved(Vec2::new(dx, dy), day_width, day, 7);
        assert_eq!(
            to(140.0, hour * 40.0 / 60.0, 2),
            Target::Moved {
                days: 1,
                minutes: 45
            }
        );
        assert_eq!(
            to(-40.0, -hour * 20.0 / 60.0, 2),
            Target::Moved {
                days: 0,
                minutes: -15
            }
        );
        // Not past the first or the last day shown.
        assert_eq!(
            to(-500.0, 0.0, 2),
            Target::Moved {
                days: -2,
                minutes: 0
            }
        );
        assert_eq!(
            to(900.0, 0.0, 2),
            Target::Moved {
                days: 4,
                minutes: 0
            }
        );
        assert_eq!(minutes(hour * 1.5), 90);
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
