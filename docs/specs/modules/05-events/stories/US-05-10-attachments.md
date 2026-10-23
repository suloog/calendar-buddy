---
id: US-05-10 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F26] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-02] # story IDs that must be done first
---

# As a User, I can open an event's attachments, so that the documents for a meeting are one click away

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F26` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-05-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

An event's existing attachments, listed in its details as links that open in the browser. Adding attachments is not planned.

## Acceptance criteria

- [ ] An event with attachments lists them in its details by name.
- [ ] Choosing an attachment opens it in the browser.

## Out of scope / what not to worry about

- Adding attachments, ICS import or export — `DEC-37`.

## Technical notes

- Decision: `DEC-37`.
