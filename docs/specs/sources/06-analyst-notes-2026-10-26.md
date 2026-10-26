---
id: SRC-06
title: Analyst notes — product owner's request 2026-10-26
kind: informal
origin: analyst — Maciej (also the product owner)
date: 2026-10-26
received_via: conversation with the product owner, 2026-10-26
supersedes:
status: read-only
---

# [SRC-06] Analyst notes — product owner's request 2026-10-26

## Summary

The product owner asks for local calendars and events that are not synchronized with Google, so that the app can be used without connecting a Google account. The welcome screen stays as an encouragement to connect one, but is no longer required. Several local calendars can be added in the settings and are grouped under "Local", as calendars are grouped by account today. The analyst's proposals for the details were all accepted.

## Items

### Local calendars

- [SRC-06-01] The app gets local calendars and events that are not synchronized with Google. — *conversation 2026-10-26*
- [SRC-06-02] Thanks to them, connecting a Google account is no longer necessary to use the app. — *conversation 2026-10-26*
- [SRC-06-03] The welcome screen stays as an encouragement and onboarding, but connecting an account is not required. — *conversation 2026-10-26*
- [SRC-06-04] The user can add several local calendars from the settings. — *conversation 2026-10-26*
- [SRC-06-05] Local calendars are grouped under "Local", as calendars are grouped by account today. — *conversation 2026-10-26*

### Details agreed on the analyst's proposals

- [SRC-06-06] A local event has what a Google event has: title, time, all-day, description, location, colour, repeating, dragging and undo. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-07] A local event has no guests, Google Meet, responses, attachments or special event types. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-08] Local events give notifications as Google events do, with a default notification set on the calendar. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-09] In the settings a local calendar can be added, renamed, recoloured and deleted; deleting one with events asks for confirmation and deletes them for good. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-10] An event is not moved between a local calendar and a Google calendar, at least for now. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-11] A local calendar can be the default calendar for new events; without a Google account it is the only choice. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-12] The group is named "Local" in the English interface and comes last in the side panel, after the accounts. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-13] The welcome screen gets a second action, "Use without an account", which creates a first local calendar ("My calendar") and goes to the calendar. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-14] The welcome screen shows only when there is neither an account nor a local calendar; removing the last account while local calendars exist does not go back to it. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-15] Local data is kept apart from the cache, in the app's data directory, so that it is not lost with the cache; no backup or export to begin with. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-16] Local events do not make the user busy for others in "Meet with…" and availability, because Google does not know them. — *conversation 2026-10-26, answer to the analyst's question*
- [SRC-06-17] The work goes through the spec workflow: a source, a decision, a new module with requirements, then stories and delivery. — *conversation 2026-10-26, answer to the analyst's question*

## Contradictions and unclear points

- `SRC-06-01`, `SRC-06-09` go against `DEC-21` (no calendar management) and against "providers other than Google" being out of scope — settled by `DEC-178`, for local calendars only.
- `SRC-06-08` goes against `DEC-43` (no local-only reminders) — settled by `DEC-178`, for local calendars only.
- `SRC-06-03`, `SRC-06-14` change `MOD-01-F12` and `MOD-01-F14` (`DEC-52`) — settled by `DEC-179`.
- `SRC-06-15` goes against `DEC-15` / `NFR-17` (Google is the system of record, no backups) — settled by `DEC-178`: local calendars have no other copy.
