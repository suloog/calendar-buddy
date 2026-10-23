---
id: US-02-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-02
covers: [MOD-02-F01, MOD-02-F02, MOD-02-F03, MOD-02-F11, MOD-02-F22, MOD-02-NFR-02] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-01-01, US-03-01] # story IDs that must be done first
---

# As a User, I can see the calendars of all my accounts together, in their Google colours, so that I have one calendar instead of several browser tabs

|  |  |
| --- | --- |
| **Module** | `[MOD-02]` Calendars, synchronisation and offline cache — `docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` |
| **Covers** | `MOD-02-F01`, `F02`, `F03`, `F11`, `F22`, `NFR-02` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-01-01`, blocked by `US-03-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The first full synchronization: every calendar the connected accounts can access, their events within the cache window, shown together in Google's colours and at the right local times. Keeping them current afterwards is `US-02-03`; read-only marking is `US-02-02`. The events are displayed by `MOD-03`'s views; the week view of `US-03-01` is enough to check this story.

## Acceptance criteria

- [ ] With two accounts connected, every calendar of both accounts is fetched, including read-only and subscribed calendars.
- [ ] Events of all these calendars are shown together in the calendar, each calendar in its Google colour.
- [ ] The cache holds events from one year back to two years ahead of today.
- [ ] An event stored in another time zone, an event across a daylight-saving change, and a moved and a cancelled occurrence of a series are each shown at the right local time.
- [ ] The first full synchronization of 5 accounts, 50 calendars and about 20,000 events runs in the background; the interface keeps responding throughout.

## Edge cases

- An event crossing the edge of the cache window → it is kept whole.
- A recurring series that started before the window → its occurrences inside the window are available.

## Out of scope / what not to worry about

- Ongoing synchronization and its cadence — `US-02-03`.
- Read-only and free/busy marking — `US-02-02`.
- The views themselves — `MOD-03`.

## Technical notes

- Full sync bounded by the window, series kept as master plus exceptions, expansion with a tested recurrence library, UTC plus IANA zone (`MOD-02` *Technical notes*, constitution).
- Decisions: `DEC-23`, `DEC-14`.
