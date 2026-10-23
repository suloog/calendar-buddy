---
id: TC-01-04 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-01
covers: [US-01-04#2, US-01-04#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-01-04] Removing an account deletes all its local data and withdraws the app's access

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-01-04#2`, `US-01-04#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- One account is connected, with cached calendars, events, a local calendar colour and a default calendar set.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Remove the account and confirm. | The local store holds no calendars, events or settings of that account, and the secret store holds no credential for it. | app state |
| 2 | Check what the fake Google received. | The fake Google received a request withdrawing that account's access. | app state |

## Pass criterion

- Nothing of the removed account is left on the device, and its access was withdrawn at Google.

## Cleanup

- The test store, secret store and fake Google are discarded with the test.

## Out of scope for this test

- The confirmation dialog, offline removal and the welcome screen after the last account — not in smoke depth.
