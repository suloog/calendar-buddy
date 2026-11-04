---
id: US-11-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-11
covers: [MOD-11-F01, MOD-11-F02, MOD-11-F03, MOD-11-F04, MOD-11-F05, MOD-11-F06, MOD-11-F07, MOD-11-F08, MOD-11-U01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-01-01, US-01-04, US-04-01, US-07-01, US-07-05] # story IDs that must be done first
---

# As a User on a Mac, I can install the app and use it like my other Mac apps, so that I don't need Linux to keep my calendar at hand

|  |  |
| --- | --- |
| **Module** | `[MOD-11]` macOS — `docs/specs/modules/11-macos/REQUIREMENTS.md` |
| **Covers** | `MOD-11-F01`…`F08`, `U01` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-01-01`, `US-01-04`, `US-04-01`, `US-07-01`, `US-07-05` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The same `cargo install` as on Linux, and one start of the installed program, put the app in `~/Applications` as a Mac app with its icon. From then on it opens from Spotlight, Launchpad or the Dock. Sign-ins are kept in the Keychain, "Start at login" works, reminders show macOS notifications and the app follows the system's light or dark appearance. There is no menu bar icon yet, so closing the window quits the app.

## Acceptance criteria

- [ ] After `cargo install --path crates/app --locked` and one start of `~/.cargo/bin/calendar-buddy`, `~/Applications` holds "Calendar Buddy.app" with the app's icon; it opens from Spotlight and Launchpad, and a debug build makes "Calendar Buddy (dev).app" beside it.
- [ ] After a new version is installed and started once from `~/.cargo/bin`, the bundle holds the new program; its files that did not change are not rewritten, and a copy running from the bundle keeps running.
- [ ] A Google account connected on the Mac keeps its token in the login Keychain under the service `calendar-buddy`; after the app is restarted the account is still connected without signing in again, and removing the account deletes the item.
- [ ] Declining to let the app read its Keychain item shows that unlocking was declined, and the account is not lost.
- [ ] Turning on "Start at login" adds `~/Library/LaunchAgents/pl.ordergroup.CalendarBuddy.plist`; at the next login the app starts with its window open; turning it off removes the file.
- [ ] A reminder shows a macOS notification with the event's title, start time and location.
- [ ] With the theme following the system, switching macOS between light and dark switches the app within a few seconds.
- [ ] Closing the window quits the app; opening the app while it runs brings its window forward and starts no second copy.

## Out of scope / what not to worry about

- The menu bar icon, notification buttons and clicks, sleep and network signals, `webcal:` links — `US-11-02`…`US-11-04`.
- A signed, notarized or packaged app — `DEC-217`.

## Technical notes

- `crates/app/src/platform/` splits by target (`linux.rs`, `macos.rs`); `main.rs` uses target-neutral names (`SystemAutostart`, `SystemLauncher`, `SystemLinkHandler`).
- The gates (`fmt`, `clippy`, `cargo test --workspace`) run on macOS; the Linux code only moved, and the gates still run on Linux before the PR is merged.
- Decisions: `DEC-47`, `DEC-164`, `DEC-217`.
