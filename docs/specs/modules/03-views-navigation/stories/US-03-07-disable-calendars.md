---
id: US-03-07 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F13] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-06] # story IDs that must be done first
---

# As a User, I can disable calendars I never want to see, so that they do not clutter my lists

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F13` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-03-06` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Disabling and re-enabling calendars in the settings. A disabled calendar is gone from the side panel, the calendar pickers and the views; stopping its synchronization is `US-02-04`.

## Acceptance criteria

- [ ] In the settings, the user can disable a calendar and enable it again.
- [ ] A disabled calendar does not appear in the side panel, in the calendar pickers or in any view.

## Out of scope / what not to worry about

- Stopping and restarting synchronization for disabled calendars — `US-02-04`.

## Technical notes

- Decision: `DEC-28`.
