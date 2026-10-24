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
| `MOD-03` Views and navigation | 24 / 5 | 9 | 7 | sliced |
| `MOD-04` Appearance and themes | 4 / 5 | 3 | 0 | sliced |
| `MOD-05` Creating and editing events | 29 / 4 | 10 | 0 | sliced |
| `MOD-06` Guests, invitations, RSVP and Google Meet | 22 / 4 | 8 | 0 | sliced |
| `MOD-07` Reminders and notifications | 20 / 3 | 6 | 0 | sliced |
| **Total** | **136 / 28** | **47** | **7** | 7 / 7 modules sliced |

A module counts as **sliced** once part 1 of its `COVERAGE.md` has no empty rows. `MOD-08` (deferred scope) holds no requirements and is not listed.

## [MOD-01] Google accounts and sign-in

`docs/specs/modules/01-accounts/REQUIREMENTS.md` · `docs/specs/modules/01-accounts/stories/`

| Story | Title | Covers | Prio | Status | Depends on | YouTrack |
| ----- | ----- | ------ | ---- | ------ | ---------- | -------- |
| `US-01-01` | As a User, I can connect my first Google account from the welcome screen, so that the app can show my calendars | F01, F03, F10, F12, F13, U03 | must | review | — | — |
| `US-01-02` | As a User, I can connect more Google accounts and see them all in the settings, so that my personal and work calendars are in one place | F02, U01, NFR-01 | must | review | `US-01-01` | — |
| `US-01-03` | As a User, I stay signed in and can sign in again when Google revokes access, so that I don't have to log in every time and a broken account doesn't stop the others | F04, F05, F06, F11 | must | review | `US-01-01` | — |
| `US-01-04` | As a User, I can remove an account, so that none of its data stays on my computer | F07, F08, F09, F14, U02 | must | review | `US-01-02` | — |

## [MOD-02] Calendars, synchronisation and offline cache

`docs/specs/modules/02-calendars-sync/REQUIREMENTS.md` · `docs/specs/modules/02-calendars-sync/stories/`

| Story | Title | Covers | Prio | Status | Depends on | YouTrack |
| ----- | ----- | ------ | ---- | ------ | ---------- | -------- |
| `US-02-01` | As a User, I can see the calendars of all my accounts together, in their Google colours, so that I have one calendar instead of several browser tabs | F01, F02, F03, F11, F22, NFR-02 | must | review | `US-01-01`, `US-03-01` | — |
| `US-02-02` | As a User, I can see read-only, subscribed and free/busy calendars without accidentally changing them, so that holidays and colleagues' calendars are there but safe | F19, F20, U01 | must | review | `US-02-01` | — |
| `US-02-03` | As a User, I see changes made on my phone or in the browser without doing anything, so that the app is always current | F07, F08, F09, F10, F23, NFR-01 | must | review | `US-02-01` | — |
| `US-02-04` | As a User, my list of calendars stays current, including calendars I lose access to or disable, so that nothing stale or unwanted is synchronized | F05, F06, F21 | must | review | `US-02-03` | — |
| `US-02-05` | As a User, I can open the app and browse my calendars without network, so that I can check my schedule anywhere | F12, F13, F14, U02, F15 | must | review | `US-02-01` | — |
| `US-02-06` | As a User, I can see whether my calendars are up to date and the app recovers from errors on its own, so that I can trust what I see | F16, F17, F18, U03 | must | review | `US-02-03` | — |
| `US-02-07` | As a User, I can give a calendar my own colour without changing it in Google, so that my calendars are easy to tell apart | F04, U04 | must | review | `US-02-01` | — |

## [MOD-03] Views and navigation

`docs/specs/modules/03-views-navigation/REQUIREMENTS.md` · `docs/specs/modules/03-views-navigation/stories/`

