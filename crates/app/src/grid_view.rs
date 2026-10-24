//! Draws a `GridViewModel` (the day, 3-day or week view) with egui. Only drawing and input live
//! here; what goes where is decided in `calendar_core::view_model`.

use calendar_core::contrast;
use calendar_core::create::{STEP, snapped};
use calendar_core::drag::{self, Target};
use calendar_core::model::Rgb;
use calendar_core::regional::Regional;
use calendar_core::texts::{self, SAVING};
use calendar_core::view_model::GridViewModel;
use chrono::{DateTime, Datelike, Duration, Utc};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, FontId, LayerId, Order, PointerButton, Pos2,
    Rect, ScrollArea, Sense, Stroke, StrokeKind, Vec2, text::LayoutJob,
};

pub const HOUR_HEIGHT: f32 = 44.0;
const GUTTER: f32 = 52.0;
const HEADER_HEIGHT: f32 = 52.0;
const LANE_HEIGHT: f32 = 22.0;
/// How far up from a block's bottom edge its end can be grabbed.
const END_HANDLE: f32 = 5.0;
/// Narrowest a column of overlapping events may get, so the start of its title stays visible.
const MIN_BLOCK_WIDTH: f32 = 18.0;
/// Lower than this, an event shows its title and time on one line.
const SHORT_BLOCK: f32 = 30.0;
/// Lowest an event is drawn, however short it is.
const MIN_BLOCK_HEIGHT: f32 = 18.0;

pub fn color(c: Rgb) -> Color32 {
    Color32::from_rgb(c.0, c.1, c.2)
}

/// The colour of text on `c`: dark or light, whichever reads better on it (`MOD-04-U04`).
pub fn text_on(c: Rgb) -> Color32 {
    color(contrast::text_on(c))
}

/// The view's background, which the panels are filled with.
fn background(painter: &egui::Painter) -> Rgb {
    let fill = painter.ctx().global_style().visuals.panel_fill;
    Rgb(fill.r(), fill.g(), fill.b())
}

/// What an event is called in the views: its title, or "Busy" for an event of a calendar shared
/// as free/busy only, after a lock when its calendar cannot be changed (`MOD-02-U01`), and marked
/// while Google has not confirmed a new event (`MOD-05-U03`).
pub fn event_label(title: Option<&str>, read_only: bool, saving: bool) -> String {
    let title = title.unwrap_or(texts::BUSY);
    match (read_only, saving) {
        (true, _) => format!("🔒 {title}"),
        (false, true) => format!("{title} · {SAVING}"),
        (false, false) => title.to_string(),
    }
}

/// Draws an event's `text` at `pos`, like `Painter::text`, on one line `width` wide at most,
/// ending in "…" when it does not fit; a declined event's struck through (`MOD-06-F14`).
#[expect(clippy::too_many_arguments)]
pub fn event_text(
    painter: &egui::Painter,
    pos: Pos2,
    align: Align2,
    text: String,
    font: FontId,
    ink: Color32,
    declined: bool,
    width: f32,
) {
    let mut job = LayoutJob::simple(text, font, ink, f32::INFINITY);
    job.wrap = egui::text::TextWrapping::truncate_at_width(width.max(0.0));
    if declined {
        job.sections[0].format.strikethrough = Stroke::new(1.0, ink);
    }
    let galley = painter.layout_job(job);
    let at = align.anchor_size(pos, galley.size()).min;
    painter.galley(at, galley, ink);
}

/// The lines of a view's grid: faint, so the events stand out from them.
pub fn grid_line(visuals: &egui::Visuals) -> Stroke {
    let line = visuals.widgets.noninteractive.bg_stroke;
    Stroke::new(line.width, line.color.gamma_multiply(0.6))
}

/// How much of the view's background veils a declined event.
const VEIL: f32 = 0.5;

