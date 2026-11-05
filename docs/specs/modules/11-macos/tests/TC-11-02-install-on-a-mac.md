---
id: TC-11-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-11
covers: [US-11-01#1, US-11-01#8] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-11-02] On a Mac, the installed app is in ~/Applications, opens from Spotlight and Launchpad and runs once

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-11-01#1`, `US-11-01#8` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- A Mac with Rust installed and the repository checked out; no "Calendar Buddy.app" in `~/Applications`.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Run `cargo install --path crates/app --locked`, then `~/.cargo/bin/calendar-buddy`; quit it. | `~/Applications/Calendar Buddy.app` exists with the app's icon in Finder. | screenshot |
| 2 | Open "Calendar Buddy" from Spotlight, then from Launchpad. | The app opens; its icon is in the Dock. | screenshot |
| 3 | With the app running, open it again from Spotlight. | The running window comes forward; `pgrep -fl calendar-buddy` lists one process. | terminal output |
| 4 | Run `cargo run -p calendar-buddy` once. | `~/Applications/Calendar Buddy (dev).app` appears beside the installed app, which is unchanged. | screenshot |

## Pass criterion

- The app installs as a Mac app and runs as one copy. (Closing the window: `TC-11-07`.)

## Cleanup

- Leave the installed app; delete "Calendar Buddy (dev).app" if not wanted.
