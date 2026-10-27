---
id: TC-10-25 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-03#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-25] Offline, or with no Google account, guests are not looked up because no Google calendar can take the event

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-03#5` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- Case A: the fake Google account of `TC-10-20` (default "Work", contact "Ania Nowak") and the local "My calendar"; the network is down — the fake Google refuses every request and records it.
- Case B: empty directories, no Google account, default "My calendar".

## Test data

| What | Value / source |
| --- | --- |
| Description | "Spotkanie z Anią jutro o 10 z Meetem" |
| Fake model's answer | add · title "Spotkanie" · tomorrow · 10:00 · guest "Ania" · Meet asked for |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Case A: send the description; read the fake Google's record. | The editor opens with the default calendar "Work", which cannot be saved to offline (`DEC-213`); the guest names are left out (`DEC-214`), and the fake Google received no contacts or directory search. | app state + fake Google record |
| 2 | Case B: send the description. | The editor of a local event opens in "My calendar" with no guests and no Meet. | app state |

## Pass criterion

- Without Google reachable or connected, a description adds no guests and searches for none.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Adding offline in general — `TC-10-17`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
