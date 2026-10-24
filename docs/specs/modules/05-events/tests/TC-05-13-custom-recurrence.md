---
id: TC-05-13 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-11#1, US-05-11#2, US-05-11#3, US-05-11#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-13] The repeat menu is Google's and "Custom…" saves every-2-weeks and last-weekday rules

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-11#1`, `#2`, `#3`, `#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the editor for a new event "Sync" on Tuesday 17 November 2026 10:00–10:30 and open the repeat menu. | The menu lists "Does not repeat", "Daily", "Weekly on Tuesday", "Monthly on the third Tuesday", "Annually on November 17", "Every weekday (Monday to Friday)", "Custom…". | app state |
| 2 | Choose "Custom…", unit "week". | The weekdays are shown with Tuesday chosen; unticking Tuesday as the only day is not allowed. | app state |
| 3 | Set every 2 weeks, tick Monday and Wednesday, untick Tuesday, end after 10 occurrences; confirm and save. | The fake Google received `RRULE:FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE;COUNT=10`; reopened, the menu shows "Every 2 weeks on Monday, Wednesday, 10 times". | API request, app state |
| 4 | Open "Custom…" for a new event on Thursday 26 November 2026, unit "month". | The choices are "Monthly on day 26", "Monthly on the fourth Thursday" and "Monthly on the last Thursday"; for days, months and years no weekdays are shown. | app state |
| 5 | Choose "Monthly on the last Thursday", save. | The fake Google received `RRULE:FREQ=MONTHLY;BYDAY=-1TH`; the views show the occurrences on 26 November, 31 December 2026 and 28 January 2027. | API request, app state |

## Pass criterion

- Google's menu and custom dialog produce the expected rules, and the app shows them in words and on the right days.

## Cleanup

- The fake Google and the local store are discarded with the test.
