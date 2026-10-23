---
id: TC-04-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-04
covers: [US-04-02#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-04-03] At 100% scale in a 1080-pixel-high window the week view shows 8:00–18:00 without scrolling

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-04-02#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A window of 1920×1080 pixels at 100% scale.
- The week view is scrolled so that 8:00 is at the top of the hours.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Read the visible range of hours in the week view. | The hours from 8:00 to 18:00 are fully visible without scrolling. | app state |

## Pass criterion

- A working day from 8:00 to 18:00 fits in a 1080-pixel-high window at 100% scale.

## Cleanup

- The test settings are discarded with the test.
