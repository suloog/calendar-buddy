//! Events from a description in the app (`US-10-01`): the side panel's box, greyed out with why
//! when Ollama cannot be used and a link to the settings (`MOD-10-U04`); the settings' section
//! (`MOD-10-U05`); and asking Ollama off the UI thread whether it can be used — at start, when
//! the settings change and when the user goes to the box, never on a timer (`MOD-10` technical
//! notes). Reading a description is `US-10-02`.

use std::sync::mpsc::{self, Receiver, Sender};

use calendar_core::ollama::{self, Availability, DescriptionBox, Form, Ollama};
use calendar_core::texts;
use eframe::egui;

use crate::widgets::{self, text_field};

/// The box's field, for `q` to put the cursor in.
pub fn box_id() -> egui::Id {
    egui::Id::unique("description-box")
}

/// Asks Ollama whether it can be used, off the UI thread (`NFR-01`).
pub struct Checker {
    asks: Option<Sender<Ollama>>,
    answers: Receiver<(Ollama, Availability)>,
}

impl Checker {
    pub fn start(ctx: &egui::Context) -> Self {
        let (asks, asked) = mpsc::channel::<Ollama>();
        let (answer, answers) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            while let Ok(mut settings) = asked.recv() {
                // Only the settings asked last matter.
                while let Ok(later) = asked.try_recv() {
                    settings = later;
                }
                let availability = ollama::check(&settings);
                if answer.send((settings, availability)).is_err() {
                    return;
                }
                ctx.request_repaint();
            }
        });
        Checker {
            asks: Some(asks),
            answers,
        }
    }

    /// Never asks anything, for the pictures of the interface.
    #[cfg(test)]
    pub fn none() -> Self {
        Checker {
            asks: None,
            answers: mpsc::channel().1,
        }
    }

    pub fn ask(&self, settings: &Ollama) {
        if let Some(asks) = &self.asks {
            let _ = asks.send(settings.clone());
        }
    }

    /// The last answer for the settings as they are now; those for earlier settings are dropped.
    pub fn answer(&self, settings: &Ollama) -> Option<Availability> {
        (self.answers.try_iter())
            .filter(|(asked, _)| asked == settings)
            .last()
            .map(|(_, availability)| availability)
    }
}

/// What the user did with the box.
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Went to the box: put the cursor in it, or clicked it while it is greyed out.
    GoneTo,
    /// Enter in the box.
    Enter,
    /// The link to the settings under an unavailable box.
    OpenSettings,
}

/// Draws the box above "Meet with…"; unavailable, it is greyed out, keeps what was typed and says
/// `why` with a link to the settings.
pub fn show_box(
    ui: &mut egui::Ui,
    described: &mut DescriptionBox,
    why: Option<&str>,
) -> Option<Action> {
    let mut action = None;
    let available = why.is_none();
    let field = ui
        .add_enabled_ui(available, |ui| {
            text_field(
                ui,
                egui::TextEdit::singleline(&mut described.field)
                    .id(box_id())
                    .hint_text(texts::DESCRIBE_EVENT)
                    .desired_width(f32::INFINITY),
            )
        })
        .inner;
    if field.gained_focus() {
        action = Some(Action::GoneTo);
    }
    if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        action = Some(Action::Enter);
        field.request_focus();
    }
    if let Some(why) = why {
        // Greyed out, the field takes no clicks of its own; a click asks Ollama again.
        let id = box_id().with("unavailable");
        if ui.interact(field.rect, id, egui::Sense::click()).clicked() {
            action = Some(Action::GoneTo);
        }
        ui.weak(why);
        if ui.link(texts::OPEN_SETTINGS).clicked() {
            action = Some(Action::OpenSettings);
        }
    }
    action
}