/// Fades a declined event drawn in `rect` (`MOD-06-F14`) under a veil of the view's background;
/// drawn under its text, which stays readable (`MOD-04-NFR-01`).
pub fn fade(painter: &egui::Painter, rect: Rect) {
    painter.rect_filled(
        rect,
        CornerRadius::same(EVENT_RADIUS),
        color(background(painter)).gamma_multiply(VEIL),
    );
}

/// An event of colour `c` in `rect`: a tint of its colour with a strip of the colour itself along
/// its left edge, faded when it is declined; gives the colour its text is on, to pick the text's
/// colour from (`MOD-04-U04`).
pub fn filled(painter: &egui::Painter, rect: Rect, c: Rgb, declined: bool) -> Rgb {
    let under = tint(painter, c);
    painter.rect_filled(rect, CornerRadius::same(EVENT_RADIUS), color(under));
    strip(painter, rect, c);
    if !declined {
        return under;
    }
    fade(painter, rect);
    contrast::over(background(painter), VEIL, under)
}

/// How strongly an event's colour tints its fill, over the light and the dark background.
const TINT_LIGHT: f32 = 0.22;
const TINT_DARK: f32 = 0.4;
/// How wide the strip of an event's own colour along its left edge is.
pub const STRIP: f32 = 3.0;
/// How rounded events are.
pub const EVENT_RADIUS: u8 = 4;

/// The fill of an event of colour `c`: its colour, light over the view's background.
pub fn tint(painter: &egui::Painter, c: Rgb) -> Rgb {
    let strength = if painter.ctx().global_style().visuals.dark_mode {
        TINT_DARK
    } else {
        TINT_LIGHT
    };
    contrast::over(c, strength, background(painter))
}

