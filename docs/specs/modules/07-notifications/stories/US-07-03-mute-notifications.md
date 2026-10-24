---
id: US-07-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-07
covers: [MOD-07-F11, MOD-07-F12, MOD-07-U03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-07-01, US-03-06] # story IDs that must be done first
---

# As a User, I can turn notifications off or mute a single calendar, so that only reminders I care about reach me

|  |  |
| --- | --- |
| **Module** | `[MOD-07]` Reminders and notifications — `docs/specs/modules/07-notifications/REQUIREMENTS.md` |
| **Covers** | `MOD-07-F11`, `F12`, `U03` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-07-01`, blocked by `US-03-06` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

A global switch for all notifications in the settings, and muting the notifications of one calendar from its menu in the side panel without hiding the calendar.

## Acceptance criteria

- [ ] A switch in the settings turns all notifications off and on again.
- [ ] A calendar muted from its menu in the side panel still shows its events but produces no notifications.
- [ ] Turning notifications off or muting a calendar leaves the reminders in Google unchanged.

## Out of scope / what not to worry about

- Hiding or disabling calendars — `MOD-03`.

## Technical notes

- Decisions: `DEC-45`, `DEC-64`.
