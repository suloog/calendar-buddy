use chrono::{
    DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, Utc, Weekday,
};
use chrono_tz::Tz;

/// Seven consecutive days starting on the first day of the week of the system's locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Week {
    pub first: NaiveDate,
}

impl Week {
    pub fn containing(date: NaiveDate, start: Weekday) -> Self {
        let offset = date.weekday().days_since(start) as i64;
        Week {
            first: date - Duration::days(offset),
        }
    }

    pub fn days(&self) -> [NaiveDate; 7] {
        std::array::from_fn(|i| self.first + Duration::days(i as i64))
    }

    pub fn next(&self) -> Self {
        Week {
            first: self.first + Duration::days(7),
        }
    }

    pub fn previous(&self) -> Self {
        Week {
            first: self.first - Duration::days(7),
        }
    }
}

/// The views drawn on the hour grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridView {
    Day,
    ThreeDays,
    Week,
}

impl GridView {
    /// The days shown for the selected date: that day, three days from it (as Google Calendar's
    /// custom view of 3 days), or the week containing it.
    pub fn days(self, selected: NaiveDate, week_start: Weekday) -> Vec<NaiveDate> {
        match self {
            GridView::Day => vec![selected],
            GridView::ThreeDays => (0..3).map(|i| selected + Duration::days(i)).collect(),
            GridView::Week => Week::containing(selected, week_start).days().to_vec(),
        }
    }
}

/// The instant a local day starts. A zone may skip midnight; the day then starts at the first
/// local time that exists.
pub fn day_start(date: NaiveDate, tz: Tz) -> DateTime<Utc> {
    let mut local = date.and_time(NaiveTime::MIN);
    loop {
        if let Some(t) = tz.from_local_datetime(&local).earliest() {
            return t.with_timezone(&Utc);
        }
        local += Duration::minutes(15);
    }
}

/// The real span of a local day: 23 or 25 hours on a daylight-saving change.
pub fn day_span(date: NaiveDate, tz: Tz) -> (DateTime<Utc>, DateTime<Utc>) {
    (day_start(date, tz), day_start(date + Duration::days(1), tz))
}

/// Minutes since local midnight on the wall clock. On a daylight-saving day the grid follows the
/// clock, as Google Calendar does (`DEC-67`): the skipped hour stays empty, the repeated hour is
/// shared by both of its occurrences.
pub fn wall_minute(t: DateTime<Utc>, tz: Tz) -> u32 {
    let local = t.with_timezone(&tz);
    local.hour() * 60 + local.minute()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Europe::Warsaw;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn week_starts_on_monday() {
        let w = Week::containing(d(2026, 11, 11), Weekday::Mon);
        assert_eq!(w.first, d(2026, 11, 9));
        assert_eq!(w.days()[6], d(2026, 11, 15));
        assert_eq!(w.next().first, d(2026, 11, 16));
        assert_eq!(w.previous().first, d(2026, 11, 2));
    }

    #[test]
    fn tc_03_05_day_shows_the_selected_day_and_three_days_start_from_it() {
        let selected = d(2026, 11, 11);
        assert_eq!(GridView::Day.days(selected, Weekday::Mon), vec![selected]);
        assert_eq!(
            GridView::ThreeDays.days(selected, Weekday::Sun),
            vec![d(2026, 11, 11), d(2026, 11, 12), d(2026, 11, 13)]
        );
        assert_eq!(
            GridView::Week.days(selected, Weekday::Mon),
            Week::containing(selected, Weekday::Mon).days()
        );
    }

    #[test]
    fn a_week_can_start_on_any_day() {
        let sunday = Week::containing(d(2026, 11, 11), Weekday::Sun);
        assert_eq!(sunday.first, d(2026, 11, 8));
        assert_eq!(sunday.days()[6], d(2026, 11, 14));
        // A Sunday starts its own week, and is the last day of a week starting on Monday.
        assert_eq!(
            Week::containing(d(2026, 11, 8), Weekday::Sun).first,
            d(2026, 11, 8)
        );
        assert_eq!(
            Week::containing(d(2026, 11, 8), Weekday::Mon).first,
            d(2026, 11, 2)
        );
        assert_eq!(
            Week::containing(d(2026, 11, 11), Weekday::Sat).first,
            d(2026, 11, 7)
        );
    }

    #[test]
    fn daylight_saving_days_have_23_and_25_hours() {
        let minutes = |date| {
            let (s, e) = day_span(date, Warsaw);
            (e - s).num_minutes()
        };
        assert_eq!(minutes(d(2026, 3, 29)), 23 * 60);
        assert_eq!(minutes(d(2026, 10, 25)), 25 * 60);
        assert_eq!(minutes(d(2026, 11, 11)), 24 * 60);
    }
}
