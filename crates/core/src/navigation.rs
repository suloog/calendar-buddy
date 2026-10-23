//! Getting around: the view on screen, the selected date, next, previous and today, and the mini
//! calendar. Independent of how any of it is drawn.

use chrono::{Datelike, Duration, Months, NaiveDate, Weekday};

use crate::month::weeks_covering;
use crate::week::GridView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Grid(GridView),
    Month,
    Agenda,
}

impl View {
    /// Every view, in the order of the view picker.
    pub const ALL: [View; 5] = [
        View::Grid(GridView::Day),
        View::Grid(GridView::ThreeDays),
        View::Grid(GridView::Week),
        View::Month,
        View::Agenda,
    ];

    pub fn name(self) -> &'static str {
        match self {
            View::Grid(GridView::Day) => "Day",
            View::Grid(GridView::ThreeDays) => "3 days",
            View::Grid(GridView::Week) => "Week",
            View::Month => "Month",
            View::Agenda => "Agenda",
        }
    }

    /// The name the view is saved under in the settings.
    pub fn key(self) -> &'static str {
        match self {
            View::Grid(GridView::Day) => "day",
            View::Grid(GridView::ThreeDays) => "3days",
            View::Grid(GridView::Week) => "week",
            View::Month => "month",
            View::Agenda => "agenda",
        }
    }

    pub fn from_key(key: &str) -> Option<View> {
        View::ALL.into_iter().find(|view| view.key() == key)
    }
}

/// The view on screen and the date it shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Navigation {
    pub view: View,
    pub selected: NaiveDate,
    /// The first day of the week of the system's locale (`DEC-29`).
    pub week_start: Weekday,
}

impl Navigation {
    /// The app starts in the view used last, on today's date (`DEC-55`); the first time, in the
    /// week view.
    pub fn start(last_view: Option<View>, today: NaiveDate, week_start: Weekday) -> Self {
        Navigation {
            view: last_view.unwrap_or(View::Grid(GridView::Week)),
            selected: today,
            week_start,
        }
    }

    /// Another view of the same selected date.
    pub fn switch(&mut self, view: View) {
        self.view = view;
    }

    pub fn next(&mut self) {
        self.step(1);
    }

    pub fn previous(&mut self) {
        self.step(-1);
    }

    /// Moves by the period of the view: 1, 3 or 7 days, or a month. The agenda always starts now
    /// and has no period to step through.
    fn step(&mut self, by: i32) {
        match self.view {
            View::Grid(grid) => {
                let days = grid.days(self.selected, self.week_start).len() as i64;
                self.selected += Duration::days(by as i64 * days)
            }
            View::Month if by > 0 => self.selected = self.selected + Months::new(1),
            View::Month => self.selected = self.selected - Months::new(1),
            View::Agenda => {}
        }
    }

    pub fn today(&mut self, today: NaiveDate) {
        self.selected = today;
    }

    /// Jumps to a date picked in the mini calendar. The agenda cannot start at a date, so a date
    /// picked there opens in the day view (`DEC-72`).
    pub fn go_to(&mut self, date: NaiveDate) {
        self.selected = date;
        if self.view == View::Agenda {
            self.view = View::Grid(GridView::Day);
        }
    }

    /// The first and last date on screen: the grid's days or the month. The agenda has none.
    pub fn period(&self) -> Option<(NaiveDate, NaiveDate)> {
        match self.view {
            View::Grid(grid) => {
                let days = grid.days(self.selected, self.week_start);
                Some((days[0], days[days.len() - 1]))
            }
            View::Month => {
                let first = self.selected.with_day(1).expect("day 1 exists");
                Some((first, first + Months::new(1) - Duration::days(1)))
            }
            View::Agenda => None,
        }
    }

