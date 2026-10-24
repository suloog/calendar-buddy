---
id: SRC-04
title: Analyst notes — product owner's requests 2026-10-24
kind: informal
origin: analyst — Maciej (also the product owner)
date: 2026-10-24
received_via: conversation with the product owner, 2026-10-24
supersedes:
status: read-only
---

# [SRC-04] Analyst notes — product owner's requests 2026-10-24

## Summary

Two features the product owner found missing while using the app: configuring how an event repeats, and zooming the hours of the day views.

## Items

### Recurrence

- [SRC-04-01] Two features are missing; the first is being able to configure how an event repeats. — *conversation 2026-10-24*
- [SRC-04-02] Configuring repetition goes as far as Google Calendar's "Custom" recurrence: every N days, weeks, months or years; chosen days of the week; in a month on the same day or on the same weekday ("the fourth Tuesday", "the last Friday"); ends never, on a date or after N occurrences; the repeat menu as Google's; rules from Google that this can express become editable in the app. — *conversation 2026-10-24, answer to the analyst's question*

### Zooming the hours

- [SRC-04-03] The second missing feature is zooming, only vertically: stretching the hours of the day. — *conversation 2026-10-24*
- [SRC-04-04] Zooming works with Ctrl + mouse wheel or a touchpad pinch, with Ctrl + plus / minus / 0, and with a slider showing the value in the settings. — *conversation 2026-10-24, answer to the analyst's question*
- [SRC-04-05] The chosen height of an hour is one for the day, 3-day and week views and is remembered after a restart. — *conversation 2026-10-24, answer to the analyst's question*

## Contradictions and unclear points

- `SRC-04-02` widens `DEC-36`, which kept the full rule editor out of the MVP — settled by `DEC-165`.
- Google Calendar in the browser has no zoom of the hours of its own (`DEC-62` follows Google) — the product owner asks for it explicitly; settled by `DEC-166`.
