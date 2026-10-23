//! A calendar's synchronized events as the views show them: series expanded into their
//! occurrences inside the cache window by the `rrule` library, changed occurrences in place of the
//! ones they replace, cancelled ones left out (`MOD-02-F22`).

use std::collections::HashSet;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use rrule::{RRuleSet, Tz as RuleZone};

use crate::model::{Access, Calendar, Event, Timing};
use crate::sync::{Start, SyncedEvent, Window};

/// Every event and occurrence of the calendar that falls inside the window, in the calendar's
/// colour.
pub fn expand(calendar: &Calendar, events: &[SyncedEvent], window: Window) -> Vec<Event> {
    let replaced: HashSet<(&str, Start)> = events
        .iter()
        .filter_map(|event| event.instance_of.as_ref())
        .map(|(series, original)| (series.as_str(), *original))
        .collect();
    let shown = |id: String, title: &str, timing: Timing| Event {
        id,
        calendar_id: calendar.id.clone(),
        // As Google Calendar names an untitled event.
        title: if title.is_empty() {
            "(No title)"
        } else {
            title
        }
        .to_string(),
        color: calendar.color,
        timing,
        busy_only: calendar.access == Access::FreeBusyReader,
        read_only: !calendar.access.can_edit(),
    };
    let mut shown_events = Vec::new();
    for event in events {
        // A cancelled occurrence only takes its place away.
        let Some(timing) = event.timing else {
            continue;
        };
        if event.recurrence.is_empty() || event.instance_of.is_some() {
            if window.overlaps(&timing) {
                shown_events.push(shown(event.id.clone(), &event.title, timing));
            }
            continue;
        }
        for (start, timing) in occurrences(timing, &event.recurrence, window) {
            if !replaced.contains(&(event.id.as_str(), start)) {
                shown_events.push(shown(occurrence_id(&event.id, start), &event.title, timing));
            }
        }
    }
    shown_events
}

/// A series' occurrences that fall inside the window, each lasting as long as the first. A rule
/// the library does not accept leaves the first occurrence alone.
fn occurrences(first: Timing, rules: &[String], window: Window) -> Vec<(Start, Timing)> {
    let starts = match first {
        Timing::Timed { start, end, zone } => {
            let first_zone = zone;
            let zone = RuleZone::Tz(zone);
            let dt_start = format!(
                "DTSTART;TZID={}:{}",
                zone.name(),
                start.with_timezone(&zone).format("%Y%m%dT%H%M%S")
            );
            let after = (window.start() - (end - start)).with_timezone(&zone);
            let before = window.end().with_timezone(&zone);
            let rules: Vec<String> = rules
                .iter()
                .map(|rule| with_until(rule, |until| until_in_utc(until, first_zone)))
                .collect();
            starts(&dt_start, &rules, after, before)
        }
        // Dates carry no zone; the library reads them in the system's, both the rule's and ours.
        Timing::AllDay {
            start,
            end_exclusive,
        } => {
            let noon = |date: NaiveDate| {
                RuleZone::LOCAL
                    .from_local_datetime(&date.and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()))
                    .earliest()
            };
            let dt_start = format!("DTSTART;VALUE=DATE:{}", start.format("%Y%m%d"));
            let after = noon(window.from - (end_exclusive - start) - Duration::days(1));
            let before = noon(window.until + Duration::days(1));
            let rules: Vec<String> = rules
                .iter()
                .map(|rule| with_until(rule, until_as_date))
                .collect();
            match (after, before) {
                (Some(after), Some(before)) => starts(&dt_start, &rules, after, before),
                _ => None,
            }
        }
    };
    let Some(starts) = starts else {
        return if window.overlaps(&first) {
            vec![(start_of(first), first)]
        } else {
            Vec::new()
        };
    };
    starts
        .into_iter()
        .map(|at| match first {
            Timing::Timed { start, end, zone } => {
                let at = at.with_timezone(&Utc);
                let timing = Timing::Timed {
                    start: at,
                    end: at + (end - start),
                    zone,
                };
                (Start::At(at), timing)
            }
            Timing::AllDay {
                start,
                end_exclusive,
            } => {
                let date = at.date_naive();
                let timing = Timing::AllDay {
                    start: date,
                    end_exclusive: date + (end_exclusive - start),
                };
                (Start::On(date), timing)
            }
        })
        .filter(|(_, timing)| window.overlaps(timing))
        .collect()
}

