---
id: TC-10-27 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-04#6, US-10-04#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-27] "przesuń spotkanie z Anią na piątek" opens that event's editing on Friday at the same time, and Save changes it without adding an event

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-04#6`, `US-10-04#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- "Spotkanie z Anią", Wednesday 28 October 11:00–12:00 in "Work", the only event from today on that matches; no event details open.

## Test data

| What | Value / source |
| --- | --- |
| Description | "przesuń spotkanie z Anią na piątek" |
| Fake model's answer | change · the event named "spotkanie z Anią" · weekday Friday |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, send the description. | The editing of "Spotkanie z Anią" opens with Friday 30 October 11:00–12:00. | app state |
| 2 | Save; read the fake Google's record. | One update of that event and no new event; the week shows one "Spotkanie z Anią", on Friday. | fake Google record + app state |

## Pass criterion

- A sentence that changes an event changes the event it names and never creates one.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Several matches — `TC-10-29`; none from today on — `TC-10-28`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
