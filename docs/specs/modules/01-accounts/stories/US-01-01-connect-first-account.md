---
id: US-01-01 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-01
covers: [MOD-01-F01, MOD-01-F03, MOD-01-F10, MOD-01-F12, MOD-01-F13, MOD-01-U03] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: done # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [] # story IDs that must be done first
---

# As a User, I can connect my first Google account from the welcome screen, so that the app can show my calendars

|  |  |
| --- | --- |
| **Module** | `[MOD-01]` Google accounts and sign-in — `docs/specs/modules/01-accounts/REQUIREMENTS.md` |
| **Covers** | `MOD-01-F01`, `F03`, `F10`, `F12`, `F13`, `U03` |
| **Priority** | must |
| **Status** | done |
| **Depends on** | — |
| **YouTrack** | — |
| **Open questions** | — |

## Description

The first-run path: the welcome screen, signing in to one Google account in the browser, and coming back to the app with the account connected. It also covers the two first-run failures — no network and no secret store. Adding further accounts and the account list in the settings are `US-01-02`; staying signed in across restarts is `US-01-03`.

## Acceptance criteria

### Scenario: first run with network

- **Given** no account is connected
- **When** the app starts
- **Then** a welcome screen shows one action, "Connect Google account"

- ~~Before the browser opens, the app explains that Google will show an "unverified app" screen and how to continue past it.~~ — withdrawn, `DEC-163`: once the checks pass the browser opens.
- [ ] After consenting in the browser, the account appears in the app without the user typing any server setting or password, and the welcome screen gives way to the calendar.
- [ ] Google's consent screen asks only for calendar, contacts and — on Workspace accounts — directory permissions.

### Scenario: first run without network

- **Given** no account is connected and there is no network
- **When** the user chooses "Connect Google account"
- **Then** the app says signing in is not possible offline
- **And** "Try again" starts the sign-in once the network is back

### Scenario: no secret store

- **Given** the system has no running secret store
- **When** the user chooses "Connect Google account"
- **Then** connecting stops and the app explains what to install or start

## Edge cases

- The user cancels the sign-in or closes the browser → no account is added; the app says so and offers to start again.
- The user unticks calendar access on the consent screen → the account is not connected, and the app says why.
- The user unticks contacts or directory access → the account is connected, and the features needing them are shown as unavailable for that account.
- A Workspace administrator blocks third-party apps, or Google's 100-user cap is reached → the app shows Google's reason and that the account was not connected.
- The Google OAuth client file is missing or unreadable → connecting stops, and the app says which file to put where (`DEC-77`).

## Out of scope / what not to worry about

- Connecting a second account and the account list in the settings — `US-01-02`.
- Staying signed in after a restart and recovering revoked access — `US-01-03`.
- Fetching the account's calendars — `MOD-02`.

## Technical notes

- See *Technical notes* of `MOD-01`: OAuth installed-app flow with PKCE, loopback redirect, scopes, tokens in the secret store (`NFR-04`, `DEC-47`, `DEC-10`).
- Decisions: `DEC-19` (first run), `DEC-51` (unverified-app heads-up, withdrawn by `DEC-163`), `DEC-52` (partial consent), `DEC-77` (where the OAuth client file lives).
