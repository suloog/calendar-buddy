---
id: TC-09-16 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-05#1, US-09-05#2, US-09-05#3, US-09-05#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-16] Hiding the local calendars takes "Local" out of sight and showing them brings it back

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-05#1`, `US-09-05#2`, `US-09-05#3`, `US-09-05#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Account A is connected with "Standup" on 12 November 09:00–09:15; the local store holds "My calendar", the default calendar, with "Dentist" on 12 November 10:00–11:00 and a notification 10 minutes before; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the settings' "Local calendars" section. | "Show local calendars" is ticked. | app state |
| 2 | Untick it. | "Local" is gone from the side panel and the calendar picker, "Dentist" from the views; "Standup" stays; the section still lists "My calendar". | app state |
| 3 | Let the clock reach 09:50. | No notification for "Dentist". | notification record |
| 4 | Press `c`. | The quick box starts in account A's primary calendar. | app state |
| 5 | Restart the app. | "Local" is still hidden. | app state |
| 6 | Tick "Show local calendars"; press `c`. | "Local", "My calendar" and "Dentist" are back; the quick box starts in "My calendar". | app state |

## Pass criterion

- The local calendars are hidden and shown again as a whole, with nothing lost (`MOD-09-F22`).

## Cleanup

- The fake Google, the cache, the settings and the local store are discarded with the test.
