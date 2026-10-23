---
id: US-03-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F01, MOD-03-F04] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-01] # story IDs that must be done first
---

# As a User, I can look at a single day or three days, so that I can focus on today or use a smaller screen

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F01`, `F04` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-03-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The day view and the 3-day view, both built on the week grid of `US-03-01`. Switching between views is `US-03-05`.

## Acceptance criteria

- [ ] A day view and a 3-day view are each available as a view the user picks.
- [ ] Both show the all-day strip, the current-time line and overlapping events the same way as the week view.

## Out of scope / what not to worry about

- Switching to the 3-day view automatically in a narrow window — `DEC-27`.

## Technical notes

- Decision: `DEC-27`.
