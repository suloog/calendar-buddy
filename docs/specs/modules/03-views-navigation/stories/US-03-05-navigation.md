---
id: US-03-05 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F06, MOD-03-F07, MOD-03-F08, MOD-03-F09, MOD-03-F15] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-01] # story IDs that must be done first
---

# As a User, I can move between views and dates quickly, so that I get to any day in a moment

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F06`, `F07`, `F08`, `F09`, `F15` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-03-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Getting around: switching views, moving to the next or previous period, coming back to today, the mini calendar, and starting where the user left off. Keyboard shortcuts for all of this are `US-03-08`.

## Acceptance criteria

- [ ] Switching between views keeps the selected date in view.
- [ ] Next and previous move by the period of the current view; "Today" brings the user back to today.
- [ ] The mini calendar marks the period on screen and jumps to any date the user picks.
- [ ] After a restart, the app opens in the view used last, on today's date.

## Out of scope / what not to worry about

- Keyboard shortcuts — `US-03-08`.

## Technical notes

- View state lives in the app settings (`MOD-03` *Technical notes*).
- Decision: `DEC-55`.
