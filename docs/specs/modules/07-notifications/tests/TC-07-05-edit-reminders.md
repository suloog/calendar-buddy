---
id: TC-07-05 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-04#1, US-07-04#2, US-07-04#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-05] A new event gets its calendar's default reminders, an added reminder reaches Google, and "Use calendar defaults" resets

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-04#1`, `US-07-04#2`, `US-07-04#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API, a fake desktop notification service that records what is shown, and a fixed, advanceable clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A is connected with the editable calendar "Work".
- "Work" has the default reminder "notification, 15 minutes before" in the fake Google.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Create an event "Plan" in "Work". | The event uses the calendar's defaults: one notification 15 minutes before. | app state |
| 2 | In the editor, add an e-mail reminder 1 day before and save. | The fake Google holds "Plan" with a notification 15 minutes before and an e-mail 1 day before. | app state |
| 3 | Choose "Use calendar defaults" and save. | The fake Google holds "Plan" back on the calendar defaults. | app state |

## Pass criterion

- Reminders start from the calendar defaults, edits reach Google, and the defaults can be restored.

## Cleanup

- The fakes and the local store are discarded with the test.
