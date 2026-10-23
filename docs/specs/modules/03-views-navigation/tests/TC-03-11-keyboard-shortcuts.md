---
id: TC-03-11 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-08#1, US-03-08#2, US-03-08#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-11] Google Calendar's keys switch views, move between periods and show the shortcut list

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-08#1`, `US-03-08#2`, `US-03-08#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- The week view of 9–15 November has keyboard focus.

## Test data

| What | Value / source |
| --- | --- |
| Keys | `d`, `x`, `w`, `m`, `a`; `j`, `k`, `t`; `?` |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `d`, `x`, `w`, `m` and `a` in turn. | The day, 3-day, week, month and agenda views are shown in that order. | app state |
| 2 | Press `w`, then `j`, then `k`, then `j` and `t`. | The week of 16–22 November, then 9–15 November, then 16–22 November, then the week containing today are shown. | app state |
| 3 | Press `?`. | The list of keyboard shortcuts is shown. | app state |

## Pass criterion

- Google Calendar's keys drive views, navigation and the shortcut list.

## Cleanup

- The fixture store and settings are discarded with the test.
