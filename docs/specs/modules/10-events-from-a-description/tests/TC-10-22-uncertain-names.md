---
id: TC-10-22 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-03#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-22] "Kasia" fitting two contacts and "Zenek" fitting nobody stay in the guest field with suggestions, and neither is a guest until picked

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-03#7` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Contacts: "Kasia Nowak" and "Kasia Lis"; nobody anywhere fits "Zenek".

## Test data

| What | Value / source |
| --- | --- |
| Description | "lunch z Kasią i Zenkiem w piątek o 13" |
| Fake model's answer | add · title "Lunch" · Friday · 13:00 · guests "Kasia", "Zenek" |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `q`, send the description. | The full editor opens for Friday 30 October 13:00 with no guests; the guest field holds "Kasia" with Kasia Nowak and Kasia Lis suggested, and "Zenek" with the usual suggestions. | app state |
| 2 | Pick Kasia Lis. | Kasia Lis is a guest; "Zenek" is still unresolved and not a guest. | app state |
| 3 | Remove "Zenek" and Save. | The fake Google receives the event with Kasia Lis as the only guest. | fake Google record |

## Pass criterion

- A name that is not certain never makes anyone a guest by itself.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- A name that fits one person — `TC-10-20`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
