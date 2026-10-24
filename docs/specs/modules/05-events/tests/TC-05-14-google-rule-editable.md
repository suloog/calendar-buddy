---
id: TC-05-14 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-11#5, US-05-11#6, US-05-11#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [TC-05-13] # test case IDs that must pass first
---

# [TC-05-14] A rule from Google the dialog can express is edited in the app; one it cannot is kept

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-11#5`, `#6`, `#7` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | `TC-05-13` |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal".
- Series "Quarterly" from Monday 9 November 2026 10:00–11:00 with `RRULE:FREQ=MONTHLY;INTERVAL=3;BYDAY=2MO;UNTIL=20270630T235959Z`.
- Series "Odd" from Monday 2 November 2026 9:00–9:15 with `RRULE:FREQ=MONTHLY;BYDAY=MO,TU;BYSETPOS=1`.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the occurrence of "Quarterly" on 8 February 2027 in the editor. | The repeat menu shows "Every 3 months on the second Monday, until June 30, 2027"; "Custom…" opens filled with every 3 months, the second Monday, ends on 30 June 2027. | app state |
| 2 | Change the interval to every 2 months and save. | The app asks "This and following events" or "All events", without "This event". | app state |
| 3 | Answer "All events". | The fake Google received the master's rule `FREQ=MONTHLY;INTERVAL=2;BYDAY=2MO;UNTIL=…`; no series was split. | API request |
| 4 | Open "Odd", change its title to "Odd 2" and save with "All events". | The fake Google received the new title and the unchanged `BYSETPOS` rule; the repeat menu offers to open the event in Google Calendar to change the rule. | API request, app state |

## Pass criterion

- Expressible rules from Google are editable with the two rule scopes; inexpressible rules survive other edits untouched.

## Cleanup

- The fake Google and the local store are discarded with the test.
