//! The system's regional settings (`DEC-29`): the first day of the week and how dates and times
//! are written, from the locale's `LC_TIME` data. Independent of how the locale is found.

use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Timelike, Weekday};
use pure_rust_locales::{Locale, locale_match};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Regional {
    pub locale: Locale,
    pub week_start: Weekday,
    twelve_hour: bool,
    /// The month comes before the day, as in `11/12/2026` for 12 November.
    month_first: bool,
    /// The locale's numeric date, as `%m/%d/%Y`; `%Y-%m-%d` when it cannot be read back.
    numeric: &'static str,
    am_pm: &'static [&'static str],
    /// Month names on their own, as in a title; some languages write them otherwise in a date.
    months: &'static [&'static str],
}

impl Regional {
    pub fn new(locale: Locale) -> Self {
        let numeric = locale_match!(locale => LC_TIME::D_FMT);
        let time = locale_match!(locale => LC_TIME::T_FMT);
        // glibc counts the first weekday from the day of `WEEK`'s second field, 1997-11-30 (a
        // Sunday) by default, and from 1.
        let base = locale_match!(locale => LC_TIME::WEEK)
            .and_then(|week| week.get(1))
            .and_then(|&day| NaiveDate::parse_from_str(&day.to_string(), "%Y%m%d").ok())
            .unwrap_or(NaiveDate::from_ymd_opt(1997, 11, 30).expect("a date"));
        let first = locale_match!(locale => LC_TIME::FIRST_WEEKDAY).unwrap_or(1);
        let at = |specifiers: &[&str]| specifiers.iter().filter_map(|f| numeric.find(f)).min();
        let month_at = at(&["%m", "%D"]);
        let day_at = at(&["%d", "%e"]);
        let year_at = at(&["%Y", "%y"]);
        Regional {
            locale,
            week_start: (base + Duration::days(first - 1)).weekday(),
            twelve_hour: ["%I", "%l", "%r"].iter().any(|f| time.contains(f)),
            // Month, day, year; a year first (as `%Y-%m-%d`) writes the day before the month.
            month_first: matches!((month_at, day_at), (Some(m), Some(d))
                if m < d && year_at.is_none_or(|y| d < y)),
            numeric: if round_trips(numeric, locale) {
                numeric
            } else {
                ISO_DATE
            },
            am_pm: locale_match!(locale => LC_TIME::AM_PM),
            months: locale_match!(locale => LC_TIME::ALT_MON)
                .unwrap_or(locale_match!(locale => LC_TIME::MON)),
        }
    }

    /// The locale of dates and times as the system sets it: `LC_ALL`, else `LC_TIME`, else
    /// `LANG`. An unset or unknown locale is the POSIX one.
    pub fn from_env(var: impl Fn(&str) -> Option<String>) -> Self {
        let name = ["LC_ALL", "LC_TIME", "LANG"]
            .into_iter()
            .filter_map(var)
            .find(|value| !value.is_empty())
            .unwrap_or_default();
        Regional::new(locale_named(&name))
    }

    fn format(&self, date: NaiveDate, day_first: &str, month_first: &str) -> String {
        let pattern = if self.month_first {
            month_first
        } else {
            day_first
        };
        date.format_localized(pattern, self.locale).to_string()
    }

    /// A time of day: `3:00 PM` on a 12-hour clock, `15:00` on a 24-hour one.
    pub fn time(&self, time: NaiveTime) -> String {
        if self.twelve_hour {
            let (pm, hour) = time.hour12();
            format!("{hour}:{:02} {}", time.minute(), self.am_pm[pm as usize])
        } else {
            format!("{:02}:{:02}", time.hour(), time.minute())
        }
    }

    /// A time typed on the 24-hour clock, as `15:00`, or on the 12-hour one with the locale's
    /// AM or PM after it, as `3:00 PM`.
    pub fn parse_time(&self, text: &str) -> Option<NaiveTime> {
        let text = text.trim();
        let half = self.am_pm.iter().enumerate().find_map(|(pm, mark)| {
            let mark = mark.to_lowercase();
            let lower = text.to_lowercase();
            let rest = lower.strip_suffix(&mark).filter(|_| !mark.is_empty())?;
            Some((pm == 1, rest.trim().to_string()))
        });
        match half {
            Some((pm, rest)) => {
                let time = NaiveTime::parse_from_str(&rest, "%H:%M")
                    .ok()
                    .filter(|time| (1..=12).contains(&time.hour()))?;
                // 12:xx AM is just after midnight, 12:xx PM just after noon.
                let hour = time.hour() % 12 + if pm { 12 } else { 0 };
                time.with_hour(hour)
            }
            None => NaiveTime::parse_from_str(text, "%H:%M").ok(),
        }
    }

