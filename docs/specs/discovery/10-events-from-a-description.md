---
id: DSC-10
title: Events from a description
kind: business # business (a functional area — becomes a module at /og:modules) | cross-cutting (roles and access, technology and integrations, non-functional requirements — feeds PROJECT.md, NFR.md and the constitution, never a module)
status: settled # open (not yet discussed) | discussed (discussed with the analyst, waiting on the client) | settled (no [W] question in this topic is open)
sources: [SRC-08, SRC-09, SRC-10, SRC-11] # SRC-NN documents whose items this topic gathers, e.g. [SRC-01, SRC-03]
module: MOD-10 # MOD-NN — filled by /og:modules; empty for cross-cutting topics
sessions: [2026-10-27] # dates of the discovery sessions in which the topic was discussed, YYYY-MM-DD
---

<!-- Copy to docs/specs/discovery/NN-<topic>.md. Content in the project language; headings fixed.
     This file records what we UNDERSTOOD about this topic after reading every source and talking
     to the analyst — the layer between the sources (what the client wrote) and the requirements
     (what we will build). It is alive: after every client workshop /og:discovery updates the
     findings. A finding is NOT a requirement: no acceptance criteria, no priority, no UI; one
     finding becomes 1–3 requirements in the module's REQUIREMENTS.md later. -->

# [DSC-10] Events from a description

## Topic scope

Adding and changing events by writing a sentence in Polish or English, read by a local model that Ollama runs on the user's computer: the description box, what is filled in, the new-event and edit windows it opens, guests and calendars named in it, the settings, Ollama being unavailable, loading the model, privacy, and the quality check. The event windows themselves are `DSC-07`; guest suggestions are `DSC-08`; local calendars are `MOD-09`.

