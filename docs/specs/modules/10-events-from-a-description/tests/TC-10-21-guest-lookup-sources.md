---
id: TC-10-21 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-03#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-21] Guests named in a description are found in the contacts, the Workspace directory and recent addresses, as when typing a guest

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-03#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fake Google with one Workspace account `user@firma.example`, default calendar "Work", default length 30 minutes.
- Contacts: "Ania Nowak". Directory: "Marek Zieliński" (`marek.zielinski@firma.example`). Recent guests: "Ola Kowalska" (`ola.k@example.org`), in neither of the others. Each is the only match for its name.

## Test data

| What | Value / source |
| --- | --- |
| Description | "Spotkanie z Anią, Markiem i Olą jutro o 11" |
| Fake model's answer | add · title "Spotkanie" · tomorrow · 11:00 · guests "Ania", "Marek", "Ola" |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, send the description. | The full editor opens with three guests: Ania Nowak, Marek Zieliński and Ola Kowalska. | app state |
| 2 | In a new event's editor, type "Marek" and then "Ola" in the guest field. | The suggestions are the same people the description found. | app state |

## Pass criterion

- Each of the three sources gives a guest from a description, the same as typing the name.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- The real Google contacts and directory — the owner's manual check named in the story.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
