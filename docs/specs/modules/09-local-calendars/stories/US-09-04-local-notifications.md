---
id: US-09-04 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-09
covers: [MOD-09-F13, MOD-09-F14, MOD-09-F15, MOD-09-U01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-09-02, US-07-01, US-07-02, US-07-04] # story IDs that must be done first
---

# As a User, I get notifications for my local events, with defaults I set for each local calendar, so that local events remind me as Google events do

|  |  |
| --- | --- |
| **Module** | `[MOD-09]` Local calendars — `docs/specs/modules/09-local-calendars/REQUIREMENTS.md` |
| **Covers** | `MOD-09-F13`, `F14`, `F15`, `U01` (default notifications) |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-09-02`, `US-07-01`, `US-07-02`, `US-07-04` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Local events give the same desktop notifications as Google events. As Google keeps default reminders per calendar, each local calendar keeps its own default notifications, set in the "Local calendars" section of the settings; in the editor a local event's notifications are changed as a Google event's are, as notifications only.

## Acceptance criteria

- [ ] A new local calendar has one default notification, 30 minutes before; a new timed event in it gives a desktop notification 30 minutes before it starts, with its title, start time and location, and only one.
- [ ] A local event's notification can be snoozed for 5 or 10 minutes and dismissed, and clicking it opens the event in the app.
- [ ] In the "Local calendars" section a calendar's default notifications can be added, changed and removed; events that use the defaults follow the change, and an all-day event that uses them gives no notification.
- [ ] In the editor a local event's notifications can be added and removed, a chosen number of minutes, hours or days before, up to 5, and set back to the calendar's defaults; there is no e-mail option.
- [ ] A muted or disabled local calendar gives no notifications, nor does any calendar when notifications are turned off in the settings.

## Out of scope / what not to worry about

- E-mail reminders — Google sends them, and only for Google events (`MOD-07-F06`).
- Default reminders of Google calendars set in the app — `DEC-43`.

## Technical notes

- Notifications of local events go through the same scheduler as Google reminders, including what happens to reminders that passed while the app was not running or the computer slept (`MOD-07-F20`).
- A local event stores "use defaults" or its own list, as Google's `reminders.useDefault` / `overrides` do, so the editor's section is shared with `US-07-04`; four weeks and five notifications stay the limits.
- All-day events count their own notifications back from midnight of their first day, and give none from the defaults (`DEC-137`).
- Decisions: `DEC-178`, `DEC-180`.
