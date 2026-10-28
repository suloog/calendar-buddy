---
id: US-10-04 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-10
covers: [MOD-10-F15, MOD-10-F16, MOD-10-F17, MOD-10-F18, MOD-10-U08, MOD-10-NFR-04] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: review # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
mode: A # A | B | C — agent autonomy · PE code review · PE pairs with the agent
depends_on: [US-10-02] # story IDs that must be done first
---

# As a User, I can change an event by describing the change, so that moving a meeting is as quick as adding one

|  |  |
| --- | --- |
| **Module** | `[MOD-10]` Events from a description — `docs/specs/modules/10-events-from-a-description/REQUIREMENTS.md` |
| **Covers** | `MOD-10-F15`, `F16`, `F17`, `F18`, `U08`, `NFR-04` |
| **Priority** | must |
| **Mode** | A |
| **Status** | review |
| **Depends on** | blocked by `US-10-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

A description can change an existing event instead of adding one: the event whose details are open, or the one the description names among the events still ahead. The event's editing opens with the change made, and saving goes the usual way — the scope of a series, asking about guests, permissions. Nothing is changed until the user saves.

## Acceptance criteria

- [ ] Adding and changing are told apart by the description ("przesuń", "zmień", "move", "change"…); a sentence that changes an event never creates a new one.
- [ ] Only events from today to the end of the cache are matched by name; an event whose details are open is changed whatever its date.
- [ ] Saving follows the usual rules: the scope for an occurrence of a series, asking whether to notify the guests, the organizer's permissions; a read-only event opens locked as it does today.
- [ ] The test set gains at least 10 change sentences in Polish and English; with them it still passes the thresholds of `US-10-02` (date 95 %, every field 80 %).

### Scenario: the event whose details are open

- **Given** the details of "Sprint planning" (Mon 10:00–11:00) are open
- **When** the user sends "przesuń na 15"
- **Then** its editing opens with 15:00–16:00, the length kept; nothing is saved until Save

### Scenario: an event named in the description

- **Given** no details are open and exactly one event from today on matches "spotkanie z Anią"
- **When** the user sends "przesuń spotkanie z Anią na piątek"
- **Then** that event's editing opens on Friday at the same time

### Scenario: several events match

- **Given** three events ahead match "standup"
- **When** the user sends "przesuń standup na 10"
- **Then** the three are listed under the box with title, day, time and calendar; arrows and Enter pick one and open its editing; Esc goes back to the description

### Scenario: nothing matches

- **Given** no event from today on matches the description (a past one may)
- **When** it is sent
- **Then** the box says no such event was found and keeps the text

## Edge cases

- The user is a guest of the matched event → its editing opens as for a guest today; what cannot be changed stays locked.
- One occurrence of a series → saving asks for the scope (this / following / all).
- The answer names a change the editor has no field for → the change is not applied; the editing opens with what could be applied.

## Out of scope / what not to worry about

- Changing several events with one description; deleting events by description — `MOD-10` out of scope.
- Past events by name — only when their details are open (`DEC-202`).
- Adding events — `US-10-02`, `US-10-03`.

## Technical notes

- Matching uses the cache, from today to its end (`DEC-23`, `DEC-202`); the model returns the change and the words naming the event, the app does the matching.
- Editing reuses the event editor and its save path, so series scope, guest notification (`MOD-06-F12`) and permissions come for free.
- Manual check (left for the owner, as for the local loop's stories): changing a real Google event with guests, and one occurrence of a series; the list of matches in both themes.
- Decisions: `DEC-195`, `DEC-202`, `DEC-204`, `DEC-208`.
