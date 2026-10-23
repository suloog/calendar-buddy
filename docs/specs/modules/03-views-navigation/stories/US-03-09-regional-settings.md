---
id: US-03-09 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F16, MOD-03-F17, MOD-03-F18] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-01] # story IDs that must be done first
---

# As a User, I see dates, times and time zones the way my system is set, so that the calendar matches the rest of my desktop

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F16`, `F17`, `F18` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-03-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The first day of the week, date and time formats and the time zone all come from the system; an event's details also show its own time zone when it differs.

## Acceptance criteria

- [ ] With a system locale whose week starts on Sunday, the day, 3-day and week views start on Sunday, and dates and times use the locale's formats.
- [ ] After the system's time zone changes, events are shown at their new local times.
- [ ] Opening an event stored in another time zone shows its time in that zone as well.

## Out of scope / what not to worry about

- A second time zone or choosing the display time zone by hand — `DEC-29`.

## Technical notes

- Decisions: `DEC-05`, `DEC-29`.
