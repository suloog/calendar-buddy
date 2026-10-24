//! How an event repeats (`US-05-06`, `US-05-11`, `DEC-165`): Google Calendar's repeat menu, its
//! presets following the event's start (`MOD-05-F32`), and its custom recurrence dialog — every N
//! days, weeks on the chosen weekdays, months on the same day or the same weekday of the month, or
//! years, ending never, on a date or after a number of occurrences (`MOD-05-F33`, `MOD-05-U05`). A
//! rule Google keeps is read into the dialog's fields when the dialog can express it, written back
//! and put in words as Google Calendar puts it (`MOD-05-F34`); any other is "complex" and kept
//! untouched (`MOD-05-F29`).

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Weekday};

use crate::model::Timing;
use crate::recurrence::until_in_utc;
use crate::regional::Regional;
use crate::texts;

/// The unit a series repeats in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Frequency {
    pub const EACH: [Frequency; 4] = [
        Frequency::Daily,
        Frequency::Weekly,
        Frequency::Monthly,
        Frequency::Yearly,
    ];

    /// What the custom dialog calls the unit, as Google Calendar does.
    pub fn label(self) -> &'static str {
        match self {
            Frequency::Daily => texts::DAY_UNIT,
            Frequency::Weekly => texts::WEEK_UNIT,
            Frequency::Monthly => texts::MONTH_UNIT,
            Frequency::Yearly => texts::YEAR_UNIT,
        }
    }
}

/// Which day of the month a monthly series repeats on, as its first occurrence falls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonthDay {
    /// The same day of the month (day 17).
    Day,
    /// The same weekday in the same week of the month (the third Tuesday); for the fifth one of
    /// the month, the last (`DEC-167`).
    Weekday,
    /// The last such weekday of the month, for a day in its last seven days.
    Last,
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

/// How a series ends, as chosen in the custom dialog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ending {
    #[default]
    Never,
    OnDate,
    AfterCount,
}

/// How a series repeats, as far as Google Calendar's custom recurrence dialog can say it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub frequency: Frequency,
    /// Every how many days, weeks, months or years.
    pub interval: u32,
    /// For a weekly series, the days of the week; empty otherwise.
    pub days: Vec<Weekday>,
    /// For a monthly series, which day of the month.
    pub month_day: MonthDay,
    pub ends: Ends,
    /// The first day of the week of a rule written so in Google (`WKST`), kept as it came.
    week_start: Option<Weekday>,
}

impl Rule {
    fn every(frequency: Frequency) -> Self {
        Rule {
            frequency,
            interval: 1,
            days: Vec::new(),
            month_day: MonthDay::Day,
            ends: Ends::Never,
            week_start: None,
        }
    }

    fn weekly(days: Vec<Weekday>) -> Self {
        Rule {
            days,
            ..Rule::every(Frequency::Weekly)
        }
    }
}

/// Monday to Friday.
const WEEKDAYS: [Weekday; 5] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
];

/// The presets of Google Calendar's repeat menu for an event starting on `start`, in its order:
/// daily, weekly on its weekday, monthly on its weekday of the month, annually on its date, and
/// every weekday (`MOD-05-F32`).
pub fn presets(start: NaiveDate) -> [Rule; 5] {
    [
        Rule::every(Frequency::Daily),
        Rule::weekly(vec![start.weekday()]),
        Rule {
            month_day: MonthDay::Weekday,
            ..Rule::every(Frequency::Monthly)
        },
        Rule::every(Frequency::Yearly),
        Rule::weekly(WEEKDAYS.to_vec()),
    ]
}

/// The day a series' first occurrence falls on, in its own zone.
pub fn first_day(first: &Timing) -> NaiveDate {
    match *first {
        Timing::Timed { start, zone, .. } => start.with_timezone(&zone).date_naive(),
        Timing::AllDay { start, .. } => start,
    }
}

/// Which occurrence of its weekday in the month the day is: 1 for the first, 5 for the fifth.
fn week_of_month(day: NaiveDate) -> u32 {
    (day.day() - 1) / 7 + 1
}

/// Whether the day falls in the last seven days of its month, its weekday's last in the month.
fn in_last_week(day: NaiveDate) -> bool {
    (day + Duration::days(7)).month() != day.month()
}

