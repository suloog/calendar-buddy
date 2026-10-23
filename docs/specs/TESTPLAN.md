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
| `MOD-04` Appearance and themes | 10 | 7 | 4 / 1 | written |
| `MOD-05` Creating and editing events | 34 | 18 | 10 / 2 | written |
| `MOD-06` Guests, invitations, RSVP and Google Meet | 28 | 0 | 0 / 0 | not written |
| `MOD-07` Reminders and notifications | 24 | 17 | 6 / 3 | written |
| **Total** | **177** | **87** | **40 / 11** | 6 / 7 modules written |

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

## [MOD-04] Appearance and themes

`docs/specs/modules/04-appearance-themes/REQUIREMENTS.md` · `docs/specs/modules/04-appearance-themes/tests/`

| TC | Title | Verifies | Layer | Type | Automation | Status |
| -- | ----- | -------- | ----- | ---- | ---------- | ------ |
| `TC-04-01` | With "System", the app starts dark on a dark desktop and turns light at once when the desktop switches | `US-04-01` | desktop | happy-path | auto | draft |
| `TC-04-02` | Choosing Light or Dark overrides the desktop and is kept after a restart | `US-04-01` | desktop | happy-path | auto | draft |
| `TC-04-03` | At 100% scale in a 1080-pixel-high window the week view shows 8:00–18:00 without scrolling | `US-04-02` | desktop | happy-path | auto | draft |
| `TC-04-04` | Event text on very light and very dark calendar colours reaches 4.5:1 contrast in both themes | `US-04-02` | desktop | nfr | auto | draft |
| `TC-04-05` | Apart from calendar colours the interface uses neutral tones and one accent, and changing views has no transition animation | `US-04-03` | desktop | happy-path | manual-only | draft |

## [MOD-05] Creating and editing events

`docs/specs/modules/05-events/REQUIREMENTS.md` · `docs/specs/modules/05-events/tests/`

| TC | Title | Verifies | Layer | Type | Automation | Status |
| -- | ----- | -------- | ----- | ---- | ---------- | ------ |
| `TC-05-01` | Selecting 10:00–11:30 and typing a title creates the event in the default calendar | `US-05-01` | desktop | happy-path | auto | draft |
| `TC-05-02` | The full editor saves title, description, times, location, busy/free and visibility to Google | `US-05-02` | desktop | happy-path | auto | draft |
| `TC-05-03` | Dragging an event moves it in 15-minute steps, keeps its length and saves the new time | `US-05-03` | desktop | happy-path | auto | draft |
| `TC-05-04` | Deleting a single event without guests needs no question and Undo restores it | `US-05-04` | desktop | happy-path | auto | draft |
| `TC-05-05` | Ctrl+Z after a drag restores the previous time, also in Google | `US-05-04` | desktop | happy-path | auto | draft |
| `TC-05-06` | An event of a read-only calendar cannot be dragged or edited and the app says why | `US-05-05` | desktop | negative | auto | draft |
| `TC-05-07` | A weekly series on Monday and Wednesday ending after 10 occurrences is created with the right rule | `US-05-06` | desktop | happy-path | auto | draft |
| `TC-05-08` | "This and following" from the 5th of 10 occurrences leaves 1–4 unchanged and moves 5–10 | `US-05-07` | desktop | happy-path | auto | draft |
| `TC-05-09` | Out-of-office can be created on a Workspace primary calendar and is not offered on a personal account | `US-05-08` | desktop | happy-path | auto | draft |
| `TC-05-10` | Changing an event's colour in the editor sends the new colour to Google | `US-05-09` | desktop | happy-path | auto | draft |
| `TC-05-11` | An event's attachments are listed and open in the browser | `US-05-10` | desktop | happy-path | manual-only | draft |
| `TC-05-12` | A series created in the app shows the same occurrences in Google Calendar in the browser | `US-05-06` | desktop | happy-path | manual-only | draft |

## [MOD-07] Reminders and notifications

`docs/specs/modules/07-notifications/REQUIREMENTS.md` · `docs/specs/modules/07-notifications/tests/`

| TC | Title | Verifies | Layer | Type | Automation | Status |
| -- | ----- | -------- | ----- | ---- | ---------- | ------ |
| `TC-07-01` | A 10-minute reminder produces exactly one notification with title, start time and location, even across synchronizations | `US-07-01` | desktop | happy-path | auto | draft |
| `TC-07-02` | Declined events and e-mail reminders produce no desktop notification | `US-07-01` | desktop | negative | auto | draft |
| `TC-07-03` | Snoozing for 5 minutes shows the notification again after 5 minutes without changing anything in Google | `US-07-02` | desktop | happy-path | auto | draft |
| `TC-07-04` | A muted calendar still shows its events but produces no notifications | `US-07-03` | desktop | happy-path | auto | draft |
| `TC-07-05` | A new event gets its calendar's default reminders, an added reminder reaches Google, and "Use calendar defaults" resets | `US-07-04` | desktop | happy-path | auto | draft |
| `TC-07-06` | Launching the app a second time brings the running window forward instead of starting another copy | `US-07-05` | desktop | happy-path | auto | draft |
| `TC-07-07` | On KDE, clicking a notification opens its event and Join opens the Meet meeting | `US-07-01` | desktop | happy-path | manual-only | draft |
| `TC-07-08` | With the window closed to the tray reminders keep arriving, quitting stops them, and "Start at login" starts the app in the tray | `US-07-05` | desktop | happy-path | manual-only | draft |
| `TC-07-09` | A `webcal:` link opened in another app opens Google Calendar's "add calendar from URL" page | `US-07-06` | desktop | happy-path | manual-only | draft |
