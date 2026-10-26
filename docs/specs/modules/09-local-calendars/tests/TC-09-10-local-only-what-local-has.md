---
id: TC-09-10 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-03#5, US-09-03#6, US-09-03#7] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: negative # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-10] A local event offers nothing that only Google has, stays in its kind of calendar and never reaches Google

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-03#5`, `US-09-03#6`, `US-09-03#7` |
| **Role** | User |
| **Type** | negative |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Account A is connected with "Standup" on 13 November 09:00–09:15; the local store holds "Private" with "Dentist" on 13 November 10:00–11:00 and "Family"; a fake Google that records every request; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open "Dentist" in the editor. | No guests, Google Meet, attachments, event type, busy/free or visibility; its notifications offer no e-mail; the calendar picker lists only "Private" and "Family". | app state |
| 2 | Open "Standup" in the editor. | The calendar picker lists no local calendar. | app state |
| 3 | Move "Dentist" to "Family", drag it to 11:00, then delete it. | Each change is shown at once. | app state |
| 4 | Read the fake Google's record. | No request names "Dentist", "Private" or "Family", so Google cannot count "Dentist" among the user's busy times. | fake Google record |

## Pass criterion

- What only Google can do is not offered on local events, and nothing about them leaves the device.

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
