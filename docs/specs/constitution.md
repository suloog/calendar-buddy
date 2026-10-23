---
project: calendar-buddy
status: active # draft (written at discovery close) | active (confirmed by the tech lead at /og:modules)
---

# Engineering constitution

Per-project engineering law for calendar-buddy. Every module, user story and PR satisfies this document.

## Stack

- Language: Rust (stable). Versions are pinned by the tech lead at `/og:modules`.
- UI: egui (eframe on wgpu); Iced is the fallback if the feasibility check in the first stories fails (`DEC-33`).
- Package manager: cargo. Distribution: a .deb package for Ubuntu 26.10 (`DEC-12`).
- Local storage: SQLite cache (`DEC-23`); tokens in the platform secret store through one interface (`DEC-47`).
- Integrations: Google Calendar API and Google People API over HTTPS; no backend of our own (`DSC-01-04`).
- Boilerplate/base: none — no company boilerplate covers a Rust desktop app.

## Architecture principles

- The domain model (accounts, calendars, events, guests, recurrence, RSVP) does not depend on Google. The Google adapter sits behind a calendar-provider interface with capability flags; provider-specific features are optional (`NFR-14`).
- Platform specifics — secret store, notifications, tray, autostart, URL handlers, single instance — sit behind platform interfaces (`NFR-13`).
- The UI never waits on network or disk: Google calls, synchronization and SQLite run off the UI thread, and views read from the local store (`NFR-01`).
- The local store is the read model for every view, and only the sync engine writes synchronized data; a user's change goes to Google first and comes back through synchronization.
- Times are stored as UTC instants plus an IANA time zone; recurrence expansion and time-zone arithmetic go through a tested library, never hand-written code.
- The UI toolkit is isolated behind view models, so a fallback from egui to Iced (`DEC-33`) does not touch the domain, synchronization or adapters.
- Tokens and event contents never appear in logs or error messages (`NFR-07`).

## Quality bar

- Lint/format/type-check/tests must be green before every push (enforced by the og gate hook).
- Gates: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
- Test coverage threshold: 80% line coverage on the domain, synchronization and Google-adapter code; no threshold for the UI layer.
- Testing approach: test-first for recurrence, time zones and daylight-saving changes, incremental synchronization, and mapping to and from the Google APIs; unit tests first elsewhere, without a TDD mandate.
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
- Maciej (tech lead and product owner) approves every PR after the agent review.
- Changes to the SQLite schema or to the calendar-provider interface are called out explicitly in the PR description.

## Security & compliance

- Tokens are kept only in the platform's secret store (Secret Service on Linux), never in plaintext files; with no secret store running, sign-in stops with an explanation (`SRC-01-21`, `DEC-47`).
- The local cache is not encrypted by the app; the user's disk encryption covers it. No backups — Google is the system of record (`DEC-15`).
- Minimal OAuth scopes; the app is an External, unverified Google app in production status; verification before any public release (`DSC-02-05`, `DEC-10`).
- The only network traffic is HTTPS to Google's APIs; no telemetry, no remote crash reports (`DSC-02-13`, `DEC-13`).
- Logs are local and never contain tokens or event contents (`DSC-02-11`).
- Personal data (including guests' e-mail addresses) stays on the device and in Google; there is no processing by us (`DEC-16`).
- The GCP project belongs to the organization; Maciej owns the project, the OAuth client and Workspace administration (`DEC-09`).
