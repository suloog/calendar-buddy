---
id: US-05-04 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F12, MOD-05-F16, MOD-05-F27, MOD-05-F28, MOD-05-U04] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-01] # story IDs that must be done first
---

# As a User, I can delete an event and undo my last change, so that mistakes cost nothing

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F12`, `F16`, `F27`, `F28`, `U04` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-05-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Deleting events and undoing the last change. A single event without guests is deleted without a question, with an Undo offered for a few seconds; the last change to an event can be undone with Ctrl+Z. Deleting an occurrence of a series is `US-05-07`. Notifying guests about a change is `MOD-06` (`MOD-06-F12`).

## Acceptance criteria

- [ ] The user can delete an event they are allowed to delete, from its details or with the keyboard.
- [ ] An event without guests that is not part of a series is deleted without a question, and an Undo notice is shown for 5 seconds; Undo brings the event back, also in Google.
- [ ] After moving, resizing or editing an event, Ctrl+Z — or Undo in the notice — restores the previous version, also in Google.

## Edge cases

- Undo after the event was changed elsewhere → the undo is refused with a message.

## Out of scope / what not to worry about

- A full undo history — `DEC-59`.
- Deleting occurrences of a series — `US-05-07`.

## Technical notes

- Undo is a compensating change sent to Google (re-create or patch back), not a local rollback (`MOD-05` *Technical notes*).
- Decisions: `DEC-59`, `DEC-60`.
