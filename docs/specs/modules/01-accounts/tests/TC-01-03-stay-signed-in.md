---
id: TC-01-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-01
covers: [US-01-03#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-01-03] Connected accounts stay connected after a restart without opening the browser

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-01-03#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Two accounts are connected, with their credentials in the test secret store and the fake Google accepting them.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Restart the app. | Both accounts are in the state "connected", and no browser sign-in was started. | app state |

## Pass criterion

- After a restart both accounts are connected without any sign-in.

## Cleanup

- The test secret store and the fake Google are discarded with the test.

## Out of scope for this test

- Revoked access and signing in again (`US-01-03#2`, `#3`) — not in smoke depth.
