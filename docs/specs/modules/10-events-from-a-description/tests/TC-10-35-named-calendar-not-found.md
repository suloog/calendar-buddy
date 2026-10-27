---
id: TC-10-35 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-05#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-35] A named calendar that does not exist or cannot take the event gives the default calendar with a note saying it was not found

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-05#5` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- A read-only calendar "Święta w Polsce"; no calendar named "Prywatny"; "Work" is the default.

## Test data

| What | Value / source |
| --- | --- |
| "Przegląd auta w piątek o 9 w kalendarzu Prywatny" | answer: calendar "Prywatny" |
| "Przegląd auta w piątek o 9 w kalendarzu Święta w Polsce" | answer: calendar "Święta w Polsce" |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Send the first description. | The window opens with "Work" and the note "Calendar "Prywatny" not found — using Work" under the calendar picker. | app state |
| 2 | Close it; send the second description. | The window opens with "Work" and the note "Calendar "Święta w Polsce" not found — using Work". | app state |

## Pass criterion

- A calendar that cannot take the event is never chosen, and the user is told the default was used instead.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Two calendars with the same name, and a disabled or hidden one (story edge cases, smoke depth).
