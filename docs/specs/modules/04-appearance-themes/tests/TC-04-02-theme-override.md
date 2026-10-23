---
id: TC-04-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-04
covers: [US-04-01#2, US-04-01#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-04-02] Choosing Light or Dark overrides the desktop and is kept after a restart

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-04-01#2`, `US-04-01#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- The desktop's simulated preference is dark.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the theme setting. | It offers System, Light and Dark. | app state |
| 2 | Choose Light. | The app uses the light theme although the desktop prefers dark. | app state |
| 3 | Restart the app. | The setting is still Light and the app uses the light theme. | app state |

## Pass criterion

- A chosen theme overrides the desktop and survives a restart.

## Cleanup

- The test settings are discarded with the test.
