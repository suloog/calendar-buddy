<!-- Copy to docs/specs/modules/NN-<slug>/COVERAGE.md. Content in the project language; headings fixed.
     One file per module, two parts: requirement → user stories (written by /og:stories) and
     acceptance criterion → tests (written by /og:tests). The `og:part` markers stay — og-status reads them. -->

# Coverage — [MOD-01] Google accounts and sign-in

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
| `MOD-01-F01` | `US-01-01` |
| `MOD-01-F02` | `US-01-02` |
| `MOD-01-F03` | `US-01-01` |
| `MOD-01-F04` | `US-01-03` |
| `MOD-01-F05` | `US-01-03` |
| `MOD-01-F06` | `US-01-03` |
| `MOD-01-F07` | `US-01-04` |
| `MOD-01-F08` | `US-01-04` |
| `MOD-01-F09` | `US-01-04` |
| `MOD-01-F10` | `US-01-01` |
| `MOD-01-F11` | `US-01-03` |
| `MOD-01-F12` | `US-01-01` |
| `MOD-01-F13` | `US-01-01` |
| `MOD-01-F14` | `US-01-04` |

### UI/UX requirements

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-01-U01` | `US-01-02` |
| `MOD-01-U02` | `US-01-04` |
| ~~`MOD-01-U03`~~ | withdrawn, `DEC-163` |

### Non-functional requirements

Module NFRs only; project-wide `NFR-nn` are delivered across modules and are not tracked per module.

| Requirement | Covered by |
| ----------- | ---------- |
| `MOD-01-NFR-01` | `US-01-02` |

<!-- og:part tests -->

## Part 2 — Acceptance criteria → tests

### User story acceptance criteria

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-01-01#1` | first run with network | `TC-01-01` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-01-01#2` | Before the browser opens, the app explains that Google will show an… | `TC-01-01` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-01-01#3` | After consenting in the browser, the account appears in the app wit… | `TC-01-01` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-01-01#4` | Google's consent screen asks only for calendar, contacts and — on W… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-01#5` | first run without network | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-01#6` | no secret store | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-02#1` | From the settings, the user connects a second account (personal or… | `TC-01-02` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-01-02#2` | The settings list every connected account by its e-mail address wit… | `TC-01-02` | manual-only | manual — checked by the product owner (`DEC-65`) |
| `US-01-02#3` | Five accounts can be connected at the same time. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-03#1` | After restarting the app, connected accounts work without opening t… | `TC-01-03` | auto | — |
| `US-01-03#2` | access revoked for one account | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-03#3` | "Sign in again" on that account restores it, with its calendars and… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-03#4` | A problem with an account is shown on the account, separately from… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-04#1` | "Remove" asks for confirmation and says that the account's calendar… | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-04#2` | After removal, none of the account's calendars, events, credentials… | `TC-01-04` | auto | — |
| `US-01-04#3` | After removal, the app no longer appears in that Google account's l… | `TC-01-04` | auto | — |
| `US-01-04#4` | Without network, removal still deletes everything locally. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-04#5` | Removing the last connected account brings back the welcome screen. | — | — | smoke depth — not tested yet (`DEC-65`) |
| `US-01-04#6` | After removal, the Google account still has all its calendars and e… | — | — | smoke depth — not tested yet (`DEC-65`) |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| The user closes the browser or cancels the sign-in → no account is… | — | smoke depth — not tested yet (`DEC-65`) |
| The user unticks some permissions on Google's consent screen → the… | — | smoke depth — not tested yet (`DEC-65`) |
| The user signs in to an account that is already connected → no dupl… | — | smoke depth — not tested yet (`DEC-65`) |
| A Workspace administrator blocks third-party apps, or the app has r… | — | smoke depth — not tested yet (`DEC-65`) |
| The system secret store is locked and the user declines to unlock i… | — | smoke depth — not tested yet (`DEC-65`) |
| No network while the app runs → connected accounts stay connected;… | — | smoke depth — not tested yet (`DEC-65`) |
