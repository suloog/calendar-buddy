---
id: US-05-09 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F25] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-02] # story IDs that must be done first
---

# As a User, I can see and change an event's own colour, so that it matches how I colour events in Google

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F25` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-05-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

An event's own Google colour (one of Google's event colours), shown in the views and changeable in the editor. Calendar colours are `MOD-02`.

## Acceptance criteria

- [ ] An event that has its own colour in Google is shown in that colour.
- [ ] Changing the event's colour in the editor shows in Google Calendar in the browser.

## Out of scope / what not to worry about

- Calendar colours and local overrides — `US-02-07`.

## Technical notes

- Colour IDs from Google's colour palette (`MOD-05` *Integrations*).
- Decision: `DEC-35`.
