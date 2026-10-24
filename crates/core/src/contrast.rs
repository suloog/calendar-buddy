//! Readable text (`MOD-04-U04`, `MOD-04-NFR-01`): contrast as WCAG 2.1 measures it, and the
//! colour of an event's text picked from the relative luminance of what is under it.

use crate::model::Rgb;

/// The contrast WCAG 2.1 AA asks of text.
pub const AA: f32 = 4.5;

const BLACK: Rgb = Rgb(0, 0, 0);
const WHITE: Rgb = Rgb(0xFF, 0xFF, 0xFF);

/// A colour's relative luminance, from 0 for black to 1 for white (WCAG 2.1).
pub fn luminance(c: Rgb) -> f32 {
    let linear = |channel: u8| {
        let s = channel as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(c.0) + 0.7152 * linear(c.1) + 0.0722 * linear(c.2)
}

/// The contrast ratio of two colours, from 1:1 to 21:1, whichever is lighter.
pub fn contrast(a: Rgb, b: Rgb) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// The text colour on `background`: black or white, whichever contrasts more. One of them always
/// reaches `AA`.
pub fn text_on(background: Rgb) -> Rgb {
    if contrast(BLACK, background) >= contrast(WHITE, background) {
        BLACK
    } else {
        WHITE
    }
}

/// `over` laid at `opacity` over `under`, as the screen shows it.
pub fn over(over: Rgb, opacity: f32, under: Rgb) -> Rgb {
    let mix = |a: u8, b: u8| (a as f32 * opacity + b as f32 * (1.0 - opacity)).round() as u8;
    Rgb(
        mix(over.0, under.0),
        mix(over.1, under.1),
        mix(over.2, under.2),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colors::{CALENDAR_COLORS, EventColor};

    #[test]
    fn contrast_is_measured_as_wcag_does() {
        assert_eq!(luminance(BLACK), 0.0);
        assert!((luminance(WHITE) - 1.0).abs() < 1e-6);
        assert!((contrast(BLACK, WHITE) - 21.0).abs() < 1e-3);
        assert!((contrast(WHITE, BLACK) - 21.0).abs() < 1e-3);
        // WCAG's own example: #767676 on white is just over 4.5:1.
        let gray = Rgb(0x76, 0x76, 0x76);
        assert!((contrast(gray, WHITE) - 4.54).abs() < 0.01);
    }

    #[test]
    fn very_light_colours_get_dark_text_and_very_dark_ones_light_text() {
        assert_eq!(text_on(Rgb(0xFF, 0xF5, 0x9D)), BLACK);
        assert_eq!(text_on(Rgb(0x1A, 0x23, 0x7E)), WHITE);
    }

    #[test]
    fn text_reaches_aa_on_any_colour() {
        let mut colours: Vec<Rgb> = CALENDAR_COLORS.iter().map(|(_, rgb)| *rgb).collect();
        colours.extend(EventColor::ALL.iter().map(|color| color.rgb()));
        // Every shade of gray, and the colours in between those of the palette.
        colours.extend((0..=255).map(|v| Rgb(v, v, v)));
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    colours.push(Rgb(r, g, b));
                }
            }
        }
        for c in colours {
            let ratio = contrast(text_on(c), c);
            assert!(ratio >= AA, "{c:?}: {ratio}");
        }
    }

    #[test]
    fn a_colour_laid_over_another_mixes_by_its_opacity() {
        assert_eq!(over(WHITE, 0.5, BLACK), Rgb(128, 128, 128));
        assert_eq!(over(Rgb(200, 100, 0), 1.0, BLACK), Rgb(200, 100, 0));
        assert_eq!(over(Rgb(200, 100, 0), 0.0, WHITE), WHITE);
    }
}
