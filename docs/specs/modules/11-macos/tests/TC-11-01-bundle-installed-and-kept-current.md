---
id: TC-11-01 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-01#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-11-01] The bundle is written with the program, its property list and its icon, and only what changed is written again

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-01#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fresh temporary folder stands for `~/Applications`; the test program stands for the installed one.
- Automated by: Unit tests `the_bundle_is_installed_with_the_program_and_its_icon_and_kept_up_to_date` and `the_icon_is_one_png_image_with_the_lengths_apple_reads`, macOS only.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Install the bundle. | `Calendar Buddy (dev).app/Contents` holds `Info.plist` with the bundle ID and the program's name, `MacOS/calendar-buddy-dev` equal to the running program and executable, and `Resources/AppIcon.icns`. | file contents |
| 2 | Change `Info.plist` by hand and install again. | `Info.plist` is written back; the icon is not rewritten (its modification time stays). | file contents, modification time |
| 3 | Read the icon file. | It starts with `icns`, its length fields match, and it holds the app's PNG as one `ic08` entry. | file contents |

## Pass criterion

- The bundle is complete after the first start and kept current without rewriting unchanged files.

## Cleanup

- The temporary folder is removed by the test.