    /// The label of an hour of the grid: `3 PM` or `15:00`.
    pub fn hour(&self, hour: u32) -> String {
        if self.twelve_hour {
            let (pm, hour) = NaiveTime::from_hms_opt(hour, 0, 0)
                .expect("an hour of the day")
                .hour12();
            format!("{hour} {}", self.am_pm[pm as usize])
        } else {
            format!("{hour:02}:00")
        }
    }

    /// The locale's numeric date, as `11/12/2026`.
    pub fn numeric_date(&self, date: NaiveDate) -> String {
        date.format_localized(self.numeric, self.locale).to_string()
    }

    /// A date typed in the locale's numeric format or as `YYYY-MM-DD`.
    pub fn parse_date(&self, text: &str) -> Option<NaiveDate> {
        let text = text.trim();
        NaiveDate::parse_from_str(text, self.numeric)
            .or_else(|_| NaiveDate::parse_from_str(text, ISO_DATE))
            .ok()
    }

    /// `November 2026`.
    pub fn month_year(&self, date: NaiveDate) -> String {
        format!("{} {}", self.months[date.month0() as usize], date.year())
    }

    /// `Wed`.
    pub fn weekday(&self, date: NaiveDate) -> String {
        date.format_localized("%a", self.locale).to_string()
    }

    /// `Wed 11`, the head of a day's column.
    pub fn weekday_day(&self, date: NaiveDate) -> String {
        date.format_localized("%a %-d", self.locale).to_string()
    }

    /// `Wed 11 Nov 2026`.
    pub fn short_date(&self, date: NaiveDate) -> String {
        self.format(date, "%a %-d %b %Y", "%a, %b %-d, %Y")
    }

    /// `11 Nov – 17 Nov 2026`.
    pub fn date_range(&self, first: NaiveDate, last: NaiveDate) -> String {
        format!(
            "{} – {}",
            self.format(first, "%-d %b", "%b %-d"),
            self.format(last, "%-d %b %Y", "%b %-d, %Y")
        )
    }

    /// `Wednesday 11 November 2026`.
    pub fn long_date(&self, date: NaiveDate) -> String {
        self.format(date, "%A %-d %B %Y", "%A, %B %-d, %Y")
    }

    /// `Wednesday 11 November`.
    pub fn weekday_day_month(&self, date: NaiveDate) -> String {
        self.format(date, "%A %-d %B", "%A, %B %-d")
    }

    /// `11 November 2026`.
    pub fn day_month_year(&self, date: NaiveDate) -> String {
        self.format(date, "%-d %B %Y", "%B %-d, %Y")
    }

    /// `11 November`.
    pub fn day_month(&self, date: NaiveDate) -> String {
        self.format(date, "%-d %B", "%B %-d")
    }

    /// `NOV, WED`, the side of a day in the agenda.
    pub fn month_weekday(&self, date: NaiveDate) -> String {
        date.format_localized("%b, %a", self.locale)
            .to_string()
            .to_uppercase()
    }

    /// The first letters of the days of the week, from the first day.
    pub fn weekday_letters(&self) -> [String; 7] {
        let mut day = self.week_start;
        std::array::from_fn(|_| {
            let name =
                locale_match!(self.locale => LC_TIME::ABDAY)[day.num_days_from_sunday() as usize];
            day = day.succ();
            name.chars()
                .next()
                .into_iter()
                .flat_map(char::to_uppercase)
                .collect()
        })
    }
}

/// The date the `g` dialog takes in any locale (`DEC-73`).
const ISO_DATE: &str = "%Y-%m-%d";

