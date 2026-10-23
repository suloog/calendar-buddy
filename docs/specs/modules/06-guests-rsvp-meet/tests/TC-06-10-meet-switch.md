---
id: TC-06-10 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-07#1, US-06-07#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-10] Adding the first guest turns Google Meet on, and the saved event has a Meet link

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-07#1`, `US-06-07#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected; its contacts include "Adam Kowalski" (adam@example.com); "ewa@example.com" was invited to an earlier event.
- A new event "Design review" is open in the editor; its "Add Google Meet" switch is off.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Add the guest Adam. | The "Add Google Meet" switch turns on. | app state |
| 2 | Save. | The fake Google received a request to create a Meet conference, and the event shows a Meet link. | app state |

## Pass criterion

- Meet is added automatically for a meeting with guests.

## Cleanup

- The fake Google and the local store are discarded with the test.
