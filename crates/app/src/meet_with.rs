//! Draws the side panel's "Meet with…" (`US-06-09`): the field, the people suggested for what is
//! typed, and the people chosen, each with their colour, whether their calendar can be seen and
//! a way to remove them; **Create event** under them. What choosing does is
//! `calendar_core::meet_with`'s.

use calendar_core::guests::Person;
use calendar_core::meet_with::MeetWith;
use calendar_core::texts;
use eframe::egui;

use crate::grid_view::color;
use crate::widgets::{ROW_ICON, close_button, color_mark, text_field};

/// What the user did in "Meet with…".
pub enum Action {
    /// A person suggested was clicked.
    Choose(Person),
    /// Enter in the field.
    Enter,
    /// The person with the address is to be removed.
    Remove(String),
    /// **Create event** (`MOD-06-F26`).
    Create,
}

/// Draws `meet` with the people `suggested` for what is typed; `seen` says of each person chosen
/// whether their calendar can be seen. Offline or without a Google account the field is
/// unavailable and says why (`MOD-06-F27`, MOD-09 *Constraints and edge cases*).
pub fn show(
    ui: &mut egui::Ui,
    meet: &mut MeetWith,
    suggested: &[Person],
    seen: &[bool],
    unavailable: Option<&str>,
) -> Option<Action> {
    let mut action = None;
    let available = unavailable.is_none();
    let field = ui
        .add_enabled_ui(available, |ui| {
            text_field(
                ui,
                egui::TextEdit::singleline(&mut meet.field)
                    .hint_text(texts::MEET_WITH)
                    .desired_width(f32::INFINITY),
            )
        })
        .inner;
    if let Some(why) = unavailable {
        ui.weak(why);
    }
    if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        action = Some(Action::Enter);
        // The field keeps the keyboard for the next person.
        field.request_focus();
    }
    if available {
        for person in suggested {
            if ui.button(person.label()).clicked() {
                action = Some(Action::Choose(person.clone()));
            }
        }
    }
    if meet.chosen.is_empty() {
        return action;
    }
    for (chosen, &seen) in meet.chosen.iter().zip(seen) {
        // The cross first, so a long name is cut short before it rather than under it.
        egui::Sides::new().shrink_left().show(
            ui,
            |ui| {
                color_mark(ui, color(chosen.color));
                ui.add(egui::Label::new(chosen.name()).truncate());
            },
            |ui| {
                if close_button(ui, texts::REMOVE).clicked() {
                    action = Some(Action::Remove(chosen.person.email.clone()));
                }
            },
        );
        if !seen {
            // Under the name, in line with it.
            ui.horizontal(|ui| {
                ui.add_space(ROW_ICON + ui.spacing().item_spacing.x);
                ui.weak(texts::CALENDAR_CANNOT_BE_SEEN);
            });
        }
    }
    ui.add_space(4.0);
    if ui.button(texts::CREATE_EVENT).clicked() {
        action = Some(Action::Create);
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::{self, texts as drawn_texts};
    use calendar_core::meet_with::{Chosen, unavailable};
    use calendar_core::model::Rgb;
    use calendar_core::theme::Scheme;

    /// The texts "Meet with…" draws with Anna chosen, whose calendar cannot be seen, when it is
    /// available or, as `why_not` says, not.
    fn drawn(why_not: Option<&str>) -> Vec<String> {
        let mut meet = MeetWith {
            field: String::new(),
            chosen: vec![Chosen {
                person: Person {
                    name: "Anna Nowak".into(),
                    email: "anna.nowak@example.com".into(),
                },
                account_id: "1".into(),
                color: Rgb(0xAD, 0x14, 0x57),
            }],
        };
        let shapes = screen::drawn(egui::vec2(300.0, 400.0), 1.0, Scheme::Light, None, |ui| {
            show(ui, &mut meet, &[], &[false], why_not);
        });
        (drawn_texts(&shapes).into_iter())
            .map(|(_, text)| text.galley.text().to_string())
            .collect()
    }

    #[test]
    fn tc_06_16_offline_the_field_says_it_needs_a_connection_and_the_list_stays() {
        let offline = drawn(unavailable(true, false));
        assert!(
            offline.iter().any(|t| t == texts::MEET_WITH_OFFLINE),
            "{offline:?}"
        );
        assert!(offline.iter().any(|t| t == "Anna Nowak"));

        let online = drawn(unavailable(true, true));
        assert!(!online.iter().any(|t| t == texts::MEET_WITH_OFFLINE));
        assert!(online.iter().any(|t| t == texts::CALENDAR_CANNOT_BE_SEEN));
        assert!(online.iter().any(|t| t == texts::CREATE_EVENT));
    }

    #[test]
    fn tc_09_03_without_a_google_account_the_field_says_it_needs_one() {
        let texts = drawn(unavailable(false, true));

        assert!(
            texts.iter().any(|t| t == texts::MEET_WITH_NEEDS_ACCOUNT),
            "{texts:?}"
        );
        assert!(!texts.iter().any(|t| t == texts::MEET_WITH_OFFLINE));
    }
}
