//! Google Calendar's keyboard shortcuts (`DEC-55`): which key does what, and the list "?" shows.
//! Independent of how keys are read or the list is drawn.

use chrono::NaiveDate;

use crate::navigation::{Navigation, View};
use crate::texts;
use crate::week::GridView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Switch(View),
    Next,
    Previous,
    Today,
    /// Asks for a date to go to.
    GoToDate,
    NewEvent,
    ShowShortcuts,
}

/// Every shortcut with what it does, in the order the list shows them. `/` (search) is not bound:
/// there is no search (`DEC-30`).
pub const SHORTCUTS: [(char, Command, &str); 13] = [
    (
        'd',
        Command::Switch(View::Grid(GridView::Day)),
        texts::DAY_VIEW,
    ),
    (
        'x',
        Command::Switch(View::Grid(GridView::ThreeDays)),
        texts::THREE_DAY_VIEW,
    ),
    (
        'w',
        Command::Switch(View::Grid(GridView::Week)),
        texts::WEEK_VIEW,
    ),
    ('m', Command::Switch(View::Month), texts::MONTH_VIEW),
    ('a', Command::Switch(View::Agenda), texts::AGENDA_VIEW),
    ('j', Command::Next, texts::NEXT_PERIOD),
    ('n', Command::Next, texts::NEXT_PERIOD),
    ('k', Command::Previous, texts::PREVIOUS_PERIOD),
    ('p', Command::Previous, texts::PREVIOUS_PERIOD),
    ('t', Command::Today, texts::TODAY),
    ('g', Command::GoToDate, texts::GO_TO_DATE),
    ('c', Command::NewEvent, texts::NEW_EVENT),
    ('?', Command::ShowShortcuts, texts::KEYBOARD_SHORTCUTS),
];

