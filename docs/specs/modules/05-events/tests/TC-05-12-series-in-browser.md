---
id: TC-05-12 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-06#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [TC-05-07] # test case IDs that must pass first
---

# [TC-05-12] A series created in the app shows the same occurrences in Google Calendar in the browser

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-06#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | `TC-05-07` |

## Preconditions

- The product owner's account is connected.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | In the app, create "Series check" weekly on Monday and Wednesday at 18:00, ending after 4 occurrences. | The app shows 4 occurrences. | screenshot |
| 2 | Open Google Calendar in the browser. | The same 4 occurrences are there, on the same days and times. | screenshot |

## Pass criterion

- The browser shows the same occurrences as the app.

## Cleanup

- Delete the series.
