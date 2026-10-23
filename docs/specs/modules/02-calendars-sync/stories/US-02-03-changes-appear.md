---
id: US-02-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-02
covers: [MOD-02-F07, MOD-02-F08, MOD-02-F09, MOD-02-F10, MOD-02-F23, MOD-02-NFR-01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-02-01] # story IDs that must be done first
---

# As a User, I see changes made on my phone or in the browser without doing anything, so that the app is always current

|  |  |
| --- | --- |
| **Module** | `[MOD-02]` Calendars, synchronisation and offline cache — `docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` |
| **Covers** | `MOD-02-F07`, `F08`, `F09`, `F10`, `F23`, `NFR-01` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-02-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Ongoing synchronization after the first one: an adaptive cadence by calendar type, immediate synchronization when the user looks or after the computer wakes, manual refresh, and fetching only what changed. The events are displayed by `MOD-03`'s views; the week view of `US-03-01` is enough to check this story.

## Acceptance criteria

- [ ] With the window open, a change made on the phone to an editable calendar appears within about a minute.
- [ ] A change to a read-only or subscribed calendar appears within about 15 minutes.
- [ ] With the window hidden in the tray, a change to an editable calendar appears within 5 minutes.
- [ ] Activating the app window synchronizes at once; so does opening an event for editing.
- [ ] Right after waking from sleep and when the network returns, the app synchronizes.
- [ ] "Refresh" synchronizes at once.
- [ ] After a calendar's first full synchronization only changes are fetched; when Google rejects the stored state, that calendar is synchronized in full again without the user doing anything.
- [ ] Background synchronization of 5 accounts and 50 enabled calendars causes no rate-limit errors during an hour of normal use.

## Out of scope / what not to worry about

- Push notifications from Google — `DEC-53`.
- Showing the synchronization state and recovering from errors — `US-02-06`.

## Technical notes

- Adaptive polling by access role, one HTTP batch per round, `syncToken` with full resync on 410, NetworkManager and logind signals (`MOD-02` *Technical notes*).
- Decisions: `DEC-53`, `DEC-54`.
