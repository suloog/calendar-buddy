---
id: TC-06-14 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-09#1, US-06-09#2, US-06-09#3, US-06-09#6] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-14] "Meet with…" lays colleagues' calendars over the week view, with details, busy blocks and a calendar that cannot be seen

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-09#1`, `US-06-09#2`, `US-06-09#3`, `US-06-09#6` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account B (Workspace) is connected and has "Weekly sync" on Friday 13 November 14:00–15:00.
- In the fake Google: Anna Nowak shares her calendar with details and has "Budget review" on 13 November 14:30–15:30 and "Planning" on Tuesday 17 November 10:00–11:00; Adam Kowalski shares only free/busy and is busy on 12 November 11:00–12:00; Ewa Zielińska shares nothing.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | In the week of 9 November, type "Ann" in "Meet with…". | Anna Nowak is suggested. | app state |
| 2 | Choose Anna, then Adam, then type Ewa's address and choose it. | The list under the field shows the three people, each with a colour of their own and a way to remove them. | app state |
| 3 | Look at Friday 13 November. | "Budget review" 14:30–15:30 in Anna's colour stands beside "Weekly sync", like overlapping events. | app state |
| 4 | Look at Thursday 12 November. | A "Busy" block 11:00–12:00 in Adam's colour, without a title. | app state |
| 5 | Read Ewa's entry in the list. | Ewa is marked as one whose calendar cannot be seen; no events of hers are shown. | app state |
| 6 | Open "Budget review". | Its details open read-only: no Edit, no Delete. | app state |
| 7 | Go to the next week. | "Planning" on Tuesday 17 November 10:00–11:00 in Anna's colour; the fake Google received a request for that week. | app state, fake Google record |

## Pass criterion

- The chosen people's events appear in the week view in their colours, with details or as busy blocks as their calendars allow, and follow the browsed week.

## Cleanup

- The fake Google and the local store are discarded with the test.
