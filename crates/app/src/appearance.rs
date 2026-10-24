//! The light or dark theme the app is shown in (`MOD-04-F01`–`F03`): the one chosen in the
//! settings, or the desktop's, followed as it changes.

use std::sync::{Arc, Mutex};

use calendar_core::platform::Appearance;
use calendar_core::theme::{Scheme, Theme};
use eframe::egui::{
    self, Color32, Stroke,
    style::{ImeComposition, Selection, TextCursorStyle},
};

/// The desktop's light or dark preference, for the UI thread to read.
pub struct Desktop(Arc<Mutex<Option<Scheme>>>);

impl Desktop {
    /// Follows the desktop's word; until it comes, the desktop states no preference.
    pub fn watch(ctx: &egui::Context, appearance: &dyn Appearance) -> Self {
        let preference = Arc::new(Mutex::new(None));
        let (kept, ctx) = (preference.clone(), ctx.clone());
        appearance.watch(Box::new(move |now| {
            if std::mem::replace(&mut *kept.lock().expect("preference"), now) != now {
                ctx.request_repaint();
            }
        }));
        Desktop(preference)
    }

    pub fn get(&self) -> Option<Scheme> {
        *self.0.lock().expect("preference")
    }
}

/// The app's one accent colour (`MOD-04-U03`, `DEC-57`), in each theme: the tone links, the
/// selection's outline, warnings and the current time are drawn in, and its pale tone that fills
/// what is selected. Everything else is neutral gray.
const ACCENT_LIGHT: (Color32, Color32) = (
    Color32::from_rgb(0x0B, 0x57, 0xD0),
    Color32::from_rgb(0xD3, 0xE3, 0xFD),
);
const ACCENT_DARK: (Color32, Color32) = (
    Color32::from_rgb(0xA8, 0xC7, 0xFA),
    Color32::from_rgb(0x08, 0x42, 0xA0),
);

/// How the app looks in `scheme`: egui's neutral grays, with weak text made darker or lighter to
/// reach 4.5:1 (`MOD-04-NFR-01`, `DEC-148`), and the accent for every colour that is not gray.
pub fn visuals(scheme: Scheme) -> egui::Visuals {
    let ((accent, fill), base) = match scheme {
        Scheme::Light => (
            ACCENT_LIGHT,
            egui::Visuals {
                weak_text_color: Some(Color32::from_gray(110)),
                ..egui::Visuals::light()
            },
        ),
        Scheme::Dark => (
            ACCENT_DARK,
            egui::Visuals {
                weak_text_color: Some(Color32::from_gray(135)),
                ..egui::Visuals::dark()
            },
        ),
    };
    let underline = Stroke::new(2.0, accent);
    egui::Visuals {
        selection: Selection {
            bg_fill: fill,
            stroke: Stroke::new(1.0, accent),
        },
        hyperlink_color: accent,
        warn_fg_color: accent,
        error_fg_color: accent,
        text_cursor: TextCursorStyle {
            stroke: underline,
            ..base.text_cursor
        },
        ime_composition: ImeComposition {
            active_underline_stroke: underline,
            inactive_underline_stroke: Stroke::new(2.0, accent.linear_multiply(0.5)),
            ..base.ime_composition
        },
        ..base
    }
}

/// Gives the light and dark themes their looks, without transition animations
/// (`MOD-04-U05`).
pub fn install(ctx: &egui::Context) {
    ctx.set_visuals_of(egui::Theme::Light, visuals(Scheme::Light));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(Scheme::Dark));
    ctx.all_styles_mut(|style| style.animation_time = 0.0);
}

