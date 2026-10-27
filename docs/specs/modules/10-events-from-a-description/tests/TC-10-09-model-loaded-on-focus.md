---
id: TC-10-09 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-09] Focusing the box asks Ollama to load the model for 5 minutes, and nothing is loaded at start

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#2` |
| **Role** | User |
| **Type** | happy-path |
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
| 1 | Start the app; read the fake Ollama's record. | It holds only the availability checks; no request to load the model. | fake Ollama record |
| 2 | Press `q`. | The fake Ollama receives one request to load `gemma4:e4b`, with no description, asking to keep it for 5 minutes. | fake Ollama record |
| 3 | Type the description and press Enter. | The request with the description also asks to keep the model for 5 minutes after it. | fake Ollama record |

## Pass criterion

- The model is loaded only when the box is focused, and every use asks Ollama to keep it for 5 minutes from then.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Ollama actually freeing the model's memory after 5 minutes — Ollama's own behaviour, outside the app.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
