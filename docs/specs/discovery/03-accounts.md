---
id: DSC-03
title: Google accounts and sign-in
kind: business # business (a functional area — becomes a module at /og:modules) | cross-cutting (roles and access, technology and integrations, non-functional requirements — feeds PROJECT.md, NFR.md and the constitution, never a module)
status: settled # open (not yet discussed) | discussed (discussed with the analyst, waiting on the client) | settled (no [W] question in this topic is open)
sources: [SRC-01, SRC-02] # SRC-NN documents whose items this topic gathers, e.g. [SRC-01, SRC-03]
module: MOD-01 # MOD-NN — filled by /og:modules; empty for cross-cutting topics
sessions: [2026-10-22] # dates of the discovery sessions in which the topic was discussed, YYYY-MM-DD
---

<!-- Copy to docs/specs/discovery/NN-<topic>.md. Content in the project language; headings fixed.
     This file records what we UNDERSTOOD about this topic after reading every source and talking
     to the analyst — the layer between the sources (what the client wrote) and the requirements
     (what we will build). It is alive: after every client workshop /og:discovery updates the
     findings. A finding is NOT a requirement: no acceptance criteria, no priority, no UI; one
     finding becomes 1–3 requirements in the module's REQUIREMENTS.md later. -->

# [DSC-03] Google accounts and sign-in

## Topic scope

Connecting Google accounts (several at once, personal and Workspace), signing in through the browser, keeping credentials safe, refreshing and re-authenticating, removing an account, and telling account problems apart from sync problems. Ends where calendars are fetched (DSC-04).

## Sources in this topic

<Items from ALL sources that speak about this topic, side by side — so a contradiction between the
contract and the kick-off note is visible in one place, not in two files.>

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-01-17` | informal | Authenticate through Google OAuth 2.0 Authorization Code Flow with PKCE using the user's default browser. |
| `SRC-01-18` | informal | Support multiple Google accounts simultaneously, including personal and Workspace accounts where permitted. |
| `SRC-01-19` | informal | Request the minimum scopes needed for calendar operations and contact lookup. |
| `SRC-01-20` | informal | Refresh credentials when possible and handle revoked access gracefully. |
| `SRC-01-21` | informal | Store tokens in the operating system's credential store, not plaintext configuration files. |
| `SRC-01-22` | informal | Allow users to add, remove, and reauthenticate accounts. |
| `SRC-01-23` | informal | Clearly distinguish account-level errors from calendar-level synchronization errors. |
| `SRC-01-97` | informal | Google sign-in and multiple account support. |
| `SRC-01-120` | informal | Connect at least one Google account without manually entering server settings. |
| `SRC-02-12` | informal | External app in production; personal and org accounts, several at once. |
| `SRC-02-24` | informal | First run: welcome screen → connect a Google account. |
| `SRC-02-25` | informal | No network at first run: message and retry; later starts: cache + offline status. |
| `SRC-02-26` | informal | Removing an account deletes its tokens, cache and local settings. |
| `SRC-02-27` | informal | Parked: non-Google providers; user-chosen scopes. |

## Key findings

- [DSC-03-01] The user connects a Google account by signing in through their own browser; nothing — no server, no password — is typed into the app. (`SRC-01-17`, `SRC-01-120`)
- [DSC-03-02] Several Google accounts, personal and organization, can be connected at once. (`SRC-01-18`, `SRC-01-97`, `SRC-02-12`) — *source: `DEC-10`*
- [DSC-03-03] Once signed in, the user stays signed in — the app refreshes access on its own; if Google revokes it, the app says so plainly and offers to sign in again instead of silently stopping. (`SRC-01-20`)
- [DSC-03-04] The user can add an account, remove it, and sign in again to an existing one. (`SRC-01-22`)
- [DSC-03-05] A problem with an account (signed out, access revoked) is shown separately from a problem synchronizing a single calendar. (`SRC-01-23`)
- [DSC-03-06] On first run with no account the user sees a welcome screen with one action — connect a Google account; the calendar appears after the first sync. (`SRC-02-24`) — *source: `DEC-19`*
- [DSC-03-07] With no network on first run, signing in is impossible; the app says so and lets the user retry once the network is back. (`SRC-02-25`) — *source: `DEC-19`*
- [DSC-03-08] Removing an account immediately deletes its tokens, its cached calendars and events, and its local settings from the device; the Google account itself is untouched. (`SRC-02-26`) — *source: `DEC-20`*

## Contradictions

- `SRC-01-18` (personal and Workspace accounts) was narrowed by `DEC-04` to organization accounts, then restored by `DEC-10` — no contradiction remains.

## Ambiguities

- Token storage and minimal scopes (`SRC-01-19`, `SRC-01-21`) also appear in `SRC-01-92`; they are recorded once, in `DSC-02-05`, and not repeated here.

## From the calls, not the documents

- `SRC-02-24`, `SRC-02-25` first run and start without network (answers to `Q24`)
- `SRC-02-26` account removal purges local data
- `SRC-02-22` local calendar colours and `SRC-02-23` default calendar for new events — said in this session, but they belong to `DSC-04` and `DSC-07` and become findings there

## For the client workshop

None open — `Q24` resolved on 2026-10-22 (`DEC-19`); its calendar-access part moved to `Q36` in `DSC-04`.

## Parked

- Sign-in to providers other than Google — later provider adapters (`SRC-01-109`, `SRC-01-110`, `SRC-02-27`).
- The user choosing permission scopes by hand — the app asks for the minimum set itself (`SRC-02-27`).
