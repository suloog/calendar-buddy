---
id: TC-07-08 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-05#1, US-07-05#3, US-07-05#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-08] With the window closed to the tray reminders keep arriving, quitting stops them, and "Start at login" starts the app in the tray

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-05#1`, `US-07-05#3`, `US-07-05#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The product owner's account is connected on KDE Plasma; a test event has a reminder in 5 minutes.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Close the app window. | The app icon stays in the tray; the reminder arrives on time. | screenshot |
| 2 | Quit from the tray menu; create another test reminder 3 minutes ahead in the browser. | No notification arrives. | screenshot |
| 3 | Turn on "Start at login" in the settings, log out and log in again. | The app is running in the tray after login. | screenshot |

## Pass criterion

- Reminders work from the tray, stop after quitting, and autostart starts the app at login.

## Cleanup

- Turn "Start at login" off again if it is not wanted; delete the test events.
