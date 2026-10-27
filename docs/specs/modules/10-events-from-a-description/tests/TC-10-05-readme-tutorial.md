---
id: TC-10-05 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-01#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-05] The README tutorial installs Ollama and pulls `gemma4:e4b`, links ollama.com and says the app works without them

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-01#5` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The product owner's computer, Ubuntu 26.10 + KDE Plasma (`DEC-12`), with Ollama not installed (or a fresh user account without it).
- The app built from the commit under test.

## Test data

| What | Value / source |
| --- | --- |
| Model | `gemma4:e4b` |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Read the README's section on events from a description from start to end. | It explains installing Ollama and pulling `gemma4:e4b`, links to ollama.com, and says the app works without them, only this function is missing. | screenshot |
| 2 | Follow it step by step. | Ollama runs and lists `gemma4:e4b`. | screenshot |
| 3 | Start the app and look at the side panel. | The description box is available. | screenshot |

## Pass criterion

- Following the README alone gives a working description box.

## Cleanup

- Ollama and the model stay installed — they are the owner's to keep.

## Out of scope for this test

- The box when Ollama is missing — `TC-10-06`.
