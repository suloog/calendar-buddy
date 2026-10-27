---
id: TC-10-20 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-03#6, US-10-03#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-20] "wrzuć mi spotkanie z Anią i Tomkiem jutro o 10 z Meetem" opens the full editor tomorrow at 10:00 with both guests and Meet on, and nobody is invited before Save

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-03#6`, `US-10-03#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- The account's contacts include "Ania Nowak" (`ania.nowak@example.com`) and "Tomek Wiśniewski" (`tomek.w@example.com`), each the only match for "Ania" and "Tomek".
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.

## Test data

| What | Value / source |
| --- | --- |
| Description | "wrzuć mi spotkanie z Anią i Tomkiem jutro o 10 z Meetem" |
| Fake model's answer | add · title "Spotkanie" · tomorrow · 10:00 · guests "Ania", "Tomek" · Meet asked for |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, send the description. | The full editor opens (not the "New event" window): "Spotkanie", Tuesday 27 October 10:00–10:30, calendar "Work", guests Ania Nowak and Tomek Wiśniewski, Google Meet on. | app state |
| 2 | Read the fake Google's record. | No event was created and no invitation sent. | fake Google record |
| 3 | Save. | The fake Google receives one new event with both guests, a Meet request and invitations to be sent. | fake Google record |

## Pass criterion

- One sentence gives the meeting with both people and Meet, and only Save sends it.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Names that fit several people or nobody — `TC-10-22`.
- Each lookup source — `TC-10-21`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
