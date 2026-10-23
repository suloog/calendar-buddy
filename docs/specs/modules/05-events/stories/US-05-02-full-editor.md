---
id: US-05-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F03, MOD-05-F04, MOD-05-F05, MOD-05-F06, MOD-05-F07, MOD-05-F08, MOD-05-F11, MOD-05-F15, MOD-05-F16, MOD-05-U03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-01] # story IDs that must be done first
---

# As a User, I can set all the details of an event in a full editor, so that the event says everything it needs to

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F03`, `F04`, `F05`, `F06`, `F07`, `F08`, `F11`, `F15`, `F16`, `U03` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-05-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The full event editor, for new and existing events: every field of an event, all-day and multi-day events, and moving the event to another calendar of the same account. Recurrence is `US-05-06`; types, colours and attachments are `US-05-08`…`US-05-10`. Notifying guests about a change is `MOD-06` (`MOD-06-F12`).

## Acceptance criteria

- [ ] "More options" in the quick-entry box opens the full editor with what was already typed.
- [ ] In the editor the user sets the title, description, start, end, time zone and calendar; the calendar list is grouped by account.
- [ ] An event can be made all-day, and can span several days.
- [ ] The user can set the location, whether the event shows them as busy or free, and its visibility (default, public, private).
- [ ] Any field of an event the user may edit can be changed later in the editor, and the change appears in Google Calendar in the browser.
- [ ] Choosing another calendar of the same account moves the event there.

## Edge cases

- The end is set before the start → the event cannot be saved, and the editor says why.
- An event stored in another time zone → edited in its own time zone, shown in the system's (`US-03-09`).

## Out of scope / what not to worry about

- Moving an event to a calendar of another account — not offered (Google moves events only within one account).

## Technical notes

- Updates as patches with the event's ETag (`MOD-05` *Technical notes*).
- Decision: `DEC-60`.
