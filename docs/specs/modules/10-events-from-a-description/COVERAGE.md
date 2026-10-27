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

`#n` is the criterion's position in the story file, counted from 1, with a Given/When/Then scenario counting as one.

| Criterion | Text (short) | Covered by | Automation | Note |
| --------- | ------------ | ---------- | ---------- | ---- |
| `US-10-01#1` | The settings have an "Events from a description" section with an on/of… | `TC-10-01` | auto | — |
| `US-10-01#2` | With the checkbox unticked, the description box, its `q` key and its e… | `TC-10-02` | auto | — |
| `US-10-01#3` | An address that is not this computer (not `localhost`, `127.0.0.1` or… | `TC-10-03` | auto | — |
| `US-10-01#4` | With the default address, the app sends no request to anything but Go… | `TC-10-04` | auto | — |
| `US-10-01#5` | The README has a section on installing Ollama and pulling `gemma4:e4b`… | `TC-10-05` | manual-only | — |
| `US-10-01#6` | Scenario: Ollama is not running — the box is greyed out and says "Olla… | `TC-10-06` | auto | — |
| `US-10-01#7` | Scenario: the model is missing — the box is greyed out and says "The m… | `TC-10-06` | auto | — |
| `US-10-01#8` | Scenario: Ollama starts later — the box is available without restarti… | `TC-10-07` | auto | — |
| `US-10-02#1` | The side panel has a field "Describe an event…" above "Meet with…"; `q`… | `TC-10-08` | auto | — |
| `US-10-02#2` | Focusing the box has Ollama load the model; the model is freed 5 minut… | `TC-10-09` | auto | — |
| `US-10-02#3` | While the model reads, the box shows "Reading…" and the rest of the ap… | `TC-10-10` | auto | — |
| `US-10-02#4` | A length in the description is used; with none, the event gets the de… | `TC-10-12` | auto | — |
| `US-10-02#5` | Location, notes, repeating, reminders and colour stay as for any new e… | `TC-10-11` | auto | — |
| `US-10-02#6` | Descriptions and the model's answers appear in no log and no file; a d… | `TC-10-15`, `TC-10-16` | auto | — |
| `US-10-02#7` | Offline, only local calendars can take the event, as when adding by ha… | `TC-10-17` | auto | — |
| `US-10-02#8` | The fixed test set of adding sentences passes with the date right in a… | `TC-10-18`, `TC-10-19` | auto + manual-only | the fake model proves the set and the app; the real model only by hand (`TC-10-19`) |
| `US-10-02#9` | Scenario: a sentence with a part of the day — "Trening wieczorem" open… | `TC-10-11` | auto | — |
| `US-10-02#10` | Scenario: days and times worked out by the app | `TC-10-13` | auto | — |
| `US-10-02#11` | Scenario: not an event — no window opens; the box says so and keeps th… | `TC-10-14` | auto | — |
| `US-10-03#1` | Names are looked up as when typing a guest — contacts, directory, rece… | `TC-10-21` | auto | — |
| `US-10-03#2` | A description with guests or Meet opens the full editor filled in; one… | `TC-10-20`, `TC-10-11` | auto | — |
| `US-10-03#3` | Meet is on for "z Meetem", off for "bez Meeta", and as for any new eve… | `TC-10-23` | auto | — |
| `US-10-03#4` | For a local calendar, guests and Meet are left out and the editor open… | `TC-10-24` | auto | — |
| `US-10-03#5` | Offline, or with no Google account, guests are not looked up and the… | `TC-10-25` | auto | — |
| `US-10-03#6` | Scenario: two guests and Meet | `TC-10-20` | auto | — |
| `US-10-03#7` | Scenario: a name that is not certain | `TC-10-22` | auto | — |
| `US-10-04#1` | Adding and changing are told apart by the description; a sentence tha… | `TC-10-27` | auto | — |
| `US-10-04#2` | Only events from today to the end of the cache are matched by name; a… | `TC-10-28` | auto | — |
| `US-10-04#3` | Saving follows the usual rules: series scope, notifying guests, permis… | `TC-10-30`, `TC-10-31` | auto | — |
| `US-10-04#4` | The test set gains at least 10 change sentences; with them it still p… | `TC-10-18`, `TC-10-19` | auto + manual-only | as `US-10-02#8` |
| `US-10-04#5` | Scenario: the event whose details are open | `TC-10-26` | auto | — |
| `US-10-04#6` | Scenario: an event named in the description | `TC-10-27` | auto | — |
| `US-10-04#7` | Scenario: several events match | `TC-10-29` | auto | — |
| `US-10-04#8` | Scenario: nothing matches | `TC-10-28` | auto | — |
| `US-10-05#1` | A day or time given in the description is filled in as in `US-10-02`;… | `TC-10-32`, `TC-10-33` | auto | — |
| `US-10-05#2` | A calendar is matched by its name as shown in the side panel, ignorin… | `TC-10-34` | auto | — |
| `US-10-05#3` | Scenario: no time | `TC-10-32` | auto | — |
| `US-10-05#4` | Scenario: no date | `TC-10-33` | auto | — |
| `US-10-05#5` | Scenario: a named calendar | `TC-10-34`, `TC-10-35` | auto | — |

### Module constraints and edge cases

| Constraint | Covered by | Note |
| ---------- | ---------- | ---- |
| A description in another language, or mixed → read as well as the model ca… | — | model quality, not the app's — smoke depth, not tested yet (`DEC-65`) |
| A new event described on a past day ("wczoraj") → the window opens on that… | `TC-10-13` | — |
| A change described while the selected event is read-only, or the user is a… | `TC-10-31` | — |
| A change of one occurrence of a series → saving asks for the scope… | `TC-10-30` | — |
| The model's answer arrives after Esc → it is dropped. | `TC-10-10` | — |
| Ollama stops while the box is open → the request fails and the box turns u… | `TC-10-07` | — |
| No account connected → no guests or Meet are looked up; local calendars onl… | `TC-10-25` | — |
| Every calendar disabled, or none can take events → the message of `MOD-09`… | — | smoke depth — not tested yet (`DEC-65`) |
| A description longer than 500 characters → the box takes it, only the firs… | `TC-10-16` | — |

### Non-functional requirements under test

| Requirement | Covered by | How | Note |
| ----------- | ---------- | --- | ---- |
| `MOD-10-NFR-01` | `TC-10-15` | a marker word in the description searched for in the logs, the captured output and the app's directories | — |
| `MOD-10-NFR-02` | `TC-10-04` | the record of every request the app makes, and the fake Google's record | — |
| `MOD-10-NFR-03` | `TC-10-10` | the views move while the fake model holds its answer; no time is measured | no response-time target by the requirement itself |
| `MOD-10-NFR-04` | `TC-10-18`, `TC-10-19` | the test set runner's shares of sentences with the date right and every field right | the thresholds are proven only by `TC-10-19`, by hand against a real Ollama |
