---
id: TC-04-05 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-04
covers: [US-04-03#1, US-04-03#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-04-05] Apart from calendar colours the interface uses neutral tones and one accent, and changing views has no transition animation

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-04-03#1`, `US-04-03#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The app runs on the product owner's desktop with real calendars.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Look at the main window in the light theme, then in the dark theme. | Apart from calendar and event colours, only neutral tones and one accent colour appear. | screenshot |
| 2 | Switch between the day, week and month views and move between weeks. | Each change appears at once, without a sliding or fading transition. | screenshot |

## Pass criterion

- The interface is restrained in colour and changes views without transition animations.

## Cleanup

- Nothing to clean up.
