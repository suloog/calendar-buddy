---
id: US-09-05 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-09
covers: [MOD-09-F22, MOD-09-U06] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-09-02] # story IDs that must be done first
---

# As a User who uses only Google, I can hide the local calendars, so that "Local" does not clutter my calendar

|  |  |
| --- | --- |
| **Module** | `[MOD-09]` Local calendars — `docs/specs/modules/09-local-calendars/REQUIREMENTS.md` |
| **Covers** | `MOD-09-F22`, `U06` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-09-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

One checkbox in the "Local calendars" section of the settings takes the whole "Local" group out of sight, as disabling does for a single calendar, without deleting anything; while no Google account is connected the local calendars are always shown.

## Acceptance criteria

- [ ] The settings' "Local calendars" section has a "Show local calendars" checkbox, ticked by default.
- [ ] Unticking it takes "Local" out of the side panel and the calendar pickers, and local events out of every view, at once; local calendars give no notifications; the section still lists the local calendars for managing.
- [ ] Ticking it again brings everything back as it was, events included; the choice is kept after a restart.
- [ ] With a local calendar as the default calendar, while the local calendars are hidden new events go to the first account's primary calendar; once they are shown again, new events go to the local default again.
- [ ] With no Google account connected the checkbox is ticked and unavailable, and says local calendars are shown while no Google account is connected; removing the last account while they are hidden shows them again.

## Out of scope / what not to worry about

- Hiding single local calendars — the side panel's toggle and disabling in the settings do that already (`MOD-09-F06`).
- Deleting local calendars — `US-09-02`.

## Technical notes

- A `local = hidden` line in the settings file; the side panel, the pickers, the views and the reminders filter the "Local" pseudo-account as they filter disabled calendars (`DEC-102`), so nothing waits for the worker.
- Decisions: `DEC-188`.
