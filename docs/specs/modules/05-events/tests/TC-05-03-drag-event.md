---
id: TC-05-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-03#1, US-05-03#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-03] Dragging an event moves it in 15-minute steps, keeps its length and saves the new time

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-03#1`, `US-05-03#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.
- An event "Call" on 12 November 10:00–11:00 without guests.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Drag "Call" down by 40 minutes and drop it. | It lands at 10:45–11:45 (rounded to 15-minute steps, length unchanged). | app state |
| 2 | Check what the fake Google received. | The event was updated to 10:45–11:45. | app state |

## Pass criterion

- A dragged event snaps to 15 minutes, keeps its length, and the new time is saved to Google.

## Cleanup

- The fake Google and the local store are discarded with the test.
