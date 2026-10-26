//! "Meet with…" in the side panel (`US-06-09`): people are suggested as guests are and chosen,
//! a typed address too (`MOD-06-F23`), and listed under the field, each in a colour of their own
//! (`MOD-06-F24`, `U06`). Their calendars, as far as they are shared with the account, are laid
//! over the user's own in the day, 3-day and week views: events with their details, or busy
//! blocks, read-only (`MOD-06-F25`, `U05`). **Create event** starts a meeting with them
//! (`MOD-06-F26`). They are held in memory only, and offline nothing of them is shown
//! (`MOD-06-F27`).

use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;

use crate::availability::Span;
use crate::colors::CALENDAR_COLORS;
use crate::create::QuickEntry;
use crate::editor::Editor;
use crate::guests::{self, Guest, Person};
use crate::model::{About, Event, Rgb, Timing};
use crate::regional::Regional;
use crate::sync::SyncedEvent;
use crate::texts;
use crate::view_model::{DayColumn, GridViewModel, Whose, grid_view};
use crate::week::{GridView, day_start};

/// A person chosen in "Meet with…".
#[derive(Clone, Debug, PartialEq)]
pub struct Chosen {
    pub person: Person,
    /// The account their calendar is read through: the default calendar's (`DEC-175`).
    pub account_id: String,
    /// Their colour in the list and on their events (`MOD-06-U06`).
    pub color: Rgb,
}

impl Chosen {
    /// Their name, or their address when they have none.
    pub fn name(&self) -> &str {
        if self.person.name.is_empty() {
            &self.person.email
        } else {
            &self.person.name
        }
    }
}

/// The side panel's "Meet with…": what is typed in its field and the people chosen, never kept
/// across restarts (`MOD-06-F27`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeetWith {
    pub field: String,
    pub chosen: Vec<Chosen>,
}

impl MeetWith {
    /// Chooses the person, their calendar read through the account, in the first colour no one
    /// chosen has (`DEC-176`); the user's own addresses and someone already chosen are ignored
    /// (MOD-06 *Constraints and edge cases*). The field is emptied.
    pub fn choose(&mut self, person: &Person, account_id: &str, own: &[String]) {
        let taken = (self.chosen.iter().map(|chosen| &chosen.person.email))
            .chain(own)
            .any(|email| guests::same(email, &person.email));
        if !taken {
            let color = (CALENDAR_COLORS.iter().map(|(_, color)| *color))
                .find(|color| self.chosen.iter().all(|chosen| chosen.color != *color))
                .unwrap_or(CALENDAR_COLORS[0].1);
            self.chosen.push(Chosen {
                person: person.clone(),
                account_id: account_id.to_string(),
                color,
            });
        }
        self.field.clear();
    }

    /// Enter in the field: the address typed when it is one, otherwise the first person
    /// suggested, as in the editor's guest field.
    pub fn enter(&mut self, suggested: &[Person], account_id: &str, own: &[String]) {
        let person = match guests::address(&self.field) {
            Some(email) => Person {
                name: String::new(),
                email,
            },
            None => match suggested.first() {
                Some(person) => person.clone(),
                None => return,
            },
        };
        self.choose(&person, account_id, own);
    }

    /// Takes the person out of the list, and their events out of the views.
    pub fn remove(&mut self, email: &str) {
        self.chosen.retain(|chosen| chosen.person.email != email);
    }

    /// Who is suggested for what is typed, from the people `found`, as guests are suggested
    /// (`MOD-06-F23`): but those chosen already and the user's `own` addresses.
    pub fn suggestions(&self, found: &[Person], own: &[String]) -> Vec<Person> {
        let chosen: Vec<Guest> = (self.chosen.iter())
            .map(|chosen| Guest::invited(&chosen.person))
            .collect();
        guests::suggestions(&self.field, found, &chosen, own)
    }

    /// What the provider is asked for the days a grid view shows, `dates`: each person's calendar
    /// over them. `None` without people, offline (`MOD-06-F27`), and in the month and agenda
    /// views, which show no one else's calendars (`DEC-173`), as `dates` is then `None`.
    pub fn asked(&self, dates: Option<&[NaiveDate]>, zone: Tz, online: bool) -> Option<Asked> {
        let (first, last) = (dates?.first()?, dates?.last()?);
        if !online || self.chosen.is_empty() {
            return None;
        }
        Some(Asked {
            people: (self.chosen.iter())
                .map(|chosen| (chosen.account_id.clone(), chosen.person.email.clone()))
                .collect(),
            span: (
                day_start(*first, zone),
                day_start(*last + Duration::days(1), zone),
            ),
        })
    }

