---
id: MOD-11
title: macOS
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | done
depends_on: [MOD-01, MOD-02, MOD-04, MOD-07] # other module IDs
sources: [SRC-12, SRC-01] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-11] macOS

## Context

The app on macOS, as a second platform next to Linux: installing it as a Mac app, keeping sign-ins in the Keychain, starting at login, notifications and the system's appearance. Comes from the product owner's request of 2026-11-04 (`SRC-12`), settled by `DEC-217`, which moves macOS out of the deferred scope (`SRC-01-119`, `MOD-08`). Everything the app does is the same on both platforms; this module names only how macOS provides what the other modules ask of the system, behind the platform interface (`NFR-13`), and what macOS does not have yet. A first stage makes the app installable and usable (`MOD-11-F01`…`F08`); a second brings the menu bar icon, notification buttons, sleep and network signals and `webcal:` links (`MOD-11-F09`…`F12`).

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-119` Production-quality macOS and Windows releases if Linux is prioritized first. | informal | macOS: this module (`DEC-217`); Windows stays deferred (`MOD-08`, `DEC-02`) |
| `SRC-12-01` The product owner wants to install the app so that it works in the system. | informal | `MOD-11-F01`, `MOD-11-F03` |
| `SRC-12-02` The product owner's computer is a Mac. | informal | this module |
| `SRC-12-03` Real macOS support: the Keychain, the app in `~/Applications`, starting at login, the menu bar icon, notifications. | informal | `MOD-11-F01`…`MOD-11-F10` |
| `SRC-12-04` The first stage: the bundle, the Keychain, start at login; the menu bar, notification buttons, sleep and network, `webcal:` later. | informal | first stage `MOD-11-F01`…`MOD-11-F08`; second stage `MOD-11-F09`…`MOD-11-F12` |
| `SRC-12-05` The work goes through the spec workflow. | informal | this module (process, no requirement) |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

### Installing

- [MOD-11-F01] Started once from where `cargo install` put it, the app puts itself in `~/Applications` as "Calendar Buddy.app", with its icon and its own copy of the program, so that it is found in Launchpad and Spotlight and can be kept in the Dock; a debug build is "Calendar Buddy (dev).app", beside it (`DEC-164`). — *source: `DEC-217`*
- [MOD-11-F02] Each start of the program from outside the bundle brings the bundle up to date: a file that changed is replaced, one that did not is left as it is, and a copy that runs keeps running. — *source: `DEC-217`*

### Signing in

- [MOD-11-F03] Each account's refresh token is kept in the user's login Keychain, as an item of the app's own service (`calendar-buddy`, or `calendar-buddy-dev` for a debug build) named by the account; after a restart the account stays connected; removing the account deletes its item (`MOD-01-F04`, `MOD-01-F07`, `NFR-04`). — *source: `DEC-47`, `DEC-217`*
- [MOD-11-F04] When the user declines to unlock the Keychain or to let the app read its item, the app says that unlocking was declined, as when the secret store on Linux stays locked. — *source: `DEC-217`*
- [MOD-11-F05] Sign-in, Join and other links open in the system's default browser. — *source: `DEC-217`*

### The system

- [MOD-11-F06] "Start at login" (`MOD-07-F16`) adds a launch agent that starts the program it was turned on from when the user logs in, with its window open; it is off until turned on, and turning it off removes the agent. — *source: `DEC-217`*
- [MOD-11-F07] A reminder (`MOD-07-F01`) shows a macOS notification with the event's title, start time and, if any, location (`MOD-07-U01`), without buttons. — *source: `DEC-217`*
- [MOD-11-F08] The app follows the system's light or dark appearance (`MOD-04-F02`), within a few seconds of a change. — *source: `DEC-217`*

### Second stage

- [MOD-11-F09] Closing the window keeps the app running with an icon in the menu bar, where it keeps synchronizing and notifying; the icon's menu opens the window and quits the app (`MOD-07-F14`, `F15`, `U02`). — *source: `DEC-217`*
- [MOD-11-F10] A notification offers Join, Snooze for 5 or 10 minutes and Dismiss, and clicking it opens its event (`MOD-07-F08`, `F09`, `F10`). — *source: `DEC-217`*
- [MOD-11-F11] The app synchronizes right after the Mac wakes from sleep and after the network returns, and shows that it is offline as the system says (`MOD-02-F08`, `MOD-02-F14`). — *source: `DEC-217`*
- [MOD-11-F12] A `webcal:` link opened anywhere on the Mac opens Google Calendar in the browser, ready to add that subscription (`MOD-07-F18`). — *source: `DEC-217`*

## UI/UX requirements

- [MOD-11-U01] Until the menu bar icon comes (`MOD-11-F09`), closing the window quits the app, and reminders arrive only while it runs. — *source: `DEC-217`*

## Constraints and edge cases

- The home directory is not known → no bundle and no launch agent are written; the app runs as it was started.
- The program runs from inside the bundle → it is its own copy; nothing is copied.
- The bundle's program is replaced while it runs → the running copy keeps running; the new one is used from the next start.
- A new version was installed → macOS may ask once whether the app may read its Keychain item; "Always Allow" keeps it from asking again; declining is `MOD-11-F04`.
- Notifications are shown on behalf of Script Editor → when they are off for Script Editor in *System Settings → Notifications*, none are shown; the system decides, as "do not disturb" does (`MOD-07`).
- "Start at login" turned on from a program outside the bundle → it starts that program; it is turned on in the installed app.
- Opening the app while it runs → macOS brings the running copy forward; no second copy runs (`MOD-07-F17`).
- No network → until `MOD-11-F11` the app counts as online, and a failed synchronization says so as it does.

## Integrations

- **Keychain** (read and write): each account's refresh token, through the Security framework.
- **Launch Services and launchd** (write): the bundle in `~/Applications`, the launch agent in `~/Library/LaunchAgents`.
- **Notification Center** (write): notifications, through AppleScript's `display notification`.

## Technical notes

- The platform code is split by target: `crates/app/src/platform/linux.rs` and `crates/app/src/platform/macos.rs`, each implementing `calendar_core::platform`; the Linux crates (`secret-service`, `zbus`, `ksni`) are Linux-only dependencies, `security-framework` macOS-only.
- The bundle: `Contents/Info.plist` (`CFBundleIdentifier` `pl.ordergroup.CalendarBuddy`, `…Dev` for a debug build), `Contents/MacOS/<app ID>` — a copy of the running program, written to a new file renamed into place — and `Contents/Resources/AppIcon.icns`, the app's 256×256 PNG in an `ic08` entry.
- The launch agent: `~/Library/LaunchAgents/<bundle ID>.plist` with `RunAtLoad`, the program and `--in-tray`; without a tray the window opens (`background::starts_shown`).
- The appearance is read from the global `AppleInterfaceStyle` default every 5 seconds.
- Single instance comes from Launch Services; the app claims nothing itself.
- The config and cache directories stay `~/.config/calendar-buddy` and `~/.cache/calendar-buddy`, shared with Linux's layout (`DEC-77`, `DEC-87`).
- The second stage needs AppKit's event loop on the main thread while the window is closed (the menu bar icon), `UNUserNotificationCenter` (buttons, clicks), `NSWorkspace` and `NWPathMonitor` (sleep, network), and `CFBundleURLTypes` with an Apple Event handler (`webcal:` arrives as an event, not as an argument).

## Out of scope / what not to worry about

- Windows — still deferred (`DEC-02`, `MOD-08`).
- A signed and notarized app, a `.dmg` or the App Store — the app is built on the Mac it runs on (`DEC-217`).
- Moving the config and cache to `~/Library` — `DEC-217`.
- Delivering reminders while the app is completely closed — as on Linux (`MOD-07`).

## Acceptance criteria

- [ ] After `cargo install` and one start, "Calendar Buddy.app" is in `~/Applications` with its icon and opens from Spotlight.
- [ ] A Google account connected on the Mac stays connected after a restart, with its token in the Keychain; removing it deletes the item.
- [ ] "Start at login" starts the app at the next login; a reminder shows a macOS notification; switching the system to dark switches the app.
- [ ] (Second stage) Closing the window leaves the app in the menu bar, reminders keep arriving, and Snooze and Join work from the notification.
