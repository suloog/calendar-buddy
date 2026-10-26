---
id: TC-09-13 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-04#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-13] On KDE, a local event's notification snoozes, dismisses and opens its event

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-04#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The installed app on Ubuntu 26.10 with KDE Plasma; a local calendar with an event starting in 6 minutes and a 5-minute notification.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Wait for the notification; choose Snooze 5 minutes. | It goes away and comes back 5 minutes later. | screenshot |
| 2 | Choose Dismiss. | It goes away and does not come back. | observation |
| 3 | Create another such event; click its notification. | The app's window comes forward with the event open. | screenshot |

## Pass criterion

- A local event's notification acts on the desktop like a Google event's (`MOD-07`).

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
