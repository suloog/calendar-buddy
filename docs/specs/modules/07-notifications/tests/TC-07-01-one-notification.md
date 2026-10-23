---
id: TC-07-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-01#1, US-07-01#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-01] A 10-minute reminder produces exactly one notification with title, start time and location, even across synchronizations

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-01#1`, `US-07-01#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API, a fake desktop notification service that records what is shown, and a fixed, advanceable clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A is connected with the editable calendar "Work".
- Event "Review", 10:00–11:00, location "Room 2", with a 10-minute notification reminder.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Advance the clock to 09:50. | One notification is shown, with "Review", 10:00 and "Room 2". | app state |
| 2 | Run three more synchronizations and advance the clock to 09:55. | No further notification for "Review" is shown. | app state |

## Pass criterion

- The reminder produces exactly one notification with the expected content.

## Cleanup

- The fakes and the local store are discarded with the test.
