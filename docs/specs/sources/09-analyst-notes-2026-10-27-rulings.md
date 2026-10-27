---
id: SRC-09
title: Analyst notes — product owner's rulings on events from a description, 2026-10-27
kind: informal
origin: analyst — Maciej (also the product owner)
date: 2026-10-27
received_via: conversation with the product owner, 2026-10-27, answering Q45–Q50 and the contradictions of SRC-08
supersedes:
status: read-only
---

# [SRC-09] Analyst notes — product owner's rulings on events from a description, 2026-10-27

## Summary

The product owner's answers to the contradictions and open questions of `SRC-08`: the function comes into the scope now that the MVP is closed, Ollama stays outside the package, and the details of filling the form, guests, the model, memory, privacy and failures are settled. It adds one thing `SRC-08` did not have: a description can also change an existing event.

## Items

### Scope

- [SRC-09-01] The MVP is closed and we build on: creating events from a description and the local model come into the scope, beyond following Google, as local calendars did; Microsoft 365, CalDAV and tasks stay out. — *conversation 2026-10-27*
- [SRC-09-02] Ollama outside the .deb is fine: the app works without it, only this function is missing; the instructions are in the README. — *conversation 2026-10-27*

### Answers to the analyst's questions

- [SRC-09-03] An "add" description opens the new-event window filled in as if the user were adding the event; the user approves it by hand. — *conversation 2026-10-27, answer to Q45*
- [SRC-09-04] A description can also change an existing event ("przesuń spotkanie z Anią na 15"): the event's editing opens with the changes made, approved by hand. The event is the selected one or the one the description names; when several fit, the app asks which. — *conversation 2026-10-27*
- [SRC-09-05] When a name fits several people or nobody, the guest field keeps the name with the suggestions open; no guest is added until the user picks one. — *conversation 2026-10-27, answer to Q46*
- [SRC-09-06] The model can be changed in the settings, next to the Ollama address; `gemma4:e4b` by default. — *conversation 2026-10-27, answer to Q47*
- [SRC-09-07] When the model is loaded and for how long it is kept: the analyst chooses. — *conversation 2026-10-27, answer to Q48*
- [SRC-09-08] Descriptions and their results are not kept anywhere — no history, no logs of their content. — *conversation 2026-10-27, answer to Q49*
- [SRC-09-09] When a description is not an event or the model's answer cannot be used, the app shows a message and the description stays in the box to be corrected. — *conversation 2026-10-27, answer to Q50*

## Contradictions and unclear points

- `SRC-09-04` goes beyond `SRC-08`, which spoke of creating events only — recorded as an extension, `DEC-195`.