/// The starts the rule set gives between the bounds; `None` when the library rejects it.
fn starts(
    dt_start: &str,
    rules: &[String],
    after: DateTime<RuleZone>,
    before: DateTime<RuleZone>,
) -> Option<Vec<DateTime<RuleZone>>> {
    let text = std::iter::once(dt_start)
        .chain(rules.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    let set: RRuleSet = text.parse().ok()?;
    Some(set.after(after).before(before).all(u16::MAX).dates)
}

/// The rule with its end (`UNTIL`) rewritten by `rewrite`; other lines and parts stay as they are.
fn with_until(rule: &str, rewrite: impl Fn(&str) -> Option<String>) -> String {
    let Some(parts) = rule.strip_prefix("RRULE:") else {
        return rule.to_string();
    };
    let parts: Vec<String> = parts
        .split(';')
        .map(
            |part| match part.strip_prefix("UNTIL=").and_then(&rewrite) {
                Some(until) => format!("UNTIL={until}"),
                None => part.to_string(),
            },
        )
        .collect();
    format!("RRULE:{}", parts.join(";"))
}

/// The library takes a timed series' `UNTIL` only in UTC; Google also keeps a date (the series
/// ends on that day) or a local time, both read in the series' zone.
fn until_in_utc(until: &str, zone: Tz) -> Option<String> {
    if until.ends_with('Z') {
        return None;
    }
    let local = match NaiveDate::parse_from_str(until, "%Y%m%d") {
        Ok(date) => date.and_hms_opt(23, 59, 59)?,
        Err(_) => NaiveDateTime::parse_from_str(until, "%Y%m%dT%H%M%S").ok()?,
    };
    let at = zone.from_local_datetime(&local).latest()?;
    Some(at.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ").to_string())
}

/// An all-day series ends on a date, as its start is one; a time given with it only names the
/// day.
fn until_as_date(until: &str) -> Option<String> {
    (until.len() > 8).then(|| until[..8].to_string())
}

fn start_of(timing: Timing) -> Start {
    match timing {
        Timing::Timed { start, .. } => Start::At(start),
        Timing::AllDay { start, .. } => Start::On(start),
    }
}

/// Google's ID of an occurrence: the series' ID and the occurrence's start, in UTC or as a date.
fn occurrence_id(series: &str, start: Start) -> String {
    match start {
        Start::At(at) => format!("{series}_{}", at.format("%Y%m%dT%H%M%SZ")),
        Start::On(date) => format!("{series}_{}", date.format("%Y%m%d")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;
    use crate::model::{Access, Rgb, Timing};
    use crate::sync::Start;
    use chrono::{DateTime, NaiveDate, TimeZone, Utc};
    use chrono_tz::Europe::{London, Warsaw};
    use chrono_tz::Tz;

    fn calendar() -> Calendar {
        Calendar {
            id: "work@example.com".into(),
            account_id: "1".into(),
            name: "Work".into(),
            color: Rgb(0x03, 0x9B, 0xE5),
            access: Access::Owner,
            zone: Warsaw,
        }
    }

    fn window() -> Window {
        Window::around(d(2026, 11, 11), Warsaw)
    }

    fn at(zone: Tz, date: NaiveDate, (h, m): (u32, u32)) -> DateTime<Utc> {
        zone.from_local_datetime(&date.and_hms_opt(h, m, 0).unwrap())
            .unwrap()
            .with_timezone(&Utc)
    }

    fn timed(zone: Tz, date: NaiveDate, start: (u32, u32), end: (u32, u32)) -> Timing {
        Timing::Timed {
            start: at(zone, date, start),
            end: at(zone, date, end),
            zone,
        }
    }

    fn event(id: &str, timing: Timing) -> SyncedEvent {
        SyncedEvent {
            id: id.into(),
            title: id.into(),
            timing: Some(timing),
            recurrence: Vec::new(),
            instance_of: None,
        }
    }

    fn series(id: &str, timing: Timing, rules: &[&str]) -> SyncedEvent {
        SyncedEvent {
            recurrence: rules.iter().map(|r| r.to_string()).collect(),
            ..event(id, timing)
        }
    }

    /// Local start times, in Warsaw, of what is shown.
    fn local_starts(events: &[Event]) -> Vec<String> {
        let mut starts: Vec<String> = events
            .iter()
            .map(|e| match e.timing {
                Timing::Timed { start, .. } => {
                    start.with_timezone(&Warsaw).format("%F %R").to_string()
                }
                Timing::AllDay { start, .. } => start.to_string(),
            })
            .collect();
        starts.sort();
        starts
    }

    /// `TC-02-02`'s series: weekly on Monday 09:00, five occurrences, the third moved to 11:00,
    /// the fourth cancelled.
    #[test]
    fn tc_02_02_a_moved_occurrence_shows_at_its_new_time_and_a_cancelled_one_not_at_all() {
        let first = d(2026, 11, 2);
        let master = series(
            "weekly",
            timed(Warsaw, first, (9, 0), (10, 0)),
            &["RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=5"],
        );
        let moved = SyncedEvent {
            instance_of: Some((
                "weekly".into(),
                Start::At(at(Warsaw, d(2026, 11, 16), (9, 0))),
            )),
            ..event(
                "weekly_20261116T080000Z",
                timed(Warsaw, d(2026, 11, 16), (11, 0), (12, 0)),
            )
        };
        let cancelled = SyncedEvent {
            timing: None,
            instance_of: Some((
                "weekly".into(),
                Start::At(at(Warsaw, d(2026, 11, 23), (9, 0))),
            )),
            ..event(
                "weekly_20261123T080000Z",
                timed(Warsaw, first, (9, 0), (10, 0)),
            )
        };

        let shown = expand(&calendar(), &[master, moved, cancelled], window());

        assert_eq!(
            local_starts(&shown),
            [
                "2026-11-02 09:00",
                "2026-11-09 09:00",
                "2026-11-16 11:00",
                "2026-11-30 09:00"
            ]
        );
    }

    #[test]
    fn tc_02_02_an_event_stored_in_another_zone_keeps_its_instant() {
        let london = event("london", timed(London, d(2027, 6, 15), (15, 0), (16, 0)));

        let shown = expand(&calendar(), &[london], window());

        assert_eq!(local_starts(&shown), ["2027-06-15 16:00"]);
        assert_eq!(
            shown[0].timing,
            timed(London, d(2027, 6, 15), (15, 0), (16, 0))
        );
    }

    #[test]
    fn occurrences_keep_their_local_time_across_a_daylight_saving_change() {
        // Europe/Warsaw leaves summer time on 25 October 2026.
        let master = series(
            "weekly",
            timed(Warsaw, d(2026, 10, 19), (9, 0), (10, 0)),
            &["RRULE:FREQ=WEEKLY;COUNT=2"],
        );

        let shown = expand(&calendar(), &[master], window());

        assert_eq!(
            local_starts(&shown),
            ["2026-10-19 09:00", "2026-10-26 09:00"]
        );
        let Timing::Timed { start, end, .. } = shown[1].timing else {
            panic!("a timed occurrence");
        };
        assert_eq!(end - start, chrono::Duration::hours(1));
    }

    #[test]
    fn a_series_started_before_the_window_has_its_occurrences_inside_it() {
        let master = series(
            "daily",
            timed(Warsaw, d(2020, 1, 1), (8, 0), (8, 30)),
            &["RRULE:FREQ=DAILY"],
        );

        let shown = expand(&calendar(), &[master], window());

        let starts = local_starts(&shown);
        assert_eq!(starts.first().unwrap(), "2025-11-11 08:00");
        assert_eq!(starts.last().unwrap(), "2028-11-10 08:00");
        assert_eq!(shown.len(), 1096);
    }

    #[test]
    fn excluded_dates_and_extra_dates_of_a_series_are_honoured() {
        let master = series(
            "weekly",
            timed(Warsaw, d(2026, 11, 2), (9, 0), (10, 0)),
            &[
                "RRULE:FREQ=WEEKLY;UNTIL=20261116T225959Z",
                "EXDATE;TZID=Europe/Warsaw:20261109T090000",
                "RDATE;TZID=Europe/Warsaw:20261111T150000",
            ],
        );

        let shown = expand(&calendar(), &[master], window());

        assert_eq!(
            local_starts(&shown),
            ["2026-11-02 09:00", "2026-11-11 15:00", "2026-11-16 09:00"]
        );
    }

    #[test]
    fn a_yearly_all_day_series_repeats_on_its_date() {
        let master = series(
            "birthday",
            Timing::AllDay {
                start: d(2000, 3, 14),
                end_exclusive: d(2000, 3, 15),
            },
            &["RRULE:FREQ=YEARLY", "EXDATE;VALUE=DATE:20270314"],
        );

        let shown = expand(&calendar(), &[master], window());

        assert_eq!(local_starts(&shown), ["2026-03-14", "2028-03-14"]);
        assert_eq!(
            shown[0].timing,
            Timing::AllDay {
                start: d(2026, 3, 14),
                end_exclusive: d(2026, 3, 15)
            }
        );
        assert_eq!(shown[0].id, "birthday_20260314");
    }

    #[test]
    fn occurrences_are_named_like_google_names_them_and_take_the_calendar_colour() {
        let master = series(
            "weekly",
            timed(Warsaw, d(2026, 11, 2), (9, 0), (10, 0)),
            &["RRULE:FREQ=WEEKLY;COUNT=1"],
        );

        let shown = expand(&calendar(), &[master], window());

        assert_eq!(shown[0].id, "weekly_20261102T080000Z");
        assert_eq!(shown[0].calendar_id, "work@example.com");
        assert_eq!(shown[0].color, Rgb(0x03, 0x9B, 0xE5));
        assert_eq!(shown[0].title, "weekly");
    }

    #[test]
    fn an_occurrence_without_its_series_is_shown_on_its_own() {
        // Invited to one occurrence of someone else's series: Google sends only that occurrence.
        let lone = SyncedEvent {
            instance_of: Some((
                "theirs".into(),
                Start::At(at(Warsaw, d(2026, 11, 3), (9, 0))),
            )),
            ..event(
                "theirs_20261103T080000Z",
                timed(Warsaw, d(2026, 11, 3), (10, 0), (11, 0)),
            )
        };

        let shown = expand(&calendar(), &[lone], window());

        assert_eq!(local_starts(&shown), ["2026-11-03 10:00"]);
    }

    #[test]
    fn a_series_with_a_rule_the_library_rejects_shows_its_first_occurrence() {
        let master = series(
            "odd",
            timed(Warsaw, d(2026, 11, 2), (9, 0), (10, 0)),
            &["RRULE:FREQ=SOMETIMES"],
        );

        let shown = expand(&calendar(), &[master], window());

        assert_eq!(local_starts(&shown), ["2026-11-02 09:00"]);
    }

    #[test]
    fn events_of_a_free_busy_calendar_are_busy_only() {
        let free_busy = Calendar {
            access: Access::FreeBusyReader,
            ..calendar()
        };
        let busy = event("busy", timed(Warsaw, d(2026, 11, 3), (10, 0), (11, 0)));

        let shown = expand(&free_busy, &[busy], window());

        assert!(shown[0].busy_only);
    }

    #[test]
    fn events_of_calendars_the_account_cannot_change_are_read_only() {
        let event = event("one", timed(Warsaw, d(2026, 11, 3), (10, 0), (11, 0)));
        let read_only = |access| {
            let calendar = Calendar {
                access,
                ..calendar()
            };
            expand(&calendar, std::slice::from_ref(&event), window())[0].read_only
        };

        assert!(!read_only(Access::Owner));
        assert!(!read_only(Access::Writer));
        assert!(read_only(Access::Reader));
        assert!(read_only(Access::FreeBusyReader));
    }

    #[test]
    fn an_untitled_event_is_named_as_google_names_it() {
        let mut untitled = event("untitled", timed(Warsaw, d(2026, 11, 3), (10, 0), (11, 0)));
        untitled.title = String::new();

        let shown = expand(&calendar(), &[untitled], window());

        assert_eq!(shown[0].title, "(No title)");
    }

    #[test]
    fn a_timed_series_ending_on_a_date_or_a_local_time_ends_there() {
        for until in ["UNTIL=20261116", "UNTIL=20261116T090000"] {
            let master = series(
                "weekly",
                timed(Warsaw, d(2026, 11, 2), (9, 0), (10, 0)),
                &[&format!("RRULE:FREQ=WEEKLY;{until};BYDAY=MO")],
            );

            let shown = expand(&calendar(), &[master], window());

            assert_eq!(
                local_starts(&shown),
                ["2026-11-02 09:00", "2026-11-09 09:00", "2026-11-16 09:00"],
                "{until}"
            );
        }
    }

    #[test]
    fn an_all_day_series_ending_on_a_date_ends_there() {
        for until in ["UNTIL=20261104", "UNTIL=20261104T000000Z"] {
            let master = series(
                "daily",
                Timing::AllDay {
                    start: d(2026, 11, 2),
                    end_exclusive: d(2026, 11, 3),
                },
                &[&format!("RRULE:FREQ=DAILY;{until}")],
            );

            let shown = expand(&calendar(), &[master], window());

            assert_eq!(
                local_starts(&shown),
                ["2026-11-02", "2026-11-03", "2026-11-04"],
                "{until}"
            );
        }
    }
}
