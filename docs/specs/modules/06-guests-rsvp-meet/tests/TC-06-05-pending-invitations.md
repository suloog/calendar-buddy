---
id: TC-06-05 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-04#1, US-06-04#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-05] An invitation awaiting an answer is marked in the views and counted in the side panel, which opens the list

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-04#1`, `US-06-04#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected; its contacts include "Adam Kowalski" (adam@example.com); "ewa@example.com" was invited to an earlier event.
- Two invitations from others this week ("Board" and "Lunch") await the user's answer.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the week view. | "Board" and "Lunch" are marked as awaiting an answer (dashed outline, light fill). | app state |
| 2 | Read the side panel and choose the invitations counter. | It shows 2 and opens a list with "Board" and "Lunch". | app state |

## Pass criterion

- Invitations awaiting an answer are marked, counted, and listed.

## Cleanup

- The fake Google and the local store are discarded with the test.
