//! Reminders as desktop notifications (`US-07-01`): each notification-type reminder set on an
//! event in Google, or its calendar's default one, is shown once at its time (`MOD-07-F01`,
//! `F02`, `F07`), with the event's title, start time and location (`MOD-07-U01`). E-mail
//! reminders are Google's to send (`MOD-07-F06`), and declined events remind of nothing
//! (`MOD-07-F19`). Reminders whose time passed before the app started are not shown; those that
//! fell while the computer slept are, on waking, if their event has not started (`MOD-07-F20`).
//! They are scheduled from the events the cache holds, so they work offline. A notification
//! offers Join for an event with Google Meet, Snooze for 5 or 10 minutes, and Dismiss
//! (`US-07-02`, `MOD-07-F09`, `F10`); a snooze is the app's own, Google is not told. All
//! notifications can be turned off in the settings, and a calendar's muted from the side panel
//! (`US-07-03`, `MOD-07-F11`, `F12`); neither changes the reminders in Google.

use std::collections::{BTreeSet, HashMap};

use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;

use crate::local;
use crate::model::{Calendar, Event, Method, Reminders, Timing};
use crate::regional::Regional;
use crate::settings::Settings;
use crate::texts;
use crate::week::day_start;

/// How long after its time a reminder is still shown once its event has started: a reminder at
/// the event's start is shown when the app gets to it, not only before.
const LATE: Duration = Duration::minutes(1);

/// The event a notification is of, which clicking it opens (`MOD-07-F08`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EventId {
    pub account_id: String,
    pub calendar_id: String,
    /// The event's ID, an occurrence's own.
    pub id: String,
}

/// A desktop notification of a reminder (`MOD-07-U01`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    pub title: String,
    /// When the event starts, in the system's formats: its time on the day, otherwise its day too.
    pub when: String,
    /// Where it takes place; empty when it does not say.
    pub location: String,
    /// The web page of the event's Google Meet call, which Join opens (`MOD-07-F09`).
    pub meet: Option<String>,
    /// The reminder it is of, which Snooze shows again.
    pub reminder: ReminderId,
}

/// What the user chooses on a notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Clicking it: opens its event in the app (`MOD-07-F08`).
    Open,
    /// Opens its event's Google Meet call in the browser (`MOD-07-F09`).
    Join,
    /// Shows it again in so many minutes (`MOD-07-F10`).
    Snooze(u32),
    /// Closes it.
    Dismiss,
}

/// The minutes a notification can be snoozed for (`MOD-07-F10`).
pub const SNOOZES: [u32; 2] = [5, 10];

impl Action {
    /// What its button says.
    pub fn label(self) -> String {
        match self {
            Action::Open => texts::OPEN.to_string(),
            Action::Join => texts::JOIN.to_string(),
            Action::Snooze(minutes) => texts::snooze(minutes),
            Action::Dismiss => texts::DISMISS.to_string(),
        }
    }
}

impl Notification {
    /// What it says under the title: when the event starts, and where on a line of its own.
    pub fn body(&self) -> String {
        if self.location.is_empty() {
            self.when.clone()
        } else {
            format!("{}\n{}", self.when, self.location)
        }
    }

    /// Its buttons: Join for an event with Google Meet, then Snooze for 5 and 10 minutes, and
    /// Dismiss.
    pub fn actions(&self) -> Vec<Action> {
        let join = self.meet.as_ref().map(|_| Action::Join);
        let snoozes = SNOOZES.map(Action::Snooze);
        (join.into_iter())
            .chain(snoozes)
            .chain([Action::Dismiss])
            .collect()
    }
}

/// The notifications not shown, as the settings say: all of them while notifications are off
/// (`MOD-07-F11`), or those of the muted calendars (`MOD-07-F12`). Their reminders still come and
/// go at their times, so turning notifications on again shows only those still to come.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Silenced {
    pub all: bool,
    /// By account ID and calendar ID.
    pub muted: BTreeSet<(String, String)>,
    /// The local calendars are hidden as a whole (`MOD-09-F22`).
    pub local: bool,
}

