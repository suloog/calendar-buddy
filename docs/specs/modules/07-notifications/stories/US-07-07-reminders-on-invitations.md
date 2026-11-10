---
id: US-07-07 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-07
covers: [MOD-07-F21] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-07-04, US-06-05] # story IDs that must be done first
---

# As a User invited to someone else's event, I can change my own reminders of it, so that I am reminded as I like even when I cannot edit the event

|  |  |
| --- | --- |
| **Module** | `[MOD-07]` Reminders and notifications — `docs/specs/modules/07-notifications/REQUIREMENTS.md` |
| **Covers** | `MOD-07-F21` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-07-04`, `US-06-05` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

An invitation whose organizer does not let guests change it offers no Edit (`DEC-110`), so its reminders could not be changed, although in Google they are the guest's own. Its details now show a "Notifications" row with the editor's reminder rows and a Save; saving changes only the user's reminders, in Google, and asks no one anything (`DEC-221`).

## Acceptance criteria

- [ ] The details of an invitation only its organizer may change show its reminders — its own, or the calendar's defaults — in rows as in the editor.
- [ ] The user adds, removes or changes a reminder there, or goes back to the calendar's defaults, and saves: Google holds the new reminders, and the reminders' notifications follow them; nothing else of the event changes, and no question about e-mailing the guests is asked.
- [ ] For an occurrence of a series, saving asks first whether the change applies to this event or to all events.
- [ ] The details of an event of a calendar the account may only read offer no reminders to change; the event stays without Edit and Delete.

## Out of scope / what not to worry about

- Changing other fields of such an event — only its organizer may (`MOD-06-F15`).
- Reminders on read-only calendars — Google refuses any change there (`DEC-22`).

## Technical notes

- Reuses the editor's reminder fields and their checks (`calendar_core::editor`), the editor's reminder rows (`crates/app/src/editor.rs`), and the save of `Patch { reminders }` through `Session::update`, which sends only that field.
- The fake Google takes from a guest a change of the reminders alone, besides the answer.
- Decisions: `DEC-43`, `DEC-110`, `DEC-221`.
