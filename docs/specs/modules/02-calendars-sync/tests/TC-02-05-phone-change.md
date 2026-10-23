---
id: TC-02-05 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-02
covers: [US-02-03#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [TC-02-04] # test case IDs that must pass first
---

# [TC-02-05] A change made on the phone appears in the open app within about a minute

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-02-03#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | `TC-02-04` |

## Preconditions

- The product owner's account is connected and the app window is open on the week view.
- A test event "Manual sync check" exists in an editable calendar this week.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | On the phone, move "Manual sync check" by one hour in Google Calendar, and note the time. | The change is saved on the phone. | screenshot |
| 2 | Watch the app without touching it. | Within about a minute, the event shows at its new time. | screenshot |

## Pass criterion

- The change made on the phone appears in the open app within about a minute.

## Cleanup

- Delete the test event.
