---
id: US-03-04 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F05] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-01] # story IDs that must be done first
---

# As a User, I can see my upcoming events as a list, so that I know what is next without reading a grid

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F05` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-03-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The agenda view: upcoming events from now on, grouped by day — the view behind "checking upcoming meetings" (`DEC-05`). The Join button in the agenda is `MOD-06`.

## Acceptance criteria

- [ ] The agenda lists events from now on, grouped by day, without empty days.
- [ ] More events load as the user scrolls down.
- [ ] With nothing ahead, the agenda says there are no upcoming events.

## Out of scope / what not to worry about

- The Join button — `MOD-06` (`MOD-06-F18`).

## Technical notes

- Decisions: `DEC-27`, `DEC-55`, `DEC-56`.
