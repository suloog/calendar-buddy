---
id: TC-05-04 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-04#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-04] Deleting a single event without guests needs no question and Undo restores it

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-04#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.
- An event "Gym" on 12 November 18:00–19:00, without guests, not part of a series.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Delete "Gym". | No question is asked; the event is gone and an Undo notice is shown for 5 seconds. | app state |
| 2 | Choose Undo within the 5 seconds. | "Gym" is back at 18:00–19:00 and exists again in the fake Google. | app state |

## Pass criterion

- Deletion of a single event needs no confirmation and Undo restores it in the app and in Google.

## Cleanup

- The fake Google and the local store are discarded with the test.
