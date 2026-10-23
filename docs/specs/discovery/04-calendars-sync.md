---
id: DSC-04
title: Calendars, synchronisation and offline cache
kind: business # business (a functional area — becomes a module at /og:modules) | cross-cutting (roles and access, technology and integrations, non-functional requirements — feeds PROJECT.md, NFR.md and the constitution, never a module)
status: settled # open (not yet discussed) | discussed (discussed with the analyst, waiting on the client) | settled (no [W] question in this topic is open)
sources: [SRC-01, SRC-02, SRC-03] # SRC-NN documents whose items this topic gathers, e.g. [SRC-01, SRC-03]
module: MOD-02 # MOD-NN — filled by /og:modules; empty for cross-cutting topics
sessions: [2026-10-22] # dates of the discovery sessions in which the topic was discussed, YYYY-MM-DD
---

<!-- Copy to docs/specs/discovery/NN-<topic>.md. Content in the project language; headings fixed.
     This file records what we UNDERSTOOD about this topic after reading every source and talking
     to the analyst — the layer between the sources (what the client wrote) and the requirements
     (what we will build). It is alive: after every client workshop /og:discovery updates the
     findings. A finding is NOT a requirement: no acceptance criteria, no priority, no UI; one
     finding becomes 1–3 requirements in the module's REQUIREMENTS.md later. -->

# [DSC-04] Calendars, synchronisation and offline cache

## Topic scope

Which calendars appear and how (all accessible calendars, unified across accounts, visibility and colours), keeping them in sync automatically and on demand, the local cache, offline browsing and recovery from errors, and the sync state shown to the user. Ends where events are drawn (DSC-05) or changed (DSC-07).

## Sources in this topic

