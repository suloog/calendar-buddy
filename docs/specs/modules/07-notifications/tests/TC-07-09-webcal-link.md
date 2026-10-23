---
id: TC-07-09 # TC-NN-MM — NN is the parent module, MM a counter within it; never reused
module: MOD-07
covers: [US-07-06#1] # what this test proves — `#n` is the acceptance criterion's number in the story
layer: desktop # desktop — the project's own layer (`DEC-65`)
type: happy-path # happy-path | edge | negative | nfr
automation: manual-only # auto | manual-only — manual-only is never green in the report
roles: [User] # actors from PROJECT.md whose roles the test runs as
status: ready # draft | ready
depends_on: [] # test case IDs that must pass first
---

# [TC-07-09] A `webcal:` link opened in another app opens Google Calendar's "add calendar from URL" page

|  |  |
| --- | --- |
| **Layer** | desktop |
| **Verifies** | `US-07-06#1` |
| **Role** | User |
| **Type** | happy-path |
| **Automation** | manual-only |
| **Depends on** | — |

## Preconditions

- The app is installed from its .deb package on KDE Plasma.

## Test data

| What | Value / source |
| --- | --- |
| Link | a public `webcal:` calendar link, e.g. a national holidays feed |

## Steps

| # | Action | Expected result | Evidence |
| - | ------ | --------------- | -------- |
| 1 | Click the `webcal:` link in another application (e.g. a text editor or e-mail). | Google Calendar opens in the browser on the page for adding a calendar from a URL, with the link's address filled in. | screenshot |

## Pass criterion

- A `webcal:` link leads to Google Calendar's "add calendar from URL" page with that address.

## Cleanup

- Do not add the calendar, or remove it afterwards in the browser.
