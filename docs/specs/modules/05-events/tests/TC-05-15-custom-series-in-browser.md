---
id: TC-05-15 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-11#8] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-15] A series set in "Custom…" shows the same occurrences in Google Calendar in the browser

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-11#8` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The product owner's real Google account, connected in the app; Google Calendar open in the browser.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | In the app, create "Custom check" repeating every 2 weeks on Monday and Wednesday, 6 times. | The app shows 6 occurrences. | screenshot |
| 2 | Open the same weeks in Google Calendar in the browser. | The same 6 occurrences, and Google's repeat field reads "Every 2 weeks on Monday, Wednesday, 6 times". | screenshot |
| 3 | In the app, create "Last Friday" monthly on the last Friday, never ending, and check three months in the browser. | Google shows it on the last Friday of each month. | screenshot |

## Pass criterion

- Google shows the rules set in the app exactly as the app does.

## Cleanup

- Delete both series.
