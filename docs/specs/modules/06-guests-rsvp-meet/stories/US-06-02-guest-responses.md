---
id: US-06-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-06
covers: [MOD-06-F05, MOD-06-U01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-02-01] # story IDs that must be done first
---

# As a User, I can see who is invited and how each guest answered, so that I know who is coming

|  |  |
| --- | --- |
| **Module** | `[MOD-06]` Guests, invitations, RSVP and Google Meet — `docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` |
| **Covers** | `MOD-06-F05`, `U01` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-02-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The guest list in an event's details, with each guest's answer — for events the user organizes and for invitations from others.

## Acceptance criteria

- [ ] An event's details list each guest's name, address and answer, shown as an icon: accepted, declined, maybe or awaiting.
- [ ] When a guest answers, the icon changes after the next synchronization.
- [ ] A guest list of more than 10 people is collapsed and can be expanded.

## Technical notes

- Decision: `DEC-63`.
