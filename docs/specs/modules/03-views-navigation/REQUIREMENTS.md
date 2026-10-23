---
id: MOD-03
title: Views and navigation
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [MOD-02] # other module IDs
sources: [SRC-01, SRC-02] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-03] Views and navigation

## Context

How the user looks at the calendar and moves around it: day, 3-day, week, month and agenda views, the side panel with calendars grouped by account, hiding and disabling calendars, keyboard-first navigation, and regional formats and time zone taken from the system. Answers for the scope area "Views and navigation"; comes from `DSC-05` (`discovery/05-views-navigation.md`), drawing on `SRC-01` and `SRC-02`. Holds the UI-toolkit feasibility risk (`DEC-33`).

## Scope coverage

<Every source item this module answers for → the requirements that deliver it. Nothing from a
formal source may be silently dropped — a dropped or changed item points at the DEC-nn that
decided it. When the project has no sources register, list PROJECT.md "In scope" lines instead.>

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-36` Day view. | informal | |
| `SRC-01-38` Full-week view. | informal | |
| `SRC-01-39` Month view. | informal | |
| `SRC-01-40` Agenda / upcoming events view. | informal | |
| `SRC-01-41` Navigate quickly between days, weeks, and months. | informal | |
| `SRC-01-42` Jump back to Today. | informal | |
| `SRC-01-43` Provide a mini calendar and sidebar listing accounts and calendars. | informal | |
| `SRC-01-44` Allow the sidebar to be collapsed. | informal | |
| `SRC-01-45` Highlight the current day and current-time indicator. | informal | |
| `SRC-01-46` Support keyboard shortcuts and keyboard-first navigation. | informal | |
| `SRC-01-47` Scroll smoothly and handle overlapping events legibly. | informal | |
| `SRC-01-48` Resize gracefully across different window dimensions and DPI scales. | informal | |
| `SRC-01-99` Day, week, and month views (workweek and agenda may be included if inexpensive). | informal | |
| `SRC-02-34` I look at the calendar mostly in the week view, sometimes in a 3-day view on a smaller… | informal | |
| `SRC-02-35` Calendars should always be grouped by account, also when choosing a calendar while crea… | informal | |
| `SRC-02-36` Hidden calendars must not clutter quick lists — I have many calendars; turning them bac… | informal | |
| `SRC-02-37` Views in the MVP besides day, full week and month: 3 days (chosen by hand) and agenda;… | informal | |
| `SRC-02-38` Two levels of hiding: a quick toggle in the side panel hides only the events; disabling… | informal | |
| `SRC-02-40` Week start and date/time formats follow the system; the calendar is shown in the system… | informal | |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

<One requirement = one checkable capability of the system. Business language only — no file
paths, no class or module names, no endpoints; those belong in Technical notes. If a requirement
needs an "and…, and…, and…" to describe it, split it.

Provenance: a requirement that answers a source item needs nothing more — the table above is its
proof. A requirement with no source item carries a trailing marker: `— *source: DEC-02*` for a
recorded decision, `— *assumption: Q04*` for a default awaiting the client. Never bake an
unrecorded assumption into a requirement.>

- [MOD-03-F01] …
- [MOD-03-F02] … — **priority: should**
- [MOD-03-F03] … — *source: `DEC-02`*
- [MOD-03-F04] … — **priority: nice-to-have** — *assumption: `Q04`*

## UI/UX requirements

<Visual or interaction requirements, same rules as above.>

- [MOD-03-U01] …

## Non-functional requirements

<Only ones specific to this module. Project-wide ones live in NFR.md as NFR-nn.>

- [MOD-03-NFR-01] …

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
