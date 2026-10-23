---
id: TC-06-09 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-06#1, US-06-06#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-09] A declined event is struck through and the switch hides and shows it

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-06#1`, `US-06-06#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected; its contacts include "Adam Kowalski" (adam@example.com); "ewa@example.com" was invited to an earlier event.
- The user has declined the invitation "Lunch" this week.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the week view. | "Lunch" is shown struck through and faded. | app state |
| 2 | Turn the "declined events" switch off, then on. | "Lunch" disappears, then is shown again. | app state |

## Pass criterion

- Declined events are shown struck through by default and the switch hides them.

## Cleanup

- The fake Google and the local store are discarded with the test.
