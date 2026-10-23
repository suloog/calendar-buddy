---
id: MOD-07
title: Reminders and notifications
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [MOD-02] # other module IDs
sources: [SRC-01, SRC-02, SRC-03] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-07] Reminders and notifications

## Context

Being reminded: desktop notifications driven by the reminders set in Google, editing an event's reminders, snooze and dismiss, muting a calendar, running from the system tray with optional autostart, a single instance, and handling `webcal:` links. Answers for the scope area "Reminders and notifications"; comes from `DSC-09` (`discovery/09-notifications.md`), drawing on `SRC-01`, `SRC-02` and `SRC-03`. The Join button in notifications comes from `MOD-06`.

## Scope coverage

<Every source item this module answers for → the requirements that deliver it. Nothing from a
formal source may be silently dropped — a dropped or changed item points at the DEC-nn that
decided it. When the project has no sources register, list PROJECT.md "In scope" lines instead.>

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-80` Use operating-system notifications for upcoming events. | informal | |
| `SRC-01-81` Honor supported event reminders. | informal | |
| `SRC-01-82` Allow notifications to be enabled or disabled. | informal | |
| `SRC-01-83` Avoid duplicate notifications across synchronization cycles. | informal | |
| `SRC-01-84` Open the relevant event when a notification is activated, where supported by the OS. | informal | |
| `SRC-01-85` Handle events occurring while the application is closed according to documented platfor… | informal | |
| `SRC-01-105` System event reminders while the app is running. | informal | |
| `SRC-01-127` Receive relevant desktop reminders while the app is running. | informal | |
| `SRC-02-45` Reminders are important too, but they can be refined later. | informal | |
| `SRC-03-07` Default reminders for new events come only from Google (the calendar's defaults, change… | informal | |
| `SRC-03-08` Changing an event's reminders in the app is synchronized with Google; both forms — noti… | informal | |
| `SRC-03-09` Closing the window hides the app to the system tray, where it keeps syncing and remindi… | informal | |
| `SRC-03-10` Notifications have Dismiss and Snooze (5 / 10 min); notifications are switched globally… | informal | |
| `SRC-03-11` Single instance: yes. `webcal:` links: fine. `mailto:` links: probably not. No update c… | informal | |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

<One requirement = one checkable capability of the system. Business language only — no file
paths, no class or module names, no endpoints; those belong in Technical notes. If a requirement
needs an "and…, and…, and…" to describe it, split it.

Provenance: a requirement that answers a source item needs nothing more — the table above is its
proof. A requirement with no source item carries a trailing marker: `— *source: DEC-02*` for a
recorded decision, `— *assumption: Q04*` for a default awaiting the client. Never bake an
unrecorded assumption into a requirement.>

- [MOD-07-F01] …
- [MOD-07-F02] … — **priority: should**
- [MOD-07-F03] … — *source: `DEC-02`*
- [MOD-07-F04] … — **priority: nice-to-have** — *assumption: `Q04`*

## UI/UX requirements

<Visual or interaction requirements, same rules as above.>

- [MOD-07-U01] …

## Non-functional requirements

<Only ones specific to this module. Project-wide ones live in NFR.md as NFR-nn.>

- [MOD-07-NFR-01] …

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
