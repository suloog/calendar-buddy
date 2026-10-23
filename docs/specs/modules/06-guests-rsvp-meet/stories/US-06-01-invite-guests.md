---
id: US-06-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-06
covers: [MOD-06-F01, MOD-06-F02, MOD-06-F03, MOD-06-F04, MOD-06-F06, MOD-06-F07, MOD-06-F08] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-02] # story IDs that must be done first
---

# As a User, I can invite people to an event with suggestions from my contacts, recent guests and my organization, so that inviting a colleague takes a few keystrokes

|  |  |
| --- | --- |
| **Module** | `[MOD-06]` Guests, invitations, RSVP and Google Meet — `docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` |
| **Covers** | `MOD-06-F01`, `F02`, `F03`, `F04`, `F06`, `F07`, `F08` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-05-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Adding and removing guests in the event editor, with suggestions as the user types, marking guests required or optional, and Google sending the invitations when the event is saved. Whether guests are notified of later changes is asked in `US-06-03`.

## Acceptance criteria

- [ ] The user adds a guest by typing an e-mail address.
- [ ] Typing part of a name or address suggests people from the account's Google Contacts and addresses the user has recently invited.
- [ ] On a Workspace account, the suggestions also include colleagues from the organization's directory.
- [ ] A guest can be marked as required or optional.
- [ ] Guests can be added to and removed from an existing event.
- [ ] Saving a new event with guests sends them invitations through Google.

## Edge cases

- An address with an invalid format → it is not added, and the field says why.
- The same guest is added twice, or the user adds their own address → added once / ignored.
- Directory access was not granted at sign-in → directory suggestions are unavailable for that account (`DEC-52`).

## Out of scope / what not to worry about

- Setting guest permissions — Google's defaults are kept (`DEC-63`).
- The notify question on later changes — `US-06-03`.

## Technical notes

- People API contacts, other contacts and directory search; recent addresses from cached events (`MOD-06` *Technical notes*).
- Decisions: `DEC-39`, `DEC-63`.
