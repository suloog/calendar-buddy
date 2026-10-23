---
id: TC-07-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-02#1, US-07-02#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-03] Snoozing for 5 minutes shows the notification again after 5 minutes without changing anything in Google

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-02#1`, `US-07-02#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API, a fake desktop notification service that records what is shown, and a fixed, advanceable clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A is connected with the editable calendar "Work".
- Event "Review" at 10:00 with a 10-minute notification reminder; the clock is at 09:50 and the notification is shown.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Read the notification's actions. | Dismiss, Snooze 5 minutes and Snooze 10 minutes are offered. | app state |
| 2 | Choose Snooze 5 minutes and advance the clock to 09:55. | The notification for "Review" is shown again. | app state |
| 3 | Check what the fake Google received. | No change to "Review" or its reminders was sent. | app state |

## Pass criterion

- Snooze brings the notification back after 5 minutes and leaves Google untouched.

## Cleanup

- The fakes and the local store are discarded with the test.
