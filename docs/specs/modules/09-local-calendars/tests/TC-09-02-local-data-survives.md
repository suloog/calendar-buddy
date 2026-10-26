---
id: TC-09-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-01#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: nfr # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-09-01] # test case IDs that must pass first
---

# [TC-09-02] Local calendars and events survive a restart and a deleted cache

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-01#5` |
| **Role** | User |
| **Type** | nfr |
| **Automation** | auto |
| **Depends on** | `TC-09-01` |

## Preconditions

- The app was started without an account; "My calendar" holds "Dentist" on 13 November 10:00–11:00; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Quit and start the app. | The week view opens, not the welcome screen; "Dentist" is shown. | app state |
| 2 | Quit the app, delete `cache.sqlite3` from the cache directory, start the app. | The week view opens with "My calendar" and "Dentist"; `local.sqlite3` in the data directory is unchanged. | app state, file system |

## Pass criterion

- Local data lives outside the cache and nothing but the user deletes it (`MOD-09-NFR-01`).

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
