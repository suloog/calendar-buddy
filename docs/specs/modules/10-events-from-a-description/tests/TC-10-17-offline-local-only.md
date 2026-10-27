---
id: TC-10-17 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-17] Offline only a local calendar can take a described event, as when adding by hand, and with no Google account the default local calendar takes it

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#7` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- Case A: the fake Google account of `TC-10-12` (default calendar "Work") and the local calendar "My calendar"; the network is down — the fake Google refuses every request and records it.
- Case B: empty directories, no Google account ("Use without an account"), default "My calendar".

## Test data

| What | Value / source |
| --- | --- |
| Description | "Kino jutro o 20" |
| Fake model's answer | add · title "Kino" · tomorrow · 20:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Case A: send the description. | The "New event" window opens for Tuesday 27 October 20:00 with the default calendar "Work" (`DEC-213`); saving to it does not succeed, as when adding by hand offline. | app state |
| 2 | Choose "My calendar" in the picker and Save. | "Kino" is in "My calendar"; the fake Google's record holds no request to create it. | app state + fake Google record |
| 3 | Case B: send the description and Save. | The window opened with "My calendar"; "Kino" is saved there. | app state |

## Pass criterion

- A description adds offline and without an account exactly where adding by hand can.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Guests and Meet offline or without an account — `TC-10-25`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
