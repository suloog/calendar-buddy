---
id: TC-10-04 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-01#4, MOD-10-NFR-02] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: nfr # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-04] With the default address, a description reaches only this computer's Ollama, and Google receives nothing of it

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-01#4`, `MOD-10-NFR-02` |
| **Role** | User |
| **Type** | nfr |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- The test records every request the app makes, with its address.

## Test data

| What | Value / source |
| --- | --- |
| Description | "Kawa jutro o 10" |
| Fake model's answer | add · title "Kawa" · tomorrow · 10:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, type the description and press Enter. | The "New event" window opens for "Kawa", Tuesday 27 October 10:00–10:30. | app state |
| 2 | Close the window without saving; read the record of the app's requests. | Every request went either to the fake Google or to `http://localhost:11434`; the fake Ollama's record holds the description. | request record |
| 3 | Read the fake Google's record. | No request carries "Kawa jutro o 10" or any part of the model's answer. | fake Google record |

## Pass criterion

- With the default address, the description goes only to the local Ollama and nothing of it reaches Google before the user saves.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Guests named in a description, looked up in the Google contacts as when typing them, which `DEC-211` allows — `TC-10-21`.
- An address off this computer — `TC-10-03` (the warning).
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
