---
id: TC-10-06 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-01#6, US-10-01#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-06] The box is greyed out naming the reason — Ollama not answering, or the model not installed — with a link to the settings

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-01#6`, `US-10-01#7` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Events from a description on, with the default address and model.

## Test data

| What | Value / source |
| --- | --- |
| Case A | nothing answers at `http://localhost:11434` |
| Case B | a fake Ollama answers there but has only `llama3.2:3b` |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Case A: start the app and look at the side panel. | The description box is greyed out, cannot be typed in, and says "Ollama is not available at http://localhost:11434" with a link to the settings. | app state |
| 2 | Click the link. | The settings open at the "Events from a description" section. | app state |
| 3 | Case B: start the app and look at the side panel. | The box is greyed out and says "The model gemma4:e4b is not installed" with a link to the settings. | app state |

## Pass criterion

- An unavailable box says which of the two reasons it is and leads to the settings.

## Cleanup

- The fake Ollama and the directories are discarded with the test.

## Out of scope for this test

- Ollama starting or stopping while the app runs — `TC-10-07`.
