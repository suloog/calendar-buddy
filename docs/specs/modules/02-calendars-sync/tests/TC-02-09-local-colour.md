---
id: TC-02-09 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-02
covers: [US-02-07#1, US-02-07#2, US-02-07#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-02-09] A calendar gets a local colour that Google does not receive, and Reset restores the Google colour

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-02-07#1`, `US-02-07#2`, `US-02-07#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API with a controllable clock and an empty local store, set up by the test.
- One account is connected with a calendar whose Google colour is blue, holding one event this week.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Set a local colour of green for the calendar. | The calendar and its event use green in the app. | app state |
| 2 | Check the requests the fake Google received. | No request changed the calendar's colour. | app state |
| 3 | Choose Reset. | The calendar and its event use blue again. | app state |

## Pass criterion

- The local colour applies only in the app and Reset brings back the Google colour.

## Cleanup

- The fake Google and the local store are discarded with the test.
