---
id: US-11-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-11
covers: [MOD-11-F09, MOD-11-U02] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-11-01] # story IDs that must be done first
---

# As a User on a Mac, the app keeps reminding me from the menu bar when its window is closed, so that reminders don't depend on an open window

|  |  |
| --- | --- |
| **Module** | `[MOD-11]` macOS — `docs/specs/modules/11-macos/REQUIREMENTS.md` |
| **Covers** | `MOD-11-F09`, `U02` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-11-01` |
| **YouTrack** | — |
| **Open questions** | — (the Dock: `DEC-218`) |

## Description

As on Linux with the tray (`US-07-05`): closing the window hides it and leaves the app running with an icon in the menu bar — the system's monochrome calendar symbol — where it keeps synchronizing and notifying. While the window is hidden the app is out of the Dock. The icon's menu opens the window and quits the app; at login it starts in the menu bar without opening the window.

## Acceptance criteria

- [ ] Closing the window leaves a monochrome calendar icon in the menu bar and takes the app out of the Dock and Cmd+Tab, and reminders keep arriving.
- [ ] The icon's menu opens the window, with the app back in the Dock, and quits the app; after quitting, no reminders arrive and the icon is gone.
- [ ] Started at login, the app shows only its menu bar icon, without opening the window.
- [ ] Opening the app from Spotlight, Launchpad or Finder while it is in the menu bar opens its window; one copy runs.

## Out of scope / what not to worry about

- Notification buttons — `US-11-03`.

## Technical notes

- eframe's loop runs with a hidden window (`platform::WINDOW_HIDES`), so AppKit's event loop keeps running; `CalendarApp::logic` does the background work while the window is hidden. Linux keeps its loop of closing and reopening the window.
- `platform/macos/menu_bar.rs`: the status item, from the main dispatch queue; the Dock through the activation policy (`platform::in_dock`).
- `background::starts_shown` already opens no window at login when the tray shows.
- Decisions: `DEC-44`, `DEC-217`, `DEC-218`.
