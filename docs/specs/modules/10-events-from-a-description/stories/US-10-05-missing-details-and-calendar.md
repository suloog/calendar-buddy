---
id: US-10-05 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-10
covers: [MOD-10-F09, MOD-10-F10, MOD-10-F13, MOD-10-F14, MOD-10-U06, MOD-10-U07] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
mode: A # A | B | C — agent autonomy · PE code review · PE pairs with the agent
depends_on: [US-10-02] # story IDs that must be done first
---

# As a User, I am asked for what my description left out and can name the calendar in it, so that a short sentence still gives the right event

|  |  |
| --- | --- |
| **Module** | `[MOD-10]` Events from a description — `docs/specs/modules/10-events-from-a-description/REQUIREMENTS.md` |
| **Covers** | `MOD-10-F09`, `F10`, `F13`, `F14`, `U06`, `U07` |
| **Priority** | must |
| **Mode** | A |
| **Status** | ready |
| **Depends on** | blocked by `US-10-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Two refinements of adding from a description: when the sentence gives no day or no time, the window leaves that field empty and asks for it instead of guessing; and a calendar named in the sentence ("w kalendarzu Praca") is chosen, or the default one is used with a note saying the named one was not found. The box, the model and the window itself are `US-10-02`.

## Acceptance criteria

- [ ] A day or time given in the description is filled in as in `US-10-02`; only a missing one is asked for.
- [ ] A calendar is matched by its name as shown in the side panel, ignoring letter case.

### Scenario: no time

- **Given** the description gives a day but no time ("Spotkanie z Michałem w sobotę")
- **When** it is sent
- **Then** the window opens on Sat 31 Oct with the time empty and the cursor in it, and Save is unavailable until a time is typed or all-day is chosen

### Scenario: no date

- **Given** the description gives no day ("Lunch o 13")
- **When** it is sent
- **Then** the window opens with the day empty and the cursor in it, and Save is unavailable until a day is chosen

### Scenario: a named calendar

- **Given** the description ends "…w kalendarzu Praca"
- **When** a calendar "Praca" exists and can take the event
- **Then** the window opens with "Praca" chosen
- **When** no such calendar exists, or it cannot take the event
- **Then** the window opens with the default calendar and the note "Calendar "Praca" not found — using <default>" under the calendar picker

## Edge cases

- Two calendars with the same name (local calendars may repeat names) → the first in the side panel's order is chosen.
- The named calendar is disabled, hidden, read-only or, offline, a Google one → treated as not found.

## Out of scope / what not to worry about

- The box, the model, dates and lengths — `US-10-02`.
- Choosing a calendar by account ("na koncie prywatnym") — only calendar names.

## Technical notes

- The "New event" window gains an empty day or time state with Save disabled; the rest of `MOD-05`'s window is unchanged.
- Manual check (left for the owner, as for the local loop's stories): the asked-for fields and the calendar note in the light and dark themes.
- Decisions: `DEC-201`, `DEC-203`.
