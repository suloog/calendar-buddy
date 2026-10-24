//! The few controls the app draws its own way: the view picker as one segmented control, quiet
//! icon buttons and the outlined "Today".

use eframe::egui::{self, Button, CornerRadius, Frame, Margin, Response, RichText, Stroke};

use crate::appearance::RADIUS;

/// A button with no fill until hovered, for icons in the bars; `hint` says what it does.
pub fn icon_button(ui: &mut egui::Ui, icon: &str, hint: &str) -> Response {
    ui.add(Button::new(RichText::new(icon).size(16.0)).frame_when_inactive(false))
        .on_hover_text(hint)
}

/// How long the refresh icon takes to turn once while synchronizing.
const TURN: f64 = 1.2;

/// An `icon_button` whose icon turns while `turning`: the refresh button while synchronizing.
/// Not a transition (`MOD-04-U05`): it shows work under way, as long as it lasts (`DEC-154`).
pub fn refresh_button(ui: &mut egui::Ui, icon: &str, hint: &str, turning: bool) -> Response {
    if !turning {
        return icon_button(ui, icon, hint);
    }
    // The button keeps its place and size; its icon is drawn turned over it.
    let hidden = RichText::new(icon)
        .size(16.0)
        .color(egui::Color32::TRANSPARENT);
    let response = ui
        .add(Button::new(hidden).frame_when_inactive(false))
        .on_hover_text(hint);
    let color = ui.visuals().widgets.style(&response).text_color();
    let galley =
        ui.painter()
            .layout_no_wrap(icon.to_owned(), egui::FontId::proportional(16.0), color);
    let time = ui.input(|i| i.time);
    let angle = (time % TURN / TURN) as f32 * std::f32::consts::TAU;
    let at = response.rect.center() - galley.size() / 2.0;
    ui.painter().add(
        egui::epaint::TextShape::new(at, galley, color)
            .with_angle_and_anchor(angle, egui::Align2::CENTER_CENTER),
    );
    ui.ctx().request_repaint();
    response
}

/// An `icon_button` that stays filled while `on`: a switch in the bars.
pub fn icon_switch(ui: &mut egui::Ui, icon: &str, on: bool, hint: &str) -> Response {
    ui.add(Button::selectable(on, RichText::new(icon).size(16.0)))
        .on_hover_text(hint)
}

/// Text in the small weak style, right-aligned in a width that fits each of `widest` too, so
/// what stands left of it keeps its place as the text changes.
pub fn steady_label(ui: &mut egui::Ui, text: RichText, widest: &[&str]) -> Response {
    let font = egui::TextStyle::Small.resolve(ui.style());
    let measure = |text: &str| {
        ui.painter()
            .layout_no_wrap(text.to_owned(), font.clone(), egui::Color32::PLACEHOLDER)
            .size()
            .x
    };
    let width = widest
        .iter()
        .map(|text| measure(text))
        .fold(measure(text.text()), f32::max);
    let size = egui::vec2(width, ui.spacing().interact_size.y);
    let layout = egui::Layout::right_to_left(egui::Align::Center);
    // The area keeps the whole width, not only the text's.
    ui.allocate_ui_with_layout(size, layout, |ui| {
        ui.set_min_size(size);
        ui.label(text)
    })
    .inner
}

/// A button outlined in the lines' gray, with no fill until hovered.
pub fn outlined_button(ui: &mut egui::Ui, text: &str) -> Response {
    let line = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.add(
        Button::new(text)
            .fill(egui::Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, line.color)),
    )
}

/// The one button of a form that does what it is for, filled with the accent.
pub fn primary<'a>(ui: &egui::Ui, text: &'a str) -> Button<'a> {
    let accent = ui.visuals().selection.stroke.color;
    let ink = crate::grid_view::text_on(calendar_core::model::Rgb(
        accent.r(),
        accent.g(),
        accent.b(),
    ));
    Button::new(RichText::new(text).color(ink)).fill(accent)
}

/// A menu behind a quiet "…" button, filled by `content`.
pub fn menu_dots(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    egui::containers::menu::MenuButton::from_button(Button::new("…").frame_when_inactive(false))
        .ui(ui, content);
}

/// One of `names` picked, shown as one control; gives the one clicked.
pub fn segmented(ui: &mut egui::Ui, names: &[&str], picked: usize) -> Option<usize> {
    let visuals = ui.visuals().clone();
    let mut clicked = None;
    Frame::new()
        .fill(visuals.widgets.inactive.weak_bg_fill)
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.spacing_mut().interact_size.y = 24.0;
            ui.spacing_mut().button_padding = egui::vec2(10.0, 3.0);
            // In the order given, also inside a right-to-left bar.
            let mut order: Vec<usize> = (0..names.len()).collect();
            if ui.layout().prefer_right_to_left() {
                order.reverse();
            }
            for i in order {
                let name = &names[i];
                {
                    let on = i == picked;
                    let button = if on {
                        Button::new(RichText::new(*name).color(visuals.strong_text_color()))
                            .fill(visuals.panel_fill)
                            .stroke(visuals.widgets.noninteractive.bg_stroke)
                    } else {
                        Button::new(*name).frame_when_inactive(false)
                    };
                    let response = ui.add(button.corner_radius(CornerRadius::same(RADIUS - 2)));
                    if response.clicked() {
                        clicked = Some(i);
                    }
                }
            }
        });
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Where a button left of the indicator showing `text` is drawn, in a right-to-left bar.
    fn left_of(text: &str) -> egui::Rect {
        let ctx = egui::Context::default();
        crate::appearance::install(&ctx);
        let mut drawn = egui::Rect::NOTHING;
        for _ in 0..2 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let widest = ["Synchronizing…", "Up to date (22:22)"];
                    steady_label(ui, RichText::new(text).small(), &widest);
                    drawn = ui.button("Week").rect;
                });
            });
            output.textures_delta.clear();
        }
        drawn
    }

    #[test]
    fn what_stands_left_of_the_indicator_keeps_its_place_as_its_text_changes() {
        let synchronizing = left_of("Synchronizing…");
        assert_eq!(left_of("Up to date (14:05)"), synchronizing);
        assert_eq!(left_of("Offline"), synchronizing);
    }
}
