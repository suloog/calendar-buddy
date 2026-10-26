//! The few controls the app draws its own way: the view picker as one segmented control, quiet
//! icon buttons and the outlined "Today".

use eframe::egui::{self, Button, CornerRadius, Frame, Margin, Response, RichText, Stroke};

use crate::appearance::RADIUS;
use crate::icons::{self, Icon};

/// A button with no fill until hovered, a glyph `icon` on it, `hint` saying what it does; grayed
/// and doing nothing unless `enabled`.
pub fn icon_button_if(ui: &mut egui::Ui, icon: &str, hint: &str, enabled: bool) -> Response {
    let button = Button::new(RichText::new(icon).size(16.0)).frame_when_inactive(false);
    ui.add_enabled(enabled, button).on_hover_text(hint)
}

/// The main menu's button: three thin lines.
pub fn menu_button(ui: &mut egui::Ui, hint: &str) -> Response {
    icons::button(ui, Icon::Menu, hint, true)
}

/// A "?" in a thin circle, as Google Calendar's help.
pub fn help_button(ui: &mut egui::Ui, hint: &str) -> Response {
    icons::button(ui, Icon::Help, hint, true)
}

/// A thin ✕ in the place of an icon button, closing what it stands in.
pub fn close_button(ui: &mut egui::Ui, hint: &str) -> Response {
    icons::button(ui, Icon::Close, hint, true)
}

/// A notice in the bottom corner of the window, on a raised card over the view.
pub fn toast(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    Frame::popup(ui.style())
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, content);
}

/// How wide a notice's text grows before it wraps.
const NOTICE_TEXT: f32 = 400.0;

/// A `toast` with the close cross of the dialogs at its end; says whether it was closed.
pub fn notice(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> bool {
    let mut closed = false;
    toast(ui, |ui| {
        ui.horizontal_top(|ui| {
            ui.scope(|ui| {
                ui.set_max_width(NOTICE_TEXT);
                ui.horizontal_wrapped(content);
            });
            ui.add_space(8.0);
            closed = close_button(ui, calendar_core::texts::CLOSE).clicked();
        });
    });
    closed
}

/// A group set apart on a light fill, such as an invitation's answers.
pub fn card<R>(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui) -> R) -> R {
    Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, content)
        .inner
}

/// How long the refresh icon takes to turn once while synchronizing.
const TURN: f64 = 1.2;

/// Refresh, its arrow turning while `turning`.
pub fn refresh_button(ui: &mut egui::Ui, hint: &str, turning: bool) -> Response {
    if !turning {
        return icons::button(ui, Icon::Refresh, hint, true);
    }
    let time = ui.input(|i| i.time);
    let angle = (time % TURN / TURN) as f32 * std::f32::consts::TAU;
    ui.ctx().request_repaint();
    icons::turned(ui, Icon::Refresh, hint, true, angle)
}

/// An icon button switching something on and off, its icon struck through while off.
pub fn icon_switch(ui: &mut egui::Ui, icon: Icon, on: bool, hint: &str) -> Response {
    let response = icons::button(ui, icon, hint, true);
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::CheckBox, true, on, hint));
    if !on {
        let color = ui.visuals().widgets.style(&response).text_color();
        let slash = egui::Rect::from_center_size(response.rect.center(), egui::vec2(16.0, 16.0));
        (ui.painter()).line_segment(
            [slash.left_top(), slash.right_bottom()],
            Stroke::new(1.5, color),
        );
    }
    response
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
    Button::new(RichText::new(text).color(ink))
        .fill(accent)
        .stroke(Stroke::NONE)
}

/// A menu behind a quiet "…" button, filled by `content`. The "…" shows while `shown`, as the
/// row it belongs to is under the pointer, and while it is focused or its menu open; the button
/// is there all the time, for the keyboard.
pub fn menu_dots(ui: &mut egui::Ui, shown: bool, content: impl FnOnce(&mut egui::Ui)) {
    let hidden = RichText::new("…").color(egui::Color32::TRANSPARENT);
    let button = Button::new(hidden).frame_when_inactive(false);
    let (response, open) = egui::containers::menu::MenuButton::from_button(button).ui(ui, content);
    if shown || open.is_some() || response.hovered() || response.has_focus() {
        let color = ui.visuals().widgets.style(&response).text_color();
        let font = egui::TextStyle::Button.resolve(ui.style());
        (ui.painter()).text(
            response.rect.center(),
            egui::Align2::CENTER_CENTER,
            "…",
            font,
            color,
        );
    }
}

/// How wide the column of icons before the rows of an event's details is.
pub const ROW_ICON: f32 = 28.0;
/// How large those icons are: the emoji font draws them small.
const ROW_ICON_SIZE: f32 = 17.0;

