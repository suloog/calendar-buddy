---
id: TC-04-06 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-04
covers: [US-04-02#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-04-06] Events that are over are faded, their text still at 4.5:1 in both themes; an event under way is not

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-04-02#5` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- It is Wednesday 11 November 2026, 10:00 in Europe/Warsaw, in the week of 9–15 November.

## Test data

| What | Value / source |
| --- | --- |
| Timed events on the 11th | 8:00–9:00 "Earlier", 9:00–10:00 "Ending now", 9:30–10:30 "Under way", 11:00–12:00 "Later" |
| Timed event on Monday 9 | 13:00–14:00 "Monday" |
| All-day events | "Yesterday" (the 10th), "Today" (the 11th), "Since yesterday" (the 10th and 11th) |
| Contrast check | very light #FFF59D and very dark #1A237E events, an invitation awaiting an answer and a declined event, all over |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the week view. | "Earlier", "Ending now", "Monday" and "Yesterday" are faded; "Under way", "Later", "Today" and "Since yesterday" are not. | app state |
| 2 | Open the month view. | The events of the days before and "Yesterday" are faded; today's "Today" and the later event are not. | app state |
| 3 | In the light and the dark theme, read the text and background colours of the events that are over, in the week and the month views. | Every contrast ratio is at least 4.5:1. | app state |

## Pass criterion

- Exactly the events that are over are faded, and their text reaches at least 4.5:1 in both themes.

## Cleanup

- The test settings are discarded with the test.
