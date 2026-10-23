---
id: TC-06-08 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-05#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-08] As a guest without edit rights, the user cannot change the event's time

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-05#4` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected; its contacts include "Adam Kowalski" (adam@example.com); "ewa@example.com" was invited to an earlier event.
- An invitation "Board" organized by someone else, where guests may not modify the event.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Try to drag "Board" to another time. | The event stays at its time and the app says only the organizer can change it. | app state |
| 2 | Open "Board". | Accept, Maybe and Decline are available; editing the time, details or guests is not. | app state |

## Pass criterion

- A guest without edit rights can answer but cannot change the event, and nothing is sent to Google.

## Cleanup

- The fake Google and the local store are discarded with the test.
