---
id: TC-10-33 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-05#4, US-10-05#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-33] "Lunch o 13" opens at 13:00 with the day empty and the cursor in it, and Save waits for a day

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-05#4`, `US-10-05#1` |
| **Role** | User |
| **Type** | edge |
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
| Description | "Lunch o 13" |
| Fake model's answer | add · title "Lunch" · no day · 13:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, send the description. | The "New event" window opens with "Lunch", 13:00–13:30, the day empty with the cursor in it; Save is unavailable. | app state |
| 2 | Choose Wednesday 28 October. | Save is available. | app state |
| 3 | Save. | "Lunch" is on Wednesday 28 October 13:00–13:30 in "Work". | fake Google record |

## Pass criterion

- A missing day is asked for, never guessed, while the given time is filled in.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- A missing time — `TC-10-32`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
