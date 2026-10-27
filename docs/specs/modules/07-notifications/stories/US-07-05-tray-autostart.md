---
id: US-07-05 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-07
covers: [MOD-07-F14, MOD-07-F15, MOD-07-F16, MOD-07-F17, MOD-07-U02] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-07-01] # story IDs that must be done first
---

# As a User, the app keeps reminding me from the tray when its window is closed and can start with my session, so that reminders don't depend on an open window

|  |  |
| --- | --- |
| **Module** | `[MOD-07]` Reminders and notifications — `docs/specs/modules/07-notifications/REQUIREMENTS.md` |
| **Covers** | `MOD-07-F14`, `F15`, `F16`, `F17`, `U02` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-07-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Running in the background: closing the window hides the app to the system tray, where it keeps synchronizing and reminding; the tray menu reopens or quits it; it can start at login; and only one copy runs at a time.

## Acceptance criteria

- [ ] Closing the window hides the app to the system tray, and reminders keep arriving.
- [ ] Clicking the tray icon shows or hides the window.
- [ ] The tray menu opens the window and quits the app completely; after quitting, no reminders arrive.
- [ ] "Start at login" in the settings is off by default; turned on, the app starts in the tray after the next login.
- [ ] Launching the app while it runs brings the existing window forward instead of starting a second copy.

## Out of scope / what not to worry about

- A separate background service independent of the app — `DEC-44`.

## Technical notes

- Tray icon, XDG autostart entry, single instance via a D-Bus name, behind the platform interface (`MOD-07` *Technical notes*).
- Decisions: `DEC-44`, `DEC-46`, `DEC-64`.
