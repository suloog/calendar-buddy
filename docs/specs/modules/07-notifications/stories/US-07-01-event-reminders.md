---
id: US-07-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-07
covers: [MOD-07-F01, MOD-07-F02, MOD-07-F06, MOD-07-F07, MOD-07-F08, MOD-07-F13, MOD-07-F19, MOD-07-F20, MOD-07-U01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-02-01] # story IDs that must be done first
---

# As a User, I get a desktop notification before my events, as set in Google, so that I don't miss a meeting

|  |  |
| --- | --- |
| **Module** | `[MOD-07]` Reminders and notifications — `docs/specs/modules/07-notifications/REQUIREMENTS.md` |
| **Covers** | `MOD-07-F01`, `F02`, `F06`, `F07`, `F08`, `F13`, `F19`, `F20`, `U01` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-02-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Desktop notifications driven by the reminders set on events in Google, exactly once per reminder, following Google's rules for declined events, e-mail reminders and missed reminders. Actions on a notification are `US-07-02`; switching notifications off is `US-07-03`.

## Acceptance criteria

- [ ] An event with a 10-minute notification reminder produces one desktop notification 10 minutes before it starts, showing its title, start time and location.
- [ ] The notification is not repeated after several synchronizations.
- [ ] Clicking the notification opens the event in the app.
- [ ] E-mail reminders produce no desktop notification.
- [ ] Declined events and events of calendars disabled in the settings produce no notifications.
- [ ] Reminders whose time passed while the app was not running are not shown later.
- [ ] Reminders that fell while the computer slept are shown on waking if their event has not started yet.

## Edge cases

- An event is moved or deleted before its reminder → the reminder moves with it or is dropped.
- Several reminders fall in the same minute → each gets its own notification.
- The system time zone changes → reminders follow the events' real times.
- The desktop's "do not disturb" mode is on → the desktop decides what is shown.

## Out of scope / what not to worry about

- Snooze, dismiss and Join — `US-07-02`.
- Delivering reminders while the app is completely closed — `SRC-01-85`.

## Technical notes

- Scheduled from the local store, so they work offline; a delivered-reminder record prevents repeats (`MOD-07` *Technical notes*).
- Decisions: `DEC-28`, `DEC-43`, `DEC-64`.
