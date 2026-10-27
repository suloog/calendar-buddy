---
id: TC-10-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-01#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-01] The settings' "Events from a description" section starts on, at `http://localhost:11434` with `gemma4:e4b`, and keeps changes after a restart

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-01#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.
- A fake Ollama at `http://localhost:11434` that has `gemma4:e4b`.

## Test data

| What | Value / source |
| --- | --- |
| Changed address | `http://127.0.0.1:11500` |
| Changed model | `llama3.2:3b` |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the settings. | An "Events from a description" section shows a ticked on/off checkbox, the Ollama address `http://localhost:11434` and the model `gemma4:e4b`. | app state |
| 2 | Change the address and the model to the values in Test data, untick the checkbox and close the settings. | The settings close without an error. | app state |
| 3 | Quit and start the app again; open the settings. | The checkbox is unticked, the address is `http://127.0.0.1:11500` and the model `llama3.2:3b`. | app state |

## Pass criterion

- The section starts with the three defaults, and all three changes are there after a restart.

## Cleanup

- The config directory is discarded with the test.

## Out of scope for this test

- What turning the function off does to the side panel — `TC-10-02`.
- The warning for an address off this computer — `TC-10-03`.
