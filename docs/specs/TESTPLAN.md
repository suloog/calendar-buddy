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
| `MOD-02` Calendars, synchronisation and offline cache | 31 | 15 | 8 / 1 | written |
| `MOD-03` Views and navigation | 31 | 22 | 10 / 2 | written |
| `MOD-04` Appearance and themes | 10 | 0 | 0 / 0 | not written |
| `MOD-05` Creating and editing events | 34 | 0 | 0 / 0 | not written |
| `MOD-06` Guests, invitations, RSVP and Google Meet | 28 | 0 | 0 / 0 | not written |
| `MOD-07` Reminders and notifications | 24 | 0 | 0 / 0 | not written |
| **Total** | **177** | **45** | **20 / 5** | 3 / 7 modules written |

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

## [MOD-02] Calendars, synchronisation and offline cache

`docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` · `docs/specs/modules/02-calendars-sync/tests/`

| TC | Title | Verifies | Layer | Type | Automation | Status |
| -- | ----- | -------- | ----- | ---- | ---------- | ------ |
| `TC-02-01` | Calendars of two accounts, including a subscribed one, are fetched and shown together in their Google colours | `US-02-01` | desktop | happy-path | auto | draft |
| `TC-02-02` | Events in another time zone, across a daylight-saving change, and moved or cancelled occurrences show at the right local times | `US-02-01` | desktop | happy-path | auto | draft |
| `TC-02-03` | Events of a read-only calendar are marked not editable, and a free/busy-only calendar shows busy blocks without details | `US-02-02` | desktop | happy-path | auto | draft |
| `TC-02-04` | A change made in Google appears after the next background synchronization, and Refresh brings it at once | `US-02-03` | desktop | happy-path | auto | draft |
| `TC-02-05` | A change made on the phone appears in the open app within about a minute | `US-02-03` | desktop | happy-path | manual-only | draft |
| `TC-02-06` | A calendar that is no longer shared disappears with its events and a one-time notice | `US-02-04` | desktop | happy-path | auto | draft |
| `TC-02-07` | Without network the cached calendar is shown with an offline indicator and editing is unavailable | `US-02-05` | desktop | happy-path | auto | draft |
| `TC-02-08` | A synchronization error on one calendar is shown on it while the others stay up to date, with the last-sync time | `US-02-06` | desktop | happy-path | auto | draft |
| `TC-02-09` | A calendar gets a local colour that Google does not receive, and Reset restores the Google colour | `US-02-07` | desktop | happy-path | auto | draft |

## [MOD-03] Views and navigation

`docs/specs/modules/03-views-navigation/REQUIREMENTS.md` · `docs/specs/modules/03-views-navigation/tests/`

| TC | Title | Verifies | Layer | Type | Automation | Status |
| -- | ----- | -------- | ----- | ---- | ---------- | ------ |
| `TC-03-01` | The week view shows seven days of events, all-day events in a strip above the hours and free/busy calendars as busy blocks | `US-03-01` | desktop | happy-path | auto | draft |
| `TC-03-02` | Overlapping events are placed side by side without covering each other | `US-03-01` | desktop | happy-path | auto | draft |
| `TC-03-03` | The week view adapts to window resizing and to 100%, 150% and 200% scale | `US-03-01` | desktop | happy-path | manual-only | draft |
| `TC-03-04` | The interface-technology feasibility check is recorded and the NFR-03 thresholds are set from the first measurement | `US-03-01` | desktop | nfr | manual-only | draft |
| `TC-03-05` | The day and 3-day views show the selected day and three days | `US-03-02` | desktop | happy-path | auto | draft |
| `TC-03-06` | The month view shows each day's events and "+N more" for a full day, opening its list | `US-03-03` | desktop | happy-path | auto | draft |
| `TC-03-07` | The agenda lists upcoming events by day without empty days, and says when nothing is ahead | `US-03-04` | desktop | happy-path | auto | draft |
| `TC-03-08` | Switching views keeps the date, next/previous/today move correctly, and the app restarts in the last view on today | `US-03-05` | desktop | happy-path | auto | draft |
| `TC-03-09` | The side panel groups calendars by account and the quick toggle hides a calendar's events while keeping it listed | `US-03-06` | desktop | happy-path | auto | draft |
| `TC-03-10` | A disabled calendar is absent from the side panel, the calendar pickers and the views, and returns when enabled | `US-03-07` | desktop | happy-path | auto | draft |
| `TC-03-11` | Google Calendar's keys switch views, move between periods and show the shortcut list | `US-03-08` | desktop | happy-path | auto | draft |
| `TC-03-12` | A Sunday-first locale and a change of system time zone are followed by the views | `US-03-09` | desktop | happy-path | auto | draft |