impl Silenced {
    pub fn of(settings: &Settings) -> Self {
        Silenced {
            all: settings.notifications_off,
            muted: settings.muted.clone(),
            local: settings.local_hidden,
        }
    }

    /// Whether the notification is not to be shown.
    pub fn silences(&self, notification: &Notification) -> bool {
        let event = &notification.reminder.event;
        self.all
            || (self.local && event.account_id == local::ACCOUNT_ID)
            || (self.muted).contains(&(event.account_id.clone(), event.calendar_id.clone()))
    }
}

/// One reminder of an event or occurrence, by what tells it apart (MOD-07 *Technical notes*): a
/// moved event's reminder is a new one.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReminderId {
    /// The event it is of, which clicking its notification opens (`MOD-07-F08`).
    pub event: EventId,
    start: DateTime<Utc>,
    minutes: u32,
}

/// A reminder to show.
#[derive(Clone, Debug)]
struct Alarm {
    at: DateTime<Utc>,
    key: ReminderId,
    title: String,
    location: String,
    meet: Option<String>,
    timing: Timing,
}

impl Alarm {
    /// Whether it is still to be shown at `now`, once its time has come.
    fn live(&self, now: DateTime<Utc>) -> bool {
        now < self.key.start || now < self.at + LATE
    }
}

/// The reminders to show, those shown in this run, and those snoozed.
pub struct Alarms {
    /// When the app started: reminders before are not shown.
    started: DateTime<Utc>,
    regional: Regional,
    /// The system's time zone, as of the events last given.
    zone: Tz,
    /// The reminders not shown yet, the earliest first.
    pending: Vec<Alarm>,
    /// The reminders shown, while their events have them, as of the events last given.
    delivered: HashMap<ReminderId, Alarm>,
    /// The reminders snoozed, at the times they are shown again, the earliest first; they go
    /// with their events' reminders.
    snoozed: Vec<Alarm>,
}

impl Alarms {
    pub fn new(started: DateTime<Utc>, regional: Regional, zone: Tz) -> Self {
        Alarms {
            started,
            regional,
            zone,
            pending: Vec::new(),
            delivered: HashMap::new(),
            snoozed: Vec::new(),
        }
    }

    /// Takes the reminders of the events as they are now, so a moved event's reminder moves with
    /// it and a deleted one's goes; `calendars` give the default reminders, and `zone` the day an
    /// all-day event starts.
    pub fn set(&mut self, calendars: &[Calendar], events: &[Event], zone: Tz, now: DateTime<Utc>) {
        self.zone = zone;
        let all: HashMap<ReminderId, Alarm> = events
            .iter()
            .filter(|event| !event.busy_only && !event.about.declined())
            .flat_map(|event| alarms(calendars, event, zone))
            .map(|alarm| (alarm.key.clone(), alarm))
            .collect();
        let delivered = std::mem::take(&mut self.delivered);
        self.delivered = (all.iter())
            .filter(|(key, _)| delivered.contains_key(key))
            .map(|(key, alarm)| (key.clone(), alarm.clone()))
            .collect();
        // A snoozed reminder shows its event as it is now.
        let snoozed = std::mem::take(&mut self.snoozed);
        self.snoozed = (snoozed.into_iter())
            .filter_map(|snoozed| {
                let alarm = all.get(&snoozed.key)?;
                Some(Alarm {
                    at: snoozed.at,
                    ..alarm.clone()
                })
            })
            .collect();
        let mut alarms: Vec<Alarm> = (all.into_values())
            .filter(|alarm| alarm.at >= self.started && (alarm.at > now || alarm.live(now)))
            .filter(|alarm| !self.delivered.contains_key(&alarm.key))
            .collect();
        alarms.sort_by_key(|alarm| alarm.at);
        self.pending = alarms;
    }

