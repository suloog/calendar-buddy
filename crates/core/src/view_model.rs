//! What the day, 3-day and week views show, independent of how it is drawn.

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use crate::layout::{AllDayBar, TimedPlacement, all_day_bars, timed_placements};
use crate::model::{Event, Kind, Rgb, Timing};
use crate::week::wall_minute;

#[derive(Clone, Debug, PartialEq)]
pub struct DayColumn {
    pub date: NaiveDate,
    pub is_today: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub placement: TimedPlacement,
    /// `None` for events of calendars shared as free/busy only.
    pub title: Option<String>,
    pub color: Rgb,
    /// The event's calendar cannot be changed (`MOD-02-U01`).
    pub read_only: bool,
    /// Google has not confirmed the new event yet (`MOD-05-U03`).
    pub saving: bool,
    /// The event can be dragged and resized (`MOD-05-F13`, `MOD-05-F14`).
    pub movable: bool,
    /// An invitation awaiting the user's answer, drawn with a dashed outline and a light fill
    /// (`MOD-06-U04`).
    pub awaiting: bool,
    /// An event the user declined, drawn struck through and faded (`MOD-06-F14`).
    pub declined: bool,
    /// An out-of-office event, drawn hatched (`MOD-05-U01`).
    pub hatched: bool,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub bar: AllDayBar,
    pub title: Option<String>,
    pub color: Rgb,
    pub read_only: bool,
    pub saving: bool,
    pub movable: bool,
    /// An invitation awaiting the user's answer, drawn with a dashed outline and a light fill
    /// (`MOD-06-U04`).
    pub awaiting: bool,
    /// An event the user declined, drawn struck through and faded (`MOD-06-F14`).
    pub declined: bool,
    /// An all-day event, not a timed one lasting a day or longer.
    pub all_day: bool,
    /// An out-of-office event, drawn hatched (`MOD-05-U01`).
    pub hatched: bool,
    /// A working location or a birthday, drawn as a label rather than a bar (`MOD-05-U01`).
    pub label: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NowLine {
    pub day: usize,
    pub minute: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridViewModel {
    pub zone: Tz,
    pub days: Vec<DayColumn>,
    pub blocks: Vec<Block>,
    pub bars: Vec<Bar>,
    pub lanes: u32,
    pub now: Option<NowLine>,
    /// Minute of the day the hours are scrolled to when the view opens: the current time.
    pub initial_scroll_minute: u32,
}

/// The title the views show, after the mark of a special type (`MOD-05-U01`); `None` for events
/// of calendars shared as free/busy only.
pub(crate) fn shown_title(event: &Event) -> Option<String> {
    (!event.busy_only).then(|| match event.about.kind.icon() {
        Some(icon) => format!("{icon} {}", event.title),
        None => event.title.clone(),
    })
}

/// Whether an event can be dragged: the user may change it (`MOD-05-F17`) and Google has
/// confirmed its last change. A series' occurrence can, the scope asked once dropped
/// (`MOD-05-F20`).
fn movable(event: &Event) -> bool {
    event.protected().is_none() && !event.saving
}

/// The hour grid over `dates`: one day, three days or a week.
pub fn grid_view(
    events: &[Event],
    dates: &[NaiveDate],
    zone: Tz,
    now: DateTime<Utc>,
) -> GridViewModel {
    let today = now.with_timezone(&zone).date_naive();
    let days: Vec<DayColumn> = dates
        .iter()
        .map(|&date| DayColumn {
            date,
            is_today: date == today,
        })
        .collect();

    let blocks = timed_placements(events, dates, zone)
        .into_iter()
        .map(|placement| {
            let event = &events[placement.event];
            let Timing::Timed { start, end, .. } = event.timing else {
                unreachable!("timed placements come from timed events")
            };
            Block {
                placement,
                title: shown_title(event),
                color: event.shown_color(),
                read_only: event.read_only,
                saving: event.saving,
                awaiting: event.about.awaiting(),
                declined: event.about.declined(),
                movable: movable(event),
                hatched: event.about.kind == Kind::OutOfOffice,
                start,
                end,
            }
        })
        .collect();

    let bars: Vec<Bar> = all_day_bars(events, dates, zone)
        .into_iter()
        .map(|bar| {
            let event = &events[bar.event];
            Bar {
                bar,
                title: shown_title(event),
                color: event.shown_color(),
                read_only: event.read_only,
                saving: event.saving,
                awaiting: event.about.awaiting(),
                declined: event.about.declined(),
                movable: movable(event),
                all_day: matches!(event.timing, Timing::AllDay { .. }),
                hatched: event.about.kind == Kind::OutOfOffice,
                label: event.about.kind.display_only(),
            }
        })
        .collect();
    let lanes = bars.iter().map(|b| b.bar.lane + 1).max().unwrap_or(0);

    let minute_now = wall_minute(now, zone);
    let now_line = days.iter().position(|d| d.is_today).map(|day| NowLine {
        day,
        minute: minute_now,
    });

    GridViewModel {
        zone,
        days,
        blocks,
        bars,
        lanes,
        now: now_line,
        initial_scroll_minute: minute_now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{d, tc_03_01_week, timed};
    use crate::model::Protected;
    use crate::week::{GridView, Week};
    use chrono::{TimeZone, Weekday};
    use chrono_tz::Europe::Warsaw;

    fn clock() -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(2026, 11, 11, 10, 0, 0)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn vm() -> GridViewModel {
        grid_view(
            &tc_03_01_week(),
            &Week::containing(d(2026, 11, 11), Weekday::Mon).days(),
            Warsaw,
            clock(),
        )
    }

    #[test]
    fn tc_03_01_week_shows_seven_days_with_timed_events_in_the_hours() {
        let vm = vm();
        assert_eq!(vm.days.len(), 7);
        assert_eq!(vm.days[0].date, d(2026, 11, 9));
        assert_eq!(vm.days[6].date, d(2026, 11, 15));
        let standup = vm
            .blocks
            .iter()
            .find(|b| b.title.as_deref() == Some("Standup"))
            .unwrap();
        assert_eq!(standup.placement.day, 0);
        assert_eq!(
            (standup.placement.start_min, standup.placement.end_min),
            (540, 600)
        );
    }

    #[test]
    fn special_types_are_hatched_marked_or_labels_in_the_strip() {
        let typed = |id: &str, hours: ((u32, u32), (u32, u32)), kind| Event {
            about: crate::model::About {
                kind,
                ..Default::default()
            },
            ..timed(id, d(2026, 11, 12), hours.0, hours.1)
        };
        let events = [
            typed("Away", ((0, 0), (24, 0)), Kind::OutOfOffice),
            typed("Trip", ((9, 0), (12, 0)), Kind::OutOfOffice),
            typed("Writing", ((13, 0), (15, 0)), Kind::FocusTime),
            typed("Home", ((9, 0), (17, 0)), Kind::WorkingLocation),
            typed("Anna", ((0, 0), (24, 0)), Kind::Birthday),
            typed("Review", ((15, 0), (16, 0)), Kind::Ordinary),
        ];
        let vm = grid_view(
            &events,
            &Week::containing(d(2026, 11, 11), Weekday::Mon).days(),
            Warsaw,
            clock(),
        );
        let block = |title: &str| vm.blocks.iter().find(|b| b.title.as_deref() == Some(title));
        let bar = |title: &str| vm.bars.iter().find(|b| b.title.as_deref() == Some(title));

        // Out of office hatched, in the hours or in the strip.
        assert!(block("🌴 Trip").unwrap().hatched);
        let away = bar("🌴 Away").unwrap();
        assert!(away.hatched && !away.label);
        // Focus time with its icon.
        let writing = block("🎧 Writing").unwrap();
        assert!(!writing.hatched);
        // Working location and birthdays as labels in the strip, also a timed working location,
        // and not movable: the app only shows them.
        for label in ["📍 Home", "🎂 Anna"] {
            let label = bar(label).unwrap();
            assert!(label.label && !label.hatched && !label.movable);
        }
        assert!(block("📍 Home").is_none());
        let review = block("Review").unwrap();
        assert!(!review.hatched);
    }

    #[test]
    fn working_locations_and_birthdays_cannot_be_changed_in_the_app() {
        let mut home = timed("Home", d(2026, 11, 12), (9, 0), (17, 0));
        home.about.kind = Kind::WorkingLocation;
        assert_eq!(home.protected(), Some(Protected::DisplayOnly));
        home.about.kind = Kind::Birthday;
        assert_eq!(home.protected(), Some(Protected::DisplayOnly));
        home.about.kind = Kind::OutOfOffice;
        assert_eq!(home.protected(), None);
    }

    #[test]
    fn tc_03_01_all_day_and_multi_day_events_are_in_the_strip() {
        let vm = vm();
        let bar = |t: &str| {
            vm.bars
                .iter()
                .find(|b| b.title.as_deref() == Some(t))
                .unwrap()
                .bar
        };
        let conference = bar("Conference day");
        assert_eq!((conference.first_day, conference.last_day), (1, 1));
        let offsite = bar("Offsite");
        assert_eq!((offsite.first_day, offsite.last_day), (3, 5));
        assert!(
            vm.blocks
                .iter()
                .all(|b| b.title.as_deref() != Some("Offsite"))
        );
    }

    #[test]
    fn tc_03_01_free_busy_events_are_busy_blocks_without_a_title() {
        let vm = vm();
        let friday: Vec<&Block> = vm.blocks.iter().filter(|b| b.placement.day == 4).collect();
        assert_eq!(friday.len(), 1);
        assert_eq!(friday[0].title, None);
        assert_eq!(
            (friday[0].placement.start_min, friday[0].placement.end_min),
            (780, 840)
        );
    }

    #[test]
    fn only_events_that_can_be_changed_now_can_be_dragged() {
        let mut events = tc_03_01_week();
        events[0].recurring = true;
        events[2].saving = true;
        let vm = grid_view(
            &events,
            &Week::containing(d(2026, 11, 11), Weekday::Mon).days(),
            Warsaw,
            clock(),
        );
        let movable = |title: &str| {
            let blocks = vm.blocks.iter().map(|b| (&b.title, b.movable));
            let bars = vm.bars.iter().map(|b| (&b.title, b.movable));
            blocks
                .chain(bars)
                .find(|(t, _)| t.as_deref().unwrap_or("Busy") == title)
                .unwrap()
                .1
        };
        // An event being saved and one of a read-only calendar stay put; a series' occurrence
        // moves.
        assert!(movable("Standup"));
        assert!(!movable("Offsite"));
        assert!(!movable("Busy"));
        assert!(movable("Conference day"));
        let offsite = vm
            .bars
            .iter()
            .find(|b| b.title.as_deref() == Some("Offsite"));
        assert!(offsite.is_some_and(|b| b.saving && b.all_day));
    }

    #[test]
    fn a_guests_copy_only_its_organizer_may_change_cannot_be_dragged_and_says_why() {
        let mut events = tc_03_01_week();
        let conference = events
            .iter_mut()
            .find(|e| e.title == "Conference day")
            .unwrap();
        conference.organizer_only = true;
        let vm = grid_view(
            &events,
            &Week::containing(d(2026, 11, 11), Weekday::Mon).days(),
            Warsaw,
            clock(),
        );

        let bar = vm
            .bars
            .iter()
            .find(|b| b.title.as_deref() == Some("Conference day"));
        assert!(bar.is_some_and(|b| !b.movable && !b.read_only));
        let why = |title: &str| {
            let event = events.iter().find(|e| e.title == title).unwrap();
            event.protected().map(Protected::why)
        };
        assert_eq!(
            why("Conference day"),
            Some("This event can't be changed: only its organizer can change it.")
        );
        assert_eq!(
            why("Room booking"),
            Some("🔒 This event can't be changed: its calendar is read-only.")
        );
        assert_eq!(why("Standup"), None);
    }

    #[test]
    fn today_is_highlighted_and_the_view_opens_at_the_current_time() {
        let vm = vm();
        let today: Vec<usize> = vm
            .days
            .iter()
            .enumerate()
            .filter(|(_, d)| d.is_today)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(today, vec![2]);
        assert_eq!(
            vm.now,
            Some(NowLine {
                day: 2,
                minute: 600
            })
        );
        assert_eq!(vm.initial_scroll_minute, 600);
    }

    #[test]
    fn another_week_has_no_now_line_but_still_opens_at_the_current_time() {
        let vm = grid_view(
            &[],
            &Week::containing(d(2026, 11, 20), Weekday::Mon).days(),
            Warsaw,
            clock(),
        );
        assert!(vm.days.iter().all(|d| !d.is_today));
        assert_eq!(vm.now, None);
        assert_eq!(vm.initial_scroll_minute, 600);
    }

    fn placed(event: crate::model::Event, date: NaiveDate) -> (u32, u32) {
        let vm = grid_view(
            &[event],
            &Week::containing(date, Weekday::Mon).days(),
            Warsaw,
            clock(),
        );
        let p = vm.blocks[0].placement;
        (p.start_min, p.end_min)
    }

    #[test]
    fn on_daylight_saving_days_events_sit_on_their_clock_hours() {
        // Clocks go back on 25 October and forward on 29 March (Europe/Warsaw).
        for date in [d(2026, 10, 25), d(2026, 3, 29)] {
            assert_eq!(
                placed(timed("ten", date, (10, 0), (11, 0)), date),
                (600, 660)
            );
        }
        // 01:00–04:00 on the 25-hour day lasts four real hours but spans 01:00–04:00 on the clock.
        let night = timed("night", d(2026, 10, 25), (1, 0), (4, 0));
        assert_eq!(placed(night, d(2026, 10, 25)), (60, 240));
    }

    /// `TC-03-05`: Wednesday 11 to Friday 13 November have events, and so do the days around.
    fn tc_03_05_events() -> Vec<crate::model::Event> {
        vec![
            timed("Tuesday", d(2026, 11, 10), (9, 0), (10, 0)),
            timed("Wednesday", d(2026, 11, 11), (9, 0), (10, 0)),
            timed("Thursday", d(2026, 11, 12), (9, 0), (10, 0)),
            timed("Friday", d(2026, 11, 13), (9, 0), (10, 0)),
            timed("Saturday", d(2026, 11, 14), (9, 0), (10, 0)),
        ]
    }

    fn titles(vm: &GridViewModel) -> Vec<(usize, &str)> {
        let mut t: Vec<(usize, &str)> = vm
            .blocks
            .iter()
            .map(|b| (b.placement.day, b.title.as_deref().unwrap()))
            .collect();
        t.sort();
        t
    }

    fn shown(view: GridView) -> GridViewModel {
        grid_view(
            &tc_03_05_events(),
            &view.days(d(2026, 11, 11), Weekday::Mon),
            Warsaw,
            clock(),
        )
    }

    #[test]
    fn tc_03_05_day_view_shows_only_the_selected_day_and_its_events() {
        let vm = shown(GridView::Day);
        let dates: Vec<NaiveDate> = vm.days.iter().map(|c| c.date).collect();
        assert_eq!(dates, vec![d(2026, 11, 11)]);
        assert_eq!(titles(&vm), vec![(0, "Wednesday")]);
    }

    #[test]
    fn tc_03_05_three_day_view_shows_three_days_from_the_selected_day() {
        let vm = shown(GridView::ThreeDays);
        let dates: Vec<NaiveDate> = vm.days.iter().map(|c| c.date).collect();
        assert_eq!(
            dates,
            vec![d(2026, 11, 11), d(2026, 11, 12), d(2026, 11, 13)]
        );
        assert_eq!(
            titles(&vm),
            vec![(0, "Wednesday"), (1, "Thursday"), (2, "Friday")]
        );
    }

    #[test]
    fn day_and_three_day_views_show_the_strip_and_the_current_time_as_the_week_does() {
        let events = tc_03_01_week();
        let day = grid_view(
            &events,
            &GridView::Day.days(d(2026, 11, 12), Weekday::Mon),
            Warsaw,
            clock(),
        );
        let offsite = day
            .bars
            .iter()
            .find(|b| b.title.as_deref() == Some("Offsite"));
        assert_eq!(
            offsite.map(|b| (b.bar.first_day, b.bar.last_day)),
            Some((0, 0))
        );
        assert_eq!(day.lanes, 1);
        assert_eq!(day.now, None);

        let three = grid_view(
            &events,
            &GridView::ThreeDays.days(d(2026, 11, 10), Weekday::Mon),
            Warsaw,
            clock(),
        );
        let bars: Vec<(&str, usize, usize)> = three
            .bars
            .iter()
            .map(|b| (b.title.as_deref().unwrap(), b.bar.first_day, b.bar.last_day))
            .collect();
        assert_eq!(bars, vec![("Conference day", 0, 0), ("Offsite", 2, 2)]);
        assert_eq!(
            three.now,
            Some(NowLine {
                day: 1,
                minute: 600
            })
        );
        assert_eq!(three.initial_scroll_minute, 600);
    }

    #[test]
    fn day_view_places_overlapping_events_side_by_side() {
        let events = [
            timed("a", d(2026, 11, 11), (10, 0), (11, 0)),
            timed("b", d(2026, 11, 11), (10, 30), (11, 30)),
        ];
        let vm = grid_view(
            &events,
            &GridView::Day.days(d(2026, 11, 11), Weekday::Mon),
            Warsaw,
            clock(),
        );
        let mut columns: Vec<(u32, u32)> = vm
            .blocks
            .iter()
            .map(|b| (b.placement.column, b.placement.columns))
            .collect();
        columns.sort();
        assert_eq!(columns, vec![(0, 2), (1, 2)]);
    }

    #[test]
    fn day_and_three_day_views_highlight_today_and_show_free_busy_as_busy_blocks() {
        let events = tc_03_01_week();
        for view in [GridView::Day, GridView::ThreeDays] {
            let vm = grid_view(
                &events,
                &view.days(d(2026, 11, 11), Weekday::Mon),
                Warsaw,
                clock(),
            );
            assert!(vm.days[0].is_today);
            assert!(vm.days[1..].iter().all(|c| !c.is_today));
            assert_eq!(
                vm.now,
                Some(NowLine {
                    day: 0,
                    minute: 600
                })
            );
        }
        let friday = grid_view(
            &events,
            &GridView::Day.days(d(2026, 11, 13), Weekday::Mon),
            Warsaw,
            clock(),
        );
        assert_eq!(friday.blocks.len(), 1);
        assert_eq!(friday.blocks[0].title, None);
        assert!(friday.days.iter().all(|c| !c.is_today));
    }

    #[test]
    fn tc_03_12_a_sunday_first_locale_and_a_new_system_zone_are_followed() {
        use crate::navigation::{Navigation, View};
        use crate::regional::Regional;
        use chrono_tz::America::New_York;

        let us = Regional::from_env(|key| (key == "LANG").then(|| "en_US.UTF-8".to_string()));
        let events = [timed("Thursday", d(2026, 11, 12), (15, 0), (16, 0))];
        let nav = Navigation::start(
            Some(View::Grid(GridView::Week)),
            d(2026, 11, 11),
            us.week_start,
        );
        let (first, last) = nav.period().unwrap();
        assert_eq!((first, last), (d(2026, 11, 8), d(2026, 11, 14)));
        assert_eq!(us.date_range(first, last), "Nov 8 – Nov 14, 2026");

        let shown = |zone: Tz| {
            let vm = grid_view(
                &events,
                &GridView::Week.days(nav.selected, nav.week_start),
                zone,
                clock(),
            );
            let block = &vm.blocks[0];
            let time = |t: DateTime<Utc>| us.time(t.with_timezone(&zone).time());
            let heads: Vec<String> = vm.days.iter().map(|c| us.weekday_day(c.date)).collect();
            (
                heads,
                block.placement.day,
                time(block.start),
                time(block.end),
            )
        };
        let (heads, day, start, end) = shown(Warsaw);
        assert_eq!(heads[0], "Sun 8");
        assert_eq!(heads[6], "Sat 14");
        assert_eq!(
            (day, start.as_str(), end.as_str()),
            (4, "3:00 PM", "4:00 PM")
        );
        // The system's time zone changes to America/New_York.
        let (_, day, start, end) = shown(New_York);
        assert_eq!(
            (day, start.as_str(), end.as_str()),
            (4, "9:00 AM", "10:00 AM")
        );
    }
}
