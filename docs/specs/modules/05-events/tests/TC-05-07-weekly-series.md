---
id: TC-05-07 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-06#1, US-05-06#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-07] A weekly series on Monday and Wednesday ending after 10 occurrences is created with the right rule

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-06#1`, `US-05-06#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Create "Training" on Monday 16 November 18:00–19:00, repeating weekly on Monday and Wednesday, ending after 10 occurrences. | The fake Google received a series with a weekly rule on Monday and Wednesday and a count of 10. | app state |
| 2 | List the occurrences in the app. | There are 10, from 16 November to 16 December, on Mondays and Wednesdays at 18:00. | app state |

## Pass criterion

- The series has the chosen pattern and ends after exactly 10 occurrences.

## Cleanup

- The fake Google and the local store are discarded with the test.
