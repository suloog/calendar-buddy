//! When each calendar is synchronized again (`MOD-02-F07`, `MOD-02-F23`, `DEC-53`, `DEC-54`):
//! adaptive polling by the account's access to the calendar and by whether the window is shown,
//! and growing waits with jitter after failures (`MOD-02-F16`).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::model::{Access, Calendar};

/// How long a calendar waits between synchronizations: calendars the account can edit about every
/// minute, read-only and subscribed ones about every 15 minutes, and less often while the window
/// is hidden.
pub fn interval(access: Access, shown: bool) -> Duration {
    let minutes = match (access.can_edit(), shown) {
        (true, true) => 1,
        (true, false) => 5,
        (false, true) => 15,
        (false, false) => 30,
    };
    Duration::from_secs(minutes * 60)
}

/// A calendar synchronized this recently is not synchronized again when the window is activated.
pub const ACTIVATION_GAP: Duration = Duration::from_secs(10);

/// The longest wait after failures: the longest polling interval.
pub const MAX_BACK_OFF: Duration = Duration::from_secs(30 * 60);

/// How long a calendar waits after failing `failures` times in a row: its interval at first,
/// doubled after each further failure up to `MAX_BACK_OFF`, and up to a quarter more by `jitter`
/// (from 0 to 1), so calendars that failed together do not retry together.
pub fn back_off(interval: Duration, failures: u32, jitter: f64) -> Duration {
    let doubled = 2u32.saturating_pow(failures.saturating_sub(1));
    interval
        .saturating_mul(doubled)
        .min(MAX_BACK_OFF)
        .mul_f64(1.0 + jitter / 4.0)
}

/// A polled calendar: when it was last synchronized or tried, and its failures since it last got
/// through, with the jitter of its wait.
#[derive(Clone)]
struct Turn {
    calendar: Calendar,
    last: Instant,
    failures: u32,
    jitter: f64,
}

/// The calendars polled in the background, and when each was last synchronized or tried.
pub struct Schedule {
    shown: bool,
    calendars: Vec<Turn>,
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule::new()
    }
}

impl Schedule {
    /// Nothing to poll yet, with the window shown.
    pub fn new() -> Self {
        Schedule {
            shown: true,
            calendars: Vec::new(),
        }
    }

    /// Whether the window is shown; hidden, calendars are polled less often.
    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    /// The account's calendars, all just synchronized; they replace those it had.
    pub fn set_account(&mut self, account_id: &str, calendars: &[Calendar], now: Instant) {
        self.forget(account_id);
        self.calendars.extend(calendars.iter().map(|calendar| Turn {
            calendar: calendar.clone(),
            last: now,
            failures: 0,
            jitter: 0.0,
        }));
    }

    /// The account's calendars to poll, as its calendar list now has them: those it had keep
    /// their turns and failures, those missing are no longer polled, and new ones are returned and
    /// taken as synchronized at `now`.
    pub fn update_account(
        &mut self,
        account_id: &str,
        calendars: &[Calendar],
        now: Instant,
    ) -> Vec<Calendar> {
        let turns: HashMap<String, Turn> = self
            .calendars
            .iter()
            .filter(|turn| turn.calendar.account_id == account_id)
            .map(|turn| (turn.calendar.id.clone(), turn.clone()))
            .collect();
        self.forget(account_id);
        let mut added = Vec::new();
        for calendar in calendars {
            let turn = match turns.get(&calendar.id) {
                Some(turn) => Turn {
                    calendar: calendar.clone(),
                    ..turn.clone()
                },
                None => {
                    added.push(calendar.clone());
                    Turn {
                        calendar: calendar.clone(),
                        last: now,
                        failures: 0,
                        jitter: 0.0,
                    }
                }
            };
            self.calendars.push(turn);
        }
        added
    }

    /// The account is no longer polled.
    pub fn forget(&mut self, account_id: &str) {
        self.calendars
            .retain(|turn| turn.calendar.account_id != account_id);
    }

    /// The calendars whose time has come.
    pub fn due(&self, now: Instant) -> Vec<Calendar> {
        self.select(|turn| now >= self.next_turn(turn))
    }

    /// Every calendar: for "Refresh", after waking from sleep and when the network returns.
    pub fn all(&self) -> Vec<Calendar> {
        self.select(|_| true)
    }

