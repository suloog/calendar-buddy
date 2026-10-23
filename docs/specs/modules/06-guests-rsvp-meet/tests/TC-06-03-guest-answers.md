---
id: TC-06-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-02#1, US-06-02#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-03] A guest's answer is shown as an icon and changes after the next synchronization

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-02#1`, `US-06-02#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected; its contacts include "Adam Kowalski" (adam@example.com); "ewa@example.com" was invited to an earlier event.
- Event "Kickoff" with guests Adam (accepted) and Ewa (awaiting).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open Kickoff's details. | Adam and Ewa are listed with name, address and an icon: accepted for Adam, awaiting for Ewa. | app state |
| 2 | In the fake Google, set Ewa's answer to declined and let the next synchronization run. | Ewa's icon shows declined. | app state |

## Pass criterion

- Guests' answers are shown and follow changes from Google.

## Cleanup

- The fake Google and the local store are discarded with the test.
