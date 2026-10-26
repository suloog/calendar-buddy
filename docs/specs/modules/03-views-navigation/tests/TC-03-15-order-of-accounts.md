---
id: TC-03-15 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-11#1, US-03-11#2, US-03-11#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-15] Accounts and the local calendars are moved up and down in the settings, everywhere at once, and the order is kept

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-11#1`, `US-03-11#2`, `US-03-11#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Accounts A (connected first) and B are connected; the local store holds "My calendar"; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the settings' Accounts section. | Rows A, B, "Local calendars", each with Move up and Move down; A cannot move up, "Local calendars" cannot move down. | app state |
| 2 | Move "Local calendars" up twice. | Order "Local calendars", A, B in the settings; the side panel shows "Local" first, then A and B, each with its calendars in their own order; the calendar picker and the Calendars list follow. | app state |
| 3 | Move B up. | Order "Local calendars", B, A everywhere. | app state |
| 4 | Restart the app. | The side panel shows "Local", B, A. | app state |

## Pass criterion

- The groups follow the order chosen in the settings everywhere and keep it (`MOD-03-F30`, `MOD-03-U09`).

## Cleanup

- The fake Google, the cache, the settings and the local store are discarded with the test.
