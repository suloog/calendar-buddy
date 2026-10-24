---
id: US-05-11 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F29, MOD-05-F32, MOD-05-F33, MOD-05-F34, MOD-05-F35, MOD-05-U05] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-06, US-05-07] # story IDs that must be done first
---

# As a User, I can set how an event repeats as freely as in Google Calendar's "Custom", so that meetings every two weeks or on the last Friday are set in the app too

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F29`, `F32`, `F33`, `F34`, `F35`, `U05` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-05-06`, `US-05-07` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The editor's repeat menu becomes Google Calendar's, with "Custom…" opening Google's custom recurrence dialog: an interval, the weekdays, the day or weekday of the month, and how the series ends. Series from Google whose rule the dialog can express become editable in the app instead of only in the browser. Builds on the presets of `US-05-06` and the scopes of `US-05-07`.

## Acceptance criteria

- [ ] For an event starting on Tuesday 17 November 2026, the repeat menu offers "Does not repeat", "Daily", "Weekly on Tuesday", "Monthly on the third Tuesday", "Annually on November 17", "Every weekday (Monday to Friday)" and "Custom…".
- [ ] In "Custom…" the user sets "every 2 weeks on Monday and Wednesday, after 10 occurrences"; the event is saved with that rule, and the menu shows it as "Every 2 weeks on Monday, Wednesday, 10 times".
- [ ] For months the dialog offers "Monthly on day 17" and "Monthly on the third Tuesday"; for an event on Thursday 26 November 2026 it also offers "Monthly on the last Thursday", and that rule falls on the last Thursday of every month.
- [ ] For weeks the dialog shows the weekdays with the start's weekday chosen and does not allow none; for days, months and years it shows no weekdays.
- [ ] A series created in Google as "every 3 months on the second Monday, until 30 June 2027" shows that rule in the menu, opens filled in "Custom…", and its rule can be changed in the app.
- [ ] Changing the rule of an occurrence asks "This and following events" or "All events", without "This event", and the change follows the answer.
- [ ] A series from Google with a rule the dialog cannot express (e.g. "the first Monday or Tuesday of the month") keeps its rule when its title is changed, and the app offers to open it in Google Calendar to change the rule.
- [ ] A series set in "Custom…" shows the same occurrences in Google Calendar in the browser.

## Out of scope / what not to worry about

- Rules beyond Google's dialog (several months, set positions, hourly, several rules) — `DEC-165`.
- Changing occurrences by dragging or deleting them — `US-05-07`.

## Technical notes

- RRULE mapping and the "complex" boundary in `MOD-05` *Technical notes*; parsing a rule into the dialog's fields, writing it back and wording it are pure functions in `calendar-core`, test-first (constitution).
- Changing the rule with "this and following" splits the series as in `US-05-07`; "all" updates the master.
- Decision: `DEC-165` (widens `DEC-36`).
