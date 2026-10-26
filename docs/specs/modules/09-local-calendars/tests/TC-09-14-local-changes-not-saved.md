---
id: TC-09-14 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-01#9] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: edge # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-14] A change the local file does not take is told in a bar until one is

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-01#9` |
| **Role** | User |
| **Type** | edge |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- The app was started without an account; the local file, in a directory of its own, holds "My calendar"; the user is not one every directory takes files from (not root); a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Make the local file's directory read-only; add the local calendar "Family" in the settings. | "Family" is not kept; a bar filled with the warning tone across the top of the main window says changes to local calendars are not being saved, why, that the last change was lost, and the file to check. | app state |
| 2 | Let a polling round pass. | The bar stays, told once. | app state |
| 3 | Make the directory writable again; add "Sport". | "Sport" is kept and the bar goes. | app state, file system |

## Pass criterion

- Nothing the user does to local calendars is lost without the app saying so (`MOD-09-F20`, `MOD-09-U05`).

## Cleanup

- The local file and its directory are deleted with the test.
