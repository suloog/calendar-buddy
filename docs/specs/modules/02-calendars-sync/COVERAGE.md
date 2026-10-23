<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-02] Calendars, synchronisation and offline cache

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
| `MOD-02-F01` | `US-02-01` |
| `MOD-02-F02` | `US-02-01` |
| `MOD-02-F03` | `US-02-01` |
| `MOD-02-F04` | `US-02-07` |
| `MOD-02-F05` | `US-02-04` |
| `MOD-02-F06` | `US-02-04` |
| `MOD-02-F07` | `US-02-03` |
| `MOD-02-F08` | `US-02-03` |
| `MOD-02-F09` | `US-02-03` |
| `MOD-02-F10` | `US-02-03` |
| `MOD-02-F11` | `US-02-01` |
| `MOD-02-F12` | `US-02-05` |
| `MOD-02-F13` | `US-02-05` |
| `MOD-02-F14` | `US-02-05` |
| `MOD-02-F15` | `US-02-05` |
| `MOD-02-F16` | `US-02-06` |
| `MOD-02-F17` | `US-02-06` |
| `MOD-02-F18` | `US-02-06` |
| `MOD-02-F19` | `US-02-02` |
| `MOD-02-F20` | `US-02-02` |
| `MOD-02-F21` | `US-02-04` |
| `MOD-02-F22` | `US-02-01` |
| `MOD-02-F23` | `US-02-03` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-02-U01` | `US-02-02` |
| `MOD-02-U02` | `US-02-05` |
| `MOD-02-U03` | `US-02-06` |
| `MOD-02-U04` | `US-02-07` |

### Non-functional requirements

Module NFRs only; project-wide `NFR-nn` are delivered across modules and are not tracked per module.

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-02-NFR-01` | `US-02-03` |
| `MOD-02-NFR-02` | `US-02-01` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-02-01#1` | With two accounts connected, every calendar of both accounts is fet… | `TC-02-01` | auto | — |
| `US-02-01#2` | Events of all these calendars are shown together in the calendar, e… | `TC-02-01` | auto | — |
| `US-02-01#3` | The cache holds events from one year back to two years ahead of today. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-01#4` | An event stored in another time zone, an event across a daylight-sa… | `TC-02-02` | auto | — |
| `US-02-01#5` | The first full synchronization of 5 accounts, 50 calendars and abou… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-02#1` | A subscribed holiday calendar and a colleague's read-only calendar… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-02#2` | Their events are visibly marked as not editable. | `TC-02-03` | auto | — |
| `US-02-02#3` | A calendar shared as free/busy only shows "busy" blocks, without ti… | `TC-02-03` | auto | — |
| `US-02-03#1` | With the window open, a change made on the phone to an editable cal… | `TC-02-04`, `TC-02-05` | auto | — |
| `US-02-03#2` | A change to a read-only or subscribed calendar appears within about… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-03#3` | With the window hidden in the tray, a change to an editable calenda… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-03#4` | Activating the app window synchronizes at once; so does opening an… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-03#5` | Right after waking from sleep and when the network returns, the app… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-03#6` | "Refresh" synchronizes at once. | `TC-02-04` | auto | — |
| `US-02-03#7` | After a calendar's first full synchronization only changes are fetc… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-03#8` | Background synchronization of 5 accounts and 50 enabled calendars c… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-04#1` | A calendar created in Google appears at the next synchronization; a… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-04#2` | a calendar is unshared | `TC-02-06` | auto | — |
| `US-02-04#3` | A calendar disabled in the settings stays on the list of known cale… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-04#4` | Enabling a disabled calendar starts its first synchronization, and… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-05#1` | On start, the cached calendars are shown at once, before synchroniz… | `TC-02-07` | auto | — |
| `US-02-05#2` | no network | `TC-02-07` | auto | — |
| `US-02-05#3` | Online, a period outside the cache window is fetched when browsed a… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-05#4` | Offline, a period outside the cache window is shown empty with a no… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-06#1` | The indicator shows whether the app is synchronizing, up to date, o… | `TC-02-08` | auto | — |
| `US-02-06#2` | The user can keep browsing and working while synchronization runs. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-06#3` | An error affecting one calendar is shown on that calendar, and the… | `TC-02-08` | auto | — |
| `US-02-06#4` | After a lost connection, a rate limit or a transient error, synchro… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-02-07#1` | From a calendar's menu in the side panel, the user picks a colour,… | `TC-02-09` | auto | — |
| `US-02-07#2` | The calendar's colour is unchanged in Google Calendar in the browser. | `TC-02-09` | auto | — |
| `US-02-07#3` | "Reset" brings back the calendar's Google colour. | `TC-02-09` | auto | — |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| An event that crosses the edge of the cache window → it is kept whole. | — | smoke depth — not tested yet (`DEC-65`) |
| A recurring series that started before the window → its occurrences… | — | smoke depth — not tested yet (`DEC-65`) |
| Google answers with a rate limit → synchronization slows down, the… | — | smoke depth — not tested yet (`DEC-65`) |
| Google rejects the stored synchronization state → that calendar is… | — | smoke depth — not tested yet (`DEC-65`) |
| An account needs sign-in (`MOD-01-F05`) → its cached calendars stay… | — | smoke depth — not tested yet (`DEC-65`) |
| A calendar's access drops from editable to read-only → it is marked… | — | smoke depth — not tested yet (`DEC-65`) |
| A calendar is deleted while the app is offline → it is removed at t… | — | smoke depth — not tested yet (`DEC-65`) |
| The cache is damaged or unreadable → the app discards it and rebuil… | — | smoke depth — not tested yet (`DEC-65`) |
| The day changes → the window moves: events older than one year are… | — | smoke depth — not tested yet (`DEC-65`) |
