---
id: TC-10-31 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-04#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-31] A described change of an event where the user is a guest without rights, or of a read-only calendar, opens its editing locked as today

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-04#3` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- "Przegląd kwartalny", Thursday 29 October 10:00–11:00, organized by someone else, the user a guest who may not change it.
- A read-only calendar "Święta w Polsce" with "Wszystkich Świętych", Sunday 1 November, all day.

## Test data

| What | Value / source |
| --- | --- |
| Description | "przesuń na 12" — answer: change · the open event · start 12:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the details of "Przegląd kwartalny"; press `q`, send the description. | Its editing opens as it does for that event without a description: the time locked at 10:00–11:00. | app state |
| 2 | Open the details of "Wszystkich Świętych"; send the description again. | It opens as a read-only event does today: nothing can be changed, and the app says why. | app state |
| 3 | Read the fake Google's record. | No change was sent. | fake Google record |

## Pass criterion

- A description never lets the user change what the editor would not.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Saving a change the user may make — `TC-10-30`.