    /// **Create event**: the full editor of the new event `entry`, started as the `c` key starts
    /// one, with the people chosen as its guests (`MOD-06-F26`, `DEC-174`).
    pub fn meeting(&self, entry: &QuickEntry, regional: &Regional) -> Editor {
        let mut editor = Editor::from_quick(entry, regional);
        for chosen in &self.chosen {
            editor.add_guest(&chosen.person, &[]);
        }
        editor
    }
}

/// What the provider is asked: the calendars of the people, each by the account it is read
/// through and the person's address, over the span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asked {
    pub people: Vec<(String, String)>,
    pub span: Span,
}

/// How much of a person's calendar is shared with the account, over the span asked.
#[derive(Clone, Debug, PartialEq)]
pub enum Shared {
    /// Its events, with their details; one without a title is a busy block.
    Events(Vec<SyncedEvent>),
    /// Only when the person is busy.
    Busy(Vec<Span>),
    /// Nothing: the calendar cannot be seen.
    Nothing,
}

/// What the provider answered, by the person's address; a person it could not answer for is not
/// in it.
pub type Answer = [(String, Shared)];

/// Why "Meet with…" cannot be used now: it needs a Google account (MOD-09 *Constraints and edge
/// cases*), and a connection to Google (`MOD-06-F27`); `None` when it can.
pub fn unavailable(accounts: bool, online: bool) -> Option<&'static str> {
    if !accounts {
        Some(texts::MEET_WITH_NEEDS_ACCOUNT)
    } else if !online {
        Some(texts::MEET_WITH_OFFLINE)
    } else {
        None
    }
}

/// Whether the person's calendar shares nothing with the account, as answered: marked so in
/// the list (`MOD-06-F25`).
pub fn cannot_be_seen(email: &str, answer: &Answer) -> bool {
    (answer.iter()).any(|(answered, shared)| {
        guests::same(answered, email) && matches!(shared, Shared::Nothing)
    })
}

/// The chosen people's calendars as the views show them: whom the user's column is headed by,
/// and each person with their events.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Overlay {
    pub user: String,
    pub people: Vec<(Whose, Vec<Event>)>,
}

impl Overlay {
    /// The `chosen` people with their events in `answer`, in their colours, read-only, at times
    /// in the views' `zone` (`MOD-06-F25`, `U06`): with their details where their calendar shares
    /// them; without a title, or from the busy times alone, "Busy" blocks. A person not answered
    /// for has none yet.
    pub fn of(user: &str, chosen: &[Chosen], answer: &Answer, zone: Tz) -> Overlay {
        let people = (chosen.iter())
            .map(|chosen| {
                let shared = (answer.iter())
                    .find(|(email, _)| guests::same(email, &chosen.person.email))
                    .map(|(_, shared)| shared);
                let whose = Whose::Person(chosen.name().to_string(), chosen.color);
                (whose, their_events(chosen, shared, zone))
            })
            .collect();
        Overlay {
            user: user.to_string(),
            people,
        }
    }

    /// The people's events, one person after another, as the views' indices count them after
    /// the user's.
    pub fn events(&self) -> impl Iterator<Item = &Event> {
        self.people.iter().flat_map(|(_, events)| events)
    }
}

/// The person's events as shared.
fn their_events(chosen: &Chosen, shared: Option<&Shared>, zone: Tz) -> Vec<Event> {
    let event = |id: String, timing: Timing| Event {
        id,
        account_id: chosen.account_id.clone(),
        calendar_id: chosen.person.email.clone(),
        title: String::new(),
        color: chosen.color,
        timing,
        busy_only: true,
        read_only: true,
        organizer_only: false,
        saving: false,
        recurring: false,
        series: None,
        recurrence: Vec::new(),
        series_first: None,
        about: About::default(),
        etag: String::new(),
    };
    let timed = |start: DateTime<Utc>, end: DateTime<Utc>| Timing::Timed { start, end, zone };
    match shared {
        Some(Shared::Events(events)) => (events.iter())
            .filter_map(|synced| {
                let timing = match synced.timing? {
                    Timing::Timed { start, end, .. } => timed(start, end),
                    all_day => all_day,
                };
                Some(Event {
                    title: synced.title.clone(),
                    busy_only: synced.title.is_empty(),
                    // In the person's colour, and not the user's to answer.
                    about: About {
                        color: None,
                        guests: (synced.about.guests.iter())
                            .map(|guest| Guest {
                                own: false,
                                ..guest.clone()
                            })
                            .collect(),
                        ..synced.about.clone()
                    },
                    ..event(synced.id.clone(), timing)
                })
            })
            .collect(),
        Some(Shared::Busy(busy)) => (busy.iter().enumerate())
            .map(|(n, &(start, end))| event(format!("busy-{n}"), timed(start, end)))
            .collect(),
        Some(Shared::Nothing) | None => Vec::new(),
    }
}

