---
id: MOD-10
title: Events from a description
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [MOD-02, MOD-05, MOD-06, MOD-09] # other module IDs
sources: [SRC-08, SRC-09, SRC-10, SRC-11] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-10] Events from a description

## Context

Adding and changing events by writing a sentence in Polish or English: a local model that Ollama runs on the user's computer reads it, and the app opens the "New event" window or the event's editing filled in, for the user to approve by hand; nothing leaves the computer. Answers for the scope area "Events from a description" of PROJECT.md; comes from `DSC-10` (`discovery/10-events-from-a-description.md`), drawing on `SRC-08`, `SRC-09` and `SRC-10`, settled by `DEC-192`…`DEC-210`. It goes beyond Google Calendar (`DEC-62`) at the product owner's word. The event windows are `MOD-05`'s, guest suggestions `MOD-06`'s, local calendars `MOD-09`'s — this module names only what a description adds.

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-08-01` Events can be created from a description written in Polish or English, e.g. "wrzuć mi spotkani… | informal | `MOD-10-F01` |
| `SRC-08-02` A local model, run through Ollama, fills in the event editor. | informal | `MOD-10-F02` |
| `SRC-08-03` It fills in the title, the day, the time, the length, guests from the contacts and Google Meet. | informal | `MOD-10-F03`, `MOD-10-F07` |
| `SRC-08-04` The user approves the event by hand. | informal | `MOD-10-F06` |
| `SRC-08-05` No API and no cloud. | informal | `MOD-10-F02`, `MOD-10-NFR-02` |
| `SRC-08-06` When Ollama is not running, the function is not there. | informal | `MOD-10-F21` |
| `SRC-08-07` The description is typed in an entry box, which also opens with a keyboard shortcut. | informal | `MOD-10-F01`, `MOD-10-U01` |
| `SRC-08-08` When the description gives no date, the app asks for it, where possible. | informal | `MOD-10-F13`, `MOD-10-U07` |
| `SRC-08-09` Guests are suggested from the contacts, the organization's directory and recent addresses. | informal | `MOD-10-F11` |
| `SRC-08-10` The description can name the calendar when it is not the default one. | informal | `MOD-10-F09` |
| `SRC-08-11` The app opens the new-event form filled in; the user accepts it by hand. | informal | `MOD-10-F06` |
| `SRC-08-12` Location, notes, repeating, reminders and colour are not filled in, for now. | informal | `MOD-10-F08` |
| `SRC-08-13` The user installs Ollama and the model by hand; the README has a short note on how. | informal | `MOD-10-F26` |
| `SRC-08-14` The Ollama address is the local one by default; the user can change it. | informal | `MOD-10-F20`, `MOD-10-U05` |
| `SRC-08-15` The function can be turned off in the settings, and is then hidden. | informal | `MOD-10-F19`, `MOD-10-U05` |
| `SRC-08-16` When it is on and Ollama is not available, the box is unavailable and says that Ollama is not… | informal | `MOD-10-F21`, `MOD-10-U04` |
| `SRC-08-17` No response-time target: it depends on the user's computer. | informal | `MOD-10-NFR-03`, `MOD-10-U02` |
| `SRC-08-18` It works offline, with local calendars only and without a Google account. | informal | `MOD-10-F24`, `MOD-10-F25` |
| `SRC-08-19` Only Polish and English, for now. | informal | `MOD-10-F01` |
| `SRC-08-20` The quality is accepted on a fixed set of test sentences with a threshold, as in the test. | informal | `MOD-10-NFR-04` |
| `SRC-08-21` Of the models tried, `gemma4:e4b` did best. | informal | `MOD-10-F20` (the default model) |
| `SRC-08-22` The model describes the day from a closed list and the app computes the date: dates right in 1… | informal | `MOD-10-F04` (technical notes: variant D) |
| `SRC-08-23` The model is loaded in advance, because loading it the first time takes about 5.6 s. | informal | `MOD-10-F22` |
| `SRC-09-01` The MVP is closed and we build on: creating events from a description and the local model come… | informal | this module (scope ruling, `DEC-192`; no requirement) |
| `SRC-09-02` Ollama outside the .deb is fine: the app works without it, only this function is missing; the… | informal | `MOD-10-F26` |
| `SRC-09-03` An "add" description opens the new-event window filled in as if the user were adding the event… | informal | `MOD-10-F06` |
| `SRC-09-04` A description can also change an existing event ("przesuń spotkanie z Anią na 15"): the event'… | informal | `MOD-10-F15`, `MOD-10-F16`, `MOD-10-F17`, `MOD-10-F18`, `MOD-10-U08` |
| `SRC-09-05` When a name fits several people or nobody, the guest field keeps the name with the suggestions… | informal | `MOD-10-F12` |
| `SRC-09-06` The model can be changed in the settings, next to the Ollama address; `gemma4:e4b` by default. | informal | `MOD-10-F20` |
| `SRC-09-07` When the model is loaded and for how long it is kept: the analyst chooses. | informal | `DEC-198` → `MOD-10-F22` |
| `SRC-09-08` Descriptions and their results are not kept anywhere — no history, no logs of their content. | informal | `MOD-10-NFR-01` |
| `SRC-09-09` When a description is not an event or the model's answer cannot be used, the app shows a messa… | informal | `MOD-10-F23` |
| `SRC-10-01` It works for all calendars, not local ones only; offline, only local calendars. | informal | `MOD-10-F24` |
| `SRC-10-02` The README has a sensible tutorial on installing Ollama and the recommended model, with links… | informal | `MOD-10-F26` |
| `SRC-10-03` When the description gives no time, the app asks for the time. | informal | `MOD-10-F14`, `MOD-10-U07` |
| `SRC-10-04` A description that changes an event names events from today on; past events are not matched. | informal | `MOD-10-F16` |
| `SRC-10-05` When the named calendar does not exist or cannot take the event, the default calendar is used,… | informal | `MOD-10-F10`, `MOD-10-U06` |
| `SRC-10-06` The test set passes with dates right in at least 95 % of the sentences and all fields in at le… | informal | `MOD-10-NFR-04` |
| `SRC-11-01` A description with guests or Meet opens the full editor filled in; one without them opens the… | informal | `MOD-10-F06` |
| `SRC-11-02` The description box opens with the `q` key. | informal | `MOD-10-U01`, `MOD-10-U03` |
| `SRC-11-03` An Ollama address that is not on this computer is allowed, with a warning that descriptions wi… | informal | `MOD-10-U05`, `MOD-10-NFR-02` |
| `SRC-11-04` The selected event is the one whose details are open when the box is opened. | informal | `MOD-10-F16` |
| `SRC-11-05` The description box is in the side panel, where there is more room than in the top bar, above… | informal | `MOD-10-U01` |
| `SRC-11-06` The function is on by default. | informal | `MOD-10-F19` |
| `SRC-11-07` A part of the day without an hour means 9:00 in the morning, 14:00 in the afternoon and 18:00… | informal | `MOD-10-F27` |
| `SRC-11-08` Only the first 500 characters of a description are sent to the model. | informal | constraints (500 characters, `DEC-210`) |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

### Describing an event

- [MOD-10-F01] The user can open the description box, type a sentence in Polish or English that describes an event, and send it; the app fills an event in from it.
- [MOD-10-F02] The description is read by the local model through Ollama, at the address and with the model set in the settings; no other service receives it.
- [MOD-10-F03] From a description that adds an event, the app fills in the title, the day, the start time, the length, the guests and Google Meet.
- [MOD-10-F04] The app works the day out itself from what the description says (today, tomorrow, the day after tomorrow, a weekday, a weekday of next week, in N days or weeks, a date), in the user's time zone; the model only says which of these it is.
- [MOD-10-F05] A length the description does not give is the default length of a new event, as when adding by hand (`MOD-05-F09`). — *source: `DEC-194`*
- [MOD-10-F06] A description without guests and without Meet opens the "New event" window filled in; one with guests or Meet opens the full editor filled in. Nothing is saved until the user saves it. — *source: `DEC-194`, `DEC-205`*
- [MOD-10-F07] The Meet switch follows the description: on when it asks for Meet, off when it refuses it, and as for any new event when it says nothing (`MOD-06-F17`). — *source: `DEC-194`*
- [MOD-10-F08] Location, notes, repeating, reminders and colour are never filled in from a description; they keep the values of a new event.
- [MOD-10-F27] A description that gives only a part of the day starts the event at 9:00 for the morning, 14:00 for the afternoon and 18:00 for the evening. — *source: `DEC-209`*

### Calendar, guests and missing details

- [MOD-10-F09] The description can name a calendar; the event then goes to it, otherwise to the default calendar (`MOD-05-F10`).
- [MOD-10-F10] When the named calendar does not exist or cannot take the event, the window opens with the default calendar and says that the named one was not found. — *source: `DEC-203`*
- [MOD-10-F11] Guests named in the description are looked up as when typing a guest — the contacts, the organization's directory and recent addresses (`MOD-06-F02`…`F04`); a name that fits exactly one person adds that person. — *source: `DEC-196`*
- [MOD-10-F12] A name that fits several people or nobody is left in the guest field with the suggestions open; nobody is added until the user picks someone. — *source: `DEC-196`*
- [MOD-10-F13] When the description gives no date, the window opens without a day and asks for it.
- [MOD-10-F14] When the description gives a day but no time, the window opens without a time and asks for it; the user types it or chooses all-day. — *source: `DEC-201`*

### Changing an event

- [MOD-10-F15] A description can change an existing event instead of adding one; the app tells the two apart by what the description says ("przesuń", "zmień", "move", "change"…). — *source: `DEC-195`*
- [MOD-10-F16] The event changed is the one whose details are open when the box is opened; otherwise the one the description names, among events from today to the end of the cache. — *source: `DEC-202`, `DEC-208`*
- [MOD-10-F17] When several events fit the description, the app lists them (title, day, time, calendar) and the user picks one; when none fits, it says so and the description stays in the box. — *source: `DEC-195`*
- [MOD-10-F18] A change opens the event's editing with the changes made; nothing is saved until the user saves it, and saving follows the usual rules — the scope for a series, asking about guests, permissions (`MOD-05`, `MOD-06-F12`). — *source: `DEC-195`*

### Settings, availability and failures

- [MOD-10-F19] The function is on by default and can be turned off in the settings; turned off, the box, its `q` key and its entry in the list of shortcuts are gone.
- [MOD-10-F20] The settings have the Ollama address, the local one by default, and the model's name, `gemma4:e4b` by default; both can be changed. — *source: `DEC-197`*
- [MOD-10-F21] When the function is on but Ollama does not answer at that address, or does not have the model, the box is unavailable and says which of the two it is.
- [MOD-10-F22] Focusing the box has Ollama load the model; the model is freed 5 minutes after it was last used. — *source: `DEC-198`*
- [MOD-10-F23] When the description is not an event, or the model's answer cannot be used, no window opens; the box says so and keeps the description to be corrected. — *source: `DEC-200`*
- [MOD-10-F24] It works with every calendar, with or without a Google account; offline, only local calendars take the event, as when adding by hand (`DEC-103`).
- [MOD-10-F25] For a local calendar, guests and Meet from the description are left out, as local events have none (`MOD-09-F10`). — *source: `DEC-178`*
- [MOD-10-F26] The README has a tutorial on installing Ollama and pulling the recommended model, with links to Ollama, and says the app works without them. — *source: `DEC-193`*

## UI/UX requirements

- [MOD-10-U01] The description box is a field in the side panel, "Describe an event…", above "Meet with…", in the same style, growing to more lines as the user types. The `q` key focuses it, opening the side panel first when it is collapsed. Enter sends the description, Shift+Enter starts a new line, and Esc leaves the box. — *source: `DEC-206`*
- [MOD-10-U02] While the model reads the description, the box shows "Reading…" and Esc cancels it; the rest of the app stays usable.
- [MOD-10-U03] The list of keyboard shortcuts (`MOD-03-U04`, `MOD-03-U07`) includes `q`. — *source: `DEC-206`*
- [MOD-10-U04] An unavailable box is greyed out and says why, e.g. "Ollama is not available at http://localhost:11434" or "The model gemma4:e4b is not installed", with a link to the settings.
- [MOD-10-U05] The settings have an "Events from a description" section: an on/off checkbox, the Ollama address and the model's name. When the address is not on this computer, the section warns that descriptions will leave it. — *source: `DEC-197`, `DEC-207`*
- [MOD-10-U06] When the named calendar was not found, a note under the calendar picker says so, e.g. "Calendar "Praca" not found — using My calendar". — *source: `DEC-203`*
- [MOD-10-U07] A missing date or time is asked for by leaving that field empty and putting the cursor in it; Save stays unavailable until it is filled in or all-day is chosen. — *source: `DEC-201`*
- [MOD-10-U08] When several events fit, they are listed under the box in the side panel; arrows and Enter pick one, Esc goes back to the description.

## Non-functional requirements

- [MOD-10-NFR-01] Descriptions and the model's answers are not stored anywhere and never appear in the logs; only the event the user saves is kept. — *source: `DEC-199`*
- [MOD-10-NFR-02] With the default, local address, nothing about a description leaves the computer. The only network traffic the function adds goes to the Ollama address set in the settings (the exception in `NFR-06`). — *source: `DEC-207`*
- [MOD-10-NFR-03] There is no response-time target, because it depends on the user's computer; the interface stays responsive while the model works (`NFR-01`).
- [MOD-10-NFR-04] With the default model, a fixed set of test sentences in Polish and English, adding and changing, passes with the date right in at least 95 % of them and every field in at least 80 %. — *source: `DEC-204`*

## Constraints and edge cases

- A description in another language, or mixed → read as well as the model can; when unusable, the message of `MOD-10-F23`.
- A new event described on a past day ("wczoraj") → the window opens on that day, as adding by hand allows.
- A change described while the selected event is read-only, or the user is a guest → its editing opens as for that event today (`MOD-06-F15`), what cannot be changed stays locked.
- A change of one occurrence of a series → saving asks for the scope (this / following / all), as today.
- The model's answer arrives after Esc → it is dropped.
- Ollama stops while the box is open → the request fails and the box turns unavailable (`MOD-10-F21`) with the description kept.
- No account connected → no guests or Meet are looked up; local calendars only (`MOD-10-F24`, `MOD-10-F25`).
- Every calendar disabled, or none can take events → the message of `MOD-09` ("no calendar to create it in").
- A description longer than 500 characters → the box takes it, only the first 500 are sent (`DEC-210`).

## Integrations

- **Ollama** — on this computer by default, or at the address set: the app asks it to load or free the model and sends it the description; it answers with the event's description in a fixed form. Nothing else leaves the app.

## Technical notes

- An Ollama client in `calendar-core`, behind an interface like the calendar provider (`NFR-14`), with a fake for tests, as the fake Google.
- Requests use `/api/chat` with a JSON schema (structured output), `temperature 0` and `think: false`. The answer gives the day as a kind from a closed list (`today`, `tomorrow`, `day_after_tomorrow`, `weekday`, `next_weekday`, `in_days`, `date`) plus a short value. The app computes the date in the system's time zone. This is variant D of the test of 2026-10-27, which scored 100 % on dates and 87 % on all fields at about 2.5 s on an RTX A2000 (`SRC-08-22`).
- The prompt carries the current date and time in the user's message, not the system prompt, so that Ollama can reuse the system prompt between descriptions.
- Loading the model sends an empty request with a `keep_alive` of 5 minutes; nothing is loaded at the app's start.
- The test set and its runner come from `~/llm-bench`, moved into the repo as an `#[ignore]` test against a live Ollama, run by hand; sentences that change events are added to it.

## Out of scope / what not to worry about

- Filling in location, notes, repeating, reminders and colour (`SRC-08-12`).
- Languages other than Polish and English (`SRC-08-19`).
- Finding a free time, suggesting times, tasks or any AI assistant beyond filling one event (`DEC-192`).
- Shipping Ollama or a model with the app, or downloading one from the app (`DEC-193`).
- A history of descriptions, or undo for one (`DEC-199`).
- Changing several events with one description, or deleting events by description.
- Past events in a change by description, unless the event is selected (`DEC-202`).

## Acceptance criteria

- [ ] "wrzuć mi spotkanie z Anią i Tomkiem jutro o 10 z Meetem" opens the full editor for tomorrow at 10:00 with both guests (when each fits one contact) and Meet on; nothing is saved before Save.
- [ ] "Trening wieczorem" opens the "New event" window today at 18:00 in the default calendar.
- [ ] With an event's details open, "przesuń na 15" opens its editing at 15:00.
- [ ] With Ollama stopped the box says so; turned off in the settings the box is gone.
- [ ] The test set passes the thresholds of `MOD-10-NFR-04`.
- [ ] No description shows up in the logs or the app's files.