/// A row of the shortcuts' list: its keys and what they do.
pub type Row = (Vec<char>, &'static str);

/// The list "?" shows: the shortcuts in groups (views, getting around, the rest), each group's
/// title with its rows, keys doing the same thing on one row, in the order of `SHORTCUTS`.
pub fn listed() -> Vec<(&'static str, Vec<Row>)> {
    let group = |command: Command| match command {
        Command::Switch(_) => texts::SHORTCUTS_VIEWS,
        Command::Next | Command::Previous | Command::Today | Command::GoToDate => {
            texts::SHORTCUTS_NAVIGATION
        }
        Command::NewEvent | Command::ShowShortcuts => texts::SHORTCUTS_ACTIONS,
    };
    let mut groups: Vec<(&str, Vec<Row>)> = Vec::new();
    for (key, command, description) in SHORTCUTS {
        let title = group(command);
        if groups.last().is_none_or(|(last, _)| *last != title) {
            groups.push((title, Vec::new()));
        }
        let rows = &mut groups.last_mut().expect("a group").1;
        match rows.iter_mut().find(|(_, said)| *said == description) {
            Some((keys, _)) => keys.push(key),
            None => rows.push((vec![key], description)),
        }
    }
    groups
}

/// The command of a typed character, if it is a shortcut.
pub fn command(key: char) -> Option<Command> {
    SHORTCUTS
        .iter()
        .find(|(shortcut, _, _)| *shortcut == key)
        .map(|(_, command, _)| *command)
}

impl Navigation {
    /// Carries out a command that only moves the view; the others (go to date, new event, the
    /// shortcut list) open something and are the caller's.
    pub fn run(&mut self, command: Command, today: NaiveDate) {
        match command {
            Command::Switch(view) => self.switch(view),
            Command::Next => self.next(),
            Command::Previous => self.previous(),
            Command::Today => self.today(today),
            Command::GoToDate | Command::NewEvent | Command::ShowShortcuts => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;

    /// `TC-03-11`: the clock says Wednesday 11 November 2026; the week of 9–15 November is shown.
    fn tc_03_11_start() -> Navigation {
        Navigation::start(
            Some(View::Grid(GridView::Week)),
            d(2026, 11, 11),
            chrono::Weekday::Mon,
        )
    }

    fn press(nav: &mut Navigation, key: char) -> Command {
        let command = command(key).expect("a shortcut");
        nav.run(command, d(2026, 11, 11));
        command
    }

    #[test]
    fn the_list_groups_the_shortcuts_and_puts_keys_doing_the_same_on_one_row() {
        let listed = listed();
        let titles: Vec<&str> = listed.iter().map(|(title, _)| *title).collect();
        assert_eq!(
            titles,
            [
                texts::SHORTCUTS_VIEWS,
                texts::SHORTCUTS_NAVIGATION,
                texts::SHORTCUTS_ACTIONS
            ]
        );
        assert_eq!(listed[0].1.len(), 5);
        assert_eq!(
            listed[1].1,
            [
                (vec!['j', 'n'], texts::NEXT_PERIOD),
                (vec!['k', 'p'], texts::PREVIOUS_PERIOD),
                (vec!['t'], texts::TODAY),
                (vec!['g'], texts::GO_TO_DATE),
            ]
        );
        assert_eq!(
            listed[2].1,
            [
                (vec!['c'], texts::NEW_EVENT),
                (vec!['?'], texts::KEYBOARD_SHORTCUTS),
            ]
        );
    }

    #[test]
    fn tc_03_11_view_keys_switch_to_day_three_days_week_month_and_agenda() {
        let mut nav = tc_03_11_start();
        let mut shown = Vec::new();
        for key in ['d', 'x', 'w', 'm', 'a'] {
            press(&mut nav, key);
            shown.push(nav.view);
        }
        assert_eq!(shown, View::ALL);
    }

    #[test]
    fn tc_03_11_j_k_and_t_move_between_weeks_and_back_to_today() {
        let mut nav = tc_03_11_start();
        let mut periods = Vec::new();
        for key in ['w', 'j', 'k', 'j', 't'] {
            press(&mut nav, key);
            periods.push(nav.period());
        }
        assert_eq!(
            periods,
            [
                Some((d(2026, 11, 9), d(2026, 11, 15))),
                Some((d(2026, 11, 16), d(2026, 11, 22))),
                Some((d(2026, 11, 9), d(2026, 11, 15))),
                Some((d(2026, 11, 16), d(2026, 11, 22))),
                Some((d(2026, 11, 9), d(2026, 11, 15))),
            ]
        );
    }

    #[test]
    fn tc_03_11_question_mark_shows_the_shortcut_list() {
        let mut nav = tc_03_11_start();
        assert_eq!(press(&mut nav, '?'), Command::ShowShortcuts);
        assert_eq!(nav, tc_03_11_start());
        let listed: Vec<char> = SHORTCUTS.iter().map(|(key, _, _)| *key).collect();
        assert_eq!(listed, "dxwmajnkptgc?".chars().collect::<Vec<_>>());
    }

    #[test]
    fn n_and_p_are_next_and_previous_like_j_and_k() {
        assert_eq!(command('n'), command('j'));
        assert_eq!(command('p'), command('k'));
        assert_eq!(command('j'), Some(Command::Next));
        assert_eq!(command('k'), Some(Command::Previous));
    }

    #[test]
    fn g_asks_for_a_date_and_c_starts_a_new_event_without_moving_the_view() {
        let mut nav = tc_03_11_start();
        assert_eq!(press(&mut nav, 'g'), Command::GoToDate);
        assert_eq!(press(&mut nav, 'c'), Command::NewEvent);
        assert_eq!(nav, tc_03_11_start());
    }

    #[test]
    fn search_and_other_keys_are_not_shortcuts() {
        for key in ['/', 'D', 'q', ' ', '1'] {
            assert_eq!(command(key), None, "{key:?}");
        }
    }
}