| Story | Title | Covers | Prio | Status | Depends on | YouTrack |
| ----- | ----- | ------ | ---- | ------ | ---------- | -------- |
| `US-03-01` | As a User, I can see my week as a clear, smoothly scrolling grid, so that I see at a glance what my week looks like | F02, F14, F19, F21, F22, U01, U02, U05, NFR-01 | must | done | — | — |
| `US-03-02` | As a User, I can look at a single day or three days, so that I can focus on today or use a smaller screen | F01, F04, F14, F22 | must | done | `US-03-01` | — |
| `US-03-03` | As a User, I can look at a whole month, so that I can plan further ahead | F03, F20, F14, F22 | must | done | `US-03-01` | — |
| `US-03-04` | As a User, I can see my upcoming events as a list, so that I know what is next without reading a grid | F05, F14, F22 | must | done | `US-03-01` | — |
| `US-03-05` | As a User, I can move between views and dates quickly, so that I get to any day in a moment | F06, F07, F08, F09, F15 | must | done | `US-03-01` | — |
| `US-03-06` | As a User, I can see my calendars grouped by account in a side panel and hide any of them quickly, so that I see only what matters now | F10, F11, F12, U03, MOD-02-U01 | must | review | `US-03-01`, `US-02-01` | — |
| `US-03-07` | As a User, I can disable calendars I never want to see, so that they do not clutter my lists | F13 | must | review | `US-03-06` | — |
| `US-03-08` | As a User, I can do everything from the keyboard with Google Calendar's shortcuts, so that I keep my muscle memory | F23, F24, U04 | must | done | `US-03-05` | — |
| `US-03-09` | As a User, I see dates, times and time zones the way my system is set, so that the calendar matches the rest of my desktop | F16, F17, F18 | must | done | `US-03-01` | — |

## [MOD-04] Appearance and themes

`docs/specs/modules/04-appearance-themes/REQUIREMENTS.md` · `docs/specs/modules/04-appearance-themes/stories/`

| Story | Title | Covers | Prio | Status | Depends on | YouTrack |
| ----- | ----- | ------ | ---- | ------ | ---------- | -------- |
| `US-04-01` | As a User, I can use the app in a light or dark theme that follows my system, so that it fits my desktop day and night | F01, F02, F03 | must | ready | — | — |
| `US-04-02` | As a User, I see a compact, readable calendar grid, so that my whole working day fits on screen | U01, U02, U04, NFR-01 | must | ready | `US-04-01`, `US-03-01` | — |
| `US-04-03` | As a User, I work in a calm, restrained English interface, so that my calendar colours are what stands out | F04, U03, U05 | must | ready | `US-04-01`, `US-03-01` | — |

## [MOD-05] Creating and editing events

`docs/specs/modules/05-events/REQUIREMENTS.md` · `docs/specs/modules/05-events/stories/`

| Story | Title | Covers | Prio | Status | Depends on | YouTrack |
| ----- | ----- | ------ | ---- | ------ | ---------- | -------- |
| `US-05-01` | As a User, I can create an event by clicking or selecting a time and typing a title, so that adding a meeting takes seconds | F01, F02, F09, F10, F16, U03 | must | review | `US-03-01`, `US-02-01` | — |
| `US-05-02` | As a User, I can set all the details of an event in a full editor, so that the event says everything it needs to | F03, F04, F05, F06, F07, F08, F11, F15, F16, U03 | must | review | `US-05-01` | — |
| `US-05-03` | As a User, I can drag an event to another time and stretch it to change its length, so that rescheduling is one gesture | F13, F14, F16, U02, U03 | must | review | `US-05-01` | — |
| `US-05-04` | As a User, I can delete an event and undo my last change, so that mistakes cost nothing | F12, F16, F27, F28, U04 | must | review | `US-05-01` | — |
| `US-05-05` | As a User, I am stopped from changing what I may not change and told why when Google refuses, so that I never break someone else's event | F17, F22 | must | review | `US-05-03` | — |
| `US-05-06` | As a User, I can make an event repeat daily, on weekdays, weekly, monthly or yearly, so that regular meetings are set once | F18, F19 | must | review | `US-05-02` | — |
| `US-05-07` | As a User, I can change or delete one occurrence, the following ones or the whole series, so that I handle exceptions the way Google does | F20, F21, F29 | must | review | `US-05-06`, `US-05-03` | — |
| `US-05-08` | As a User, I can see out-of-office, focus-time and similar events at a glance and create out-of-office and focus time, so that my availability is clear | F23, F24, U01 | must | review | `US-05-02` | — |
| `US-05-09` | As a User, I can see and change an event's own colour, so that it matches how I colour events in Google | F25 | must | review | `US-05-02` | — |
| `US-05-10` | As a User, I can open an event's attachments, so that the documents for a meeting are one click away | F26 | must | review | `US-05-02` | — |

