---
id: TC-10-18 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#8, US-10-04#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: nfr # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-18] The fixed test set of adding and changing sentences passes its thresholds against the fake model

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#8`, `US-10-04#4` |
| **Role** | User |
| **Type** | nfr |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- The test set in the repo: the sentences of the test of 2026-10-27 and the change sentences, each with the expected event.
- A fake model that answers each sentence with the answer recorded for it.

## Test data

| What | Value / source |
| --- | --- |
| Size | at least 30 adding sentences and at least 10 changing ones, each kind in Polish and in English |
| Thresholds | date right in ≥ 95 %, every field right in ≥ 80 % (`MOD-10-NFR-04`) |
| Clock | the date and time the set's expected events are written for |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Count the sentences of the set. | At least 30 adding and 10 changing; both kinds have Polish and English sentences. | runner output |
| 2 | Run the set against the fake model. | The runner reports, per sentence, whether the date and every field are right, and in total the share with the date right ≥ 95 % and the share with every field right ≥ 80 %. | runner output |

## Pass criterion

- The set has the required size and the app's path from answer to event meets both thresholds.

## Cleanup

- Nothing is kept; the runner writes no files.

## Out of scope for this test

- The real model — `TC-10-19`; this test proves the set, the runner and the app's computing, not the model's understanding.