<Items from ALL sources that speak about this topic, side by side — so a contradiction between the
contract and the kick-off note is visible in one place, not in two files.>

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-01-11` | informal | Local-first: cache calendar data locally for fast access and offline browsing. |
| `SRC-01-25` | informal | Retrieve all accessible calendars from connected Google accounts. |
| `SRC-01-26` | informal | Show calendars from multiple accounts in unified views. |
| `SRC-01-27` | informal | Let users toggle calendar visibility and preserve provider-defined colors. |
| `SRC-01-28` | informal | Synchronize changes automatically and offer manual refresh. |
| `SRC-01-29` | informal | Use incremental synchronization (`syncToken`) where supported by the Google Calendar API. |
| `SRC-01-30` | informal | Cache events and relevant calendar metadata in a local SQLite database. |
| `SRC-01-31` | informal | Support offline browsing of previously synchronized events. |
| `SRC-01-32` | informal | Recover from connectivity loss, expired synchronization tokens, rate limits, and transient API errors. |
| `SRC-01-33` | informal | Represent the synchronization state clearly without blocking the user interface. |
| `SRC-01-34` | informal | Handle time zones, daylight-saving changes, and recurring-event exceptions correctly. |
| `SRC-01-35` | informal | MVP offline boundary: offline browsing is included; offline creation/editing with a durable outgoing operat… |
| `SRC-01-98` | informal | Multiple calendars displayed in a unified interface. |
| `SRC-01-106` | informal | Local synchronization cache and offline browsing. |
| `SRC-01-111` | informal | Full offline editing with conflict resolution and queued writes. |
| `SRC-01-118` | informal | Calendar sharing and administrative management. |
| `SRC-01-121` | informal | See events from selected calendars across accounts. |
| `SRC-01-128` | informal | View cached events when temporarily offline. |
| `SRC-02-22` | informal | Calendar colour overridable locally, not written back to Google. |
| `SRC-02-25` | informal | Start without network: cache + offline status. |
| `SRC-02-26` | informal | Removing an account purges its cached data. |
| `SRC-02-28` | informal | Existing calendars only; management in the browser. |
| `SRC-02-29` | informal | Read-only/subscribed shown, not editable; free/busy as blocks. |
| `SRC-02-30` | informal | Cache window −1 year / +2 years. |
| `SRC-02-31` | informal | Calendar losing access: removed, user told once. |
| `SRC-02-32` | informal | Offline = browsing only. |
| `SRC-02-33` | informal | Changes from elsewhere within ~1 minute. |
| `SRC-03-15` | informal | Adaptive sync: editable ~1 min, read-only ~15 min, sync on focus. |

## Key findings

- [DSC-04-01] The app fetches every calendar the connected accounts can access. (`SRC-01-25`)
- [DSC-04-02] Calendars from all accounts are shown together, in one view. (`SRC-01-26`, `SRC-01-98`, `SRC-01-121`)
- [DSC-04-03] Each calendar can be shown or hidden; it takes its colour from Google by default, and the user can change it locally without changing it in Google. (`SRC-01-27`, `SRC-02-22`)
- [DSC-04-04] Changes synchronize on their own in the background, and the user can also refresh by hand; each time, only what changed is fetched where Google allows it. (`SRC-01-28`, `SRC-01-29`)
- [DSC-04-05] Calendar data is kept locally, so the app opens fast and previously synchronized events can be browsed without network. (`SRC-01-11`, `SRC-01-30`, `SRC-01-31`, `SRC-01-106`, `SRC-01-128`)
- [DSC-04-06] Synchronization recovers on its own from lost network, expired sync state, Google's limits and transient errors. (`SRC-01-32`)
- [DSC-04-07] The synchronization state is visible and never blocks work; after a start without network the app shows the calendar from the cache with a clear "offline" status. (`SRC-01-33`, `SRC-02-25`) — *source: `DEC-19`*
- [DSC-04-08] Events show at the right local time across time zones and daylight-saving changes, including moved or cancelled occurrences of a series. (`SRC-01-34`)
- [DSC-04-09] The app works with calendars that already exist in Google; creating, renaming and deleting calendars happens in the browser. (`SRC-02-28`) — *source: `DEC-21`*
- [DSC-04-10] Read-only and subscribed calendars (holidays, birthdays, colleagues') are shown like the others; their events cannot be edited, and that is clearly visible; free/busy-only calendars show busy blocks without details. (`SRC-02-29`) — *source: `DEC-22`*
- [DSC-04-11] The cache holds events from one year back to two years ahead; older or later periods browsed online are fetched from Google as needed. (`SRC-02-30`) — *source: `DEC-23`*
- [DSC-04-12] When a calendar stops being accessible while its account still works, it disappears from the app with its cached events, and the user is told once. (`SRC-02-31`) — *source: `DEC-24`*
- [DSC-04-13] Without network the user can only browse; creating, editing and responding to invitations are unavailable, and the app says why. (`SRC-01-35`, `SRC-02-32`) — *source: `DEC-25`*
- ~~[DSC-04-14] While the app is open, changes made elsewhere appear within about a minute, and right after the computer wakes or the network returns. (`SRC-02-33`) — *source: `DEC-26`*~~ — withdrawn, `DEC-53`
- [DSC-04-15] While the app runs, changes made elsewhere to calendars the user can edit appear within about a minute, read-only and subscribed calendars are refreshed about every 15 minutes, and the app synchronizes at once when its window becomes active or an event is opened for editing. (`SRC-03-15`) — *source: `DEC-53`*

## Contradictions

- `SRC-01-35` and `SRC-01-111` put offline editing after the MVP, SRC-01 §16 keeps it an open decision for the first public release → settled: browsing only offline, `DEC-25` (was `Q04`)

## Ambiguities

- "All accessible calendars" (`SRC-01-25`): (a) own calendars only, (b) also read-only, subscribed and shared ones → (b), not editable, `DEC-22` (was `Q14`)
- "Preserve provider-defined colours" (`SRC-01-27`) vs. local colours (`SRC-02-22`): the Google colour is the default, a local override never goes back to Google (`DSC-04-03`)
- "Every calendar" (`DSC-04-01`) after `DEC-28`: the list of calendars is always fetched, but events are synchronized and cached only for calendars that are not disabled in the settings (`DSC-05-12`)
- Calendar management (`SRC-01-118` excludes only sharing and admin): (a) create/rename/delete in the app, (b) existing calendars only → (b), `DEC-21` (was `Q13`)

## From the calls, not the documents

- `SRC-02-22` local calendar colours (said during `DSC-03`)
- `SRC-02-25`, `SRC-02-26` offline start and account removal (from `DSC-03`)
- `SRC-02-28`…`SRC-02-33` answers to `Q13`, `Q14`, `Q22`, `Q36`, `Q04` and the sync latency

## For the client workshop

None open — `Q04`, `Q13`, `Q14`, `Q22`, `Q36` resolved on 2026-10-22.

## Parked

- Calendar sharing and administration (`SRC-01-118`) — outside the MVP per the PRD.
- Creating, renaming and deleting calendars — done in the browser, `DEC-21`.
- Offline editing with a change queue and conflict resolution (`SRC-01-111`) — after the MVP, `DEC-25`.
- Push notifications from Google (changes within seconds) — would need a server of ours, `DEC-26`, `DEC-53`.
