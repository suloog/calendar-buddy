---
id: TC-02-08 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-02
covers: [US-02-06#1, US-02-06#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-02-08] A synchronization error on one calendar is shown on it while the others stay up to date, with the last-sync time

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-02-06#1`, `US-02-06#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API with a controllable clock and an empty local store, set up by the test.
- One account is connected with calendars "Work" and "Home"; the first synchronization finished at a known clock time.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Make the fake Google fail every request for "Work", change an event in "Home", and let the next synchronization run. | "Work" is in error, "Home" shows the change, and the indicator shows the time of the last successful synchronization. | app state |

## Pass criterion

- The error is confined to the failing calendar and the indicator reports the last successful synchronization time.

## Cleanup

- The fake Google and the local store are discarded with the test.
