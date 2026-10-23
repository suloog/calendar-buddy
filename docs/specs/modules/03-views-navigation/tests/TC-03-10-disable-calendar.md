---
id: TC-03-10 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-07#1, US-03-07#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-10] A disabled calendar is absent from the side panel, the calendar pickers and the views, and returns when enabled

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-07#1`, `US-03-07#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- One account with calendars "Work" and "Birthdays", each with an event this week.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Disable "Birthdays" in the settings. | "Birthdays" is gone from the side panel, from the calendar picker of a new event, and its event is gone from the week view. | app state |
| 2 | Enable "Birthdays" again in the settings. | "Birthdays" is back in the side panel and the picker. | app state |

## Pass criterion

- Disabling removes a calendar from every list and view, and enabling brings it back.

## Cleanup

- The fixture store and settings are discarded with the test.

## Out of scope for this test

- Synchronization stopping for a disabled calendar — `US-02-04`.
