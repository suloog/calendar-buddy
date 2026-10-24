//! The light or dark theme the app is shown in (`MOD-04-F01`–`F03`): the one chosen in the
//! settings, or the desktop's, followed as it changes.

use std::sync::{Arc, Mutex};

use calendar_core::platform::Appearance;
use calendar_core::theme::{Scheme, Theme};
use eframe::egui::{
    self, Color32, CornerRadius, Stroke,
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

/// The panels' grays in each theme: the calendar's background, the side panel's a step off it,
/// and the lines between hours and days a step further.
struct Grays {
    background: u8,
    /// Windows over the views, which stand off the background in the dark.
    window: u8,
    side: u8,
    line: u8,
    field: u8,
    button: u8,
    hovered: u8,
    pressed: u8,
    text: u8,
    weak: u8,
    strong: u8,
}

impl Grays {
    /// The box of a checkbox on a button's gray `bg`: a step further from the background.
    fn check(&self, bg: u8) -> u8 {
        if self.background > self.text {
            bg.saturating_sub(0x14)
        } else {
            bg.saturating_add(0x14)
        }
    }
}

const LIGHT: Grays = Grays {
    background: 0xFF,
    window: 0xFF,
    side: 0xF7,
    line: 0xE6,
    field: 0xF2,
    button: 0xF0,
    hovered: 0xE4,
    pressed: 0xD9,
    text: 0x26,
    weak: 0x69,
    strong: 0x0A,
};

const DARK: Grays = Grays {
    background: 0x18,
    window: 0x22,
    side: 0x1E,
    line: 0x2E,
    field: 0x26,
    button: 0x2A,
    hovered: 0x34,
    pressed: 0x3E,
    text: 0xE6,
    weak: 0x9C,
    strong: 0xFF,
};

/// How rounded buttons, fields and events are, and windows.
pub const RADIUS: u8 = 8;
const WINDOW_RADIUS: u8 = 12;

/// The font of headings and of events' titles: Inter's semibold.
pub fn strong(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name(SEMIBOLD.into()))
}

const SEMIBOLD: &str = "semibold";

/// The outline of text fields, and its darker step under the pointer.
pub fn field_lines(visuals: &egui::Visuals) -> (Color32, Color32) {
    let grays = if visuals.dark_mode { &DARK } else { &LIGHT };
    (
        Color32::from_gray(grays.pressed),
        Color32::from_gray(grays.weak),
    )
}

/// The track of a segmented control, the gray of a button, also where buttons are outlined.
pub fn track(visuals: &egui::Visuals) -> Color32 {
    let grays = if visuals.dark_mode { &DARK } else { &LIGHT };
    Color32::from_gray(grays.button)
}

/// The side panel's fill, a step off the calendar's background.
pub fn side_fill(visuals: &egui::Visuals) -> Color32 {
    let grays = if visuals.dark_mode { &DARK } else { &LIGHT };
    Color32::from_gray(grays.side)
}

/// How the app looks in `scheme`: calm grays, with weak text dark or light enough to reach 4.5:1
/// (`MOD-04-NFR-01`, `DEC-148`), and the accent for every colour that is not gray.
pub fn visuals(scheme: Scheme) -> egui::Visuals {
    let ((accent, fill), base, g) = match scheme {
        Scheme::Light => (ACCENT_LIGHT, egui::Visuals::light(), &LIGHT),
        Scheme::Dark => (ACCENT_DARK, egui::Visuals::dark(), &DARK),
    };
    let gray = Color32::from_gray;
    let underline = Stroke::new(2.0, accent);
    // Buttons are filled with `weak_bg_fill`; checkboxes' and radio buttons' boxes with
    // `bg_fill`, a step darker so an empty box still shows.
    let widget = |bg: u8, text: u8| egui::style::WidgetVisuals {
        bg_fill: gray(g.check(bg)),
        weak_bg_fill: gray(bg),
        bg_stroke: Stroke::NONE,
        corner_radius: CornerRadius::same(RADIUS),
        fg_stroke: Stroke::new(1.0, gray(text)),
        expansion: 0.0,
    };
    let widgets = egui::style::Widgets {
        noninteractive: egui::style::WidgetVisuals {
            bg_fill: gray(g.background),
            weak_bg_fill: gray(g.background),
            bg_stroke: Stroke::new(1.0, gray(g.line)),
            ..widget(g.background, g.text)
        },
        inactive: widget(g.button, g.text),
        hovered: widget(g.hovered, g.strong),
        active: widget(g.pressed, g.strong),
        // Also the title bar of the window with the keyboard, which reads as part of it.
        open: widget(g.window, g.strong),
    };
    egui::Visuals {
        widgets,
        override_text_color: None,
        weak_text_color: Some(gray(g.weak)),
        panel_fill: gray(g.background),
        window_fill: gray(g.window),
        window_stroke: Stroke::new(1.0, gray(g.line)),
        window_corner_radius: CornerRadius::same(WINDOW_RADIUS),
        menu_corner_radius: CornerRadius::same(RADIUS),
        faint_bg_color: gray(g.side),
        extreme_bg_color: gray(g.field),
        // Fields are outlined on the window's fill (`widgets::text_field`), unlike buttons.
        text_edit_bg_color: Some(gray(g.window)),
        code_bg_color: gray(g.field),
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

/// Inter, regular and semibold, ahead of egui's own fonts, which still give the symbols and
/// emoji Inter lacks.
fn fonts() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    for (name, bytes) in [
        (
            "inter-regular",
            &include_bytes!("../assets/fonts/Inter-Regular.ttf")[..],
        ),
        (
            "inter-semibold",
            &include_bytes!("../assets/fonts/Inter-SemiBold.ttf")[..],
        ),
    ] {
        fonts.font_data.insert(
            name.into(),
            std::sync::Arc::new(egui::FontData::from_static(bytes)),
        );
    }
    let fallbacks = fonts.families[&egui::FontFamily::Proportional].clone();
    let family = |first: &str| [vec![first.to_string()], fallbacks.clone()].concat();
    fonts
        .families
        .insert(egui::FontFamily::Proportional, family("inter-regular"));
    fonts.families.insert(
        egui::FontFamily::Name(SEMIBOLD.into()),
        family("inter-semibold"),
    );
    fonts
}

/// Roomier than egui's defaults: buttons and rows a comfortable click high, text a size up.
fn spacing(style: &mut egui::Style) {
    use egui::{FontFamily::Proportional, FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Small, FontId::new(12.0, Proportional)),
        (TextStyle::Body, FontId::new(14.0, Proportional)),
        (TextStyle::Button, FontId::new(14.0, Proportional)),
        (TextStyle::Heading, strong(20.0)),
        (
            TextStyle::Monospace,
            FontId::new(13.0, egui::FontFamily::Monospace),
        ),
    ]
    .into();
    let spacing = &mut style.spacing;
    spacing.item_spacing = egui::vec2(8.0, 6.0);
    spacing.button_padding = egui::vec2(12.0, 6.0);
    spacing.interact_size.y = 30.0;
    spacing.window_margin = egui::Margin::same(16);
    spacing.menu_margin = egui::Margin::same(8);
    spacing.icon_width = 16.0;
    spacing.icon_width_inner = 10.0;
    spacing.icon_spacing = 8.0;
}

/// Gives the light and dark themes their looks and fonts, without transition animations
/// (`MOD-04-U05`).
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.set_visuals_of(egui::Theme::Light, visuals(Scheme::Light));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(Scheme::Dark));
    ctx.all_styles_mut(|style| {
        style.animation_time = 0.0;
        spacing(style);
    });
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
