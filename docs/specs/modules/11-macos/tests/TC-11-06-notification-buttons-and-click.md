---
id: TC-11-06 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-03#2, US-11-03#3, US-11-03#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-11-06] Each notification gets the buttons of its kind, and what the notification center answers maps back to the click or the button

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-03#2`, `US-11-03#3`, `US-11-03#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- No notification center is used: the tests read the identifiers the app hands it and the ones it answers with.
- Automated by: Unit tests `a_notification_with_google_meet_has_its_own_buttons` and `a_click_opens_the_notification_and_a_button_does_its_action` (macOS only), and `each_action_of_a_notification_comes_back_from_its_key` (both systems).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Name the category of a notification without and with Google Meet. | `snooze-5+snooze-10+dismiss` and `join+snooze-5+snooze-10+dismiss`: two categories, Join only with Meet. | test output |
| 2 | Map the notification center's click identifier back. | Open — the event is opened. | test output |
| 3 | Map each button's identifier back. | Join, Snooze 5, Snooze 10 and Dismiss, each its own action; an identifier the app did not give maps to nothing. | test output |

## Pass criterion

- Every button and the click reach the action the reminders already handle (`MOD-07-F08`…`F10`).

## Cleanup

- None.
