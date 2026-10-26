<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-09] Local calendars

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
| `MOD-09-F01` | `US-09-01` |
| `MOD-09-F02` | `US-09-02` |
| `MOD-09-F03` | `US-09-02` |
| `MOD-09-F04` | `US-09-02` |
| `MOD-09-F05` | `US-09-02` |
| `MOD-09-F06` | `US-09-02` |
| `MOD-09-F07` | `US-09-01` |
| `MOD-09-F08` | `US-09-01` |
| `MOD-09-F09` | `US-09-03` |
| `MOD-09-F10` | `US-09-03` |
| `MOD-09-F11` | `US-09-01`, `US-09-02` |
| `MOD-09-F12` | `US-09-03` |
| `MOD-09-F13` | `US-09-04` |
| `MOD-09-F14` | `US-09-04` |
| `MOD-09-F15` | `US-09-04` |
| `MOD-09-F16` | `US-09-03` |
| `MOD-09-F17` | `US-09-01` |
| `MOD-09-F18` | `US-09-01` |
| ~~`MOD-09-F19`~~ | withdrawn, `DEC-185` |
| `MOD-09-F20` | `US-09-01` |
| `MOD-09-F21` | `US-09-02` |
| `MOD-09-F22` | `US-09-05` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-09-U01` | `US-09-02`, `US-09-04` |
| `MOD-09-U02` | `US-09-01` |
| `MOD-09-U03` | `US-09-02` |
| `MOD-09-U04` | `US-09-01` |
| `MOD-09-U05` | `US-09-01` |
| `MOD-09-U06` | `US-09-05` |

### Non-functional requirements

Module NFRs only; project-wide `NFR-nn` are delivered across modules and are not tracked per module.

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-09-NFR-01` | `US-09-01` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-09-01#1` | A first run with neither an account nor a local calendar shows the w… | `TC-09-01` | auto | — |
| `US-09-01#2` | "Use without an account" opens the week view; the side panel shows o… | `TC-09-01` | auto | — |
| `US-09-01#3` | Clicking a time, typing a title and pressing Enter shows the event a… | `TC-09-01` | auto | — |
| `US-09-01#4` | Opening the event shows its details; the full editor opened from the… | `TC-09-01` | auto | — |
| `US-09-01#5` | After the app is restarted the welcome screen is not shown and the e… | `TC-09-02` | auto | — |
| `US-09-01#6` | With no account connected, a new event without a range lasts 60 minu… | `TC-09-03` | auto | — |
| `US-09-01#7` | A Google account connected later from the settings is listed above "… | `TC-09-03` | auto | — |
| `US-09-01#8` | When the local file cannot be read at start, the app says so, leaves… | `TC-09-04` | auto | — |
| `US-09-01#9` | When a change cannot be written to the local file, a bar filled with… | `TC-09-14` | auto | — |
| `US-09-02#1` | The settings have a "Local calendars" section listing each local cal… | `TC-09-05` | auto | — |
| `US-09-02#2` | Renaming a local calendar and choosing another colour from the palet… | `TC-09-05` | auto | — |
| `US-09-02#3` | A local calendar can be hidden with the side panel's toggle, disable… | `TC-09-06` | auto | — |
| `US-09-02#4` | A local calendar can be chosen as the default calendar in the settin… | `TC-09-05` | auto | — |
| `US-09-02#5` | Deleting a local calendar with events asks for confirmation, naming … | `TC-09-07` | auto | — |
| `US-09-02#6` | Deleting the default local calendar forgets the choice, and new even… | `TC-09-07` | auto | — |
| `US-09-02#7` | The last local calendar cannot be deleted: its **Delete** is unavail… | `TC-09-07` | auto | — |
| `US-09-02#8` | With a Google account connected and no local calendar (Google connec… | `TC-09-15` | auto | — |
| `US-09-03#1` | Every field of a local event the editor offers (title, description, … | `TC-09-08` | auto | — |
| `US-09-03#2` | A local event can be dragged to another time or day and its end drag… | `TC-09-08` | auto | — |
| `US-09-03#3` | Deleting a local event that is not part of a series deletes it witho… | `TC-09-08` | auto | — |
| `US-09-03#4` | A local event can be made repeating from the repeat menu and the cus… | `TC-09-09` | auto | — |
| `US-09-03#5` | The editor of a local event has no guests, Google Meet, attachments,… | `TC-09-10` | auto | — |
| `US-09-03#6` | The calendar picker of an existing local event lists only local cale… | `TC-09-10` | auto | — |
| `US-09-03#7` | Creating, changing and deleting local events sends nothing to Google… | `TC-09-10` | auto | — |
| `US-09-04#1` | A new local calendar has one default notification, 30 minutes before;… | `TC-09-11` | auto | — |
| `US-09-04#2` | A local event's notification can be snoozed for 5 or 10 minutes and … | `TC-09-13` | manual-only | — |
| `US-09-04#3` | In the "Local calendars" section a calendar's default notifications … | `TC-09-11` | auto | — |
| `US-09-04#4` | In the editor a local event's notifications can be added and removed… | `TC-09-12` | auto | — |
| `US-09-04#5` | A muted or disabled local calendar gives no notifications, nor does … | `TC-09-12` | auto | — |
| `US-09-05#1` | The settings' "Local calendars" section has a "Show local calendars" c… | `TC-09-16` | auto | — |
| `US-09-05#2` | Unticking it takes "Local" out of the side panel and the calendar pi… | `TC-09-16` | auto | — |
| `US-09-05#3` | Ticking it again brings everything back as it was, events included;… | `TC-09-16` | auto | — |
| `US-09-05#4` | With a local calendar as the default calendar, while the local calen… | `TC-09-16` | auto | — |
| `US-09-05#5` | With no Google account connected the checkbox is ticked and unavaila… | `TC-09-17` | auto | — |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| The app runs offline → local events are created, changed and deleted… | `TC-09-08` | — |
| No account is connected → the default event length is Google's defau… | `TC-09-03` | — |
| No account is connected and every local calendar is disabled → creat… | — | smoke depth — not tested yet (`DEC-65`) |
| The disk is full, or the file or its directory cannot be written → th… | `TC-09-14` | — |
| The cache does not take the local calendars when the file is opened… | — | covered by a unit test, `local_calendars_the_cache_did_not_take_are_started_again` |
| The local file cannot be read at start (damaged, or from a newer ver… | `TC-09-04` | — |
| Deleting a local calendar is not undone with Ctrl+Z; the confirmatio… | `TC-09-07` | — |
| A local calendar's name may repeat another calendar's name; it is no… | — | smoke depth — not tested yet (`DEC-65`) |
| A local series' rule is always one the custom recurrence dialog can … | — | smoke depth — not tested yet (`DEC-65`) |
