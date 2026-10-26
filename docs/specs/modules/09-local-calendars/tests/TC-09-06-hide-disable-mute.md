---
id: TC-09-06 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-02#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-06] A local calendar is hidden, disabled and muted as a Google calendar is

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-02#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- The local store holds "Family" with "Dinner" on 12 November 09:20–10:00, which has a notification 10 minutes before; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Untick "Family" in the side panel. | "Dinner" disappears from the views; "Family" stays listed. | app state |
| 2 | Tick it again; mute it from its menu; let the clock reach 09:10. | "Dinner" is shown; no notification is shown. | app state, notification record |
| 3 | Disable "Family" in the settings. | It disappears from the side panel and the calendar picker, and "Dinner" from the views. | app state |
| 4 | Enable it again. | "Family" and "Dinner" are back. | app state |

## Pass criterion

- Hiding, disabling and muting work on local calendars as on Google ones.

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
