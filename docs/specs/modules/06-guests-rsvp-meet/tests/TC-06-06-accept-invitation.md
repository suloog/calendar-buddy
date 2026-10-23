---
id: TC-06-06 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-06
covers: [US-06-05#1, US-06-05#2] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: auto # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [TC-06-05] # test case IDs that must pass first
---

# [TC-06-06] Accepting an invitation from the list sends the answer to Google

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-06-05#1`, `US-06-05#2` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | auto |
| **Depends on** | `TC-06-05` |

## Preconditions

- A fake Google API (calendar, contacts, directory, free/busy) that records what it receives, and a fixed clock (Thursday 2026-11-12 09:00, Europe/Warsaw), set up by the test.
- Account A (personal) is connected; its contacts include "Adam Kowalski" (adam@example.com); "ewa@example.com" was invited to an earlier event.
- The invitation "Board" awaits the user's answer.

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Open the invitations list and choose Accept on "Board". | Accept, Maybe and Decline were offered; the counter drops to the remaining invitations. | app state |
| 2 | Check what the fake Google received. | The user's answer to "Board" is accepted. | app state |

## Pass criterion

- An answer given in the app reaches Google.

## Cleanup

- The fake Google and the local store are discarded with the test.
