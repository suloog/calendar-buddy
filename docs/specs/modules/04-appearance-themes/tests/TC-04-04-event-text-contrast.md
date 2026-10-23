---
id: TC-04-04 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-04
covers: [US-04-02#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: nfr # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-04-04] Event text on very light and very dark calendar colours reaches 4.5:1 contrast in both themes

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-04-02#3` |
| **Role** | User |
| **Type** | nfr |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Two calendars with one event each this week.

## Test data

| What | Value / source |
| --- | --- |
| Very light calendar colour | #FFF59D |
| Very dark calendar colour | #1A237E |
| Measure | contrast ratio (WCAG 2.1) between the event text colour and the event background |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | In the light theme, read the text and background colours of both events. | Both contrast ratios are at least 4.5:1. | app state |
| 2 | Switch to the dark theme and read them again. | Both contrast ratios are at least 4.5:1. | app state |

## Pass criterion

- Event text reaches at least 4.5:1 contrast on both extreme colours in both themes.

## Cleanup

- The test settings are discarded with the test.
