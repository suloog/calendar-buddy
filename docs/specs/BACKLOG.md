<!-- Copy to docs/specs/BACKLOG.md. Content in the project language; headings fixed. -->

# Backlog — user story register

> Every user story in the project, in one view. The story file
> `docs/specs/modules/NN-<slug>/stories/US-NN-MM-<slug>.md` is the source of truth for content; this file
> **mirrors** it and is updated in the same run that changes a story.
>
> The work itself runs in YouTrack. The `YouTrack` column ties a row to its issue.

## Legend

| Status | Means |
| ------ | ----- |
| `draft` | written, not yet gated by `/og:clarify` |
| `ready` | gated — free to pick up |
| `in-progress` | being implemented |
| `review` | implemented, in review / testing |
| `done` | accepted against its acceptance criteria |
| `blocked` | held — a dependency or an unanswered question |

**Priority:** `must` (default, blocking) · `should` · `nice-to-have`.

**IDs:** `US-NN-MM`, where `NN` is the parent module (`MOD-NN`) and `MM` a counter inside that
module. Numbers are never reused or renumbered — a new story is appended to the series.

**Covers:** the requirements this story delivers (the story's `covers:` frontmatter). The full map
the other way round (requirement → stories) lives in the module's `COVERAGE.md`.

## Progress

| Module | Requirements (F/U) | Stories | done | Slicing |
| ------ | ------------------ | ------- | ---- | ------- |
| `MOD-01` Google accounts and sign-in | 14 / 3 | 4 | 0 | sliced |
| `MOD-02` Calendars, synchronisation and offline cache | 23 / 4 | 7 | 0 | sliced |
| `MOD-03` Views and navigation | 24 / 5 | 0 | 0 | not sliced |
| `MOD-04` Appearance and themes | 4 / 5 | 0 | 0 | not sliced |
| `MOD-05` Creating and editing events | 29 / 4 | 0 | 0 | not sliced |
| `MOD-06` Guests, invitations, RSVP and Google Meet | 22 / 4 | 0 | 0 | not sliced |
| `MOD-07` Reminders and notifications | 20 / 3 | 0 | 0 | not sliced |
| **Total** | **136 / 28** | **11** | **0** | 2 / 7 modules sliced |

A module counts as **sliced** once part 1 of its `COVERAGE.md` has no empty rows. `MOD-08` (deferred scope) holds no requirements and is not listed.

## [MOD-01] Google accounts and sign-in

`docs/specs/modules/01-accounts/REQUIREMENTS.md` · `docs/specs/modules/01-accounts/stories/`

| Story | Title | Covers | Prio | Status | Depends on | YouTrack |
| ----- | ----- | ------ | ---- | ------ | ---------- | -------- |
| `US-01-01` | As a User, I can connect my first Google account from the welcome screen, so that the app can show my calendars | F01, F03, F10, F12, F13, U03 | must | draft | — | — |
| `US-01-02` | As a User, I can connect more Google accounts and see them all in the settings, so that my personal and work calendars are in one place | F02, U01, NFR-01 | must | draft | `US-01-01` | — |
| `US-01-03` | As a User, I stay signed in and can sign in again when Google revokes access, so that I don't have to log in every time and a broken account doesn't stop the others | F04, F05, F06, F11 | must | draft | `US-01-01` | — |
| `US-01-04` | As a User, I can remove an account, so that none of its data stays on my computer | F07, F08, F09, F14, U02 | must | draft | `US-01-02` | — |

## [MOD-02] Calendars, synchronisation and offline cache

`docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` · `docs/specs/modules/02-calendars-sync/stories/`

| Story | Title | Covers | Prio | Status | Depends on | YouTrack |
| ----- | ----- | ------ | ---- | ------ | ---------- | -------- |
| `US-02-01` | As a User, I can see the calendars of all my accounts together, in their Google colours, so that I have one calendar instead of several browser tabs | F01, F02, F03, F11, F22, NFR-02 | must | draft | `US-01-01` | — |
| `US-02-02` | As a User, I can see read-only, subscribed and free/busy calendars without accidentally changing them, so that holidays and colleagues' calendars are there but safe | F19, F20, U01 | must | draft | `US-02-01` | — |
| `US-02-03` | As a User, I see changes made on my phone or in the browser without doing anything, so that the app is always current | F07, F08, F09, F10, F23, NFR-01 | must | draft | `US-02-01` | — |
| `US-02-04` | As a User, my list of calendars stays current, including calendars I lose access to or disable, so that nothing stale or unwanted is synchronized | F05, F06, F21 | must | draft | `US-02-03` | — |
| `US-02-05` | As a User, I can open the app and browse my calendars without network, so that I can check my schedule anywhere | F12, F13, F14, U02, F15 | must | draft | `US-02-01` | — |
| `US-02-06` | As a User, I can see whether my calendars are up to date and the app recovers from errors on its own, so that I can trust what I see | F16, F17, F18, U03 | must | draft | `US-02-03` | — |
| `US-02-07` | As a User, I can give a calendar my own colour without changing it in Google, so that my calendars are easy to tell apart | F04, U04 | must | draft | `US-02-01` | — |
