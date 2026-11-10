---
id: TC-07-12 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-07#3, US-07-08#2, US-07-08#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-07-11] # test case IDs that must pass first
---

# [TC-07-12] With a real Google account, an invitation's notification answers it and opens it, and its reminders change from its details

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-07#3`, `US-07-08#2`, `US-07-08#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | `TC-07-11` |

## Preconditions

- The installed app with a connected Google account; a second Google account to invite from; notifications allowed.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | From the second account, invite the first to a weekly event, guests not allowed to modify it; close the app's window. | Within a synchronization a notification "Invitation: …" with Accept, Maybe and Decline. | screenshot |
| 2 | Click the notification. | The window opens on the invitation's day, the invitation picked. | screenshot |
| 3 | In the invitation's details, add a notification 1 hour before to one occurrence, choose "This event" and save. | Google Calendar in the browser shows the reminder on that occurrence only; the organizer sees no change. | screenshot |
| 4 | From the second account send another invitation; choose Decline on its notification. | The organizer gets the answer; the side panel no longer counts it. | screenshot |

## Pass criterion

- The buttons and the click work on the real desktop, and a guest's reminders reach Google.

## Cleanup

- Delete the test events in the second account.
