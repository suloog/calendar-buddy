---
id: TC-07-06 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-05#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-06] Launching the app a second time brings the running window forward instead of starting another copy

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-05#5` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- The app is running with its window hidden in the tray.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Launch the app a second time. | No second copy keeps running, and the running copy shows its window. | app state |

## Pass criterion

- Only one copy of the app runs, and a second launch shows the existing window.

## Cleanup

- Quit the app.
