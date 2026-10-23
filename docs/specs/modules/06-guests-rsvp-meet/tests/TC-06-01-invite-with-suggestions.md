---
id: TC-06-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-01#2, US-06-01#6] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-01] Typing part of a name suggests people from contacts and recent guests, and saving sends the invitations

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-01#2`, `US-06-01#6` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected; its contacts include "Adam Kowalski" (adam@example.com); "ewa@example.com" was invited to an earlier event.
- A new event "Kickoff" on 13 November 10:00–11:00 is open in the editor.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Type "Ada" in the guest field. | "Adam Kowalski" is suggested. | app state |
| 2 | Choose him, then type "ewa" and choose "ewa@example.com". | Both are listed as guests. | app state |
| 3 | Save the event. | The fake Google received "Kickoff" with both guests and with invitations to be sent. | app state |

## Pass criterion

- Suggestions come from contacts and recent guests, and saving sends the invitations.

## Cleanup

- The fake Google and the local store are discarded with the test.
