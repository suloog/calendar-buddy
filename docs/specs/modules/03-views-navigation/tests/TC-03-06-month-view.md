---
id: TC-03-06 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-03#1, US-03-03#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-06] The month view shows each day's events and "+N more" for a full day, opening its list

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-03#1`, `US-03-03#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- November holds single events on several days and eight events on Monday 16 November.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Choose the month view. | November is shown, with each day's events; Monday 16 shows the events that fit and "+N more", where N is the number not shown. | app state |
| 2 | Choose "+N more" on Monday 16. | A list of all eight events of that day opens. | app state |

## Pass criterion

- The month view shows the events of each day and gives access to all events of a full day.

## Cleanup

- The fixture store and settings are discarded with the test.
