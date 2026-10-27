---
id: US-06-04 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-06
covers: [MOD-06-F09, MOD-06-F10, MOD-06-F21, MOD-06-U04] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-02-01] # story IDs that must be done first
---

# As a User, I can spot invitations waiting for my answer and list them, so that none slips through

|  |  |
| --- | --- |
| **Module** | `[MOD-06]` Guests, invitations, RSVP and Google Meet — `docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` |
| **Covers** | `MOD-06-F09`, `F10`, `F21`, `U04` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-02-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Invitations the user has not answered yet: marked in every view and counted in the side panel, with the counter opening a list to answer from.

## Acceptance criteria

- [ ] An invitation awaiting the user's answer has a dashed outline and a light fill in every view.
- [ ] The side panel shows how many invitations await an answer; choosing it opens a list of them.
- [ ] Invitations in a calendar disabled in the settings are neither shown nor counted.

## Out of scope / what not to worry about

- Answering — `US-06-05`.

## Technical notes

- "Awaiting" = the user's attendee entry has `needsAction` (`MOD-06` *Technical notes*).
- Decisions: `DEC-41`, `DEC-61`, `DEC-63`.
