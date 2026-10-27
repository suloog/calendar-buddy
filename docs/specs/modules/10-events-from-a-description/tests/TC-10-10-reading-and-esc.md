---
id: TC-10-10 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#3, MOD-10-NFR-03] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-10] While the model reads, the box shows "Reading…" and the views stay usable; Esc cancels and the late answer is dropped

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#3`, `MOD-10-NFR-03` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- The fake Ollama holds its answer until the test releases it.

## Test data

| What | Value / source |
| --- | --- |
| Description | "Kino jutro o 20" |
| Fake model's answer | add · title "Kino" · tomorrow · 20:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, type the description and press Enter. | The box shows "Reading…"; no window opens. | app state |
| 2 | Press the key for the next week, then the key for today. | The week view moves forward and back while the box still shows "Reading…". | app state |
| 3 | Press `q`, then Esc. | "Reading…" is gone; no window opens. | app state |
| 4 | Release the fake Ollama's answer. | No window opens and nothing in the views or the local calendar changes. | app state |

## Pass criterion

- Reading does not block the app, and a cancelled description never opens a window.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- How long reading takes on a real computer — there is no target (`MOD-10-NFR-03`); the wait without a GPU is the owner's manual check.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
