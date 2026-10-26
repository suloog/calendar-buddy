---
id: MOD-08
title: Deferred scope
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | done
depends_on: [] # other module IDs
sources: [SRC-01, SRC-02, SRC-03] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-08] Deferred scope

## Context

What came up and is deliberately not planned for delivery: the items parked during discovery, kept here so the other modules carry only planned work. Nothing in this module is built unless a later decision moves it into a delivery module. Corresponds to the "Out of scope (non-goals)" list of PROJECT.md and the *Parked* sections of the discovery topics.

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-37` Workweek view. | informal | not delivered — workweek view (`DEC-27`) |
| `SRC-01-109` CalDAV integration. | informal | not delivered — CalDAV, a later provider (PRD §13) |
| `SRC-01-110` Microsoft 365 / Outlook integration. | informal | not delivered — Microsoft 365 / Outlook, a later provider (PRD §13) |
| `SRC-01-111` Full offline editing with conflict resolution and queued writes. | informal | not delivered — offline editing with a change queue (`DEC-25`) |
| `SRC-01-113` Advanced calendar search. | informal | not delivered — event search (`DEC-30`) |
| `SRC-01-115` Natural-language event creation. | informal | not delivered — natural-language creation (PRD §13) |
| `SRC-01-116` Task management or AI scheduling assistant. | informal | not delivered — tasks, AI scheduling assistant (PRD §13) |
| `SRC-01-117` Advanced theming and layout customization. | informal | not delivered — advanced theming and layout (PRD §13, `DEC-57`) |
| `SRC-01-118` Calendar sharing and administrative management. | informal | not delivered — calendar management and sharing (`DEC-21`) |
| `SRC-01-119` Production-quality macOS and Windows releases if Linux is prioritized first. | informal | not delivered — macOS and Windows releases (`DEC-02`) |
| `SRC-02-27` Out of the MVP for accounts: providers other than Google; the user choosing permission… | informal | not delivered — non-Google sign-in, scopes chosen by hand |
| `SRC-02-39` No search in the MVP. | informal | not delivered — event search (`DEC-30`) |
| `SRC-03-01` Responding to an invitation with a comment and proposing another time are not needed —… | informal | not delivered — RSVP with a comment, proposing another time |

## Out of scope / what not to worry about

This module holds no requirements and nothing in it is planned for delivery. Each item above stays recorded here so it is not lost; it enters delivery only through a new `DEC-nn` that moves it into a delivery module (or a new module appended after `MOD-08`), followed by `/og:requirements` for that module.

Also deferred, recorded in the delivery modules' *Out of scope* sections rather than here because no source item asks for them on its own:

- Adding Google Drive attachments, ICS import and export — `DEC-37` (`MOD-05`).
- Recurrence rules beyond Google's custom recurrence dialog, creating working-location events — `DEC-165`, `DEC-34` (`MOD-05`).
- Suggesting free times for guests and dragging over other people's time — `DEC-42`, `DEC-173` (`MOD-06`).
- A second time zone, choosing the display time zone by hand — `DEC-29` (`MOD-03`).
- Interface translations — `DEC-31` (`MOD-04`).
- `mailto:` links, update checking and automatic updates — `DEC-46` (`MOD-07`).

Narrowed by a later decision: `SRC-01-118` (calendar management) and `SRC-02-27` (providers other than Google) stay deferred for Google calendars and other providers, but calendars kept only on the device — created, renamed and deleted in the app — are delivered by `MOD-09` (`DEC-178`).
