---
id: TC-06-16 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-09#7, US-06-09#8] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-16] Chosen people go away when removed, without network and after a restart

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-09#7`, `US-06-09#8` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy), a fake network state and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account B (Workspace) is connected; in the fake Google, Anna Nowak and Adam Kowalski share their calendars with details and each has an event on 13 November; both are chosen in "Meet with…" and their events are shown in the week view.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Remove Adam from the list. | Adam's events leave the views; Anna's stay. | app state |
| 2 | Take the network away. | Anna's events leave the views, her entry stays in the list; "Meet with…" is unavailable and says it needs a connection. | app state |
| 3 | Bring the network back. | Anna's events are shown again; the field is available. | app state |
| 4 | Restart the app on the same local store. | The "Meet with…" list is empty and only the user's events are shown. | app state |

## Pass criterion

- People's calendars are shown only while they are chosen and the app is online, and the list is not kept across restarts.

## Cleanup

- The fake Google and the local store are discarded with the test.
