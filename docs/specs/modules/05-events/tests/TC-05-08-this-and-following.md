---
id: TC-05-08 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-07#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-08] "This and following" from the 5th of 10 occurrences leaves 1–4 unchanged and moves 5–10

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-07#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.
- A weekly series "Standup", Mondays 09:00–09:15, 10 occurrences starting 16 November.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the 5th occurrence (14 December), change its start to 11:00 and choose "this and following events". | The app asked for the scope before saving. | app state |
| 2 | List all occurrences in the app and in the fake Google. | Occurrences 1–4 are at 09:00 and occurrences 5–10 at 11:00, in both. | app state |

## Pass criterion

- "This and following" changes exactly the chosen occurrence and the ones after it.

## Cleanup

- The fake Google and the local store are discarded with the test.
