---
id: US-01-02 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-01
covers: [MOD-01-F02, MOD-01-U01, MOD-01-NFR-01] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-01-01] # story IDs that must be done first
---

# As a User, I can connect more Google accounts and see them all in the settings, so that my personal and work calendars are in one place

|  |  |
| --- | --- |
| **Module** | `[MOD-01]` Google accounts and sign-in — `docs/specs/modules/01-accounts/REQUIREMENTS.md` |
| **Covers** | `MOD-01-F02`, `U01`, `NFR-01` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-01-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Adding a second and further Google accounts — personal and Workspace side by side — and the account list in the settings where every account is shown with its state. The actions on the list (sign in again, remove) are delivered by `US-01-03` and `US-01-04`; this story provides the list and "add".

## Acceptance criteria

- [ ] From the settings, the user connects a second account (personal or Workspace), and both accounts work at the same time.
- [ ] The settings list every connected account by its e-mail address with its state — connected or needs sign-in — and offer adding, signing in again and removing.
- [ ] Five accounts can be connected at the same time.

## Edge cases

- The user signs in to an account that is already connected → no duplicate is created; the existing account is refreshed.

## Out of scope / what not to worry about

- What "sign in again" does — `US-01-03`; what "remove" does — `US-01-04`.
- Showing accounts and their calendars in the side panel — `MOD-03`.

## Technical notes

- Accounts are keyed by Google's stable account ID (`MOD-01` *Technical notes*).
- Decision: `DEC-51` (account management in the settings); scale profile `DEC-14`.
