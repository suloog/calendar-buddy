---
id: US-11-04 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-11
covers: [MOD-11-F11, MOD-11-F12] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-11-01, US-11-02] # story IDs that must be done first
---

# As a User on a Mac, the app catches up after sleep, knows when I am offline and opens calendar subscription links, so that it behaves as it does on Linux

|  |  |
| --- | --- |
| **Module** | `[MOD-11]` macOS — `docs/specs/modules/11-macos/REQUIREMENTS.md` |
| **Covers** | `MOD-11-F11`, `F12` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-11-01`, `US-11-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

As on Linux (`US-02-03`, `US-02-05`, `US-07-06`): the app synchronizes right after the Mac wakes and after the network returns, shows that it is offline when the system says so, and a `webcal:` link opened anywhere on the Mac opens Google Calendar's page for adding that subscription.

## Acceptance criteria

- [ ] After the Mac wakes from sleep, the calendars are synchronized within seconds, and reminders that fell during sleep are shown if their event has not started (`MOD-07-F20`).
- [ ] Turning Wi-Fi off shows that the app is offline; turning it on synchronizes at once.
- [ ] A `webcal:` link clicked in a browser or mail opens Google Calendar's "add calendar" page in the default browser.

## Technical notes

- Sleep: `NSWorkspaceDidWakeNotification`; network: `NWPathMonitor`.
- `webcal:`: `CFBundleURLTypes` in the bundle's `Info.plist`, and an Apple Event handler for `kAEGetURL` — on macOS the link does not come as an argument, as `webcal::link` expects on Linux.
- Decisions: `DEC-46`, `DEC-53`, `DEC-217`.
