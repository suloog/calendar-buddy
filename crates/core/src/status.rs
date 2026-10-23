//! The synchronization indicator (`US-02-06`): whether the app is synchronizing, up to date,
//! offline or in error, with the time of the last successful synchronization (`MOD-02-F17`,
//! `MOD-02-U03`), and what keeps a calendar from being up to date, shown on that calendar
//! (`MOD-02-F18`).

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use crate::model::Calendar;
use crate::regional::Regional;

/// What keeps a calendar from being up to date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    /// Its last synchronization failed; it is tried again, waiting longer each time.
    Failed,
    /// Google asked to slow down; it is tried again later (`MOD-02-F16`).
    Delayed,
    /// Its account needs sign-in; its cached events stay shown (`MOD-01-F05`).
    NotSynchronizing,
}

/// Where synchronization stands, as the worker last told.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Status {
    /// A synchronization is under way.
    pub synchronizing: bool,
    /// Some cached calendars wait for their account's first synchronization of the run.
    pub waiting: bool,
    /// When a synchronization last got through; `None` before the first.
    pub last_synced: Option<DateTime<Utc>>,
    /// The calendars that are not up to date, and why.
    pub problems: Vec<(Calendar, Problem)>,
}

/// What the indicator says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Synchronizing,
    UpToDate,
    Offline,
    /// Google asked to slow down.
    Delayed,
    Error,
}

impl Status {
    /// Offline comes first: requests fail quietly then (`DEC-95`). A calendar in error or not
    /// synchronizing outweighs one delayed, and both a first synchronization still to come.
    pub fn state(&self, online: bool) -> State {
        let any =
            |problem: &dyn Fn(Problem) -> bool| self.problems.iter().any(|(_, p)| problem(*p));
        if !online {
            State::Offline
        } else if self.synchronizing {
            State::Synchronizing
        } else if any(&|p| p != Problem::Delayed) {
            State::Error
        } else if any(&|p| p == Problem::Delayed) {
            State::Delayed
        } else if self.waiting {
            State::Synchronizing
        } else {
            State::UpToDate
        }
    }

    /// What keeps the calendar from being up to date, as the user is told on its row; offline
    /// only an account needing sign-in, as the other failures are expected then.
    pub fn problem(&self, calendar: &Calendar, online: bool) -> Option<Problem> {
        self.problems
            .iter()
            .find(|(c, _)| c.account_id == calendar.account_id && c.id == calendar.id)
            .map(|(_, problem)| *problem)
            .filter(|problem| online || *problem == Problem::NotSynchronizing)
    }
}

pub fn label(state: State) -> &'static str {
    match state {
        State::Synchronizing => "Synchronizing…",
        State::UpToDate => "Up to date",
        State::Offline => "Offline",
        State::Delayed => "Synchronization delayed: Google asked to slow down",
        State::Error => "Synchronization error",
    }
}

/// `Last synchronized at 14:05`, with the date when it was not today.
pub fn last_synced(
    at: Option<DateTime<Utc>>,
    zone: Tz,
    today: NaiveDate,
    regional: &Regional,
) -> String {
    let Some(at) = at else {
        return "Not synchronized yet".to_string();
    };
    let local = at.with_timezone(&zone);
    let time = regional.time(local.time());
    if local.date_naive() == today {
        format!("Last synchronized at {time}")
    } else {
        let date = regional.short_date(local.date_naive());
        format!("Last synchronized on {date} at {time}")
    }
}

/// A problem as its calendar's row in the side panel says it (`DEC-97`).
pub fn on_row(problem: Problem) -> &'static str {
    match problem {
        Problem::Failed => "Could not be synchronized; trying again.",
        Problem::Delayed => "Synchronized later: Google asked to slow down.",
        Problem::NotSynchronizing => "Not synchronizing: its account needs sign-in.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::d;
    use crate::model::{Access, Rgb};
    use chrono::TimeZone;
    use chrono_tz::Europe::Warsaw;
    use pure_rust_locales::Locale;

    fn calendar(name: &str) -> Calendar {
        Calendar {
            id: name.to_lowercase(),
            account_id: "1".into(),
            name: name.into(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
        }
    }

    fn status(problems: &[(&str, Problem)]) -> Status {
        Status {
            synchronizing: false,
            waiting: false,
            last_synced: None,
            problems: problems
                .iter()
                .map(|(name, problem)| (calendar(name), *problem))
                .collect(),
        }
    }

    #[test]
    fn the_indicator_says_synchronizing_up_to_date_offline_delayed_or_error() {
        let up_to_date = status(&[]);
        let synchronizing = Status {
            synchronizing: true,
            ..status(&[])
        };
        let waiting = Status {
            waiting: true,
            ..status(&[])
        };
        let delayed = status(&[("Work", Problem::Delayed)]);
        let failed = status(&[("Work", Problem::Delayed), ("Home", Problem::Failed)]);
        let signed_out = status(&[("Home", Problem::NotSynchronizing)]);

        assert_eq!(up_to_date.state(true), State::UpToDate);
        assert_eq!(synchronizing.state(true), State::Synchronizing);
        assert_eq!(waiting.state(true), State::Synchronizing);
        let delayed_while_waiting = Status {
            waiting: true,
            ..status(&[("Work", Problem::Delayed)])
        };
        assert_eq!(delayed_while_waiting.state(true), State::Delayed);
        assert_eq!(delayed.state(true), State::Delayed);
        assert_eq!(failed.state(true), State::Error);
        assert_eq!(signed_out.state(true), State::Error);
        for status in [up_to_date, synchronizing, delayed, failed] {
            assert_eq!(status.state(false), State::Offline);
        }
        assert!(label(State::Delayed).contains("delayed"));
    }

    #[test]
    fn each_problem_is_told_on_its_calendar_and_offline_only_sign_in() {
        let status = status(&[
            ("Work", Problem::Failed),
            ("Team", Problem::Delayed),
            ("Home", Problem::NotSynchronizing),
        ]);

        let told = |online| {
            status
                .problems
                .iter()
                .map(|(calendar, _)| status.problem(calendar, online).map(on_row))
                .collect::<Vec<_>>()
        };

        assert_eq!(
            told(true),
            [
                Some("Could not be synchronized; trying again."),
                Some("Synchronized later: Google asked to slow down."),
                Some("Not synchronizing: its account needs sign-in."),
            ]
        );
        assert_eq!(
            told(false),
            [
                None,
                None,
                Some("Not synchronizing: its account needs sign-in.")
            ]
        );
        assert_eq!(status.problem(&calendar("Other"), true), None);
    }

    #[test]
    fn the_last_synchronization_is_given_in_local_time_with_the_date_when_not_today() {
        let regional = Regional::new(Locale::en_GB);
        let at = Utc.with_ymd_and_hms(2026, 11, 11, 13, 5, 0).unwrap();

        assert_eq!(
            last_synced(Some(at), Warsaw, d(2026, 11, 11), &regional),
            "Last synchronized at 14:05"
        );
        assert_eq!(
            last_synced(Some(at), Warsaw, d(2026, 11, 12), &regional),
            "Last synchronized on Wed 11 Nov 2026 at 14:05"
        );
        assert_eq!(
            last_synced(None, Warsaw, d(2026, 11, 12), &regional),
            "Not synchronized yet"
        );
    }
}
