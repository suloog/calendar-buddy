---
id: TC-11-07 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-02#1, US-11-02#2, US-11-02#3, US-11-02#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-11-02] # test case IDs that must pass first
---

# [TC-11-07] On a Mac, closing the window leaves the app in the menu bar, out of the Dock, still reminding, and its menu opens or quits it

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-02#1`, `US-11-02#2`, `US-11-02#3`, `US-11-02#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | `TC-11-02` |

## Preconditions

- The installed app on a Mac, opened from `~/Applications`, with a connected account or a local calendar; notifications allowed for Calendar Buddy.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Close the window. | A monochrome calendar icon is in the menu bar; the app is gone from the Dock and from Cmd+Tab; `pgrep -fl calendar-buddy` lists it. | screenshot, terminal output |
| 2 | With the window closed, let a reminder of an event fall due (an event 2 minutes ahead with a 1-minute notification). | The notification is shown. | screenshot |
| 3 | Choose "Open" in the icon's menu. | The window opens in front, as it was left; the app is back in the Dock. | screenshot |
| 4 | Close the window, then open the app from Spotlight. | The window opens; `pgrep -fl calendar-buddy` lists one process. | terminal output |
| 5 | Turn on "Start at login", log out and in. | Only the menu bar icon appears; no window opens and the app is not in the Dock. | screenshot |
| 6 | Choose "Quit" in the icon's menu. | The icon is gone and `pgrep -fl calendar-buddy` lists nothing; no reminder arrives afterwards. | terminal output |

## Pass criterion

- The app lives in the menu bar while its window is closed, as it does in the tray on Linux (`TC-07-08`).

## Cleanup

- Turn "Start at login" back to what it was; delete the test event.
