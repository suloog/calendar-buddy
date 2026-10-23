---
id: US-03-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F03, MOD-03-F20] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-01] # story IDs that must be done first
---

# As a User, I can look at a whole month, so that I can plan further ahead

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F03`, `F20` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-03-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The month view: each day of the month with its events, and a way to see a day's full list when it has more events than fit.

## Acceptance criteria

- [ ] The month view shows each day of the month with its events.
- [ ] A day with more events than fit shows how many more there are; choosing it opens the full list for that day.

## Technical notes

- Decision: `DEC-56`.