/// The grid view of `dates` with the people's calendars over the user's `events`
/// (`MOD-06-U05`): in the day view the user's column, headed by `overlay.user`, then a column
/// of each person's, headed by their name, their all-day events in their column of the strip;
/// in the 3-day and week views their events among the user's, laid out like overlapping
/// events. The events' indices count `events` first, then the people's as
/// `Overlay::events` gives them. The people's events carry no lock: it marks the user's own
/// calendars that cannot be changed (`MOD-02-U01`). Without people, the view of the user's events
/// alone.
pub fn grid(
    events: &[Event],
    overlay: &Overlay,
    view: GridView,
    dates: &[NaiveDate],
    zone: Tz,
    now: DateTime<Utc>,
) -> GridViewModel {
    if overlay.people.is_empty() {
        return grid_view(events, dates, zone, now);
    }
    let mut grid = if view == GridView::Day {
        columns(events, overlay, dates, zone, now)
    } else {
        let all: Vec<Event> = (events.iter().chain(overlay.events()).cloned()).collect();
        grid_view(&all, dates, zone, now)
    };
    let theirs = |event: usize| event >= events.len();
    for block in grid.blocks.iter_mut().filter(|b| theirs(b.placement.event)) {
        block.read_only = false;
    }
    for bar in grid.bars.iter_mut().filter(|b| theirs(b.bar.event)) {
        bar.read_only = false;
    }
    grid
}

