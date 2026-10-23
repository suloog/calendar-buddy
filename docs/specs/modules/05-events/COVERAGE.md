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
| `MOD-05-F16` | `US-05-01` |
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
| `MOD-05-F29` | `US-05-07` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-05-U01` | `US-05-08` |
| `MOD-05-U02` | `US-05-03` |
| `MOD-05-U03` | `US-05-01` |
| `MOD-05-U04` | `US-05-04` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

<One row per criterion of every story in the module, in story order. `#n` is the criterion's
position in the story file — counted from 1, with a Given/When/Then scenario counting as one.>

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-NN-01#1` | … | `TC-NN-01` | auto | — |
| `US-NN-01#2` | … | `TC-NN-02`, `TC-NN-03` | auto | — |
| `US-NN-02#1` | … | — | manual-only | email delivery — out of the executor's reach |

### Module constraints and edge cases

<Items from the "Constraints and edge cases" section of REQUIREMENTS.md. This is where `type: edge`
and `type: negative` tests come from — the module states them once, the tests prove them one by one.>

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| … | `TC-NN-04` | — |

### Non-functional requirements under test

<Only the `NFR-nn` and `MOD-NN-NFR-nn` that can be checked against this module. The "How" column
says what the evidence is measured with — response time from the network log, query count,
contrast, touch target size. An NFR with no measurable evidence is `manual-only`, not green.>

| Requirement | Covered by | How | Note |
| ----------- | ---------- | --- | ---- |
| `NFR-03` | `TC-NN-05` | response time from the network log | — |
