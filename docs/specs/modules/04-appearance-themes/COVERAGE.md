<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-04] Appearance and themes

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
| `MOD-04-F01` | `US-04-01` |
| `MOD-04-F02` | `US-04-01` |
| `MOD-04-F03` | `US-04-01` |
| `MOD-04-F04` | `US-04-03` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-04-U01` | `US-04-02` |
| `MOD-04-U02` | `US-04-02` |
| `MOD-04-U03` | `US-04-03` |
| `MOD-04-U04` | `US-04-02` |
| `MOD-04-U05` | `US-04-03` |

### Non-functional requirements

Module NFRs only; project-wide `NFR-nn` are delivered across modules and are not tracked per module.

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-04-NFR-01` | `US-04-02` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-04-01#1` | following the system | `TC-04-01` | auto | — |
| `US-04-01#2` | The settings offer System, Light and Dark. | `TC-04-02` | auto | — |
| `US-04-01#3` | Choosing Light or Dark overrides the desktop setting and is kept af… | `TC-04-02` | auto | — |
| `US-04-02#1` | At 100% scale, in a window 1080 pixels high, the week view shows 8:… | `TC-04-03` | auto | — |
| `US-04-02#2` | Event blocks in the day, 3-day and week views show only the title a… | — | — | no test case (`DEC-65`); unit test `an_event_block_shows_only_its_title_and_time` |
| `US-04-02#3` | Text inside events of a very light and a very dark calendar colour… | `TC-04-04` | auto | — |
| `US-04-02#4` | All interface text has a contrast of at least 4.5:1 against its bac… | — | — | no test case (`DEC-65`); unit tests `interface_text_reaches_4_5_to_1_on_its_backgrounds_in_both_themes` and the views' contrast tests; also looked over by the product owner |
| `US-04-03#1` | Apart from calendar and event colours, the interface uses only neut… | `TC-04-05` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-04-03#2` | Changing views and periods happens without transition animations. | `TC-04-05` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-04-03#3` | All interface texts are in English. | — | — | smoke depth — not tested yet (`DEC-65`) |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| A calendar colour is very light or very dark → the event text switc… | — | smoke depth — not tested yet (`DEC-65`) |
| The system changes between light and dark while the app runs → the… | — | smoke depth — not tested yet (`DEC-65`) |
| The desktop does not report a light or dark preference → the app us… | — | smoke depth — not tested yet (`DEC-65`) |
| The screen scale is 150% or 200% → the compact grid scales with it… | — | smoke depth — not tested yet (`DEC-65`) |
