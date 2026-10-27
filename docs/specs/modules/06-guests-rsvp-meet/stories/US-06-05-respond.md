---
id: US-06-05 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-06
covers: [MOD-06-F11, MOD-06-F13, MOD-06-F15, MOD-06-U03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-06-04] # story IDs that must be done first
---

# As a User, I can accept, decline or answer maybe — for one occurrence or a whole series — while guest rules are respected, so that organizers know whether I come

|  |  |
| --- | --- |
| **Module** | `[MOD-06]` Guests, invitations, RSVP and Google Meet — `docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` |
| **Covers** | `MOD-06-F11`, `F13`, `F15`, `U03` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-06-04` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Answering invitations from an invitation's details and from the invitations list, including occurrences of recurring invitations, and what a guest may and may not change.

## Acceptance criteria

- [ ] An invitation's details open with Accept, Maybe and Decline at the top; the same answers are available in the invitations list.
- [ ] An answer given in the app appears in Google Calendar in the browser.
- [ ] Answering an occurrence of a recurring invitation asks whether the answer applies to this event or to all events.
- [ ] As a guest without edit rights, the user can answer but cannot change the event's time, details or guests.

## Edge cases

- No network → answering is unavailable (`US-02-05`).
- The organizer changes or cancels the event → the change arrives through synchronization; a cancelled event disappears.

## Out of scope / what not to worry about

- Answering with a comment, proposing another time — `MOD-08`.

## Technical notes

- Patching the user's own `responseStatus`; "this event" on the instance, "all events" on the master (`MOD-06` *Technical notes*).
- Decisions: `DEC-61`, `DEC-63`.
