//! Making a new event repeat (`US-05-06`, `DEC-36`): Google Calendar's everyday presets — daily,
//! every weekday, weekly on chosen days, monthly, yearly (`MOD-05-F18`) — and how the series ends:
//! on a date, after a number of occurrences, or never (`MOD-05-F19`), as the rules Google keeps.

use chrono::{NaiveDate, Weekday};

use crate::model::Timing;
use crate::recurrence::until_in_utc;

/// How often a series repeats.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pattern {
    Daily,
    /// Monday to Friday.
    Weekdays,
    /// On the chosen days of each week.
    Weekly(Vec<Weekday>),
    /// On the same day of each month.
    Monthly,
    /// On the same date each year.
    Yearly,
}

/// How a series ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ends {
    Never,
    /// With the last occurrence on or before this day.
    On(NaiveDate),
    /// After this many occurrences.
    After(u32),
}

/// The rule (`RRULE`) of a series repeating with `pattern` from `first`, its first occurrence,
/// until `ends`.
pub fn rule(pattern: &Pattern, ends: Ends, first: &Timing) -> String {
    let mut parts = match pattern {
        Pattern::Daily => "FREQ=DAILY".to_string(),
        Pattern::Weekdays => "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR".to_string(),
        Pattern::Weekly(days) => {
            let mut days = days.clone();
            days.sort_by_key(|day| day.num_days_from_monday());
            days.dedup();
            let days: Vec<&str> = days.into_iter().map(day_code).collect();
            format!("FREQ=WEEKLY;BYDAY={}", days.join(","))
        }
        Pattern::Monthly => "FREQ=MONTHLY".to_string(),
        Pattern::Yearly => "FREQ=YEARLY".to_string(),
    };
    match ends {
        Ends::Never => {}
        Ends::After(count) => parts.push_str(&format!(";COUNT={count}")),
        Ends::On(last) => parts.push_str(&format!(";UNTIL={}", until(last, first))),
    }
    format!("RRULE:{parts}")
}

/// The end of the last day of a series, as Google Calendar writes it: for a timed series the
/// last second of that day in the series' zone, in UTC; for an all-day one the date.
fn until(last: NaiveDate, first: &Timing) -> String {
    match first {
        Timing::Timed { zone, .. } => until_in_utc(&last.format("%Y%m%d").to_string(), *zone)
            .expect("no zone skips the last second of a day"),
        Timing::AllDay { .. } => last.format("%Y%m%d").to_string(),
    }
}

fn day_code(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "MO",
        Weekday::Tue => "TU",
        Weekday::Wed => "WE",
        Weekday::Thu => "TH",
        Weekday::Fri => "FR",
        Weekday::Sat => "SA",
        Weekday::Sun => "SU",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{all_day, d, timed, timed_in};
    use chrono_tz::America::New_York;

    fn training() -> Timing {
        timed("Training", d(2026, 11, 16), (18, 0), (19, 0)).timing
    }

    #[test]
    fn tc_05_07_each_preset_becomes_the_rule_google_keeps() {
        let never = |pattern| rule(&pattern, Ends::Never, &training());
        assert_eq!(never(Pattern::Daily), "RRULE:FREQ=DAILY");
        assert_eq!(
            never(Pattern::Weekdays),
            "RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"
        );
        assert_eq!(
            never(Pattern::Weekly(vec![Weekday::Mon])),
            "RRULE:FREQ=WEEKLY;BYDAY=MO"
        );
        assert_eq!(never(Pattern::Monthly), "RRULE:FREQ=MONTHLY");
        assert_eq!(never(Pattern::Yearly), "RRULE:FREQ=YEARLY");
    }

    #[test]
    fn tc_05_07_weekly_on_monday_and_wednesday_ending_after_10_occurrences() {
        let days = Pattern::Weekly(vec![Weekday::Mon, Weekday::Wed]);
        assert_eq!(
            rule(&days, Ends::After(10), &training()),
            "RRULE:FREQ=WEEKLY;BYDAY=MO,WE;COUNT=10"
        );
    }

    #[test]
    fn the_chosen_days_go_in_the_order_of_the_week_once_each() {
        let days = Pattern::Weekly(vec![Weekday::Sun, Weekday::Wed, Weekday::Mon, Weekday::Wed]);
        assert_eq!(
            rule(&days, Ends::Never, &training()),
            "RRULE:FREQ=WEEKLY;BYDAY=MO,WE,SU"
        );
    }

    #[test]
    fn a_timed_series_ending_on_a_date_ends_with_that_day_in_its_own_zone() {
        // Warsaw is an hour ahead of UTC in December.
        assert_eq!(
            rule(&Pattern::Daily, Ends::On(d(2026, 12, 16)), &training()),
            "RRULE:FREQ=DAILY;UNTIL=20261216T225959Z"
        );
        let call = timed_in("Call", d(2026, 11, 16), (9, 0), (10, 0), New_York).timing;
        assert_eq!(
            rule(&Pattern::Daily, Ends::On(d(2026, 7, 1)), &call),
            "RRULE:FREQ=DAILY;UNTIL=20260702T035959Z"
        );
    }

    #[test]
    fn an_all_day_series_ending_on_a_date_ends_on_that_date() {
        let trip = all_day("Trip", d(2026, 11, 16), 1).timing;
        assert_eq!(
            rule(&Pattern::Yearly, Ends::On(d(2030, 11, 16)), &trip),
            "RRULE:FREQ=YEARLY;UNTIL=20301116"
        );
        assert_eq!(
            rule(&Pattern::Monthly, Ends::After(3), &trip),
            "RRULE:FREQ=MONTHLY;COUNT=3"
        );
    }
}