    /// Shows the reminder shown before again `minutes` after `now` (`MOD-07-F10`); nothing when
    /// its event no longer has it.
    pub fn snooze(&mut self, reminder: &ReminderId, minutes: u32, now: DateTime<Utc>) {
        let Some(alarm) = self.delivered.get(reminder) else {
            return;
        };
        let at = now + Duration::minutes(minutes.into());
        let snoozed = Alarm {
            at,
            ..alarm.clone()
        };
        let place = self.snoozed.partition_point(|alarm| alarm.at <= at);
        self.snoozed.insert(place, snoozed);
    }

    /// When the next reminder is due, snoozed ones too; `None` when none is to come.
    pub fn next(&self) -> Option<DateTime<Utc>> {
        let pending = self.pending.first().map(|alarm| alarm.at);
        let snoozed = self.snoozed.first().map(|alarm| alarm.at);
        pending.into_iter().chain(snoozed).min()
    }

    /// The notifications due at `now`, each given once, in the order of their times, and then
    /// the snoozed ones due again. Those whose time passed long ago, while the computer slept,
    /// only if their event has not started.
    pub fn due(&mut self, now: DateTime<Utc>) -> Vec<Notification> {
        let come = self.pending.partition_point(|alarm| alarm.at <= now);
        let mut shown = Vec::new();
        for alarm in self.pending.drain(..come) {
            if alarm.live(now) && !self.delivered.contains_key(&alarm.key) {
                self.delivered.insert(alarm.key.clone(), alarm.clone());
                shown.push(alarm);
            }
        }
        let come = self.snoozed.partition_point(|alarm| alarm.at <= now);
        shown.extend(self.snoozed.drain(..come).filter(|alarm| alarm.live(now)));
        (shown.into_iter())
            .map(|alarm| Notification {
                when: when(&alarm.timing, now, self.zone, &self.regional),
                title: alarm.title,
                location: alarm.location,
                meet: alarm.meet,
                reminder: alarm.key,
            })
            .collect()
    }
}

/// The event's notification-type reminders, its own or its calendar's defaults. An all-day event
/// starts at midnight in `zone`; it has no reminders by the defaults, as Google does not give
/// the defaults of all-day events.
fn alarms(calendars: &[Calendar], event: &Event, zone: Tz) -> Vec<Alarm> {
    let reminders = match (&event.about.reminders, event.timing) {
        (Reminders::Own(own), _) => own.as_slice(),
        (Reminders::Default, Timing::AllDay { .. }) => &[],
        (Reminders::Default, Timing::Timed { .. }) => calendars
            .iter()
            .find(|calendar| {
                calendar.account_id == event.account_id && calendar.id == event.calendar_id
            })
            .map_or(&[][..], |calendar| calendar.default_reminders.as_slice()),
    };
    let start = match event.timing {
        Timing::Timed { start, .. } => start,
        Timing::AllDay { start, .. } => day_start(start, zone),
    };
    let event_id = EventId {
        account_id: event.account_id.clone(),
        calendar_id: event.calendar_id.clone(),
        id: event.id.clone(),
    };
    (reminders.iter())
        .filter(|reminder| reminder.method == Method::Notification)
        .map(|reminder| Alarm {
            at: start - Duration::minutes(reminder.minutes.into()),
            key: ReminderId {
                event: event_id.clone(),
                start,
                minutes: reminder.minutes,
            },
            title: event.title.clone(),
            location: event.about.location.clone(),
            meet: event.about.meet.clone(),
            timing: event.timing,
        })
        .collect()
}

