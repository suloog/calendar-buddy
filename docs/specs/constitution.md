---
project: calendar-buddy
status: draft # draft (written at discovery close) | active (confirmed by the tech lead at /og:modules)
---

# Engineering constitution

<Per-project engineering law. Every module, user story, and PR must satisfy this document. Keep it short; link to company/repo standards instead of restating them.>

## Stack

- Language: Rust (stable). Versions are pinned by the tech lead at `/og:modules`.
- UI: egui (eframe on wgpu); Iced is the fallback if the feasibility check in the first stories fails (`DEC-33`).
- Package manager: cargo. Distribution: a .deb package for Ubuntu 26.10 (`DEC-12`).
- Local storage: SQLite cache (`DEC-23`); tokens in the platform secret store through one interface (`DEC-47`).
- Integrations: Google Calendar API and Google People API over HTTPS; no backend of our own (`DSC-01-04`).
- Boilerplate/base: none — no company boilerplate covers a Rust desktop app.

## Architecture principles

<3–7 rules specific to this project, e.g. "all external calls go through services/", "no business logic in views/components".>

- …

## Quality bar

- Lint/format/type-check/tests must be green before every push (enforced by the og gate hook).
- Test coverage threshold: <e.g. 80% on changed code>
- Testing approach: <unit-first / TDD required / integration focus — per project tier>
- Required test levels: automated unit and integration tests for synchronization, recurrence, event editing and RSVP; acceptance is manual, by the product owner, against the PRD's §14; no load testing (`DEC-17`, `DEC-07`).

## Definition of done (per user story)

- [ ] Every acceptance criterion in the story spec verified, with evidence
- [ ] Every requirement in the story's `Covers` field actually delivered
- [ ] Tests added or updated for the behaviour the criteria describe
- [ ] Gates green (`/og:pre-push`)
- [ ] PR created with the standard template, linked to story spec + ticket
- [ ] Story spec `status` updated, and its row in `docs/specs/BACKLOG.md` with it

## Review rules

- Every PR reviewed by a human; agent review (code-reviewer + spec-compliance-reviewer) runs first.
- <Who must approve, e.g. tech lead for schema/API changes>

## Security & compliance

- Tokens are kept only in the platform's secret store (Secret Service on Linux), never in plaintext files; with no secret store running, sign-in stops with an explanation (`SRC-01-21`, `DEC-47`).
- The local cache is not encrypted by the app; the user's disk encryption covers it. No backups — Google is the system of record (`DEC-15`).
- Minimal OAuth scopes; the app is an External, unverified Google app in production status; verification before any public release (`DSC-02-05`, `DEC-10`).
- The only network traffic is HTTPS to Google's APIs; no telemetry, no remote crash reports (`DSC-02-13`, `DEC-13`).
- Logs are local and never contain tokens or event contents (`DSC-02-11`).
- Personal data (including guests' e-mail addresses) stays on the device and in Google; there is no processing by us (`DEC-16`).
- The GCP project belongs to the organization; Maciej owns the project, the OAuth client and Workspace administration (`DEC-09`).
