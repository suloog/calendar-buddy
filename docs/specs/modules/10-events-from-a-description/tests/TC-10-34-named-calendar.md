---
id: TC-10-34 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-05#5, US-10-05#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-34] A calendar named in the description is chosen, matched by its name ignoring letter case

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-05#5`, `US-10-05#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- The account also owns the calendar "Praca"; the local calendar "My calendar" exists; "Work" is the default.

## Test data

| What | Value / source |
| --- | --- |
| "Przegląd auta w piątek o 9 w kalendarzu praca" | answer: calendar "praca" → "Praca" |
| "Kino jutro o 20 w kalendarzu MY CALENDAR" | answer: calendar "MY CALENDAR" → "My calendar" |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Send the first description. | The window opens for Friday 30 October 09:00 with "Praca" chosen and no note under the picker. | app state |
| 2 | Close it; send the second description. | The window opens with "My calendar" chosen and no note. | app state |

## Pass criterion

- A Google or local calendar named in any letter case is the one chosen.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- A name that fits no calendar that can take the event — `TC-10-35`.
