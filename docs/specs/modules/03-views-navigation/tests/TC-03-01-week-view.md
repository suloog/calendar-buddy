---
id: TC-03-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-01#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-01] The week view shows seven days of events, all-day events in a strip above the hours and free/busy calendars as busy blocks

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-01#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- The fixture week (9–15 November) holds a timed event on Monday 09:00–10:00, an all-day event on Tuesday, a 3-day event Thursday–Saturday, and a free/busy-only calendar with a block on Friday 13:00–14:00.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the week view. | Seven days, 9–15 November, are shown with their hours; the Monday event sits at 09:00–10:00 in the hours. | app state |
| 2 | Read the strip above the hours. | It holds the Tuesday all-day event and the Thursday–Saturday event spanning three days. | app state |
| 3 | Read Friday 13:00–14:00. | A "busy" block without a title is shown. | app state |

## Pass criterion

- The week view places timed, all-day, multi-day and free/busy events where the story says.

## Cleanup

- The fixture store and settings are discarded with the test.
