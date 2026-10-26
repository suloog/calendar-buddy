---
id: TC-09-05 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-09
covers: [US-09-02#1, US-09-02#2, US-09-02#4] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-09-05] Local calendars are added, renamed, recoloured and made the default in the settings

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-09-02#1`, `US-09-02#2`, `US-09-02#4` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | — |

## Preconditions

- Account A is connected; the local store holds "My calendar" with "Dentist" on 13 November; a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | In the settings' "Local calendars" section add "Family" and "Sport". | The section lists three calendars with their colours; the side panel's "Local" group, the calendar picker and the settings' list of calendars to disable show all three under "Local", after account A. | app state |
| 2 | Rename "My calendar" to "Private" and choose Basil as its colour. | The side panel, the picker and "Dentist" show "Private" in Basil at once. | app state |
| 3 | Choose "Sport" as the default calendar; click 18:00 on 13 November. | The quick box starts in "Sport". | app state |

## Pass criterion

- Any number of local calendars can be managed in the settings and behave as an account's calendars in the lists.

## Cleanup

- The fake Google, the cache and the local store are discarded with the test.
