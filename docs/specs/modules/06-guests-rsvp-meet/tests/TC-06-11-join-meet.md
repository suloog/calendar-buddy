---
id: TC-06-11 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-07#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-11] Join in the event details and in the agenda opens the Meet meeting in the browser

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-07#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The product owner's account is connected; a test event this week has a Google Meet link.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the event's details and choose Join. | The Meet meeting opens in the browser. | screenshot |
| 2 | Open the agenda and choose Join on the same event. | The same meeting opens in the browser. | screenshot |

## Pass criterion

- Join opens the Meet meeting from the details and from the agenda.

## Cleanup

- Delete the test event.
