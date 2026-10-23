---
id: US-03-06 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F10, MOD-03-F11, MOD-03-F12, MOD-03-U03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-01, US-02-01] # story IDs that must be done first
---

# As a User, I can see my calendars grouped by account in a side panel and hide any of them quickly, so that I see only what matters now

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F10`, `F11`, `F12`, `U03` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-03-01`, blocked by `US-02-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The side panel with each account and its calendars, the quick toggle that hides a calendar's events, and collapsing the panel. Disabling a calendar for good is `US-03-07`; a calendar's local colour is `US-02-07`.

## Acceptance criteria

- [ ] The side panel lists each connected account and, under it, its calendars.
- [ ] A quick toggle next to a calendar hides or shows its events; the calendar stays in the list.
- [ ] The panel can be collapsed and expanded, and the app remembers which after a restart.
- [ ] When the window becomes too narrow, the panel collapses on its own.

## Edge cases

- No calendar is visible (all hidden or disabled) → the view is empty and points to the side panel.

## Out of scope / what not to worry about

- Disabling calendars in the settings — `US-03-07`.
- Local calendar colours — `US-02-07`.

## Technical notes

- Decisions: `DEC-28`, `DEC-56`.
