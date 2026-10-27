---
id: TC-10-26 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-04#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-26] With "Sprint planning" (Monday 10:00–11:00) open, "przesuń na 15" opens its editing at 15:00–16:00, and nothing is saved before Save

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-04#5` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- A fake Google with one personal account `user@example.com` whose primary calendar "Work" is the default calendar, with a default event length of 30 minutes; it records every request it receives.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- "Sprint planning", Monday 26 October 10:00–11:00 in "Work", organized by the user, no guests.

## Test data

| What | Value / source |
| --- | --- |
| Description | "przesuń na 15" |
| Fake model's answer | change · the open event · start 15:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the details of "Sprint planning"; press `q`, send the description. | The editing of "Sprint planning" opens with Monday 26 October 15:00–16:00, the title unchanged. | app state |
| 2 | Read the fake Google's record. | No update was sent; the week view still shows the event at 10:00. | fake Google record + app state |
| 3 | Save. | The fake Google receives one update of "Sprint planning" to 15:00–16:00; the week view shows it at 15:00. | fake Google record |

## Pass criterion

- The open event is the one changed, its length kept, and only Save changes it.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- An event named instead of open — `TC-10-27`.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
