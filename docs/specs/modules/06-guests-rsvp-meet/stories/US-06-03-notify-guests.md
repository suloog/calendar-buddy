---
id: US-06-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-06
covers: [MOD-06-F12] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-06-01, US-05-03] # story IDs that must be done first
---

# As a User, I am asked whether to notify guests when I change or delete an event, so that I decide when people get an e-mail

|  |  |
| --- | --- |
| **Module** | `[MOD-06]` Guests, invitations, RSVP and Google Meet — `docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` |
| **Covers** | `MOD-06-F12` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-06-01`, blocked by `US-05-03` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The one question asked whenever an event with guests is changed — in the editor or by dragging — or deleted: send the guests an update or not. This is Google's own behaviour.

## Acceptance criteria

### Scenario: moving a meeting

- **Given** an event with guests that the user organizes
- **When** the user drags it to another time
- **Then** the app asks once whether to send the guests an update
- **And** answering "No" sends no e-mail, while answering "Yes" notifies the guests

- [ ] The same question is asked when saving a change in the editor and when deleting the event.

## Technical notes

- `sendUpdates=all` or `none` on update and delete (`MOD-06` *Technical notes*).
- Decision: `DEC-40`.
