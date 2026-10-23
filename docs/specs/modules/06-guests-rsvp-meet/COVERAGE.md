<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-06] Guests, invitations, RSVP and Google Meet

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
| `MOD-06-F01` | `US-06-01` |
| `MOD-06-F02` | `US-06-01` |
| `MOD-06-F03` | `US-06-01` |
| `MOD-06-F04` | `US-06-01` |
| `MOD-06-F05` | `US-06-02` |
| `MOD-06-F06` | `US-06-01` |
| `MOD-06-F07` | `US-06-01` |
| `MOD-06-F08` | `US-06-01` |
| `MOD-06-F09` | `US-06-04` |
| `MOD-06-F10` | `US-06-04` |
| `MOD-06-F11` | `US-06-05` |
| `MOD-06-F12` | `US-06-03` |
| `MOD-06-F13` | `US-06-05` |
| `MOD-06-F14` | `US-06-06` |
| `MOD-06-F22` | `US-06-06` |
| `MOD-06-F15` | `US-06-05` |
| `MOD-06-F21` | `US-06-04` |
| `MOD-06-F16` | `US-06-07` |
| `MOD-06-F17` | `US-06-07` |
| `MOD-06-F18` | `US-06-07` |
| `MOD-06-F19` | `US-06-08` |
| `MOD-06-F20` | `US-06-08` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-06-U01` | `US-06-02` |
| `MOD-06-U02` | `US-06-08` |
| `MOD-06-U03` | `US-06-05` |
| `MOD-06-U04` | `US-06-04` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-06-01#1` | The user adds a guest by typing an e-mail address. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-06-01#2` | Typing part of a name or address suggests people from the account's… | `TC-06-01` | auto | — |
| `US-06-01#3` | On a Workspace account, the suggestions also include colleagues fro… | `TC-06-02` | auto | — |
| `US-06-01#4` | A guest can be marked as required or optional. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-06-01#5` | Guests can be added to and removed from an existing event. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-06-01#6` | Saving a new event with guests sends them invitations through Google. | `TC-06-01`, `TC-06-13` | auto | — |
| `US-06-02#1` | An event's details list each guest's name, address and answer, show… | `TC-06-03` | auto | — |
| `US-06-02#2` | When a guest answers, the icon changes after the next synchronization. | `TC-06-03` | auto | — |
| `US-06-02#3` | A guest list of more than 10 people is collapsed and can be expanded. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-06-03#1` | moving a meeting | `TC-06-04` | auto | — |
| `US-06-03#2` | The same question is asked when saving a change in the editor and w… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-06-04#1` | An invitation awaiting the user's answer has a dashed outline and a… | `TC-06-05` | auto | — |
| `US-06-04#2` | The side panel shows how many invitations await an answer; choosing… | `TC-06-05` | auto | — |
| `US-06-04#3` | Invitations in a calendar disabled in the settings are neither show… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-06-05#1` | An invitation's details open with Accept, Maybe and Decline at the… | `TC-06-06` | auto | — |
| `US-06-05#2` | An answer given in the app appears in Google Calendar in the browser. | `TC-06-06` | auto | — |
| `US-06-05#3` | Answering an occurrence of a recurring invitation asks whether the… | `TC-06-07` | auto | — |
| `US-06-05#4` | As a guest without edit rights, the user can answer but cannot chan… | `TC-06-08` | auto | — |
| `US-06-06#1` | A declined event stays in the views, struck through and faded. | `TC-06-09` | auto | — |
| `US-06-06#2` | One switch in the views hides and shows declined events; they are s… | `TC-06-09` | auto | — |
| `US-06-07#1` | The editor has an "Add Google Meet" switch; it turns on when the fi… | `TC-06-10` | auto | — |
| `US-06-07#2` | A saved event with the switch on has a Google Meet link, also in Go… | `TC-06-10` | auto | — |
| `US-06-07#3` | An event with a Meet link has a Join button in its details and in t… | `TC-06-11` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-06-08#1` | Under the chosen time, the editor shows one strip per guest with th… | `TC-06-12` | auto | — |
| `US-06-08#2` | A guest busy at the chosen time is marked as a conflict. | `TC-06-12` | auto | — |
| `US-06-08#3` | A guest who does not share availability is shown as "no data". | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-06-08#4` | The preview shows up to the first 20 guests. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-06-08#5` | Without network, the preview is not shown. | — | — | smoke depth — not tested yet (`DEC-65`) |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| An address with an invalid format → it is not added, and the field… | — | smoke depth — not tested yet (`DEC-65`) |
| The same guest is added twice, or the user adds their own address →… | — | smoke depth — not tested yet (`DEC-65`) |
| An event has many guests → the guest list is collapsed beyond the f… | — | smoke depth — not tested yet (`DEC-65`) |
| Google cannot create a Meet link for the account → the event is sav… | — | smoke depth — not tested yet (`DEC-65`) |
| Directory permission was not granted at sign-in → directory suggest… | — | smoke depth — not tested yet (`DEC-65`) |
| Responding or editing guests without network → unavailable (`MOD-02… | — | smoke depth — not tested yet (`DEC-65`) |
| The organizer changes or cancels the event → the change arrives thr… | — | smoke depth — not tested yet (`DEC-65`) |
