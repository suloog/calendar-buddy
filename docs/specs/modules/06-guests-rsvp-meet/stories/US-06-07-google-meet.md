---
id: US-06-07 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-06
covers: [MOD-06-F16, MOD-06-F17, MOD-06-F18] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-06-01] # story IDs that must be done first
---

# As a User, I can add Google Meet to a meeting and join it with one click, so that video calls start on time

|  |  |
| --- | --- |
| **Module** | `[MOD-06]` Guests, invitations, RSVP and Google Meet — `docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` |
| **Covers** | `MOD-06-F16`, `F17`, `F18` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-06-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Google Meet in the event editor — a switch that turns on by itself with the first guest — and a Join button that opens the meeting. The Join button in reminders is `MOD-07`.

## Acceptance criteria

- [ ] The editor has an "Add Google Meet" switch; it turns on when the first guest is added and is off for events without guests.
- [ ] A saved event with the switch on has a Google Meet link, also in Google Calendar in the browser.
- [ ] An event with a Meet link has a Join button in its details and in the agenda that opens the meeting in the browser.

## Edge cases

- Google cannot create a Meet link for the account → the event is saved without it, and the app says so.

## Out of scope / what not to worry about

- Join in notifications — `MOD-07`.

## Technical notes

- `conferenceData` with a `hangoutsMeet` create request (`MOD-06` *Technical notes*).
- Decisions: `DEC-38`, `DEC-61`.