/// Draws the settings' section, scrolled to when `scroll`; says whether `settings` changed.
pub fn settings_section(
    ui: &mut egui::Ui,
    form: &mut Form,
    settings: &mut Ollama,
    scroll: bool,
) -> bool {
    if scroll {
        ui.scroll_to_cursor(Some(egui::Align::TOP));
    }
    widgets::heading(ui, texts::EVENTS_FROM_DESCRIPTION);
    let mut changed = ui
        .checkbox(&mut settings.on, texts::DESCRIBE_EVENTS)
        .on_hover_text(texts::DESCRIBE_EVENTS_HINT)
        .changed();
    let field = |ui: &mut egui::Ui, text: &mut String| {
        text_field(
            ui,
            egui::TextEdit::singleline(text).desired_width(f32::INFINITY),
        )
        .lost_focus()
    };
    if widgets::form_row(ui, texts::OLLAMA_ADDRESS, |ui| field(ui, &mut form.address)) {
        changed |= form.enter_address(settings);
    }
    let warn = ui.visuals().warn_fg_color;
    if form.wrong_address {
        ui.colored_label(warn, texts::NOT_AN_OLLAMA_ADDRESS);
    }
    if !ollama::is_local(&settings.address) {
        ui.colored_label(warn, texts::LEAVES_COMPUTER);
    }
    if widgets::form_row(ui, texts::OLLAMA_MODEL, |ui| field(ui, &mut form.model)) {
        changed |= form.enter_model(settings);
    }
    changed
}

