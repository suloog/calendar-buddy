---
id: TC-02-07 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-02
covers: [US-02-05#1, US-02-05#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-02-07] Without network the cached calendar is shown with an offline indicator and editing is unavailable

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-02-05#1`, `US-02-05#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- The local store holds a synchronized calendar with events this week.
- The network is unavailable to the app.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Start the app. | The cached events are available before any synchronization, and the offline indicator is on. | app state |
| 2 | Try to create, edit and delete an event and to answer an invitation. | Each action is refused with a reason saying the app is offline. | app state |

## Pass criterion

- Offline, the cached calendar is available with the offline indicator and every change is refused with a reason.

## Cleanup

- The fake Google and the local store are discarded with the test.
