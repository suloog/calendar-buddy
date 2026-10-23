---
id: US-05-05 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F17, MOD-05-F22] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-03] # story IDs that must be done first
---

# As a User, I am stopped from changing what I may not change and told why when Google refuses, so that I never break someone else's event

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F17`, `F22` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-05-03` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Guarding events the user may not change — events of read-only calendars and events where the user is a guest without edit rights — and handling Google refusing a change or a conflict with a change made elsewhere.

## Acceptance criteria

- [ ] Events of read-only calendars, and events the user is not allowed to change, cannot be dragged, resized, edited or deleted; trying shows why.
- [ ] When Google or the organizer refuses a change, a clear message is shown and the event returns to how it was.

## Edge cases

- The event was changed elsewhere since it was opened → the app reloads it, says so, and the user applies the change again.

## Out of scope / what not to worry about

- What a guest may still do (respond) — `MOD-06`.

## Technical notes

- A 412 answer on an ETag patch triggers the reload (`MOD-05` *Technical notes*).
- Decisions: `DEC-22`, `DEC-60`.
