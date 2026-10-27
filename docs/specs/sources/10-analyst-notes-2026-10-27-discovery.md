---
id: SRC-10
title: Analyst notes — discovery session on events from a description, 2026-10-27
kind: informal
origin: analyst — Maciej (also the product owner)
date: 2026-10-27
received_via: interview, /og:discovery session on DSC-10, 2026-10-27
supersedes:
status: read-only
---

# [SRC-10] Analyst notes — discovery session on events from a description, 2026-10-27

## Summary

What the analyst added and ruled while walking the topic "Events from a description" (`DSC-10`): the function works with every calendar, the README carries a proper installation tutorial, and four open points — a missing time, which events a change can name, an unknown calendar, the quality threshold — are settled.

## Items

### Corrections to the reading of SRC-08 and SRC-09

- [SRC-10-01] It works for all calendars, not local ones only; offline, only local calendars. — *interview 2026-10-27, verdict on DSC-10-09*
- [SRC-10-02] The README has a sensible tutorial on installing Ollama and the recommended model, with links to Ollama. — *interview 2026-10-27, verdict on DSC-10-10*

### Open points settled by the analyst

- [SRC-10-03] When the description gives no time, the app asks for the time. — *interview 2026-10-27*
- [SRC-10-04] A description that changes an event names events from today on; past events are not matched. — *interview 2026-10-27*
- [SRC-10-05] When the named calendar does not exist or cannot take the event, the default calendar is used, with a note that the named one was not found. — *interview 2026-10-27*
- [SRC-10-06] The test set passes with dates right in at least 95 % of the sentences and all fields in at least 80 %. — *interview 2026-10-27*