    /// The calendars not synchronized within `ACTIVATION_GAP`: for activating the window. One
    /// that failed waits out its back-off.
    pub fn stale(&self, now: Instant) -> Vec<Calendar> {
        self.select(|turn| {
            now >= turn.last + ACTIVATION_GAP && (turn.failures == 0 || now >= self.next_turn(turn))
        })
    }

    /// Every account's calendar with this ID: for opening one of its events for editing.
    pub fn calendar(&self, id: &str) -> Vec<Calendar> {
        self.select(|turn| turn.calendar.id == id)
    }

    /// These calendars were synchronized, or tried, at `now`.
    pub fn synced(&mut self, calendars: &[Calendar], now: Instant) {
        for turn in &mut self.calendars {
            if calendars.iter().any(|c| same(c, &turn.calendar)) {
                turn.last = now;
            }
        }
    }

    /// The calendar's synchronization got through: it is polled at its interval again.
    pub fn succeeded(&mut self, calendar: &Calendar) {
        for turn in &mut self.calendars {
            if same(calendar, &turn.calendar) {
                turn.failures = 0;
            }
        }
    }

    /// The calendar's synchronization failed: it waits longer before each retry (`back_off`),
    /// `jitter` (from 0 to 1) adding up to a quarter.
    pub fn failed(&mut self, calendar: &Calendar, jitter: f64) {
        for turn in &mut self.calendars {
            if same(calendar, &turn.calendar) {
                turn.failures += 1;
                turn.jitter = jitter;
            }
        }
    }

    /// When the next calendar is due; `None` with nothing to poll.
    pub fn next(&self) -> Option<Instant> {
        self.calendars.iter().map(|turn| self.next_turn(turn)).min()
    }

    fn next_turn(&self, turn: &Turn) -> Instant {
        let interval = interval(turn.calendar.access, self.shown);
        let wait = match turn.failures {
            0 => interval,
            failures => back_off(interval, failures, turn.jitter),
        };
        turn.last + wait
    }

    fn select(&self, pick: impl Fn(&Turn) -> bool) -> Vec<Calendar> {
        self.calendars
            .iter()
            .filter(|turn| pick(turn))
            .map(|turn| turn.calendar.clone())
            .collect()
    }
}

