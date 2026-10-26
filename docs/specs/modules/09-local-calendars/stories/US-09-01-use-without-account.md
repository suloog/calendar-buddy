---
id: US-09-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-09
covers: [MOD-09-F01, MOD-09-F07, MOD-09-F08, MOD-09-F11, MOD-09-F17, MOD-09-F18, MOD-09-U02, MOD-09-U04, MOD-09-NFR-01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-01-01, US-03-06, US-05-01, US-05-02] # story IDs that must be done first
---

# As a User, I can start using the app without a Google account, with a calendar kept on my computer, so that I can plan my time before or without connecting Google

|  |  |
| --- | --- |
| **Module** | `[MOD-09]` Local calendars — `docs/specs/modules/09-local-calendars/REQUIREMENTS.md` |
| **Covers** | `MOD-09-F01`, `F07`, `F08`, `F11`, `F17`, `F18`, `U02`, `U04`, `NFR-01` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-01-01`, `US-03-06`, `US-05-01`, `US-05-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The foundation of local calendars: their own store on the device, a "Local" group next to the accounts, local events created through the quick box and the full editor and shown in every view, and a welcome screen that invites to connect Google but lets the user start without it. Managing several local calendars (`US-09-02`), editing local events in full (`US-09-03`) and their notifications (`US-09-04`) build on this story.

## Acceptance criteria

- [ ] A first run with neither an account nor a local calendar shows the welcome screen: "Connect Google account" as the main action with a line on what Google brings (synchronization, invitations, Google Meet), and "Use without an account" as the secondary one.
- [ ] "Use without an account" opens the week view; the side panel shows only the "Local" group, without an address or account state, holding "My calendar", which is the default calendar.
- [ ] Clicking a time, typing a title and pressing Enter shows the event at once in "My calendar"'s colour, without network and without a "saving" mark; it is also in the day, 3-day, month and agenda views and marks its day in the mini calendar.
- [ ] Opening the event shows its details; the full editor opened from the quick box saves the title, description, start and end, all-day and location of a new local event.
- [ ] After the app is restarted the welcome screen is not shown and the event is there; after deleting the cache file and restarting, it is still there.
- [ ] With no account connected, a new event without a range lasts 60 minutes, and "Meet with…" is unavailable and says it needs a Google account.
- [ ] A Google account connected later from the settings is listed above "Local" in the side panel and the calendar picker; local events stay as they were.
- [ ] When the local file cannot be read at start, the app says so, leaves the file as it is and keeps working with Google calendars.

## Out of scope / what not to worry about

- Adding, renaming, recolouring and deleting local calendars — `US-09-02`.
- Dragging, deleting, undo, repeating and colours of local events — `US-09-03`.
- Notifications of local events — `US-09-04`.

## Technical notes

- `$XDG_DATA_HOME/<app ID>/local.sqlite3` with a schema version, owned by the cache worker and never deleted by it (`MOD-09` *Technical notes*, `DEC-87`, `DEC-164`).
- "Local" is a pseudo-account with a fixed ID in `calendar-core`, so the side panel, the pickers and colours keyed by account and calendar (`DEC-99`) work unchanged; it is not in the accounts file and not checked with Google.
- Writes of local events go to the local store behind the calendar-provider interface (`NFR-14`), not to the Google adapter, and need no network.
- Decisions: `DEC-178`, `DEC-179`.
