---
id: TC-10-19 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#8, US-10-04#4, MOD-10-NFR-04] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: nfr # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-10-18] # test case IDs that must pass first
---

# [TC-10-19] The fixed test set passes with the date right in at least 95 % and every field in at least 80 % against a real Ollama with `gemma4:e4b`

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#8`, `US-10-04#4`, `MOD-10-NFR-04` |
| **Role** | User |
| **Type** | nfr |
| **Automation** | manual-only |
| **Depends on** | `TC-10-18` |

## Preconditions

- The product owner's computer with Ollama running and `gemma4:e4b` pulled.
- The app's code at the commit under test.
- `TC-10-18` passes.

## Test data

| What | Value / source |
| --- | --- |
| Model | `gemma4:e4b`, the default |
| Thresholds | date ≥ 95 %, every field ≥ 80 %, adding and changing sentences together |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | List the models Ollama has. | `gemma4:e4b` is listed. | console |
| 2 | Run the test set's hand-run test against the live Ollama. | The runner reports the date right in ≥ 95 % of the sentences and every field right in ≥ 80 %. | console |

## Pass criterion

- With the default model the real reading of descriptions meets both thresholds.

## Cleanup

- Nothing to remove; Ollama stays the owner's.

## Out of scope for this test

- Response time — no target (`MOD-10-NFR-03`).
- Languages other than Polish and English — out of scope.
