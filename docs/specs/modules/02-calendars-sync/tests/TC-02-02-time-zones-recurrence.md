---
id: TC-02-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-02
covers: [US-02-01#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-02-02] Events in another time zone, across a daylight-saving change, and moved or cancelled occurrences show at the right local times

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-02-01#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API with a controllable clock and an empty local store, set up by the test.
- The system time zone is Europe/Warsaw.
- One account is connected with the events below.

## Test data

| What | Value / source |
| --- | --- |
| Event 1 | 15:00–16:00 stored in Europe/London on a day in June → expected 16:00–17:00 local |
| Event 2 | 01:30–03:30 Europe/Warsaw on the last Sunday of March → expected to end at 03:30 local, lasting one hour |
| Series | weekly on Monday 09:00, 5 occurrences; the 3rd moved to 11:00, the 4th cancelled |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Let the synchronization finish and read the events of the relevant weeks. | Event 1 and Event 2 have the expected local times. | app state |
| 2 | Read the occurrences of the series. | Occurrences 1, 2 and 5 are at 09:00, occurrence 3 at 11:00, occurrence 4 is absent. | app state |

## Pass criterion

- Every event and occurrence is shown at the expected local time, and the cancelled occurrence is not shown.

## Cleanup

- The fake Google and the local store are discarded with the test.
