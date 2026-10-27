---
id: TC-10-13 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#10] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-13] Days and times in Polish and English descriptions are worked out by the app — weekday, next week, in a week, the day after tomorrow, a date, a part of the day, yesterday

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#10` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- Sunday 25 October 2026 is the day summer time ends in Europe/Warsaw.

## Test data

| What | Value / source |
| --- | --- |
| "Dentysta w środę o 8:15" / "Dentist on Wednesday at 8:15" | weekday Wednesday, 08:15 → Wed 28 Oct 08:15 |
| "przegląd w przyszły czwartek o 15" / "review next Thursday at 3 pm" | next week's Thursday, 15:00 → Thu 5 Nov 15:00 |
| "standup za tydzień o 9" / "standup in a week at 9" | in 7 days, 09:00 → Mon 2 Nov 09:00 |
| "rozmowa pojutrze o 14:30" / "call the day after tomorrow at 2:30 pm" | day after tomorrow, 14:30 → Wed 28 Oct 14:30 |
| "urodziny 14 listopada o 18" / "birthday on 14 November at 6 pm" | date 14 November, 18:00 → Sat 14 Nov 18:00 |
| "kawa jutro rano" / "coffee tomorrow morning" | tomorrow, morning → Tue 27 Oct 09:00 |
| "retro w piątek po południu" / "retro on Friday afternoon" | weekday Friday, afternoon → Fri 30 Oct 14:00 |
| "Serwis auta wczoraj o 12" | yesterday (`DEC-212`), 12:00 → Sun 25 Oct 12:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Send each Polish description of the first seven rows, closing the window after each. | Each window opens on the day and at the time in the row's last column. | app state |
| 2 | Send each English description of the same rows. | The same days and times as for Polish. | app state |
| 3 | Send "Serwis auta wczoraj o 12". | The window opens on Sunday 25 October at 12:00, as adding by hand on a past day allows. | app state |

## Pass criterion

- From the kind of day the model names, the app computes the right date and time in the user's time zone for every row.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- Whether a real model names the right kind of day — `TC-10-19`.
- A description with no day or no time — `TC-10-32`, `TC-10-33`.
