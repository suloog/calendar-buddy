---
id: US-05-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F01, MOD-05-F02, MOD-05-F09, MOD-05-F10, MOD-05-F16, MOD-05-U03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-03-01, US-02-01] # story IDs that must be done first
---

# As a User, I can create an event by clicking or selecting a time and typing a title, so that adding a meeting takes seconds

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F01`, `F02`, `F09`, `F10`, `F16`, `U03` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-03-01`, blocked by `US-02-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The fastest way to add an event: click a time or select a range in the calendar, type a title in the small quick-entry box, press Enter. The event goes to the default calendar and is saved to Google. The full editor behind "More options" is `US-05-02`.

## Acceptance criteria

- [ ] Selecting 10:00–11:30 in the week view opens a quick-entry box with that time already filled in and the default calendar chosen.
- [ ] Typing a title and pressing Enter creates the event, and it appears in Google Calendar in the browser.
- [ ] A single click on a time, or the `c` key, starts an event of the default length set in the user's Google Calendar settings.
- [ ] The default calendar is chosen in the settings; until the user chooses one, it is the primary calendar of the first connected account.
- [ ] A new event is marked "saving" until Google confirms it.

## Edge cases

- No network → creating is unavailable, and the app says why (`US-02-05`).

## Out of scope / what not to worry about

- The full editor — `US-05-02`.
- Guests and Google Meet — `MOD-06`.
- Reminders — `MOD-07`.

## Technical notes

- Every change goes to Google first and reaches the local store through synchronization; "saving" is view state only (`MOD-05` *Technical notes*).
- Default length from the Calendar setting `defaultEventLength`.
- Decisions: `DEC-59`, `DEC-60`.
