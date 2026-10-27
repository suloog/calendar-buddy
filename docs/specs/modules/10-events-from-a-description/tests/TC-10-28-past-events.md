---
id: TC-10-28 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-04#2, US-10-04#8] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-28] A past event is not matched by name — "no such event", the text kept — but is changed when its details are open

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-04#2`, `US-10-04#8` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- "Spotkanie z Anią", Monday 19 October 11:00–12:00 in "Work"; nothing from today on matches it.

## Test data

| What | Value / source |
| --- | --- |
| First description | "przesuń spotkanie z Anią na piątek" — answer: change · the event named "spotkanie z Anią" · weekday Friday |
| Second description | "przesuń na piątek" — answer: change · the open event · weekday Friday |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | With no details open, send the first description. | No editing opens; the box says no such event was found and still holds the text. | app state |
| 2 | Go to the week of 19 October, open the event's details; press `q`, send the second description. | The editing of the 19 October event opens with Friday 30 October 11:00–12:00. | app state |

## Pass criterion

- Names reach only events from today on, while an open event is changed whatever its date.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Saving the change — `TC-10-26`.
