<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-05] Creating and editing events

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
| `MOD-05-F01` | `US-05-01` |
| `MOD-05-F02` | `US-05-01` |
| `MOD-05-F03` | `US-05-02` |
| `MOD-05-F04` | `US-05-02` |
| `MOD-05-F05` | `US-05-02` |
| `MOD-05-F06` | `US-05-02` |
| `MOD-05-F07` | `US-05-02` |
| `MOD-05-F08` | `US-05-02` |
| `MOD-05-F09` | `US-05-01` |
| `MOD-05-F10` | `US-05-01` |
| `MOD-05-F11` | `US-05-02` |
| `MOD-05-F12` | `US-05-04` |
| `MOD-05-F13` | `US-05-03` |
| `MOD-05-F14` | `US-05-03` |
| `MOD-05-F15` | `US-05-02` |
| `MOD-05-F16` | `US-05-01`, `US-05-02`, `US-05-03`, `US-05-04` |
| `MOD-05-F17` | `US-05-05` |
| `MOD-05-F18` | `US-05-06` |
| `MOD-05-F19` | `US-05-06` |
| `MOD-05-F20` | `US-05-07` |
| `MOD-05-F21` | `US-05-07` |
| `MOD-05-F22` | `US-05-05` |
| `MOD-05-F23` | `US-05-08` |
| `MOD-05-F24` | `US-05-08` |
| `MOD-05-F25` | `US-05-09` |
| `MOD-05-F26` | `US-05-10` |
| `MOD-05-F27` | `US-05-04` |
| `MOD-05-F28` | `US-05-04` |
| `MOD-05-F29` | `US-05-07`, `US-05-11` |
| `MOD-05-F30` | `US-05-02` |
| `MOD-05-F31` | `US-05-02` |
| `MOD-05-F32` | `US-05-11` |
| `MOD-05-F33` | `US-05-11` |
| `MOD-05-F34` | `US-05-11` |
| `MOD-05-F35` | `US-05-11` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-05-U01` | `US-05-08` |
| `MOD-05-U02` | `US-05-03` |
| `MOD-05-U03` | `US-05-01`, `US-05-02`, `US-05-03` |
| `MOD-05-U04` | `US-05-04` |
| `MOD-05-U05` | `US-05-11` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-05-01#1` | Selecting 10:00–11:30 in the week view opens a quick-entry box with… | `TC-05-01` | auto | — |
| `US-05-01#2` | Typing a title and pressing Enter creates the event, and it appears… | `TC-05-01` | auto | — |
| `US-05-01#3` | A single click on a time, or the `c` key, starts an event of the de… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-01#4` | The default calendar is chosen in the settings; until the user choo… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-01#5` | A new event is marked "saving" until Google confirms it. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-02#1` | "More options" in the quick-entry box opens the full editor with wh… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-02#2` | In the editor the user sets the title, description, start, end, tim… | `TC-05-02` | auto | — |
| `US-05-02#3` | An event can be made all-day, and can span several days. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-02#4` | The user can set the location, whether the event shows them as busy… | `TC-05-02` | auto | — |
| `US-05-02#5` | Any field of an event the user may edit can be changed later in the… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-02#6` | Choosing another calendar of the same account moves the event there. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-03#1` | Dragging an event moves it in 15-minute steps, shows the new time w… | `TC-05-03` | auto | — |
| `US-05-03#2` | An event can be dragged to another day in the 3-day and week views. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-03#3` | Dragging the end of an event changes its length, in 15-minute steps. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-03#4` | After dropping, the new time appears in Google Calendar in the brow… | `TC-05-03` | auto | — |
| `US-05-04#1` | The user can delete an event they are allowed to delete, from its d… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-04#2` | An event without guests that is not part of a series is deleted wit… | `TC-05-04` | auto | — |
| `US-05-04#3` | After moving, resizing or editing an event, Ctrl+Z — or Undo in the… | `TC-05-05` | auto | — |
| `US-05-05#1` | Events of read-only calendars, and events the user is not allowed t… | `TC-05-06` | auto | — |
| `US-05-05#2` | When Google or the organizer refuses a change, a clear message is s… | — | — | no test case (smoke depth, `DEC-65`); proven by `a_change_google_refuses_is_told_clearly_and_the_event_returns_to_how_it_was` |
| `US-05-06#1` | In the editor the user makes an event repeat daily, every weekday,… | `TC-05-07` | auto | — |
| `US-05-06#2` | The series ends on a chosen date, after a chosen number of occurren… | `TC-05-07` | auto | — |
| `US-05-06#3` | A series created in the app shows the same occurrences in Google Ca… | `TC-05-12` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-05-07#1` | changing from the fifth occurrence on | `TC-05-08` | auto | — |
| `US-05-07#2` | Changing an occurrence — in the editor, by dragging or by resizing… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-07#3` | Deleting an occurrence asks the same three-way question, and the re… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-07#4` | A series created in Google with a rule the presets cannot express k… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-08#1` | Out-of-office events are hatched, focus-time events carry an icon,… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-08#2` | On the primary calendar of a Workspace account, the editor offers o… | `TC-05-09` | auto | — |
| `US-05-08#3` | On personal accounts and other calendars, these types are not offered. | `TC-05-09` | auto | — |
| `US-05-09#1` | An event that has its own colour in Google is shown in that colour. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-05-09#2` | Changing the event's colour in the editor shows in Google Calendar… | `TC-05-10` | auto | — |
| `US-05-10#1` | An event with attachments lists them in its details by name. | `TC-05-11` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-05-10#2` | Choosing an attachment opens it in the browser. | `TC-05-11` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-05-11#1` | For an event starting on Tuesday 17 November 2026, the repeat menu… | `TC-05-13` | auto | — |
| `US-05-11#2` | In "Custom…" the user sets "every 2 weeks on Monday and Wednesday,… | `TC-05-13` | auto | — |
| `US-05-11#3` | For months the dialog offers "Monthly on day 17" and "Monthly on t… | `TC-05-13` | auto | — |
| `US-05-11#4` | For weeks the dialog shows the weekdays with the start's weekday c… | `TC-05-13` | auto | — |
| `US-05-11#5` | A series created in Google as "every 3 months on the second Monday… | `TC-05-14` | auto | — |
| `US-05-11#6` | Changing the rule of an occurrence asks "This and following events… | `TC-05-14` | auto | — |
| `US-05-11#7` | A series from Google with a rule the dialog cannot express (e.g. `… | `TC-05-14` | auto | — |
| `US-05-11#8` | A series set in "Custom…" shows the same occurrences in Google Cal… | `TC-05-15` | manual-only | manual — checked by the product owner (`DEC-65`) |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| The end is set before the start in the editor → the event cannot be… | — | smoke depth — not tested yet (`DEC-65`) |
| An all-day event is dragged into the hours → it becomes a timed eve… | — | smoke depth — not tested yet (`DEC-65`) |
| The event was changed elsewhere since it was opened → the app reloa… | — | smoke depth — not tested yet (`DEC-65`) |
| Undo after the event was changed elsewhere → the undo is refused wi… | — | smoke depth — not tested yet (`DEC-65`) |
| "This and following" on the first occurrence → treated as "all even… | — | smoke depth — not tested yet (`DEC-65`) |
| Moving an event to a calendar of another account → not offered (Goo… | — | smoke depth — not tested yet (`DEC-65`) |
| No network → creating and changing are unavailable (`MOD-02-F15`). | — | smoke depth — not tested yet (`DEC-65`) |
| An event stored in another time zone → edited in its own time zone,… | — | smoke depth — not tested yet (`DEC-65`) |
| Weekly with no weekday chosen → the dialog does not allow it; the e… | `TC-05-13` | — |
| Monthly on day 29–31 → months without that day are skipped, as in G… | — | smoke depth — not tested yet (`DEC-65`) |
| "Ends on" a date before the start, or "after" 0 occurrences → the d… | — | smoke depth — not tested yet (`DEC-65`) |
| The event's start date changes after a rule was chosen → the weekda… | — | smoke depth — not tested yet (`DEC-65`) |
