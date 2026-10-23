---
id: MOD-08
title: Deferred scope
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [] # other module IDs
sources: [SRC-01, SRC-02, SRC-03] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-08] Deferred scope

## Context

What came up and is deliberately not planned for delivery: the items parked during discovery, kept here so the other modules carry only planned work. Nothing in this module is built unless a later decision moves it into a delivery module. Corresponds to the "Out of scope (non-goals)" list of PROJECT.md and the *Parked* sections of the discovery topics.

## Scope coverage

<Every source item this module answers for → the requirements that deliver it. Nothing from a
formal source may be silently dropped — a dropped or changed item points at the DEC-nn that
decided it. When the project has no sources register, list PROJECT.md "In scope" lines instead.>

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-37` Workweek view. | informal | |
| `SRC-01-109` CalDAV integration. | informal | |
| `SRC-01-110` Microsoft 365 / Outlook integration. | informal | |
| `SRC-01-111` Full offline editing with conflict resolution and queued writes. | informal | |
| `SRC-01-113` Advanced calendar search. | informal | |
| `SRC-01-115` Natural-language event creation. | informal | |
| `SRC-01-116` Task management or AI scheduling assistant. | informal | |
| `SRC-01-117` Advanced theming and layout customization. | informal | |
| `SRC-01-118` Calendar sharing and administrative management. | informal | |
| `SRC-01-119` Production-quality macOS and Windows releases if Linux is prioritized first. | informal | |
| `SRC-02-27` Out of the MVP for accounts: providers other than Google; the user choosing permission… | informal | |
| `SRC-02-39` No search in the MVP. | informal | |
| `SRC-03-01` Responding to an invitation with a comment and proposing another time are not needed —… | informal | |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

<One requirement = one checkable capability of the system. Business language only — no file
paths, no class or module names, no endpoints; those belong in Technical notes. If a requirement
needs an "and…, and…, and…" to describe it, split it.

Provenance: a requirement that answers a source item needs nothing more — the table above is its
proof. A requirement with no source item carries a trailing marker: `— *source: DEC-02*` for a
recorded decision, `— *assumption: Q04*` for a default awaiting the client. Never bake an
unrecorded assumption into a requirement.>

- [MOD-08-F01] …
- [MOD-08-F02] … — **priority: should**
- [MOD-08-F03] … — *source: `DEC-02`*
- [MOD-08-F04] … — **priority: nice-to-have** — *assumption: `Q04`*

## UI/UX requirements

<Visual or interaction requirements, same rules as above.>

- [MOD-08-U01] …

## Non-functional requirements

<Only ones specific to this module. Project-wide ones live in NFR.md as NFR-nn.>

- [MOD-08-NFR-01] …

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
