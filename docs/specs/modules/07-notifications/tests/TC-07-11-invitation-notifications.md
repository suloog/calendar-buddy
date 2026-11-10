---
id: TC-07-11 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-08#1, US-07-08#2, US-07-08#4, US-07-08#5, US-07-08#6] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-11] A new invitation is told once with Accept, Maybe and Decline, answered from its button, and not told on a first synchronization or again after a restart

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-08#1`, `US-07-08#2`, `US-07-08#4`, `US-07-08#5`, `US-07-08#6` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API, a fake desktop notification service that records what is shown, a fixed, advanceable clock and a cache file, set up by the test.
- Account A is newly connected with calendar "Work", which holds an invitation "Old" awaiting the answer.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Synchronize for the first time. | No notification is shown. | notifications recorded |
| 2 | Add an invitation "Planning" awaiting the answer to "Work" in the fake Google, and synchronize. | One notification "Invitation: Planning" with its start, and the buttons Accept, Maybe and Decline. | notifications recorded |
| 3 | Synchronize again; then start the engine again on the same cache. | No further notification. | notifications recorded |
| 4 | Stop the engine, add an invitation "Retro" in the fake Google, start the engine again. | One notification "Invitation: Retro". | notifications recorded |
| 5 | Choose Accept on "Retro". | The fake Google got the answer "accepted" for "Retro"; it no longer awaits the answer. | app state |
| 6 | Mute "Work", add an invitation "Muted", synchronize; turn notifications off, add "Off", synchronize. | No notification is shown for either. | notifications recorded |

## Pass criterion

- Each invitation is told once, never on a first look, and answered from its button.

## Cleanup

- The fakes and the local store are discarded with the test.
