---
id: DSC-07
title: Creating and editing events
kind: business # business (a functional area — becomes a module at /og:modules) | cross-cutting (roles and access, technology and integrations, non-functional requirements — feeds PROJECT.md, NFR.md and the constitution, never a module)
status: settled # open (not yet discussed) | discussed (discussed with the analyst, waiting on the client) | settled (no [W] question in this topic is open)
sources: [SRC-01, SRC-02] # SRC-NN documents whose items this topic gathers, e.g. [SRC-01, SRC-03]
module: # MOD-NN — filled by /og:modules; empty for cross-cutting topics
sessions: [2026-10-22] # dates of the discovery sessions in which the topic was discussed, YYYY-MM-DD
---

<!-- Copy to docs/specs/discovery/NN-<topic>.md. Content in the project language; headings fixed.
     This file records what we UNDERSTOOD about this topic after reading every source and talking
     to the analyst — the layer between the sources (what the client wrote) and the requirements
     (what we will build). It is alive: after every client workshop /og:discovery updates the
     findings. A finding is NOT a requirement: no acceptance criteria, no priority, no UI; one
     finding becomes 1–3 requirements in the module's REQUIREMENTS.md later. -->

# [DSC-07] Creating and editing events

## Topic scope

Creating events (click/select, quick entry, full editor, fields, all-day and multi-day, recurrence), editing, deleting, dragging and resizing, moving between calendars, recurring instances and series, warnings and permission failures. Guests, RSVP and Meet are DSC-08; reminders are DSC-09.

## Sources in this topic

