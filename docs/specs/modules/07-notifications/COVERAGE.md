<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-07] Reminders and notifications

> The maps the other way round from a story's **Covers** field and a test case's **Verifies** field.
> They stop a requirement quietly falling out when the module is sliced, and a criterion quietly
> falling out when the tests are written.
>
> Fill it in the same run that writes the story or the test case. A row with an empty "Covered by"
> is a requirement not yet sliced or a criterion with no test. The module is **sliced** when part 1
> has no empty row; it is **covered** when every row of part 2 has a test **or** an explicit
> justification in the "Note" column (e.g. "manual verification — email delivery").
>
> Source: `REQUIREMENTS.md`, `stories/`, `tests/` in this directory · Registers: `docs/specs/BACKLOG.md`,
> `docs/specs/TESTPLAN.md`

<!-- og:part stories -->

## Part 1 — Requirements → user stories

### Functional requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-07-F01` | `US-07-01` |
| `MOD-07-F02` | `US-07-01` |
| `MOD-07-F03` | `US-07-04` |
| `MOD-07-F04` | `US-07-04` |
| `MOD-07-F05` | `US-07-04` |
| `MOD-07-F06` | `US-07-01` |
| `MOD-07-F07` | `US-07-01` |
| `MOD-07-F19` | `US-07-01` |
| `MOD-07-F20` | `US-07-01` |
| `MOD-07-F08` | `US-07-01` |
| `MOD-07-F09` | `US-07-02` |
| `MOD-07-F10` | `US-07-02` |
| `MOD-07-F11` | `US-07-03` |
| `MOD-07-F12` | `US-07-03` |
| `MOD-07-F13` | `US-07-01` |
| `MOD-07-F14` | `US-07-05` |
| `MOD-07-F15` | `US-07-05` |
| `MOD-07-F16` | `US-07-05` |
| `MOD-07-F17` | `US-07-05` |
| `MOD-07-F18` | `US-07-06` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-07-U01` | `US-07-01` |
| `MOD-07-U02` | `US-07-05` |
| `MOD-07-U03` | `US-07-03` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-07-01#1` | An event with a 10-minute notification reminder produces one deskto… | `TC-07-01` | auto | — |
| `US-07-01#2` | The notification is not repeated after several synchronizations. | `TC-07-01` | auto | — |
| `US-07-01#3` | Clicking the notification opens the event in the app. | `TC-07-07` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-07-01#4` | E-mail reminders produce no desktop notification. | `TC-07-02` | auto | — |
| `US-07-01#5` | Declined events and events of calendars disabled in the settings pr… | `TC-07-02` | auto | — |
| `US-07-01#6` | Reminders whose time passed while the app was not running are not s… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-07-01#7` | Reminders that fell while the computer slept are shown on waking if… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-07-02#1` | A notification offers Dismiss, Snooze 5 minutes and Snooze 10 minutes. | `TC-07-03` | auto | — |
| `US-07-02#2` | A snoozed notification appears again after the chosen time; nothing… | `TC-07-03` | auto | — |
| `US-07-02#3` | A notification for an event with Google Meet has a Join button that… | `TC-07-07` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-07-03#1` | A switch in the settings turns all notifications off and on again. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-07-03#2` | A calendar muted from its menu in the side panel still shows its ev… | `TC-07-04` | auto | — |
| `US-07-03#3` | Turning notifications off or muting a calendar leaves the reminders… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-07-04#1` | A new event gets its calendar's default reminders from Google. | `TC-07-05` | auto | — |
| `US-07-04#2` | In the editor the user adds and removes reminders — as a notificati… | `TC-07-05` | auto | — |
| `US-07-04#3` | A reminder changed in the app shows in Google Calendar in the browser. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-07-04#4` | "Use calendar defaults" brings the event back to its calendar's def… | `TC-07-05` | auto | — |
| `US-07-05#1` | Closing the window hides the app to the system tray, and reminders… | `TC-07-08` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-07-05#2` | Clicking the tray icon shows or hides the window. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-07-05#3` | The tray menu opens the window and quits the app completely; after… | `TC-07-08` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-07-05#4` | "Start at login" in the settings is off by default; turned on, the… | `TC-07-08` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-07-05#5` | Launching the app while it runs brings the existing window forward… | `TC-07-06` | auto | — |
| `US-07-06#1` | Clicking a `webcal:` link in another application opens Google Calen… | `TC-07-09` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-07-06#2` | After the user adds the calendar there, it appears in the app at th… | — | — | smoke depth — not tested yet (`DEC-65`) |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| An event is moved or deleted before its reminder → the reminder mov… | — | smoke depth — not tested yet (`DEC-65`) |
| Several reminders fall in the same minute → each gets its own notif… | — | smoke depth — not tested yet (`DEC-65`) |
| The desktop's "do not disturb" mode is on → the desktop decides wha… | — | smoke depth — not tested yet (`DEC-65`) |
| The desktop's notification service does not support buttons → click… | — | smoke depth — not tested yet (`DEC-65`) |
| The system time zone changes → reminders follow the events' real ti… | — | smoke depth — not tested yet (`DEC-65`) |
| Notifications are off globally → nothing is shown, but the reminder… | — | smoke depth — not tested yet (`DEC-65`) |