/// When the event starts, as of `now`: `10:00` today, `Friday 13 November, 10:00` on another day,
/// and the day alone for an all-day event.
fn when(timing: &Timing, now: DateTime<Utc>, zone: Tz, regional: &Regional) -> String {
    match *timing {
        Timing::Timed { start, .. } => {
            let start = start.with_timezone(&zone);
            let time = regional.time(start.time());
            if start.date_naive() == now.with_timezone(&zone).date_naive() {
                time
            } else {
                format!("{}, {time}", regional.weekday_day_month(start.date_naive()))
            }
        }
        Timing::AllDay { start, .. } => regional.weekday_day_month(start),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{all_day, d, timed};
    use crate::guests::{Guest, Response};
    use crate::model::{Access, Reminder, Rgb};
    use chrono::TimeZone;
    use chrono_tz::Europe::{London, Warsaw};
    use pure_rust_locales::Locale;

    /// Thursday 12 November 2026, at the time in Warsaw.
    fn at(h: u32, m: u32) -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(2026, 11, 12, h, m, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn regional() -> Regional {
        Regional::new(Locale::en_GB)
    }

    fn reminder(method: Method, minutes: u32) -> Reminder {
        Reminder { method, minutes }
    }

    /// The calendar "Work", with a 30-minute notification by default.
    fn work() -> Calendar {
        Calendar {
            id: "work".to_string(),
            account_id: "1".to_string(),
            name: "Work".to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
            default_reminders: vec![reminder(Method::Notification, 30)],
        }
    }

    /// The event at `from`–`until` on 12 November with its own reminders.
    fn event(title: &str, from: (u32, u32), until: (u32, u32), own: Vec<Reminder>) -> Event {
        let mut event = timed(title, d(2026, 11, 12), from, until);
        event.about.reminders = Reminders::Own(own);
        event
    }

    /// Reminders of the app started at 9:00, given `events` then.
    fn started(events: &[Event]) -> Alarms {
        let mut alarms = Alarms::new(at(9, 0), regional(), Warsaw);
        alarms.set(&[work()], events, Warsaw, at(9, 0));
        alarms
    }

    fn titles(notifications: &[Notification]) -> Vec<&str> {
        notifications.iter().map(|n| n.title.as_str()).collect()
    }

    #[test]
    fn tc_07_01_a_10_minute_reminder_gives_one_notification_with_title_start_and_location() {
        let mut review = event(
            "Review",
            (10, 0),
            (11, 0),
            vec![reminder(Method::Notification, 10)],
        );
        review.about.location = "Room 2".to_string();
        let mut alarms = started(std::slice::from_ref(&review));

        // 1. At 9:50 the notification is shown, with the title, the start and the place.
        assert_eq!(alarms.due(at(9, 49)), []);
        assert_eq!(alarms.next(), Some(at(9, 50)));
        assert_eq!(
            alarms.due(at(9, 50)),
            [Notification {
                title: "Review".to_string(),
                when: "10:00".to_string(),
                location: "Room 2".to_string(),
                meet: None,
                reminder: ReminderId {
                    event: EventId {
                        account_id: "1".to_string(),
                        calendar_id: "work".to_string(),
                        id: "Review".to_string(),
                    },
                    start: at(10, 0),
                    minutes: 10,
                },
            }]
        );

        // 2. Three more synchronizations bring the event again; nothing is shown again.
        for minute in [51, 52, 53] {
            alarms.set(
                &[work()],
                std::slice::from_ref(&review),
                Warsaw,
                at(9, minute),
            );
            assert_eq!(alarms.due(at(9, minute)), []);
        }
        assert_eq!(alarms.due(at(9, 55)), []);
        assert_eq!(alarms.next(), None);
    }

    /// The notification of "Review" at 10:00 in Room 2, reminded of 10 minutes before.
    fn review_notification() -> Notification {
        Notification {
            title: "Review".to_string(),
            when: "10:00".to_string(),
            location: "Room 2".to_string(),
            meet: None,
            reminder: ReminderId {
                event: EventId {
                    account_id: "1".to_string(),
                    calendar_id: "work".to_string(),
                    id: "review".to_string(),
                },
                start: at(10, 0),
                minutes: 10,
            },
        }
    }

    #[test]
    fn a_notification_says_when_the_event_starts_and_where_under_its_title() {
        let mut notification = review_notification();
        assert_eq!(notification.body(), "10:00\nRoom 2");
        notification.location.clear();
        assert_eq!(notification.body(), "10:00");
    }

    #[test]
    fn notifications_off_silence_every_one_and_a_muted_calendar_only_its_own() {
        let review = review_notification();
        let mut settings = Settings::default();
        assert!(!Silenced::of(&settings).silences(&review));

        settings.notifications_off = true;
        assert!(Silenced::of(&settings).silences(&review));

        settings.notifications_off = false;
        settings.muted.insert(("1".to_string(), "team".to_string()));
        assert!(!Silenced::of(&settings).silences(&review));
        settings.muted.insert(("1".to_string(), "work".to_string()));
        assert!(Silenced::of(&settings).silences(&review));
    }

    #[test]
    fn tc_07_03_a_notification_offers_dismiss_and_snooze_for_5_or_10_minutes() {
        let notification = review_notification();
        let actions = notification.actions();
        assert_eq!(
            actions,
            [Action::Snooze(5), Action::Snooze(10), Action::Dismiss]
        );
        let labels: Vec<String> = actions.into_iter().map(Action::label).collect();
        assert_eq!(labels, ["Snooze 5 minutes", "Snooze 10 minutes", "Dismiss"]);
    }

    #[test]
    fn a_notification_of_an_event_with_google_meet_offers_join_first() {
        let mut notification = review_notification();
        notification.meet = Some("https://meet.google.com/abc-defg-hij".to_string());
        assert_eq!(
            notification.actions(),
            [
                Action::Join,
                Action::Snooze(5),
                Action::Snooze(10),
                Action::Dismiss
            ]
        );
        assert_eq!(Action::Join.label(), "Join");
    }

    #[test]
    fn tc_07_03_a_snoozed_notification_is_shown_again_after_the_chosen_minutes() {
        let mut review = event(
            "Review",
            (10, 0),
            (11, 0),
            vec![reminder(Method::Notification, 10)],
        );
        review.about.meet = Some("https://meet.google.com/abc-defg-hij".to_string());
        let mut alarms = started(std::slice::from_ref(&review));
        let shown = alarms.due(at(9, 50));
        assert_eq!(titles(&shown), ["Review"]);

        alarms.snooze(&shown[0].reminder, 5, at(9, 50));

        assert_eq!(alarms.next(), Some(at(9, 55)));
        assert_eq!(alarms.due(at(9, 54)), []);
        let again = alarms.due(at(9, 55));
        assert_eq!(again, shown);
        assert_eq!(alarms.next(), None);
        assert_eq!(alarms.due(at(10, 0)), []);
    }

    #[test]
    fn a_snooze_outlives_synchronization_follows_the_event_and_can_be_repeated() {
        let reminders = vec![reminder(Method::Notification, 0)];
        let call = event("Call", (10, 0), (10, 30), reminders.clone());
        let mut alarms = started(std::slice::from_ref(&call));
        let shown = alarms.due(at(10, 0));

        alarms.snooze(&shown[0].reminder, 10, at(10, 0));
        // Synchronization brings the call renamed after it started.
        let mut renamed = event("Call with Ann", (10, 0), (10, 30), reminders);
        renamed.id = call.id.clone();
        alarms.set(&[work()], std::slice::from_ref(&renamed), Warsaw, at(10, 5));

        // Shown again after the event started, as it was snoozed past its start.
        let again = alarms.due(at(10, 10));
        assert_eq!(titles(&again), ["Call with Ann"]);
        alarms.snooze(&again[0].reminder, 5, at(10, 11));
        assert_eq!(titles(&alarms.due(at(10, 16))), ["Call with Ann"]);
    }

    #[test]
    fn a_snooze_of_a_deleted_or_moved_events_reminder_goes_with_it() {
        let reminders = vec![reminder(Method::Notification, 10)];
        let review = event("Review", (10, 0), (11, 0), reminders.clone());
        let standup = event("Standup", (10, 0), (10, 15), reminders.clone());
        let mut alarms = started(&[review, standup]);
        let shown = alarms.due(at(9, 50));
        for notification in &shown {
            alarms.snooze(&notification.reminder, 5, at(9, 50));
        }

        // Synchronization brings Review moved to 11:00 and Standup deleted.
        let moved = event("Review", (11, 0), (12, 0), reminders);
        alarms.set(&[work()], std::slice::from_ref(&moved), Warsaw, at(9, 52));

        assert_eq!(alarms.due(at(9, 55)), []);
        assert_eq!(alarms.next(), Some(at(10, 50)));
    }

    #[test]
    fn tc_07_02_declined_events_and_e_mail_reminders_give_no_notification() {
        let mut offsite = event(
            "Offsite",
            (10, 0),
            (11, 0),
            vec![reminder(Method::Notification, 10)],
        );
        offsite.about.guests = vec![Guest {
            email: "me@example.com".to_string(),
            name: String::new(),
            optional: false,
            response: Response::Declined,
            own: true,
        }];
        let report = event(
            "Report",
            (10, 0),
            (11, 0),
            vec![reminder(Method::Email, 10)],
        );
        let mut alarms = started(&[offsite, report]);

        assert_eq!(alarms.due(at(9, 55)), []);
        assert_eq!(alarms.next(), None);
    }

    #[test]
    fn an_event_using_the_defaults_gets_its_calendars_and_one_of_its_own_only_its_own() {
        // Google gives "Planning" the calendar's defaults: 30 minutes before.
        let planning = timed("Planning", d(2026, 11, 12), (10, 0), (11, 0));
        let quiet = event("Quiet", (10, 0), (11, 0), Vec::new());
        let mut alarms = started(&[planning, quiet]);

        assert_eq!(titles(&alarms.due(at(9, 30))), ["Planning"]);
        assert_eq!(alarms.next(), None);
    }

    #[test]
    fn a_reminder_whose_time_passed_before_the_app_started_is_not_shown() {
        let review = event(
            "Review",
            (9, 10),
            (10, 0),
            vec![
                reminder(Method::Notification, 15),
                reminder(Method::Notification, 5),
            ],
        );
        let mut alarms = started(&[review]);

        // 8:55 was before the start at 9:00; 9:05 comes.
        assert_eq!(alarms.due(at(9, 1)), []);
        assert_eq!(titles(&alarms.due(at(9, 5))), ["Review"]);
    }

    #[test]
    fn reminders_that_fell_during_sleep_are_shown_on_waking_if_their_event_has_not_started() {
        let review = event(
            "Review",
            (10, 0),
            (11, 0),
            vec![reminder(Method::Notification, 10)],
        );
        let standup = event(
            "Standup",
            (9, 30),
            (9, 45),
            vec![reminder(Method::Notification, 10)],
        );
        let mut alarms = started(&[review, standup]);

        // Asleep from 9:15 to 9:55: Review has not started, Standup has.
        assert_eq!(titles(&alarms.due(at(9, 55))), ["Review"]);
        assert_eq!(alarms.next(), None);
    }

    #[test]
    fn a_reminder_at_the_start_is_shown_as_its_event_starts() {
        let call = event(
            "Call",
            (10, 0),
            (10, 30),
            vec![reminder(Method::Notification, 0)],
        );
        let mut alarms = started(&[call]);

        let late = at(10, 0) + Duration::seconds(2);
        assert_eq!(titles(&alarms.due(late)), ["Call"]);
    }

    #[test]
    fn a_moved_events_reminder_moves_with_it_and_a_deleted_ones_goes() {
        let reminders = vec![reminder(Method::Notification, 10)];
        let review = event("Review", (10, 0), (11, 0), reminders.clone());
        let standup = event("Standup", (10, 0), (10, 15), reminders.clone());
        let mut alarms = started(&[review, standup]);

        // Synchronization brings Review moved to 11:00 and Standup deleted.
        let moved = event("Review", (11, 0), (12, 0), reminders);
        alarms.set(&[work()], std::slice::from_ref(&moved), Warsaw, at(9, 20));

        assert_eq!(alarms.due(at(9, 50)), []);
        assert_eq!(alarms.next(), Some(at(10, 50)));
        let shown = alarms.due(at(10, 50));
        assert_eq!(titles(&shown), ["Review"]);
        assert_eq!(shown[0].when, "11:00");
    }

    #[test]
    fn several_reminders_in_the_same_minute_each_give_a_notification() {
        let reminders = vec![reminder(Method::Notification, 10)];
        let mut alarms = started(&[
            event("Review", (10, 0), (11, 0), reminders.clone()),
            event("Lunch", (10, 0), (10, 30), reminders.clone()),
            event("Call", (10, 0), (10, 30), reminders),
        ]);

        let due = alarms.due(at(9, 50));
        let mut shown = titles(&due);
        shown.sort();
        assert_eq!(shown, ["Call", "Lunch", "Review"]);
    }

    #[test]
    fn an_occurrence_of_a_series_has_its_own_reminder() {
        let reminders = vec![reminder(Method::Notification, 10)];
        let mut monday = event("Standup", (10, 0), (10, 15), reminders.clone());
        monday.id = "standup_20261112T090000Z".to_string();
        let mut tomorrow = timed("Standup", d(2026, 11, 13), (10, 0), (10, 15));
        tomorrow.id = "standup_20261113T090000Z".to_string();
        tomorrow.about.reminders = Reminders::Own(reminders);
        let mut alarms = started(&[monday, tomorrow]);

        assert_eq!(titles(&alarms.due(at(9, 50))), ["Standup"]);
        let next = alarms.due(at(9, 50) + Duration::days(1));
        assert_eq!(next[0].reminder.event.id, "standup_20261113T090000Z");
        assert_eq!(next[0].when, "10:00");
    }

    #[test]
    fn a_reminder_a_day_ahead_says_the_day_and_an_all_day_events_own_reminder_counts_from_midnight()
    {
        let mut trip = all_day("Trip", d(2026, 11, 13), 2);
        // The day before at 17:00.
        trip.about.reminders = Reminders::Own(vec![reminder(Method::Notification, 7 * 60)]);
        let mut board = timed("Board", d(2026, 11, 13), (10, 0), (11, 0));
        board.about.reminders = Reminders::Own(vec![reminder(Method::Notification, 24 * 60)]);
        // An all-day event by the defaults has none: Google does not give them.
        let holiday = all_day("Holiday", d(2026, 11, 13), 1);
        let mut alarms = started(&[trip, board, holiday]);

        let board = alarms.due(at(10, 0));
        assert_eq!(titles(&board), ["Board"]);
        assert_eq!(board[0].when, "Friday 13 November, 10:00");
        let trip = alarms.due(at(17, 0));
        assert_eq!(titles(&trip), ["Trip"]);
        assert_eq!(trip[0].when, "Friday 13 November");
        assert_eq!(alarms.next(), None);
    }

    #[test]
    fn a_new_system_time_zone_keeps_reminders_at_the_events_real_times() {
        let review = event(
            "Review",
            (10, 0),
            (11, 0),
            vec![reminder(Method::Notification, 10)],
        );
        let mut alarms = started(std::slice::from_ref(&review));

        alarms.set(&[work()], std::slice::from_ref(&review), London, at(9, 10));

        assert_eq!(alarms.next(), Some(at(9, 50)));
        // 10:00 in Warsaw is 9:00 in London.
        assert_eq!(alarms.due(at(9, 50))[0].when, "09:00");
    }

    #[test]
    fn free_busy_events_remind_of_nothing() {
        let mut busy = event(
            "Busy",
            (10, 0),
            (11, 0),
            vec![reminder(Method::Notification, 10)],
        );
        busy.busy_only = true;
        let mut alarms = started(&[busy]);

        assert_eq!(alarms.due(at(9, 50)), []);
    }
}
