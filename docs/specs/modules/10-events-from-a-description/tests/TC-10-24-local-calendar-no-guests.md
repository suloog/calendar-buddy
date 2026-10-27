---
id: TC-10-24 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-03#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-24] For a local calendar the guests and Meet of a description are left out and the editor opens as for a local event

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-03#4` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- The local calendar "My calendar" is the default calendar.
- Contacts: "Ania Nowak", the only match for "Ania".

## Test data

| What | Value / source |
| --- | --- |
| Description | "Spotkanie z Anią jutro o 10 z Meetem" |
| Fake model's answer | add · title "Spotkanie" · tomorrow · 10:00 · guest "Ania" · Meet asked for |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, send the description. | The editor of a local event opens in "My calendar": "Spotkanie", Tuesday 27 October 10:00, with no guests and no Google Meet offered. | app state |
| 2 | Save; read the fake Google's record. | "Spotkanie" is in "My calendar" with no guests; the fake Google received no event and no invitation. | app state + fake Google record |

## Pass criterion

- A local calendar takes the described event without guests or Meet, and nobody is invited.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Offline or without an account — `TC-10-25`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
