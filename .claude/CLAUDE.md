# calendar-buddy

A calendar application. The repo is at the specification stage — no application code yet; scope is being defined in `docs/specs/`.

## Repo map

- `docs/specs/` — specification (sources, discovery, modules, stories, tests); see `docs/specs/README.md`
- `crates/core` (`calendar-core`) — domain model, week layout and view models; no UI toolkit, no Google
- `crates/google` (`calendar-google`) — the Google adapter: sign-in (OAuth with PKCE, loopback redirect); tests use a fake Google
- `crates/app` (`calendar-buddy`) — the egui/eframe desktop app; only draws view models and handles input

## Commands

- Run the app: `cargo run -p calendar-buddy` (the welcome screen first — connecting needs `~/.config/calendar-buddy/google-oauth-client.json`, `DEC-77`; then test data: ~20,000 generated events until sync exists)
- Tests: `cargo test --workspace`
- Gates (also run by the commit/push hook, `.claude/og-gates.json`): `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`

## Invariants

<!-- TODO: none yet. Per-area mechanics belong in .claude/rules/<topic>.md with `paths:` frontmatter, not here. -->

## Secrets

**`.env` files are off-limits — never read, write, copy or delete one** (`.env`, `.env.local`, `.env.*`, `*.env`), with any tool or shell command. A hook blocks it. The only exception is `.env.example` / `.env_example`: read it freely, and keep it current with **key names only, no values**. Need a new variable? Add the key to the example file and ask the user to set the value in their `.env`. A git-tracked `default.env` (committed defaults) opens up only while an `/og:deliver` loop runs with that option on.

## Engineering discipline

- **Think before coding.** Don't assume — surface uncertainties and competing interpretations; ask instead of proceeding confused.
- **Simplicity first.** Minimum code that solves the problem. No speculative abstraction, configurability, or defensive handling of impossible cases.
- **Surgical changes.** Every changed line traces to the task. Preserve existing style; report (don't silently delete) dead code you find.
- **Goal-driven execution.** Convert work into verifiable outcomes — failing test first, then make it pass; never "looks done".

## Workflow

Specs live in `docs/specs/`, written in the language named by `language:` in `PROJECT.md` — `SOURCES.md` + `sources/NN-<slug>.md` (what the client said, `SRC-NN-nn`, read-only), `discovery/NN-<topic>.md` (what we understood together, findings `DSC-NN-nn`, plus `DISCOVERY.md` and `AGENDA.md`), `PROJECT.md`, `NFR.md`, `modules/NN-<slug>/REQUIREMENTS.md` (numbered requirements `MOD-NN-Fnn`/`Unn`, each traced to a source item), `modules/NN-<slug>/stories/US-NN-MM-<slug>.md`, `modules/NN-<slug>/tests/TC-NN-MM-<slug>.md`, `modules/NN-<slug>/COVERAGE.md`, `BACKLOG.md`, `TESTPLAN.md`, `QUESTIONS.md` (open questions for the client) and `DECISIONS.md` (every decision — who, when, on what basis; append-only, never edit an entry). The user story is the unit of work: one story, one branch, one PR. There is no task layer — the breakdown into work happens in plan mode, against the code as it stands. Not sure where the project stands? `/og:guide`. Start every feature/bug with `/og:start-story <US-NN-MM | TICKET-ID | path>`; it asks up front where to branch from and whether to commit/PR automatically, then interviews you in plan mode before anything is written. `/og:commit` and `/og:pr` also stand alone. Ready stories can also be delivered autonomously: `/og:deliver` sets up the loop (`docs/delivery/`), `/loop /og:deliver-next` runs it. `og-status` prints module and story status. Gates (lint/type-check/tests) run automatically before every commit/push and must be green.
