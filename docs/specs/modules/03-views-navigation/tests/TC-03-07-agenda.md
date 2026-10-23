---
id: TC-03-07 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-04#1, US-03-04#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-07] The agenda lists upcoming events by day without empty days, and says when nothing is ahead

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-04#1`, `US-03-04#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- Upcoming events: today 14:00 and Friday 13 November 09:00; nothing on Thursday.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Choose the agenda view. | Two day groups are shown — today with the 14:00 event and Friday with the 09:00 event — and no group for Thursday. | app state |
| 2 | Remove all upcoming fixture events and open the agenda again. | The agenda says there are no upcoming events. | app state |

## Pass criterion

- The agenda groups upcoming events by day, skips empty days, and reports when nothing is ahead.

## Cleanup

- The fixture store and settings are discarded with the test.
