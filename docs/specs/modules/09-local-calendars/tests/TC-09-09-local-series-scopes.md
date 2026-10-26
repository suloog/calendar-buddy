---
id: TC-09-09 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-03#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-09-08] # test case IDs that must pass first
---

# [TC-09-09] A local weekly series is changed for this and following events

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-03#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | `TC-09-08` |

## Preconditions

- The local store holds "Private"; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Create "Swimming" on Monday 16 November 18:00–19:00 and choose "Weekly on Monday". | It is shown on every Monday from 16 November. | app state |
| 2 | Open the occurrence of 30 November, change its start to 19:00, save. | The question "This event / This and following events / All events" is asked. | app state |
| 3 | Choose "This and following events". | 16 and 23 November stay at 18:00; 30 November and later Mondays are at 19:00–20:00. | app state |
| 4 | Delete the occurrence of 7 December and choose "This event". | Only 7 December disappears. | app state |
| 5 | Open "Custom…" on the occurrence of 14 December. | The dialog shows the series' rule, every week on Monday. | app state |

## Pass criterion

- Local series follow Google's three edit scopes.

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
