---
id: TC-10-08 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-10
covers: [US-10-02#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-10-08] The box sits above "Meet with…", `q` focuses it opening a collapsed side panel, Shift+Enter breaks the line, Enter sends, Esc leaves, and `q` is in the shortcut list

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-10-02#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Empty config, cache and data directories.
- No Google account: the app was started with "Use without an account"; the local calendar "My calendar" is the default calendar.
- Events from a description on, with the default address `http://localhost:11434` and model `gemma4:e4b`; a fake Ollama at that address that has the model, records every request it receives and answers each description with the fixed answer given in Test data.
- A fixed clock: Monday 2026-10-26 09:00, Europe/Warsaw.
- The side panel is collapsed.

## Test data

| What | Value / source |
| --- | --- |
| Description | "Kino jutro", a new line, "o 20" |
| Fake model's answer | add · title "Kino" · tomorrow · 20:00 |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | In the week view press `q`. | The side panel opens; the field "Describe an event…" above "Meet with…" has the cursor. | app state |
| 2 | Type "Kino jutro", press Shift+Enter, type "o 20". | The field has grown to two lines; the fake Ollama has received no description. | app state + fake Ollama record |
| 3 | Press Esc, then the key for the next week. | The field no longer has the cursor and the week view moves to the next week. | app state |
| 4 | Press `q`, then Enter. | The fake Ollama receives one description, "Kino jutro" and "o 20" on two lines; the "New event" window opens. | fake Ollama record + app state |
| 5 | Close the window; open the list of keyboard shortcuts. | The list includes `q`. | app state |

## Pass criterion

- The box is where the story puts it and every key of `MOD-10-U01` and `MOD-10-U03` does what it says.

## Cleanup

- The fakes and the config, cache and data directories are discarded with the test.

## Out of scope for this test

- "In the same style" as "Meet with…", in the light and dark themes — the owner's manual check named in the story.
- The fake model's answers are fixed, so this test proves what the app does with an answer, not what a real model understands — that is `TC-10-19`.
