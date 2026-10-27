---
id: TC-10-11 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#9, US-10-02#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-11] "Trening wieczorem" opens the "New event" window today at 18:00 with the default length in "My calendar", as a new event, and nothing is saved before Save

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#9`, `US-10-02#5` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- With no account, a new event lasts 60 minutes (`US-09-01#6`).

## Test data

| What | Value / source |
| --- | --- |
| Description | "Trening wieczorem" |
| Fake model's answer | add · title "Trening" · today · evening, no hour · no length |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, type the description and press Enter. | The "New event" window (not the full editor) opens with the title "Trening", Monday 26 October 18:00–19:00, calendar "My calendar". | app state |
| 2 | Open the full editor from the window. | Location and description are empty, the repeat is "Does not repeat", the reminders and the colour are those of a new event made by hand in "My calendar". | app state |
| 3 | Before saving, read the local calendar. | It holds no "Trening". | app state |
| 4 | Save. | "Trening" is on Monday 26 October 18:00–19:00 in "My calendar". | app state |

## Pass criterion

- The sentence gives a new event exactly like one made by hand at 18:00 today, and only Save stores it.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Other parts of the day and other days — `TC-10-13`.
- Lengths given in the description — `TC-10-12`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
