---
id: US-10-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-10
covers: [MOD-10-F01, MOD-10-F02, MOD-10-F03, MOD-10-F04, MOD-10-F05, MOD-10-F06, MOD-10-F08, MOD-10-F22, MOD-10-F23, MOD-10-F24, MOD-10-F27, MOD-10-U01, MOD-10-U02, MOD-10-U03, MOD-10-NFR-01, MOD-10-NFR-03, MOD-10-NFR-04] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
mode: A # A | B | C — agent autonomy · PE code review · PE pairs with the agent
depends_on: [US-10-01] # story IDs that must be done first
---

# As a User, I can add an event by describing it in a sentence in Polish or English, so that it is in my calendar in seconds

|  |  |
| --- | --- |
| **Module** | `[MOD-10]` Events from a description — `docs/specs/modules/10-events-from-a-description/REQUIREMENTS.md` |
| **Covers** | `MOD-10-F01`, `F02`, `F03`, `F04`, `F05`, `F06`, `F08`, `F22`, `F23`, `F24`, `F27`, `U01`, `U02`, `U03`, `NFR-01`, `NFR-03`, `NFR-04` |
| **Priority** | must |
| **Mode** | A |
| **Status** | ready |
| **Depends on** | blocked by `US-10-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The description box in the side panel and the path from a sentence to a filled "New event" window: the title, the day, the time and the length in the default calendar, and an unusable description told as such. The user always saves by hand. Asking for a missing day or time and a calendar named in the description are `US-10-05`; guests and Meet are `US-10-03`; changing an existing event is `US-10-04`.

## Acceptance criteria

- [ ] The side panel has a field "Describe an event…" above "Meet with…", in the same style, growing to more lines as the user types; `q` focuses it, opening a collapsed side panel first; Enter sends, Shift+Enter starts a new line, Esc leaves the box; `q` is in the list of shortcuts.
- [ ] Focusing the box has Ollama load the model; the model is freed 5 minutes after it was last used.
- [ ] While the model reads, the box shows "Reading…" and the rest of the app stays usable; Esc cancels, and an answer arriving after Esc is dropped.
- [ ] A length in the description ("45 minut", "two hours", "godzina i kwadrans") is used; with none, the event gets the default length of a new event.
- [ ] Location, notes, repeating, reminders and colour stay as for any new event.
- [ ] Descriptions and the model's answers appear in no log and no file; a description over 500 characters is cut to 500 before it is sent.
- [ ] Offline, only local calendars can take the event, as when adding by hand; with no Google account, the default local calendar takes it.
- [ ] The fixed test set of adding sentences — Polish and English, at least the 30 of the test of 2026-10-27 — passes with the date right in at least 95 % and every field in at least 80 %: automatically against a fake model, and by hand against a real Ollama with `gemma4:e4b`.

### Scenario: a sentence with a part of the day

- **Given** it is Monday 26 Oct 2026, 09:00, and the default calendar is "My calendar"
- **When** the user types "Trening wieczorem" and presses Enter
- **Then** the "New event" window opens with the title "Trening", today 18:00 to 18:00 plus the default length, in "My calendar"; nothing is saved until Save

### Scenario: days and times worked out by the app

- **Given** it is Monday 26 Oct 2026, 09:00
- **When** the user describes "Dentysta w środę o 8:15", "przegląd w przyszły czwartek o 15", "standup za tydzień o 9", "rozmowa pojutrze o 14:30", "urodziny 14 listopada o 18", "kawa jutro rano", "retro w piątek po południu", or the same in English
- **Then** the window opens on Wed 28 Oct 08:15, Thu 5 Nov 15:00, Mon 2 Nov 09:00, Wed 28 Oct 14:30, Sat 14 Nov 18:00, Tue 27 Oct 09:00 and Fri 30 Oct 14:00 respectively

### Scenario: not an event

- **Given** the user types "jaka jest pogoda?", or the model's answer cannot be used
- **When** it is sent
- **Then** no window opens; the box says it found no event in the description and keeps the text to be corrected

## Edge cases

- A description in another language, or mixed → read as well as the model can; when unusable, the "not an event" message.
- A new event on a past day ("wczoraj") → the window opens on that day, as adding by hand allows.
- Ollama stops while the box is open → the request fails, the box turns unavailable (`US-10-01`) and keeps the text.
- Every calendar disabled, or none can take events → the message of `MOD-09` ("no calendar to create it in").

## Out of scope / what not to worry about

- Asking for a missing day or time, and a calendar named in the description — `US-10-05`.
- Guests and Meet from a description, and the full editor they open — `US-10-03`.
- Changing an existing event — `US-10-04`.
- The settings and the availability of Ollama — `US-10-01`.
- Filling in location, notes, repeating, reminders or colour; languages other than Polish and English — `MOD-10` out of scope.

## Technical notes

- Variant D of the test of 2026-10-27: structured output with the day as a kind from a closed list plus a short value; the app computes the date in the system's time zone; the current date and time go in the user's message so the system prompt is reused. See `MOD-10` technical notes.
- The test set moves from `~/llm-bench/cases.json` into the repo; a fake model answers it in unit tests, and an `#[ignore]` test runs it against a live Ollama.
- `NFR-07` (no event contents in logs) covers descriptions too.
- Manual check (left for the owner, as for the local loop's stories): the `#[ignore]` test set against a real Ollama with `gemma4:e4b`; the box and "Reading…" in the light and dark themes; the wait on a computer without a GPU.
- Decisions: `DEC-192`, `DEC-194`, `DEC-198`…`DEC-200`, `DEC-204`, `DEC-206`, `DEC-209`, `DEC-210`, `DEC-212`, `DEC-213`.
