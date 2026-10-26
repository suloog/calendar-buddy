---
id: TC-09-04 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-01#8] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-04] An unreadable local file is reported and left as it is, while Google calendars keep working

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-01#8` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Account A is connected with "Standup" on 13 November; `local.sqlite3` in the data directory holds bytes that are not an SQLite database; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Start the app. | It says local calendars cannot be read and where the file is; the side panel shows account A without the "Local" group; "Standup" is shown. | app state |
| 2 | Compare the file with its bytes before the start. | It is unchanged. | file system |

## Pass criterion

- A damaged local file is never silently replaced (`MOD-09-NFR-01`).

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
