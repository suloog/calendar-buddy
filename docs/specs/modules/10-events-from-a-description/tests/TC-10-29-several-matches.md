---
id: TC-10-29 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-04#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-29] Three matching standups are listed with title, day, time and calendar; arrows and Enter open the chosen one, and Esc returns to the description

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-04#7` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Single events "Standup" on Tuesday 27, Wednesday 28 and Thursday 29 October, 09:30–09:45 in "Work", and one on Friday 23 October (past).

## Test data

| What | Value / source |
| --- | --- |
| Description | "przesuń standup na 10" |
| Fake model's answer | change · the event named "standup" · start 10:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, send the description. | Under the box in the side panel three rows are listed — the 27, 28 and 29 October standups, each with its title, day, 09:30 and "Work"; the 23 October one is not listed. | app state |
| 2 | Press Esc. | The list is gone; the box holds the description and has the cursor. | app state |
| 3 | Press Enter, then Down twice and Enter. | The editing of the Thursday 29 October standup opens with 10:00–10:15. | app state |

## Pass criterion

- When several events fit, the user picks the one changed, from the keyboard.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- A single match — `TC-10-27`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
