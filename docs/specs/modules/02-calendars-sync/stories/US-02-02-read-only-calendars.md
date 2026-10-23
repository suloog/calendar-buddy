---
id: US-02-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-02
covers: [MOD-02-F19, MOD-02-F20, MOD-02-U01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-02-01] # story IDs that must be done first
---

# As a User, I can see read-only, subscribed and free/busy calendars without accidentally changing them, so that holidays and colleagues' calendars are there but safe

|  |  |
| --- | --- |
| **Module** | `[MOD-02]` Calendars, synchronisation and offline cache — `docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` |
| **Covers** | `MOD-02-F19`, `F20`, `U01` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-02-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Calendars the user cannot edit — subscribed calendars such as holidays, colleagues' calendars shared read-only, and calendars shared as free/busy only — are shown like the others but visibly protected. Blocking edits on them in the editor and by dragging is enforced by `MOD-05`; this story shows the state.

## Acceptance criteria

- [ ] A subscribed holiday calendar and a colleague's read-only calendar are shown like the other calendars.
- [ ] Their events are visibly marked as not editable.
- [ ] A calendar shared as free/busy only shows "busy" blocks, without titles or details.

## Edge cases

- A calendar's access drops from editable to read-only → it is marked read-only at the next synchronization.

## Out of scope / what not to worry about

- Refusing drags and edits of these events — `MOD-05` (`MOD-05-F17`).

## Technical notes

- Access level comes from the calendar list's access role (`MOD-02` *Technical notes*).
- Decision: `DEC-22`.
