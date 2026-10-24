---
id: US-06-08 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-06
covers: [MOD-06-F19, MOD-06-F20, MOD-06-U02] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-06-01] # story IDs that must be done first
---

# As a User, I can see when my guests are busy while picking a time, so that I avoid conflicts

|  |  |
| --- | --- |
| **Module** | `[MOD-06]` Guests, invitations, RSVP and Google Meet — `docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` |
| **Covers** | `MOD-06-F19`, `F20`, `U02` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-06-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

A simple preview of guests' availability in the event editor for the chosen day. Advanced scheduling (a multi-day grid, suggesting free slots) is not planned.

## Acceptance criteria

- [ ] Under the chosen time, the editor shows one strip per guest with their busy blocks for that day, with the chosen time highlighted across all strips.
- [ ] A guest busy at the chosen time is marked as a conflict.
- [ ] A guest who does not share availability is shown as "no data".
- [ ] The preview shows up to the first 20 guests.
- [ ] Without network, the preview is not shown.

## Out of scope / what not to worry about

- Advanced scheduling — `DEC-42`, `MOD-08`.

## Technical notes

- `freebusy.query` for the chosen day (`MOD-06` *Technical notes*).
- Decisions: `DEC-42`, `DEC-63`.
