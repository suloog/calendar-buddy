---
id: TC-09-07 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-02#5, US-09-02#6, US-09-02#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-07] Deleting a local calendar asks with the count of its events, forgets the default and keeps the last one

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-02#5`, `US-09-02#6`, `US-09-02#7` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Account A is connected; the local store holds "Family" with 3 events, the default calendar, the empty "Sport" and "My calendar"; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Delete "Sport" in the settings. | It is deleted without a question. | app state |
| 2 | Delete "Family". | A confirmation says 3 events will be deleted and that this cannot be undone. | app state |
| 3 | Confirm; press Ctrl+Z. | "Family" and its events are nowhere in the app and Ctrl+Z does not bring them back; a new event starts in account A's primary calendar. | app state |
| 4 | Look at the **Delete** of "My calendar", now the only local calendar. | With only "My calendar" left, **Delete** is unavailable and says the last local calendar cannot be deleted. | app state |
| 5 | Remove account A. | The week view and the settings stay, with only "Local" and "My calendar"; no welcome screen. | app state |

## Pass criterion

- A local calendar goes away with its events only after the user confirms, and there is always one left.

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