## [MOD-06] Guests, invitations, RSVP and Google Meet

`docs/specs/modules/06-guests-rsvp-meet/REQUIREMENTS.md` · `docs/specs/modules/06-guests-rsvp-meet/stories/`

| Story | Title | Covers | Prio | Status | Depends on | YouTrack |
| ----- | ----- | ------ | ---- | ------ | ---------- | -------- |
| `US-06-01` | As a User, I can invite people to an event with suggestions from my contacts, recent guests and my organization, so that inviting a colleague takes a few keystrokes | F01, F02, F03, F04, F06, F07, F08 | must | review | `US-05-02` | — |
| `US-06-02` | As a User, I can see who is invited and how each guest answered, so that I know who is coming | F05, U01 | must | review | `US-02-01` | — |
| `US-06-03` | As a User, I am asked whether to notify guests when I change or delete an event, so that I decide when people get an e-mail | F12 | must | review | `US-06-01`, `US-05-03` | — |
| `US-06-04` | As a User, I can spot invitations waiting for my answer and list them, so that none slips through | F09, F10, F21, U04 | must | review | `US-02-01` | — |
| `US-06-05` | As a User, I can accept, decline or answer maybe — for one occurrence or a whole series — while guest rules are respected, so that organizers know whether I come | F11, F13, F15, U03 | must | review | `US-06-04` | — |
| `US-06-06` | As a User, I see declined events struck through and can hide them with one switch, so that my calendar stays readable | F14, F22 | must | review | `US-06-05` | — |
| `US-06-07` | As a User, I can add Google Meet to a meeting and join it with one click, so that video calls start on time | F16, F17, F18 | must | review | `US-06-01` | — |
| `US-06-08` | As a User, I can see when my guests are busy while picking a time, so that I avoid conflicts | F19, F20, U02 | must | review | `US-06-01` | — |

## [MOD-07] Reminders and notifications

`docs/specs/modules/07-notifications/REQUIREMENTS.md` · `docs/specs/modules/07-notifications/stories/`

| Story | Title | Covers | Prio | Status | Depends on | YouTrack |
| ----- | ----- | ------ | ---- | ------ | ---------- | -------- |
| `US-07-01` | As a User, I get a desktop notification before my events, as set in Google, so that I don't miss a meeting | F01, F02, F06, F07, F08, F13, F19, F20, U01 | must | review | `US-02-01` | — |
| `US-07-02` | As a User, I can snooze or dismiss a reminder and join a Meet call straight from it, so that I act on a reminder without opening the app | F09, F10 | must | review | `US-07-01`, `US-06-07` | — |
| `US-07-03` | As a User, I can turn notifications off or mute a single calendar, so that only reminders I care about reach me | F11, F12, U03 | must | review | `US-07-01`, `US-03-06` | — |
| `US-07-04` | As a User, I can set an event's reminders in the editor, starting from my calendar's defaults, so that my phone and computer remind me the same way | F03, F04, F05 | must | review | `US-05-02` | — |
| `US-07-05` | As a User, the app keeps reminding me from the tray when its window is closed and can start with my session, so that reminders don't depend on an open window | F14, F15, F16, F17, U02 | must | ready | `US-07-01` | — |
| `US-07-06` | As a User, I can open calendar subscription links from anywhere on my system, so that subscribing to a calendar works like in the browser | F18 | must | ready | `US-01-01` | — |
