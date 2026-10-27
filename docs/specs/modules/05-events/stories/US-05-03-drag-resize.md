---
id: US-05-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F13, MOD-05-F14, MOD-05-F16, MOD-05-U02, MOD-05-U03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-01] # story IDs that must be done first
---

# As a User, I can drag an event to another time and stretch it to change its length, so that rescheduling is one gesture

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F13`, `F14`, `F16`, `U02`, `U03` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-05-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Rescheduling with the mouse in the day, 3-day and week views: dragging an event to another time or day, and dragging its end to change its length. Dragging an occurrence of a series asks for the scope — `US-05-07`. Notifying guests about a change is `MOD-06` (`MOD-06-F12`).

## Acceptance criteria

- [ ] Dragging an event moves it in 15-minute steps, shows the new time while dragging, and keeps its length.
- [ ] An event can be dragged to another day in the 3-day and week views.
- [ ] Dragging the end of an event changes its length, in 15-minute steps.
- [ ] After dropping, the new time appears in Google Calendar in the browser.

## Edge cases

- An all-day event is dragged into the hours → it becomes a timed event of the default length; a timed event dragged into the all-day strip becomes all-day.

## Out of scope / what not to worry about

- Events the user may not change — `US-05-05`.
- Occurrences of a series — `US-05-07`.

## Technical notes

- Decision: `DEC-60`.
