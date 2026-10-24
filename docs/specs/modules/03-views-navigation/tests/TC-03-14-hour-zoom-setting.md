---
id: TC-03-14 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-10#6, US-03-10#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [TC-03-13] # test case IDs that must pass first
---

# [TC-03-14] The hour height is a slider in the settings, shared by the three views and kept after a restart

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-10#6`, `#7` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | `TC-03-13` |

## Preconditions

- Fixture data and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw); settings in a temporary directory; the week view at 100 %.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the settings, "Appearance". | A slider for the hour height reads 100 %. | app state |
| 2 | Move the slider to 175 % and close the settings. | The week view's hours are 1.75 times the usual height. | app state |
| 3 | Press Ctrl + minus in the week view and reopen the settings. | The slider reads 150 %. | app state |
| 4 | Switch to the day view, then the 3-day view. | Both show the hours at 150 %. | app state |
| 5 | Restart the app on the same settings. | The views open at 150 %, and the settings file holds it. | app state, settings file |

## Pass criterion

- One hour height, set from the views or the slider, is shared by the three views and survives a restart.

## Cleanup

- The temporary settings directory is removed.
