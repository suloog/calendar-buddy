---
id: US-09-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-09
covers: [MOD-09-F09, MOD-09-F10, MOD-09-F12, MOD-09-F16] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-09-01, US-05-03, US-05-04, US-05-07, US-05-09, US-05-11] # story IDs that must be done first
---

# As a User, I can change, move, repeat and delete local events the same way as my Google events, so that a local calendar is as handy as a Google one

|  |  |
| --- | --- |
| **Module** | `[MOD-09]` Local calendars — `docs/specs/modules/09-local-calendars/REQUIREMENTS.md` |
| **Covers** | `MOD-09-F09`, `F10`, `F12`, `F16` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-09-01`, `US-05-03`, `US-05-04`, `US-05-07`, `US-05-09`, `US-05-11` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Everything the app does to a Google event it does to a local one, saved to the local store at once: the full editor, dragging and resizing, deleting with Undo and Ctrl+Z, the event colour, repeating with Google's repeat menu and custom dialog, and the three edit scopes. What only Google can do — guests, Meet, attachments, special types, busy/free and visibility — is not offered.

## Acceptance criteria

- [ ] Every field of a local event the editor offers (title, description, start and end with time zone, all-day and several days, location, event colour, repeating) can be changed, and the change is shown at once, also without network.
- [ ] A local event can be dragged to another time or day and its end dragged to change its length, in 15-minute steps; Ctrl+Z and the Undo notice undo the last change.
- [ ] Deleting a local event that is not part of a series deletes it without a question and offers Undo.
- [ ] A local event can be made repeating from the repeat menu and the custom dialog; changing or deleting an occurrence asks "this event / this and following / all events", and "this and following" splits the series as for a Google event.
- [ ] The editor of a local event has no guests, Google Meet, attachments, event type, busy/free or visibility, and no e-mail option among its notifications.
- [ ] The calendar picker of an existing local event lists only local calendars, and that of an existing Google event lists no local ones; a new event can be put in any calendar.
- [ ] Creating, changing and deleting local events sends nothing to Google, and local events are not among the user's busy times that others see in "Meet with…" or in the editor's availability preview.

## Out of scope / what not to worry about

- Moving or copying an event between a local and a Google calendar — `DEC-178`.
- Notifications of local events and their editing in the editor — `US-09-04`.

## Technical notes

- Local events use the same event model as Google events (master plus changed or cancelled occurrences, RRULE), so recurrence expansion, layout, dragging and the split of "this and following" are shared; only the write goes to the local store (`MOD-09` *Technical notes*).
- Undo reverts through the local store; there is no "saving" state and no refusal as `MOD-05-F22` describes, since nothing waits for Google.
- Decisions: `DEC-178`, `DEC-180`.
