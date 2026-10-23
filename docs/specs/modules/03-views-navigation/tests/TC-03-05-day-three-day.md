---
id: TC-03-05 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-02#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-05] The day and 3-day views show the selected day and three days

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-02#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- Events exist on Wednesday 11, Thursday 12 and Friday 13 November.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Choose the day view. | Only Wednesday 11 November and its events are shown. | app state |
| 2 | Choose the 3-day view. | Wednesday 11 to Friday 13 November and their events are shown. | app state |

## Pass criterion

- The day view shows one day and the 3-day view three days starting from the selected day.

## Cleanup

- The fixture store and settings are discarded with the test.
