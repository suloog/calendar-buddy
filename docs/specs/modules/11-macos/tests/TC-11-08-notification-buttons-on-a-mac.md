---
id: TC-11-08 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-03#1, US-11-03#2, US-11-03#3, US-11-03#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-11-07] # test case IDs that must pass first
---

# [TC-11-08] On a Mac, a notification comes in the app's name with Join, Snooze and Dismiss, and a click opens its event

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-03#1`, `US-11-03#2`, `US-11-03#3`, `US-11-03#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | `TC-11-07` |

## Preconditions

- The installed app on a Mac, opened from `~/Applications` for the first time since this version; a connected Google account; notifications shown as alerts or banners.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the app. | macOS asks once whether Calendar Buddy may send notifications; allow it. | screenshot |
| 2 | Create an event with Google Meet 2 minutes ahead with a 1-minute notification; wait. | The notification shows the app's name and icon, the event's title and time, and under "Options" (or on hover) Join, Snooze 5 minutes, Snooze 10 minutes and Dismiss. | screenshot |
| 3 | Choose Snooze 5 minutes. | It is shown again 5 minutes later. | screenshot |
| 4 | On it, choose Join. | The Meet call opens in the default browser. | screenshot |
| 5 | Make another reminder fall due; choose Dismiss. | It goes and does not come back. | observation |
| 6 | Close the window; make another reminder fall due and click the notification's body. | The window opens on the event's day with the event shown. | screenshot |

## Pass criterion

- Notifications on a Mac do what they do on Linux (`US-07-01`, `US-07-02`).

## Cleanup

- Delete the test events.
