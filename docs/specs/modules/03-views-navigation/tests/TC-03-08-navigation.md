---
id: TC-03-08 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-05#1, US-03-05#2, US-03-05#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-08] Switching views keeps the date, next/previous/today move correctly, and the app restarts in the last view on today

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-05#1`, `US-03-05#2`, `US-03-05#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- The week view of 9–15 November is shown.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Switch to the month view. | November is shown. | app state |
| 2 | Choose next, then Today. | December is shown, then the month containing 11 November. | app state |
| 3 | Choose the 3-day view, move two periods ahead, and restart the app. | The app opens in the 3-day view starting today, 11 November. | app state |

## Pass criterion

- Navigation keeps the selected date across views and the app restarts in the last view on today.

## Cleanup

- The fixture store and settings are discarded with the test.
