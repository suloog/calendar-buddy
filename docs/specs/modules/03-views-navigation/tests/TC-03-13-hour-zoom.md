---
id: TC-03-13 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-10#1, US-03-10#2, US-03-10#3, US-03-10#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-13] Ctrl + wheel and Ctrl + plus / minus / 0 zoom the hours around the pointer or the middle, within 50–300 %

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-10#1`, `#2`, `#3`, `#5` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw); the week view in a window 1080 pixels high at 100 % scale, hour height 100 %.
- An event "Standup" on 11 November 9:00–9:15.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Point at 14:00 on Wednesday and send Ctrl + wheel up. | The hours are taller; 14:00 is still under the pointer (within 1 pixel). | app state |
| 2 | Send a pinch-out over the same point, then Ctrl + wheel down. | Taller after the pinch, shorter after the wheel; 14:00 stays under the pointer. | app state |
| 3 | Press Ctrl + 0, then Ctrl + plus twice. | 100 %, then 150 %; the time at the middle of the view is the same as before the keys; the top bar and side panel keep their size. | app state |
| 4 | Press Ctrl + plus ten times, then Ctrl + minus twenty times. | It stops at 300 %, then at 50 %; at 50 % the grid from 0:00 to 24:00 fits without scrolling. | app state |
| 5 | At 200 %, drag "Standup" down by one hour of the grid; click at 16:30 on Thursday. | "Standup" moves to 10:00–10:15; the quick entry opens at 16:30; the current-time line is at 10:00. | app state |

## Pass criterion

- The hours zoom within 50–300 % around the pointer or the middle, and every time on the grid stays right.

## Cleanup

- The local store is discarded with the test.
