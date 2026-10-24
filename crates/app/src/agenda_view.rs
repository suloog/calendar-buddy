//! Draws an `AgendaViewModel` with egui. Only drawing and input live here; which events are
//! upcoming and on which days is decided in `calendar_core::agenda`. An event with a Google Meet
//! call has a Join button (`MOD-06-F18`).

use calendar_core::agenda::{AgendaItem, AgendaViewModel};
use calendar_core::regional::Regional;
use eframe::egui::{self, Align2, FontId, Pos2, ScrollArea, Sense, Vec2};

use crate::grid_view::{awaiting, color, event_label, event_text, fade, openable, text_on};

const DATE_WIDTH: f32 = 96.0;
const TIME_WIDTH: f32 = 140.0;
const ITEM_HEIGHT: f32 = 24.0;
const DAY_GAP: f32 = 8.0;
const JOIN_WIDTH: f32 = 48.0;
/// How close to the end of the list scrolling has to get before more days load.
const LOAD_MARGIN: f32 = 200.0;

fn time_label(item: &AgendaItem, view: &AgendaViewModel, regional: &Regional) -> String {
    let time = |t: chrono::DateTime<chrono::Utc>| regional.time(t.with_timezone(&view.zone).time());
    match item.time {
        Some((start, end)) => format!("{} – {}", time(start), time(end)),
        None => "All day".to_string(),
    }
}

/// Returns true when the user has scrolled near the end and more days should be loaded. Join
/// gives the link of the Google Meet call to open in `join`.
pub fn show(
    ui: &mut egui::Ui,
    view: &AgendaViewModel,
    regional: &Regional,
    opened: &mut Option<usize>,
    join: &mut Option<String>,
) -> bool {
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
            // Days scrolled out of sight are not drawn, however far the list has grown. Those
            // within a screen of it still take the keyboard, so Tab can scroll on to them.
            let screen = Vec2::new(0.0, ui.clip_rect().height());
            if !ui.is_rect_visible(rect.expand2(screen)) {
                continue;
            }
            for (n, item) in day.items.iter().enumerate() {
                let row = egui::Rect::from_min_size(
                    Pos2::new(
                        rect.left() + DATE_WIDTH,
                        rect.top() + n as f32 * ITEM_HEIGHT,
                    ),
                    Vec2::new(rect.width() - DATE_WIDTH, ITEM_HEIGHT),
                );
                let response = ui.interact(
                    row,
                    ui.scope_id().with(("item", day.date, n)),
                    Sense::click(),
                );
                openable(ui, &response, row, item.event, opened);
            }
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
                regional.month_weekday(day.date),
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
                let row = egui::Rect::from_center_size(
                    Pos2::new((x + rect.right()) / 2.0, y),
                    Vec2::new(rect.right() - x, ITEM_HEIGHT - 2.0),
                );
                // An invitation awaiting an answer (`MOD-06-U04`).
                let ink = if item.awaiting {
                    text_on(awaiting(&painter, row, item.color))
                } else {
                    visuals.text_color()
                };
                painter.circle_filled(Pos2::new(x + 6.0, y), 5.0, color(item.color));
                // A declined event, struck through and faded (`MOD-06-F14`).
                if item.declined {
                    fade(&painter, row);
                }
                painter.text(
                    Pos2::new(x + 20.0, y),
                    Align2::LEFT_CENTER,
                    time_label(item, view, regional),
                    FontId::proportional(13.0),
                    ink,
                );
                event_text(
                    &painter,
                    Pos2::new(x + 20.0 + TIME_WIDTH, y),
                    Align2::LEFT_CENTER,
                    event_label(item.title.as_deref(), item.read_only, item.saving),
                    FontId::proportional(13.0),
                    ink,
                    item.declined,
                );
                // Join opens its Google Meet call in the browser (`MOD-06-F18`).
                if let Some(link) = &item.meet {
                    let button = egui::Rect::from_min_max(
                        Pos2::new(row.right() - JOIN_WIDTH, row.top()),
                        row.max,
                    );
                    // Placed over the row, not laid out: the day's room is taken already.
                    if ui.place(button, egui::Button::new("Join")).clicked() {
                        *join = Some(link.clone());
                    }
                }
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
    use crate::screen::{self, VERY_DARK, VERY_LIGHT, answered, contrast_of};
    use calendar_core::agenda::agenda_view;
    use calendar_core::contrast::AA;
    use calendar_core::fixtures::{d, timed};
    use calendar_core::guests::Response;
    use calendar_core::theme::Scheme;
    use chrono::{TimeZone, Utc};
    use chrono_tz::Europe::Warsaw;

    #[test]
    fn text_of_invitations_and_declined_events_reads_at_4_5_in_both_themes() {
        let at = |title: &str, day| timed(title, d(2026, 11, day), (13, 0), (14, 0));
        let events = [
            answered(at("Invited light", 11), VERY_LIGHT, Response::Awaiting),
            answered(at("Invited dark", 12), VERY_DARK, Response::Awaiting),
            answered(at("Declined light", 13), VERY_LIGHT, Response::Declined),
            answered(at("Declined dark", 14), VERY_DARK, Response::Declined),
        ];
        let now = Utc.with_ymd_and_hms(2026, 11, 11, 9, 0, 0).unwrap();
        let view = agenda_view(&events, Warsaw, now, 14);
        let regional = Regional::from_env(|_| None);

        for scheme in [Scheme::Light, Scheme::Dark] {
            let shapes = screen::drawn(Vec2::new(1600.0, 900.0), 1.0, scheme, None, |ui| {
                show(ui, &view, &regional, &mut None, &mut None);
            });
            for event in &events {
                let ratio = contrast_of(&shapes, &event.title);
                assert!(ratio >= AA, "{scheme:?}, {}: {ratio}", event.title);
            }
        }
    }

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
