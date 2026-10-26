---
id: MOD-01
title: Google accounts and sign-in
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: ready # draft | ready | in-progress | done
depends_on: [] # other module IDs
sources: [SRC-01, SRC-02] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-01] Google accounts and sign-in

## Context

Connecting and keeping Google accounts: signing in through the system browser, several personal and Workspace accounts at once, staying signed in and recovering from revoked access, removing an account together with its local data, and the first run. Answers for the PROJECT.md scope area "Google accounts and sign-in"; comes from discovery topic `DSC-03` (`discovery/03-accounts.md`), drawing on the PRD (`SRC-01`) and the analyst notes (`SRC-02`).

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-17` Authenticate through Google OAuth 2.0 Authorization Code Flow with PKCE using the user'… | informal | `MOD-01-F01` |
| `SRC-01-18` Support multiple Google accounts simultaneously, including personal and Workspace accou… | informal | `MOD-01-F02` |
| `SRC-01-19` Request the minimum scopes needed for calendar operations and contact lookup. | informal | `MOD-01-F03` |
| `SRC-01-20` Refresh credentials when possible and handle revoked access gracefully. | informal | `MOD-01-F04`, `MOD-01-F05` |
| `SRC-01-21` Store tokens in the operating system's credential store, not plaintext configuration fi… | informal | `NFR-04`, `MOD-01-F10` |
| `SRC-01-22` Allow users to add, remove, and reauthenticate accounts. | informal | `MOD-01-F06`, `MOD-01-F07` |
| `SRC-01-23` Clearly distinguish account-level errors from calendar-level synchronization errors. | informal | `MOD-01-F11` |
| `SRC-01-97` Google sign-in and multiple account support. | informal | `MOD-01-F01`, `MOD-01-F02` |
| `SRC-01-120` Connect at least one Google account without manually entering server settings. | informal | `MOD-01-F01` |
| `SRC-02-12` Supporting more than one account only needs the Google app switched to External and "In… | informal | `MOD-01-F02` |
| `SRC-02-24` First run with no account: a welcome screen with one action, connect a Google account;… | informal | `MOD-01-F12` |
| `SRC-02-25` Start without network: the calendar is shown from the cache with a visible "offline" st… | informal | `MOD-01-F13` |
| `SRC-02-26` Removing an account deletes at once its tokens, its cached calendars and events, and it… | informal | `MOD-01-F08` |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

- [MOD-01-F01] The user can connect a Google account by signing in through the system's default browser, without typing any server settings or password into the app.
- [MOD-01-F02] The user can connect more than one Google account; personal and Workspace accounts can be connected at the same time.
- [MOD-01-F03] Connecting an account asks Google only for the permissions the app's features need: calendars, contacts and, on Workspace accounts, the organization's directory.
- [MOD-01-F04] A connected account stays signed in across restarts of the app, without the user signing in again, for as long as Google keeps the access valid.
- [MOD-01-F05] When Google revokes or ends the app's access to an account, the app marks that account as needing sign-in and tells the user; the other accounts keep working.
- [MOD-01-F06] The user can sign in again to an account that needs it; its calendars and local settings are kept.
- [MOD-01-F07] The user can remove a connected account.
- [MOD-01-F08] Removing an account immediately deletes from the device its credentials, its cached calendars and events, and its local settings; the Google account itself is not changed. — *source: `DEC-20`*
- [MOD-01-F09] Removing an account also withdraws the app's access to that account at Google; if Google cannot be reached, the local deletion still happens. — *source: `DEC-52`*
- [MOD-01-F10] When the system has no running secret store, connecting an account stops and the app explains what to install or start. — *source: `DEC-47`*
- [MOD-01-F11] A problem with an account (signed out, access revoked) is shown on the account, separately from a problem synchronizing a single calendar.
- ~~[MOD-01-F12] On the first run, with no account connected, the user sees a welcome screen whose one action is connecting a Google account.~~ — changed by `DEC-179`, now `MOD-09-F17`, `MOD-09-F18`
- [MOD-01-F13] On a first run without network, the app says that signing in is not possible offline and lets the user try again once the network is back.
- ~~[MOD-01-F14] Removing the last connected account brings the user back to the welcome screen. — *source: `DEC-52`*~~ — changed by `DEC-179` and `DEC-185`, now `MOD-09-F21`

## UI/UX requirements

- [MOD-01-U01] Accounts are managed in one place in the app's settings, which lists every connected account by its e-mail address with its state (connected, needs sign-in) and offers adding, signing in again and removing. — *source: `DEC-51`*
- [MOD-01-U02] Removing an account asks for confirmation and says what will be deleted from the device and that the Google account itself stays untouched. — *source: `DEC-51`*
- ~~[MOD-01-U03] Before opening the browser, the app tells the user that Google will show an "unverified app" warning and how to continue past it. — **priority: should** — *source: `DEC-51`*~~ — withdrawn, `DEC-163`

## Non-functional requirements

- [MOD-01-NFR-01] Up to 5 accounts can be connected at the same time. — *source: `DEC-14`*

## Constraints and edge cases

- The user closes the browser or cancels the sign-in → no account is added, the app says the account was not connected, and the user can start again.
- The user unticks some permissions on Google's consent screen → the account is connected only if calendar access was granted; features that need a missing permission (contact or directory suggestions) are shown as unavailable for that account. — *source: `DEC-52`*
- The user signs in to an account that is already connected → no duplicate is created; the existing account is refreshed.
- A Workspace administrator blocks third-party apps, or the app has reached Google's 100-user cap → the sign-in fails with Google's reason, and the app says the account could not be connected.
- The system secret store is locked and the user declines to unlock it → the app shows its cached calendars but cannot synchronize or change anything until it is unlocked. — *source: `DEC-52`*
- No network while the app runs → connected accounts stay connected; signing in a new account is not possible until the network returns.
- The Google OAuth client file is missing or unreadable → connecting an account stops, and the app says which file to put where (`~/.config/calendar-buddy/google-oauth-client.json`). — *source: `DEC-77`*

## Integrations

- **Google OAuth 2.0** — authorization in the system browser and the token exchange; the app sends the authorization request, Google returns an authorization code to the app.
- **Google token service** — refreshing access for each connected account; revoking access when an account is removed.
- **Google user info** — the account's e-mail address and name, to identify and list the account.
- **Platform secret store** — on Linux the Secret Service API (KWallet, GNOME Keyring, KeePassXC); the app writes and deletes each account's credentials there.

## Technical notes

- OAuth 2.0 authorization code flow for installed apps with PKCE (S256), the system browser opened through the desktop's URL handler, and a loopback redirect to `127.0.0.1` on a random port.
- The OAuth client is an External app in production status, unverified, in the product owner's GCP project (`DEC-10`); the desktop client secret is not treated as confidential.
- The client ID and secret are read at runtime from `$XDG_CONFIG_HOME/calendar-buddy/google-oauth-client.json` (default `~/.config/…`), the JSON exactly as downloaded from Google Cloud Console (`installed.client_id`, `installed.client_secret`, `installed.redirect_uris`) — never from the repository, an environment file or the binary (`DEC-77`). Tests point the app at a test client file and the fake Google.
- Scopes: `openid`, `email`, `profile`, Google Calendar, Google contacts (including "other contacts"), and the directory scope on Workspace accounts — the exact set is fixed by the stories of `MOD-02`, `MOD-05` and `MOD-06` and kept minimal (`NFR-05`).
- Accounts are keyed by Google's stable account ID, not the e-mail address.
- The refresh token lives only in the secret store, behind the platform interface (constitution, `DEC-47`); access tokens are kept in memory only.
- An `invalid_grant` answer on refresh moves the account to "needs sign-in" (`MOD-01-F05`).
- The sign-in wait on the loopback port has a time-out after which the app stops listening and offers to retry.

## Out of scope / what not to worry about

- Providers other than Google (CalDAV, Microsoft 365) — `MOD-08`.
- Using the app without an account, with local calendars — `MOD-09` (`DEC-178`, `DEC-179`).
- Choosing permission scopes by hand — the app asks for the minimal set itself (`SRC-02-27`, `MOD-08`).
- A fallback credential store (encrypted file) when no secret store runs — `DEC-47`.
- Google OAuth verification and removing the "unverified app" warning — only before a public release (`DEC-10`).
- Colours or labels for accounts — colours are per calendar, in `MOD-02`.
- Fetching and synchronizing the account's calendars — `MOD-02`.
- Showing accounts and their calendars in the side panel — `MOD-03`.

## Acceptance criteria

- [ ] A personal and a Workspace Google account are connected at the same time through the browser, with no server settings typed in.
- [ ] After restarting the app, both accounts are still connected without opening the browser.
- [ ] After revoking the app's access in one Google account's security settings, that account shows "needs sign-in" while the other keeps working; signing in again restores it with its local settings.
- [ ] After removing an account, none of its calendars, events, credentials or local settings remain on the device, and the Google account still has all its data.
- [ ] A first run with no account shows the welcome screen; a first run without network says sign-in is not possible and offers a retry.
- [ ] On a system with no running secret store, connecting an account stops with an explanation of what to install or start.
