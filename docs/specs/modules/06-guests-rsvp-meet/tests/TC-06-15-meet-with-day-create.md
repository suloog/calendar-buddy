---
id: TC-06-15 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-09#4, US-06-09#5] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-15] With people chosen, the day view shows columns side by side and Create event invites them

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-09#4`, `US-06-09#5` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account B (Workspace) is connected and has "Weekly sync" on Friday 13 November 14:00–15:00; in the fake Google, Anna Nowak shares her calendar with details and has "Budget review" that day 14:30–15:30; Adam Kowalski shares only free/busy and is busy that day 10:00–11:00; the account's default event length is 30 minutes.
- Anna and Adam are chosen in "Meet with…".

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the day view of Friday 13 November. | Three columns side by side, headed by the user, Anna Nowak and Adam Kowalski: "Weekly sync" in the user's, "Budget review" in Anna's, a "Busy" block 10:00–11:00 in Adam's. | app state |
| 2 | Choose **Create event** under the list. | The full editor opens with Anna's and Adam's addresses as guests, on 13 November at the next full hour (10:00–10:30 with the default length of 30 minutes), in the default calendar. | app state |

## Pass criterion

- The day view puts each chosen person in a column of their own, and Create event starts a meeting with them.

## Cleanup

- The fake Google and the local store are discarded with the test.
