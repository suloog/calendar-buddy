---
id: TC-10-14 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#11] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-14] A description that is not an event, or an answer that cannot be used, opens no window, says no event was found and keeps the text

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#11` |
| **Role** | User |
| **Type** | negative |
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
| "jaka jest pogoda?" | answer: not an event |
| "Kino jutro o 20" | answer: text that is not in the fixed form |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, type "jaka jest pogoda?" and press Enter. | No window opens; the box says it found no event in the description and still holds "jaka jest pogoda?" with the cursor in it. | app state |
| 2 | Replace the text with "Kino jutro o 20" and press Enter. | No window opens; the same message; the box holds "Kino jutro o 20". | app state |
| 3 | Read the local calendar. | No event was added. | app state |

## Pass criterion

- Neither an unusable description nor an unusable answer opens a window or adds anything, and the text stays to be corrected.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Ollama not answering at all — `TC-10-07`.
