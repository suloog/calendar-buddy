---
id: TC-01-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-01
covers: [US-01-02#1, US-01-02#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [TC-01-01] # test case IDs that must pass first
---

# [TC-01-02] A second account is connected from the settings and both are listed with their state

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-01-02#1`, `US-01-02#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | `TC-01-01` |

## Preconditions

- Account A is connected.

## Test data

| What | Value / source |
| --- | --- |
| Account A | the product owner's first Google test account |
| Account B | a second Google test account (personal if A is Workspace, or the other way round) |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | In the settings, add account B and consent in the browser. | Account B is connected next to account A. | screenshot |
| 2 | Look at the account list in the settings. | Both accounts are listed by e-mail address, each marked "connected", with actions to add, sign in again and remove. | screenshot |

## Pass criterion

- Two accounts are connected at the same time and both appear in the settings with their state.

## Cleanup

- Remove account B.

## Out of scope for this test

- Five accounts at once (`US-01-02#3`) — not in smoke depth.
