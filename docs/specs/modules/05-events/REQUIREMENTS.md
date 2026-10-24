---
id: MOD-05
title: Creating and editing events
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | done
depends_on: [MOD-02, MOD-03] # other module IDs
sources: [SRC-01, SRC-02, SRC-04] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-05] Creating and editing events

## Context

Creating, changing and deleting events: quick entry and the full editor, the default calendar, dragging and resizing, moving between calendars, recurrence presets with three edit scopes, Google's special event types, event colours and attachments shown as links. Answers for the scope area "Creating and editing events"; comes from `DSC-07` (`discovery/07-events.md`), drawing on `SRC-01` and `SRC-02`. Holds the recurrence risk.

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-54` Create an event by clicking or selecting a time range. | informal | `MOD-05-F01` |
| `SRC-01-55` Offer quick entry plus a full event editor. | informal | `MOD-05-F02`, `MOD-05-F03`, `MOD-05-F04` |
| `SRC-01-56` Set title, description, start/end, time zone, and calendar. | informal | `MOD-05-F04` |
| `SRC-01-57` Support all-day and multi-day events. | informal | `MOD-05-F05` |
| `SRC-01-58` Set location, reminders, availability (busy/free), and visibility where supported. | informal | `MOD-05-F06`, `MOD-05-F07`, `MOD-05-F08`; reminders in `MOD-07` |
| `SRC-01-59` Create recurring events. | informal | `MOD-05-F18` |
| `SRC-01-61` Edit and delete events subject to user permissions. | informal | `MOD-05-F11`, `MOD-05-F12`, `MOD-05-F17` |
| `SRC-01-62` Drag events to a different time/date and resize their duration. | informal | `MOD-05-F13`, `MOD-05-F14` |
| `SRC-01-63` Move events between calendars where the provider permits it. | informal | `MOD-05-F15` |
| `SRC-01-64` Handle recurring event instances and entire series; support changes to future occurrenc… | informal | `MOD-05-F20`, `MOD-05-F21` |
| `SRC-01-65` Warn users when changes affect invited participants or an entire series. | informal | `MOD-05-F20`; guests in `MOD-06-F12` |
| `SRC-01-66` Display clear failures if an operation is disallowed by Google or the organizer's permi… | informal | `MOD-05-F22` |
| `SRC-01-100` Create, edit, delete, and reschedule events. | informal | `MOD-05-F11`, `MOD-05-F12`, `MOD-05-F13`, `MOD-05-F16` |
| `SRC-01-101` Basic recurring event management. | informal | `MOD-05-F18`, `MOD-05-F19` |
| `SRC-01-114` Google Drive attachments. | informal | `MOD-05-F26`; adding attachments not built (`DEC-37`, `MOD-08`) |
| `SRC-01-124` Reschedule, edit, and cancel events within their permissions. | informal | `MOD-05-F11`, `MOD-05-F12`, `MOD-05-F13`, `MOD-05-F16` |
| `SRC-01-125` Manage common recurring event cases. | informal | `MOD-05-F18`…`MOD-05-F21` |
| `SRC-02-23` The user picks which calendar is the default for creating new events. | informal | `MOD-05-F10` |
| `SRC-02-44` It is important to be able to choose the event type where possible, e.g. out of office,… | informal | `MOD-05-F23`, `MOD-05-F24` |
| `SRC-02-46` Event types: out of office and focus time can be created where the account allows; work… | informal | `MOD-05-F23`, `MOD-05-F24` |
| `SRC-02-47` Event colours from Google are shown and can be changed in the editor. | informal | `MOD-05-F25` |
| `SRC-02-48` Recurrence: presets daily, weekdays, weekly on chosen days, monthly, yearly; end by dat… | informal | `MOD-05-F18`…`MOD-05-F21` |
| `SRC-04-01` Two features are missing; the first is being able to configure how an event repeats. | informal | `MOD-05-F32`…`MOD-05-F35` |
| `SRC-04-02` Configuring repetition goes as far as Google Calendar's "Custom" recurrence: every N d… | informal | `MOD-05-F32`…`MOD-05-F35`, `MOD-05-U05` |
| `SRC-02-49` Existing attachments shown as links; no ICS import or export in the MVP. | informal | `MOD-05-F26`; no ICS import or export (`DEC-37`) |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

### Creating

- [MOD-05-F01] The user can start a new event by clicking a time in the calendar or by selecting a time range.
- [MOD-05-F02] Starting a new event opens a quick-entry box with the title, the time already filled in and the default calendar; Enter saves the event. — *source: `DEC-59`*
- [MOD-05-F03] From the quick-entry box the user can open the full editor, which keeps what was already typed. — *source: `DEC-59`*
- [MOD-05-F04] In the full editor the user sets the title, description, start, end, time zone and calendar.
- [MOD-05-F05] An event can be all-day, and it can span several days.
- [MOD-05-F06] The user can set the event's location.
- [MOD-05-F07] The user can set whether the event shows the user as busy or free.
- [MOD-05-F08] The user can set the event's visibility (default, public, private) where Google supports it.
- [MOD-05-F09] A new event started without a range gets the default length from the user's Google Calendar settings. — *source: `DEC-59`*
- [MOD-05-F10] A new event goes to the default calendar the user chose in the settings; another calendar can be picked from the list grouped by account. Until the user chooses, the default is the primary calendar of the first connected account. — *source: `DEC-60`*

### Changing

- [MOD-05-F11] The user can change any field of an event they are allowed to edit.
- [MOD-05-F12] The user can delete an event they are allowed to delete.
- [MOD-05-F13] The user can drag an event to another time or day; its length stays the same.
- [MOD-05-F14] The user can drag the end of an event to change its length.
- [MOD-05-F15] The user can move an event to another calendar of the same account, where Google allows it.
- [MOD-05-F16] Every change and deletion is saved to Google.
- [MOD-05-F17] Events the user is not allowed to change cannot be dragged, resized, edited or deleted, and the app says why when the user tries. — *source: `DEC-22`*

### Recurring events

- [MOD-05-F18] The user can make an event recurring with one of these patterns: daily, every weekday, weekly on chosen days, monthly, yearly; since `DEC-165` they are offered through Google's repeat menu and "Custom…" (`MOD-05-F32`, `F33`). — *source: `DEC-36`, `DEC-165`*
- [MOD-05-F19] A recurring event ends on a chosen date, after a chosen number of occurrences, or never. — *source: `DEC-36`*
- [MOD-05-F20] Changing an occurrence of a series — in the editor or by dragging or resizing — asks whether the change applies to this event, this and following events, or all events. — *source: `DEC-36`*
- [MOD-05-F21] Deleting an occurrence of a series asks the same three-way question. — *source: `DEC-36`*
- [MOD-05-F22] When Google or the organizer does not allow a change, the app shows a clear message and the event returns to how it was.

- [MOD-05-F32] The editor's repeat menu is Google Calendar's: "Does not repeat", "Daily", "Weekly on <weekday>", "Monthly on the <nth> <weekday>", "Annually on <date>", "Every weekday (Monday to Friday)" and "Custom…", the labels following the event's start date; a start on the fifth weekday of its month gives "Monthly on the last <weekday>". — *source: `DEC-165`, `DEC-167`*
- [MOD-05-F33] "Custom…" opens the custom recurrence dialog: repeat every N days, weeks, months or years; for weeks, on the chosen weekdays; for months, on the same day of the month or on the same weekday of the same week of the month, and on its last occurrence in the month when the start falls in the month's last seven days — for the fifth weekday of the month only the last, never "the fifth"; ending never, on a date, or after N occurrences. — *source: `DEC-165`, `DEC-167`*
- [MOD-05-F34] A series whose rule the dialog can express — made in the app or in Google — shows the rule in the repeat menu in words ("Every 2 weeks on Monday, Wednesday") and can be changed in the app; reopening "Custom…" shows the dialog filled with that rule. — *source: `DEC-165`*
- [MOD-05-F35] Changing the rule of an occurrence of a series asks whether it applies to this and following events or to all events. — *source: `DEC-165`*

### Types, colours and attachments

- [MOD-05-F23] Out-of-office, focus-time, working-location and birthday events are shown so they stand out from ordinary events. — *source: `DEC-34`*
- [MOD-05-F24] Where the account allows it (the primary calendar of a Workspace account), the user can create out-of-office and focus-time events; elsewhere these types are not offered. — *source: `DEC-34`*
- [MOD-05-F25] An event's own Google colour is shown, and the user can change it in the editor. — *source: `DEC-35`*
- [MOD-05-F26] An event's existing attachments are listed in its details as links that open in the browser. — *source: `DEC-37`*

### Undo

- [MOD-05-F27] An event without guests that is not part of a series is deleted without a question, and an Undo is offered for a few seconds. — *source: `DEC-59`*
- [MOD-05-F28] The user can undo the last change to an event with Ctrl+Z or with the Undo in the notice that follows the change. — *source: `DEC-59`*
- [MOD-05-F29] A series created in Google with a rule the custom recurrence dialog (`MOD-05-F33`) cannot express keeps its rule when its other fields are edited; to change the rule, the app offers to open the event in Google Calendar in the browser. — *source: `DEC-36`, `DEC-60`, `DEC-165`*

### Details

- [MOD-05-F30] An event's details show its location and its description, as set in the app or in Google; a description written in Google with formatting is shown with its bold, italics, underlines, lists and links, and a link opens in the browser. — *source: `DEC-156`, `DEC-157`*
- [MOD-05-F31] In the editor, a description with formatting from Google is shown formatted and saved unchanged; when the user starts to edit it, the app says that editing removes the formatting and turns it into plain text only if the user agrees. — *source: `DEC-157`*

## UI/UX requirements

- [MOD-05-U01] Out-of-office events are hatched, focus-time events carry an icon, working-location and birthday events are shown as labels in the all-day strip. — *source: `DEC-60`*
- [MOD-05-U02] While an event is dragged or resized, it moves in 15-minute steps and shows the new time. — *source: `DEC-60`*
- [MOD-05-U03] After a change, the event stays at its new place marked as "saving" until Google confirms it. — *source: `DEC-60`*
- [MOD-05-U04] The Undo notice stays visible for 5 seconds. — *source: `DEC-60`*
- [MOD-05-U05] The custom recurrence dialog shows only the fields of the chosen unit — the weekdays for weeks, the day-or-weekday choice for months — with the weekday of the event's start chosen when switching to weeks. — *source: `DEC-165`*

## Constraints and edge cases

- The end is set before the start in the editor → the event cannot be saved, and the editor says why.
- An all-day event is dragged into the hours → it becomes a timed event of the default length; a timed event dragged into the all-day strip becomes all-day. — *source: `DEC-60`*
- The event was changed elsewhere since it was opened → the app reloads it, says so, and the user applies the change again. — *source: `DEC-60`*
- Undo after the event was changed elsewhere → the undo is refused with a message.
- "This and following" on the first occurrence → treated as "all events".
- Weekly with no weekday chosen → the dialog does not allow it; the event's own weekday stays chosen. — *source: `DEC-165`*
- Monthly on day 29–31 → months without that day are skipped, as in Google. — *source: `DEC-165`*
- "Ends on" a date before the start, or "after" 0 occurrences → the dialog does not accept it and says why.
- The event's start date changes after a rule was chosen → the weekday- and day-based rules follow the new start, as in Google. — *source: `DEC-165`*
- Moving an event to a calendar of another account → not offered (Google moves events only within one account).
- No network → creating and changing are unavailable (`MOD-02-F15`).
- An event stored in another time zone → edited in its own time zone, shown in the system's (`MOD-03-F18`).

## Integrations

- **Google Calendar API — events** (write): create, update, delete, move between calendars, and occurrences of a series.
- **Google Calendar API — settings** (read): the user's default event length.
- **Google Calendar API — colours**: the event colour IDs (`colorId` 1–11); the palette itself is built in, as Google Calendar shows it (`DEC-118`).

## Technical notes

- Every change goes to Google first and reaches the local store through the sync engine (constitution); the "saving" mark (`MOD-05-U03`) is view state only, rolled back on refusal.
- Updates are sent as patches with the event's ETag; a 412 answer triggers the reload of the conflict edge case.
- Series scopes: "this event" changes one instance (an exception); "all" updates the master; "this and following" splits the series — the original master gets an `UNTIL` before the chosen occurrence and a new series starts from it. Test-first (constitution).
- Presets map to RRULEs: `FREQ=DAILY`; `FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR`; `FREQ=WEEKLY;BYDAY=…`; `FREQ=MONTHLY` (same day of month); `FREQ=YEARLY`; with `UNTIL`, `COUNT` or neither.
- The custom dialog (`DEC-165`) adds `INTERVAL=N` to any of them and `FREQ=MONTHLY;BYDAY=<n><DAY>` (e.g. `4TU`) or `BYDAY=-1<DAY>` (last); `FREQ=MONTHLY;BYMONTHDAY=<d>` and `FREQ=YEARLY;BYMONTH=<m>;BYMONTHDAY=<d>` as Google writes them read as the same-day rules. Any other rule (`BYSETPOS`, several `BYMONTH`, `BYHOUR`, several `RRULE` lines, `RDATE`) is "complex" and kept untouched (`MOD-05-F29`); `EXDATE` exceptions are kept with any rule.
- The rule's words in the menu are built from the rule, in Google's phrasing; parsing and writing rules are pure functions in `calendar-core`, test-first (constitution).
- Special types: `eventType` `outOfOffice` / `focusTime` are offered only when the calendar is the primary of a Workspace account; `workingLocation` and `birthday` are read-only.
- Undo is a compensating change sent to Google (re-create, or patch back to the previous values), not a local rollback.
- The default length comes from the Calendar settings value `defaultEventLength`.

## Out of scope / what not to worry about

- Guests, invitations, responses, notifying guests and Google Meet — `MOD-06`.
- Editing an event's reminders — `MOD-07`.
- Recurrence rules beyond Google's custom recurrence dialog, and creating working-location events — `DEC-165`, `DEC-34`.
- Adding attachments, ICS import and export — `DEC-37`, `MOD-08`.
- Natural-language event creation, tasks, AI assistant — `MOD-08`.
- Moving events between accounts.
- Editing offline — `DEC-25`, `MOD-08`.
- A full undo history — `DEC-59`.

## Acceptance criteria

- [ ] Selecting 10:00–11:30 in the week view, typing a title and pressing Enter creates the event in the default calendar, and it appears in Google Calendar in the browser.
- [ ] An event dragged to another day and resized keeps its new time in Google, and Ctrl+Z restores the previous time.
- [ ] A weekly event on Monday and Wednesday ending after 10 occurrences is created; changing "this and following" from the fifth occurrence leaves the first four unchanged.
- [ ] Deleting a single occurrence, then a whole series, works and is reflected in the browser.
- [ ] On a Workspace account's primary calendar an out-of-office event can be created and is shown hatched; on a personal account the option is not offered.
- [ ] Events in read-only calendars cannot be dragged or edited, and the app says why.
- [ ] An event deleted by mistake is restored with Undo within 5 seconds.
- [ ] An event's colour changed in the app shows in the browser.
- [ ] An event set in "Custom…" to repeat every 2 weeks on Monday and Wednesday, and one on the last Thursday of every month, show the same occurrences in the browser; a rule from Google the dialog can express is changed in the app.
