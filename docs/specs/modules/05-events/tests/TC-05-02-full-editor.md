---
id: TC-05-02 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-02#2, US-05-02#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-02] The full editor saves title, description, times, location, busy/free and visibility to Google

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-02#2`, `US-05-02#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API and a fixed clock (Wednesday 2026-11-11 10:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected with an editable calendar "Personal", which is the default calendar.

## Test data

| What | Value / source |
| --- | --- |
| Fields | title "Review", description "Q4 numbers", 13 November 14:00–15:00 Europe/Warsaw, location "Room 2", free, private |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the full editor for a new event and fill in the fields from Test data. | Every field holds the entered value; the calendar list is grouped by account. | app state |
| 2 | Save. | The fake Google received the event with exactly these values. | app state |

## Pass criterion

- Every field set in the full editor reaches Google unchanged.

## Cleanup

- The fake Google and the local store are discarded with the test.
