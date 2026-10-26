---
id: TC-09-12 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-04#4, US-09-04#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-09-11] # test case IDs that must pass first
---

# [TC-09-12] A local event's own notifications are set in the editor, and muted, disabled or switched-off calendars stay silent

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-04#4`, `US-09-04#5` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | `TC-09-11` |

## Preconditions

- The local store holds "Private" with its default of 30 minutes, and "Family"; notifications are on; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Create "Lunch" in "Private" on 12 November 13:00–14:00; in the editor remove the 30-minute notification, add 1 hour and 5 minutes, save. | The editor offers minutes, hours and days, no e-mail, and refuses a sixth notification. | app state |
| 2 | Let the clock reach 12:00, then 12:55. | One notification at each time. | notification record |
| 3 | Choose "Use calendar defaults" for "Lunch"; reopen it. | It shows the calendar's 30 minutes. | app state |
| 4 | Create "Tea" in "Family" on 12 November 16:00, mute "Family"; create "Walk" in "Private" at 17:00 and turn notifications off in the settings; let the clock reach 17:00. | No notification for "Tea" or "Walk". | notification record |

## Pass criterion

- A local event's notifications are edited as a Google event's, without e-mail, and the usual silences hold.

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
