---
id: TC-09-11 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-04#1, US-09-04#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-11] A local calendar's default notification of 30 minutes reminds once, and changing the default changes the events that use it

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-04#1`, `US-09-04#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- The local store holds the new calendar "Private"; notifications are on; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Read "Private" in the "Local calendars" section. | Its default notifications are one notification 30 minutes before. | app state |
| 2 | Create "Dentist" on 12 November 10:00–11:00 with the location "Main St 5"; let the clock reach 09:30; let it run to 09:45. | Exactly one notification at 09:30 with "Dentist", 10:00 and "Main St 5". | notification record |
| 3 | Create "Call" on 12 November 12:00–12:30; change "Private"'s default for timed events to 10 minutes; let the clock reach 11:30, then 11:50. | No notification at 11:30; one at 11:50 for "Call". | notification record |
| 4 | Create the all-day "Birthday" on 14 November, keeping the calendar's defaults; let the clock reach 14 November 00:00. | No notification for "Birthday" is shown. | notification record |

## Pass criterion

- Local events remind as Google events do, from defaults the user sets per local calendar.

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
