---
id: TC-03-09 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-06#1, US-03-06#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-09] The side panel groups calendars by account and the quick toggle hides a calendar's events while keeping it listed

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-06#1`, `US-03-06#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- Account A has calendars "Work" and "Team"; account B has "Home"; each has an event this week.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Read the side panel. | It lists account A with "Work" and "Team" under it, and account B with "Home" under it. | app state |
| 2 | Use the quick toggle of "Team". | "Team" stays in the list, and its event is no longer in the week view. | app state |
| 3 | Use the toggle again. | The "Team" event is back. | app state |

## Pass criterion

- Calendars are grouped by account and the quick toggle hides only the events.

## Cleanup

- The fixture store and settings are discarded with the test.
