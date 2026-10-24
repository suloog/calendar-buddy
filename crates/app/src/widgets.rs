//! The few controls the app draws its own way: the view picker as one segmented control, quiet
//! icon buttons and the outlined "Today".

use eframe::egui::{self, Button, CornerRadius, Frame, Margin, Response, RichText, Stroke};

use crate::appearance::RADIUS;

/// A button with no fill until hovered, for icons in the bars; `hint` says what it does.
pub fn icon_button(ui: &mut egui::Ui, icon: &str, hint: &str) -> Response {
    ui.add(Button::new(RichText::new(icon).size(16.0)).frame_when_inactive(false))
        .on_hover_text(hint)
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
