---
id: TC-01-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-01
covers: [US-01-01#1, US-01-01#2, US-01-01#3] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-01-01] A first Google account is connected from the welcome screen and its calendar appears

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-01-01#1`, `US-01-01#2`, `US-01-01#3` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The app has no connected account.
- The network is up and a secret store is running.

## Test data

| What | Value / source |
| --- | --- |
| Account | a Google test account of the product owner (personal or Workspace) |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Start the app. | The welcome screen shows one action, "Connect Google account". | screenshot |
| 2 | Choose "Connect Google account". | Before the browser opens, the app explains that Google will show an "unverified app" screen and how to continue. | screenshot |
| 3 | Continue, consent in the browser with the test account, and return to the app. | The app shows the connected account and the calendar instead of the welcome screen; no server setting or password was typed into the app. | screenshot |

## Pass criterion

- The test account is connected after consenting in the browser, without typing anything into the app.

## Cleanup

- Remove the account in the settings.

## Out of scope for this test

- Failure paths of the first run (no network, no secret store, partial consent) — not in smoke depth.
