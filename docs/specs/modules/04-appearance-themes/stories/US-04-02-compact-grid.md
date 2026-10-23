---
id: US-04-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-04
covers: [MOD-04-U01, MOD-04-U02, MOD-04-U04, MOD-04-NFR-01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-04-01, US-03-01] # story IDs that must be done first
---

# As a User, I see a compact, readable calendar grid, so that my whole working day fits on screen

|  |  |
| --- | --- |
| **Module** | `[MOD-04]` Appearance and themes — `docs/specs/modules/04-appearance-themes/REQUIREMENTS.md` |
| **Covers** | `MOD-04-U01`, `U02`, `U04`, `NFR-01` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-04-01`, blocked by `US-03-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The density and readability of the day, 3-day and week grids: a working day fits without scrolling, event blocks carry only title and time, and text stays readable on any calendar colour in both themes. The look is checked on `MOD-03`'s views, starting with the week view of `US-03-01`.

## Acceptance criteria

- [ ] At 100% scale, in a window 1080 pixels high, the week view shows 8:00–18:00 without scrolling.
- [ ] Event blocks in the day, 3-day and week views show only the title and the time.
- [ ] Text inside events of a very light and a very dark calendar colour is readable in both themes, with a contrast of at least 4.5:1.
- [ ] All interface text has a contrast of at least 4.5:1 against its background, in both themes.

## Edge cases

- The screen scale is 150% or 200% → the grid scales with it and keeps its proportions.

## Out of scope / what not to worry about

- How overlapping events are laid out — `MOD-03`.
- A density setting — not planned (`DEC-57`).

## Technical notes

- Event text colour picked from the relative luminance of the background (`MOD-04` *Technical notes*).
- Decisions: `DEC-57`, `DEC-58`; WCAG 2.1 AA.
