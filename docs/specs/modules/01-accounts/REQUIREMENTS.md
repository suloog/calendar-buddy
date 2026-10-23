---
id: MOD-01
title: Google accounts and sign-in
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [] # other module IDs
sources: [SRC-01, SRC-02] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-01] Google accounts and sign-in

## Context

Connecting and keeping Google accounts: signing in through the system browser, several personal and Workspace accounts at once, staying signed in and recovering from revoked access, removing an account together with its local data, and the first run. Answers for the PROJECT.md scope area "Google accounts and sign-in"; comes from discovery topic `DSC-03` (`discovery/03-accounts.md`), drawing on the PRD (`SRC-01`) and the analyst notes (`SRC-02`).

## Scope coverage

<Every source item this module answers for → the requirements that deliver it. Nothing from a
formal source may be silently dropped — a dropped or changed item points at the DEC-nn that
decided it. When the project has no sources register, list PROJECT.md "In scope" lines instead.>

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-17` Authenticate through Google OAuth 2.0 Authorization Code Flow with PKCE using the user'… | informal | |
| `SRC-01-18` Support multiple Google accounts simultaneously, including personal and Workspace accou… | informal | |
| `SRC-01-19` Request the minimum scopes needed for calendar operations and contact lookup. | informal | |
| `SRC-01-20` Refresh credentials when possible and handle revoked access gracefully. | informal | |
| `SRC-01-21` Store tokens in the operating system's credential store, not plaintext configuration fi… | informal | |
| `SRC-01-22` Allow users to add, remove, and reauthenticate accounts. | informal | |
| `SRC-01-23` Clearly distinguish account-level errors from calendar-level synchronization errors. | informal | |
| `SRC-01-97` Google sign-in and multiple account support. | informal | |
| `SRC-01-120` Connect at least one Google account without manually entering server settings. | informal | |
| `SRC-02-12` Supporting more than one account only needs the Google app switched to External and "In… | informal | |
| `SRC-02-24` First run with no account: a welcome screen with one action, connect a Google account;… | informal | |
| `SRC-02-25` Start without network: the calendar is shown from the cache with a visible "offline" st… | informal | |
| `SRC-02-26` Removing an account deletes at once its tokens, its cached calendars and events, and it… | informal | |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

<One requirement = one checkable capability of the system. Business language only — no file
paths, no class or module names, no endpoints; those belong in Technical notes. If a requirement
needs an "and…, and…, and…" to describe it, split it.

Provenance: a requirement that answers a source item needs nothing more — the table above is its
proof. A requirement with no source item carries a trailing marker: `— *source: DEC-02*` for a
recorded decision, `— *assumption: Q04*` for a default awaiting the client. Never bake an
unrecorded assumption into a requirement.>

- [MOD-01-F01] …
- [MOD-01-F02] … — **priority: should**
- [MOD-01-F03] … — *source: `DEC-02`*
- [MOD-01-F04] … — **priority: nice-to-have** — *assumption: `Q04`*

## UI/UX requirements

<Visual or interaction requirements, same rules as above.>

- [MOD-01-U01] …

## Non-functional requirements

<Only ones specific to this module. Project-wide ones live in NFR.md as NFR-nn.>

- [MOD-01-NFR-01] …

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
