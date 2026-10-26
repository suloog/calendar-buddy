---
id: TC-09-17 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-05#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-09-16] # test case IDs that must pass first
---

# [TC-09-17] Without a Google account the local calendars cannot be hidden

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-05#5` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | `TC-09-16` |

## Preconditions

- Account A is connected; the local store holds "My calendar"; the local calendars are hidden; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Remove account A. | "Local" with "My calendar" is shown in the side panel. | app state |
| 2 | Open the settings' "Local calendars" section. | "Show local calendars" is ticked and unavailable, saying local calendars are shown while no Google account is connected. | app state |

## Pass criterion

- An app without accounts always shows its local calendars (`MOD-09-F22`, `MOD-09-U06`).

## Cleanup

- The fake Google, the cache, the settings and the local store are discarded with the test.
