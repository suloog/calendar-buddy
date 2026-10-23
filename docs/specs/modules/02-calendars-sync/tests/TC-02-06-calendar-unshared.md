---
id: TC-02-06 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-02
covers: [US-02-04#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-02-06] A calendar that is no longer shared disappears with its events and a one-time notice

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-02-04#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API with a controllable clock and an empty local store, set up by the test.
- One account is connected with a shared calendar "Colleague" holding two events.
- The first synchronization has finished.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Remove "Colleague" from the account in the fake Google and let the next synchronization run. | The calendar and its two events are gone from the app, and one notice says the calendar is no longer available. | app state |
| 2 | Let one more synchronization run. | No second notice is raised. | app state |

## Pass criterion

- The unshared calendar is removed with its events and the user is told exactly once.

## Cleanup

- The fake Google and the local store are discarded with the test.
