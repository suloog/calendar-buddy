---
id: US-07-04 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-07
covers: [MOD-07-F03, MOD-07-F04, MOD-07-F05] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-02] # story IDs that must be done first
---

# As a User, I can set an event's reminders in the editor, starting from my calendar's defaults, so that my phone and computer remind me the same way

|  |  |
| --- | --- |
| **Module** | `[MOD-07]` Reminders and notifications — `docs/specs/modules/07-notifications/REQUIREMENTS.md` |
| **Covers** | `MOD-07-F03`, `F04`, `F05` |
| **Priority** | must |
| **Status** | review |
| **Depends on** | blocked by `US-05-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Reminders in the event editor: new events start with their calendar's default reminders from Google, the user adds or removes notification and e-mail reminders, and can go back to the calendar's defaults. Everything is saved to Google.

## Acceptance criteria

- [ ] A new event gets its calendar's default reminders from Google.
- [ ] In the editor the user adds and removes reminders — as a notification or an e-mail, a chosen number of minutes, hours or days before — up to Google's limit.
- [ ] A reminder changed in the app shows in Google Calendar in the browser.
- [ ] "Use calendar defaults" brings the event back to its calendar's default reminders.

## Out of scope / what not to worry about

- Changing a calendar's default reminders — done in Google (`DEC-43`).
- App-level default reminders and local-only reminders — `DEC-43`.

## Technical notes

- `useDefault` and override reminders on the event; `defaultReminders` from the calendar list (`MOD-07` *Integrations*).
- Decision: `DEC-43`.
