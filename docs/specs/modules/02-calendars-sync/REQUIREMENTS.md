---
id: MOD-02
title: Calendars, synchronisation and offline cache
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [MOD-01] # other module IDs
sources: [SRC-01, SRC-02] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-02] Calendars, synchronisation and offline cache

## Context

Which calendars the app shows and how it keeps them current: every accessible calendar in one view, local colours, background synchronization within about a minute and on demand, the local cache from one year back to two years ahead, offline browsing, read-only and subscribed calendars, and what happens when a calendar loses access. Answers for the scope area "Calendars, synchronisation and offline cache"; comes from `DSC-04` (`discovery/04-calendars-sync.md`), drawing on `SRC-01` and `SRC-02`.

## Scope coverage

<Every source item this module answers for → the requirements that deliver it. Nothing from a
formal source may be silently dropped — a dropped or changed item points at the DEC-nn that
decided it. When the project has no sources register, list PROJECT.md "In scope" lines instead.>

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-11` Local-first: cache calendar data locally for fast access and offline browsing. | informal | |
| `SRC-01-25` Retrieve all accessible calendars from connected Google accounts. | informal | |
| `SRC-01-26` Show calendars from multiple accounts in unified views. | informal | |
| `SRC-01-27` Let users toggle calendar visibility and preserve provider-defined colors. | informal | |
| `SRC-01-28` Synchronize changes automatically and offer manual refresh. | informal | |
| `SRC-01-29` Use incremental synchronization (`syncToken`) where supported by the Google Calendar API. | informal | |
| `SRC-01-30` Cache events and relevant calendar metadata in a local SQLite database. | informal | |
| `SRC-01-31` Support offline browsing of previously synchronized events. | informal | |
| `SRC-01-32` Recover from connectivity loss, expired synchronization tokens, rate limits, and transi… | informal | |
| `SRC-01-33` Represent the synchronization state clearly without blocking the user interface. | informal | |
| `SRC-01-34` Handle time zones, daylight-saving changes, and recurring-event exceptions correctly. | informal | |
| `SRC-01-35` MVP offline boundary: offline browsing is included; offline creation/editing with a dur… | informal | |
| `SRC-01-98` Multiple calendars displayed in a unified interface. | informal | |
| `SRC-01-106` Local synchronization cache and offline browsing. | informal | |
| `SRC-01-121` See events from selected calendars across accounts. | informal | |
| `SRC-01-128` View cached events when temporarily offline. | informal | |
| `SRC-02-22` Colours should be settable locally: each calendar starts with its Google colour and can… | informal | |
| `SRC-02-28` The app uses only calendars that already exist in Google; creating, renaming and deleti… | informal | |
| `SRC-02-29` Read-only and subscribed calendars are shown like the others, without editing; free/bus… | informal | |
| `SRC-02-30` The cache keeps events from one year back to two years ahead, with no separate size lim… | informal | |
| `SRC-02-31` A calendar that loses access while the account works is removed with its cached events,… | informal | |
| `SRC-02-32` Offline, the MVP allows browsing only; creating, editing and RSVP are unavailable, with… | informal | |
| `SRC-02-33` While the app is open, changes made elsewhere appear within about a minute, and right a… | informal | |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

<One requirement = one checkable capability of the system. Business language only — no file
paths, no class or module names, no endpoints; those belong in Technical notes. If a requirement
needs an "and…, and…, and…" to describe it, split it.

Provenance: a requirement that answers a source item needs nothing more — the table above is its
proof. A requirement with no source item carries a trailing marker: `— *source: DEC-02*` for a
recorded decision, `— *assumption: Q04*` for a default awaiting the client. Never bake an
unrecorded assumption into a requirement.>

- [MOD-02-F01] …
- [MOD-02-F02] … — **priority: should**
- [MOD-02-F03] … — *source: `DEC-02`*
- [MOD-02-F04] … — **priority: nice-to-have** — *assumption: `Q04`*

## UI/UX requirements

<Visual or interaction requirements, same rules as above.>

- [MOD-02-U01] …

## Non-functional requirements

<Only ones specific to this module. Project-wide ones live in NFR.md as NFR-nn.>

- [MOD-02-NFR-01] …

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
