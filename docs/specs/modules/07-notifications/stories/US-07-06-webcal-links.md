---
id: US-07-06 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-07
covers: [MOD-07-F18] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-01-01] # story IDs that must be done first
---

# As a User, I can open calendar subscription links from anywhere on my system, so that subscribing to a calendar works like in the browser

|  |  |
| --- | --- |
| **Module** | `[MOD-07]` Reminders and notifications — `docs/specs/modules/07-notifications/REQUIREMENTS.md` |
| **Covers** | `MOD-07-F18` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-01-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Calendar subscription links (`webcal:`) opened in other applications go to the app, which opens Google Calendar's "add calendar from URL" page in the browser. The subscription itself is made in Google and arrives through synchronization.

## Acceptance criteria

- [ ] Clicking a `webcal:` link in another application opens Google Calendar in the browser on its page for adding a calendar from a URL, with that address.
- [ ] After the user adds the calendar there, it appears in the app at the next synchronization.

## Out of scope / what not to worry about

- `mailto:` links and update checking — `DEC-46`.
- Managing calendars in the app — `DEC-21`.

## Technical notes

- Registered as `x-scheme-handler/webcal` (`MOD-07` *Technical notes*).
- Decision: `DEC-46`.
