---
id: TC-02-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-02
covers: [US-02-01#1, US-02-01#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-02-01] Calendars of two accounts, including a subscribed one, are fetched and shown together in their Google colours

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-02-01#1`, `US-02-01#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API with a controllable clock and an empty local store, set up by the test.
- Account A has two own calendars; account B has one own calendar and a subscribed holiday calendar; each calendar has one event this week and its own Google colour.
- Both accounts are connected.

## Test data

| What | Value / source |
| --- | --- |
| Week shown | the week containing the fake clock's "today" |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Let the first synchronization finish. | All four calendars are known to the app. | app state |
| 2 | Open the week view. | The week's view contains the four events, each drawn in its calendar's Google colour. | app state |

## Pass criterion

- All four calendars of both accounts are shown together, each in its Google colour.

## Cleanup

- The fake Google and the local store are discarded with the test.

## Out of scope for this test

- The cache window and the first-sync load — not in smoke depth.
