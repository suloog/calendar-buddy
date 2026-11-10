---
id: US-07-08 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-07
covers: [MOD-07-F22, MOD-07-F23, MOD-07-F24] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-07-01, US-06-04, US-06-05] # story IDs that must be done first
---

# As a User, I am notified of a new invitation and can answer it from the notification, so that I don't miss invitations while I am in other apps

|  |  |
| --- | --- |
| **Module** | `[MOD-07]` Reminders and notifications — `docs/specs/modules/07-notifications/REQUIREMENTS.md` |
| **Covers** | `MOD-07-F22`, `F23`, `F24` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-07-01`, `US-06-04`, `US-06-05` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

An invitation awaiting the user's answer, as the side panel counts them, is told once as a desktop notification with Accept, Maybe and Decline; a click opens it on its day. What was told is kept in the cache, so a restart repeats nothing and what came while the app was off is told at its start; a calendar's first synchronization tells nothing (`DEC-222`).

## Acceptance criteria

- [ ] An invitation that comes in a synchronization produces one notification "Invitation: <title>" with when it starts and who invites, and the buttons Accept, Maybe and Decline; later synchronizations and a restart do not repeat it.
- [ ] Accept, Maybe or Decline answers the invitation in Google and lets the organizer know; an invitation to a series is answered for all its events; the side panel no longer counts it.
- [ ] Clicking the notification opens the invitation in the window, on its day, also while the window is closed to the tray or the menu bar.
- [ ] An invitation that came while the app was off is told at the app's next start.
- [ ] The first synchronization of a newly connected account produces no notifications of invitations.
- [ ] Notifications turned off, a muted calendar or a disabled calendar produce no notification of an invitation.

## Edge cases

- An invitation is answered elsewhere before it is told → it is not told.
- An answered invitation awaits an answer again (the organizer asked again) → it is told again.
- The desktop's notifications have no buttons → the click opens the invitation, where it is answered (`DEC-64`).

## Out of scope / what not to worry about

- Notifications of changed or cancelled invitations — Google mails those.
- A switch of its own for notifications of invitations — the global switch and muting cover it.

## Technical notes

- The notification carries what it is of: a reminder or an invitation; the invitation's buttons are new action keys on Linux and a new category on macOS (`MOD-11-F10`).
- The engine tells invitations next to setting the alarms, from the events the views get; the cache keeps `announced` and `announced_calendars` (`DEC-222`).
- An answer from a notification goes through the engine as the details' answer does (`invitations::answer`), a series for all events.
- Decisions: `DEC-125`, `DEC-127`, `DEC-128`, `DEC-215`, `DEC-222`.