/// Whether both are the same calendar of the same account.
fn same(a: &Calendar, b: &Calendar) -> bool {
    a.account_id == b.account_id && a.id == b.id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Rgb;
    use chrono_tz::Europe::Warsaw;

    fn calendar(account_id: &str, id: &str, access: Access) -> Calendar {
        Calendar {
            id: id.into(),
            account_id: account_id.into(),
            name: id.into(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access,
            zone: Warsaw,
        }
    }

    fn ids(calendars: Vec<Calendar>) -> Vec<String> {
        calendars.into_iter().map(|c| c.id).collect()
    }

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    /// One editable and one read-only calendar of account "1", synchronized at `start`.
    fn two_calendars(start: Instant) -> Schedule {
        let mut schedule = Schedule::new();
        schedule.set_account(
            "1",
            &[
                calendar("1", "work", Access::Writer),
                calendar("1", "holidays", Access::Reader),
            ],
            start,
        );
        schedule
    }

    #[test]
    fn editable_calendars_are_due_every_minute_and_read_only_ones_every_15_minutes() {
        let start = Instant::now();
        let mut schedule = two_calendars(start);

        assert!(schedule.due(start + secs(59)).is_empty());
        assert_eq!(schedule.next(), Some(start + secs(60)));
        assert_eq!(ids(schedule.due(start + secs(60))), ["work"]);
        schedule.synced(&schedule.due(start + secs(60)), start + secs(60));
        assert_eq!(schedule.next(), Some(start + secs(120)));

        assert_eq!(
            ids(schedule.due(start + secs(15 * 60))),
            ["work", "holidays"]
        );
    }

    #[test]
    fn an_updated_calendar_list_keeps_turns_adds_new_calendars_and_drops_those_gone() {
        let start = Instant::now();
        let mut schedule = two_calendars(start);
        schedule.set_account("2", &[calendar("2", "other", Access::Owner)], start);

        // "work" was dropped to read-only, "holidays" is gone and "team" is new.
        let added = schedule.update_account(
            "1",
            &[
                calendar("1", "work", Access::Reader),
                calendar("1", "team", Access::Writer),
            ],
            start + secs(30),
        );

        assert_eq!(ids(added), ["team"]);
        assert_eq!(ids(schedule.all()), ["other", "work", "team"]);
        assert_eq!(ids(schedule.due(start + secs(60))), ["other"]);
        assert_eq!(ids(schedule.due(start + secs(90))), ["other", "team"]);
        assert_eq!(
            ids(schedule.due(start + secs(15 * 60))),
            ["other", "work", "team"]
        );
    }

    #[test]
    fn with_the_window_hidden_editable_calendars_are_due_at_least_every_5_minutes() {
        let start = Instant::now();
        let mut schedule = two_calendars(start);

        schedule.set_shown(false);

        assert!(schedule.due(start + secs(60)).is_empty());
        assert_eq!(ids(schedule.due(start + secs(5 * 60))), ["work"]);
        assert_eq!(
            ids(schedule.due(start + secs(30 * 60))),
            ["work", "holidays"]
        );
        // Shown again, what waited longer than its shown interval is due at once.
        schedule.set_shown(true);
        assert_eq!(ids(schedule.due(start + secs(90))), ["work"]);
    }

    #[test]
    fn refresh_takes_every_calendar_and_activation_those_not_just_synchronized() {
        let start = Instant::now();
        let mut schedule = two_calendars(start);
        schedule.synced(&[calendar("1", "work", Access::Writer)], start + secs(30));

        assert_eq!(ids(schedule.all()), ["work", "holidays"]);
        assert_eq!(ids(schedule.stale(start + secs(35))), ["holidays"]);
        assert_eq!(ids(schedule.stale(start + secs(40))), ["work", "holidays"]);
    }

    #[test]
    fn opening_an_event_takes_its_calendar_on_every_account() {
        let start = Instant::now();
        let mut schedule = two_calendars(start);
        schedule.set_account("2", &[calendar("2", "holidays", Access::Reader)], start);

        let both = schedule.calendar("holidays");

        assert_eq!(
            both.iter()
                .map(|c| c.account_id.as_str())
                .collect::<Vec<_>>(),
            ["1", "2"]
        );
    }

    #[test]
    fn a_failing_calendar_waits_longer_after_each_failure_up_to_30_minutes() {
        let start = Instant::now();
        let mut schedule = Schedule::new();
        let work = calendar("1", "work", Access::Writer);
        schedule.set_account("1", std::slice::from_ref(&work), start);
        let mut now = start;
        let mut waits = Vec::new();
        for _ in 0..7 {
            schedule.synced(std::slice::from_ref(&work), now);
            schedule.failed(&work, 0.0);
            let next = schedule.next().unwrap();
            waits.push((next - now).as_secs() / 60);
            assert!(schedule.due(next - secs(1)).is_empty());
            assert_eq!(ids(schedule.due(next)), ["work"]);
            now = next;
        }

        assert_eq!(waits, [1, 2, 4, 8, 16, 30, 30]);

        // Once it gets through, it is polled every minute again.
        schedule.synced(std::slice::from_ref(&work), now);
        schedule.succeeded(&work);
        assert_eq!(schedule.next(), Some(now + secs(60)));
    }

    #[test]
    fn the_wait_after_a_failure_has_up_to_a_quarter_of_jitter() {
        let minute = secs(60);

        assert_eq!(back_off(minute, 1, 0.0), secs(60));
        assert_eq!(
            back_off(minute, 1, 0.5),
            secs(67) + Duration::from_millis(500)
        );
        assert_eq!(back_off(minute, 3, 1.0), secs(300));
        assert_eq!(back_off(secs(15 * 60), 3, 0.0), MAX_BACK_OFF);
        assert_eq!(back_off(minute, 40, 0.0), MAX_BACK_OFF);
    }

    #[test]
    fn activating_the_window_does_not_cut_a_back_off_short_but_refresh_does() {
        let start = Instant::now();
        let mut schedule = two_calendars(start);
        let work = calendar("1", "work", Access::Writer);
        schedule.failed(&work, 0.0);
        schedule.failed(&work, 0.0);

        assert_eq!(ids(schedule.stale(start + secs(60))), ["holidays"]);
        assert_eq!(ids(schedule.stale(start + secs(120))), ["work", "holidays"]);
        assert_eq!(ids(schedule.all()), ["work", "holidays"]);
    }

    #[test]
    fn an_updated_calendar_list_keeps_a_calendars_failures() {
        let start = Instant::now();
        let mut schedule = two_calendars(start);
        let work = calendar("1", "work", Access::Writer);
        schedule.failed(&work, 0.0);
        schedule.failed(&work, 0.0);

        schedule.update_account("1", &[work], start + secs(30));

        assert_eq!(schedule.next(), Some(start + secs(120)));
    }

    #[test]
    fn a_forgotten_account_is_no_longer_polled() {
        let start = Instant::now();
        let mut schedule = two_calendars(start);

        schedule.forget("1");

        assert!(schedule.all().is_empty());
        assert_eq!(schedule.next(), None);
    }

    /// Google's default limits for the Calendar API: requests per minute per user, and per
    /// project.
    const PER_USER_PER_MINUTE: usize = 600;
    const PER_PROJECT_PER_MINUTE: usize = 10_000;

    /// The busiest minute of an hour, per account and in all, when the schedule is followed
    /// second by second and the window is activated as `activations` says.
    fn busiest_minutes(
        schedule: &mut Schedule,
        activations: impl Fn(u64) -> bool,
    ) -> (usize, usize) {
        let start = Instant::now();
        let mut requests: Vec<(u64, String)> = Vec::new();
        for second in 0..3600 {
            let now = start + secs(second);
            let mut round = schedule.due(now);
            if activations(second) {
                round = schedule.stale(now);
            }
            requests.extend(round.iter().map(|c| (second, c.account_id.clone())));
            schedule.synced(&round, now);
        }
        let mut per_user = 0;
        let mut per_project = 0;
        for from in 0..3600 - 60 {
            let minute: Vec<_> = requests
                .iter()
                .filter(|(at, _)| (from..from + 60).contains(at))
                .collect();
            let mut accounts: HashMap<&str, usize> = HashMap::new();
            for (_, account) in &minute {
                *accounts.entry(account).or_default() += 1;
            }
            per_user = per_user.max(accounts.into_values().max().unwrap_or(0));
            per_project = per_project.max(minute.len());
        }
        (per_user, per_project)
    }

    /// `MOD-02-NFR-01`: 5 accounts and 50 calendars, half of them editable.
    #[test]
    fn polling_5_accounts_and_50_calendars_stays_within_googles_limits() {
        let mut schedule = Schedule::new();
        let start = Instant::now();
        for account in ["a1", "a2", "a3", "a4", "a5"] {
            let calendars: Vec<_> = (0..10)
                .map(|n| {
                    let access = if n % 2 == 0 {
                        Access::Owner
                    } else {
                        Access::Reader
                    };
                    calendar(account, &format!("{account}-{n}"), access)
                })
                .collect();
            schedule.set_account(account, &calendars, start);
        }

        let (per_user, per_project) = busiest_minutes(&mut schedule, |_| false);
        // Each account: 5 editable calendars every minute, 5 read-only ones every 15 minutes.
        assert!(per_user <= 10, "{per_user} requests in a minute");
        assert!(per_project <= 50, "{per_project} requests in a minute");

        // Even when the window is activated every second, half of Google's limits stays free.
        let (per_user, per_project) = busiest_minutes(&mut schedule, |_| true);
        assert!(per_user <= PER_USER_PER_MINUTE / 2, "{per_user} per user");
        assert!(
            per_project <= PER_PROJECT_PER_MINUTE / 2,
            "{per_project} in all"
        );
    }

    /// The worst case of the profile: all 50 calendars on one account, editable, and the window
    /// activated every second.
    #[test]
    fn one_account_with_50_editable_calendars_stays_within_the_per_user_limit() {
        let mut schedule = Schedule::new();
        let calendars: Vec<_> = (0..50)
            .map(|n| calendar("a1", &format!("c{n}"), Access::Owner))
            .collect();
        schedule.set_account("a1", &calendars, Instant::now());

        let (per_user, _) = busiest_minutes(&mut schedule, |_| true);

        assert!(per_user <= PER_USER_PER_MINUTE / 2, "{per_user} per user");
    }
}
