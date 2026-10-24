//! Dragging and resizing an event in the day, 3-day and week views (`US-05-03`): when the event
//! happens once dropped (`MOD-05-F13`, `MOD-05-F14`), and the change sent to Google for it like
//! any other (`MOD-05-F16`). Times move on the clock of the zone the views show, so a move across
//! a daylight-saving change keeps the hour the event was shown at.

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Timelike, Utc};
use chrono_tz::Tz;

use crate::create::{STEP, clicked, wall};
use crate::editor::{Edit, Patch};
use crate::model::{Calendar, Event, Timing};

/// Where a dragged event was dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Moved by whole days and minutes on the clock, keeping its length (`MOD-05-F13`); an
    /// all-day event moves by the days only.
    Moved { days: i64, minutes: i64 },
    /// Its end moved by minutes on the clock (`MOD-05-F14`), the event lasting a step at least.
    Resized { minutes: i64 },
    /// Dropped into the hours of `date` at `minute` of the day: a timed event from that step,
    /// of the default length.
    Timed { date: NaiveDate, minute: u32 },
    /// Dropped into the all-day strip on `date`: an all-day event of that day.
    AllDay { date: NaiveDate },
}

/// The minutes on the clock a drag of `minutes` moves by: whole steps, the nearest.
pub fn snap(minutes: f32) -> i64 {
    (minutes / STEP as f32).round() as i64 * i64::from(STEP)
}

/// The times of an event from `start` to `end` moved by `days` and `minutes` on the clock of
/// `zone`, keeping its length.
pub fn moved(
    (start, end): (DateTime<Utc>, DateTime<Utc>),
    days: i64,
    minutes: i64,
    zone: Tz,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let shift = Duration::days(days) + Duration::minutes(minutes);
    let start_at = at(local(start, zone) + shift, zone);
    (start_at, start_at + (end - start))
}

/// The end of an event from `start` to `end` whose end moved by `minutes` on the clock of `zone`,
/// a step after the start at least.
pub fn stretched(
    (start, end): (DateTime<Utc>, DateTime<Utc>),
    minutes: i64,
    zone: Tz,
) -> DateTime<Utc> {
    let end_at = at(local(end, zone) + Duration::minutes(minutes), zone);
    end_at.max(start + Duration::minutes(STEP.into()))
}

/// When `event` happens once dropped, with the clock of `zone`, the zone the views show;
/// `length` minutes long when an all-day event becomes timed. `None` when nothing changes.
pub fn dropped(event: &Event, drop: Target, zone: Tz, length: u32) -> Option<Timing> {
    let after = match (drop, event.timing) {
        // Dropped where it was: not even a time the clock shows twice is read again.
        (
            Target::Moved {
                days: 0,
                minutes: 0,
            }
            | Target::Resized { minutes: 0 },
            _,
        ) => return None,
        (
            Target::Moved { days, minutes },
            Timing::Timed {
                start,
                end,
                zone: own,
            },
        ) => {
            let (start, end) = moved((start, end), days, minutes, zone);
            Timing::Timed {
                start,
                end,
                zone: own,
            }
        }
        (
            Target::Moved { days, .. },
            Timing::AllDay {
                start,
                end_exclusive,
            },
        ) => Timing::AllDay {
            start: start + Duration::days(days),
            end_exclusive: end_exclusive + Duration::days(days),
        },
        (
            Target::Resized { minutes },
            Timing::Timed {
                start,
                end,
                zone: own,
            },
        ) => Timing::Timed {
            start,
            end: stretched((start, end), minutes, zone),
            zone: own,
        },
        (Target::Resized { .. }, Timing::AllDay { .. }) => return None,
        (Target::Timed { date, minute }, _) => {
            let (start, end) = clicked(date, minute, length, zone);
            Timing::Timed { start, end, zone }
        }
        // Out of office and focus time stay timed, as Google keeps them: midnight to midnight.
        (Target::AllDay { date }, _) if event.about.kind.timed_only() => Timing::Timed {
            start: wall(date, 0, zone),
            end: wall(date + Duration::days(1), 0, zone),
            zone,
        },
        (Target::AllDay { date }, _) => Timing::AllDay {
            start: date,
            end_exclusive: date + Duration::days(1),
        },
    };
    (after != event.timing).then_some(after)
}

/// The change sent to Google for an event of `calendar` dropped at `timing`: its times only,
/// against the version shown.
pub fn edit(event: &Event, calendar: Calendar, timing: Timing) -> Edit {
    Edit {
        event: event.clone(),
        calendar,
        patch: Patch {
            timing: Some(timing),
            ..Patch::default()
        },
        to: None,
        notify: false,
    }
}

