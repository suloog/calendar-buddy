---
id: US-02-04 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-02
covers: [MOD-02-F05, MOD-02-F06, MOD-02-F21] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-02-03] # story IDs that must be done first
---

# As a User, my list of calendars stays current, including calendars I lose access to or disable, so that nothing stale or unwanted is synchronized

|  |  |
| --- | --- |
| **Module** | `[MOD-02]` Calendars, synchronisation and offline cache — `docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` |
| **Covers** | `MOD-02-F05`, `F06`, `F21` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-02-03` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The list of calendars follows Google: new calendars appear, deleted ones disappear, a calendar the user lost access to is removed with a one-time notice, and calendars disabled in the settings are not synchronized. The disabling switch in the settings is `MOD-03`; this story honours it.

## Acceptance criteria

- [ ] A calendar created in Google appears at the next synchronization; a calendar deleted in Google disappears.

### Scenario: a calendar is unshared

- **Given** a colleague's shared calendar is shown
- **When** the colleague stops sharing it
- **Then** at the next synchronization it disappears with its cached events
- **And** the user gets a one-time notice

- [ ] A calendar disabled in the settings stays on the list of known calendars, but its events are not fetched or cached.
- [ ] Enabling a disabled calendar starts its first synchronization, and its events appear.

## Edge cases

- A calendar is deleted while the app is offline → it is removed at the next synchronization, with the same notice.

## Out of scope / what not to worry about

- The switch for disabling a calendar — `MOD-03` (`MOD-03-F13`).
- Creating, renaming or deleting calendars — done in the browser (`DEC-21`).

## Technical notes

- Decisions: `DEC-24`, `DEC-28`.