/// The week of the month a monthly series on a weekday repeats in, from `start`: 1 to 4, or -1
/// for the last.
fn ordinal(month_day: MonthDay, start: NaiveDate) -> i32 {
    let week = week_of_month(start);
    match month_day {
        MonthDay::Weekday if week < 5 => week as i32,
        MonthDay::Last if in_last_week(start) => -1,
        MonthDay::Last => week as i32,
        _ => -1,
    }
}

/// Whether the rule falls on the same days started from either of two days: on the same day of
/// the month, week of the month or date of the year where it repeats so.
pub fn same_days(rule: &Rule, (a, b): (NaiveDate, NaiveDate)) -> bool {
    match (rule.frequency, rule.month_day) {
        (Frequency::Daily | Frequency::Weekly, _) => true,
        (Frequency::Monthly, MonthDay::Day) => a.day() == b.day(),
        (Frequency::Monthly, month_day) => {
            a.weekday() == b.weekday() && ordinal(month_day, a) == ordinal(month_day, b)
        }
        (Frequency::Yearly, _) => (a.month(), a.day()) == (b.month(), b.day()),
    }
}

/// The rule (`RRULE`) of a series repeating as `rule` says from `first`, its first occurrence, as
/// Google Calendar writes it.
pub fn rule(rule: &Rule, first: &Timing) -> String {
    let start = first_day(first);
    let mut parts = vec![format!(
        "FREQ={}",
        match rule.frequency {
            Frequency::Daily => "DAILY",
            Frequency::Weekly => "WEEKLY",
            Frequency::Monthly => "MONTHLY",
            Frequency::Yearly => "YEARLY",
        }
    )];
    if let Some(day) = rule.week_start {
        parts.push(format!("WKST={}", day_code(day)));
    }
    if rule.interval > 1 {
        parts.push(format!("INTERVAL={}", rule.interval));
    }
    match rule.frequency {
        Frequency::Weekly => {
            let mut days = rule.days.clone();
            days.sort_by_key(|day| day.num_days_from_monday());
            days.dedup();
            let days: Vec<&str> = days.into_iter().map(day_code).collect();
            parts.push(format!("BYDAY={}", days.join(",")));
        }
        Frequency::Monthly if rule.month_day != MonthDay::Day => {
            let ordinal = ordinal(rule.month_day, start);
            parts.push(format!("BYDAY={ordinal}{}", day_code(start.weekday())));
        }
        _ => {}
    }
    match rule.ends {
        Ends::Never => {}
        Ends::After(count) => parts.push(format!("COUNT={count}")),
        Ends::On(last) => parts.push(format!("UNTIL={}", until(last, first))),
    }
    format!("RRULE:{}", parts.join(";"))
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

/// The rule of a series from `first`, its first occurrence, read from its rule lines when the
/// custom dialog can express it; `None` for a rule it cannot, which is kept as it is
/// (MOD-05 *Technical notes*). `EXDATE` lines go with any rule.
pub fn parse(lines: &[String], first: &Timing) -> Option<Rule> {
    let mut rules = lines.iter().filter(|line| !line.starts_with("EXDATE"));
    let (Some(line), None) = (rules.next(), rules.next()) else {
        return None;
    };
    let start = first_day(first);
    let mut frequency = None;
    let mut rule = Rule::every(Frequency::Daily);
    let (mut by_day, mut month_day, mut month) = (None, None, None);
    for part in line.strip_prefix("RRULE:")?.split(';') {
        let (name, value) = part.split_once('=')?;
        match name {
            "FREQ" => {
                frequency = Some(match value {
                    "DAILY" => Frequency::Daily,
                    "WEEKLY" => Frequency::Weekly,
                    "MONTHLY" => Frequency::Monthly,
                    "YEARLY" => Frequency::Yearly,
                    _ => return None,
                })
            }
            "INTERVAL" => rule.interval = value.parse().ok().filter(|n| *n > 0)?,
            "COUNT" if rule.ends == Ends::Never => {
                rule.ends = Ends::After(value.parse().ok().filter(|n| *n > 0)?)
            }
            "UNTIL" if rule.ends == Ends::Never => rule.ends = Ends::On(last_day(value, first)?),
            "WKST" => rule.week_start = Some(day_named(value)?),
            "BYDAY" => by_day = Some(value),
            "BYMONTHDAY" => month_day = Some(value.parse::<u32>().ok()?),
            "BYMONTH" => month = Some(value.parse::<u32>().ok()?),
            _ => return None,
        }
    }
    rule.frequency = frequency?;
    match (rule.frequency, by_day, month_day, month) {
        (Frequency::Daily, None, None, None) => {}
        (Frequency::Weekly, days, None, None) => {
            rule.days = match days {
                Some(days) => days.split(',').map(day_named).collect::<Option<_>>()?,
                None => vec![start.weekday()],
            };
        }
        (Frequency::Monthly, None, day, None) if day.is_none_or(|day| day == start.day()) => {}
        (Frequency::Monthly, Some(day), None, None) => {
            let (ordinal, day) = day.split_at(day.len().checked_sub(2)?);
            if day_named(day)? != start.weekday() {
                return None;
            }
            rule.month_day = match ordinal.parse::<i32>().ok()? {
                -1 if in_last_week(start) => MonthDay::Last,
                n if n == week_of_month(start) as i32 && n < 5 => MonthDay::Weekday,
                _ => return None,
            };
        }
        (Frequency::Yearly, None, None, None) => {}
        (Frequency::Yearly, None, Some(day), Some(month))
            if day == start.day() && month == start.month() => {}
        _ => return None,
    }
    Some(rule)
}

/// The last day a series ending at `until` can have an occurrence on: for a timed series, the
/// last whose start in the series' zone is not after it.
fn last_day(until: &str, first: &Timing) -> Option<NaiveDate> {
    let date = |text: &str| NaiveDate::parse_from_str(text.get(..8)?, "%Y%m%d").ok();
    match *first {
        Timing::AllDay { .. } => date(until),
        Timing::Timed { start, zone, .. } => {
            let utc = match until.ends_with('Z') {
                true => until.to_string(),
                false => until_in_utc(until, zone)?,
            };
            let until = NaiveDateTime::parse_from_str(&utc, "%Y%m%dT%H%M%SZ").ok()?;
            let until = until.and_utc().with_timezone(&zone).naive_local();
            let starts = start.with_timezone(&zone).time();
            Some(match until.time() < starts {
                true => until.date() - Duration::days(1),
                false => until.date(),
            })
        }
    }
}

/// The rule in words, as Google Calendar's repeat menu puts it: "Every 2 weeks on Monday,
/// Wednesday, 10 times" (`MOD-05-F34`); `start` is the day of its first occurrence.
pub fn describe(rule: &Rule, start: NaiveDate, regional: &Regional) -> String {
    let n = rule.interval;
    let every = |one: &str, unit: &str| match n {
        1 => one.to_string(),
        n => format!("Every {n} {unit}"),
    };
    let mut days = rule.days.clone();
    days.sort_by_key(|day| day.num_days_from_monday());
    days.dedup();
    let mut words = match rule.frequency {
        Frequency::Daily => every(texts::DAILY, "days"),
        Frequency::Weekly if n == 1 && days == WEEKDAYS => texts::EVERY_WEEKDAY.to_string(),
        Frequency::Weekly => {
            let days: Vec<&str> = days.into_iter().map(day_name).collect();
            format!("{} on {}", every(texts::WEEKLY, "weeks"), days.join(", "))
        }
        Frequency::Monthly => format!(
            "{} {}",
            every(texts::MONTHLY, "months"),
            on_month_day(rule.month_day, start)
        ),
        Frequency::Yearly => format!(
            "{} on {}",
            every(texts::ANNUALLY, "years"),
            regional.day_month(start)
        ),
    };
    match rule.ends {
        Ends::Never => {}
        Ends::After(1) => words.push_str(", once"),
        Ends::After(count) => words.push_str(&format!(", {count} times")),
        Ends::On(last) => words.push_str(&format!(", until {}", regional.day_month_year(last))),
    }
    words
}

/// "on day 17", "on the third Tuesday", "on the last Tuesday".
fn on_month_day(month_day: MonthDay, start: NaiveDate) -> String {
    let weekday = day_name(start.weekday());
    match (month_day, ordinal(month_day, start)) {
        (MonthDay::Day, _) => format!("on day {}", start.day()),
        (_, -1) => format!("on the last {weekday}"),
        (_, n) => {
            let nth = ["first", "second", "third", "fourth"][n as usize - 1];
            format!("on the {nth} {weekday}")
        }
    }
}

/// The custom recurrence dialog's fields, numbers and dates as typed (`MOD-05-F33`).
#[derive(Clone, Debug, PartialEq)]
pub struct Custom {
    /// The day the event starts, which the weekday- and day-based choices follow.
    pub start: NaiveDate,
    /// Every how many units, as typed.
    pub interval: String,
    frequency: Frequency,
    days: Vec<Weekday>,
    pub month_day: MonthDay,
    pub ends: Ending,
    /// The last day of a series ending on a date, as typed.
    pub last_date: String,
    /// The number of occurrences of a series ending after them, as typed.
    pub count: String,
    /// Why the dialog was not closed with Done.
    pub problem: Option<String>,
    week_start: Option<Weekday>,
}

/// How many occurrences a series ending after a number of them has until another is typed:
/// Google Calendar's default (`DEC-112`).
const DEFAULT_COUNT: u32 = 13;

impl Custom {
    /// The dialog opened on the rule chosen, for an event starting on `start`; on weekly on the
    /// start's weekday when the event does not repeat, as Google Calendar opens it.
    pub fn of(rule: Option<&Rule>, start: NaiveDate, regional: &Regional) -> Self {
        let rule = rule
            .cloned()
            .unwrap_or_else(|| Rule::weekly(vec![start.weekday()]));
        let (ends, last, count) = match rule.ends {
            Ends::Never => (Ending::Never, start, DEFAULT_COUNT),
            Ends::On(last) => (Ending::OnDate, last, DEFAULT_COUNT),
            Ends::After(count) => (Ending::AfterCount, start, count),
        };
        Custom {
            start,
            interval: rule.interval.to_string(),
            frequency: rule.frequency,
            days: rule.days,
            // A weekday's fifth occurrence in the month is its last (`DEC-167`).
            month_day: match rule.month_day {
                MonthDay::Weekday if week_of_month(start) == 5 => MonthDay::Last,
                month_day => month_day,
            },
            ends,
            last_date: regional.numeric_date(last),
            count: count.to_string(),
            problem: None,
            week_start: rule.week_start,
        }
    }

    pub fn frequency(&self) -> Frequency {
        self.frequency
    }

    /// Repeats in another unit; for weeks, on the start's weekday when no day is chosen
    /// (`MOD-05-U05`).
    pub fn set_frequency(&mut self, frequency: Frequency) {
        self.frequency = frequency;
        if frequency == Frequency::Weekly && self.days.is_empty() {
            self.days = vec![self.start.weekday()];
        }
    }

    /// The weekdays a weekly series repeats on.
    pub fn days(&self) -> &[Weekday] {
        &self.days
    }

    /// Adds the day to the days a weekly series repeats on, or takes it away; the last one stays
    /// (MOD-05 *Constraints and edge cases*).
    pub fn toggle_day(&mut self, day: Weekday) {
        if !self.days.contains(&day) {
            self.days.push(day);
        } else if self.days.len() > 1 {
            self.days.retain(|d| *d != day);
        }
    }

    /// The days of the month a monthly series can repeat on from the start, each in words:
    /// "Monthly on day 26", "Monthly on the fourth Thursday", and "Monthly on the last Thursday"
    /// for a day in the month's last seven; only the day and the last for the fifth weekday of
    /// the month (`MOD-05-F33`, `DEC-167`).
    pub fn month_days(&self) -> Vec<(MonthDay, String)> {
        let mut choices = vec![MonthDay::Day];
        if week_of_month(self.start) < 5 {
            choices.push(MonthDay::Weekday);
        }
        if in_last_week(self.start) {
            choices.push(MonthDay::Last);
        }
        (choices.into_iter())
            .map(|choice| {
                let words = format!("{} {}", texts::MONTHLY, on_month_day(choice, self.start));
                (choice, words)
            })
            .collect()
    }

    /// The rule the fields say; why the dialog cannot be closed with it otherwise.
    pub fn rule(&self, regional: &Regional) -> Result<Rule, String> {
        let interval = (self.interval.trim().parse().ok())
            .filter(|n| *n > 0)
            .ok_or(texts::INTERVAL_UNREADABLE)?;
        let ends = match self.ends {
            Ending::Never => Ends::Never,
            Ending::OnDate => {
                let last = regional
                    .parse_date(&self.last_date)
                    .ok_or(texts::SERIES_END_UNREADABLE)?;
                if last < self.start {
                    return Err(texts::SERIES_ENDS_BEFORE_IT_STARTS.to_string());
                }
                Ends::On(last)
            }
            Ending::AfterCount => Ends::After(
                (self.count.trim().parse().ok())
                    .filter(|n| *n > 0)
                    .ok_or(texts::COUNT_UNREADABLE)?,
            ),
        };
        let weekly = self.frequency == Frequency::Weekly;
        Ok(Rule {
            frequency: self.frequency,
            interval,
            days: if weekly {
                self.days.clone()
            } else {
                Vec::new()
            },
            month_day: match self.frequency {
                Frequency::Monthly => self.month_day,
                _ => MonthDay::Day,
            },
            ends,
            week_start: self.week_start,
        })
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

fn day_named(code: &str) -> Option<Weekday> {
    Some(match code {
        "MO" => Weekday::Mon,
        "TU" => Weekday::Tue,
        "WE" => Weekday::Wed,
        "TH" => Weekday::Thu,
        "FR" => Weekday::Fri,
        "SA" => Weekday::Sat,
        "SU" => Weekday::Sun,
        _ => return None,
    })
}

/// The day's name in the interface's language.
pub fn day_name(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{all_day, d, timed, timed_in};
    use chrono_tz::America::New_York;
    use pure_rust_locales::Locale;

    fn us() -> Regional {
        Regional::new(Locale::en_US)
    }

    fn training() -> Timing {
        timed("Training", d(2026, 11, 16), (18, 0), (19, 0)).timing
    }

    fn on(date: NaiveDate) -> Timing {
        timed("Sync", date, (10, 0), (10, 30)).timing
    }

    fn lines(rules: &[&str]) -> Vec<String> {
        rules.iter().map(|line| line.to_string()).collect()
    }

    fn weekly(days: &[Weekday]) -> Rule {
        Rule::weekly(days.to_vec())
    }

    fn ends(rule: Rule, ends: Ends) -> Rule {
        Rule { ends, ..rule }
    }

    #[test]
    fn tc_05_07_each_preset_becomes_the_rule_google_keeps() {
        let [daily, weekly, monthly, yearly, weekdays] = presets(d(2026, 11, 16));
        let never = |rule_| rule(&rule_, &training());
        assert_eq!(never(daily), "RRULE:FREQ=DAILY");
        assert_eq!(never(weekly), "RRULE:FREQ=WEEKLY;BYDAY=MO");
        assert_eq!(never(monthly), "RRULE:FREQ=MONTHLY;BYDAY=3MO");
        assert_eq!(never(yearly), "RRULE:FREQ=YEARLY");
        assert_eq!(never(weekdays), "RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR");
    }

    #[test]
    fn tc_05_07_weekly_on_monday_and_wednesday_ending_after_10_occurrences() {
        let days = weekly(&[Weekday::Mon, Weekday::Wed]);
        assert_eq!(
            rule(&ends(days, Ends::After(10)), &training()),
            "RRULE:FREQ=WEEKLY;BYDAY=MO,WE;COUNT=10"
        );
    }

    #[test]
    fn the_chosen_days_go_in_the_order_of_the_week_once_each() {
        let days = weekly(&[Weekday::Sun, Weekday::Wed, Weekday::Mon, Weekday::Wed]);
        assert_eq!(rule(&days, &training()), "RRULE:FREQ=WEEKLY;BYDAY=MO,WE,SU");
        assert_eq!(
            describe(&days, d(2026, 11, 16), &us()),
            "Weekly on Monday, Wednesday, Sunday"
        );
    }

    #[test]
    fn a_timed_series_ending_on_a_date_ends_with_that_day_in_its_own_zone() {
        let daily = |last| ends(Rule::every(Frequency::Daily), Ends::On(last));
        // Warsaw is an hour ahead of UTC in December.
        assert_eq!(
            rule(&daily(d(2026, 12, 16)), &training()),
            "RRULE:FREQ=DAILY;UNTIL=20261216T225959Z"
        );
        let call = timed_in("Call", d(2026, 11, 16), (9, 0), (10, 0), New_York).timing;
        assert_eq!(
            rule(&daily(d(2026, 7, 1)), &call),
            "RRULE:FREQ=DAILY;UNTIL=20260702T035959Z"
        );
    }

    #[test]
    fn an_all_day_series_ending_on_a_date_ends_on_that_date() {
        let trip = all_day("Trip", d(2026, 11, 16), 1).timing;
        let yearly = ends(Rule::every(Frequency::Yearly), Ends::On(d(2030, 11, 16)));
        assert_eq!(rule(&yearly, &trip), "RRULE:FREQ=YEARLY;UNTIL=20301116");
        let monthly = ends(Rule::every(Frequency::Monthly), Ends::After(3));
        assert_eq!(rule(&monthly, &trip), "RRULE:FREQ=MONTHLY;COUNT=3");
    }

    #[test]
    fn tc_05_13_the_presets_for_tuesday_17_november_read_as_googles_menu() {
        let start = d(2026, 11, 17);
        let words: Vec<String> = (presets(start).iter())
            .map(|preset| describe(preset, start, &us()))
            .collect();
        assert_eq!(
            words,
            [
                "Daily",
                "Weekly on Tuesday",
                "Monthly on the third Tuesday",
                "Annually on November 17",
                "Every weekday (Monday to Friday)",
            ]
        );
    }

    #[test]
    fn tc_05_13_every_2_weeks_on_monday_and_wednesday_after_10_is_written_and_read_back() {
        let sync = on(d(2026, 11, 17));
        let custom = Rule {
            interval: 2,
            ..ends(weekly(&[Weekday::Mon, Weekday::Wed]), Ends::After(10))
        };

        let line = rule(&custom, &sync);

        assert_eq!(line, "RRULE:FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE;COUNT=10");
        let read = parse(&lines(&[&line]), &sync).unwrap();
        assert_eq!(read, custom);
        assert_eq!(
            describe(&read, d(2026, 11, 17), &us()),
            "Every 2 weeks on Monday, Wednesday, 10 times"
        );
    }

    #[test]
    fn tc_05_13_the_month_choices_follow_the_start() {
        let choices = |start| {
            let custom = Custom::of(None, start, &us());
            let words: Vec<String> = (custom.month_days().into_iter())
                .map(|(_, words)| words)
                .collect();
            words
        };
        assert_eq!(
            choices(d(2026, 11, 17)),
            ["Monthly on day 17", "Monthly on the third Tuesday"]
        );
        assert_eq!(
            choices(d(2026, 11, 26)),
            [
                "Monthly on day 26",
                "Monthly on the fourth Thursday",
                "Monthly on the last Thursday"
            ]
        );
        // The fifth Tuesday of December is its last; never "the fifth" (`DEC-167`).
        assert_eq!(
            choices(d(2026, 12, 29)),
            ["Monthly on day 29", "Monthly on the last Tuesday"]
        );
    }

    #[test]
    fn tc_05_13_monthly_on_the_last_thursday_falls_on_the_last_thursday_of_every_month() {
        let thanksgiving = on(d(2026, 11, 26));
        let last = Rule {
            month_day: MonthDay::Last,
            ..Rule::every(Frequency::Monthly)
        };

        let line = rule(&last, &thanksgiving);

        assert_eq!(line, "RRULE:FREQ=MONTHLY;BYDAY=-1TH");
        assert_eq!(parse(&lines(&[&line]), &thanksgiving), Some(last.clone()));
        assert_eq!(
            describe(&last, d(2026, 11, 26), &us()),
            "Monthly on the last Thursday"
        );
    }

    #[test]
    fn tc_05_13_on_the_fifth_tuesday_the_menu_reads_the_last_tuesday() {
        let start = d(2026, 12, 29);
        let monthly = &presets(start)[2];
        assert_eq!(
            describe(monthly, start, &us()),
            "Monthly on the last Tuesday"
        );
        assert_eq!(rule(monthly, &on(start)), "RRULE:FREQ=MONTHLY;BYDAY=-1TU");
        // Read back, it is the last Tuesday, as the dialog offers it.
        let read = parse(&lines(&["RRULE:FREQ=MONTHLY;BYDAY=-1TU"]), &on(start)).unwrap();
        assert_eq!(read.month_day, MonthDay::Last);
        assert_eq!(
            Custom::of(Some(monthly), start, &us()).month_day,
            MonthDay::Last
        );
    }

    #[test]
    fn tc_05_13_the_dialog_shows_the_weekdays_only_for_weeks_with_the_start_chosen_and_never_none()
    {
        let mut custom = Custom::of(None, d(2026, 11, 17), &us());
        // It opens on weeks, on the start's weekday.
        assert_eq!(custom.frequency(), Frequency::Weekly);
        assert_eq!(custom.days(), [Weekday::Tue]);
        custom.toggle_day(Weekday::Tue);
        assert_eq!(custom.days(), [Weekday::Tue]);

        custom.set_frequency(Frequency::Monthly);
        let monthly = custom.rule(&us()).unwrap();
        assert!(monthly.days.is_empty());
        assert_eq!(monthly.frequency, Frequency::Monthly);

        let mut daily = Custom::of(Some(&presets(d(2026, 11, 17))[0]), d(2026, 11, 17), &us());
        assert!(daily.days().is_empty());
        daily.set_frequency(Frequency::Weekly);
        assert_eq!(daily.days(), [Weekday::Tue]);
    }

    #[test]
    fn tc_05_14_every_3_months_on_the_second_monday_until_30_june_2027_is_read_and_worded() {
        let quarterly = on(d(2026, 11, 9));
        let from_google =
            lines(&["RRULE:FREQ=MONTHLY;INTERVAL=3;BYDAY=2MO;UNTIL=20270630T235959Z"]);

        let read = parse(&from_google, &quarterly).unwrap();

        assert_eq!(
            read,
            Rule {
                interval: 3,
                month_day: MonthDay::Weekday,
                ends: Ends::On(d(2027, 6, 30)),
                ..Rule::every(Frequency::Monthly)
            }
        );
        assert_eq!(
            describe(&read, d(2026, 11, 9), &us()),
            "Every 3 months on the second Monday, until June 30, 2027"
        );
        // Read from one of its occurrences, the 8th of February, the second Monday too.
        assert_eq!(parse(&from_google, &on(d(2027, 2, 8))), Some(read.clone()));
        let every_2 = Rule {
            interval: 2,
            ..read
        };
        assert_eq!(
            rule(&every_2, &on(d(2027, 2, 8))),
            "RRULE:FREQ=MONTHLY;INTERVAL=2;BYDAY=2MO;UNTIL=20270630T215959Z"
        );
    }

    #[test]
    fn tc_05_14_rules_beyond_the_dialog_are_complex() {
        let odd = on(d(2026, 11, 2));
        for complex in [
            vec!["RRULE:FREQ=MONTHLY;BYDAY=MO,TU;BYSETPOS=1"],
            vec!["RRULE:FREQ=YEARLY;BYMONTH=1,7"],
            vec!["RRULE:FREQ=DAILY;BYHOUR=9,15"],
            vec!["RRULE:FREQ=HOURLY"],
            vec!["RRULE:FREQ=WEEKLY;BYDAY=MO", "RRULE:FREQ=MONTHLY"],
            vec![
                "RRULE:FREQ=WEEKLY",
                "RDATE;TZID=Europe/Warsaw:20261111T150000",
            ],
            // Not the start's own day or week of the month.
            vec!["RRULE:FREQ=MONTHLY;BYMONTHDAY=5"],
            vec!["RRULE:FREQ=MONTHLY;BYDAY=3MO"],
            vec!["RRULE:FREQ=MONTHLY;BYDAY=1TU"],
            vec!["RRULE:FREQ=MONTHLY;BYDAY=-1MO"],
            vec!["RRULE:FREQ=WEEKLY;BYDAY=1MO"],
            vec!["RRULE:FREQ=DAILY;COUNT=3;UNTIL=20261130"],
            vec![],
        ] {
            assert_eq!(parse(&lines(&complex), &odd), None, "{complex:?}");
        }
    }

    #[test]
    fn rules_as_google_writes_them_are_read_as_the_same_day_rules_with_their_exceptions() {
        let start = d(2026, 11, 17);
        let first = on(start);
        let read = |rules: &[&str]| parse(&lines(rules), &first);
        assert_eq!(
            read(&["RRULE:FREQ=MONTHLY;BYMONTHDAY=17"]),
            Some(Rule::every(Frequency::Monthly))
        );
        assert_eq!(
            read(&["RRULE:FREQ=YEARLY;BYMONTH=11;BYMONTHDAY=17"]),
            Some(Rule::every(Frequency::Yearly))
        );
        assert_eq!(read(&["RRULE:FREQ=WEEKLY"]), Some(weekly(&[Weekday::Tue])));
        assert_eq!(
            read(&[
                "EXDATE;TZID=Europe/Warsaw:20261124T100000",
                "RRULE:FREQ=DAILY;COUNT=5"
            ]),
            Some(ends(Rule::every(Frequency::Daily), Ends::After(5)))
        );
        // A week start Google wrote is kept.
        let with_week_start = read(&["RRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=2;BYDAY=TU"]).unwrap();
        assert_eq!(
            rule(&with_week_start, &first),
            "RRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=2;BYDAY=TU"
        );
    }

    #[test]
    fn an_end_given_as_a_date_or_a_local_time_is_the_last_day_with_an_occurrence() {
        let read = |until: &str| {
            let rules = lines(&[&format!("RRULE:FREQ=DAILY;UNTIL={until}")]);
            parse(&rules, &on(d(2026, 11, 17))).map(|rule| rule.ends)
        };
        for until in [
            "20261130",
            "20261130T100000",
            "20261130T235959Z",
            "20261130T225959Z",
        ] {
            assert_eq!(read(until), Some(Ends::On(d(2026, 11, 30))), "{until}");
        }
        // Before the time of day it starts, the day before.
        assert_eq!(read("20261130T085959Z"), Some(Ends::On(d(2026, 11, 29))));
        let trip = all_day("Trip", d(2026, 11, 16), 1).timing;
        let rules = lines(&["RRULE:FREQ=DAILY;UNTIL=20261130T000000Z"]);
        assert_eq!(
            parse(&rules, &trip).map(|rule| rule.ends),
            Some(Ends::On(d(2026, 11, 30)))
        );
    }

    #[test]
    fn rules_are_worded_with_their_interval_and_end() {
        let start = d(2026, 11, 17);
        let words = |rule_: Rule| describe(&rule_, start, &us());
        let every = |frequency, interval| Rule {
            interval,
            ..Rule::every(frequency)
        };
        assert_eq!(words(every(Frequency::Daily, 3)), "Every 3 days");
        assert_eq!(
            words(every(Frequency::Yearly, 2)),
            "Every 2 years on November 17"
        );
        assert_eq!(words(every(Frequency::Monthly, 1)), "Monthly on day 17");
        assert_eq!(
            words(ends(every(Frequency::Daily, 1), Ends::After(1))),
            "Daily, once"
        );
        assert_eq!(
            words(ends(
                Rule::weekly(WEEKDAYS.to_vec()),
                Ends::On(d(2026, 12, 31))
            )),
            "Every weekday (Monday to Friday), until December 31, 2026"
        );
    }

    #[test]
    fn the_dialog_does_not_take_an_end_before_the_start_nor_no_occurrences_and_says_why() {
        let mut custom = Custom::of(None, d(2026, 11, 17), &us());
        custom.ends = Ending::OnDate;
        // It ends on the day it starts until another is typed.
        assert_eq!(custom.last_date, "11/17/2026");
        assert_eq!(custom.rule(&us()).unwrap().ends, Ends::On(d(2026, 11, 17)));
        custom.last_date = "11/16/2026".into();
        assert_eq!(
            custom.rule(&us()),
            Err(texts::SERIES_ENDS_BEFORE_IT_STARTS.to_string())
        );
        custom.last_date = "someday".into();
        assert_eq!(
            custom.rule(&us()),
            Err(texts::SERIES_END_UNREADABLE.to_string())
        );
        custom.ends = Ending::AfterCount;
        // After 13 occurrences, Google's default, until another number is typed.
        assert_eq!(custom.rule(&us()).unwrap().ends, Ends::After(13));
        custom.count = "0".into();
        assert_eq!(custom.rule(&us()), Err(texts::COUNT_UNREADABLE.to_string()));
        custom.ends = Ending::Never;
        custom.interval = "0".into();
        assert_eq!(
            custom.rule(&us()),
            Err(texts::INTERVAL_UNREADABLE.to_string())
        );
    }

    #[test]
    fn the_dialog_opens_filled_with_the_rule_chosen() {
        let quarterly = Rule {
            interval: 3,
            month_day: MonthDay::Weekday,
            ends: Ends::On(d(2027, 6, 30)),
            ..Rule::every(Frequency::Monthly)
        };

        let custom = Custom::of(Some(&quarterly), d(2026, 11, 9), &us());

        assert_eq!(custom.interval, "3");
        assert_eq!(custom.frequency(), Frequency::Monthly);
        assert_eq!(custom.month_day, MonthDay::Weekday);
        assert_eq!(custom.ends, Ending::OnDate);
        assert_eq!(custom.last_date, "06/30/2027");
        assert_eq!(custom.rule(&us()), Ok(quarterly));
    }
}
