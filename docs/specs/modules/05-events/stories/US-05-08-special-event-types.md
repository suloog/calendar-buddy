---
id: US-05-08 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F23, MOD-05-F24, MOD-05-U01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-02] # story IDs that must be done first
---

# As a User, I can see out-of-office, focus-time and similar events at a glance and create out-of-office and focus time, so that my availability is clear

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F23`, `F24`, `U01` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-05-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Google's special event types: shown distinctly in the views, and out-of-office and focus-time events creatable where the account allows it.

## Acceptance criteria

- [ ] Out-of-office events are hatched, focus-time events carry an icon, and working-location and birthday events are shown as labels in the all-day strip.
- [ ] On the primary calendar of a Workspace account, the editor offers out-of-office and focus-time as event types, and the created events appear as such in Google.
- [ ] On personal accounts and other calendars, these types are not offered.

## Out of scope / what not to worry about

- Creating working-location events — `DEC-34`.

## Technical notes

- `eventType` values and where they are allowed in `MOD-05` *Technical notes*.
- Decisions: `DEC-34`, `DEC-60`.
