---
id: US-07-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-07
covers: [MOD-07-F09, MOD-07-F10] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-07-01, US-06-07] # story IDs that must be done first
---

# As a User, I can snooze or dismiss a reminder and join a Meet call straight from it, so that I act on a reminder without opening the app

|  |  |
| --- | --- |
| **Module** | `[MOD-07]` Reminders and notifications — `docs/specs/modules/07-notifications/REQUIREMENTS.md` |
| **Covers** | `MOD-07-F09`, `F10` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-07-01`, blocked by `US-06-07` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Actions on a reminder notification: dismissing it, snoozing it for 5 or 10 minutes, and joining the Google Meet call of the event.

## Acceptance criteria

- [ ] A notification offers Dismiss, Snooze 5 minutes and Snooze 10 minutes.
- [ ] A snoozed notification appears again after the chosen time; nothing changes in Google.
- [ ] A notification for an event with Google Meet has a Join button that opens the meeting in the browser.

## Edge cases

- The desktop's notification service does not support buttons → clicking the notification opens the event, where Join and the event's details are available.

## Out of scope / what not to worry about

- Keeping a snooze after the app is quit — not kept (snooze is local).

## Technical notes

- Notification actions over the desktop notification service (`MOD-07` *Technical notes*).
- Decisions: `DEC-38`, `DEC-45`, `DEC-64`.
