<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-10] Events from a description

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
| `MOD-10-F01` | `US-10-02` |
| `MOD-10-F02` | `US-10-02` |
| `MOD-10-F03` | `US-10-02`, `US-10-03` |
| `MOD-10-F04` | `US-10-02` |
| `MOD-10-F05` | `US-10-02` |
| `MOD-10-F06` | `US-10-02`, `US-10-03` |
| `MOD-10-F07` | `US-10-03` |
| `MOD-10-F08` | `US-10-02` |
| `MOD-10-F09` | `US-10-05` |
| `MOD-10-F10` | `US-10-05` |
| `MOD-10-F11` | `US-10-03` |
| `MOD-10-F12` | `US-10-03` |
| `MOD-10-F13` | `US-10-05` |
| `MOD-10-F14` | `US-10-05` |
| `MOD-10-F15` | `US-10-04` |
| `MOD-10-F16` | `US-10-04` |
| `MOD-10-F17` | `US-10-04` |
| `MOD-10-F18` | `US-10-04` |
| `MOD-10-F19` | `US-10-01` |
| `MOD-10-F20` | `US-10-01` |
| `MOD-10-F21` | `US-10-01` |
| `MOD-10-F22` | `US-10-02` |
| `MOD-10-F23` | `US-10-02` |
| `MOD-10-F24` | `US-10-02` |
| `MOD-10-F25` | `US-10-03` |
| `MOD-10-F26` | `US-10-01` |
| `MOD-10-F27` | `US-10-02` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-10-U01` | `US-10-02` |
| `MOD-10-U02` | `US-10-02` |
| `MOD-10-U03` | `US-10-02` |
| `MOD-10-U04` | `US-10-01` |
| `MOD-10-U05` | `US-10-01` |
| `MOD-10-U06` | `US-10-05` |
| `MOD-10-U07` | `US-10-05` |
| `MOD-10-U08` | `US-10-04` |

### Non-functional requirements

<Module NFRs only. Global NFR-nn are delivered across modules and are not tracked per module.>

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-10-NFR-01` | `US-10-02` |
| `MOD-10-NFR-02` | `US-10-01` |
| `MOD-10-NFR-03` | `US-10-02` |
| `MOD-10-NFR-04` | `US-10-02`, `US-10-04` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

<One row per criterion of every story in the module, in story order. `#n` is the criterion's
position in the story file — counted from 1, with a Given/When/Then scenario counting as one.>

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |

### Module constraints and edge cases

<Items from the "Constraints and edge cases" section of REQUIREMENTS.md. This is where `type: edge`
and `type: negative` tests come from — the module states them once, the tests prove them one by one.>

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |

### Non-functional requirements under test

<Only the `NFR-nn` and `MOD-NN-NFR-nn` that can be checked against this module. The "How" column
says what the evidence is measured with — response time from the network log, query count,
contrast, touch target size. An NFR with no measurable evidence is `manual-only`, not green.>

| Requirement | Covered by | How | Note |
| ----------- | ---------- | --- | ---- |
