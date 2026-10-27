---
id: TC-10-30 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-04#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-30] Saving a described change of a series occurrence with a guest asks for the scope and whether to notify the guest

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-04#3` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- A weekly series "Weekly sync", Wednesdays 14:00–15:00 from 7 October in "Work", organized by the user, with the guest Ania Nowak.

## Test data

| What | Value / source |
| --- | --- |
| Description | "przesuń na 16" |
| Fake model's answer | change · the open event · start 16:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the details of the 28 October occurrence; press `q`, send the description. | Its editing opens with 16:00–17:00. | app state |
| 2 | Save. | The app asks for the scope: this event, this and following, all. | app state |
| 3 | Choose "This event". | The app asks whether to notify the guests. | app state |
| 4 | Choose not to notify. | The fake Google receives a change of the 28 October occurrence only, without notifying the guests. | fake Google record |

## Pass criterion

- A described change is saved through the same questions as a change made in the editor.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Events the user cannot change — `TC-10-31`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
