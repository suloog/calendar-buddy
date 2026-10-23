---
id: TC-05-05 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-04#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-05] Ctrl+Z after a drag restores the previous time, also in Google

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-04#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.
- An event "Call" on 12 November 10:00–11:00.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Drag "Call" to 14:00–15:00. | The fake Google holds 14:00–15:00. | app state |
| 2 | Press Ctrl+Z. | The app and the fake Google hold 10:00–11:00 again. | app state |

## Pass criterion

- Ctrl+Z reverts the last change in the app and in Google.

## Cleanup

- The fake Google and the local store are discarded with the test.
