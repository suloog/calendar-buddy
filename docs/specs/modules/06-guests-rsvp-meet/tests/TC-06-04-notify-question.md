---
id: TC-06-04 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-03#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-04] Dragging an event with guests asks once whether to notify them, and "No" sends no update

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-03#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected; its contacts include "Adam Kowalski" (adam@example.com); "ewa@example.com" was invited to an earlier event.
- Event "Sync" on 12 November 10:00–10:30, organized by the user, with guest Adam.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Drag "Sync" to 11:00. | The app asks once whether to send the guests an update. | app state |
| 2 | Answer "No". | The fake Google received the new time with guest notifications turned off. | app state |

## Pass criterion

- The question is asked once and "No" changes the event without notifying the guests.

## Cleanup

- The fake Google and the local store are discarded with the test.