/// The day view's columns: the user's, then each person's.
fn columns(
    events: &[Event],
    overlay: &Overlay,
    dates: &[NaiveDate],
    zone: Tz,
    now: DateTime<Utc>,
) -> GridViewModel {
    let mut grid = grid_view(events, dates, zone, now);
    for column in &mut grid.days {
        column.whose = Some(Whose::User(overlay.user.clone()));
    }
    let mut offset = events.len();
    for (whose, theirs) in &overlay.people {
        let day = grid.days.len();
        let column = grid_view(theirs, dates, zone, now);
        grid.days
            .extend(column.days.into_iter().map(|column| DayColumn {
                whose: Some(whose.clone()),
                ..column
            }));
        grid.blocks
            .extend(column.blocks.into_iter().map(|mut block| {
                block.placement.day += day;
                block.placement.event += offset;
                block
            }));
        grid.bars.extend(column.bars.into_iter().map(|mut bar| {
            bar.bar.first_day += day;
            bar.bar.last_day += day;
            bar.bar.event += offset;
            bar
        }));
        grid.lanes = grid.lanes.max(column.lanes);
        offset += theirs.len();
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{d, timed};
    use crate::model::{Access, Calendar};
    use crate::sync::SyncedEvent;
    use chrono::{TimeZone, Weekday};
    use chrono_tz::Europe::Warsaw;
    use pure_rust_locales::Locale;

    fn person(name: &str, email: &str) -> Person {
        Person {
            name: name.to_string(),
            email: email.to_string(),
        }
    }

    fn anna() -> Person {
        person("Anna Nowak", "anna.nowak@example.com")
    }

    fn adam() -> Person {
        person("Adam Kowalski", "adam@example.com")
    }

    /// Anna, Adam and Ewa's typed address chosen, as in `TC-06-14`.
    fn three_chosen() -> MeetWith {
        let mut meet = MeetWith::default();
        meet.choose(&anna(), "1", &[]);
        meet.choose(&adam(), "1", &[]);
        meet.field = " ewa@example.com ".to_string();
        meet.enter(&[], "1", &[]);
        meet
    }

    /// 13 November 2026 at the hour and minute, in Warsaw.
    fn at(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(2026, 11, day, hour, minute, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn synced(id: &str, title: &str, timing: Timing) -> SyncedEvent {
        SyncedEvent {
            id: id.to_string(),
            title: title.to_string(),
            timing: Some(timing),
            recurrence: Vec::new(),
            instance_of: None,
            about: About::default(),
            etag: "\"1\"".to_string(),
            organizer_only: false,
        }
    }

    /// Google's answer of `TC-06-14` for the week of 9 November: Anna's calendar with details,
    /// Adam's free/busy, nothing of Ewa's.
    fn tc_06_14_answer() -> Vec<(String, Shared)> {
        let timed = |start, end| Timing::Timed {
            start,
            end,
            zone: chrono_tz::UTC,
        };
        vec![
            (
                "anna.nowak@example.com".to_string(),
                Shared::Events(vec![
                    synced(
                        "budget",
                        "Budget review",
                        timed(at(13, 14, 30), at(13, 15, 30)),
                    ),
                    synced(
                        "conference",
                        "Conference",
                        Timing::AllDay {
                            start: d(2026, 11, 13),
                            end_exclusive: d(2026, 11, 14),
                        },
                    ),
                ]),
            ),
            (
                "adam@example.com".to_string(),
                Shared::Busy(vec![(at(12, 11, 0), at(12, 12, 0))]),
            ),
            ("ewa@example.com".to_string(), Shared::Nothing),
        ]
    }

    fn clock() -> DateTime<Utc> {
        at(12, 9, 0)
    }

    fn week() -> Vec<NaiveDate> {
        GridView::Week.days(d(2026, 11, 12), Weekday::Mon)
    }

    #[test]
    fn tc_06_14_chosen_people_are_listed_each_in_a_colour_of_their_own() {
        let meet = three_chosen();

        let listed: Vec<&str> = meet.chosen.iter().map(Chosen::name).collect();
        assert_eq!(listed, ["Anna Nowak", "Adam Kowalski", "ewa@example.com"]);
        let colors: Vec<Rgb> = meet.chosen.iter().map(|chosen| chosen.color).collect();
        assert_eq!(
            colors,
            CALENDAR_COLORS[..3]
                .iter()
                .map(|(_, color)| *color)
                .collect::<Vec<_>>()
        );
        assert_eq!(meet.field, "");
    }

    #[test]
    fn tc_06_14_typing_part_of_a_name_suggests_the_person_but_those_chosen_and_the_user() {
        let found = [anna(), adam(), person("Anna Owner", "owner@example.com")];
        let mut meet = MeetWith {
            field: "Ann".to_string(),
            ..MeetWith::default()
        };
        let own = ["owner@example.com".to_string()];

        assert_eq!(meet.suggestions(&found, &own), [anna()]);

        meet.choose(&anna(), "1", &own);
        meet.field = "Ann".to_string();
        assert!(meet.suggestions(&found, &own).is_empty());
    }

    #[test]
    fn choosing_the_user_or_someone_chosen_already_is_ignored() {
        let mut meet = MeetWith::default();
        let own = ["owner@example.com".to_string()];
        meet.choose(&anna(), "1", &own);
        meet.choose(&person("", "ANNA.NOWAK@example.com"), "1", &own);
        meet.choose(&person("Owner", "owner@example.com"), "1", &own);

        assert_eq!(meet.chosen.len(), 1);
    }

    #[test]
    fn enter_chooses_the_address_typed_or_else_the_first_suggested_and_nothing_else() {
        let mut meet = MeetWith {
            field: "An".to_string(),
            ..MeetWith::default()
        };
        meet.enter(&[anna()], "1", &[]);
        assert_eq!(meet.chosen[0].person, anna());

        meet.field = "nobody".to_string();
        meet.enter(&[], "1", &[]);
        assert_eq!(meet.chosen.len(), 1);
        assert_eq!(meet.field, "nobody");
    }

    #[test]
    fn a_person_removed_frees_their_colour_for_the_next() {
        let mut meet = three_chosen();
        meet.remove("adam@example.com");
        assert_eq!(meet.chosen.len(), 2);

        meet.choose(&person("", "jan@example.com"), "1", &[]);
        assert_eq!(meet.chosen[2].color, CALENDAR_COLORS[1].1);
    }

    #[test]
    fn tc_06_14_the_people_are_asked_for_the_days_shown_and_not_offline_or_in_other_views() {
        let meet = three_chosen();
        let week = week();

        let asked = meet.asked(Some(&week), Warsaw, true).unwrap();

        assert_eq!(
            asked.people,
            [
                ("1".to_string(), "anna.nowak@example.com".to_string()),
                ("1".to_string(), "adam@example.com".to_string()),
                ("1".to_string(), "ewa@example.com".to_string()),
            ]
        );
        assert_eq!(asked.span, (at(9, 0, 0), at(16, 0, 0)));
        // 7. The next week is asked for anew.
        let next = GridView::Week.days(d(2026, 11, 19), Weekday::Mon);
        let asked_next = meet.asked(Some(&next), Warsaw, true).unwrap();
        assert_eq!(asked_next.span, (at(16, 0, 0), at(23, 0, 0)));
        assert_eq!(meet.asked(Some(&week), Warsaw, false), None);
        assert_eq!(meet.asked(None, Warsaw, true), None);
        assert_eq!(MeetWith::default().asked(Some(&week), Warsaw, true), None);
    }

    #[test]
    fn tc_06_14_their_events_lie_beside_the_users_in_their_colours_with_details_or_busy() {
        let meet = three_chosen();
        let [anna, adam, ewa] = [0, 1, 2].map(|i| meet.chosen[i].clone());
        let overlay = Overlay::of(
            "owner@example.com",
            &meet.chosen,
            &tc_06_14_answer(),
            Warsaw,
        );
        let own = [timed("Weekly sync", d(2026, 11, 13), (14, 0), (15, 0))];

        let view = grid(&own, &overlay, GridView::Week, &week(), Warsaw, clock());

        // 3. On Friday "Budget review" in Anna's colour beside "Weekly sync", like overlapping
        //    events; "Conference" in the all-day strip in Anna's colour.
        let block = |title: &str| {
            (view.blocks.iter())
                .find(|block| block.title.as_deref() == Some(title))
                .unwrap()
        };
        let (sync, budget) = (block("Weekly sync"), block("Budget review"));
        assert_eq!(budget.color, anna.color);
        assert_eq!((budget.placement.day, sync.placement.day), (4, 4));
        assert_eq!(
            (budget.placement.start_min, budget.placement.end_min),
            (870, 930)
        );
        assert_eq!(budget.placement.columns, 2);
        assert_ne!(budget.placement.column, sync.placement.column);
        assert!(!budget.movable && !budget.read_only);
        let conference = (view.bars.iter())
            .find(|bar| bar.title.as_deref() == Some("Conference"))
            .unwrap();
        assert_eq!(conference.color, anna.color);
        assert_eq!((conference.bar.first_day, conference.bar.last_day), (4, 4));

        // 4. On Thursday a "Busy" block 11:00–12:00 in Adam's colour, without a title.
        let busy: Vec<_> = (view.blocks.iter())
            .filter(|block| block.placement.day == 3)
            .collect();
        let [busy] = busy.as_slice() else {
            panic!("one busy block on Thursday");
        };
        assert_eq!(busy.title, None);
        assert_eq!(busy.color, adam.color);
        assert_eq!(
            (busy.placement.start_min, busy.placement.end_min),
            (660, 720)
        );

        // 5. Ewa's calendar cannot be seen, and nothing of hers is shown.
        let answer = tc_06_14_answer();
        assert!(cannot_be_seen(&ewa.person.email, &answer));
        assert!(!cannot_be_seen(&anna.person.email, &answer));
        assert!(!cannot_be_seen(&adam.person.email, &answer));
        assert!(overlay.people[2].1.is_empty());
        assert_eq!(view.blocks.len() + view.bars.len(), 4);
    }

    #[test]
    fn tc_06_14_a_persons_event_opens_read_only_and_answers_nothing_for_the_user() {
        let meet = three_chosen();
        let mut answer = tc_06_14_answer();
        let Shared::Events(events) = &mut answer[0].1 else {
            panic!("Anna's events");
        };
        // The user is invited to "Budget review", which has its own colour.
        events[0].about.color = Some(crate::colors::EventColor::Tomato);
        events[0].about.guests = vec![Guest {
            own: true,
            ..Guest::invited(&person("", "owner@example.com"))
        }];
        let overlay = Overlay::of("owner@example.com", &meet.chosen, &answer, Warsaw);
        let own = [timed("Weekly sync", d(2026, 11, 13), (14, 0), (15, 0))];
        let view = grid(&own, &overlay, GridView::Week, &week(), Warsaw, clock());

        // Opening a block reaches the person's event after the user's.
        let budget = (view.blocks.iter())
            .find(|block| block.title.as_deref() == Some("Budget review"))
            .unwrap();
        let theirs: Vec<&Event> = overlay.events().collect();
        let opened = theirs[budget.placement.event - own.len()];

        assert_eq!(opened.title, "Budget review");
        assert_eq!(
            opened.protected(),
            Some(crate::model::Protected::ReadOnlyCalendar)
        );
        assert_eq!(opened.shown_color(), meet.chosen[0].color);
        assert_eq!(crate::invitations::own_answer(opened), None);
        assert!(!opened.busy_only);
    }

    #[test]
    fn an_event_shared_without_a_title_is_busy() {
        let meet = three_chosen();
        let answer = [(
            "anna.nowak@example.com".to_string(),
            Shared::Events(vec![synced(
                "private",
                "",
                Timing::Timed {
                    start: at(13, 9, 0),
                    end: at(13, 10, 0),
                    zone: chrono_tz::UTC,
                },
            )]),
        )];

        let overlay = Overlay::of("owner@example.com", &meet.chosen, &answer, Warsaw);

        let [private] = overlay.people[0].1.as_slice() else {
            panic!("Anna's event");
        };
        assert!(private.busy_only);
        // At the views' times.
        assert!(matches!(private.timing, Timing::Timed { zone, .. } if zone == Warsaw));
    }

    #[test]
    fn tc_06_15_the_day_view_puts_the_user_and_each_person_in_columns_headed_by_name() {
        let mut meet = MeetWith::default();
        meet.choose(&anna(), "1", &[]);
        meet.choose(&adam(), "1", &[]);
        let timed = |start, end| Timing::Timed {
            start,
            end,
            zone: chrono_tz::UTC,
        };
        let answer = [
            (
                "anna.nowak@example.com".to_string(),
                Shared::Events(vec![
                    synced(
                        "budget",
                        "Budget review",
                        timed(at(13, 14, 30), at(13, 15, 30)),
                    ),
                    synced(
                        "conference",
                        "Conference",
                        Timing::AllDay {
                            start: d(2026, 11, 13),
                            end_exclusive: d(2026, 11, 14),
                        },
                    ),
                ]),
            ),
            (
                "adam@example.com".to_string(),
                Shared::Busy(vec![(at(13, 10, 0), at(13, 11, 0))]),
            ),
        ];
        let overlay = Overlay::of("owner@example.com", &meet.chosen, &answer, Warsaw);
        let own = [timed_sync()];
        let friday = [d(2026, 11, 13)];

        let view = grid(&own, &overlay, GridView::Day, &friday, Warsaw, clock());

        let heads: Vec<_> = view.days.iter().map(|day| day.whose.clone()).collect();
        assert_eq!(
            heads,
            [
                Some(Whose::User("owner@example.com".to_string())),
                Some(Whose::Person(
                    "Anna Nowak".to_string(),
                    meet.chosen[0].color
                )),
                Some(Whose::Person(
                    "Adam Kowalski".to_string(),
                    meet.chosen[1].color
                )),
            ]
        );
        assert!(view.days.iter().all(|day| day.date == friday[0]));
        let placed: Vec<(usize, Option<&str>, u32, u32, u32)> = (view.blocks.iter())
            .map(|block| {
                let p = block.placement;
                (
                    p.day,
                    block.title.as_deref(),
                    p.start_min,
                    p.end_min,
                    p.columns,
                )
            })
            .collect();
        assert_eq!(
            placed,
            [
                (0, Some("Weekly sync"), 840, 900, 1),
                (1, Some("Budget review"), 870, 930, 1),
                (2, None, 600, 660, 1),
            ]
        );
        // Anna's all-day event in her column of the strip; the indices count the user's events
        // first.
        let [conference] = view.bars.as_slice() else {
            panic!("one bar");
        };
        assert_eq!((conference.bar.first_day, conference.bar.last_day), (1, 1));
        let theirs: Vec<&Event> = overlay.events().collect();
        assert_eq!(theirs[conference.bar.event - 1].title, "Conference");
        assert_eq!(theirs[view.blocks[2].placement.event - 1].id, "busy-0");
        assert_eq!(view.lanes, 1);
    }

    /// The user's "Weekly sync" on Friday 13 November, 14:00–15:00.
    fn timed_sync() -> Event {
        timed("Weekly sync", d(2026, 11, 13), (14, 0), (15, 0))
    }

    #[test]
    fn the_three_day_view_lays_their_events_beside_the_users_as_the_week_does() {
        let meet = three_chosen();
        let overlay = Overlay::of(
            "owner@example.com",
            &meet.chosen,
            &tc_06_14_answer(),
            Warsaw,
        );
        let dates = GridView::ThreeDays.days(d(2026, 11, 12), Weekday::Mon);

        let view = grid(
            &[timed_sync()],
            &overlay,
            GridView::ThreeDays,
            &dates,
            Warsaw,
            clock(),
        );

        assert!(view.days.iter().all(|day| day.whose.is_none()));
        let friday: Vec<(Option<&str>, u32)> = (view.blocks.iter())
            .filter(|block| block.placement.day == 1)
            .map(|block| (block.title.as_deref(), block.placement.columns))
            .collect();
        assert_eq!(
            friday,
            [(Some("Weekly sync"), 2), (Some("Budget review"), 2)]
        );
    }

    #[test]
    fn without_people_the_views_show_the_users_events_alone() {
        let own = [timed_sync()];
        for view in [GridView::Day, GridView::Week] {
            let dates = view.days(d(2026, 11, 13), Weekday::Mon);
            assert_eq!(
                grid(&own, &Overlay::default(), view, &dates, Warsaw, clock()),
                grid_view(&own, &dates, Warsaw, clock())
            );
        }
    }

    #[test]
    fn tc_06_15_create_event_opens_the_editor_with_the_people_as_guests_at_the_next_full_hour() {
        let mut meet = MeetWith::default();
        meet.choose(&anna(), "1", &[]);
        meet.choose(&adam(), "1", &[]);
        let calendar = Calendar {
            id: "owner@example.com".to_string(),
            account_id: "1".to_string(),
            name: "Owner".to_string(),
            color: Rgb(0x9f, 0xe1, 0xe7),
            access: Access::Owner,
            zone: Warsaw,
            default_reminders: Vec::new(),
        };
        let regional = Regional::new(Locale::pl_PL);
        // Friday 13 November selected on Thursday at 9:00; the default length is 30 minutes.
        let times = crate::create::keyed(d(2026, 11, 13), clock(), 30, Warsaw);
        let entry = QuickEntry::new(times, Warsaw, Some(calendar.clone()));

        let editor = meet.meeting(&entry, &regional);

        let guests: Vec<&str> = editor.guests.iter().map(|g| g.email.as_str()).collect();
        assert_eq!(guests, ["anna.nowak@example.com", "adam@example.com"]);
        assert_eq!(editor.calendar, Some(calendar));
        let Ok(Timing::Timed { start, end, .. }) = editor.timing(&regional) else {
            panic!("a timed event");
        };
        assert_eq!((start, end), (at(13, 10, 0), at(13, 10, 30)));
        assert!(editor.opened.is_none());
    }

    #[test]
    fn tc_06_16_a_person_removed_leaves_the_views_and_the_others_stay() {
        let mut meet = MeetWith::default();
        meet.choose(&anna(), "1", &[]);
        meet.choose(&adam(), "1", &[]);
        let timed = |start, end| Timing::Timed {
            start,
            end,
            zone: chrono_tz::UTC,
        };
        let answer = [
            (
                "anna.nowak@example.com".to_string(),
                Shared::Events(vec![synced(
                    "a",
                    "Anna's",
                    timed(at(13, 9, 0), at(13, 10, 0)),
                )]),
            ),
            (
                "adam@example.com".to_string(),
                Shared::Events(vec![synced(
                    "b",
                    "Adam's",
                    timed(at(13, 11, 0), at(13, 12, 0)),
                )]),
            ),
        ];
        let titles = |meet: &MeetWith| -> Vec<String> {
            let overlay = Overlay::of("owner@example.com", &meet.chosen, &answer, Warsaw);
            let view = grid(&[], &overlay, GridView::Week, &week(), Warsaw, clock());
            (view.blocks.iter())
                .filter_map(|block| block.title.clone())
                .collect()
        };
        assert_eq!(titles(&meet), ["Anna's", "Adam's"]);

        // 1. Adam removed: his events leave, Anna's stay, from the answer already there.
        meet.remove("adam@example.com");
        assert_eq!(titles(&meet), ["Anna's"]);
    }
}
