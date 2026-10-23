---
id: DSC-05
title: Views and navigation
kind: business # business (a functional area — becomes a module at /og:modules) | cross-cutting (roles and access, technology and integrations, non-functional requirements — feeds PROJECT.md, NFR.md and the constitution, never a module)
status: settled # open (not yet discussed) | discussed (discussed with the analyst, waiting on the client) | settled (no [W] question in this topic is open)
sources: [SRC-01, SRC-02] # SRC-NN documents whose items this topic gathers, e.g. [SRC-01, SRC-03]
module: # MOD-NN — filled by /og:modules; empty for cross-cutting topics
sessions: [2026-10-22] # dates of the discovery sessions in which the topic was discussed, YYYY-MM-DD
---

<!-- Copy to docs/specs/discovery/NN-<topic>.md. Content in the project language; headings fixed.
     This file records what we UNDERSTOOD about this topic after reading every source and talking
     to the analyst — the layer between the sources (what the client wrote) and the requirements
     (what we will build). It is alive: after every client workshop /og:discovery updates the
     findings. A finding is NOT a requirement: no acceptance criteria, no priority, no UI; one
     finding becomes 1–3 requirements in the module's REQUIREMENTS.md later. -->

# [DSC-05] Views and navigation

## Topic scope

The calendar views (day, workweek, full week, month, agenda), moving between periods, Today, the mini calendar and sidebar, the current-time indicator, keyboard-first navigation, scrolling, overlapping events and resizing. Ends at how it looks (DSC-06) and what can be done to an event (DSC-07).

## Sources in this topic

<Items from ALL sources that speak about this topic, side by side — so a contradiction between the
contract and the kick-off note is visible in one place, not in two files.>

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-01-36` | informal | Day view. |
| `SRC-01-37` | informal | Workweek view. |
| `SRC-01-38` | informal | Full-week view. |
| `SRC-01-39` | informal | Month view. |
| `SRC-01-40` | informal | Agenda / upcoming events view. |
| `SRC-01-41` | informal | Navigate quickly between days, weeks, and months. |
| `SRC-01-42` | informal | Jump back to Today. |
| `SRC-01-43` | informal | Provide a mini calendar and sidebar listing accounts and calendars. |
| `SRC-01-44` | informal | Allow the sidebar to be collapsed. |
| `SRC-01-45` | informal | Highlight the current day and current-time indicator. |
| `SRC-01-46` | informal | Support keyboard shortcuts and keyboard-first navigation. |
| `SRC-01-47` | informal | Scroll smoothly and handle overlapping events legibly. |
| `SRC-01-48` | informal | Resize gracefully across different window dimensions and DPI scales. |
| `SRC-01-99` | informal | Day, week, and month views (workweek and agenda may be included if inexpensive). |
| `SRC-02-34` | informal | Mostly the week view; sometimes 3 days on a smaller screen. |
| `SRC-02-35` | informal | Calendars grouped by account, also in the picker. |
| `SRC-02-36` | informal | Hidden calendars must not clutter quick lists. |
| `SRC-02-37` | informal | MVP views: + 3 days (manual) and agenda; no workweek. |
| `SRC-02-38` | informal | Quick toggle vs. disable in settings; disabled = not synced. |
| `SRC-02-39` | informal | No search in the MVP. |
| `SRC-02-40` | informal | Formats and time zone from the system; no second zone. |

## Key findings

- [DSC-05-01] The core views are day, full week and month. (`SRC-01-36`, `SRC-01-38`, `SRC-01-39`, `SRC-01-99`)
- [DSC-05-02] Moving between days, weeks and months is quick, and one action goes back to today. (`SRC-01-41`, `SRC-01-42`)
- [DSC-05-03] Next to the calendar there is a mini calendar and a side panel with the accounts and their calendars; the panel can be collapsed. (`SRC-01-43`, `SRC-01-44`)
- [DSC-05-04] Today and the current time are highlighted. (`SRC-01-45`)
- [DSC-05-05] The app is operated mainly from the keyboard, with shortcuts for common actions. (`SRC-01-46`)
- [DSC-05-06] Scrolling is smooth, and overlapping events stay readable. (`SRC-01-47`)
- [DSC-05-07] The layout adapts to the window size and the DPI scale. (`SRC-01-48`)
- [DSC-05-08] There is a 3-day view, chosen by hand like any other, for smaller screens. (`SRC-02-34`, `SRC-02-37`) — *source: `DEC-27`*
- [DSC-05-09] There is an agenda view listing upcoming events. (`SRC-01-40`, `SRC-02-37`) — *source: `DEC-27`*
- [DSC-05-10] Calendars are always grouped by account — in the side panel and when choosing a calendar for a new event. (`SRC-02-35`) — *source: `DEC-28`*
- [DSC-05-11] A calendar can be hidden in two ways: a quick toggle in the side panel hides its events but keeps it listed; disabling it in the settings removes it from the side panel, the calendar picker and the views, and it is re-enabled in the settings. (`SRC-01-27`, `SRC-02-36`, `SRC-02-38`) — *source: `DEC-28`*
- [DSC-05-12] A disabled calendar is not synchronized or cached and gives no reminders; when re-enabled it goes through a first synchronization. (`SRC-02-38`) — *source: `DEC-28`*
- [DSC-05-13] Week start and date and time formats follow the system's regional settings. (`SRC-02-40`) — *source: `DEC-29`*
- [DSC-05-14] The calendar is shown in the system time zone; an event stored in another time zone also shows its own time in its details. (`SRC-01-34`, `SRC-02-40`) — *source: `DEC-29`*

## Contradictions

- `SRC-01-37` and `SRC-01-40` are main views (§6.1), `SRC-01-99` makes them optional, SRC-01 §16 leaves them open → settled: agenda in, workweek out, `DEC-27` (was `Q01`)

## Ambiguities

- "Week" in `SRC-01-99`: (a) full week, (b) workweek → (a), `DEC-27` (was `Q02`)
- Visibility toggle (`SRC-01-27`) vs. "hidden calendars must not clutter lists" (`SRC-02-36`) → two levels, quick toggle and disable in settings, `DEC-28`

## From the calls, not the documents

- `SRC-02-34` how the product owner looks at the calendar (week; 3 days on a smaller screen)
- `SRC-02-35`, `SRC-02-36`, `SRC-02-38` grouping by account and two levels of hiding
- `SRC-02-37`, `SRC-02-39`, `SRC-02-40` answers to `Q01`, `Q02`, `Q17`, `Q19`

## For the client workshop

None open — `Q01`, `Q02`, `Q17`, `Q19` resolved on 2026-10-22. Default reminders (part of `Q19`) continue in `Q21`, `DSC-09`.

## Parked

- Workweek view (`SRC-01-37`) — not in the MVP, `DEC-27`.
- Event search, basic or advanced (`SRC-01-113`) — done in the browser, `DEC-30`.
- A second time zone and choosing the display time zone by hand — after the MVP, `DEC-29`.
- Switching to the 3-day view automatically in a narrow window — rejected, the choice is manual, `DEC-27`.
