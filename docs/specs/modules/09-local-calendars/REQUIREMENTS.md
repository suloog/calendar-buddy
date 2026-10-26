---
id: MOD-09
title: Local calendars
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | done
depends_on: [MOD-01, MOD-03, MOD-05, MOD-07] # other module IDs
sources: [SRC-06] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-09] Local calendars

## Context

Calendars and events kept only on the device, never synchronized with Google, so that the app can be used without a Google account: managing local calendars in the settings, the "Local" group next to the accounts, local events edited as Google events are, their notifications, and a welcome screen that invites to connect Google without requiring it. Comes from the product owner's request of 2026-10-26 (`SRC-06`), settled by `DEC-178` and `DEC-179`; it goes beyond Google Calendar (`DEC-62`) at the product owner's word. Wherever a local event can behave as a Google event already does in `MOD-03`, `MOD-05` and `MOD-07`, it does — this module names only what differs.

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-06-01` The app gets local calendars and events that are not synchronized with Google. | informal | `MOD-09-F01`, `MOD-09-F07`, `MOD-09-F08` |
| `SRC-06-02` Thanks to them, connecting a Google account is no longer necessary to use the app. | informal | `MOD-09-F17`, `MOD-09-F18` |
| `SRC-06-03` The welcome screen stays as an encouragement and onboarding, but connecting an accoun… | informal | `MOD-09-F17`, `MOD-09-U04` |
| `SRC-06-04` The user can add several local calendars from the settings. | informal | `MOD-09-F02`, `MOD-09-U01` |
| `SRC-06-05` Local calendars are grouped under "Local", as calendars are grouped by account today. | informal | `MOD-09-F05`, `MOD-09-U02` |
| `SRC-06-06` A local event has what a Google event has: title, time, all-day, description, locati… | informal | `MOD-09-F08`, `MOD-09-F09` |
| `SRC-06-07` A local event has no guests, Google Meet, responses, attachments or special event ty… | informal | `MOD-09-F10` |
| `SRC-06-08` Local events give notifications as Google events do, with a default notification se… | informal | `MOD-09-F13`, `MOD-09-F14`, `MOD-09-F15` |
| `SRC-06-09` In the settings a local calendar can be added, renamed, recoloured and deleted; dele… | informal | `MOD-09-F02`, `MOD-09-F03`, `MOD-09-F04`, `MOD-09-U03` |
| `SRC-06-10` An event is not moved between a local calendar and a Google calendar, at least for n… | informal | `MOD-09-F12` |
| `SRC-06-11` A local calendar can be the default calendar for new events; without a Google accoun… | informal | `MOD-09-F11` |
| `SRC-06-12` The group is named "Local" in the English interface and comes last in the side panel… | informal | `MOD-09-U02` |
| `SRC-06-13` The welcome screen gets a second action, "Use without an account", which creates a f… | informal | `MOD-09-F17` |
| `SRC-06-14` The welcome screen shows only when there is neither an account nor a local calendar;… | informal | `MOD-09-F18`, `MOD-09-F19` |
| `SRC-06-15` Local data is kept apart from the cache, in the app's data directory, so that it is… | informal | `MOD-09-F01`, `MOD-09-NFR-01` |
| `SRC-06-16` Local events do not make the user busy for others in "Meet with…" and availability,… | informal | `MOD-09-F16` |
| `SRC-06-17` The work goes through the spec workflow: a source, a decision, a new module with req… | informal | this module (process, no requirement) |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

### Local calendars

- [MOD-09-F01] The app keeps local calendars and their events only on the device; nothing about them is sent to Google, and they work the same with or without network. — *source: `DEC-178`*
- [MOD-09-F02] In the settings the user can add a local calendar, giving it a name; any number of local calendars can be added. — *source: `DEC-178`*
- [MOD-09-F03] In the settings the user can rename a local calendar and change its colour, chosen from the same palette as a Google calendar's local colour (`MOD-02-F04`). — *source: `DEC-178`*
- [MOD-09-F04] In the settings the user can delete a local calendar; its events are deleted with it for good. — *source: `DEC-178`*
- [MOD-09-F05] Local calendars are listed as one group, "Local", in the side panel, the calendar picker and the settings, as an account's calendars are (`MOD-03-F10`, `DEC-28`). — *source: `DEC-178`*
- [MOD-09-F06] A local calendar can be hidden from the side panel, disabled and enabled in the settings, and muted, as a Google calendar can (`MOD-03-F12`, `MOD-03-F13`, `MOD-07-F12`, `MOD-07-F13`). — *source: `DEC-178`*

### Local events

- [MOD-09-F07] Events of local calendars are shown together with Google events in every view (day, 3-day, week, month, agenda) and in the mini calendar. — *source: `DEC-178`*
- [MOD-09-F08] The user creates a local event as a Google event — by clicking or selecting a time, through the quick-entry box and the full editor — by choosing a local calendar; it is saved at once, without waiting for Google. — *source: `DEC-178`*
- [MOD-09-F09] A local event has the title, description, start and end with time zone, all-day and several days, location, event colour and repeating (the repeat menu and the custom dialog, with the three edit scopes) of a Google event; it can be dragged, resized, edited, deleted and the change undone as a Google event can (`MOD-05-F01`…`F05`, `F11`…`F14`, `F18`…`F21`, `F25`, `F27`, `F28`, `F32`…`F35`). — *source: `DEC-178`*
- [MOD-09-F10] A local event has no guests, Google Meet, responses, attachments, special event types, busy/free or visibility settings; the editor does not offer them for a local calendar. — *source: `DEC-178`*
- [MOD-09-F11] A local calendar can be chosen as the default calendar for new events (`MOD-05-F10`); with no account connected, new events go to a local calendar. — *source: `DEC-178`*
- [MOD-09-F12] An event cannot be moved between a local calendar and a Google calendar; the editor's calendar picker offers, for an existing event, only calendars of its own kind (local, or the same account's as today). — *source: `DEC-178`*
- [MOD-09-F13] A local event gives desktop notifications as a Google event does (`MOD-07-F01`, `F07`, `F08`, `F10`, `F20`, `MOD-07-U01`). — *source: `DEC-178`*
- [MOD-09-F14] Each local calendar has its own default notifications, set in the settings; a new local calendar starts with one notification 30 minutes before. A new local event uses its calendar's defaults, and events that use them follow when they change; an all-day event that uses them gives no notification, as an all-day Google event does (`DEC-137`). — *source: `DEC-178`, `DEC-180`*
- [MOD-09-F15] In the editor the user can change a local event's notifications and set it back to its calendar's defaults, as for a Google event (`MOD-07-F04`, `MOD-07-F05`), with notifications only — no e-mail reminders. — *source: `DEC-178`*
- [MOD-09-F16] Local events are seen only by the user: Google does not know them, so they never make the user busy for other people, in their "Meet with…" or in any availability preview. In the user's own views, the user's column of "Meet with…" included, they are shown as other events are. — *source: `DEC-178`, `DEC-180`*

### Welcome screen

- [MOD-09-F17] The welcome screen offers, next to connecting a Google account, "Use without an account", which creates a local calendar named "My calendar", makes it the default calendar and opens the calendar views. — *source: `DEC-179`*
- [MOD-09-F18] The welcome screen shows only when there is neither a connected account nor a local calendar; a Google account can be connected later from the settings at any time. — *source: `DEC-179`*
- [MOD-09-F19] Removing the last account goes back to the welcome screen only when no local calendar is left; deleting the last local calendar does the same when no account is connected. — *source: `DEC-179`*

## UI/UX requirements

- [MOD-09-U01] The settings have a "Local calendars" section listing every local calendar with its colour and name, an action to add one, and for each one renaming, its colour, its default notifications and deleting. — *source: `DEC-178`*
- [MOD-09-U02] In the side panel and the calendar picker the "Local" group comes last, after the accounts, and has no account address or account state. — *source: `DEC-178`*
- [MOD-09-U03] Deleting a local calendar that has events asks for confirmation, saying how many events will be deleted and that this cannot be undone; an empty one is deleted without a question. — *source: `DEC-178`*
- [MOD-09-U04] The welcome screen keeps connecting a Google account as its main action and says what it brings (synchronization, invitations, Google Meet); "Use without an account" is the secondary one. — *source: `DEC-179`*

## Non-functional requirements

- [MOD-09-NFR-01] Local calendars and events survive deleting or rebuilding the cache, removing any account and updating the app; they live in their own file under the app's data directory, and a damaged file is not silently replaced. — *source: `DEC-178`*

## Constraints and edge cases

- The app runs offline → local events are created, changed and deleted as usual; there is no "saving" mark (`MOD-05-U03`) and no offline refusal for them.
- No account is connected → the default event length is Google's default, 60 minutes, as there is no Google setting to read (`MOD-05-F09`); the side panel shows only the "Local" group; "Meet with…" and anything needing Google is unavailable and says it needs a Google account.
- No account is connected and every local calendar is disabled → creating an event says there is no calendar to create it in.
- The local file cannot be read at start (damaged, or from a newer version) → the app says so, does not overwrite it, and keeps working with Google calendars; local calendars are unavailable until the problem is solved.
- Deleting a local calendar is not undone with Ctrl+Z; the confirmation stands in for undo.
- A local calendar's name may repeat another calendar's name; it is not required to be unique.
- A local series' rule is always one the custom recurrence dialog can express, since it is made only in the app.

## Integrations

- **Platform notifications** — as for Google events (`MOD-07`); nothing else outside the app.

## Technical notes

- A separate SQLite file, `$XDG_DATA_HOME/<app ID>/local.sqlite3` (default `~/.local/share/calendar-buddy/…`, `calendar-buddy-dev` for a debug build, `DEC-164`), owned by the same worker as the cache; never touched by the cache's "delete a damaged file and start anew" (`DEC-87`).
- Local calendars are a pseudo-account in the domain model (a fixed account ID for "Local") so that the side panel, pickers, hidden/disabled/muted settings and colours keyed by account ID and calendar ID (`DEC-99`) reuse what exists; a local calendar's ID is generated by the app.
- Local events use the same event model as Google events (series as master plus changed or cancelled occurrences, RRULE), so recurrence expansion, layout and the three edit scopes are shared; writes go to the local store instead of the Google adapter, behind the calendar-provider interface (`NFR-14`).
- The local store has a schema version from the first release, so a later version can migrate it instead of losing it.
- Notifications of local events go through the same scheduler as Google reminders; snooze stays local as it is today.

## Out of scope / what not to worry about

- Synchronizing local calendars anywhere (Google, CalDAV, another device), or moving them to Google — `DEC-178`.
- Moving or copying an event between a local and a Google calendar — `DEC-178`.
- Backup, ICS import and export of local calendars — `DEC-178`, `DEC-37`.
- Guests, Meet, responses, attachments and special event types on local events — `DEC-178`.
- Managing Google calendars (creating, renaming, deleting) — still `DEC-21`.
- Default reminders of Google calendars set in the app — still `DEC-43`.

## Acceptance criteria

- [ ] On a first run, "Use without an account" opens the week view with a "Local" group holding "My calendar", and an event made there by clicking a time is shown at once, without network.
- [ ] In the settings two more local calendars are added, one is renamed and recoloured; the side panel and the calendar picker list all three under "Local", after the accounts.
- [ ] A weekly local event is edited "this and following" and then dragged; the series splits and the occurrence moves as a Google series would, and Ctrl+Z undoes the drag.
- [ ] A new local event gives a desktop notification 30 minutes before it starts; after changing the calendar's default to 10 minutes, a new event gives it 10 minutes before.
- [ ] The editor of a local event offers no guests, Meet, attachments, special types, busy/free or visibility, and its calendar picker lists only local calendars.
- [ ] Deleting a local calendar with events asks for confirmation with their count; afterwards neither the calendar nor its events remain.
- [ ] After deleting the cache file and restarting, local calendars and events are all still there.
- [ ] With a Google account and local calendars, removing the account leaves the local calendars and does not show the welcome screen; deleting the last local calendar then shows it.
