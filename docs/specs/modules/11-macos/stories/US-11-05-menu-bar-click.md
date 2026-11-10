---
id: US-11-05 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-11
covers: [MOD-11-U03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-11-02] # story IDs that must be done first
---

# As a User on a Mac, a click on the menu bar icon takes me to the app, and a right click shows its menu, so that I get back to my calendar in one click

|  |  |
| --- | --- |
| **Module** | `[MOD-11]` macOS — `docs/specs/modules/11-macos/REQUIREMENTS.md` |
| **Covers** | `MOD-11-U03` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-11-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Until now any click on the menu bar icon opened its menu (`DEC-218`). Now a click opens the window, as "Open" in the menu does — from the menu bar while it is hidden, or to the front while it is behind other apps — and the menu with "Open" and "Quit" comes on a right click or a Control click (`DEC-220`).

## Acceptance criteria

- [ ] With the window closed, a click on the menu bar icon opens it in front, with the app back in the Dock.
- [ ] With the window shown behind other apps, a click on the icon brings it to the front; it never hides it.
- [ ] A right click, or a click with Control held, shows the menu with "Open Calendar Buddy" and "Quit", which work as before.

## Out of scope / what not to worry about

- The tray on Linux — its click keeps showing and hiding the window (`MOD-07-U02`).

## Technical notes

- `platform/macos/menu_bar.rs`: the status item gets no menu; its button sends its action on left and right mouse-up, and the action reads `NSApplication.currentEvent` to pop the menu up or send `Signal::Open`, which `background::respond` turns into Show or Raise.
- Decisions: `DEC-218`, `DEC-220`.