/// Whether a date of every month written in `pattern` reads back as the same date. Some locales
/// write theirs with names or in specifiers chrono does not know.
fn round_trips(pattern: &str, locale: Locale) -> bool {
    use std::fmt::Write;
    (1..=12).all(|month| {
        let date = NaiveDate::from_ymd_opt(2026, month, 12).expect("a date");
        let mut text = String::new();
        write!(text, "{}", date.format_localized(pattern, locale)).is_ok()
            && NaiveDate::parse_from_str(&text, pattern) == Ok(date)
    })
}

/// The locale of a name as the system writes it, as `pl_PL.UTF-8` or `aa_ER.UTF-8@saaho`.
fn locale_named(name: &str) -> Locale {
    let (base, modifier) = name.split_once('@').unwrap_or((name, ""));
    let language = base.split('.').next().unwrap_or_default();
    let with_modifier = if modifier.is_empty() {
        language.to_string()
    } else {
        format!("{language}@{modifier}")
    };
    Locale::try_from(with_modifier.as_str())
        .or_else(|_| Locale::try_from(language))
        .unwrap_or(Locale::POSIX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;

    fn named(name: &str) -> Regional {
        Regional::from_env(|key| (key == "LANG").then(|| name.to_string()))
    }

    fn at(hour: u32, minute: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
    }

    #[test]
    fn lc_all_comes_before_lc_time_and_lc_time_before_lang() {
        let env = |all: &'static str| {
            move |key: &str| {
                match key {
                    "LC_ALL" => Some(all),
                    "LC_TIME" => Some("en_US.UTF-8"),
                    "LANG" => Some("pl_PL.UTF-8"),
                    _ => None,
                }
                .map(str::to_string)
            }
        };
        assert_eq!(Regional::from_env(env("de_DE.UTF-8")).locale, Locale::de_DE);
        assert_eq!(Regional::from_env(env("")).locale, Locale::en_US);
        assert_eq!(Regional::from_env(|_| None).locale, Locale::POSIX);
    }

    #[test]
    fn a_locale_name_is_read_with_or_without_its_charset_and_modifier() {
        assert_eq!(named("pl_PL.UTF-8").locale, Locale::pl_PL);
        assert_eq!(named("en_GB").locale, Locale::en_GB);
        assert_eq!(named("aa_ER.UTF-8@saaho").locale, Locale::aa_ER_saaho);
        assert_eq!(named("de_DE.UTF-8@nope").locale, Locale::de_DE);
        for unknown in ["C", "C.UTF-8", "POSIX", "xx_YY.UTF-8"] {
            assert_eq!(named(unknown).locale, Locale::POSIX, "{unknown}");
        }
    }

    #[test]
    fn tc_03_12_a_us_locale_starts_the_week_on_sunday_with_us_formats() {
        let us = named("en_US.UTF-8");
        assert_eq!(us.week_start, Weekday::Sun);
        assert_eq!(us.time(at(15, 0)), "3:00 PM");
        assert_eq!(us.time(at(0, 30)), "12:30 AM");
        assert_eq!(us.hour(9), "9 AM");
        assert_eq!(us.hour(12), "12 PM");
        assert_eq!(us.numeric_date(d(2026, 11, 12)), "11/12/2026");
        assert_eq!(us.long_date(d(2026, 11, 12)), "Thursday, November 12, 2026");
        assert_eq!(us.short_date(d(2026, 11, 12)), "Thu, Nov 12, 2026");
        assert_eq!(
            us.date_range(d(2026, 11, 8), d(2026, 11, 14)),
            "Nov 8 – Nov 14, 2026"
        );
        assert_eq!(us.weekday_day(d(2026, 11, 8)), "Sun 8");
        assert_eq!(us.weekday_letters().join(""), "SMTWTFS");
    }

    #[test]
    fn a_european_locale_starts_on_monday_with_a_24_hour_clock_and_the_day_first() {
        let gb = named("en_GB.UTF-8");
        assert_eq!(gb.week_start, Weekday::Mon);
        assert_eq!(gb.time(at(15, 0)), "15:00");
        assert_eq!(gb.hour(9), "09:00");
        assert_eq!(gb.long_date(d(2026, 11, 12)), "Thursday 12 November 2026");
        assert_eq!(
            gb.date_range(d(2026, 11, 9), d(2026, 11, 15)),
            "9 Nov – 15 Nov 2026"
        );
        assert_eq!(gb.weekday_letters().join(""), "MTWTFSS");
    }

    #[test]
    fn names_of_days_and_months_are_the_locales_own() {
        let pl = named("pl_PL.UTF-8");
        assert_eq!(pl.week_start, Weekday::Mon);
        assert_eq!(pl.numeric_date(d(2026, 11, 12)), "12.11.2026");
        // A month in a date and a month on its own are written differently in Polish.
        assert_eq!(pl.day_month_year(d(2026, 11, 12)), "12 listopada 2026");
        assert_eq!(pl.month_year(d(2026, 11, 12)), "listopad 2026");
        assert_eq!(pl.weekday_letters().join(""), "PWŚCPSN");
        assert_eq!(named("en_US").month_year(d(2026, 11, 1)), "November 2026");
    }

    #[test]
    fn a_typed_date_is_read_in_the_locales_format_or_as_year_month_day() {
        let us = named("en_US.UTF-8");
        assert_eq!(us.parse_date(" 3/4/2027 "), Some(d(2027, 3, 4)));
        assert_eq!(us.parse_date("2027-03-04"), Some(d(2027, 3, 4)));
        assert_eq!(named("pl_PL").parse_date("04.03.2027"), Some(d(2027, 3, 4)));
        assert_eq!(us.parse_date("2/30/2027"), None);
        assert_eq!(us.parse_date("tomorrow"), None);
    }

    #[test]
    fn a_typed_time_is_read_on_the_24_hour_clock_or_with_the_locales_am_and_pm() {
        let us = named("en_US.UTF-8");
        assert_eq!(us.parse_time("3:05 PM"), Some(at(15, 5)));
        assert_eq!(us.parse_time(" 12:30am"), Some(at(0, 30)));
        assert_eq!(us.parse_time("12:00 PM"), Some(at(12, 0)));
        assert_eq!(us.parse_time("15:00"), Some(at(15, 0)));
        let pl = named("pl_PL");
        assert_eq!(pl.parse_time("9:00"), Some(at(9, 0)));
        assert_eq!(pl.parse_time(&pl.time(at(23, 45))), Some(at(23, 45)));
        assert_eq!(us.parse_time(&us.time(at(23, 45))), Some(at(23, 45)));
        assert_eq!(us.parse_time("25:00"), None);
        assert_eq!(us.parse_time("13:00 PM"), None);
        assert_eq!(us.parse_time("noon"), None);
    }

    #[test]
    fn the_posix_locale_starts_on_sunday_with_a_24_hour_clock() {
        let posix = Regional::new(Locale::POSIX);
        assert_eq!(posix.week_start, Weekday::Sun);
        assert_eq!(posix.time(at(9, 5)), "09:05");
        assert_eq!(posix.numeric_date(d(2026, 11, 12)), "11/12/26");
    }

    #[test]
    fn a_numeric_date_that_cannot_be_read_back_is_written_year_month_day() {
        for name in ["th_TH", "fa_IR", "my_MM", "nb_NO", "ar_EG", "kl_GL"] {
            let regional = named(name);
            for month in 1..=12 {
                let date = d(2026, month, 1);
                let text = regional.numeric_date(date);
                assert_eq!(regional.parse_date(&text), Some(date), "{name} {text}");
            }
        }
        assert_eq!(named("th_TH").numeric_date(d(2026, 11, 12)), "2026-11-12");
    }

    #[test]
    fn a_locale_writing_the_year_first_puts_the_day_before_the_month() {
        let se = named("sv_SE.UTF-8");
        assert_eq!(se.numeric_date(d(2026, 11, 12)), "2026-11-12");
        assert_eq!(se.long_date(d(2026, 11, 12)), "torsdag 12 november 2026");
    }

    #[test]
    fn unusual_locales_write_and_read_back_their_dates_and_times() {
        let date = d(2026, 11, 12);
        for name in [
            "POSIX", "en_US", "th_TH", "lzh_TW", "or_IN", "ja_JP", "ar_SA", "az_IR",
        ] {
            let regional = named(name);
            assert_eq!(
                regional.parse_date(&regional.numeric_date(date)),
                Some(date)
            );
            regional.long_date(date);
            regional.date_range(date, date);
            regional.month_weekday(date);
            regional.time(at(15, 0));
            regional.weekday_letters();
        }
    }
}
