---
id: TC-11-03 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-01#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-11-03] Start at login is off until turned on, writes a launch agent for the running program, and is removed when turned off

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-01#5` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fresh temporary folder stands for `~/Library/LaunchAgents`.
- Automated by: Unit test `starting_at_login_is_off_until_turned_on_and_off_again_when_turned_off` in `platform/macos.rs`. Starting at a real login is `TC-11-05`.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Ask whether the app starts at login. | It does not. | app state |
| 2 | Turn it on. | The agent `<bundle ID>.plist` exists, labelled with the bundle ID, with `RunAtLoad` and the running program followed by `--in-tray`. | file contents |
| 3 | Turn it off, twice. | The agent is gone and turning it off again is not an error. | app state |

## Pass criterion

- The launch agent follows the setting exactly.

## Cleanup

- The temporary folder is removed by the test.
