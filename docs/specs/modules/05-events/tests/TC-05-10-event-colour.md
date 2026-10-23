---
id: TC-05-10 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-09#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-10] Changing an event's colour in the editor sends the new colour to Google

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-09#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.
- An event "Dentist" on 12 November 16:00–17:00 without its own colour.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open "Dentist" in the editor, choose a different event colour and save. | The fake Google received the event with that colour, and the app shows it in that colour. | app state |

## Pass criterion

- The event's own colour is changed in Google from the editor.

## Cleanup

- The fake Google and the local store are discarded with the test.
