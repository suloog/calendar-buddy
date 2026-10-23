---
id: TC-02-04 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-02
covers: [US-02-03#1, US-02-03#6] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-02-04] A change made in Google appears after the next background synchronization, and Refresh brings it at once

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-02-03#1`, `US-02-03#6` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API with a controllable clock and an empty local store, set up by the test.
- One account is connected with an editable calendar holding the event "Standup".
- The first synchronization has finished.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Rename the event in the fake Google to "Standup 2" and advance the clock by 60 seconds. | The app shows the event as "Standup 2". | app state |
| 2 | Rename it in the fake Google to "Standup 3" and choose Refresh without advancing the clock. | The app shows the event as "Standup 3". | app state |

## Pass criterion

- A change in an editable calendar arrives within a minute on its own, and at once on Refresh.

## Cleanup

- The fake Google and the local store are discarded with the test.

## Out of scope for this test

- The 15-minute cadence of read-only calendars and the tray cadence — not in smoke depth.
