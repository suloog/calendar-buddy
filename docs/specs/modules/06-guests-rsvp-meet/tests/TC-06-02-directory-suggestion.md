---
id: TC-06-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-01#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-02] On a Workspace account, a colleague from the directory is suggested

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-01#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account B (Workspace) is connected; its organization directory includes "Anna Nowak", who is not in its contacts.
- A new event in account B is open in the editor.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Type "Anna" in the guest field. | "Anna Nowak" is suggested, from the directory. | app state |

## Pass criterion

- Directory colleagues are suggested on a Workspace account.

## Cleanup

- The fake Google and the local store are discarded with the test.
