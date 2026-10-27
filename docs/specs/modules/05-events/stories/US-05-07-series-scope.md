---
id: US-05-07 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-05
covers: [MOD-05-F20, MOD-05-F21, MOD-05-F29] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-05-06, US-05-03] # story IDs that must be done first
---

# As a User, I can change or delete one occurrence, the following ones or the whole series, so that I handle exceptions the way Google does

|  |  |
| --- | --- |
| **Module** | `[MOD-05]` Creating and editing events — `docs/specs/modules/05-events/REQUIREMENTS.md` |
| **Covers** | `MOD-05-F20`, `F21`, `F29` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | blocked by `US-05-06`, blocked by `US-05-03` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Working with occurrences of a series — in the editor and by dragging or resizing — with Google's three scopes, and keeping series with rules the presets cannot express intact. Notifying guests about a change is `MOD-06` (`MOD-06-F12`).

## Acceptance criteria

### Scenario: changing from the fifth occurrence on

- **Given** a weekly series of 10 occurrences
- **When** the user changes the time of the 5th occurrence and chooses "this and following events"
- **Then** occurrences 1–4 are unchanged and 5–10 have the new time, also in Google

- [ ] Changing an occurrence — in the editor, by dragging or by resizing — asks: this event, this and following events, or all events.
- [ ] Deleting an occurrence asks the same three-way question, and the result matches in Google.
- [ ] A series created in Google with a rule the presets cannot express keeps its rule when its other fields are edited, and the app offers to open it in Google Calendar to change the rule. Narrowed by `US-05-11` to rules the custom recurrence dialog cannot express (`DEC-165`).

## Edge cases

- "This and following" on the first occurrence → treated as "all events".

## Out of scope / what not to worry about

- Responding to occurrences of an invitation — `MOD-06`.

## Technical notes

- "This and following" splits the series (`UNTIL` on the old master, a new series from the chosen occurrence); test-first (`MOD-05` *Technical notes*, constitution).
- Decisions: `DEC-36`, `DEC-60`.
