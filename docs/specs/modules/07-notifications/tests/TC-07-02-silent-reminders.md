---
id: TC-07-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-01#4, US-07-01#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-02] Declined events and e-mail reminders produce no desktop notification

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-01#4`, `US-07-01#5` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API, a fake desktop notification service that records what is shown, and a fixed, advanceable clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A is connected with the editable calendar "Work".
- An invitation "Offsite" at 10:00 that the user declined, with a 10-minute notification reminder.
- An event "Report" at 10:00 with only an e-mail reminder 10 minutes before.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Advance the clock to 09:55. | No notification was shown for "Offsite" or for "Report". | app state |

## Pass criterion

- Neither a declined event nor an e-mail-only reminder produces a desktop notification.

## Cleanup

- The fakes and the local store are discarded with the test.
