---
id: TC-04-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-04
covers: [US-04-01#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-04-01] With "System", the app starts dark on a dark desktop and turns light at once when the desktop switches

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-04-01#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- The theme setting is "System".
- The desktop's light/dark preference is simulated by the test and set to dark.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Start the app. | The app uses the dark theme. | app state |
| 2 | Switch the simulated desktop preference to light while the app runs. | The app uses the light theme without a restart. | app state |

## Pass criterion

- With "System", the app's theme follows the desktop preference, also while running.

## Cleanup

- The test settings are discarded with the test.
