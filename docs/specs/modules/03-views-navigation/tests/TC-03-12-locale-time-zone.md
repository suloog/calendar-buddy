---
id: TC-03-12 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-09#1, US-03-09#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-12] A Sunday-first locale and a change of system time zone are followed by the views

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-09#1`, `US-03-09#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- An event on Thursday 12 November 15:00–16:00 Europe/Warsaw.
- The system locale is set to en-US.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the week view. | The week runs Sunday 8 to Saturday 14 November, with US date and time formats. | app state |
| 2 | Change the system time zone to America/New_York. | The Thursday event is shown at 09:00–10:00. | app state |

## Pass criterion

- The views follow the system locale and the system time zone.

## Cleanup

- Restore the locale and time zone used by the other tests.
