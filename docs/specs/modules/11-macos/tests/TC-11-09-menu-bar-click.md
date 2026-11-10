---
id: TC-11-09 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-05#1, US-11-05#2, US-11-05#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-11-07] # test case IDs that must pass first
---

# [TC-11-09] On a Mac, a click on the menu bar icon opens or raises the window, and a right click shows the menu

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-05#1`, `US-11-05#2`, `US-11-05#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | `TC-11-07` |

## Preconditions

- The installed app on a Mac, opened from `~/Applications`.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Close the window, then click the menu bar icon. | The window opens in front; the app is back in the Dock; no menu shows. | screenshot |
| 2 | Bring another app to the front over the window, then click the icon. | The window comes to the front; it stays shown. | screenshot |
| 3 | Click the icon again while the window is in front. | The window stays shown and in front. | screenshot |
| 4 | Right-click the icon. | The menu shows "Open Calendar Buddy" and "Quit". | screenshot |
| 5 | Close the window, Control-click the icon and choose "Open Calendar Buddy". | The menu shows; the window opens in front. | screenshot |
| 6 | Right-click the icon and choose "Quit". | The icon is gone and `pgrep -fl calendar-buddy` lists nothing. | terminal output |

## Pass criterion

- A click always takes the user to the window; the menu comes only on a right or Control click.

## Cleanup

- Start the app again if it is used day to day.
