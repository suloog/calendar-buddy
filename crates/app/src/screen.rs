//! For tests: what a view draws in a window, read back from egui's shapes, and the contrast of
//! its texts with what is under them (`MOD-04-NFR-01`).

use calendar_core::contrast::contrast;
use calendar_core::guests::{Guest, Response};
use calendar_core::model::{Event, Rgb};
use calendar_core::theme::Scheme;
use eframe::egui::{
    self, Color32, Pos2, Rect, Vec2,
    epaint::{ClippedShape, Shape, TextShape},
};

/// What `draw` draws in a central panel of a `window` points large at `scale`, in `scheme`,
/// with the pointer at `pointer`: the last of a few frames a second apart, so that scrolling has
/// ended.
pub fn drawn(
    window: Vec2,
    scale: f32,
    scheme: Scheme,
    pointer: Option<Pos2>,
    mut draw: impl FnMut(&mut egui::Ui),
) -> Vec<ClippedShape> {
    let ctx = egui::Context::default();
    crate::appearance::install(&ctx);
    ctx.set_theme(match scheme {
        Scheme::Light => egui::Theme::Light,
        Scheme::Dark => egui::Theme::Dark,
    });
    let mut shapes = Vec::new();
    for second in 0..4 {
        let mut input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, window)),
            time: Some(f64::from(second)),
            events: pointer.map(egui::Event::PointerMoved).into_iter().collect(),
            ..Default::default()
        };
        input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(scale);
        let mut output = ctx.run_ui(input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| draw(ui));
        });
        output.textures_delta.clear();
        assert_eq!(output.pixels_per_point, scale);
        shapes = output.shapes;
    }
    shapes
}

/// The very light and very dark calendar colours of `TC-04-04`.
pub const VERY_LIGHT: Rgb = Rgb(0xFF, 0xF5, 0x9D);
pub const VERY_DARK: Rgb = Rgb(0x1A, 0x23, 0x7E);

/// `event` in `color`, answered so by the account whose calendar it is on.
pub fn answered(mut event: Event, color: Rgb, response: Response) -> Event {
    event.color = color;
    event.about.guests = vec![Guest {
        email: "me@example.com".to_string(),
        name: String::new(),
        optional: false,
        response,
        own: true,
    }];
    event
}

/// The shapes drawn, in the order drawn, each with the clip it is drawn in.
pub fn in_order(shapes: &[ClippedShape]) -> Vec<(Rect, Shape)> {
    fn flatten(clip: Rect, shape: &Shape, out: &mut Vec<(Rect, Shape)>) {
        match shape {
            Shape::Vec(shapes) => shapes.iter().for_each(|s| flatten(clip, s, out)),
            shape => out.push((clip, shape.clone())),
        }
    }
    let mut out = Vec::new();
    for clipped in shapes {
        flatten(clipped.clip_rect, &clipped.shape, &mut out);
    }
    out
}

/// The texts drawn, each with the clip it is drawn in.
pub fn texts(shapes: &[ClippedShape]) -> Vec<(Rect, TextShape)> {
    in_order(shapes)
        .into_iter()
        .filter_map(|(clip, shape)| match shape {
            Shape::Text(text) => Some((clip, text)),
            _ => None,
        })
        .collect()
}

/// The first text drawn that `matches`.
pub fn text_where(shapes: &[ClippedShape], matches: impl Fn(&str) -> bool) -> TextShape {
    texts(shapes)
        .into_iter()
        .map(|(_, text)| text)
        .find(|text| matches(text.galley.text()))
        .expect("the text is drawn")
}

/// The fills of rectangles and discs drawn at `at` among `shapes`, laid one over the other on `base` as egui blends
/// them, premultiplied.
fn laid(shapes: &[(Rect, Shape)], at: Pos2, base: Rgb) -> Rgb {
    let channels = shapes
        .iter()
        .filter_map(|(clip, shape)| match shape {
            Shape::Rect(rect) if clip.contains(at) && rect.rect.contains(at) => Some(rect.fill),
            Shape::Circle(circle)
                if clip.contains(at) && circle.center.distance(at) <= circle.radius =>
            {
                Some(circle.fill)
            }
            _ => None,
        })
        .fold([base.0, base.1, base.2].map(f32::from), |below, fill| {
            let alpha = fill.a() as f32 / 255.0;
            let [r, g, b] = [fill.r(), fill.g(), fill.b()].map(f32::from);
            [
                r + below[0] * (1.0 - alpha),
                g + below[1] * (1.0 - alpha),
                b + below[2] * (1.0 - alpha),
            ]
        });
    let [r, g, b] = channels.map(|c| c.round() as u8);
    Rgb(r, g, b)
}

/// The colour a text is drawn in.
pub fn ink(text: &TextShape) -> Rgb {
    let mut c = text.galley.job.sections[0].format.color;
    if c == Color32::PLACEHOLDER {
        c = text.fallback_color;
    }
    if let Some(color) = text.override_text_color {
        c = color;
    }
    assert_eq!(c.a(), 255, "a text in {c:?}, not opaque");
    assert_eq!(text.opacity_factor, 1.0, "a faded text");
    Rgb(c.r(), c.g(), c.b())
}

/// The contrast on the screen of the first text drawn that has `part` in it: its colour, under
/// whatever is drawn over it, against the fills drawn under it.
pub fn contrast_of(shapes: &[ClippedShape], part: &str) -> f32 {
    contrast_where(shapes, |text| text.contains(part))
}

/// `contrast_of` the first text drawn that `matches`.
pub fn contrast_where(shapes: &[ClippedShape], matches: impl Fn(&str) -> bool) -> f32 {
    let shown = in_order(shapes);
    let index = shown
        .iter()
        .position(|(_, shape)| matches!(shape, Shape::Text(t) if matches(t.galley.text())))
        .expect("the text is drawn");
    let Shape::Text(text) = &shown[index].1 else {
        unreachable!("found as a text")
    };
    let at = text.visual_bounding_rect().left_center();
    let background = laid(&shown[..index], at, Rgb(0, 0, 0));
    let glyphs = laid(&shown[index + 1..], at, ink(text));
    let covered = laid(&shown[index + 1..], at, background);
    contrast(glyphs, covered)
}

/// The hue of the accent, in degrees: the same in its tones for both themes.
const ACCENT_HUE: f32 = 217.0;

/// Whether `c` is a gray, or a tone of the accent, at any opacity.
pub fn neutral_or_accent(c: Color32) -> bool {
    let [r, g, b, a] = c.to_srgba_unmultiplied().map(f32::from);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    if a == 0.0 || max - min <= 2.0 {
        return true;
    }
    // Blue is the strongest of the accent's channels, which gives its hue.
    max == b && (240.0 + 60.0 * (r - g) / (max - min) - ACCENT_HUE).abs() <= 3.0
}
