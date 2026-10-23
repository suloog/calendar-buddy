---
id: TC-06-12 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-08#1, US-06-08#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-12] The editor shows each guest's busy blocks for the day and marks a conflict with the chosen time

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-08#1`, `US-06-08#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account B (Workspace) is connected; in the fake Google, Anna Nowak is busy on 13 November 14:00–15:00 and Adam has no busy time that day.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open a new event on 13 November 14:30–15:00 and add Anna and Adam. | A strip per guest shows the day's busy blocks: 14:00–15:00 for Anna, none for Adam; the chosen time is highlighted. | app state |
| 2 | Read the conflict marks. | Anna is marked as a conflict; Adam is not. | app state |

## Pass criterion

- Guests' busy blocks are shown and a conflict with the chosen time is marked.

## Cleanup

- The fake Google and the local store are discarded with the test.