    /// The first and last day on screen: the grid's days, the month's whole weeks, or the
    /// agenda's days from today.
    pub fn on_screen(&self, today: NaiveDate, agenda_days: i64) -> (NaiveDate, NaiveDate) {
        match (self.view, self.period()) {
            (View::Month, Some((month, _))) => {
                let (first, weeks) = weeks_covering(month, self.week_start);
                (first, first + Duration::days(7 * weeks as i64 - 1))
            }
            (_, Some(period)) => period,
            (_, None) => (today, today + Duration::days(agenda_days - 1)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiniDay {
    pub date: NaiveDate,
    /// False for the days of the previous and next month that fill the first and last week.
    pub in_month: bool,
    pub is_today: bool,
    /// The day is on screen in the main view.
    pub in_period: bool,
}

/// The mini calendar: one month in whole weeks from the first day of the week, marking the period
/// on screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MiniCalendar {
    /// The first day of the month shown.
    pub month: NaiveDate,
    pub weeks: Vec<[MiniDay; 7]>,
}

/// The mini calendar of the month containing `month`, which the user browses on its own.
pub fn mini_calendar(month: NaiveDate, navigation: &Navigation, today: NaiveDate) -> MiniCalendar {
    let month = month.with_day(1).expect("day 1 exists");
    let (first_shown, week_count) = weeks_covering(month, navigation.week_start);
    let period = navigation.period();
    let weeks = (0..week_count)
        .map(|week| {
            std::array::from_fn(|day| {
                let date = first_shown + Duration::days((week * 7 + day) as i64);
                MiniDay {
                    date,
                    in_month: date.month() == month.month(),
                    is_today: date == today,
                    in_period: period.is_some_and(|(first, last)| first <= date && date <= last),
                }
            })
        })
        .collect();
    MiniCalendar { month, weeks }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;

    /// `TC-03-08`: the clock says Wednesday 11 November 2026; the week of 9–15 November is shown.
    fn tc_03_08_start() -> Navigation {
        Navigation::start(
            Some(View::Grid(GridView::Week)),
            d(2026, 11, 11),
            Weekday::Mon,
        )
    }

    #[test]
    fn tc_03_08_switching_to_the_month_view_keeps_the_selected_date() {
        let mut nav = tc_03_08_start();
        assert_eq!(nav.period(), Some((d(2026, 11, 9), d(2026, 11, 15))));
        nav.switch(View::Month);
        assert_eq!(nav.period(), Some((d(2026, 11, 1), d(2026, 11, 30))));
        for view in View::ALL {
            nav.switch(view);
            assert_eq!(nav.selected, d(2026, 11, 11));
            if let Some((first, last)) = nav.period() {
                assert!(first <= nav.selected && nav.selected <= last);
            }
        }
    }

    #[test]
    fn tc_03_08_next_moves_a_month_and_today_comes_back() {
        let mut nav = tc_03_08_start();
        nav.switch(View::Month);
        nav.next();
        assert_eq!(nav.period(), Some((d(2026, 12, 1), d(2026, 12, 31))));
        nav.today(d(2026, 11, 11));
        assert_eq!(nav.period(), Some((d(2026, 11, 1), d(2026, 11, 30))));
    }

    #[test]
    fn tc_03_08_the_app_restarts_in_the_last_view_on_today() {
        let mut nav = tc_03_08_start();
        nav.switch(View::Grid(GridView::ThreeDays));
        nav.next();
        nav.next();
        assert_eq!(nav.period(), Some((d(2026, 11, 17), d(2026, 11, 19))));
        let restarted = Navigation::start(Some(nav.view), d(2026, 11, 11), Weekday::Mon);
        assert_eq!(restarted.view, View::Grid(GridView::ThreeDays));
        assert_eq!(restarted.period(), Some((d(2026, 11, 11), d(2026, 11, 13))));
    }

    #[test]
    fn every_day_on_screen_is_the_grid_the_months_whole_weeks_or_the_agenda_from_today() {
        let mut nav = tc_03_08_start();
        assert_eq!(
            nav.on_screen(d(2026, 11, 11), 30),
            (d(2026, 11, 9), d(2026, 11, 15))
        );
        nav.switch(View::Month);
        assert_eq!(
            nav.on_screen(d(2026, 11, 11), 30),
            (d(2026, 10, 26), d(2026, 12, 6))
        );
        nav.switch(View::Agenda);
        assert_eq!(
            nav.on_screen(d(2026, 11, 11), 30),
            (d(2026, 11, 11), d(2026, 12, 10))
        );
    }

    #[test]
    fn the_first_start_is_the_week_view_on_today() {
        let nav = Navigation::start(None, d(2026, 11, 11), Weekday::Mon);
        assert_eq!(nav.view, View::Grid(GridView::Week));
        assert_eq!(nav.selected, d(2026, 11, 11));
    }

    #[test]
    fn next_and_previous_move_by_the_period_of_the_view() {
        let cases = [
            (View::Grid(GridView::Day), d(2026, 11, 12), d(2026, 11, 10)),
            (
                View::Grid(GridView::ThreeDays),
                d(2026, 11, 14),
                d(2026, 11, 8),
            ),
            (View::Grid(GridView::Week), d(2026, 11, 18), d(2026, 11, 4)),
            (View::Month, d(2026, 12, 11), d(2026, 10, 11)),
            (View::Agenda, d(2026, 11, 11), d(2026, 11, 11)),
        ];
        for (view, after_next, after_previous) in cases {
            let mut nav = Navigation::start(Some(view), d(2026, 11, 11), Weekday::Mon);
            nav.next();
            assert_eq!(nav.selected, after_next, "{view:?} next");
            nav.previous();
            nav.previous();
            assert_eq!(nav.selected, after_previous, "{view:?} previous");
        }
    }

    #[test]
    fn a_month_ahead_of_the_31st_is_the_last_day_of_a_shorter_month() {
        let mut nav = Navigation::start(Some(View::Month), d(2027, 1, 31), Weekday::Mon);
        nav.next();
        assert_eq!(nav.selected, d(2027, 2, 28));
    }

    #[test]
    fn a_date_picked_in_the_mini_calendar_is_shown_in_the_view() {
        let mut nav = tc_03_08_start();
        nav.go_to(d(2027, 3, 4));
        assert_eq!(nav.view, View::Grid(GridView::Week));
        assert_eq!(nav.period(), Some((d(2027, 3, 1), d(2027, 3, 7))));

        let mut agenda = Navigation::start(Some(View::Agenda), d(2026, 11, 11), Weekday::Mon);
        agenda.go_to(d(2026, 12, 24));
        assert_eq!(agenda.view, View::Grid(GridView::Day));
        assert_eq!(agenda.period(), Some((d(2026, 12, 24), d(2026, 12, 24))));
    }

    fn marked(mini: &MiniCalendar) -> Vec<NaiveDate> {
        mini.weeks
            .iter()
            .flatten()
            .filter(|day| day.in_period)
            .map(|day| day.date)
            .collect()
    }

    #[test]
    fn the_mini_calendar_shows_a_month_and_marks_the_period_on_screen() {
        let nav = tc_03_08_start();
        let mini = mini_calendar(d(2026, 11, 11), &nav, d(2026, 11, 11));
        assert_eq!(mini.month, d(2026, 11, 1));
        assert_eq!(mini.weeks.len(), 6);
        assert_eq!(mini.weeks[0][0].date, d(2026, 10, 26));
        assert!(!mini.weeks[0][0].in_month);
        assert!(mini.weeks[0][6].in_month);
        let today: Vec<NaiveDate> = mini
            .weeks
            .iter()
            .flatten()
            .filter(|day| day.is_today)
            .map(|day| day.date)
            .collect();
        assert_eq!(today, vec![d(2026, 11, 11)]);
        assert_eq!(
            marked(&mini),
            (9..=15).map(|day| d(2026, 11, day)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn the_mini_calendar_marks_the_part_of_the_period_in_the_month_it_shows() {
        // The week of 30 November to 6 December, seen from December's mini calendar.
        let nav = Navigation::start(
            Some(View::Grid(GridView::Week)),
            d(2026, 12, 2),
            Weekday::Mon,
        );
        let mini = mini_calendar(d(2026, 12, 1), &nav, d(2026, 11, 11));
        assert_eq!(
            marked(&mini),
            [d(2026, 11, 30)]
                .into_iter()
                .chain((1..=6).map(|day| d(2026, 12, day)))
                .collect::<Vec<_>>()
        );
        let month = Navigation::start(Some(View::Month), d(2026, 11, 11), Weekday::Mon);
        let mini = mini_calendar(d(2026, 11, 1), &month, d(2026, 11, 11));
        assert_eq!(marked(&mini).len(), 30);
        let agenda = Navigation::start(Some(View::Agenda), d(2026, 11, 11), Weekday::Mon);
        let mini = mini_calendar(d(2026, 11, 1), &agenda, d(2026, 11, 11));
        assert!(marked(&mini).is_empty());
    }

    #[test]
    fn with_a_sunday_first_locale_the_week_and_the_mini_calendar_start_on_sunday() {
        let nav = Navigation::start(
            Some(View::Grid(GridView::Week)),
            d(2026, 11, 11),
            Weekday::Sun,
        );
        assert_eq!(nav.period(), Some((d(2026, 11, 8), d(2026, 11, 14))));
        let mini = mini_calendar(d(2026, 11, 11), &nav, d(2026, 11, 11));
        // 1 November 2026 is a Sunday: five weeks, 1 November to 5 December.
        assert_eq!(mini.weeks.len(), 5);
        assert_eq!(mini.weeks[0][0].date, d(2026, 11, 1));
        assert_eq!(
            marked(&mini),
            (8..=14).map(|day| d(2026, 11, day)).collect::<Vec<_>>()
        );
    }
}
