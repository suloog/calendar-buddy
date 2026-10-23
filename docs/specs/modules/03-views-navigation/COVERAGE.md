<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-03] Views and navigation

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
| `MOD-03-F01` | `US-03-02` |
| `MOD-03-F02` | `US-03-01` |
| `MOD-03-F03` | `US-03-03` |
| `MOD-03-F04` | `US-03-02` |
| `MOD-03-F05` | `US-03-04` |
| `MOD-03-F06` | `US-03-05` |
| `MOD-03-F07` | `US-03-05` |
| `MOD-03-F08` | `US-03-05` |
| `MOD-03-F09` | `US-03-05` |
| `MOD-03-F10` | `US-03-06` |
| `MOD-03-F11` | `US-03-06` |
| `MOD-03-F12` | `US-03-06` |
| `MOD-03-F13` | `US-03-07` |
| `MOD-03-F14` | `US-03-01` |
| `MOD-03-F15` | `US-03-05` |
| `MOD-03-F16` | `US-03-09` |
| `MOD-03-F17` | `US-03-09` |
| `MOD-03-F18` | `US-03-09` |
| `MOD-03-F19` | `US-03-01` |
| `MOD-03-F20` | `US-03-03` |
| `MOD-03-F21` | `US-03-01` |
| `MOD-03-F22` | `US-03-01` |
| `MOD-03-F23` | `US-03-08` |
| `MOD-03-F24` | `US-03-08` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-03-U01` | `US-03-01` |
| `MOD-03-U02` | `US-03-01` |
| `MOD-03-U03` | `US-03-06` |
| `MOD-03-U04` | `US-03-08` |
| `MOD-03-U05` | `US-03-01` |

### Non-functional requirements

Module NFRs only; project-wide `NFR-nn` are delivered across modules and are not tracked per module.

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-03-NFR-01` | `US-03-01` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-03-01#1` | A week grid shows seven days with their hours; all-day and multi-da… | `TC-03-01` | auto | — |
| `US-03-01#2` | Today is highlighted, a line marks the current time, and the view o… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-03-01#3` | Overlapping events are placed side by side, each one readable. | `TC-03-02` | auto | — |
| `US-03-01#4` | The layout adapts to the window size and to 100%, 150% and 200% scr… | `TC-03-03` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-03-01#5` | On a profile of 5 accounts, 50 calendars and about 20,000 events, s… | `TC-03-04` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-03-01#6` | The feasibility check of the interface technology — smooth scrollin… | `TC-03-04` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-03-02#1` | A day view and a 3-day view are each available as a view the user p… | `TC-03-05` | auto | — |
| `US-03-02#2` | Both show the all-day strip, the current-time line and overlapping… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-03-03#1` | The month view shows each day of the month with its events. | `TC-03-06` | auto | — |
| `US-03-03#2` | A day with more events than fit shows how many more there are; choo… | `TC-03-06` | auto | — |
| `US-03-04#1` | The agenda lists events from now on, grouped by day, without empty… | `TC-03-07` | auto | — |
| `US-03-04#2` | More events load as the user scrolls down. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-03-04#3` | With nothing ahead, the agenda says there are no upcoming events. | `TC-03-07` | auto | — |
| `US-03-05#1` | Switching between views keeps the selected date in view. | `TC-03-08` | auto | — |
| `US-03-05#2` | Next and previous move by the period of the current view; "Today" b… | `TC-03-08` | auto | — |
| `US-03-05#3` | The mini calendar marks the period on screen and jumps to any date… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-03-05#4` | After a restart, the app opens in the view used last, on today's date. | `TC-03-08` | auto | — |
| `US-03-06#1` | The side panel lists each connected account and, under it, its cale… | `TC-03-09` | auto | — |
| `US-03-06#2` | A quick toggle next to a calendar hides or shows its events; the ca… | `TC-03-09` | auto | — |
| `US-03-06#3` | The panel can be collapsed and expanded, and the app remembers whic… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-03-06#4` | When the window becomes too narrow, the panel collapses on its own. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-03-07#1` | In the settings, the user can disable a calendar and enable it again. | `TC-03-10` | auto | — |
| `US-03-07#2` | A disabled calendar does not appear in the side panel, in the calen… | `TC-03-10` | auto | — |
| `US-03-08#1` | `d`, `x` (3 days), `w`, `m` and `a` switch to the day, 3-day, week,… | `TC-03-11` | auto | — |
| `US-03-08#2` | `j` or `n` moves to the next period, `k` or `p` to the previous one… | `TC-03-11` | auto | — |
| `US-03-08#3` | `c` starts a new event. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-03-08#4` | `?` shows the list of shortcuts. | `TC-03-11` | auto | — |
| `US-03-08#5` | Without a mouse, the user moves between the side panel, the calenda… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-03-09#1` | With a system locale whose week starts on Sunday, the day, 3-day an… | `TC-03-12` | auto | — |
| `US-03-09#2` | After the system's time zone changes, events are shown at their new… | `TC-03-12` | auto | — |
| `US-03-09#3` | Opening an event stored in another time zone shows its time in that… | — | — | smoke depth — not tested yet (`DEC-65`) |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| Many events overlap at the same time → each gets a narrower column… | — | smoke depth — not tested yet (`DEC-65`) |
| An event runs past midnight → it is shown on both days. | — | smoke depth — not tested yet (`DEC-65`) |
| A day with a daylight-saving change has 23 or 25 hours → the hours… | — | smoke depth — not tested yet (`DEC-65`) |
| No calendar is visible (all hidden or disabled) → the view is empty… | — | smoke depth — not tested yet (`DEC-65`) |
| The agenda has nothing ahead → it says there are no upcoming events. | — | smoke depth — not tested yet (`DEC-65`) |
| Offline, the user moves outside the cache window → the period is sh… | — | smoke depth — not tested yet (`DEC-65`) |
| The window is made very small → it does not go below a minimum size… | — | smoke depth — not tested yet (`DEC-65`) |
