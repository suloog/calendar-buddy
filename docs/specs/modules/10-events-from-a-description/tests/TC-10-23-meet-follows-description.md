---
id: TC-10-23 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-03#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-23] Meet is on for "z Meetem", off for "bez Meeta", and turns on by itself with a guest when the description says nothing

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-03#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Contacts: "Ania Nowak", the only match for "Ania".

## Test data

| What | Value / source |
| --- | --- |
| "Rozmowa jutro o 12 z Meetem" | Meet asked for, no guests → full editor, Meet on |
| "Call tomorrow at 12 with Meet" | Meet asked for, no guests → full editor, Meet on |
| "Spotkanie z Anią jutro o 12 bez Meeta" | guest "Ania", Meet refused → full editor, Ania, Meet off |
| "Spotkanie z Anią jutro o 12" | guest "Ania", Meet not mentioned → full editor, Ania, Meet on, as when a first guest is added |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Send each description in turn, closing the editor without saving after each. | Each opens the full editor with the guests and the Meet switch of its row. | app state |

## Pass criterion

- The Meet switch follows what the description says, and the editor's own rule when it says nothing.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- An account that cannot add Meet (a story edge case, smoke depth).
