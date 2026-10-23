---
id: TC-07-07 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-01#3, US-07-02#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-07] On KDE, clicking a notification opens its event and Join opens the Meet meeting

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-01#3`, `US-07-02#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The product owner's account is connected on KDE Plasma.
- A test event "Join check" with a Google Meet link starts in 12 minutes and has a 10-minute notification reminder.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Wait for the notification and click its body. | The app opens "Join check". | screenshot |
| 2 | On a second test event with Meet, choose Join in its notification. | The Meet meeting opens in the browser. | screenshot |

## Pass criterion

- The notification opens its event, and Join opens the meeting.

## Cleanup

- Delete the test events.