#[cfg(test)]
mod tests {
    //! The test cases of `US-10-01` (`TC-10-01`…`TC-10-07`), driving the whole app: the side
    //! panel, the keys, the settings and a fake Ollama. Each step of a test case is announced
    //! with `step` before it is carried out, so the run's report shows which steps ran.

    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    use super::*;
    use crate::{CalendarApp, Dialog};
    use calendar_core::accounts::Accounts;
    use calendar_core::local;
    use calendar_core::navigation::View;
    use calendar_core::ollama::fake::{FakeOllama, nothing_answers};
    use calendar_core::ollama::{DEFAULT_ADDRESS, DEFAULT_MODEL};
    use calendar_core::settings::Settings;
    use calendar_core::theme::Theme;
    use calendar_core::week::GridView;
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT, Queryable};

    /// Announces a step of a test case: `og-step <ID> <n> — <what it does>`.
    fn step(id: &str, n: u32, what: &str) {
        println!("og-step {id} {n} — {what}");
    }

    /// The app with no Google account, "Use without an account" chosen, the settings as given,
    /// saved to `saved_to` when given, Ollama asked as at start.
    fn app(settings: Settings, saved_to: Option<PathBuf>) -> Harness<'static, CalendarApp> {
        // Tall, so that the whole of the settings fits in the window.
        app_sized(egui::vec2(1440.0, 2400.0), settings, saved_to)
    }

    fn app_sized(
        size: egui::Vec2,
        settings: Settings,
        saved_to: Option<PathBuf>,
    ) -> Harness<'static, CalendarApp> {
        let mut harness = Harness::builder().with_size(size).build_eframe(move |cc| {
            let ctx = &cc.egui_ctx;
            let mut app = CalendarApp::preview(ctx, View::Grid(GridView::Week), Theme::Light);
            app.accounts = Accounts::default();
            app.settings = settings;
            app.settings_writer = saved_to.map(|path| {
                let (sender, receiver) = mpsc::channel::<Settings>();
                std::thread::spawn(move || {
                    for settings in receiver {
                        settings.save(&path).unwrap();
                    }
                });
                sender
            });
            app.checker = Checker::start(ctx);
            if app.settings.ollama.on {
                app.checker.ask(&app.settings.ollama);
            }
            app
        });
        until(&mut harness, |app| app.sync.local().is_some());
        if !local::of(harness.state().sync.calendars()).any(|_| true) {
            harness.state_mut().use_without_account();
        }
        harness.run_steps(3);
        harness
    }

    /// Steps the app until `done` says so, for at most 10 seconds.
    fn until(harness: &mut Harness<'static, CalendarApp>, done: impl Fn(&CalendarApp) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !done(harness.state()) {
            assert!(Instant::now() < deadline, "the app never got there");
            std::thread::sleep(Duration::from_millis(10));
            harness.step();
        }
        harness.run_steps(2);
    }

    /// Whether the window shows `text`, as a label or a field's value.
    fn shows(harness: &Harness<'static, CalendarApp>, text: &str) -> bool {
        harness.query_all_by_value(text).next().is_some()
            || harness.query_all_by_label(text).next().is_some()
    }

    /// The text field holding `value`.
    fn field<'h>(
        harness: &'h Harness<'static, CalendarApp>,
        value: &str,
    ) -> egui_kittest::Node<'h> {
        let value = value.to_string();
        harness.get_by(move |node| {
            node.role() == egui::accesskit::Role::TextInput
                && node.value().as_deref() == Some(value.as_str())
        })
    }

    /// The description box, if the side panel shows it.
    fn description_box<'h>(
        harness: &'h Harness<'static, CalendarApp>,
    ) -> Option<egui_kittest::Node<'h>> {
        harness.query_by(|node| node.locate().0 == box_id().accesskit_id())
    }

    fn meet_with<'h>(harness: &'h Harness<'static, CalendarApp>) -> egui_kittest::Node<'h> {
        harness.get_by(|node| node.placeholder() == Some(texts::MEET_WITH))
    }

    /// Presses a key that types `key`, as on the keyboard.
    fn press(harness: &mut Harness<'static, CalendarApp>, key: char) {
        harness.event(egui::Event::Text(key.to_string()));
        harness.run_steps(2);
    }

    /// Replaces the text of the field showing `value` with `text`, then presses Enter.
    fn enter(harness: &mut Harness<'static, CalendarApp>, value: &str, text: &str) {
        field(harness, value).focus();
        harness.run_steps(2);
        harness.key_combination_modifiers(egui::Modifiers::COMMAND, &[egui::Key::A]);
        field(harness, value).type_text(text);
        harness.run_steps(2);
        harness.key_press(egui::Key::Enter);
        harness.run_steps(2);
    }

    /// Whether the shortcuts' list shows `q`.
    fn q_listed(harness: &mut Harness<'static, CalendarApp>) -> bool {
        harness.state_mut().dialog = Some(Dialog::Shortcuts);
        harness.run_steps(3);
        let listed = shows(harness, texts::DESCRIBE_EVENT_SHORTCUT);
        harness.state_mut().dialog = None;
        harness.run_steps(2);
        listed
    }

    fn on(ollama: &FakeOllama) -> Settings {
        let mut settings = Settings::default();
        settings.ollama.address = ollama.address.clone();
        settings
    }

    fn open_settings(harness: &mut Harness<'static, CalendarApp>) {
        harness.state_mut().dialog = Some(Dialog::Settings);
        harness.run_steps(3);
    }

    fn close_settings(harness: &mut Harness<'static, CalendarApp>) {
        harness.key_press(egui::Key::Escape);
        harness.run_steps(3);
    }

    fn settings_file(test: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("calendar-buddy-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("settings")
    }

    /// Waits until the settings file holds `line`.
    fn saved(path: &Path, line: &str) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !std::fs::read_to_string(path).is_ok_and(|text| text.contains(line)) {
            assert!(Instant::now() < deadline, "{line} never saved");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn tc_10_01_the_section_starts_on_with_the_defaults_and_keeps_changes_after_a_restart() {
        const ID: &str = "TC-10-01";
        let path = settings_file("tc-10-01");
        let mut harness = app(Settings::load(&path), Some(path.clone()));
        assert_eq!(harness.state().settings.ollama.address, DEFAULT_ADDRESS);

        step(ID, 1, "Open the settings.");
        open_settings(&mut harness);
        assert!(shows(&harness, texts::EVENTS_FROM_DESCRIPTION));
        let checkbox = harness.get_by_label(texts::DESCRIBE_EVENTS);
        assert_eq!(
            checkbox.accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        assert!(shows(&harness, DEFAULT_ADDRESS));
        assert!(shows(&harness, DEFAULT_MODEL));

        step(
            ID,
            2,
            "Change the address and the model, untick the checkbox and close the settings.",
        );
        enter(&mut harness, DEFAULT_ADDRESS, "http://127.0.0.1:11500");
        enter(&mut harness, DEFAULT_MODEL, "llama3.2:3b");
        harness.get_by_label(texts::DESCRIBE_EVENTS).click();
        harness.run_steps(2);
        close_settings(&mut harness);
        assert!(harness.state().dialog.is_none());
        assert!(!shows(&harness, texts::NOT_AN_OLLAMA_ADDRESS));
        saved(&path, "descriptions = off");
        saved(&path, "ollama_model = llama3.2:3b");
        drop(harness);

        step(ID, 3, "Quit and start the app again; open the settings.");
        let mut harness = app(Settings::load(&path), Some(path.clone()));
        open_settings(&mut harness);
        let checkbox = harness.get_by_label(texts::DESCRIBE_EVENTS);
        assert_eq!(
            checkbox.accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::False)
        );
        assert!(shows(&harness, "http://127.0.0.1:11500"));
        assert!(shows(&harness, "llama3.2:3b"));
    }

    #[test]
    fn tc_10_02_turned_off_the_box_its_key_and_its_shortcut_are_gone_and_back_when_on() {
        const ID: &str = "TC-10-02";
        let ollama = FakeOllama::start(&[DEFAULT_MODEL]);
        let mut harness = app(on(&ollama), None);
        until(&mut harness, |app| {
            app.description.availability == Availability::Available
        });

        step(
            ID,
            1,
            "Look at the side panel; open the list of keyboard shortcuts.",
        );
        let field = description_box(&harness).expect("the box is in the side panel");
        assert!(field.rect().bottom() <= meet_with(&harness).rect().top());
        assert!(q_listed(&mut harness));

        step(
            ID,
            2,
            "Untick the function; look at the side panel and the list; press q.",
        );
        open_settings(&mut harness);
        harness.get_by_label(texts::DESCRIBE_EVENTS).click();
        harness.run_steps(2);
        close_settings(&mut harness);
        assert!(description_box(&harness).is_none());
        assert!(!q_listed(&mut harness));
        let before = harness.state().navigation;
        press(&mut harness, 'q');
        assert_eq!(harness.state().navigation, before);
        assert!(harness.state().dialog.is_none());
        assert!(!harness.ctx.text_edit_focused());

        step(
            ID,
            3,
            "Tick it again; look at the side panel and the list; press q.",
        );
        open_settings(&mut harness);
        harness.get_by_label(texts::DESCRIBE_EVENTS).click();
        harness.run_steps(2);
        close_settings(&mut harness);
        until(&mut harness, |app| {
            app.description.availability == Availability::Available
        });
        let field = description_box(&harness).expect("the box is back");
        assert!(field.rect().bottom() <= meet_with(&harness).rect().top());
        assert!(q_listed(&mut harness));
        press(&mut harness, 'q');
        assert!(description_box(&harness).unwrap().is_focused());
    }

    #[test]
    fn tc_10_03_an_address_off_this_computer_warns_and_a_local_one_does_not() {
        const ID: &str = "TC-10-03";
        let mut harness = app(Settings::default(), None);
        open_settings(&mut harness);

        step(ID, 1, "Enter each local address in turn.");
        let mut current = DEFAULT_ADDRESS.to_string();
        for local in [
            "http://localhost:11434",
            "http://127.0.0.1:11434",
            "http://[::1]:11434",
        ] {
            enter(&mut harness, &current, local);
            assert_eq!(harness.state().settings.ollama.address, local);
            assert!(!shows(&harness, texts::LEAVES_COMPUTER), "{local}");
            current = local.to_string();
        }

        step(ID, 2, "Enter http://192.168.1.20:11434.");
        enter(&mut harness, &current, "http://192.168.1.20:11434");
        assert!(shows(&harness, texts::LEAVES_COMPUTER));

        step(ID, 3, "Enter http://gpu-box.lan:11434.");
        enter(
            &mut harness,
            "http://192.168.1.20:11434",
            "http://gpu-box.lan:11434",
        );
        assert!(shows(&harness, texts::LEAVES_COMPUTER));

        step(ID, 4, "Enter http://localhost:11434 again.");
        enter(
            &mut harness,
            "http://gpu-box.lan:11434",
            "http://localhost:11434",
        );
        assert!(!shows(&harness, texts::LEAVES_COMPUTER));
    }

    #[test]
    fn tc_10_06_the_box_is_greyed_out_naming_the_reason_with_a_link_to_the_settings() {
        const ID: &str = "TC-10-06";
        let address = nothing_answers();
        let mut settings = Settings::default();
        settings.ollama.address = address.clone();

        step(ID, 1, "Case A: start the app and look at the side panel.");
        // Short, so that the settings scroll and the section is further down than they show.
        let window = egui::vec2(1100.0, 560.0);
        let mut harness = app_sized(window, settings, None);
        until(&mut harness, |app| {
            app.description.availability == Availability::NotRunning
        });
        let field = description_box(&harness).expect("the box is in the side panel");
        assert!(field.accesskit_node().is_disabled());
        field.type_text("Kawa");
        harness.run_steps(2);
        assert_eq!(harness.state().description.field, "");
        assert!(shows(
            &harness,
            &format!("Ollama is not available at {address}")
        ));
        assert!(shows(&harness, texts::OPEN_SETTINGS));

        step(ID, 2, "Click the link.");
        harness.get_by_label(texts::OPEN_SETTINGS).click();
        harness.run_steps(3);
        assert!(matches!(harness.state().dialog, Some(Dialog::Settings)));
        let heading = (harness
            .get_all_by_value(texts::EVENTS_FROM_DESCRIPTION)
            .next())
        .unwrap()
        .rect();
        assert!(
            heading.top() >= 0.0 && heading.bottom() <= window.y,
            "{heading:?}"
        );
        assert!(shows(&harness, &address));
        drop(harness);

        step(ID, 3, "Case B: start the app and look at the side panel.");
        let ollama = FakeOllama::start(&["llama3.2:3b"]);
        let mut harness = app(on(&ollama), None);
        until(&mut harness, |app| {
            app.description.availability == Availability::ModelMissing
        });
        assert!(
            description_box(&harness)
                .unwrap()
                .accesskit_node()
                .is_disabled()
        );
        assert!(shows(&harness, "The model gemma4:e4b is not installed"));
        assert!(shows(&harness, texts::OPEN_SETTINGS));
    }

    #[test]
    fn tc_10_07_ollama_stopping_greys_the_box_keeping_the_text_and_starting_brings_it_back() {
        const ID: &str = "TC-10-07";
        let ollama = FakeOllama::start(&[DEFAULT_MODEL]);
        let mut harness = app(on(&ollama), None);
        until(&mut harness, |app| {
            app.description.availability == Availability::Available
        });

        step(
            ID,
            1,
            "Press q and type the description, without sending it.",
        );
        press(&mut harness, 'q');
        description_box(&harness)
            .unwrap()
            .type_text("Kino jutro o 20");
        harness.run_steps(2);
        let field = description_box(&harness).unwrap();
        assert!(!field.accesskit_node().is_disabled());
        assert_eq!(field.value().as_deref(), Some("Kino jutro o 20"));

        step(ID, 2, "Stop the fake Ollama; press Enter.");
        ollama.stop();
        harness.key_press(egui::Key::Enter);
        until(&mut harness, |app| {
            app.description.availability == Availability::NotRunning
        });
        assert!(harness.state().dialog.is_none());
        let field = description_box(&harness).unwrap();
        assert!(field.accesskit_node().is_disabled());
        assert_eq!(field.value().as_deref(), Some("Kino jutro o 20"));
        let why = format!("Ollama is not available at {}", ollama.address);
        assert!(shows(&harness, &why));

        step(ID, 3, "Start the fake Ollama again; click the box.");
        ollama.resume();
        description_box(&harness).unwrap().click();
        until(&mut harness, |app| {
            app.description.availability == Availability::Available
        });
        let field = description_box(&harness).unwrap();
        assert!(!field.accesskit_node().is_disabled());
        assert!(field.is_focused());
        assert_eq!(field.value().as_deref(), Some("Kino jutro o 20"));
        // Step 4, Enter opening "New event" for "Kino", is reading a description: `US-10-02`
        // (`DEC-215`).
    }

    #[test]
    fn ollama_is_asked_only_for_its_version_and_models_at_the_address_set() {
        // `US-10-01#4` as far as this story goes (`DEC-215`): what the app asks Ollama at start,
        // at the box and on Enter goes to the address set, and only for its version and models;
        // that no proxy is used on the way is `ollama::tests::the_check_ignores_a_proxy_…`.
        let ollama = FakeOllama::start(&[DEFAULT_MODEL]);
        let mut harness = app(on(&ollama), None);
        until(&mut harness, |app| {
            app.description.availability == Availability::Available
        });
        press(&mut harness, 'q');
        description_box(&harness)
            .unwrap()
            .type_text("Kawa jutro o 10");
        harness.key_press(egui::Key::Enter);
        harness.run_steps(3);
        let deadline = Instant::now() + Duration::from_secs(10);
        while ollama.requests().len() < 4 {
            assert!(Instant::now() < deadline, "{:?}", ollama.requests());
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            (ollama.requests().iter())
                .all(|request| request == "GET /api/version" || request == "GET /api/tags"),
            "{:?}",
            ollama.requests()
        );
        assert!(ollama::is_local(&harness.state().settings.ollama.address));
    }
}
