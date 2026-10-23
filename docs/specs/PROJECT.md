---
project: calendar-buddy
client: internal
language: en # language of every spec artifact (en | pl) — chosen once, at /og:analyze
status: draft # draft | approved | active | done
created: 2026-10-22
analyst: Maciej
---

<!-- Content in the project language. Section headings are fixed by this template — do not
     rename, reorder or add sections; the same structure is used in every Order Group project. -->

# calendar-buddy

## Overview

<2–4 sentences: what is being built, for whom, and why now. No tech stack here.>

## Sources

The scope comes from the register in `SOURCES.md`.

- Binding: none yet
- Informative: `SRC-01` Desktop Calendar — Product Requirements Document, v0.1 (draft PRD, 129 items)

## Goals

<Measurable outcomes, not features. Each goal should be verifiable at project end. Cite the
source item where one exists.>

- G1: … (`SRC-01-03`)
- G2: …

## Actors

<The canonical roles. Every user story's "As a …" must name one of these, spelled exactly as
here — this table is what stops the same person being called three things in three modules.>

| Actor | Who they are | How they sign in | Access, in short | Source |
| ----- | ------------ | ---------------- | ---------------- | ------ |
| …     | …            | …                | …                | `SRC-01-05` |

## Glossary

<Terms that carry one fixed meaning across every module, so modules can use the word instead of
repeating the definition. Include the collective terms the requirements will lean on.>

| Term | Means exactly |
| ---- | ------------- |
| …    | …             |

## Scope

### In scope

<The areas of functionality this project delivers. One line each, with the source items behind
it — the detail is enumerated per module by /og:requirements, and each module's "Scope
coverage" table maps the source items to requirements. An area may also point at the discovery
topic it comes from (`discovery/NN-<topic>.md`).>

- … (`discovery/02-bookings.md` · `SRC-01-10`…`SRC-01-14`)

### Out of scope (non-goals)

<Explicit non-goals prevent scope creep. Be concrete enough to reject a real feature request.
A non-goal that contradicts a binding source needs a DEC-nn.>

- …

## Stack & integrations

<Chosen stack and external systems, and which of them owns the business logic. Details and
standards live in constitution.md.>

## Milestones

| Milestone | Contains | Target |
| --------- | -------- | ------ |
| M1        | …        | …      |

## Risks

| Risk | Impact | Mitigation |
| ---- | ------ | ---------- |
| …    | …      | …          |

<Requirements are NOT listed here. They are enumerated per module in
docs/specs/modules/NN-<slug>/REQUIREMENTS.md (MOD-NN-Fnn / Unn) and, project-wide, in
docs/specs/NFR.md (NFR-nn). Source documents are registered in
docs/specs/SOURCES.md, open questions in docs/specs/QUESTIONS.md, decisions in
docs/specs/DECISIONS.md.>
