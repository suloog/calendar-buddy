---
id: US-01-03 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-01
covers: [MOD-01-F04, MOD-01-F05, MOD-01-F06, MOD-01-F11] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-01-01] # story IDs that must be done first
---

# As a User, I stay signed in and can sign in again when Google revokes access, so that I don't have to log in every time and a broken account doesn't stop the others

|  |  |
| --- | --- |
| **Module** | `[MOD-01]` Google accounts and sign-in — `docs/specs/modules/01-accounts/REQUIREMENTS.md` |
| **Covers** | `MOD-01-F04`, `F05`, `F06`, `F11` |
| **Priority** | must |
| **Status** | ready |
| **Depends on** | blocked by `US-01-01` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Keeping accounts signed in across restarts, noticing when Google revokes or ends access to one account, showing that on the account itself, and signing in to it again without losing its calendars or local settings.

## Acceptance criteria

- [ ] After restarting the app, connected accounts work without opening the browser.

### Scenario: access revoked for one account

- **Given** two accounts are connected
- **When** the app's access is revoked in one account's Google security settings
- **Then** that account is marked "needs sign-in" and the user is told
- **And** the other account keeps working

- [ ] "Sign in again" on that account restores it, with its calendars and local settings kept.
- [ ] A problem with an account is shown on the account, separately from a synchronization error of a single calendar.

## Edge cases

- The secret store is locked and the user declines to unlock it → cached calendars are shown, but nothing synchronizes or changes until it is unlocked.
- No network while the app runs → connected accounts stay connected; signing in again waits for the network.

## Out of scope / what not to worry about

- Per-calendar synchronization errors themselves — `MOD-02`.

## Technical notes

- An `invalid_grant` answer on token refresh moves the account to "needs sign-in" (`MOD-01` *Technical notes*).
- Decision: `DEC-52` (locked secret store).