/// The strip of an event's colour `c` along the left edge of `rect`.
pub fn strip(painter: &egui::Painter, rect: Rect, c: Rgb) {
    let strip = Rect::from_min_max(
        rect.left_top(),
        Pos2::new(rect.left() + STRIP, rect.bottom()),
    );
    let r = EVENT_RADIUS;
    let corners = CornerRadius {
        nw: r,
        sw: r,
        ne: 0,
        se: 0,
    };
    painter.rect_filled(strip, corners, color(c));
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

/// Clears the stripes of a hatched event from under its `text`, to be drawn at `pos`, with the
/// colour `c` under them: the text reads on that colour alone (`MOD-04-NFR-01`).
pub fn clear_under(
    painter: &egui::Painter,
    pos: Pos2,
    align: Align2,
    text: &str,
    font: FontId,
    c: Rgb,
) {
    let size = painter
        .layout_no_wrap(text.to_owned(), font, Color32::PLACEHOLDER)
        .size();
    // Also under the ends of a strikethrough.
    let area = align.anchor_size(pos, size).expand(2.0);
    painter.rect_filled(area, CornerRadius::ZERO, color(c));
}

/// How long the dashes of an invitation's outline are, and the gaps between them.
const DASH: f32 = 4.0;
/// An invitation awaiting the user's answer (`MOD-06-U04`): the view's background inside a dashed
/// outline of its colour `c`, unlike the tinted events answered; gives the colour of the fill, to
/// pick its text's colour from (`MOD-04-U04`).
pub fn awaiting(painter: &egui::Painter, rect: Rect, c: Rgb) -> Rgb {
    let under = background(painter);
    // Square, as the dashes are.
    painter.rect_filled(rect, CornerRadius::ZERO, color(under));
    let rect = rect.shrink(0.5);
    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    let outline = Stroke::new(1.0, color(c));
    painter.extend(egui::Shape::dashed_line(&corners, outline, DASH, DASH));
    under
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
            CornerRadius::same(EVENT_RADIUS),
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
    let grid_line = grid_line(&visuals);
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
        // The weekday small above the day's number; today's number on a disc of the accent.
        // Alone in the day view, at the start of its column, by the hours, as in Google Calendar.
        let top = if view.days.len() == 1 {
            rect.left_top() + Vec2::new(24.0, 0.0)
        } else {
            rect.center_top()
        };
        let accent = visuals.selection.stroke.color;
        painter.text(
            top + Vec2::new(0.0, 6.0),
            Align2::CENTER_TOP,
            regional.weekday(day.date).to_uppercase(),
            FontId::proportional(11.0),
            if day.is_today { accent } else { weak_text },
        );
        let number = top + Vec2::new(0.0, 34.0);
        let ink = if day.is_today {
            painter.circle_filled(number, 14.0, accent);
            text_on(Rgb(accent.r(), accent.g(), accent.b()))
        } else {
            visuals.strong_text_color()
        };
        painter.text(
            number,
            Align2::CENTER_CENTER,
            day.date.day().to_string(),
            crate::appearance::strong(18.0),
            ink,
        );
    }

    // All-day and multi-day strip.
    let strip_height = view.lanes as f32 * LANE_HEIGHT + 4.0;
    // The hours right under the strip.
    ui.spacing_mut().item_spacing.y = 0.0;
    let (strip, _) = ui.allocate_exact_size(Vec2::new(width, strip_height), Sense::hover());
    // Above the hours, a dragged event is over the strip.
    let over_strip = pointer.is_some_and(|p| p.y < strip.bottom());
    let painter = ui.painter_at(strip);
    // The days' lines go on through the strip, under its bars, so each day reads as a column
    // from its date down.
    for i in 0..=columns {
        let x = strip.left() + GUTTER + i as f32 * day_width;
        painter.vline(x, strip.y_range(), grid_line);
    }
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
        let (ink, under) = if bar.label {
            let outline = Stroke::new(1.0, color(bar.color));
            painter.rect_stroke(
                rect,
                CornerRadius::same(EVENT_RADIUS),
                outline,
                StrokeKind::Inside,
            );
            (visuals.text_color(), background(&painter))
        } else {
            let under = if bar.awaiting {
                awaiting(&painter, rect, bar.color)
            } else {
                filled(&painter, rect, bar.color, bar.declined)
            };
            (text_on(under), under)
        };
        let painter = painter.with_clip_rect(rect);
        let at = rect.left_center() + Vec2::new(STRIP + 5.0, 0.0);
        let label = event_label(bar.title.as_deref(), bar.read_only, bar.saving);
        let font = crate::appearance::strong(12.0);
        if bar.hatched {
            hatch(&painter, rect, ink);
            clear_under(
                &painter,
                at,
                Align2::LEFT_CENTER,
                &label,
                font.clone(),
                under,
            );
        }
        event_text(
            &painter,
            at,
            Align2::LEFT_CENTER,
            label,
            font,
            ink,
            bar.declined,
            rect.right() - at.x - 4.0,
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
    // The hours' lines go on from the strip, not faded at its edge.
    ui.spacing_mut().scroll.fade.strength = 0.0;

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
                Pos2::new(x + column_width - 3.0, minute_y(p.end_min as f32) - 1.0),
            );
            // A short event stays tall enough for a line of its title.
            rect.max.y = rect.max.y.max(rect.min.y + MIN_BLOCK_HEIGHT);
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
            let under = if block.awaiting {
                awaiting(&painter, rect, block.color)
            } else {
                filled(&painter, rect, block.color, block.declined)
            };
            let ink = text_on(under);
            let clipped = painter.with_clip_rect(rect.shrink(1.0));
            let name = event_label(block.title.as_deref(), block.read_only, block.saving);
            let time = match times {
                Some((start, end)) => {
                    let time =
                        |at: DateTime<Utc>| regional.time(at.with_timezone(&view.zone).time());
                    format!("{} – {}", time(start), time(end))
                }
                None => regional.time(block.start.with_timezone(&view.zone).time()),
            };
            // The title in semibold, and the time under it; on one line in a short event.
            let short = rect.height() < SHORT_BLOCK;
            let mut lines = vec![(name, crate::appearance::strong(12.0))];
            if block.title.is_some() {
                lines.push((time, FontId::proportional(11.5)));
            }
            if short && lines.len() == 2 {
                let time = lines.pop().map(|(time, _)| time).unwrap_or_default();
                lines[0].0 = format!("{}  {time}", lines[0].0);
                lines[0].1 = crate::appearance::strong(11.0);
            }
            let left = rect.left() + STRIP + 4.0;
            let mut y = if short {
                rect.center().y - 7.0
            } else {
                rect.top() + 3.0
            };
            if block.hatched {
                hatch(&painter, rect, ink);
            }
            for (text, font) in lines {
                let at = Pos2::new(left, y);
                y += font.size + 3.0;
                if block.hatched {
                    clear_under(&clipped, at, Align2::LEFT_TOP, &text, font.clone(), under);
                }
                event_text(
                    &clipped,
                    at,
                    Align2::LEFT_TOP,
                    text,
                    font,
                    ink,
                    block.declined,
                    rect.right() - left - 3.0,
                );
            }
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
            // In the accent, the one colour of the interface (`MOD-04-U03`).
            let accent = visuals.selection.stroke.color;
            painter.hline(x..=(x + day_width), y, Stroke::new(2.0, accent));
            painter.circle_filled(Pos2::new(x, y), 4.0, accent);
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
    use crate::screen::{self, VERY_DARK, VERY_LIGHT, answered, contrast_of, texts};
    use calendar_core::contrast::AA;
    use calendar_core::fixtures::{d, tc_03_01_week, timed};
    use calendar_core::guests::Response;
    use calendar_core::model::{Event, Kind};
    use calendar_core::theme::Scheme;
    use calendar_core::view_model::grid_view;
    use chrono::TimeZone;
    use chrono_tz::Europe::Warsaw;
    use egui::epaint::{ClippedShape, Shape};

    /// What a window, `CHROME` points of which go to its title bar and the app's view bar, draws
    /// of the week of 9–15 November 2026 with `events`, scrolled towards 8:00 at the top; today is
    /// Wednesday 11.
    fn drawn(events: &[Event], window: Vec2, scale: f32, scheme: Scheme) -> Vec<ClippedShape> {
        let dates: Vec<_> = (9..=15).map(|day| d(2026, 11, day)).collect();
        let now = Utc.with_ymd_and_hms(2026, 11, 11, 12, 0, 0).unwrap();
        let mut view = grid_view(events, &dates, Warsaw, now);
        view.initial_scroll_minute = 9 * 60;
        let regional = Regional::from_env(|_| None);
        let mut state = State::default();
        let window = (window - Vec2::new(0.0, CHROME)) / scale;
        screen::drawn(window, scale, scheme, None, |ui| {
            show(ui, &view, &regional, &mut state, true, &mut None);
        })
    }

    /// Room for the window's title bar and the app's view bar above the grid: more than either
    /// takes (`DEC-149`).
    const CHROME: f32 = 100.0;
    const FULL_HD: Vec2 = Vec2::new(1920.0, 1080.0);

    /// Where the hours' line of `hour` is drawn, from its label, and the clip it is drawn in.
    fn hour_line(shapes: &[ClippedShape], hour: u32) -> (f32, Rect) {
        let label = Regional::from_env(|_| None).hour(hour);
        let (clip, text) = texts(shapes)
            .into_iter()
            .find(|(_, text)| text.galley.text() == label)
            .unwrap_or_else(|| panic!("{label} is drawn"));
        (text.visual_bounding_rect().center().y, clip)
    }

    #[test]
    fn tc_04_03_a_1080_pixel_window_at_100_percent_shows_8_to_18_without_scrolling() {
        let shapes = drawn(&tc_03_01_week(), FULL_HD, 1.0, Scheme::Light);

        // As far towards 8:00 at the top as the hours scroll: in this window they end at midnight
        // before 8:00 gets there.
        let (eight, shown) = hour_line(&shapes, 8);
        let (eighteen, _) = hour_line(&shapes, 18);
        let (midnight, _) = hour_line(&shapes, 23);
        assert!(midnight + HOUR_HEIGHT <= shown.bottom() + 1.0);
        assert!(
            shown.top() <= eight && eighteen <= shown.bottom(),
            "8:00 at {eight} and 18:00 at {eighteen}, the hours shown are {shown:?}"
        );
        assert_eq!(eighteen - eight, 10.0 * HOUR_HEIGHT);
    }

    #[test]
    fn at_150_and_200_percent_the_grid_scales_and_keeps_its_proportions() {
        let lines = |scale: f32| {
            let shapes = drawn(&tc_03_01_week(), FULL_HD, scale, Scheme::Light);
            let (nine, _) = hour_line(&shapes, 9);
            let (ten, _) = hour_line(&shapes, 10);
            let label = texts(&shapes)
                .into_iter()
                .find(|(_, text)| text.galley.text().starts_with("Standup"))
                .map(|(_, text)| text.galley.size())
                .expect("the event is drawn");
            // In pixels on the screen.
            ((ten - nine) * scale, label * scale)
        };
        let (hour, label) = lines(1.0);
        for scale in [1.5, 2.0] {
            let (scaled_hour, scaled_label) = lines(scale);
            assert_eq!(scaled_hour, hour * scale);
            // Line heights are rounded to whole pixels, so up to one pixel apart.
            assert!((scaled_label - label * scale).length() <= 1.0);
        }
    }

    #[test]
    fn an_event_block_shows_only_its_title_and_time() {
        let mut standup = timed("Standup", d(2026, 11, 10), (9, 0), (10, 0));
        standup.about.location = "Room 5".to_string();
        standup.about.description = "Agenda: the sprint".to_string();
        standup.about.meet = Some("https://meet.google.com/abc-defg-hij".to_string());

        let shapes = drawn(&[standup], FULL_HD, 1.0, Scheme::Light);

        let nine =
            Regional::from_env(|_| None).time(chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        // Right of the hours' labels, in the days.
        let shown: Vec<String> = texts(&shapes)
            .into_iter()
            .filter(|(_, text)| text.pos.x > GUTTER)
            .map(|(_, text)| text.galley.text().to_string())
            .filter(|text| {
                text.contains("Standup")
                    || text.contains("Room")
                    || text.contains("meet")
                    || *text == nine
            })
            .collect();
        assert_eq!(shown, ["Standup".to_string(), nine]);
    }

    #[test]
    fn tc_04_04_text_on_a_very_light_and_a_very_dark_calendar_reads_at_4_5_in_both_themes() {
        let mut light = timed("Light", d(2026, 11, 10), (9, 0), (10, 0));
        light.color = VERY_LIGHT;
        let mut dark = timed("Dark", d(2026, 11, 11), (9, 0), (10, 0));
        dark.color = VERY_DARK;

        for scheme in [Scheme::Light, Scheme::Dark] {
            let shapes = drawn(&[light.clone(), dark.clone()], FULL_HD, 1.0, scheme);
            for title in ["Light", "Dark"] {
                let ratio = contrast_of(&shapes, title);
                assert!(ratio >= AA, "{scheme:?}, {title}: {ratio}");
            }
        }
    }

    #[test]
    fn text_of_invitations_and_declined_events_reads_at_4_5_in_both_themes() {
        let at = |title: &str, day| timed(title, d(2026, 11, day), (9, 0), (10, 0));
        let events = [
            answered(at("Invited light", 9), VERY_LIGHT, Response::Awaiting),
            answered(at("Invited dark", 10), VERY_DARK, Response::Awaiting),
            answered(at("Declined light", 11), VERY_LIGHT, Response::Declined),
            answered(at("Declined dark", 12), VERY_DARK, Response::Declined),
        ];
        assert!(events[0].about.awaiting() && events[2].about.declined());

        for scheme in [Scheme::Light, Scheme::Dark] {
            let shapes = drawn(&events, FULL_HD, 1.0, scheme);
            for event in &events {
                let ratio = contrast_of(&shapes, &event.title);
                assert!(ratio >= AA, "{scheme:?}, {}: {ratio}", event.title);
            }
        }
    }

    #[test]
    fn todays_date_in_the_day_header_reads_at_4_5_on_its_mark_in_both_themes() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let shapes = drawn(&[], FULL_HD, 1.0, scheme);
            let ratio = screen::contrast_where(&shapes, |text| text == "11");
            assert!(ratio >= AA, "{scheme:?}: {ratio}");
        }
    }

    #[test]
    fn apart_from_calendar_colours_the_week_draws_in_grays_and_the_accent_in_both_themes() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let shapes = drawn(&[], FULL_HD, 1.0, scheme);
            let mut colors = Vec::new();
            for (_, shape) in screen::in_order(&shapes) {
                match shape {
                    Shape::Rect(r) => colors.extend([r.fill, r.stroke.color]),
                    Shape::Circle(c) => colors.extend([c.fill, c.stroke.color]),
                    Shape::LineSegment { stroke, .. } => colors.push(stroke.color),
                    Shape::Path(p) => {
                        colors.push(p.fill);
                        if let egui::epaint::ColorMode::Solid(c) = p.stroke.color {
                            colors.push(c);
                        }
                    }
                    Shape::Text(t) => colors.push(color(screen::ink(&t))),
                    _ => {}
                }
            }
            // The current time's line is drawn, in the accent.
            let accent = crate::appearance::visuals(scheme).selection.stroke.color;
            assert!(
                colors.contains(&accent),
                "{scheme:?}: no line in the accent"
            );
            for c in colors {
                assert!(screen::neutral_or_accent(c), "{scheme:?}: {c:?}");
            }
        }
    }

    #[test]
    fn the_stripes_of_an_out_of_office_event_do_not_pass_under_its_text() {
        let away = |title: &str, day, until, color, response| {
            let mut event = timed(title, d(2026, 11, day), (9, 0), until);
            event.about.kind = Kind::OutOfOffice;
            answered(event, color, response)
        };
        // A middle gray, on which black text only just reaches 4.5:1.
        let gray = Rgb(118, 118, 118);
        let events = [
            away("Away", 10, (12, 0), gray, Response::Accepted),
            // Over two days, in the all-day strip.
            away("Trip", 11, (24 + 12, 0), gray, Response::Accepted),
            away("Declined", 13, (12, 0), VERY_DARK, Response::Declined),
            away("Invited", 14, (12, 0), VERY_DARK, Response::Awaiting),
        ];

        for scheme in [Scheme::Light, Scheme::Dark] {
            let shown = screen::in_order(&drawn(&events, FULL_HD, 1.0, scheme));
            for event in &events {
                let text = shown
                    .iter()
                    .position(|(_, shape)| {
                        matches!(shape, Shape::Text(t) if t.galley.text().contains(&event.title))
                    })
                    .expect("the event is drawn");
                let Shape::Text(label) = &shown[text].1 else {
                    unreachable!("found as a text")
                };
                let area = label.visual_bounding_rect();
                let stripe = shown[..text].iter().rposition(|(_, shape)| match shape {
                    Shape::LineSegment { points, .. } => {
                        Rect::from_two_pos(points[0], points[1]).intersects(area)
                    }
                    _ => false,
                });
                let cleared = shown[..text]
                    .iter()
                    .rposition(|(_, s)| matches!(s, Shape::Rect(r) if r.rect.contains_rect(area)));
                let title = &event.title;
                assert!(stripe.is_some(), "{scheme:?}, {title} is striped");
                assert!(
                    cleared > stripe,
                    "{scheme:?}, {title}: stripes under the text"
                );
                let Shape::Rect(clear) = &shown[cleared.unwrap()].1 else {
                    unreachable!("found as a rectangle")
                };
                let fill = Rgb(clear.fill.r(), clear.fill.g(), clear.fill.b());
                let ratio = calendar_core::contrast::contrast(screen::ink(label), fill);
                assert!(ratio >= AA, "{scheme:?}, {title}: {ratio}");
            }
        }
    }

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
