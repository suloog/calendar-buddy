---
id: TC-03-16 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-11#4, US-03-11#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-03-15] # test case IDs that must pass first
---

# [TC-03-16] An account connected after the order was chosen comes last, a removed one leaves the order, and the default calendar does not move with it

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-11#4`, `US-03-11#5` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | `TC-03-15` |

## Preconditions

- Accounts A (connected first) and B are connected, no default calendar chosen; the local store holds "My calendar"; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Look at the side panel. | A, B, "Local". | app state |
| 2 | Move B to the top; press `c`. | B, A, "Local"; the quick box still starts in account A's primary calendar. | app state |
| 3 | Connect account C. | B, A, "Local", C. | app state |
| 4 | Remove account A. | B, "Local", C. | app state |

## Pass criterion

- The order is only how groups are shown, and new or removed accounts fit into it (`DEC-189`).

## Cleanup

- The fake Google, the cache, the settings and the local store are discarded with the test.
