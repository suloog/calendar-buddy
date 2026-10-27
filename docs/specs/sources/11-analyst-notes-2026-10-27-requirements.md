---
id: SRC-11
title: Analyst notes — requirements session on events from a description, 2026-10-27
kind: informal
origin: analyst — Maciej (also the product owner)
date: 2026-10-27
received_via: interview, /og:requirements MOD-10, 2026-10-27
supersedes:
status: read-only
---

# [SRC-11] Analyst notes — requirements session on events from a description, 2026-10-27

## Summary

The details the requirements of `MOD-10` needed and no finding gave, settled by the analyst while writing them: which window opens for guests and Meet, the key and the place of the description box, an Ollama address off the computer, what "the selected event" is, the function being on by default, the hours of a part of the day, and a limit on a description's length.

## Items

- [SRC-11-01] A description with guests or Meet opens the full editor filled in; one without them opens the "New event" window. — *interview 2026-10-27*
- [SRC-11-02] The description box opens with the `q` key. — *interview 2026-10-27*
- [SRC-11-03] An Ollama address that is not on this computer is allowed, with a warning that descriptions will leave it. — *interview 2026-10-27*
- [SRC-11-04] The selected event is the one whose details are open when the box is opened. — *interview 2026-10-27*
- [SRC-11-05] The description box is in the side panel, where there is more room than in the top bar, above "Meet with…". — *interview 2026-10-27*
- [SRC-11-06] The function is on by default. — *interview 2026-10-27*
- [SRC-11-07] A part of the day without an hour means 9:00 in the morning, 14:00 in the afternoon and 18:00 in the evening. — *interview 2026-10-27*
- [SRC-11-08] Only the first 500 characters of a description are sent to the model. — *interview 2026-10-27*
