---
id: TC-11-05 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-01#5, US-11-01#6, US-11-01#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-11-05] On a Mac, the app starts at login, shows a reminder as a macOS notification and follows the system's dark mode

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-01#5`, `US-11-01#6`, `US-11-01#7` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The installed app on a Mac with a connected account or a local calendar; the theme set to follow the system; notifications allowed for Calendar Buddy.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Turn on "Start at login" in the settings; log out and in. | `~/Library/LaunchAgents/pl.ordergroup.CalendarBuddy.plist` exists; after logging in the app's icon is in the menu bar, without its window (`US-11-02`). | screenshot |
| 2 | Create an event with a location starting in 2 minutes with a 1-minute notification. | A macOS notification shows its title, start time and location. | screenshot |
| 3 | Switch *System Settings → Appearance* from Light to Dark and back. | The app switches within a few seconds each time. | screenshot |
| 4 | Turn "Start at login" off. | The launch agent file is gone. | terminal output |

## Pass criterion

- Start at login, notifications and the appearance work on macOS as the other modules ask.

## Cleanup

- Delete the test event.
