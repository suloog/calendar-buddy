---
id: TC-09-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-01#1, US-09-01#2, US-09-01#3, US-09-01#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-01] A first run without an account starts with "My calendar", where an event is created and shown offline

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-01#1`, `US-09-01#2`, `US-09-01#3`, `US-09-01#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories; no network; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).
- A fake Google that records every request it receives.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Start the app. | The welcome screen shows "Connect Google account" as the main action with a line on synchronization, invitations and Google Meet, and "Use without an account" as the secondary one. | app state |
| 2 | Choose "Use without an account". | The week of 9 November is shown; the side panel has only the group "Local", with no address or state, holding "My calendar"; the settings name "My calendar" as the default calendar. | app state |
| 3 | Click Friday 13 November at 10:00, type "Dentist", press Enter. | "Dentist" 10:00–11:00 is shown at once in "My calendar"'s colour, without a "saving" mark. | app state |
| 4 | Look at the day, 3-day, month and agenda views and the mini calendar. | "Dentist" is in each view; 13 November is marked in the mini calendar. | app state |
| 5 | Click 15:00 on 13 November, type "Gym", open the full editor, add the location "Fit Club" and the description "Legs", make it all-day, save; open it. | "Gym" is in the all-day strip of 13 November; its details show "Fit Club" and "Legs". | app state |
| 6 | Read the fake Google's record. | It received no request. | fake Google record |

## Pass criterion

- Without an account and without network, the user gets a local calendar and creates events that show in every view at once.

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
