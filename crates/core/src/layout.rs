//! Placing a week's events: timed events into day columns (side by side where they overlap),
//! all-day and multi-day events into lanes of the strip above the hours.

use chrono::{Duration, NaiveDate};
use chrono_tz::Tz;

use crate::model::{Event, Timing};
use crate::week::{Week, day_span, wall_minute};

/// A timed event's part that falls on one day of the week, in wall-clock minutes of that day
/// (0 to 1440).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimedPlacement {
    pub event: usize,
    pub day: usize,
    pub start_min: u32,
    pub end_min: u32,
    pub column: u32,
    pub columns: u32,
}

/// An all-day or multi-day event in the strip, over days `first_day..=last_day` of the week.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllDayBar {
    pub event: usize,
    pub first_day: usize,
    pub last_day: usize,
    pub lane: u32,
}

/// Timed events lasting a full day or longer go to the strip, as in Google Calendar.
fn is_strip_event(event: &Event) -> bool {
    match event.timing {
        Timing::AllDay { .. } => true,
        Timing::Timed { start, end, .. } => end - start >= Duration::hours(24),
    }
}

pub fn timed_placements(events: &[Event], week: Week, tz: Tz) -> Vec<TimedPlacement> {
    let spans = week.days().map(|d| day_span(d, tz));
    let mut out = Vec::new();
    for (i, event) in events.iter().enumerate() {
        let Timing::Timed { start, end, .. } = event.timing else {
            continue;
        };
        if is_strip_event(event) {
            continue;
        }
        for (day, (day_start, day_end)) in spans.iter().enumerate() {
            let s = start.max(*day_start);
            let e = end.min(*day_end);
            if s < e {
                let start_min = wall_minute(s, tz);
                let mut end_min = if e == *day_end {
                    24 * 60
                } else {
                    wall_minute(e, tz)
                };
                // An event inside the repeated hour of a daylight-saving day ends "before" it
                // starts on the clock; keep its real length instead.
                if end_min <= start_min {
                    end_min = (start_min + (e - s).num_minutes() as u32).min(24 * 60);
                }
                out.push(TimedPlacement {
                    event: i,
                    day,
                    start_min,
                    end_min,
                    column: 0,
                    columns: 1,
                });
            }
        }
    }
    pack_columns(&mut out);
    out
}

/// Gives overlapping placements of a day separate columns. A group of transitively overlapping
/// placements shares one column count, so all of them get the same width.
fn pack_columns(placements: &mut [TimedPlacement]) {
    placements.sort_by_key(|p| (p.day, p.start_min, std::cmp::Reverse(p.end_min)));
    let mut group_start = 0;
    while group_start < placements.len() {
        let day = placements[group_start].day;
        let mut group_end = group_start;
        let mut group_last_end = 0;
        let mut column_ends: Vec<u32> = Vec::new();
        while group_end < placements.len()
            && placements[group_end].day == day
            && (group_end == group_start || placements[group_end].start_min < group_last_end)
        {
            let p = &mut placements[group_end];
            let column = match column_ends.iter().position(|&end| end <= p.start_min) {
                Some(c) => c,
                None => {
                    column_ends.push(0);
                    column_ends.len() - 1
                }
            };
            column_ends[column] = p.end_min;
            p.column = column as u32;
            group_last_end = group_last_end.max(p.end_min);
            group_end += 1;
        }
        for p in &mut placements[group_start..group_end] {
            p.columns = column_ends.len() as u32;
        }
        group_start = group_end;
    }
}

fn strip_dates(event: &Event, tz: Tz) -> (NaiveDate, NaiveDate) {
    match event.timing {
        Timing::AllDay {
            start,
            end_exclusive,
        } => (start, end_exclusive - Duration::days(1)),
        Timing::Timed { start, end, .. } => (
            start.with_timezone(&tz).date_naive(),
            (end - Duration::seconds(1)).with_timezone(&tz).date_naive(),
        ),
    }
}

pub fn all_day_bars(events: &[Event], week: Week, tz: Tz) -> Vec<AllDayBar> {
    let days = week.days();
    let mut bars: Vec<AllDayBar> = events
        .iter()
        .enumerate()
        .filter(|(_, e)| is_strip_event(e))
        .filter_map(|(i, e)| {
            let (first, last) = strip_dates(e, tz);
            if last < days[0] || first > days[6] {
                return None;
            }
            let index = |d: NaiveDate| (d - days[0]).num_days().clamp(0, 6) as usize;
            Some(AllDayBar {
                event: i,
                first_day: index(first),
                last_day: index(last),
                lane: 0,
            })
        })
        .collect();
    bars.sort_by_key(|b| (b.first_day, std::cmp::Reverse(b.last_day)));
    let mut lane_ends: Vec<usize> = Vec::new();
    for bar in &mut bars {
        let lane = match lane_ends.iter().position(|&end| end < bar.first_day) {
            Some(l) => l,
            None => {
                lane_ends.push(0);
                lane_ends.len() - 1
            }
        };
        lane_ends[lane] = bar.last_day;
        bar.lane = lane as u32;
    }
    bars
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{d, timed};
    use chrono_tz::Europe::Warsaw;

    fn week() -> Week {
        Week::containing(d(2026, 11, 11))
    }

    #[test]
    fn three_overlapping_events_get_three_disjoint_columns() {
        let events = [
            timed("a", d(2026, 11, 11), (10, 0), (11, 0)),
            timed("b", d(2026, 11, 11), (10, 30), (11, 30)),
            timed("c", d(2026, 11, 11), (10, 45), (12, 0)),
        ];
        let p = timed_placements(&events, week(), Warsaw);
        assert_eq!(p.len(), 3);
        assert!(p.iter().all(|p| p.columns == 3));
        let mut columns: Vec<u32> = p.iter().map(|p| p.column).collect();
        columns.sort();
        assert_eq!(columns, vec![0, 1, 2]);
    }

    #[test]
    fn separate_groups_keep_full_width() {
        let events = [
            timed("a", d(2026, 11, 11), (9, 0), (10, 0)),
            timed("b", d(2026, 11, 11), (10, 0), (11, 0)),
        ];
        let p = timed_placements(&events, week(), Warsaw);
        assert!(p.iter().all(|p| p.columns == 1 && p.column == 0));
    }

    #[test]
    fn an_event_past_midnight_is_shown_on_both_days() {
        let events = [timed("late", d(2026, 11, 11), (23, 0), (25, 0))];
        let p = timed_placements(&events, week(), Warsaw);
        assert_eq!(p.len(), 2);
        assert_eq!(
            (p[0].day, p[0].start_min, p[0].end_min),
            (2, 23 * 60, 24 * 60)
        );
        assert_eq!((p[1].day, p[1].start_min, p[1].end_min), (3, 0, 60));
    }

    #[test]
    fn events_outside_the_week_are_not_placed() {
        let events = [timed("x", d(2026, 11, 20), (9, 0), (10, 0))];
        assert!(timed_placements(&events, week(), Warsaw).is_empty());
    }

    #[test]
    fn overlapping_all_day_bars_take_separate_lanes() {
        let events = crate::fixtures::all_day_pair();
        let bars = all_day_bars(&events, week(), Warsaw);
        assert_eq!(bars.len(), 2);
        assert_ne!(bars[0].lane, bars[1].lane);
    }
}