/// A row of an event's details: `icon` in a column of its own, weak, and what `content` adds
/// beside it, so the rows read as a list. The icon is centred on the content's first line, or on
/// the content when it is one control taller than a line, such as a button.
pub fn detail_row<R>(ui: &mut egui::Ui, icon: &str, content: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let (inner, column, shown) = ui
        .horizontal_top(|ui| {
            let (column, _) =
                ui.allocate_exact_size(egui::vec2(ROW_ICON, 0.0), egui::Sense::hover());
            let shown = ui.vertical(content);
            (shown.inner, column, shown.response.rect)
        })
        .inner;
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let first = if shown.height() < 1.8 * line {
        shown.center().y
    } else {
        shown.top() + line / 2.0
    };
    let weak = ui.visuals().weak_text_color();
    let font = egui::FontId::proportional(ROW_ICON_SIZE);
    let at = egui::pos2(column.left() + ROW_ICON_SIZE / 2.0, first);
    ui.painter()
        .text(at, egui::Align2::CENTER_CENTER, icon, font, weak);
    // Rows a little apart, so each reads on its own.
    ui.add_space(2.0);
    inner
}

/// The buttons and drop-downs of `ui` drawn as `text_field`s are: an outline on the view's
/// background rather than a gray fill, as in a form.
pub fn outlined_controls(ui: &mut egui::Ui) {
    let (line, hovered) = crate::appearance::field_lines(ui.visuals());
    let widgets = &mut ui.visuals_mut().widgets;
    widgets.inactive.bg_stroke = Stroke::new(1.0, line);
    widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    widgets.hovered.bg_stroke = Stroke::new(1.0, hovered);
}

/// A text field outlined in gray, darker under the pointer and in the accent while typed in, so
/// it reads as a place to type rather than as a button.
pub fn text_field(ui: &mut egui::Ui, field: egui::TextEdit) -> Response {
    ui.scope(|ui| {
        let (line, hovered) = crate::appearance::field_lines(ui.visuals());
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.bg_stroke = Stroke::new(1.0, line);
        widgets.hovered.bg_stroke = Stroke::new(1.0, hovered);
        ui.add(field)
    })
    .inner
}

/// Lines of text with their bold, italics, underlines and links (`MOD-05-F30`); gives the link
/// clicked.
pub fn rich_text(ui: &mut egui::Ui, lines: &[Vec<calendar_core::rich::Span>]) -> Option<String> {
    let mut clicked = None;
    // Lines as close as in a paragraph, not a row of buttons apart.
    ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
    ui.spacing_mut().interact_size.y = 0.0;
    for line in lines {
        if line.is_empty() {
            ui.add_space(ui.text_style_height(&egui::TextStyle::Body) / 2.0);
            continue;
        }
        ui.horizontal_wrapped(|ui| {
            for span in line {
                let mut text = RichText::new(&span.text);
                if span.bold {
                    text = text.font(crate::appearance::strong(14.0));
                }
                if span.italic {
                    text = text.italics();
                }
                if span.underline {
                    text = text.underline();
                }
                match &span.link {
                    Some(link) => {
                        if ui.link(text).on_hover_text(link).clicked() {
                            clicked = Some(link.clone());
                        }
                    }
                    None => {
                        ui.label(text);
                    }
                }
            }
        });
    }
    clicked
}

/// How wide the column of a form's labels is.
const FORM_LABEL: f32 = 112.0;

/// A row of a form: `label` in a column of its own, weak and level with the first line of what
/// `content` adds beside it.
pub fn form_row<R>(ui: &mut egui::Ui, label: &str, content: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.horizontal_top(|ui| {
        let size = egui::vec2(FORM_LABEL, ui.spacing().interact_size.y);
        ui.allocate_ui_with_layout(
            size,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_min_size(size);
                ui.weak(label);
            },
        );
        ui.vertical(content).inner
    })
    .inner
}

/// A key of the keyboard, as the shortcuts' list shows it: its character on a small key, as
/// wide as it is tall at least.
pub fn keycap(ui: &mut egui::Ui, key: char) {
    let font = crate::appearance::strong(12.0);
    let color = ui.visuals().text_color();
    let galley = ui.painter().layout_no_wrap(key.to_string(), font, color);
    let size = egui::vec2((galley.size().x + 10.0).max(22.0), 22.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let visuals = ui.visuals();
    let painter = ui.painter();
    let radius = CornerRadius::same(4);
    painter.rect_filled(rect, radius, crate::appearance::track(visuals));
    let line = visuals.widgets.noninteractive.bg_stroke;
    painter.rect_stroke(rect, radius, line, egui::StrokeKind::Inside);
    let at = rect.center() - galley.size() / 2.0;
    painter.galley(at, galley, color);
}

/// A section's title in a dialog, as the settings' "Accounts".
pub fn heading(ui: &mut egui::Ui, text: &str) {
    let font = crate::appearance::strong(15.0);
    ui.label(
        RichText::new(text)
            .font(font)
            .color(ui.visuals().strong_text_color()),
    );
    ui.add_space(2.0);
}

/// A thin line between the groups of a form or of an event's details, with room around it.
pub fn section(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
}

/// A small square in `color` before a title, as Google marks an event's calendar.
pub fn color_mark(ui: &mut egui::Ui, color: egui::Color32) {
    let size = egui::vec2(ROW_ICON, 28.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let mark = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 1.0, rect.center().y - 7.0),
        egui::vec2(14.0, 14.0),
    );
    ui.painter().rect_filled(mark, CornerRadius::same(4), color);
}

/// One of `names` picked, shown as one control; gives the one clicked.
pub fn segmented(ui: &mut egui::Ui, names: &[&str], picked: usize) -> Option<usize> {
    let visuals = ui.visuals().clone();
    let mut clicked = None;
    Frame::new()
        .fill(crate::appearance::track(&visuals))
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
