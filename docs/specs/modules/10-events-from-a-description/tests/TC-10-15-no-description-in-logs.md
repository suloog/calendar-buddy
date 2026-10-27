---
id: TC-10-15 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#6, MOD-10-NFR-01] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: nfr # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-15] Descriptions and the model's answers appear in no log and no file of the app

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#6`, `MOD-10-NFR-01` |
| **Role** | User |
| **Type** | nfr |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Logging at its most detailed level; the app's output captured.

## Test data

| What | Value / source |
| --- | --- |
| Description | "Spotkanie Zebra-7731 jutro o 10" — "Zebra-7731" is the marker searched for |
| Fake model's answer | add · title "Spotkanie Zebra-7731" · tomorrow · 10:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, send the description; close the window without saving. | The window opened with the title "Spotkanie Zebra-7731" and closed. | app state |
| 2 | Search the log files, the captured output and the config, cache and data directories for "Zebra-7731". | Not found anywhere. | file search |
| 3 | Send the description again and Save; search again. | Found only in the local calendar's file, as the saved event; still in no log and no other file. | file search |

## Pass criterion

- Only the event the user saved is kept; the description and the answer leave no trace.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- The 500-character cut — `TC-10-16`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
