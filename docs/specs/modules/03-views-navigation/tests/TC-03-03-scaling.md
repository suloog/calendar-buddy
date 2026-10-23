---
id: TC-03-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-01#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-03] The week view adapts to window resizing and to 100%, 150% and 200% scale

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-01#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The app runs on the product owner's desktop (Ubuntu 26.10, KDE Plasma) with real calendars.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | With the display at 100%, resize the window from full screen to half the screen width. | The week grid follows the window; nothing is cut off or overlapping. | screenshot |
| 2 | Set the display scale to 150%, then to 200%, and look at the week view each time. | Text and events scale sharply and the grid keeps its proportions. | screenshot |

## Pass criterion

- The week view stays usable and sharp across window sizes and the three scales.

## Cleanup

- Set the display scale back to its usual value.
