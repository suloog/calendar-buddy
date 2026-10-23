---
id: US-01-04 # US-NN-MM — NN is the parent module, MM a counter inside it; never reused
module: MOD-01
covers: [MOD-01-F07, MOD-01-F08, MOD-01-F09, MOD-01-F14, MOD-01-U02] # every requirement this story delivers
ticket: # optional YouTrack ID — filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | review | done | blocked
priority: must # must | should | nice-to-have — inherited from the requirement
depends_on: [US-01-02] # story IDs that must be done first
---

# As a User, I can remove an account, so that none of its data stays on my computer

|  |  |
| --- | --- |
| **Module** | `[MOD-01]` Google accounts and sign-in — `docs/specs/modules/01-accounts/REQUIREMENTS.md` |
| **Covers** | `MOD-01-F07`, `F08`, `F09`, `F14`, `U02` |
| **Priority** | must |
| **Status** | draft |
| **Depends on** | blocked by `US-01-02` |
| **YouTrack** | — |
| **Open questions** | — |

## Description

Removing a connected account from the settings: a confirmation that says what will be deleted, deleting everything the app holds for that account, withdrawing the app's access at Google, and returning to the welcome screen when it was the last account. The Google account itself is never changed.

## Acceptance criteria

- [ ] "Remove" asks for confirmation and says that the account's calendars, events, credentials and local settings will be deleted from this computer and that the Google account itself stays untouched.
- [ ] After removal, none of the account's calendars, events, credentials or local settings remain on the device.
- [ ] After removal, the app no longer appears in that Google account's list of third-party apps with access.
- [ ] Without network, removal still deletes everything locally.
- [ ] Removing the last connected account brings back the welcome screen.
- [ ] After removal, the Google account still has all its calendars and events.

## Out of scope / what not to worry about

- Deleting anything in the Google account — never done.

## Technical notes

- Revocation is best effort through Google's token revocation; the local deletion never waits for it (`DEC-52`).
- Decisions: `DEC-20`, `DEC-51`, `DEC-52`.
