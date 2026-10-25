---
id: US-06-09 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-06
covers: [MOD-06-F23, MOD-06-F24, MOD-06-F25, MOD-06-F26, MOD-06-F27, MOD-06-U05, MOD-06-U06] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-06-01, US-03-01, US-03-02] # story IDs that must be done first
---

# As a User, I can lay my colleagues' calendars over mine from the side panel, so that I find a time that suits everyone

|  |  |
| --- | --- |
| **Module** | `[MOD-06]` Guests, invitations, RSVP and Google Meet — `docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` |
| **Covers** | `MOD-06-F23`, `F24`, `F25`, `F26`, `F27`, `U05`, `U06` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-06-01`, `US-03-01`, `US-03-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Google Calendar's "Meet with…" in the side panel: the user picks people, their calendars appear in the day, 3-day and week views beside the user's own, and **Create event** starts a meeting with them. The editor's availability preview (`US-06-08`) stays as it is.

## Acceptance criteria

- [ ] Typing part of a colleague's name in the side panel's "Meet with…" field suggests them as adding guests does; choosing one adds them to a list under the field, with their colour and a way to remove them; a typed address can be chosen too.
- [ ] In the week view, a chosen person's events appear among the user's, in the person's colour, laid out beside the user's events at the same time like overlapping events; their all-day events are in the all-day strip.
- [ ] A person's event shows its title where their calendar shares details, and opens its details read-only; where the calendar shares only free/busy it is a "Busy" block; a person whose calendar shares nothing is marked in the list as one whose calendar cannot be seen.
- [ ] In the day view, the user and each chosen person are columns side by side, headed by name.
- [ ] **Create event** under the list opens the full editor with the chosen people as guests, on the selected day at the next full hour, with the default length and calendar.
- [ ] Moving to another day or week shows the chosen people's events for it.
- [ ] Removing a person takes their events out of the views; after the app is restarted the list is empty.
- [ ] Without network the field is unavailable and says why, and the people's calendars are not shown; they come back with the network.

## Out of scope / what not to worry about

- Suggesting free times, dragging over other people's time, other people's calendars in the month and agenda views — `DEC-173`.
- The editor's availability preview — unchanged, `US-06-08`.

## Technical notes

- `events.list` on the person's address for the shown range, falling back to `freeBusy.query`; held in memory, not in the cache (`MOD-06` *Technical notes*).
- Suggestions reuse those of `US-06-01`; the day view's columns and the week view's overlap layout reuse `calendar-core`'s week layout.
- Decisions: `DEC-173`, `DEC-174`.
