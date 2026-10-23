---
id: TC-05-09 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-08#2, US-05-08#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-09] Out-of-office can be created on a Workspace primary calendar and is not offered on a personal account

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-08#2`, `US-05-08#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.
- Account B (Workspace) is also connected; its primary calendar is "B".

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Start a new event in calendar "B" and look at the event types offered. | Out of office and Focus time are offered. | app state |
| 2 | Create an out-of-office event on 13 November, all day. | The fake Google received an out-of-office event in "B". | app state |
| 3 | Start a new event in "Personal" (account A). | Out of office and Focus time are not offered. | app state |

## Pass criterion

- Special types are offered and created only where the account allows them.

## Cleanup

- The fake Google and the local store are discarded with the test.
