---
id: US-02-07 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-02
covers: [MOD-02-F04, MOD-02-U04] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-02-01] # story IDs that must be done first
---

# As a User, I can give a calendar my own colour without changing it in Google, so that my calendars are easy to tell apart

|  |  |
| --- | --- |
| **Module** | `[MOD-02]` Calendars, synchronisation and offline cache — `docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` |
| **Covers** | `MOD-02-F04`, `U04` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-02-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

A local colour for a calendar, chosen from the calendar's menu in the side panel and kept only in the app; resetting brings back the Google colour. The side panel itself is `MOD-03`.

## Acceptance criteria

- [ ] From a calendar's menu in the side panel, the user picks a colour, and the calendar's events change to it in the app.
- [ ] The calendar's colour is unchanged in Google Calendar in the browser.
- [ ] "Reset" brings back the calendar's Google colour.

## Out of scope / what not to worry about

- Event colours (per event) — `MOD-05`.
- The side panel — `MOD-03`.

## Technical notes

- Local colours live in the app settings keyed by calendar ID (`MOD-02` *Technical notes*).
- Decision: `DEC-54`.