<Items from ALL sources that speak about this topic, side by side — so a contradiction between the
contract and the kick-off note is visible in one place, not in two files.>

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-01-54` | informal | Create an event by clicking or selecting a time range. |
| `SRC-01-55` | informal | Offer quick entry plus a full event editor. |
| `SRC-01-56` | informal | Set title, description, start/end, time zone, and calendar. |
| `SRC-01-57` | informal | Support all-day and multi-day events. |
| `SRC-01-58` | informal | Set location, reminders, availability (busy/free), and visibility where supported. |
| `SRC-01-59` | informal | Create recurring events. |
| `SRC-01-61` | informal | Edit and delete events subject to user permissions. |
| `SRC-01-62` | informal | Drag events to a different time/date and resize their duration. |
| `SRC-01-63` | informal | Move events between calendars where the provider permits it. |
| `SRC-01-64` | informal | Handle recurring event instances and entire series; support changes to future occurrences where technically… |
| `SRC-01-65` | informal | Warn users when changes affect invited participants or an entire series. |
| `SRC-01-66` | informal | Display clear failures if an operation is disallowed by Google or the organizer's permissions. |
| `SRC-01-100` | informal | Create, edit, delete, and reschedule events. |
| `SRC-01-101` | informal | Basic recurring event management. |
| `SRC-01-113` | informal | Advanced calendar search. |
| `SRC-01-114` | informal | Google Drive attachments. |
| `SRC-01-115` | informal | Natural-language event creation. |
| `SRC-01-116` | informal | Task management or AI scheduling assistant. |
| `SRC-01-124` | informal | Reschedule, edit, and cancel events within their permissions. |
| `SRC-01-125` | informal | Manage common recurring event cases. |
| `SRC-02-23` | informal | Default calendar for new events chosen by the user. |
| `SRC-02-35` | informal | Calendars grouped by account in the picker. |
| `SRC-02-44` | informal | Choose event types (out of office…) and make them stand out. |
| `SRC-02-46` | informal | Out of office + focus time creatable; working location shown. |
| `SRC-02-47` | informal | Event colours shown and editable. |
| `SRC-02-48` | informal | Recurrence presets; three edit scopes; complex rules in the browser. |
| `SRC-02-49` | informal | Attachments as links; no ICS import/export. |

## Key findings

- [DSC-07-01] An event is created by clicking in the calendar or selecting a time range. (`SRC-01-54`)
- [DSC-07-02] There is a quick way to add the essentials and a full event editor. (`SRC-01-55`)
- [DSC-07-03] An event has a title, description, start and end, time zone and calendar; it can be all-day or span several days. (`SRC-01-56`, `SRC-01-57`)
- [DSC-07-04] Where Google allows it, the location, reminders, busy/free status and visibility can be set. (`SRC-01-58`)
- [DSC-07-05] A new event goes to the calendar the user chose as the default; for a given event another calendar can be picked from a list grouped by account. (`SRC-02-23`, `SRC-02-35`)
- [DSC-07-06] Events can be edited and deleted within the user's permissions; the time is changed by dragging and the length by resizing. (`SRC-01-61`, `SRC-01-62`, `SRC-01-100`, `SRC-01-124`)
- [DSC-07-07] An event can be moved to another calendar where Google allows it. (`SRC-01-63`)
- [DSC-07-08] The app warns when a change affects invited people or a whole series, and shows a clear message when Google or the organizer does not allow an operation. (`SRC-01-65`, `SRC-01-66`)
- [DSC-07-09] Google's special event types — out of office, focus time, working location, birthdays — stand out in the views. (`SRC-02-44`) — *source: `DEC-34`*
- [DSC-07-10] The user can create out-of-office and focus-time events where the account allows it (the primary calendar of a Workspace account); working location is shown only. (`SRC-02-46`) — *source: `DEC-34`*
- [DSC-07-11] An event's Google colour is shown and can be changed in the editor; the change is saved to Google. (`SRC-02-47`) — *source: `DEC-35`*
- [DSC-07-12] Recurring events are created from presets — daily, weekdays, weekly on chosen days, monthly, yearly — ending on a date, after a number of occurrences, or never. (`SRC-01-59`, `SRC-01-101`, `SRC-02-48`) — *source: `DEC-36`*
- [DSC-07-13] Changing or deleting an occurrence of a series offers three scopes: this event, this and following, all; series with more complex rules created in Google are shown correctly, and their rule is changed in the browser. (`SRC-01-64`, `SRC-01-125`, `SRC-02-48`) — *source: `DEC-36`*
- [DSC-07-14] An event's existing attachments are shown in its details as links that open in the browser; adding attachments is not possible. (`SRC-01-114`, `SRC-02-49`) — *source: `DEC-37`*

## Contradictions

- Recurrence depth: "future occurrences where technically feasible" (`SRC-01-64`), "basic" (`SRC-01-101`), "common cases" (`SRC-01-125`) → settled: presets and three edit scopes, `DEC-36` (was `Q07`)

## Ambiguities

- "Set … visibility where supported" (`SRC-01-58`) and Google's special event types: (a) ordinary events only, (b) also out of office / focus time → (b) where the account allows, `DEC-34` (was `Q15`)
- Editing in read-only calendars and offline is governed by `DSC-04-10` and `DSC-04-13`; not repeated here.

## From the calls, not the documents

- `SRC-02-23` default calendar for new events (said during `DSC-03`)
- `SRC-02-44` event types matter and must stand out
- `SRC-02-45` reminders matter — continued in `DSC-09`
- `SRC-02-46`…`SRC-02-49` answers to `Q15`, `Q07`, `Q16`, `Q25`

## For the client workshop

None open — `Q07`, `Q15`, `Q16`, `Q25` resolved on 2026-10-22.

## Parked

- ICS import and export — not in the MVP, `DEC-37`.
- Adding Google Drive attachments (`SRC-01-114`) — outside the MVP per the PRD; existing ones are shown as links.
- Natural-language event creation (`SRC-01-115`) — outside the MVP per the PRD.
- Task management or an AI scheduling assistant (`SRC-01-116`) — outside the MVP per the PRD.
- Event search (`SRC-01-113`) — `DEC-30`.
- Creating working-location events — display only, `DEC-34`.
- A full recurrence-rule editor (every N weeks, second Tuesday…) — such rules are changed in the browser, `DEC-36`.
