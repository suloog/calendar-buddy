---
id: TC-05-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-01#1, US-05-01#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-01] Selecting 10:00–11:30 and typing a title creates the event in the default calendar

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-01#1`, `US-05-01#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.
- The week view of 9–15 November is shown.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Select Thursday 12 November 10:00–11:30 in the week view. | A quick-entry box opens with 10:00–11:30 on 12 November and the calendar "Personal". | app state |
| 2 | Type "Planning" and press Enter. | The fake Google received a new event "Planning", 12 November 10:00–11:30, in "Personal", and the app shows it. | app state |

## Pass criterion

- The quick-entry box creates the event at the selected time in the default calendar.

## Cleanup

- The fake Google and the local store are discarded with the test.