## Sources in this topic

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-08-01` | informal | Events created from a description in Polish or English, e.g. "wrzuć mi spotkanie z Anią i Tomkiem jutro o 10 z Meetem". |
| `SRC-08-02` | informal | A local model, run through Ollama, fills in the event editor. |
| `SRC-08-03` | informal | It fills in the title, day, time, length, guests from the contacts and Google Meet. |
| `SRC-08-04` | informal | The user approves the event by hand. |
| `SRC-08-05` | informal | No API and no cloud. |
| `SRC-08-06` | informal | When Ollama is not running, the function is not there. |
| `SRC-08-07` | informal | An entry box, which also opens with a keyboard shortcut. |
| `SRC-08-08` | informal | When the description gives no date, the app asks for it, where possible. |
| `SRC-08-09` | informal | Guests suggested from the contacts, the directory and recent addresses. |
| `SRC-08-10` | informal | The description can name the calendar when it is not the default one. |
| `SRC-08-11` | informal | The new-event form opens filled in; the user accepts it by hand. |
| `SRC-08-12` | informal | Location, notes, repeating, reminders and colour not filled in, for now. |
| `SRC-08-13` | informal | The user installs Ollama and the model by hand; a note in the README. |
| `SRC-08-14` | informal | The Ollama address is the local one by default; the user can change it. |
| `SRC-08-15` | informal | The function can be turned off in the settings, and is then hidden. |
| `SRC-08-16` | informal | On, with Ollama not available: the box is unavailable and says so. |
| `SRC-08-17` | informal | No response-time target: it depends on the user's computer. |
| `SRC-08-18` | informal | Works offline, with local calendars only and without a Google account. |
| `SRC-08-19` | informal | Only Polish and English, for now. |
| `SRC-08-20` | informal | Quality accepted on a fixed set of test sentences with a threshold. |
| `SRC-08-21` | informal | `gemma4:e4b` did best in the test. |
| `SRC-08-22` | informal | The model describes the day, the app computes the date: dates 100 %, all fields 87 %, ~2.5 s on a GPU. |
| `SRC-08-23` | informal | The model is loaded in advance; the first load takes ~5.6 s. |
| `SRC-09-01` | informal | MVP closed; events from a description and the local model come into scope. |
| `SRC-09-02` | informal | Ollama outside the .deb is fine; instructions in the README. |
| `SRC-09-03` | informal | "Add" opens the new-event window filled in as if added by hand. |
| `SRC-09-04` | informal | A description can change an existing event — the selected one or the one it names. |
| `SRC-09-05` | informal | A name fitting several people or nobody stays in the guest field with suggestions. |
| `SRC-09-06` | informal | The model can be changed in the settings; `gemma4:e4b` by default. |
| `SRC-09-07` | informal | Loading and keeping the model: the analyst chooses. |
| `SRC-09-08` | informal | Descriptions and results not kept anywhere. |
| `SRC-09-09` | informal | No usable event: a message, the description stays in the box. |
| `SRC-10-01` | informal | Works for all calendars; offline, only local ones. |
| `SRC-10-02` | informal | A tutorial in the README on installing Ollama and the model, with links. |
| `SRC-10-03` | informal | No time given: the app asks for it. |
| `SRC-10-04` | informal | A change names events from today on. |
| `SRC-10-05` | informal | Unknown calendar: the default one, with a note. |
| `SRC-10-06` | informal | Test set threshold: dates ≥ 95 %, all fields ≥ 80 %. |
| `SRC-11-01` | informal | A description with guests or Meet opens the full editor filled in; one without them opens the "New event" win… |
| `SRC-11-02` | informal | The description box opens with the `q` key. |
| `SRC-11-03` | informal | An Ollama address that is not on this computer is allowed, with a warning that descriptions will leave it. |
| `SRC-11-04` | informal | The selected event is the one whose details are open when the box is opened. |
| `SRC-11-05` | informal | The description box is in the side panel, where there is more room than in the top bar, above "Meet with…". |
| `SRC-11-06` | informal | The function is on by default. |
| `SRC-11-07` | informal | A part of the day without an hour means 9:00 in the morning, 14:00 in the afternoon and 18:00 in the evening. |
| `SRC-11-08` | informal | Only the first 500 characters of a description are sent to the model. |

## Key findings

- [DSC-10-01] The user can describe an event in a sentence in Polish or English, and the app fills the event in from it. (`SRC-08-01`, `SRC-08-19`) — *source: `DEC-192`*
- [DSC-10-02] The description is read by a local model that Ollama runs on the user's computer; nothing is sent to an API or the cloud, and neither descriptions nor results are kept. (`SRC-08-02`, `SRC-08-05`, `SRC-09-08`) — *source: `DEC-192`, `DEC-199`*
- [DSC-10-03] The description is typed in a box of its own, which also opens with a keyboard shortcut. (`SRC-08-07`)
- [DSC-10-04] From a description the app fills in the title, day, time, length, guests and Google Meet; location, notes, repeating, reminders and colour are left to the user. (`SRC-08-03`, `SRC-08-12`)
- [DSC-10-05] Adding opens the same "New event" window as adding by hand, already filled in; nothing is saved until the user saves it. (`SRC-08-04`, `SRC-08-11`, `SRC-09-03`) — *source: `DEC-194`*
- [DSC-10-06] A description can also change an existing event, either the selected one or one the description names. The event's editing opens with the changes made, approved by hand; when several events fit, the app asks which. (`SRC-09-04`) — *source: `DEC-195`*
- [DSC-10-07] The description can name the calendar; otherwise the event goes to the default calendar. (`SRC-08-10`)
- [DSC-10-08] Guests named in the description are looked up in the contacts, the directory and recent addresses. A name that fits one person adds that person; a name that fits several people or nobody stays in the guest field with the suggestions open. (`SRC-08-09`, `SRC-09-05`) — *source: `DEC-196`*
- [DSC-10-09] It works with every calendar, with or without a Google account. Offline only local calendars take the event, as when adding by hand (`DEC-103`). Local events get no guests and no Meet. (`SRC-08-18`, `SRC-10-01`) — *source: `DEC-178`*
- [DSC-10-10] The user installs Ollama and the model by hand, outside the .deb. The README has a proper tutorial: installing Ollama and pulling the recommended model, with links to Ollama. The app works without them; only this function is missing. (`SRC-08-13`, `SRC-09-02`, `SRC-10-02`) — *source: `DEC-193`*
- [DSC-10-11] In the settings the function can be turned off, and is then hidden; the Ollama address (the local one by default) and the model (`gemma4:e4b` by default) can be changed. (`SRC-08-14`, `SRC-08-15`, `SRC-09-06`) — *source: `DEC-197`*
- [DSC-10-12] When the function is on and Ollama is not available, the box is unavailable and says that Ollama is not available. (`SRC-08-06`, `SRC-08-16`)
- [DSC-10-13] The model is loaded when the box opens and freed 5 minutes after its last use. (`SRC-08-23`) — *source: `DEC-198`*
- [DSC-10-14] When a description is not an event, or the model's answer cannot be used, the app says so and the description stays in the box. (`SRC-09-09`) — *source: `DEC-200`*
- [DSC-10-15] There is no response-time target, because it depends on the user's computer. The model only describes the day, and the app computes the date. (`SRC-08-17`, `SRC-08-22`)
- [DSC-10-16] The quality is accepted on a fixed set of test sentences in both languages, against a threshold. (`SRC-08-20`) — *source: `DEC-204`*
- [DSC-10-17] When the description gives no time, the window opens with the time empty and asks for it; the user types it or chooses all-day. (`SRC-08-08`, `SRC-10-03`) — *source: `DEC-201`*
- [DSC-10-18] A description that changes an event is matched against events from today up to the end of the cache; past events are not matched. (`SRC-09-04`, `SRC-10-04`) — *source: `DEC-202`*
- [DSC-10-19] When the description names a calendar that does not exist or cannot take the event, the window opens with the default calendar and says the named one was not found. (`SRC-08-10`, `SRC-10-05`) — *source: `DEC-203`*
- [DSC-10-20] The fixed test set passes when dates are right in at least 95 % of the sentences and all fields in at least 80 %. (`SRC-08-20`, `SRC-10-06`) — *source: `DEC-204`*

## Contradictions

- `SRC-08-01` and `SRC-08-02` against `SRC-01-115` and `SRC-01-116` (natural-language creation and an AI assistant outside the MVP) → settled: in scope now the MVP is closed, `DEC-192`.
- `SRC-08-02`, `SRC-08-13` against G4 (installs from a .deb and works) → settled: Ollama outside the package, the app works without it, `DEC-193`.
- `SRC-08-06` ("not there") against `SRC-08-16` (shown unavailable) → read together with `SRC-08-15`: hidden only when turned off.
- `SRC-08-18` ("local calendars only") against `SRC-10-01` → every calendar; offline only local ones, `DSC-10-09`.
- `SRC-09-04` goes beyond `SRC-08`, which spoke of creating only → recorded as an extension, `DEC-195`.

## Ambiguities

- "Event editor" (`SRC-08-02`) or "new-event form" (`SRC-08-11`): (a) the "New event" window, (b) the full editor → (a), "More options" opens the full editor as usual, `DEC-194` (was `Q45`).
- A guest name that fits several people or nobody (`SRC-08-09`) → stays in the field with suggestions, `DEC-196` (was `Q46`).
- "Asks for the date" (`SRC-08-08`) said nothing of a missing time → asks for the time too, `DEC-201`.

## From the calls, not the documents

- `SRC-10-01` every calendar, offline only local ones
- `SRC-10-02` a tutorial in the README with links to Ollama
- `SRC-10-03`…`SRC-10-06` rulings on a missing time, which events a change names, an unknown calendar and the threshold

## For the client workshop

None open — `Q45`…`Q50` resolved on 2026-10-27 (`DEC-194`, `DEC-196`…`DEC-200`).

## Parked

- Filling in location, notes, repeating, reminders and colour — not for now (`SRC-08-12`).
- Languages other than Polish and English — not for now (`SRC-08-19`).
- An AI assistant beyond filling events — finding a free time, tasks — `DEC-192`.
- Ollama or a model shipped with the app, or downloaded by it — `DEC-193`.
- A history of descriptions — `DEC-199`.
