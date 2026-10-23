//! Draws an `AgendaViewModel` with egui. Only drawing and input live here; which events are
//! upcoming and on which days is decided in `calendar_core::agenda`.

use calendar_core::agenda::{AgendaItem, AgendaViewModel};
use eframe::egui::{self, Align2, FontId, Pos2, ScrollArea, Sense, Vec2};

use crate::grid_view::color;

const DATE_WIDTH: f32 = 96.0;
const TIME_WIDTH: f32 = 110.0;
const ITEM_HEIGHT: f32 = 24.0;
const DAY_GAP: f32 = 8.0;
/// How close to the end of the list scrolling has to get before more days load.
const LOAD_MARGIN: f32 = 200.0;

fn time_label(item: &AgendaItem, view: &AgendaViewModel) -> String {
    match item.time {
        Some((start, end)) => format!(
            "{} – {}",
            start.with_timezone(&view.zone).format("%H:%M"),
            end.with_timezone(&view.zone).format("%H:%M")
        ),
        None => "All day".to_string(),
    }
}

/// Returns true when the user has scrolled near the end and more days should be loaded.
pub fn show(ui: &mut egui::Ui, view: &AgendaViewModel) -> bool {
    if view.is_empty() {
        ui.centered_and_justified(|ui| ui.label("No upcoming events"));
        return false;
    }
    let output = ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        let visuals = ui.visuals().clone();
        for day in &view.days {
            let height = day.items.len() as f32 * ITEM_HEIGHT;
            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), height + DAY_GAP),
                Sense::hover(),
            );
            // Days scrolled out of sight are not drawn, however far the list has grown.
            if !ui.is_rect_visible(rect) {
                continue;
            }
            let painter = ui.painter_at(rect);
            ui.painter().hline(
                rect.x_range(),
                rect.bottom(),
                visuals.widgets.noninteractive.bg_stroke,
            );

            let date_center = Pos2::new(rect.left() + 16.0, rect.top() + ITEM_HEIGHT / 2.0);
            let date_color = if day.is_today {
                painter.circle_filled(date_center, 11.0, visuals.selection.bg_fill);
                visuals.strong_text_color()
            } else {
                visuals.text_color()
            };
            painter.text(
                date_center,
                Align2::CENTER_CENTER,
                day.date.format("%-d").to_string(),
                FontId::proportional(14.0),
                date_color,
            );
            painter.text(
                date_center + Vec2::new(18.0, 0.0),
                Align2::LEFT_CENTER,
                day.date.format("%b, %a").to_string().to_uppercase(),
                FontId::proportional(11.0),
                if day.is_today {
                    visuals.strong_text_color()
                } else {
                    visuals.weak_text_color()
                },
            );

            for (n, item) in day.items.iter().enumerate() {
                let y = rect.top() + (n as f32 + 0.5) * ITEM_HEIGHT;
                let x = rect.left() + DATE_WIDTH;
                painter.circle_filled(Pos2::new(x + 6.0, y), 5.0, color(item.color));
                painter.text(
                    Pos2::new(x + 20.0, y),
                    Align2::LEFT_CENTER,
                    time_label(item, view),
                    FontId::proportional(13.0),
                    visuals.text_color(),
                );
                painter.text(
                    Pos2::new(x + 20.0 + TIME_WIDTH, y),
                    Align2::LEFT_CENTER,
                    item.title.as_deref().unwrap_or("Busy"),
                    FontId::proportional(13.0),
                    visuals.text_color(),
                );
            }
        }
    });
    let seen_to = output.state.clamped_offset().y + output.inner_rect.height();
    view.has_more && near_end(seen_to, output.content_size.y)
}

/// Whether the list has been scrolled (or is short enough) to within `LOAD_MARGIN` of its end.
fn near_end(seen_to: f32, content_height: f32) -> bool {
    seen_to >= content_height - LOAD_MARGIN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn more_days_load_only_when_scrolled_near_the_end() {
        // 600 points of a 5,000-point list on screen.
        assert!(!near_end(600.0, 5000.0));
        assert!(!near_end(4700.0, 5000.0));
        assert!(near_end(4800.0, 5000.0));
        // A list shorter than the window keeps loading until it fills it.
        assert!(near_end(600.0, 300.0));
    }
}
