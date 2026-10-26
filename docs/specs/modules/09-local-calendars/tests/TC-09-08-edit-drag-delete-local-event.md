---
id: TC-09-08 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-03#1, US-09-03#2, US-09-03#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-08] A local event is edited, dragged, resized, deleted and undone offline

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-03#1`, `US-09-03#2`, `US-09-03#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- No network; the local store holds "Private" with "Dentist" on Friday 13 November 10:00–11:00; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open "Dentist" in the editor; change the title to "Dentist — check-up", the end to 11:30, the time zone to Europe/London, the colour to Tomato; save. | The event shows the new title, 10:00–11:30 local time in Tomato, without a "saving" mark. | app state |
| 2 | Drag it to Monday 16 November 14:00. | It moves in 15-minute steps and lands at 14:00–15:30. | app state |
| 3 | Drag its end down by 30 minutes. | It lasts until 16:00. | app state |
| 4 | Press Ctrl+Z. | It ends at 15:30 again. | app state |
| 5 | Delete it. | It disappears with no question, and an Undo notice is shown. | app state |
| 6 | Choose Undo. | It is back at 14:00–15:30. | app state |

## Pass criterion

- A local event changes as a Google event does, at once and without network.

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
