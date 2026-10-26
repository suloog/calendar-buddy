---
id: TC-09-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-01#6, US-09-01#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-09-01] # test case IDs that must pass first
---

# [TC-09-03] Without an account new events last 60 minutes and "Meet with…" waits for Google; an account connected later goes above "Local"

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-01#6`, `US-09-01#7` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | `TC-09-01` |

## Preconditions

- The app was started without an account and holds "My calendar" with "Dentist" on 13 November 10:00–11:00; network up; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).
- A fake Google with account A, whose primary calendar holds "Standup" on 13 November 09:00–09:15.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Press `c` on Thursday 12 November. | The quick box starts at 10:00 and ends at 11:00. | app state |
| 2 | Look at "Meet with…" in the side panel. | It is unavailable and says it needs a Google account. | app state |
| 3 | In the settings connect account A through the fake Google. | The side panel shows account A's group above "Local"; the calendar picker lists account A's calendars, then "Local" last. | app state |
| 4 | Look at 13 November. | "Standup" and "Dentist" are both shown; "Dentist" is unchanged. | app state |

## Pass criterion

- Without Google the app uses Google's defaults where it cannot ask; connecting Google later just adds its calendars before "Local".

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
