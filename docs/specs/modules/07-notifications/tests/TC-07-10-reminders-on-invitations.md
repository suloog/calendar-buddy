---
id: TC-07-10 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-07#1, US-07-07#2, US-07-07#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-10] On an invitation only its organizer may change, the user changes their own reminders, and nothing else; a read-only calendar offers none

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-07#1`, `US-07-07#2`, `US-07-07#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock, set up by the test.
- Account A is connected with its own calendar "Work" and a read-only calendar "Holidays".
- "Work" holds "Review", organized by someone else, without `guestsCanModify`, on the calendar's defaults (notification 10 minutes before); "Holidays" holds "Day off".

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the reminders of "Review". | They are offered: the calendar's defaults. | app state |
| 2 | Add a notification 30 minutes before and save. | The fake Google got a change of the reminders alone and holds "Review" with its own reminder 30 minutes before; no question about the guests is asked. | app state |
| 3 | Look at the reminders the app schedules. | "Review" is reminded 30 minutes before. | app state |
| 4 | Try to change the title of "Review" through Google. | Google refuses it, as only its organizer may. | app state |
| 5 | Open the reminders of "Day off". | None are offered. | app state |

## Pass criterion

- A guest's own reminders reach Google; the rest of the event and read-only calendars stay unchangeable.

## Cleanup

- The fakes and the local store are discarded with the test.
