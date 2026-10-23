---
id: MOD-06
title: Guests, invitations, RSVP and Google Meet
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [MOD-05] # other module IDs
sources: [SRC-01, SRC-03] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-06] Guests, invitations, RSVP and Google Meet

## Context

Meetings with other people: adding guests with suggestions from contacts, the Workspace directory and recent addresses, responding to invitations, asking before guests are notified of a change, a simple free/busy preview of guests, and Google Meet with a one-click Join. Answers for the scope area "Guests, invitations, RSVP and Google Meet"; comes from `DSC-08` (`discovery/08-guests-rsvp-meet.md`), drawing on `SRC-01` and `SRC-03`.

## Scope coverage

<Every source item this module answers for → the requirements that deliver it. Nothing from a
formal source may be silently dropped — a dropped or changed item points at the DEC-nn that
decided it. When the project has no sources register, list PROJECT.md "In scope" lines instead.>

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-60` Add a Google Meet link through Google Calendar API where permitted. | informal | |
| `SRC-01-67` Add attendees using email addresses. | informal | |
| `SRC-01-68` Suggest people from Google Contacts through Google People API. | informal | |
| `SRC-01-69` Optionally include Google Workspace directory search when authorized. | informal | |
| `SRC-01-70` Offer suggestions from recently used attendee email addresses. | informal | |
| `SRC-01-71` Show attendee names, addresses, and invitation status. | informal | |
| `SRC-01-72` Support required and optional attendees to the extent exposed by Google Calendar API. | informal | |
| `SRC-01-73` Add or remove attendees when editing an event. | informal | |
| `SRC-01-74` Send appropriate event updates through Google Calendar API. | informal | |
| `SRC-01-75` Identify invitations awaiting a response. | informal | |
| `SRC-01-76` Support Accept, Decline, and Maybe responses. | informal | |
| `SRC-01-77` Display attendee response statuses. | informal | |
| `SRC-01-78` Synchronize RSVP changes with Google Calendar. | informal | |
| `SRC-01-79` Respect permissions and organizer/attendee distinctions. | informal | |
| `SRC-01-102` Add attendees with Google Contacts suggestions. | informal | |
| `SRC-01-103` Accept, decline, or tentatively accept invitations. | informal | |
| `SRC-01-104` Google Meet link creation where supported. | informal | |
| `SRC-01-112` Advanced attendee free/busy scheduling. | informal | |
| `SRC-01-122` Create a meeting and invite attendees using contact suggestions. | informal | |
| `SRC-01-123` Receive an invitation and respond Accept, Decline, or Maybe. | informal | |
| `SRC-01-126` Create or open a Google Meet link. | informal | |
| `SRC-03-02` When an event with guests is changed, the app asks every time whether to send an update… | informal | |
| `SRC-03-03` On Workspace accounts, guest suggestions also include the organization's directory. | informal | |
| `SRC-03-04` "Open a Meet link" means joining with one click: a clear Join button on the event, also… | informal | |
| `SRC-03-05` Invitations in a calendar disabled in the settings are not visible; disabling is the us… | informal | |
| `SRC-03-06` A simple free/busy preview of guests is wanted in the MVP: busy blocks per guest for th… | informal | |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

<One requirement = one checkable capability of the system. Business language only — no file
paths, no class or module names, no endpoints; those belong in Technical notes. If a requirement
needs an "and…, and…, and…" to describe it, split it.

Provenance: a requirement that answers a source item needs nothing more — the table above is its
proof. A requirement with no source item carries a trailing marker: `— *source: DEC-02*` for a
recorded decision, `— *assumption: Q04*` for a default awaiting the client. Never bake an
unrecorded assumption into a requirement.>

- [MOD-06-F01] …
- [MOD-06-F02] … — **priority: should**
- [MOD-06-F03] … — *source: `DEC-02`*
- [MOD-06-F04] … — **priority: nice-to-have** — *assumption: `Q04`*

## UI/UX requirements

<Visual or interaction requirements, same rules as above.>

- [MOD-06-U01] …

## Non-functional requirements

<Only ones specific to this module. Project-wide ones live in NFR.md as NFR-nn.>

- [MOD-06-NFR-01] …

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
