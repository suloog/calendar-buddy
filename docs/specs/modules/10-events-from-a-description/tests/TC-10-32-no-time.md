---
id: TC-10-32 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-05#3, US-10-05#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-32] "Spotkanie z Michałem w sobotę" opens on Saturday 31 October with the time empty and the cursor in it, and Save waits for a time or all-day

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-05#3`, `US-10-05#1` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Contacts: "Michał Wójcik", the only match for "Michał".

## Test data

| What | Value / source |
| --- | --- |
| Description | "Spotkanie z Michałem w sobotę" |
| Fake model's answer | add · title "Spotkanie" · weekday Saturday · no time · guest "Michał" |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, send the description. | The full editor (a guest is named) opens on Saturday 31 October with Michał Wójcik; the time is empty with the cursor in it; Save is unavailable. | app state |
| 2 | Type 11:00. | The event is 11:00–11:30; Save is available. | app state |
| 3 | Send the description again; choose all-day instead of typing a time. | Save is available for an all-day event on 31 October. | app state |

## Pass criterion

- A missing time is asked for, never guessed, while the given day is filled in.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- A missing day — `TC-10-33`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
