---
id: US-04-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-04
covers: [MOD-04-F01, MOD-04-F02, MOD-04-F03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [] # story IDs that must be done first
---

# As a User, I can use the app in a light or dark theme that follows my system, so that it fits my desktop day and night

|  |  |
| --- | --- |
| **Module** | `[MOD-04]` Appearance and themes — `docs/specs/modules/04-appearance-themes/REQUIREMENTS.md` |
| **Covers** | `MOD-04-F01`, `F02`, `F03` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | — |
| **YouTrack** | — |
| **Open questions** | — |

## Description

A light and a dark theme, following the desktop's light or dark setting by default and switching live, with a System / Light / Dark choice in the settings. The look is checked on `MOD-03`'s views, starting with the week view of `US-03-01`.

## Acceptance criteria

### Scenario: following the system

- **Given** the desktop is set to dark and the app is set to System
- **When** the app starts
- **Then** it is dark
- **And** when the desktop switches to light, the app switches at once

- [ ] The settings offer System, Light and Dark.
- [ ] Choosing Light or Dark overrides the desktop setting and is kept after a restart.

## Edge cases

- The desktop reports no light or dark preference → the light theme is used.

## Out of scope / what not to worry about

- The palette, density and other parts of the look — `US-04-02`, `US-04-03`.
- Advanced theming — `MOD-08`.

## Technical notes

- Preference read from the XDG desktop portal `color-scheme` and its change signal, behind the platform interface (`MOD-04` *Technical notes*).
- Decisions: `DEC-57`, `DEC-58`.
