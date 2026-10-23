---
id: MOD-05
title: Creating and editing events
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [MOD-02, MOD-03] # other module IDs
sources: [SRC-01, SRC-02] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-05] Creating and editing events

## Context

Creating, changing and deleting events: quick entry and the full editor, the default calendar, dragging and resizing, moving between calendars, recurrence presets with three edit scopes, Google's special event types, event colours and attachments shown as links. Answers for the scope area "Creating and editing events"; comes from `DSC-07` (`discovery/07-events.md`), drawing on `SRC-01` and `SRC-02`. Holds the recurrence risk.

## Scope coverage

<Every source item this module answers for → the requirements that deliver it. Nothing from a
formal source may be silently dropped — a dropped or changed item points at the DEC-nn that
decided it. When the project has no sources register, list PROJECT.md "In scope" lines instead.>

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-54` Create an event by clicking or selecting a time range. | informal | |
| `SRC-01-55` Offer quick entry plus a full event editor. | informal | |
| `SRC-01-56` Set title, description, start/end, time zone, and calendar. | informal | |
| `SRC-01-57` Support all-day and multi-day events. | informal | |
| `SRC-01-58` Set location, reminders, availability (busy/free), and visibility where supported. | informal | |
| `SRC-01-59` Create recurring events. | informal | |
| `SRC-01-61` Edit and delete events subject to user permissions. | informal | |
| `SRC-01-62` Drag events to a different time/date and resize their duration. | informal | |
| `SRC-01-63` Move events between calendars where the provider permits it. | informal | |
| `SRC-01-64` Handle recurring event instances and entire series; support changes to future occurrenc… | informal | |
| `SRC-01-65` Warn users when changes affect invited participants or an entire series. | informal | |
| `SRC-01-66` Display clear failures if an operation is disallowed by Google or the organizer's permi… | informal | |
| `SRC-01-100` Create, edit, delete, and reschedule events. | informal | |
| `SRC-01-101` Basic recurring event management. | informal | |
| `SRC-01-114` Google Drive attachments. | informal | |
| `SRC-01-124` Reschedule, edit, and cancel events within their permissions. | informal | |
| `SRC-01-125` Manage common recurring event cases. | informal | |
| `SRC-02-23` The user picks which calendar is the default for creating new events. | informal | |
| `SRC-02-44` It is important to be able to choose the event type where possible, e.g. out of office,… | informal | |
| `SRC-02-46` Event types: out of office and focus time can be created where the account allows; work… | informal | |
| `SRC-02-47` Event colours from Google are shown and can be changed in the editor. | informal | |
| `SRC-02-48` Recurrence: presets daily, weekdays, weekly on chosen days, monthly, yearly; end by dat… | informal | |
| `SRC-02-49` Existing attachments shown as links; no ICS import or export in the MVP. | informal | |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

<One requirement = one checkable capability of the system. Business language only — no file
paths, no class or module names, no endpoints; those belong in Technical notes. If a requirement
needs an "and…, and…, and…" to describe it, split it.

Provenance: a requirement that answers a source item needs nothing more — the table above is its
proof. A requirement with no source item carries a trailing marker: `— *source: DEC-02*` for a
recorded decision, `— *assumption: Q04*` for a default awaiting the client. Never bake an
unrecorded assumption into a requirement.>

- [MOD-05-F01] …
- [MOD-05-F02] … — **priority: should**
- [MOD-05-F03] … — *source: `DEC-02`*
- [MOD-05-F04] … — **priority: nice-to-have** — *assumption: `Q04`*

## UI/UX requirements

<Visual or interaction requirements, same rules as above.>

- [MOD-05-U01] …

## Non-functional requirements

<Only ones specific to this module. Project-wide ones live in NFR.md as NFR-nn.>

- [MOD-05-NFR-01] …

## Constraints and edge cases

<Condition → expected behaviour: no data from the API, limits, boundary and failure cases.>

- …

## Integrations

<External systems this module depends on, which endpoints, and the direction of data flow.>

- …

## Technical notes

<Implementation assumptions and technical detail, kept out of the requirement text so the
requirements stay readable by the client.>

- …

## Out of scope / what not to worry about

<Explicit exclusions that pre-empt the team's questions: what deliberately does NOT belong here
and why — another module, handled globally by an NFR, computed upstream, dropped by a DEC.>

- …

## Acceptance criteria

<Module-level and verifiable. Each maps down to story-level criteria.>

- [ ] …
