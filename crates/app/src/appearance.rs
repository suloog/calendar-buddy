//! The light or dark theme the app is shown in (`MOD-04-F01`–`F03`): the one chosen in the
//! settings, or the desktop's, followed as it changes.

use std::sync::{Arc, Mutex};

use calendar_core::platform::Appearance;
use calendar_core::theme::{Scheme, Theme};
use eframe::egui;

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
