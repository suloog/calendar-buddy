---
id: TC-10-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-01#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-02] Turned off, the description box, its `q` key and its shortcut entry are gone, and turned on they are back

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-01#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Look at the side panel; open the list of keyboard shortcuts. | The side panel has the field "Describe an event…" above "Meet with…"; the list of shortcuts includes `q`. | app state |
| 2 | In the settings, untick "Events from a description"; look at the side panel and the list of shortcuts; press `q` in the week view. | The field is gone from the side panel, `q` is gone from the list, and `q` changes nothing on screen. | app state |
| 3 | Tick it again; look at the side panel and the list; press `q`. | The field is back above "Meet with…", `q` is in the list again, and `q` puts the cursor in the field. | app state |

## Pass criterion

- Unticking the function removes the box, its key and its shortcut entry, and ticking it brings all three back.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- How the box itself behaves — `TC-10-08`.
