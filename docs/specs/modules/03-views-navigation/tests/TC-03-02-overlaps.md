---
id: TC-03-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-03
covers: [US-03-01#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-03-02] Overlapping events are placed side by side without covering each other

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-03-01#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Fixture data in the local store and a fixed clock: Wednesday 2026-11-11 10:00, system time zone Europe/Warsaw.
- Wednesday holds three events: 10:00–11:00, 10:30–11:30 and 10:45–12:00.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the week view and read the layout of Wednesday. | The three events are placed in three columns side by side, and no two of their areas overlap. | app state |

## Pass criterion

- Overlapping events never cover each other.

## Cleanup

- The fixture store and settings are discarded with the test.

## Out of scope for this test

- Many overlapping events narrowing to their title start — not in smoke depth.
