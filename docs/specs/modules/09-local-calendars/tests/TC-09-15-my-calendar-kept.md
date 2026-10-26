---
id: TC-09-15 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-02#8] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-15] With an account and no local calendar, "My calendar" is added once

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-02#8` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Account A is connected, as before local calendars existed; the local file holds no calendar; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Start the app. | The side panel lists account A, then "Local" with "My calendar"; the default calendar is still account A's primary. | app state |
| 2 | Quit and start the app again. | Still one "My calendar". | app state |

## Pass criterion

- There is always a local calendar once the app is in use (`MOD-09-F21`).

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
