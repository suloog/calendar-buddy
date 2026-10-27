---
id: TC-10-07 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-01#8] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-07] Ollama stopping while the box is open makes it unavailable keeping the text, and Ollama starting again makes it available without a restart

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-01#8` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.

## Test data

| What | Value / source |
| --- | --- |
| Description | "Kino jutro o 20" |
| Fake model's answer | add · title "Kino" · tomorrow · 20:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q` and type the description, without sending it. | The box is available and holds the text. | app state |
| 2 | Stop the fake Ollama; press Enter. | No window opens; the box turns greyed out with "Ollama is not available at http://localhost:11434" and still holds "Kino jutro o 20". | app state |
| 3 | Start the fake Ollama again; click the box. | The box is available again, with the text, and the app was not restarted. | app state |
| 4 | Press Enter. | The "New event" window opens for "Kino", Tuesday 27 October 20:00. | app state |

## Pass criterion

- The box follows Ollama going away and coming back while the app runs, and the description is never lost.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- The reasons' wording when Ollama is missing from the start — `TC-10-06`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
