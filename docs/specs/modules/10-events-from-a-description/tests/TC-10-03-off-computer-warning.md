---
id: TC-10-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-01#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-03] An Ollama address off this computer shows a warning that descriptions will leave it; `localhost`, `127.0.0.1` and `::1` show none

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-01#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.

## Test data

| What | Value / source |
| --- | --- |
| Local addresses | `http://localhost:11434`, `http://127.0.0.1:11434`, `http://[::1]:11434` |
| Addresses off this computer | `http://192.168.1.20:11434`, `http://gpu-box.lan:11434` |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the "Events from a description" section; enter each local address in turn. | No warning is shown for any of them. | app state |
| 2 | Enter `http://192.168.1.20:11434`. | The section warns that descriptions will leave this computer. | app state |
| 3 | Enter `http://gpu-box.lan:11434`. | The warning is shown. | app state |
| 4 | Enter `http://localhost:11434` again. | The warning is gone. | app state |

## Pass criterion

- The warning is shown exactly for the addresses that are not this computer.

## Cleanup

- The config directory is discarded with the test.

## Out of scope for this test

- Whether anything is actually sent off the computer — `TC-10-04`.
- An address that is not a valid URL (a story edge case, smoke depth).
