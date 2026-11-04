---
id: US-11-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-11
covers: [MOD-11-F09] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-11-01] # story IDs that must be done first
---

# As a User on a Mac, the app keeps reminding me from the menu bar when its window is closed, so that reminders don't depend on an open window

|  |  |
| --- | --- |
| **Module** | `[MOD-11]` macOS — `docs/specs/modules/11-macos/REQUIREMENTS.md` |
| **Covers** | `MOD-11-F09` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-11-01` |
| **YouTrack** | — |
| **Open questions** | Whether the app also stays in the Dock while its window is closed, or only in the menu bar |

## Description

As on Linux with the tray (`US-07-05`): closing the window leaves the app running with an icon in the menu bar, where it keeps synchronizing and notifying. The icon's menu opens the window and quits the app; at login it starts in the menu bar without opening the window.

## Acceptance criteria

- [ ] Closing the window leaves an icon in the menu bar, and reminders keep arriving.
- [ ] The icon's menu opens the window and quits the app; after quitting, no reminders arrive.
- [ ] Started at login, the app shows only its menu bar icon, without opening the window.
- [ ] Opening the app from Spotlight or the Dock while it is in the menu bar opens its window.

## Out of scope / what not to worry about

- Notification buttons — `US-11-03`.

## Technical notes

- The menu bar icon needs AppKit's event loop on the main thread while the window is closed; the loop in `main.rs` that waits on `recv_timeout` with the window closed has to pump it, or run eframe's loop with a hidden window.
- `background::starts_shown` already opens no window at login when the tray shows.
- Decisions: `DEC-44`, `DEC-217`.
