//! What the agenda view shows, independent of how it is drawn: the events from now on, grouped
//! by day, without empty days, up to a horizon that grows as the user scrolls.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;

use crate::layout::{is_strip_event, strip_dates};
use crate::model::{Event, Rgb, Timing};

#[derive(Clone, Debug, PartialEq)]
pub struct AgendaItem {
    /// Index of the event in the store, to open it.
    pub event: usize,
    /// `None` for events of calendars shared as free/busy only.
    pub title: Option<String>,
    pub color: Rgb,
    /// The event's calendar cannot be changed (`MOD-02-U01`).
    pub read_only: bool,
    /// Google has not confirmed the new event yet (`MOD-05-U03`).
    pub saving: bool,
    /// Start and end of a timed event, also one lasting a day or more; `None` for all-day events.
    pub time: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgendaDay {
    pub date: NaiveDate,
    pub is_today: bool,
    /// All-day and multi-day events first, then timed ones by start.
    pub items: Vec<AgendaItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgendaViewModel {
    pub zone: Tz,
    /// Days from today to the horizon that have at least one event.
    pub days: Vec<AgendaDay>,
    /// Whether events lie past the horizon, to be loaded by scrolling further.
    pub has_more: bool,
}

impl AgendaViewModel {
    /// Nothing ahead at all: the agenda says there are no upcoming events.
    pub fn is_empty(&self) -> bool {
        self.days.is_empty() && !self.has_more
    }
}

/// The events that have not ended by `now`, on the days from today up to `horizon` days later.
/// An event is listed on every one of those days it covers.
pub fn agenda_view(
    events: &[Event],
    zone: Tz,
    now: DateTime<Utc>,
    horizon: i64,
) -> AgendaViewModel {
    let today = now.with_timezone(&zone).date_naive();
    let last = today + Duration::days(horizon - 1);
    let mut per_day: BTreeMap<NaiveDate, Vec<usize>> = BTreeMap::new();
    let mut has_more = false;
    for (i, event) in events.iter().enumerate() {
        let (from, to) = strip_dates(event, zone);
        // A zero-length event still sits on its day.
        let to = to.max(from);
        let ended = match event.timing {
            Timing::Timed { end, .. } => end <= now,
            Timing::AllDay { .. } => to < today,
        };
        if ended {
            continue;
        }
        has_more |= to > last;
        let mut date = from.max(today);
        while date <= to.min(last) {
            per_day.entry(date).or_default().push(i);
            date += Duration::days(1);
        }
    }

    // All-day and multi-day events first, earliest and longest first; then timed ones by start.
    let order = |&i: &usize| {
        let event = &events[i];
        let (from, to) = strip_dates(event, zone);
        (
            !is_strip_event(event),
            event_time(event),
            from,
            std::cmp::Reverse(to),
        )
    };
    let days = per_day
        .into_iter()
        .map(|(date, mut indices)| {
            indices.sort_by_cached_key(order);
            AgendaDay {
                date,
                is_today: date == today,
                items: indices
                    .into_iter()
                    .map(|i| {
                        let event = &events[i];
                        AgendaItem {
                            event: i,
                            title: (!event.busy_only).then(|| event.title.clone()),
                            color: event.color,
                            read_only: event.read_only,
                            saving: event.saving,
                            time: event_time(event),
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    AgendaViewModel {
        zone,
        days,
        has_more,
    }
}

/// Start and end of a timed event; all-day events have none.
fn event_time(event: &Event) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    match event.timing {
        Timing::Timed { start, end, .. } => Some((start, end)),
        Timing::AllDay { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{all_day, d, timed};
    use chrono::TimeZone;
    use chrono_tz::Europe::Warsaw;

    /// `TC-03-07`: Wednesday 2026-11-11 10:00 in Warsaw.
    fn clock() -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(2026, 11, 11, 10, 0, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn titles(day: &AgendaDay) -> Vec<&str> {
        day.items
            .iter()
            .map(|i| i.title.as_deref().unwrap_or("Busy"))
            .collect()
    }

    #[test]
    fn tc_03_07_agenda_groups_upcoming_events_by_day_without_empty_days() {
        let events = [
            timed("Review", d(2026, 11, 11), (14, 0), (15, 0)),
            timed("Planning", d(2026, 11, 13), (9, 0), (10, 0)),
        ];
        let vm = agenda_view(&events, Warsaw, clock(), 30);
        let dates: Vec<NaiveDate> = vm.days.iter().map(|d| d.date).collect();
        assert_eq!(dates, vec![d(2026, 11, 11), d(2026, 11, 13)]);
        assert_eq!(titles(&vm.days[0]), vec!["Review"]);
        assert_eq!(titles(&vm.days[1]), vec!["Planning"]);
        assert!(!vm.has_more);
        assert!(!vm.is_empty());
    }

    #[test]
    fn tc_03_07_agenda_with_nothing_ahead_says_so() {
        let vm = agenda_view(&[], Warsaw, clock(), 30);
        assert!(vm.days.is_empty());
        assert!(vm.is_empty());
    }

    #[test]
    fn past_events_are_left_out_and_an_event_in_progress_stays() {
        let events = [
            timed("Yesterday", d(2026, 11, 10), (14, 0), (15, 0)),
            timed("Breakfast", d(2026, 11, 11), (8, 0), (9, 0)),
            timed("Ongoing", d(2026, 11, 11), (9, 30), (10, 30)),
            all_day("Holiday", d(2026, 11, 11), 1),
        ];
        let vm = agenda_view(&events, Warsaw, clock(), 30);
        assert_eq!(vm.days.len(), 1);
        assert_eq!(titles(&vm.days[0]), vec!["Holiday", "Ongoing"]);

        let only_past = &events[..2];
        assert!(agenda_view(only_past, Warsaw, clock(), 30).is_empty());
    }

    #[test]
    fn events_past_the_horizon_load_with_a_longer_horizon() {
        let events = [
            timed("Soon", d(2026, 11, 12), (9, 0), (10, 0)),
            timed("Later", d(2027, 2, 1), (9, 0), (10, 0)),
        ];
        let first = agenda_view(&events, Warsaw, clock(), 30);
        assert_eq!(first.days.len(), 1);
        assert!(first.has_more);
        assert!(!first.is_empty());

        let more = agenda_view(&events, Warsaw, clock(), 90);
        let dates: Vec<NaiveDate> = more.days.iter().map(|d| d.date).collect();
        assert_eq!(dates, vec![d(2026, 11, 12), d(2027, 2, 1)]);
        assert!(!more.has_more);
    }

    #[test]
    fn multi_day_and_overnight_events_are_listed_on_each_day_ahead() {
        let events = [
            timed("Late", d(2026, 11, 13), (23, 0), (25, 0)),
            all_day("Trip", d(2026, 11, 9), 5),
            timed("Early", d(2026, 11, 13), (7, 0), (8, 0)),
        ];
        let vm = agenda_view(&events, Warsaw, clock(), 30);
        let dates: Vec<NaiveDate> = vm.days.iter().map(|d| d.date).collect();
        // The trip started before today: it is listed from today on, not on its past days.
        assert_eq!(
            dates,
            vec![
                d(2026, 11, 11),
                d(2026, 11, 12),
                d(2026, 11, 13),
                d(2026, 11, 14)
            ]
        );
        assert_eq!(titles(&vm.days[2]), vec!["Trip", "Early", "Late"]);
        assert_eq!(titles(&vm.days[3]), vec!["Late"]);
        assert!(vm.days[2].items[0].time.is_none());
        assert!(vm.days[2].items[1].time.is_some());
    }

    #[test]
    fn a_timed_event_of_a_day_or_more_keeps_its_times_and_comes_first() {
        let events = [
            timed("Early", d(2026, 11, 12), (7, 0), (8, 0)),
            timed("Shift", d(2026, 11, 12), (10, 0), (35, 0)),
        ];
        let vm = agenda_view(&events, Warsaw, clock(), 30);
        assert_eq!(titles(&vm.days[0]), vec!["Shift", "Early"]);
        assert!(vm.days[0].items[0].time.is_some());
        assert_eq!(titles(&vm.days[1]), vec!["Shift"]);
    }

    #[test]
    fn today_is_highlighted_and_free_busy_events_are_busy_without_details() {
        let mut busy = timed("Secret", d(2026, 11, 12), (13, 0), (14, 0));
        busy.busy_only = true;
        let events = [timed("Review", d(2026, 11, 11), (14, 0), (15, 0)), busy];
        let vm = agenda_view(&events, Warsaw, clock(), 30);
        assert!(vm.days[0].is_today);
        assert!(!vm.days[1].is_today);
        let item = &vm.days[1].items[0];
        assert_eq!(item.title, None);
        assert!(item.time.is_some());
    }

    #[test]
    fn days_follow_the_given_zone() {
        // 23:30 in Warsaw on the 12th is already the 13th in Tokyo.
        let events = [timed("Call", d(2026, 11, 12), (23, 30), (24, 0))];
        let vm = agenda_view(&events, chrono_tz::Asia::Tokyo, clock(), 30);
        assert_eq!(vm.days[0].date, d(2026, 11, 13));
    }
}
