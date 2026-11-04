---
id: TC-11-04 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-01#3, US-11-01#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-11-04] On a Mac, a connected Google account is kept in the Keychain across restarts, declining access says so, and removing the account deletes the item

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-01#3`, `US-11-01#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The installed app on a Mac (`TC-11-02`); the Google OAuth client file in `~/.config/calendar-buddy/`; a Google account not yet connected.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Connect the Google account; finish signing in in the browser. | The account's calendars appear; Keychain Access lists an item of the service `calendar-buddy` named by the account. | screenshot |
| 2 | Quit the app and open it again. | The account is still connected; no browser opens. | observation |
| 3 | In Keychain Access, remove the app from the item's access list; open the app and choose **Deny** when asked. | The app says that unlocking was declined; the account stays listed. | screenshot |
| 4 | Open the app again and choose **Always Allow**; then remove the account in the settings. | The account is gone, and Keychain Access no longer lists its item. | screenshot |

## Pass criterion

- The sign-in lives only in the Keychain and goes with the account.

## Cleanup

- Connect the account again if it is wanted.
