---
id: TC-03-04 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-01#5, US-03-01#6] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: nfr # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-04] The interface-technology feasibility check and the first performance observation are recorded

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-01#5`, `US-03-01#6` |
| **Role** | User |
| **Type** | nfr |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The week view is built on test data for 5 accounts, 50 calendars and about 20,000 events.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Run the feasibility check: smooth scrolling, overlapping events, resizing at 100/150/200 %, dragging an event. | A verdict for the chosen technology is recorded as a decision, with the reasons. | screenshot |
| 2 | Start the app, scroll the week and switch weeks on the test data, watching for stutter. | The product owner records that start-up, scrolling and switching show no visible stutter; this observation is the NFR-03 measurement (`DEC-66`). | screenshot |

## Pass criterion

- The feasibility verdict and the product owner's performance observation exist as recorded decisions (`DEC-66`).

## Cleanup

- The test data stays as the performance profile for later measurements.
