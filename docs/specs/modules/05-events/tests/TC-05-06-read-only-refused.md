---
id: TC-05-06 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-05#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-06] An event of a read-only calendar cannot be dragged or edited and the app says why

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-05#1` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.
- A read-only calendar "Colleague" with an event "Their meeting" on 12 November 12:00–13:00.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Try to drag "Their meeting" to 14:00. | The event stays at 12:00–13:00 and a message says the calendar is read-only. | app state |
| 2 | Try to open "Their meeting" in the editor and change its title. | Editing is not offered, with the same reason; the fake Google received no change. | app state |

## Pass criterion

- Changes to an event of a read-only calendar are refused with a reason and nothing reaches Google.

## Cleanup

- The fake Google and the local store are discarded with the test.
