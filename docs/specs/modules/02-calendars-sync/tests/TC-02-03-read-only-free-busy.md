---
id: TC-02-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-02
covers: [US-02-02#2, US-02-02#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-02-03] Events of a read-only calendar are marked not editable, and a free/busy-only calendar shows busy blocks without details

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-02-02#2`, `US-02-02#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API with a controllable clock and an empty local store, set up by the test.
- One account is connected with a read-only shared calendar ("Colleague") and a free/busy-only calendar ("Room"), each with one event this week.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Read the Colleague calendar's event. | It is marked as not editable. | app state |
| 2 | Read the Room calendar's event. | It is a "busy" block with no title, location or description. | app state |

## Pass criterion

- The read-only event is marked not editable and the free/busy event carries no details.

## Cleanup

- The fake Google and the local store are discarded with the test.
