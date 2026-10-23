---
id: US-02-05 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-02
covers: [MOD-02-F12, MOD-02-F13, MOD-02-F14, MOD-02-U02, MOD-02-F15] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-02-01] # story IDs that must be done first
---

# As a User, I can open the app and browse my calendars without network, so that I can check my schedule anywhere

|  |  |
| --- | --- |
| **Module** | `[MOD-02]` Calendars, synchronisation and offline cache — `docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` |
| **Covers** | `MOD-02-F12`, `F13`, `F14`, `U02`, `F15` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-02-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Working from the cache: the app opens with cached data at once, everything cached can be browsed without network with a visible offline indicator, and actions that need Google are unavailable with an explanation. Periods outside the cache window are only available online.

## Acceptance criteria

- [ ] On start, the cached calendars are shown at once, before synchronization finishes.

### Scenario: no network

- **Given** the network is cut
- **Then** all cached events can be browsed
- **And** an offline indicator is visible in the main window
- **And** creating, editing, deleting and responding to invitations are unavailable, with an explanation

- [ ] Online, a period outside the cache window is fetched when browsed and is not kept for offline use.
- [ ] Offline, a period outside the cache window is shown empty with a note that it is not available offline.

## Out of scope / what not to worry about

- Editing offline with a change queue — `DEC-25`, `MOD-08`.

## Technical notes

- Decisions: `DEC-19`, `DEC-23`, `DEC-25`, `DEC-54`.
