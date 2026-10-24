//! The icons of the bars and dialogs, drawn as thin lines in one box, so they are of one size and
//! weight, which the emoji font's are not.

use eframe::egui::{self, Button, Color32, Painter, Pos2, Response, RichText, Shape, Stroke, Vec2};
use std::f32::consts::{PI, TAU};

/// How thick an icon's lines are.
const LINE: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Menu,
    Previous,
    Next,
    Calendar,
    Eye,
    Refresh,
    Help,
    Settings,
    Close,
}

/// A button with no fill until hovered, `icon` drawn on it, `hint` saying what it does; grayed and
/// doing nothing unless `enabled`. Every icon button has the same size.
pub fn button(ui: &mut egui::Ui, icon: Icon, hint: &str, enabled: bool) -> Response {
    turned(ui, icon, hint, enabled, 0.0)
}

/// A `button` whose icon is turned by `angle`, as Refresh while synchronizing.
pub fn turned(ui: &mut egui::Ui, icon: Icon, hint: &str, enabled: bool, angle: f32) -> Response {
    // The room of a 16-point glyph, which the icon is drawn over.
    let room = RichText::new("🗑").size(16.0).color(Color32::TRANSPARENT);
    let response = ui
        .add_enabled(enabled, Button::new(room).frame_when_inactive(false))
        .on_hover_text(hint);
    let color = ui.visuals().widgets.style(&response).text_color();
    paint(ui.painter(), icon, response.rect.center(), color, angle);
    response
}

/// Draws `icon` about `center`, in a box of 16 points, turned by `angle`.
pub fn paint(painter: &Painter, icon: Icon, center: Pos2, color: Color32, angle: f32) {
    let line = Stroke::new(LINE, color);
    let (sin, cos) = angle.sin_cos();
    let at = |x: f32, y: f32| center + Vec2::new(x * cos - y * sin, x * sin + y * cos);
    let path = |points: Vec<Pos2>| painter.add(Shape::line(points, line));
    match icon {
        Icon::Menu => {
            for y in [-5.0, 0.0, 5.0] {
                path(vec![at(-7.0, y), at(7.0, y)]);
            }
        }
        Icon::Previous => {
            path(vec![at(2.5, -5.5), at(-3.0, 0.0), at(2.5, 5.5)]);
        }
        Icon::Next => {
            path(vec![at(-2.5, -5.5), at(3.0, 0.0), at(-2.5, 5.5)]);
        }
        Icon::Calendar => {
            let page =
                egui::Rect::from_center_size(center + Vec2::new(0.0, 1.0), Vec2::new(14.0, 13.0));
            painter.rect_stroke(page, 2.0, line, egui::StrokeKind::Middle);
            path(vec![at(-7.0, -1.5), at(7.0, -1.5)]);
            for x in [-3.5, 3.5] {
                path(vec![at(x, -7.0), at(x, -3.5)]);
            }
        }
        Icon::Eye => {
            let lid = |side: f32| {
                (0..=16)
                    .map(|i| {
                        let x = -8.0 + i as f32;
                        at(x, side * 4.5 * (1.0 - (x / 8.0).powi(2)))
                    })
                    .collect()
            };
            path(lid(-1.0));
            path(lid(1.0));
            painter.circle_stroke(center, 2.2, line);
        }
        Icon::Refresh => {
            // A circle's arrow, open at its top right, its head at the end.
            let radius = 6.0;
            let (from, to) = (-PI / 3.0, 4.0 * PI / 3.0);
            let point = |a: f32| at(radius * a.cos(), radius * a.sin());
            path(
                (0..=24)
                    .map(|i| point(from + (to - from) * i as f32 / 24.0))
                    .collect(),
            );
            let end = Vec2::new(to.cos(), to.sin()) * radius;
            let along = Vec2::new(-to.sin(), to.cos());
            let out = Vec2::new(to.cos(), to.sin());
            let head = |v: Vec2| at(v.x, v.y);
            let tip = [
                head(end + along * 2.5),
                head(end - along * 1.5 + out * 3.0),
                head(end - along * 1.5 - out * 3.0),
            ];
            painter.add(Shape::convex_polygon(tip.to_vec(), color, Stroke::NONE));
        }
        Icon::Help => {
            painter.circle_stroke(center, 7.5, line);
            let font = crate::appearance::strong(11.0);
            painter.text(center, egui::Align2::CENTER_CENTER, "?", font, color);
        }
        Icon::Settings => {
            painter.circle_stroke(center, 2.5, line);
            painter.circle_stroke(center, 5.5, line);
            let teeth = Stroke::new(2.2, color);
            for i in 0..8 {
                let a = TAU * i as f32 / 8.0;
                let (s, c) = a.sin_cos();
                painter.line_segment([at(c * 5.5, s * 5.5), at(c * 7.8, s * 7.8)], teeth);
            }
        }
        Icon::Close => {
            path(vec![at(-5.0, -5.0), at(5.0, 5.0)]);
            path(vec![at(5.0, -5.0), at(-5.0, 5.0)]);
        }
    }
}
