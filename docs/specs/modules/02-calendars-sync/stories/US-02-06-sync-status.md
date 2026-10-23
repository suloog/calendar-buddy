---
id: US-02-06 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-02
covers: [MOD-02-F16, MOD-02-F17, MOD-02-F18, MOD-02-U03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-02-03] # story IDs that must be done first
---

# As a User, I can see whether my calendars are up to date and the app recovers from errors on its own, so that I can trust what I see

|  |  |
| --- | --- |
| **Module** | `[MOD-02]` Calendars, synchronisation and offline cache — `docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` |
| **Covers** | `MOD-02-F16`, `F17`, `F18`, `U03` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-02-03` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The synchronization indicator and error handling: the overall state with the time of the last successful synchronization, errors shown on the calendar they affect, and automatic recovery from network loss, Google's rate limits and transient errors.

## Acceptance criteria

- [ ] The indicator shows whether the app is synchronizing, up to date, offline or in error, and the time of the last successful synchronization.
- [ ] The user can keep browsing and working while synchronization runs.
- [ ] An error affecting one calendar is shown on that calendar, and the other calendars keep synchronizing.
- [ ] After a lost connection, a rate limit or a transient error, synchronization resumes on its own with growing intervals and nothing is lost; while rate-limited, the indicator says it is delayed.

## Edge cases

- An account needs sign-in → its cached calendars stay visible and are marked as not synchronizing.
- The cache is damaged or unreadable → the app discards it and rebuilds it from Google.

## Out of scope / what not to worry about

- Account-level problems and signing in again — `MOD-01` (`US-01-03`).

## Technical notes

- Exponential back-off with jitter on 403 rate-limit and 429; per-calendar error state (`MOD-02` *Technical notes*).
- Decision: `DEC-54`.
