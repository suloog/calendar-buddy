//! What the month view shows, independent of how it is drawn: the weeks covering the month, each
//! day with its events, and how many of them fit.

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use chrono_tz::Tz;

use crate::layout::{is_strip_event, strip_dates};
use crate::model::{Event, Rgb, Timing};
use crate::view_model::shown_title;
use crate::week::Week;

#[derive(Clone, Debug, PartialEq)]
pub struct MonthItem {
    /// Index of the event in the store, to open it.
    pub event: usize,
    /// `None` for events of calendars shared as free/busy only.
    pub title: Option<String>,
    pub color: Rgb,
    /// The event's calendar cannot be changed (`MOD-02-U01`).
    pub read_only: bool,
    /// Google has not confirmed the new event yet (`MOD-05-U03`).
    pub saving: bool,
    /// An invitation awaiting the user's answer, drawn with a dashed outline and a light fill
    /// (`MOD-06-U04`).
    pub awaiting: bool,
    /// Start of a timed event; `None` for all-day and multi-day events.
    pub start: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MonthDay {
    pub date: NaiveDate,
    /// False for the days of the previous and next month that fill the first and last week.
    pub in_month: bool,
    pub is_today: bool,
    /// Every event of the day: all-day and multi-day first, then timed ones by start.
    pub items: Vec<MonthItem>,
}

impl MonthDay {
    /// How many items to draw in `slots` rows, and how many are left for "+N more". When not all
    /// fit, the last row is taken by "+N more".
    pub fn fit(&self, slots: usize) -> (usize, usize) {
        let all = self.items.len();
        if all <= slots {
            (all, 0)
        } else {
            let shown = slots.saturating_sub(1);
            (shown, all - shown)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MonthViewModel {
    pub zone: Tz,
    /// The first day of the month shown.
    pub month: NaiveDate,
    /// Whole weeks covering the month, from the first day of the week.
    pub weeks: Vec<[MonthDay; 7]>,
}

impl MonthViewModel {
    pub fn day(&self, date: NaiveDate) -> Option<&MonthDay> {
        self.weeks.iter().flatten().find(|d| d.date == date)
    }
}

/// Start of an event drawn with its time; all-day and multi-day events have none.
fn timed_start(event: &Event) -> Option<DateTime<Utc>> {
    match event.timing {
        Timing::Timed { start, .. } if !is_strip_event(event) => Some(start),
        _ => None,
    }
}

/// The whole weeks covering the month that starts on `month`, from `week_start`: the first day
/// shown and how many weeks there are (four to six).
pub fn weeks_covering(month: NaiveDate, week_start: Weekday) -> (NaiveDate, usize) {
    let last = month + chrono::Months::new(1) - Duration::days(1);
    let first_shown = Week::containing(month, week_start).first;
    let last_week = Week::containing(last, week_start).first;
    let week_count = ((last_week - first_shown).num_days() / 7 + 1) as usize;
    (first_shown, week_count)
}

/// The month containing `selected`. An event is listed on every day it covers.
pub fn month_view(
    events: &[Event],
    selected: NaiveDate,
    zone: Tz,
    now: DateTime<Utc>,
    week_start: Weekday,
) -> MonthViewModel {
    let month = selected.with_day(1).expect("day 1 exists");
    let (first_shown, week_count) = weeks_covering(month, week_start);
    let last_shown = first_shown + Duration::days(7 * week_count as i64 - 1);

    let mut per_day: Vec<Vec<usize>> = vec![Vec::new(); 7 * week_count];
    for (i, event) in events.iter().enumerate() {
        let (from, to) = strip_dates(event, zone);
        // A zero-length event still sits on its day.
        let to = to.max(from);
        if to < first_shown || from > last_shown {
            continue;
        }
        let mut date = from.max(first_shown);
        while date <= to.min(last_shown) {
            per_day[(date - first_shown).num_days() as usize].push(i);
            date += Duration::days(1);
        }
    }

    // All-day and multi-day events first, earliest and longest first; then timed ones by start.
    let order = |&i: &usize| {
        let event = &events[i];
        let (from, to) = strip_dates(event, zone);
        (
            !is_strip_event(event),
            timed_start(event),
            from,
            std::cmp::Reverse(to),
        )
    };
    let today = now.with_timezone(&zone).date_naive();
    let mut days = per_day.into_iter().enumerate().map(|(n, mut indices)| {
        indices.sort_by_cached_key(order);
        let date = first_shown + Duration::days(n as i64);
        MonthDay {
            date,
            in_month: date.month() == month.month(),
            is_today: date == today,
            items: indices
                .into_iter()
                .map(|i| {
                    let event = &events[i];
                    MonthItem {
                        event: i,
                        title: shown_title(event),
                        color: event.shown_color(),
                        read_only: event.read_only,
                        saving: event.saving,
                        awaiting: event.about.awaiting(),
                        start: timed_start(event),
                    }
                })
                .collect(),
        }
    });
    let weeks = (0..week_count)
        .map(|_| std::array::from_fn(|_| days.next().expect("seven days a week")))
        .collect();

    MonthViewModel { zone, month, weeks }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{all_day, d, timed};
    use chrono::TimeZone;
    use chrono_tz::Europe::Warsaw;

    fn clock() -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(2026, 11, 11, 10, 0, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    /// `TC-03-06`: single events on several November days and eight on Monday 16.
    fn tc_03_06_events() -> Vec<Event> {
        let mut events = vec![
            timed("Dentist", d(2026, 11, 3), (8, 0), (9, 0)),
            timed("Review", d(2026, 11, 11), (14, 0), (15, 0)),
            timed("Dinner", d(2026, 11, 27), (19, 0), (21, 0)),
        ];
        for hour in 8..16 {
            events.push(timed(
                &format!("Monday {hour}"),
                d(2026, 11, 16),
                (hour, 0),
                (hour, 30),
            ));
        }
        events
    }

    fn titles(day: &MonthDay) -> Vec<&str> {
        day.items
            .iter()
            .map(|i| i.title.as_deref().unwrap_or("Busy"))
            .collect()
    }

    #[test]
    fn tc_03_06_month_view_shows_each_day_of_november_with_its_events() {
        let vm = month_view(
            &tc_03_06_events(),
            d(2026, 11, 11),
            Warsaw,
            clock(),
            Weekday::Mon,
        );
        assert_eq!(vm.month, d(2026, 11, 1));
        // 1 November 2026 is a Sunday, 30 November a Monday: six weeks, 26 Oct to 6 Dec.
        assert_eq!(vm.weeks.len(), 6);
        assert_eq!(vm.weeks[0][0].date, d(2026, 10, 26));
        assert_eq!(vm.weeks[5][6].date, d(2026, 12, 6));
        let november: Vec<&MonthDay> = vm.weeks.iter().flatten().filter(|d| d.in_month).collect();
        assert_eq!(november.len(), 30);
        assert_eq!(titles(vm.day(d(2026, 11, 3)).unwrap()), vec!["Dentist"]);
        assert_eq!(titles(vm.day(d(2026, 11, 11)).unwrap()), vec!["Review"]);
        assert_eq!(titles(vm.day(d(2026, 11, 27)).unwrap()), vec!["Dinner"]);
        assert!(vm.day(d(2026, 11, 4)).unwrap().items.is_empty());
    }

    #[test]
    fn tc_03_06_a_full_day_shows_what_fits_and_how_many_more() {
        let vm = month_view(
            &tc_03_06_events(),
            d(2026, 11, 11),
            Warsaw,
            clock(),
            Weekday::Mon,
        );
        let monday = vm.day(d(2026, 11, 16)).unwrap();
        // Four rows fit: three events and "+5 more".
        assert_eq!(monday.fit(4), (3, 5));
        assert_eq!(&titles(monday)[..3], ["Monday 8", "Monday 9", "Monday 10"]);
        // A day whose events fit has no "+N more".
        assert_eq!(vm.day(d(2026, 11, 3)).unwrap().fit(4), (1, 0));
        assert_eq!(monday.fit(8), (8, 0));
    }

    #[test]
    fn tc_03_06_choosing_more_lists_all_eight_events_of_the_day() {
        let vm = month_view(
            &tc_03_06_events(),
            d(2026, 11, 11),
            Warsaw,
            clock(),
            Weekday::Mon,
        );
        let monday = vm.day(d(2026, 11, 16)).unwrap();
        let expected: Vec<String> = (8..16).map(|h| format!("Monday {h}")).collect();
        assert_eq!(titles(monday), expected);
    }

    #[test]
    fn today_is_highlighted_and_free_busy_events_are_busy_without_details() {
        let mut busy = timed("Secret", d(2026, 11, 12), (13, 0), (14, 0));
        busy.busy_only = true;
        let vm = month_view(&[busy], d(2026, 11, 20), Warsaw, clock(), Weekday::Mon);
        let today: Vec<NaiveDate> = vm
            .weeks
            .iter()
            .flatten()
            .filter(|d| d.is_today)
            .map(|d| d.date)
            .collect();
        assert_eq!(today, vec![d(2026, 11, 11)]);
        let item = &vm.day(d(2026, 11, 12)).unwrap().items[0];
        assert_eq!(item.title, None);
        assert!(item.start.is_some());
    }

    #[test]
    fn all_day_and_multi_day_events_come_first_on_every_day_they_cover() {
        let events = [
            timed("Early", d(2026, 11, 13), (7, 0), (8, 0)),
            all_day("Trip", d(2026, 11, 12), 3),
            timed("Late", d(2026, 11, 13), (23, 0), (25, 0)),
        ];
        let vm = month_view(&events, d(2026, 11, 1), Warsaw, clock(), Weekday::Mon);
        assert_eq!(titles(vm.day(d(2026, 11, 12)).unwrap()), vec!["Trip"]);
        assert_eq!(
            titles(vm.day(d(2026, 11, 13)).unwrap()),
            vec!["Trip", "Early", "Late"]
        );
        // The event past midnight is also on the next day.
        assert_eq!(
            titles(vm.day(d(2026, 11, 14)).unwrap()),
            vec!["Trip", "Late"]
        );
        assert!(vm.day(d(2026, 11, 12)).unwrap().items[0].start.is_none());
    }

    #[test]
    fn a_month_starting_on_monday_has_no_days_of_the_previous_month() {
        // February 2027 starts on a Monday and has four whole weeks.
        let vm = month_view(&[], d(2027, 2, 14), Warsaw, clock(), Weekday::Mon);
        assert_eq!(vm.weeks.len(), 4);
        assert!(vm.weeks.iter().flatten().all(|d| d.in_month));
        assert!(vm.weeks.iter().flatten().all(|d| !d.is_today));
    }

    #[test]
    fn with_a_sunday_first_locale_the_month_weeks_start_on_sunday() {
        let vm = month_view(&[], d(2026, 11, 11), Warsaw, clock(), Weekday::Sun);
        // 1 November 2026 is a Sunday, 30 November a Monday: five weeks, 1 Nov to 5 Dec.
        assert_eq!(vm.weeks.len(), 5);
        assert_eq!(vm.weeks[0][0].date, d(2026, 11, 1));
        assert_eq!(vm.weeks[4][6].date, d(2026, 12, 5));
    }
}
