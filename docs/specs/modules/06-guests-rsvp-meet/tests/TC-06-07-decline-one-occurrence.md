---
id: TC-06-07 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-05#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-07] Declining one occurrence of a recurring invitation leaves the other occurrences unanswered

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-05#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected; its contacts include "Adam Kowalski" (adam@example.com); "ewa@example.com" was invited to an earlier event.
- A weekly invitation "Team call" on Mondays 10:00, 4 occurrences from 16 November, awaiting the user's answer.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Choose Decline on the 2nd occurrence (23 November) and answer "this event". | The app asked "this event or all events". | app state |
| 2 | Check the occurrences in the app and in the fake Google. | The 2nd occurrence is declined; occurrences 1, 3 and 4 still await an answer. | app state |

## Pass criterion

- Answering one occurrence affects only that occurrence.

## Cleanup

- The fake Google and the local store are discarded with the test.
