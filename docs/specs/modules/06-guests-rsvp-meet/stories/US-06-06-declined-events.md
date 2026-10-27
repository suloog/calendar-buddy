---
id: US-06-06 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-06
covers: [MOD-06-F14, MOD-06-F22] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-06-05] # story IDs that must be done first
---

# As a User, I see declined events struck through and can hide them with one switch, so that my calendar stays readable

|  |  |
| --- | --- |
| **Module** | `[MOD-06]` Guests, invitations, RSVP and Google Meet — `docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` |
| **Covers** | `MOD-06-F14`, `F22` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-06-05` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Events the user declined stay in the views, struck through and faded, and one easy switch shows or hides them.

## Acceptance criteria

- [ ] A declined event stays in the views, struck through and faded.
- [ ] One switch in the views hides and shows declined events; they are shown by default.

## Technical notes

- Decisions: `DEC-61`, `DEC-62`.
