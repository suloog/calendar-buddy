<!-- Copy to docs/specs/TESTPLAN.md. Content in the project language; headings fixed. -->

# Test plan — test case register

> Every test in the project in one view. The test case file
> `docs/specs/modules/NN-<slug>/tests/TC-NN-MM-<slug>.md` is the source of truth for content; this file is
> its **mirror** and changes in the same pass.
>
> 🔴 **This register carries no results.** There is no "last result" column and no last-run date —
> results live in `run.json` and in that run's HTML report. The test plan says what must be
> checked; the report says what happened. Mixing the two gives a register that lies a week after
> the last run.

## Legend

| Status | Meaning |
| ------ | ------- |
| `draft` | written, not yet reviewed |
| `ready` | reviewed — ready to run |

| Layer | Executed by |
| ----- | ----------- |
| `admin` | Django panel, through Playwright |
| `web` | web application, through Playwright |
| `api` | HTTP, no browser |
| `mobile` | mobile app, through Maestro (+ the Metro bridge for app state) |
| `desktop` | this project's layer (`DEC-65`): `auto` in the project's own `cargo test` suite against a fake Google; `manual-only` checked by the product owner |

| Type | Meaning |
| ---- | ------- |
| `happy-path` | the main path, everything works in order |
| `edge` | a boundary case from the module's "Constraints and edge cases" |
| `negative` | the system must **refuse** — validation, no permission, conflict |
| `nfr` | a measurable non-functional requirement |

**Automation:** `auto` (run by the executor) · `manual-only` (out of the executor's reach — never
green in the report, goes onto the manual-check list).

**Identifiers:** `TC-NN-MM`, where `NN` is the parent module (`MOD-NN`) and `MM` a counter within
that module. Numbers are never reused or renumbered.

## Progress

| Module | Story criteria | Covered | TC (auto / manual) | Written |
| ------ | -------------- | ------- | ------------------ | ------- |
| `MOD-01` Google accounts and sign-in | 19 | 8 | 2 / 2 | written |
| `MOD-02` Calendars, synchronisation and offline cache | 31 | 0 | 0 / 0 | not written |
| `MOD-03` Views and navigation | 31 | 0 | 0 / 0 | not written |
| `MOD-04` Appearance and themes | 10 | 0 | 0 / 0 | not written |
| `MOD-05` Creating and editing events | 34 | 0 | 0 / 0 | not written |
| `MOD-06` Guests, invitations, RSVP and Google Meet | 28 | 0 | 0 / 0 | not written |
| `MOD-07` Reminders and notifications | 24 | 0 | 0 / 0 | not written |
| **Total** | **177** | **8** | **2 / 2** | 1 / 7 modules written |

A module counts as **written** when part 2 of its `COVERAGE.md` has no row without either a test or a note. Depth so far: smoke (`DEC-65`) — criteria without a test carry the note "smoke depth — not tested yet".

## Environments

Desktop app, no `.claude/og-test-targets.json` and no `/og:run-tests` (`DEC-65`).

| Environment | Used for | Data |
| ----------- | -------- | ---- |
| `local` (cargo test) | `auto` test cases, before every PR | a fake Google API and a test secret store, set up by the tests |
| `local` (product owner's desktop) | `manual-only` test cases, acceptance | the product owner's real Google accounts, Ubuntu 26.10 + KDE Plasma (`DEC-12`) |

## [MOD-01] Google accounts and sign-in

`docs/specs/modules/01-accounts/REQUIREMENTS.md` · `docs/specs/modules/01-accounts/tests/`

| TC | Title | Verifies | Layer | Type | Automation | Status |
| -- | ----- | -------- | ----- | ---- | ---------- | ------ |
| `TC-01-01` | A first Google account is connected from the welcome screen and its calendar appears | `US-01-01` | desktop | happy-path | manual-only | draft |
| `TC-01-02` | A second account is connected from the settings and both are listed with their state | `US-01-02` | desktop | happy-path | manual-only | draft |
| `TC-01-03` | Connected accounts stay connected after a restart without opening the browser | `US-01-03` | desktop | happy-path | auto | draft |
| `TC-01-04` | Removing an account deletes all its local data and withdraws the app's access | `US-01-04` | desktop | happy-path | auto | draft |
