---
id: TC-06-13 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-01#6] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-06-13] A guest invited from the app receives Google's invitation e-mail

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-01#6` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The product owner's account is connected; a second mailbox the product owner can read is available as the guest.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | In the app, create "Invite check" tomorrow at 12:00 with the second mailbox as a guest, and save. | The event is saved with the guest. | screenshot |
| 2 | Open the second mailbox. | Google's invitation e-mail for "Invite check" has arrived. | screenshot |

## Pass criterion

- The guest receives the invitation e-mail sent by Google.

## Cleanup

- Delete "Invite check" (answering "Yes" to notify the guest, or "No").
