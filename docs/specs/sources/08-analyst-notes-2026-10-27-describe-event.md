---
id: SRC-08
title: Analyst notes — product owner's request 2026-10-27 (events from a description, local model)
kind: informal
origin: analyst — Maciej (also the product owner)
date: 2026-10-27
received_via: conversation with the product owner, 2026-10-27, with the test of local models run the same day (~/llm-bench)
supersedes:
status: read-only
---

# [SRC-08] Analyst notes — product owner's request 2026-10-27 (events from a description, local model)

## Summary

The product owner asks for creating events from a description written in Polish or English: a local model, run through Ollama, fills in the new-event form, and the user accepts it by hand. Nothing goes to an API or the cloud. The request comes with the results of a test of local models run the same day, and the product owner answered the analyst's questions at once; how the form is filled, when the model is loaded and whether descriptions are kept are left to the analyst.

## Items

### Events from a description

- [SRC-08-01] Events can be created from a description written in Polish or English, e.g. "wrzuć mi spotkanie z Anią i Tomkiem jutro o 10 z Meetem". — *conversation 2026-10-27*
- [SRC-08-02] A local model, run through Ollama, fills in the event editor. — *conversation 2026-10-27*
- [SRC-08-03] It fills in the title, the day, the time, the length, guests from the contacts and Google Meet. — *conversation 2026-10-27*
- [SRC-08-04] The user approves the event by hand. — *conversation 2026-10-27*
- [SRC-08-05] No API and no cloud. — *conversation 2026-10-27*
- [SRC-08-06] When Ollama is not running, the function is not there. — *conversation 2026-10-27*

### Details agreed on the analyst's questions

- [SRC-08-07] The description is typed in an entry box, which also opens with a keyboard shortcut. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-08] When the description gives no date, the app asks for it, where possible. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-09] Guests are suggested from the contacts, the organization's directory and recent addresses. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-10] The description can name the calendar when it is not the default one. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-11] The app opens the new-event form filled in; the user accepts it by hand. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-12] Location, notes, repeating, reminders and colour are not filled in, for now. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-13] The user installs Ollama and the model by hand; the README has a short note on how. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-14] The Ollama address is the local one by default; the user can change it. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-15] The function can be turned off in the settings, and is then hidden. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-16] When it is on and Ollama is not available, the box is unavailable and says that Ollama is not available. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-17] No response-time target: it depends on the user's computer. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-18] It works offline, with local calendars only and without a Google account. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-19] Only Polish and English, for now. — *conversation 2026-10-27, answer to the analyst's question*
- [SRC-08-20] The quality is accepted on a fixed set of test sentences with a threshold, as in the test. — *conversation 2026-10-27, answer to the analyst's question*

### Findings of the test of local models, 2026-10-27

- [SRC-08-21] Of the models tried, `gemma4:e4b` did best. — *test 2026-10-27, `~/llm-bench/out-r2/summary.md`*
- [SRC-08-22] The model describes the day from a closed list and the app computes the date: dates right in 100 % of the test sentences, all fields in 87 %; about 2.5 s per description on the RTX A2000 GPU, several times slower on a CPU only. — *test 2026-10-27, variant D*
- [SRC-08-23] The model is loaded in advance, because loading it the first time takes about 5.6 s. — *test 2026-10-27, latency breakdown*

## Noted but not scope

- The Claude subscription through `claude -p` is not faster (about 5 s with Haiku) and needs the network — the reason behind `SRC-08-05`. — *test 2026-10-27*
- How the form is filled (the quick-entry box or the full editor), when the model is loaded and for how long it is kept, and whether descriptions are kept anywhere are left to the analyst. — *conversation 2026-10-27, answer to the analyst's question*

## Contradictions and unclear points

- `SRC-08-01` goes against `SRC-01-115` (natural-language event creation, outside the MVP) and `SRC-08-02` against `SRC-01-116` (AI scheduling assistant), both kept in `MOD-08` — needs a decision.
- `SRC-08-01` goes beyond Google Calendar, against following Google (`DEC-62`), as local calendars did (`DEC-178`) — needs a decision.
- `SRC-08-02`, `SRC-08-13` add a program the .deb does not ship (G4): the app works without it, the function does not — needs a decision.
- `SRC-08-06` ("not there") and `SRC-08-16` (shown unavailable) read together with `SRC-08-15`: hidden only when turned off in the settings.
- `SRC-08-02` ("event editor") and `SRC-08-11` ("new-event form") — the quick-entry box or the full editor → `Q45`.
- `SRC-08-03`, `SRC-08-09` do not say what the form shows when a name fits several people or nobody → `Q46`.
- `SRC-08-14`, `SRC-08-21` do not say whether the model can be chosen → `Q47`.
- `SRC-08-23` does not say when the model is loaded and how long it is kept → `Q48`.
- `SRC-08-05` does not say whether descriptions or results are kept → `Q49`.
- `SRC-08-01` does not say what happens when the description is not an event or the model's answer cannot be used → `Q50`.