pub(crate) fn local(at: DateTime<Utc>, zone: Tz) -> NaiveDateTime {
    at.with_timezone(&zone).naive_local()
}

/// The instant of a time on the clock of `zone`, to the second; one the zone skips moves to the
/// first that exists after it.
pub(crate) fn at(local: NaiveDateTime, zone: Tz) -> DateTime<Utc> {
    let minute = local.time().num_seconds_from_midnight() / 60;
    wall(local.date(), minute, zone) + Duration::seconds(local.second().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{all_day, d, timed, timed_in};
    use crate::model::Access;
    use crate::model::Rgb;
    use chrono::TimeZone;
    use chrono_tz::America::New_York;
    use chrono_tz::Europe::Warsaw;

    fn times(timing: Option<Timing>, zone: Tz) -> (String, String) {
        let Some(Timing::Timed { start, end, .. }) = timing else {
            panic!("not timed: {timing:?}");
        };
        let show = |at: DateTime<Utc>| at.with_timezone(&zone).format("%F %R").to_string();
        (show(start), show(end))
    }

    fn pair(start: &str, end: &str) -> (String, String) {
        (start.to_string(), end.to_string())
    }

    #[test]
    fn a_drag_snaps_to_the_nearest_15_minutes() {
        assert_eq!(snap(40.0), 45);
        assert_eq!(snap(37.0), 30);
        assert_eq!(snap(7.0), 0);
        assert_eq!(snap(-8.0), -15);
    }

    #[test]
    fn a_moved_event_keeps_its_length_and_zone() {
        let call = timed("Call", d(2026, 11, 12), (10, 0), (11, 0));
        let moved = dropped(
            &call,
            Target::Moved {
                days: 0,
                minutes: 45,
            },
            Warsaw,
            60,
        );
        assert_eq!(
            times(moved, Warsaw),
            pair("2026-11-12 10:45", "2026-11-12 11:45")
        );
        let moved = dropped(
            &call,
            Target::Moved {
                days: 1,
                minutes: -30,
            },
            Warsaw,
            60,
        );
        assert_eq!(
            times(moved, Warsaw),
            pair("2026-11-13 09:30", "2026-11-13 10:30")
        );
        let Some(Timing::Timed { zone, .. }) = moved else {
            panic!("timed");
        };
        assert_eq!(zone, Warsaw);
    }

    #[test]
    fn an_event_dropped_where_it_was_is_not_changed() {
        let call = timed("Call", d(2026, 11, 12), (10, 0), (11, 0));
        let still = Target::Moved {
            days: 0,
            minutes: 0,
        };
        assert_eq!(dropped(&call, still, Warsaw, 60), None);
        assert_eq!(
            dropped(&call, Target::Resized { minutes: 0 }, Warsaw, 60),
            None
        );
        let trip = all_day("Trip", d(2026, 11, 12), 1);
        let same_day = Target::AllDay {
            date: d(2026, 11, 12),
        };
        assert_eq!(dropped(&trip, same_day, Warsaw, 60), None);
    }

    #[test]
    fn a_move_to_the_next_day_across_a_daylight_saving_change_keeps_the_hour_on_the_clock() {
        // 25 October 2026: Warsaw's clocks go back from 3:00 to 2:00.
        let call = timed("Call", d(2026, 10, 24), (10, 0), (11, 0));
        let moved = dropped(
            &call,
            Target::Moved {
                days: 1,
                minutes: 0,
            },
            Warsaw,
            60,
        );
        assert_eq!(
            times(moved, Warsaw),
            pair("2026-10-25 10:00", "2026-10-25 11:00")
        );
    }

    #[test]
    fn an_event_in_the_hour_the_clock_shows_twice_dropped_where_it_was_is_not_changed() {
        // 25 October 2026, 1:30 UTC: the second 2:30 on Warsaw's clocks.
        let start = Utc.with_ymd_and_hms(2026, 10, 25, 1, 30, 0).unwrap();
        let mut call = timed("Call", d(2026, 10, 25), (5, 0), (6, 0));
        call.timing = Timing::Timed {
            start,
            end: start + Duration::hours(1),
            zone: Warsaw,
        };
        let still = Target::Moved {
            days: 0,
            minutes: 0,
        };
        assert_eq!(dropped(&call, still, Warsaw, 60), None);
        assert_eq!(
            dropped(&call, Target::Resized { minutes: 0 }, Warsaw, 60),
            None
        );
    }

    #[test]
    fn a_move_into_an_hour_the_clock_skips_lands_on_the_first_time_that_exists() {
        // 29 March 2026: Warsaw's clocks skip from 2:00 to 3:00.
        let call = timed("Call", d(2026, 3, 29), (0, 45), (1, 45));
        let moved = dropped(
            &call,
            Target::Moved {
                days: 0,
                minutes: 90,
            },
            Warsaw,
            60,
        );
        assert_eq!(
            times(moved, Warsaw),
            pair("2026-03-29 03:00", "2026-03-29 04:00")
        );
    }

    #[test]
    fn an_event_of_another_zone_moves_on_the_clock_shown_and_keeps_its_zone() {
        let call = timed_in("Call", d(2026, 11, 12), (9, 0), (10, 0), New_York);
        let moved = dropped(
            &call,
            Target::Moved {
                days: 0,
                minutes: 60,
            },
            Warsaw,
            60,
        );
        assert_eq!(
            times(moved, New_York),
            pair("2026-11-12 10:00", "2026-11-12 11:00")
        );
        let Some(Timing::Timed { zone, .. }) = moved else {
            panic!("timed");
        };
        assert_eq!(zone, New_York);
    }

    #[test]
    fn an_all_day_event_moves_by_whole_days() {
        let trip = all_day("Trip", d(2026, 11, 12), 3);
        let moved = dropped(
            &trip,
            Target::Moved {
                days: 2,
                minutes: 30,
            },
            Warsaw,
            60,
        );
        assert_eq!(
            moved,
            Some(Timing::AllDay {
                start: d(2026, 11, 14),
                end_exclusive: d(2026, 11, 17),
            })
        );
    }

    #[test]
    fn dragging_the_end_changes_the_length_and_keeps_a_step_at_least() {
        let call = timed("Call", d(2026, 11, 12), (10, 0), (11, 0));
        let longer = dropped(&call, Target::Resized { minutes: 30 }, Warsaw, 60);
        assert_eq!(
            times(longer, Warsaw),
            pair("2026-11-12 10:00", "2026-11-12 11:30")
        );
        let shortest = dropped(&call, Target::Resized { minutes: -90 }, Warsaw, 60);
        assert_eq!(
            times(shortest, Warsaw),
            pair("2026-11-12 10:00", "2026-11-12 10:15")
        );
        let trip = all_day("Trip", d(2026, 11, 12), 1);
        assert_eq!(
            dropped(&trip, Target::Resized { minutes: 30 }, Warsaw, 60),
            None
        );
    }

    #[test]
    fn an_all_day_event_dropped_into_the_hours_becomes_timed_of_the_default_length() {
        let trip = all_day("Trip", d(2026, 11, 12), 3);
        let drop = Target::Timed {
            date: d(2026, 11, 13),
            minute: 14 * 60 + 10,
        };
        let timed = dropped(&trip, drop, Warsaw, 30);
        assert_eq!(
            times(timed, Warsaw),
            pair("2026-11-13 14:00", "2026-11-13 14:30")
        );
        let Some(Timing::Timed { zone, .. }) = timed else {
            panic!("timed");
        };
        assert_eq!(zone, Warsaw);
    }

    #[test]
    fn a_timed_event_dropped_into_the_all_day_strip_becomes_all_day_that_day() {
        let call = timed("Call", d(2026, 11, 12), (10, 0), (11, 0));
        let drop = Target::AllDay {
            date: d(2026, 11, 14),
        };
        assert_eq!(
            dropped(&call, drop, Warsaw, 60),
            Some(Timing::AllDay {
                start: d(2026, 11, 14),
                end_exclusive: d(2026, 11, 15),
            })
        );
    }

    #[test]
    fn focus_time_dropped_into_the_all_day_strip_lasts_from_midnight_to_midnight() {
        let mut focus = timed("Focus", d(2026, 11, 12), (10, 0), (11, 0));
        focus.about.kind = crate::model::Kind::FocusTime;
        let drop = Target::AllDay {
            date: d(2026, 11, 14),
        };
        assert_eq!(
            times(dropped(&focus, drop, Warsaw, 60), Warsaw),
            pair("2026-11-14 00:00", "2026-11-15 00:00")
        );
    }

    #[test]
    fn the_change_sent_is_the_times_only_against_the_version_shown() {
        let call = timed("Call", d(2026, 11, 12), (10, 0), (11, 0));
        let calendar = Calendar {
            id: "work".to_string(),
            account_id: "1".to_string(),
            name: "Work".to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
            default_reminders: Vec::new(),
        };
        let timing = dropped(
            &call,
            Target::Moved {
                days: 0,
                minutes: 45,
            },
            Warsaw,
            60,
        )
        .unwrap();
        let edit = edit(&call, calendar, timing);
        assert_eq!(
            edit.patch,
            Patch {
                timing: Some(timing),
                ..Patch::default()
            }
        );
        assert_eq!(edit.event.etag, call.etag);
        assert_eq!(edit.to, None);
        assert_eq!(edit.changed().timing, timing);
    }
}
