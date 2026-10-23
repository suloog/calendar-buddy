---
id: MOD-04
title: Appearance and themes
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [MOD-03] # other module IDs
sources: [SRC-01, SRC-02] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-04] Appearance and themes

## Context

How the app looks: the Morgen and Apple Calendar references, light and dark themes following the system, restrained use of colour, no unnecessary animation, and an English interface refined on the working app without mock-ups. Answers for the scope area "Visual direction and themes"; comes from `DSC-06` (`discovery/06-visual-themes.md`), drawing on `SRC-01` and `SRC-02`.

## Scope coverage

<Every source item this module answers for → the requirements that deliver it. Nothing from a
formal source may be silently dropped — a dropped or changed item points at the DEC-nn that
decided it. When the project has no sources register, list PROJECT.md "In scope" lines instead.>

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-02` The experience should combine the visual clarity of Apple Calendar with the modern layo… | informal | |
| `SRC-01-49` Inspired by Morgen for layout density, clear scheduling grids, and modern aesthetics. | informal | |
| `SRC-01-50` Inspired by Apple Calendar for simplicity and intuitive interactions. | informal | |
| `SRC-01-51` Provide light and dark themes, optionally following the system setting. | informal | |
| `SRC-01-52` Preserve useful calendar color coding while keeping the interface restrained. | informal | |
| `SRC-01-53` Avoid unnecessary animations and unrelated productivity features in the MVP. | informal | |
| `SRC-01-107` Light/dark themes and keyboard shortcuts. | informal | |
| `SRC-02-41` The interface is in English only in the MVP; texts kept in one place so translations ca… | informal | |
| `SRC-02-42` No mock-ups before coding; Morgen and Apple Calendar are the references and the look is… | informal | |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

<One requirement = one checkable capability of the system. Business language only — no file
paths, no class or module names, no endpoints; those belong in Technical notes. If a requirement
needs an "and…, and…, and…" to describe it, split it.

Provenance: a requirement that answers a source item needs nothing more — the table above is its
proof. A requirement with no source item carries a trailing marker: `— *source: DEC-02*` for a
recorded decision, `— *assumption: Q04*` for a default awaiting the client. Never bake an
unrecorded assumption into a requirement.>

- [MOD-04-F01] …
- [MOD-04-F02] … — **priority: should**
- [MOD-04-F03] … — *source: `DEC-02`*
- [MOD-04-F04] … — **priority: nice-to-have** — *assumption: `Q04`*

## UI/UX requirements

<Visual or interaction requirements, same rules as above.>

- [MOD-04-U01] …

## Non-functional requirements

<Only ones specific to this module. Project-wide ones live in NFR.md as NFR-nn.>

- [MOD-04-NFR-01] …

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
