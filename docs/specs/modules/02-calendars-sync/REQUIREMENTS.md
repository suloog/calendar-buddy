---
id: MOD-02
title: Calendars, synchronisation and offline cache
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | done
depends_on: [MOD-01] # other module IDs
sources: [SRC-01, SRC-02] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-02] Calendars, synchronisation and offline cache

## Context

Which calendars the app shows and how it keeps them current: every accessible calendar in one view, local colours, background synchronization within about a minute and on demand, the local cache from one year back to two years ahead, offline browsing, read-only and subscribed calendars, and what happens when a calendar loses access. Answers for the scope area "Calendars, synchronisation and offline cache"; comes from `DSC-04` (`discovery/04-calendars-sync.md`), drawing on `SRC-01` and `SRC-02`.

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-11` Local-first: cache calendar data locally for fast access and offline browsing. | informal | `MOD-02-F13`, `MOD-02-F14` |
| `SRC-01-25` Retrieve all accessible calendars from connected Google accounts. | informal | `MOD-02-F01`, `MOD-02-F05` |
| `SRC-01-26` Show calendars from multiple accounts in unified views. | informal | `MOD-02-F02` |
| `SRC-01-27` Let users toggle calendar visibility and preserve provider-defined colors. | informal | `MOD-02-F03`; visibility toggle in `MOD-03` |
| `SRC-01-28` Synchronize changes automatically and offer manual refresh. | informal | `MOD-02-F07`, `MOD-02-F09` |
| `SRC-01-29` Use incremental synchronization (`syncToken`) where supported by the Google Calendar API. | informal | `MOD-02-F10` |
| `SRC-01-30` Cache events and relevant calendar metadata in a local SQLite database. | informal | `MOD-02-F11` |
| `SRC-01-31` Support offline browsing of previously synchronized events. | informal | `MOD-02-F14` |
| `SRC-01-32` Recover from connectivity loss, expired synchronization tokens, rate limits, and transi… | informal | `MOD-02-F10`, `MOD-02-F16` |
| `SRC-01-33` Represent the synchronization state clearly without blocking the user interface. | informal | `MOD-02-F17`, `MOD-02-F18` |
| `SRC-01-34` Handle time zones, daylight-saving changes, and recurring-event exceptions correctly. | informal | `MOD-02-F22` |
| `SRC-01-35` MVP offline boundary: offline browsing is included; offline creation/editing with a dur… | informal | `MOD-02-F15` |
| `SRC-01-98` Multiple calendars displayed in a unified interface. | informal | `MOD-02-F02` |
| `SRC-01-106` Local synchronization cache and offline browsing. | informal | `MOD-02-F11`, `MOD-02-F14` |
| `SRC-01-121` See events from selected calendars across accounts. | informal | `MOD-02-F02` |
| `SRC-01-128` View cached events when temporarily offline. | informal | `MOD-02-F14` |
| `SRC-02-22` Colours should be settable locally: each calendar starts with its Google colour and can… | informal | `MOD-02-F04` |
| `SRC-02-28` The app uses only calendars that already exist in Google; creating, renaming and deleti… | informal | `MOD-02-F01`; no calendar management in the app (`DEC-21`) |
| `SRC-02-29` Read-only and subscribed calendars are shown like the others, without editing; free/bus… | informal | `MOD-02-F19`, `MOD-02-F20`, `MOD-02-U01` |
| `SRC-02-30` The cache keeps events from one year back to two years ahead, with no separate size lim… | informal | `MOD-02-F11`, `MOD-02-F12` |
| `SRC-02-31` A calendar that loses access while the account works is removed with its cached events,… | informal | `MOD-02-F21` |
| `SRC-02-32` Offline, the MVP allows browsing only; creating, editing and RSVP are unavailable, with… | informal | `MOD-02-F15` |
| `SRC-02-33` While the app is open, changes made elsewhere appear within about a minute, and right a… | informal | `MOD-02-F07`, `MOD-02-F08`, `MOD-02-F23` (cadence changed by `DEC-53`) |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

- [MOD-02-F01] The app fetches the list of every calendar each connected account can access, including read-only and subscribed calendars.
- [MOD-02-F02] Events from the calendars of all connected accounts are shown together in the same views.
- [MOD-02-F03] Each calendar takes its colour from Google by default.
- [MOD-02-F04] The user can give a calendar a local colour, which is not written to Google, and can reset it to the Google colour. — *source: `DEC-54`*
- [MOD-02-F05] The list of calendars stays current: calendars added in Google appear in the app, and calendars that are gone disappear, at the next synchronization.
- [MOD-02-F06] Calendars disabled in the settings stay on the list of known calendars, but their events are not synchronized or cached; enabling one starts its first synchronization. — *source: `DEC-28`*
- [MOD-02-F07] While the app runs, changes made elsewhere to calendars the user can edit appear within about a minute, and read-only and subscribed calendars are refreshed about every 15 minutes; while the window is hidden in the tray, calendars the user can edit are refreshed at least every 5 minutes. — *source: `DEC-53`, `DEC-54`*
- [MOD-02-F08] The app synchronizes right after the computer wakes from sleep and right after the network returns. — *source: `DEC-53`*
- [MOD-02-F09] The user can start a synchronization by hand at any time.
- [MOD-02-F10] After a calendar's first full synchronization, only what changed is fetched; when Google no longer accepts the stored synchronization state, the app synchronizes that calendar in full again without the user's involvement.
- [MOD-02-F11] The cache holds the events of enabled calendars from one year back to two years ahead of today. — *source: `DEC-23`*
- [MOD-02-F12] Periods outside the cache window can be browsed while online; their events are fetched from Google as needed and are not kept for offline use. — *source: `DEC-54`*
- [MOD-02-F13] On start the app shows the cached calendars at once, without waiting for synchronization.
- [MOD-02-F14] Without network the user can browse every cached event, and the app shows that it is offline.
- [MOD-02-F15] Without network, creating, editing and deleting events and responding to invitations are unavailable, and the app says why. — *source: `DEC-25`*
- [MOD-02-F16] Synchronization recovers on its own from a lost connection, Google's rate limits and transient errors, retrying with growing intervals and without losing data.
- [MOD-02-F17] The app shows whether it is synchronizing, up to date, offline or in error, and the user can keep working while synchronization runs.
- [MOD-02-F18] An error that affects one calendar is shown on that calendar, and the other calendars keep synchronizing.
- [MOD-02-F19] Read-only and subscribed calendars are shown like the others, and their events cannot be changed. — *source: `DEC-22`*
- [MOD-02-F20] Calendars shared as free/busy only show busy blocks without event details. — *source: `DEC-22`*
- [MOD-02-F21] When a calendar stops being accessible while its account still works, it is removed from the app with its cached events, and the user is told once. — *source: `DEC-24`*
- [MOD-02-F22] Events are shown at the right local time across time zones and daylight-saving changes, including occurrences of a series that were moved or cancelled.
- [MOD-02-F23] The app synchronizes at once when its window becomes active and before an event is opened for editing. — *source: `DEC-53`*

## UI/UX requirements

- [MOD-02-U01] Read-only calendars and their events are visibly marked as not editable. — *source: `DEC-22`*
- [MOD-02-U02] While there is no network, an offline indicator is visible in the main window. — *source: `DEC-19`*
- [MOD-02-U03] The synchronization indicator shows the time of the last successful synchronization. — *source: `DEC-54`*
- [MOD-02-U04] A calendar's local colour is chosen from the calendar's menu in the side panel. — *source: `DEC-54`*

## Non-functional requirements

- [MOD-02-NFR-01] Adaptive background synchronization stays within Google's API limits for a profile of 5 accounts and 50 enabled calendars. — *source: `DEC-14`, `DEC-53`*
- [MOD-02-NFR-02] The first full synchronization of that profile (about 20,000 events) runs in the background without blocking the interface; its duration is measured together with `NFR-03`. — *source: `DEC-14`*

## Constraints and edge cases

- An event that crosses the edge of the cache window → it is kept whole. — *source: `DEC-54`*
- A recurring series that started before the window → its occurrences inside the window are available offline.
- Google answers with a rate limit → synchronization slows down, the indicator says it is delayed, and nothing is lost.
- Google rejects the stored synchronization state → that calendar is synchronized in full again (`MOD-02-F10`).
- An account needs sign-in (`MOD-01-F05`) → its cached calendars stay visible and are marked as not synchronizing.
- A calendar's access drops from editable to read-only → it is marked read-only at the next synchronization.
- A calendar is deleted while the app is offline → it is removed at the next synchronization, with the notice of `MOD-02-F21`.
- The cache is damaged or unreadable → the app discards it and rebuilds it from Google. — *source: `DEC-54`*
- The day changes → the window moves: events older than one year are dropped from the cache, and events newly inside two years ahead are fetched.

## Integrations

- **Google Calendar API — calendar list** (read): the user's calendars, their access level and their colours.
- **Google Calendar API — events** (read): full and incremental fetching of each enabled calendar's events, including recurring series and their exceptions.
- **Google Calendar API — colours** (read): the palette that calendar and event colours refer to.
- **Operating system** (read): network status, and sleep and wake signals, to trigger synchronization.

## Technical notes

- Local store: SQLite, the read model for every view; only the sync engine writes synchronized data (constitution).
- Per calendar: a first full sync bounded by the window (`timeMin` one year back, `timeMax` two years ahead), then incremental sync with the stored `syncToken`; incremental answers are filtered to the window locally. HTTP 410 on a `syncToken` → full resync of that calendar.
- Series are kept as master events plus exceptions (not pre-expanded by Google), so editing scopes in `MOD-05` work on the real series; expansion into occurrences uses a tested recurrence library, with times stored as UTC plus the IANA zone (constitution).
- Adaptive polling per calendar by access role: `owner`/`writer` about every 60 s, `reader`/`freeBusyReader` about every 15 min; longer intervals while the window is hidden; an immediate round on window activation and before opening the editor. Each calendar of a round is asked for in its own request (`DEC-91`). No push channels (`events.watch` needs a public endpoint — `DEC-53`). Exponential back-off with jitter on 403 rate-limit and 429; per-calendar error state.
- Network and sleep/wake signals from NetworkManager and systemd-logind over D-Bus, behind the platform interface.
- Local calendar colours live in the app's settings, keyed by account ID and calendar ID (`DEC-99`), never sent to Google.
- The window is re-evaluated daily; purged events leave no trace in the store.

## Out of scope / what not to worry about

- Signing in and account states — `MOD-01`.
- Showing, hiding and disabling calendars in the side panel and settings, and the views themselves — `MOD-03`. This module only honours "disabled" (`MOD-02-F06`).
- Writing changes to Google: creating, editing, deleting, RSVP — `MOD-05`, `MOD-06`. This module reflects them once Google confirms them.
- Event colours (per event) — `MOD-05`.
- Reminders and notifications — `MOD-07`.
- Creating, renaming, deleting or sharing calendars — done in the browser (`DEC-21`, `MOD-08`).
- Offline editing with a change queue — `MOD-08` (`DEC-25`).
- Changes appearing within seconds through push notifications — `DEC-53`.
- Event search — `MOD-08`.

## Acceptance criteria

- [ ] Calendars from two accounts, including a subscribed holiday calendar, are shown together in their Google colours.
- [ ] An event changed on the phone in an editable calendar appears in the open app within about a minute, and at once when the app window is activated; a manual refresh also shows it at once.
- [ ] With the network cut, cached events can be browsed, the offline indicator is shown, and editing actions are unavailable with an explanation; when the network returns, synchronization resumes on its own.
- [ ] A local colour set for a calendar shows in the app and is unchanged in Google Calendar in the browser.
- [ ] A calendar unshared by its owner disappears from the app with a one-time notice.
- [ ] An event in another time zone, an event across a daylight-saving change and a moved occurrence of a series are all shown at the right local times.
- [ ] A calendar disabled in the settings has no events fetched; after enabling it, its events appear once it has synchronized.
- [ ] The app opens with the cached calendars visible before synchronization finishes.
