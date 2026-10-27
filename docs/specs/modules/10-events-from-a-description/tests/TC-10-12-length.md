---
id: TC-10-12 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-12] A length in the description is used, and without one the event gets the default length of a new event

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.

## Test data

| What | Value / source |
| --- | --- |
| "Spotkanie zespołu jutro o 10, 45 minut" | answer: length 45 minutes → Tue 27 Oct 10:00–10:45 |
| "Code review tomorrow at 10 for two hours" | answer: length 120 minutes → Tue 27 Oct 10:00–12:00 |
| "Warsztat jutro o 10, godzina i kwadrans" | answer: length 75 minutes → Tue 27 Oct 10:00–11:15 |
| "Kawa jutro o 10" | answer: no length → Tue 27 Oct 10:00–10:30 (the account's default of 30 minutes) |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Send each of the first three descriptions in turn, closing the window after each. | Each window opens on Tuesday 27 October from 10:00 to the end given in Test data. | app state |
| 2 | Send "Kawa jutro o 10". | The window opens for 10:00–10:30. | app state |

## Pass criterion

- A given length sets the end, and a missing one gives the default length of the default calendar's account, not a fixed hour.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Days and start times — `TC-10-13`.
