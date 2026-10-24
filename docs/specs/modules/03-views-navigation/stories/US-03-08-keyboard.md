---
id: US-03-08 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-03
covers: [MOD-03-F23, MOD-03-F24, MOD-03-U04, MOD-03-U07] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-05] # story IDs that must be done first
---

# As a User, I can do everything from the keyboard with Google Calendar's shortcuts, so that I keep my muscle memory

|  |  |
| --- | --- |
| **Module** | `[MOD-03]` Views and navigation — `docs/specs/modules/03-views-navigation/REQUIREMENTS.md` |
| **Covers** | `MOD-03-F23`, `F24`, `U04`, `U07` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-03-05` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Google Calendar's keyboard shortcuts for views, navigation and creating events, a list of them on "?", and moving around the app without a mouse.

## Acceptance criteria

- [ ] `d`, `x` (3 days), `w`, `m` and `a` switch to the day, 3-day, week, month and agenda views.
- [ ] `j` or `n` moves to the next period, `k` or `p` to the previous one, `t` goes to today, and `g` asks for a date to go to.
- [ ] `c` starts a new event.
- [ ] `?` shows the list of shortcuts.
- [ ] Without a mouse, the user moves between the side panel, the calendar and its events, and opens an event.
- [ ] A help button ("?" in a circle) in the top bar, beside the settings, opens the list of shortcuts as `?` does; the list shows them under Views, Navigation and Actions, each key drawn as a key, `j`/`n` and `k`/`p` on one row each.

## Out of scope / what not to worry about

- Search (`/`) — not built (`DEC-30`).
- The event editor that `c` opens — `MOD-05`.

## Technical notes

- Key map in `MOD-03` *Technical notes*.
- Decision: `DEC-55`; `NFR-09`.
