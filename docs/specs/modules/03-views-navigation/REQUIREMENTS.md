---
id: MOD-03
title: Views and navigation
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | done
depends_on: [MOD-02] # other module IDs
sources: [SRC-01, SRC-02] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-03] Views and navigation

## Context

How the user looks at the calendar and moves around it: day, 3-day, week, month and agenda views, the side panel with calendars grouped by account, hiding and disabling calendars, keyboard-first navigation, and regional formats and time zone taken from the system. Answers for the scope area "Views and navigation"; comes from `DSC-05` (`discovery/05-views-navigation.md`), drawing on `SRC-01` and `SRC-02`. Holds the UI-toolkit feasibility risk (`DEC-33`).

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-36` Day view. | informal | `MOD-03-F01` |
| `SRC-01-38` Full-week view. | informal | `MOD-03-F02` |
| `SRC-01-39` Month view. | informal | `MOD-03-F03` |
| `SRC-01-40` Agenda / upcoming events view. | informal | `MOD-03-F05` |
| `SRC-01-41` Navigate quickly between days, weeks, and months. | informal | `MOD-03-F06`, `MOD-03-F07` |
| `SRC-01-42` Jump back to Today. | informal | `MOD-03-F08` |
| `SRC-01-43` Provide a mini calendar and sidebar listing accounts and calendars. | informal | `MOD-03-F09`, `MOD-03-F10`, `MOD-03-F25` |
| `SRC-01-44` Allow the sidebar to be collapsed. | informal | `MOD-03-F11` |
| `SRC-01-45` Highlight the current day and current-time indicator. | informal | `MOD-03-F14` |
| `SRC-01-46` Support keyboard shortcuts and keyboard-first navigation. | informal | `MOD-03-F23`, `MOD-03-F24` |
| `SRC-01-47` Scroll smoothly and handle overlapping events legibly. | informal | `MOD-03-F19`, `MOD-03-U01` |
| `SRC-01-48` Resize gracefully across different window dimensions and DPI scales. | informal | `MOD-03-U02` |
| `SRC-01-99` Day, week, and month views (workweek and agenda may be included if inexpensive). | informal | `MOD-03-F01`, `MOD-03-F02`, `MOD-03-F03` |
| `SRC-02-34` I look at the calendar mostly in the week view, sometimes in a 3-day view on a smaller… | informal | `MOD-03-F02`, `MOD-03-F04` |
| `SRC-02-35` Calendars should always be grouped by account, also when choosing a calendar while crea… | informal | `MOD-03-F10` |
| `SRC-02-36` Hidden calendars must not clutter quick lists — I have many calendars; turning them bac… | informal | `MOD-03-F13` |
| `SRC-02-37` Views in the MVP besides day, full week and month: 3 days (chosen by hand) and agenda;… | informal | `MOD-03-F04`, `MOD-03-F05`; workweek view not built (`DEC-27`) |
| `SRC-02-38` Two levels of hiding: a quick toggle in the side panel hides only the events; disabling… | informal | `MOD-03-F12`, `MOD-03-F13` |
| `SRC-02-40` Week start and date/time formats follow the system; the calendar is shown in the system… | informal | `MOD-03-F16`, `MOD-03-F17` |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

- [MOD-03-F01] There is a day view.
- [MOD-03-F02] There is a full-week view showing all seven days.
- [MOD-03-F03] There is a month view.
- [MOD-03-F04] There is a 3-day view, chosen by hand like any other view. — *source: `DEC-27`*
- [MOD-03-F05] There is an agenda view listing events from now on, grouped by day, skipping empty days and loading more while the user scrolls. — *source: `DEC-27`, `DEC-55`*
- [MOD-03-F06] The user can switch between views at any time, and the selected date stays in view.
- [MOD-03-F07] The user can move to the next or previous period of the current view.
- [MOD-03-F08] One action brings the user back to today.
- [MOD-03-F09] A mini calendar shows a month at a time, marks the period on screen except in the month view, and jumps to any date the user picks. — *source: `DEC-158`*
- [MOD-03-F10] The side panel lists the connected accounts and, under each, its calendars. — *source: `DEC-28`*
- [MOD-03-F11] The side panel can be collapsed and expanded, and the app remembers which. — *source: `DEC-56`*
- [MOD-03-F12] A quick toggle next to each calendar in the side panel hides or shows its events; the calendar stays listed. — *source: `DEC-28`*
- [MOD-03-F13] In the settings, a calendar can be disabled and enabled again; a disabled calendar disappears from the side panel, the calendar pickers and the views. — *source: `DEC-28`*
- [MOD-03-F14] Today is highlighted in every view, and the day, 3-day and week views show a line at the current time.
- [MOD-03-F15] The app starts in the view used last, on today's date. — *source: `DEC-55`*
- [MOD-03-F16] The first day of the week and the date and time formats follow the system's regional settings. — *source: `DEC-29`*
- [MOD-03-F17] Events are shown in the system's time zone, and the views follow when the system's time zone changes. — *source: `DEC-29`*
- [MOD-03-F18] Opening an event shows its details; an event stored in another time zone also shows its time in that zone. — *source: `DEC-05`, `DEC-29`*
- [MOD-03-F19] Overlapping events are placed side by side so that each one stays readable.
- [MOD-03-F20] In the month view, a day with more events than fit shows how many more there are and opens the full list for that day. — *source: `DEC-56`*
- [MOD-03-F21] All-day and multi-day events are shown in a strip above the hours in the day, 3-day and week views. — *source: `DEC-56`*
- [MOD-03-F22] Calendars shared as free/busy only appear as "busy" blocks in every view. — *source: `DEC-22`*
- [MOD-03-F23] Every view, period change, "today", "go to date", "new event" and the list of shortcuts has a keyboard shortcut, using Google Calendar's keys. — *source: `DEC-55`*
- [MOD-03-F24] The user can move between the side panel, the calendar and its events, and open an event, without a mouse.
- [MOD-03-F25] In the mini calendar, a day with events shown in the views has a dot under its number. — *source: `DEC-158`*
- [MOD-03-F26] In the month view, an all-day or multi-day event is one bar across the days it covers in each week, above the days' timed events. — *source: `DEC-159`*

## UI/UX requirements

- [MOD-03-U01] Scrolling through the hours of the day, 3-day and week views is smooth.
- [MOD-03-U02] The layout adapts to the window size and to the screen's scale.
- [MOD-03-U03] When the window becomes too narrow, the side panel collapses on its own. — *source: `DEC-56`*
- [MOD-03-U04] Pressing "?" shows the list of keyboard shortcuts. — *source: `DEC-55`*
- [MOD-03-U05] The day, 3-day and week views open scrolled to the current time. — *source: `DEC-56`*
- [MOD-03-U06] "Go to date" also opens from a button in the top bar, beside next and previous. — *source: `DEC-161`*
- [MOD-03-U07] The list of keyboard shortcuts also opens from a help button in the top bar, beside the settings, and shows the shortcuts in groups, keys doing the same thing on one row. — *source: `DEC-162`*

## Non-functional requirements

- [MOD-03-NFR-01] Switching the view or the period and scrolling the week view meet the thresholds of `NFR-03` on its profile. — *source: `DEC-14`*

## Constraints and edge cases

- Many events overlap at the same time → each gets a narrower column and keeps at least the start of its title visible; opening it shows everything.
- An event runs past midnight → it is shown on both days.
- A day with a daylight-saving change → the hour grid stays 0:00–24:00 and events sit on their clock hours; the skipped hour stays empty, the repeated hour is shared. — *source: `DEC-67`*
- No calendar is visible (all hidden or disabled) → the view is empty and points to the side panel. — *source: `DEC-56`*
- The agenda has nothing ahead → it says there are no upcoming events. — *source: `DEC-56`*
- Offline, the user moves outside the cache window → the period is shown empty with a note that it is not available offline (`MOD-02-F12`).
- The window is made very small → it does not go below a minimum size at which a day stays readable.

## Integrations

- None outside the app. The views read only the local store kept by `MOD-02`, and the regional settings and time zone of the operating system.

## Technical notes

- The first story of this module is the UI-toolkit feasibility check of `DEC-33`: a week view in egui on fixture data with smooth scrolling, overlapping events, resizing at several scales, and dragging an event. A failed check falls back to Iced before the other views are built. It can run in parallel with `MOD-01`/`MOD-02`.
- Views render from view models fed by the local store; no view calls Google (constitution).
- Overlap layout: the usual column-packing of overlapping intervals per day; all-day and multi-day events in a separate strip.
- Shortcuts follow Google Calendar's keys: `d`, `x` (3 days — Google's custom view key), `w`, `m`, `a`; `j`/`n` next, `k`/`p` previous; `t` today; `g` go to date; `c` new event (opens `MOD-05`'s editor); `?` the list. `/` (search) is not bound — no search (`DEC-30`).
- Regional settings and the time zone come from the system locale and the system's zone database; a change of zone triggers a re-render.
- View state (last view, side panel collapsed, calendars hidden) lives in the app's settings; calendars disabled are honoured by `MOD-02-F06`.

## Out of scope / what not to worry about

- A workweek view and switching to the 3-day view automatically in a narrow window — `DEC-27`.
- Event search — `DEC-30`, `MOD-08`.
- A second time zone or choosing the display time zone by hand — `DEC-29`.
- Themes, colours of the interface and visual style — `MOD-04`; calendar colours — `MOD-02`.
- Creating events by clicking or selecting a range, dragging and resizing events, and everything in the event editor — `MOD-05`.
- Guests, responses and the Join button in the event details — `MOD-06`.
- Fetching and synchronizing the data the views show — `MOD-02`.

## Acceptance criteria

- [ ] Day, 3-day, week, month and agenda views each show the events of the visible calendars of all accounts.
- [ ] Every view, period change, "today" and "go to date" works from the keyboard with Google Calendar's keys, and "?" lists them.
- [ ] The side panel shows calendars grouped by account; a quick toggle hides a calendar's events, and disabling it in the settings removes it from the panel and the views.
- [ ] After restarting, the app opens in the last used view on today's date.
- [ ] Overlapping events, an event past midnight and a day with a daylight-saving change are all drawn readably and at the right times.
- [ ] With the system set to a locale starting the week on Sunday and a different time zone, the views follow both.
- [ ] Scrolling the week view on the 20,000-event profile is smooth and within the measured thresholds.