/// Shows the app in the theme `theme` makes of the desktop's preference.
pub fn apply(ctx: &egui::Context, theme: Theme, desktop: &Desktop) {
    let wanted = match theme.shown(desktop.get()) {
        Scheme::Light => egui::Theme::Light,
        Scheme::Dark => egui::Theme::Dark,
    };
    // The preference, not the theme egui resolves, so the window is told the theme too.
    if ctx.options(|o| o.theme_preference) != wanted.into() {
        ctx.set_theme(wanted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::neutral_or_accent;
    use calendar_core::contrast::{AA, contrast};

    type Changed = Box<dyn Fn(Option<Scheme>) + Send + Sync>;

    /// A desktop whose preference the test sets.
    struct FakeDesktop(Mutex<Option<Changed>>, Option<Scheme>);

    impl FakeDesktop {
        fn prefers(preference: Option<Scheme>) -> Self {
            FakeDesktop(Mutex::default(), preference)
        }

        fn switch(&self, preference: Option<Scheme>) {
            (self.0.lock().unwrap().as_ref().unwrap())(preference);
        }
    }

    impl Appearance for FakeDesktop {
        fn watch(&self, changed: Changed) {
            changed(self.1);
            *self.0.lock().unwrap() = Some(changed);
        }
    }

    fn rgb(c: Color32) -> calendar_core::model::Rgb {
        assert_eq!(c.a(), 255, "{c:?} is not opaque");
        calendar_core::model::Rgb(c.r(), c.g(), c.b())
    }

    #[test]
    fn interface_text_reaches_4_5_to_1_on_its_backgrounds_in_both_themes() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let v = visuals(scheme);
            let w = &v.widgets;
            // Text on the panels, windows and text fields.
            let text = [
                v.text_color(),
                v.weak_text_color(),
                v.strong_text_color(),
                v.hyperlink_color,
                v.warn_fg_color,
            ];
            let mut pairs = Vec::new();
            for background in [v.panel_fill, v.window_fill, v.text_edit_bg_color()] {
                pairs.extend(text.map(|t| (t, background)));
            }
            pairs.push((w.inactive.text_color(), v.text_edit_bg_color()));
            // Widgets' text on their own backgrounds, in every state.
            for state in [&w.inactive, &w.hovered, &w.active, &w.open] {
                pairs.push((state.text_color(), state.weak_bg_fill));
                pairs.push((state.text_color(), state.bg_fill));
            }
            // Selected widgets, and today's date marked in the selection's colour.
            pairs.push((v.selection.stroke.color, v.selection.bg_fill));
            pairs.push((v.strong_text_color(), v.selection.bg_fill));
            for (n, (text, background)) in pairs.into_iter().enumerate() {
                let ratio = contrast(rgb(text), rgb(background));
                assert!(
                    ratio >= AA,
                    "{scheme:?} #{n}: {text:?} on {background:?}: {ratio}"
                );
            }
        }
    }

    #[test]
    fn apart_from_grays_the_themes_have_only_the_accent_in_both_themes() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let style = egui::Style {
                visuals: visuals(scheme),
                ..Default::default()
            };
            // Every colour of the look, as its `Debug` writes it: `#RR_GG_BB_AA`, premultiplied.
            let debug = format!("{style:?}");
            let colors: Vec<Color32> = debug
                .split('#')
                .skip(1)
                .filter_map(|after| {
                    let hex = after.get(..11)?.replace('_', "");
                    let v = u32::from_str_radix(&hex, 16).ok()?;
                    let [r, g, b, a] = v.to_be_bytes();
                    Some(Color32::from_rgba_premultiplied(r, g, b, a))
                })
                .collect();
            assert!(colors.len() > 20, "{scheme:?}: {} colours", colors.len());
            for c in colors {
                assert!(neutral_or_accent(c), "{scheme:?}: {c:?}");
            }
        }
    }

    #[test]
    fn nothing_in_the_interface_animates() {
        let ctx = egui::Context::default();
        install(&ctx);
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            assert_eq!(ctx.style_of(theme).animation_time, 0.0);
        }
        let id = egui::Id::unique("shown");
        let mut shown = Vec::new();
        for (frame, value) in [false, true, false].into_iter().enumerate() {
            let input = egui::RawInput {
                time: Some(frame as f64 * 0.01),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| shown.push(ui.ctx().animate_bool(id, value)));
            output.textures_delta.clear();
        }
        assert_eq!(shown, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn the_themes_are_given_their_looks() {
        let ctx = egui::Context::default();
        install(&ctx);
        for (theme, scheme) in [
            (egui::Theme::Light, Scheme::Light),
            (egui::Theme::Dark, Scheme::Dark),
        ] {
            assert_eq!(ctx.style_of(theme).visuals, visuals(scheme));
        }
    }

    #[test]
    fn tc_04_01_with_system_the_app_starts_dark_and_turns_light_with_the_desktop() {
        let ctx = egui::Context::default();
        let fake = FakeDesktop::prefers(Some(Scheme::Dark));
        let desktop = Desktop::watch(&ctx, &fake);

        apply(&ctx, Theme::System, &desktop);
        assert_eq!(ctx.theme(), egui::Theme::Dark);

        fake.switch(Some(Scheme::Light));
        apply(&ctx, Theme::System, &desktop);
        assert_eq!(ctx.theme(), egui::Theme::Light);

        fake.switch(Some(Scheme::Dark));
        apply(&ctx, Theme::System, &desktop);
        assert_eq!(ctx.theme(), egui::Theme::Dark);
    }

    #[test]
    fn tc_04_02_a_chosen_theme_is_shown_whatever_the_desktop_prefers() {
        let ctx = egui::Context::default();
        let fake = FakeDesktop::prefers(Some(Scheme::Dark));
        let desktop = Desktop::watch(&ctx, &fake);

        apply(&ctx, Theme::Light, &desktop);
        assert_eq!(ctx.theme(), egui::Theme::Light);

        fake.switch(Some(Scheme::Light));
        apply(&ctx, Theme::Dark, &desktop);
        assert_eq!(ctx.theme(), egui::Theme::Dark);
    }

    #[test]
    fn a_desktop_without_a_preference_or_not_answering_gets_the_light_theme() {
        let ctx = egui::Context::default();
        ctx.set_theme(egui::Theme::Dark);
        let fake = FakeDesktop::prefers(None);
        apply(&ctx, Theme::System, &Desktop::watch(&ctx, &fake));
        assert_eq!(ctx.theme(), egui::Theme::Light);

        /// A desktop that never answers.
        struct Silent;
        impl Appearance for Silent {
            fn watch(&self, _changed: Changed) {}
        }
        ctx.set_theme(egui::Theme::Dark);
        apply(&ctx, Theme::System, &Desktop::watch(&ctx, &Silent));
        assert_eq!(ctx.theme(), egui::Theme::Light);
    }
}
