---
id: TC-05-11 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-05
covers: [US-05-10#1, US-05-10#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: draft # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-05-11] An event's attachments are listed and open in the browser

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-05-10#1`, `US-05-10#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The product owner's account has an event this week with a Google Drive document attached (added in the browser).

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the event in the app. | Its details list the attachment by name. | screenshot |
| 2 | Choose the attachment. | The document opens in the browser. | screenshot |

## Pass criterion

- The event's attachment is listed and opens in the browser.

## Cleanup

- Delete the test event.
