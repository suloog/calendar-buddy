---
id: US-05-06 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F18, MOD-05-F19] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-02] # story IDs that must be done first
---

# As a User, I can make an event repeat daily, on weekdays, weekly, monthly or yearly, so that regular meetings are set once

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F18`, `F19` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-05-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Making an event recurring in the editor with the presets Google offers for everyday use, and choosing how the series ends. Changing or deleting occurrences is `US-05-07`.

## Acceptance criteria

- [ ] In the editor the user makes an event repeat daily, every weekday, weekly on chosen days, monthly or yearly.
- [ ] The series ends on a chosen date, after a chosen number of occurrences, or never.
- [ ] A series created in the app shows the same occurrences in Google Calendar in the browser.

## Out of scope / what not to worry about

- Google's repeat menu and custom recurrence (every N weeks, the second Tuesday…) — `US-05-11` (`DEC-165`).

## Technical notes

- Presets map to RRULEs as listed in `MOD-05` *Technical notes*; test-first (constitution).
- Decision: `DEC-36`.
