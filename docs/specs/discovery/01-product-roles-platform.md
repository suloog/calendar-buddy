---
id: DSC-01
title: Product, roles, platform and integrations
kind: cross-cutting # business (a functional area — becomes a module at /og:modules) | cross-cutting (roles and access, technology and integrations, non-functional requirements — feeds PROJECT.md, NFR.md and the constitution, never a module)
status: settled # open (not yet discussed) | discussed (discussed with the analyst, waiting on the client) | settled (no [W] question in this topic is open)
sources: [SRC-01, SRC-02] # SRC-NN documents whose items this topic gathers, e.g. [SRC-01, SRC-03]
module: # MOD-NN — filled by /og:modules; empty for cross-cutting topics
sessions: [2026-10-22] # dates of the discovery sessions in which the topic was discussed, YYYY-MM-DD
---

<!-- Copy to docs/specs/discovery/NN-<topic>.md. Content in the project language; headings fixed.
     This file records what we UNDERSTOOD about this topic after reading every source and talking
     to the analyst — the layer between the sources (what the client wrote) and the requirements
     (what we will build). It is alive: after every client workshop /og:discovery updates the
     findings. A finding is NOT a requirement: no acceptance criteria, no priority, no UI; one
     finding becomes 1–3 requirements in the module's REQUIREMENTS.md later. -->

# [DSC-01] Product, roles, platform and integrations

## Topic scope

What the product is and for whom (the single end user, product framing, the "replace the browser" goal), the Google integration strategy (Calendar API, People API, OAuth verification), the provider abstraction and future providers, the target platforms, and the preliminary stack (see SRC-01 *Noted*: §10.1, §10.2, §15, §16). Ends where a business topic describes behaviour. Feeds PROJECT.md (actors, glossary, goals, stack and integrations) and the constitution's *Stack* section. Project-level questions with no other home (acceptance, milestones, support) live here.

## Sources in this topic

<Items from ALL sources that speak about this topic, side by side — so a contradiction between the
contract and the kick-off note is visible in one place, not in two files.>

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-01-01` | informal | Build a modern, lightweight, fast desktop calendar application designed primarily for Linux. |
| `SRC-01-03` | informal | Provide deep Google Calendar integration. |
| `SRC-01-04` | informal | The application should replace the need to open Google Calendar in a browser for everyday calendar management. |
| `SRC-01-05` | informal | The architecture should allow eventual macOS and Windows support without separate implementations. |
| `SRC-01-07` | informal | Cross-platform: keep platform-specific functionality behind well-defined interfaces. |
| `SRC-01-08` | informal | Google-first: make Google Calendar the initial fully supported provider. |
| `SRC-01-12` | informal | Privacy-first: connect directly to Google where feasible; do not require a proprietary synchronization back… |
| `SRC-01-13` | informal | Extensible: keep provider integrations separate from the application domain model. |
| `SRC-01-14` | informal | Use the Google Calendar API for the first version (chosen for feature coverage and integration quality, e.g… |
| `SRC-01-15` | informal | Use the Google People API for contact suggestions. |
| `SRC-01-16` | informal | Google Workspace directory suggestions may require additional permissions and may not be available for ever… |
| `SRC-01-24` | informal | Account for Google's OAuth app verification requirements before public distribution. |
| `SRC-01-86` | informal | Future provider adapters could include CalDAV and Microsoft Graph / Microsoft 365; provider-specific functi… |
| `SRC-01-109` | informal | CalDAV integration. |
| `SRC-01-110` | informal | Microsoft 365 / Outlook integration. |
| `SRC-01-119` | informal | Production-quality macOS and Windows releases if Linux is prioritized first. |
| `SRC-01-129` | informal | Complete everyday calendar tasks without opening Google Calendar in the browser. |
| `SRC-02-01` | informal | The app is just for me for now. |
| `SRC-02-02` | informal | MVP Linux only, architecture open for macOS/Windows. |
| `SRC-02-03` | informal | Toolkit: prototype decides, Iced → Slint → Tauri. |
| `SRC-02-04` | informal | My GCP project, shareable with my team; no publication, only the organization. |
| `SRC-02-05` | informal | Organization accounts only; personal accounts not needed now. |
| `SRC-02-06` | informal | Everyday = check upcoming meetings, accept invites, create meetings with Meet; browser OK for advanced. |
| `SRC-02-07` | informal | Straight to MVP, no prototype milestone, no dates. |
| `SRC-02-08` | informal | I accept against §14 by daily use; no support. |
| `SRC-02-09` | informal | Licence decided before a public release. |
| `SRC-02-10` | informal | No server infrastructure; only the GCP project in the org Workspace. |
| `SRC-02-11` | informal | I own the GCP project and OAuth client and am the Workspace admin. |
| `SRC-02-12` | informal | More than one account: External app "In production", no verification; personal and org accounts in MVP. |
| `SRC-02-43` | informal | Toolkit check: egui first (as in WordCraft), Iced as the fallback. |
| `SRC-02-13` | informal | The "unverified app" warning and the 100-user cap are acceptable; verification only before a public release. |
| SRC-01 *Noted* | informal | Preliminary stack (§10.1), logical components (§10.2), technical prototype (§15), open decisions (§16) |

## Key findings

- [DSC-01-01] A lightweight, fast desktop calendar application, built primarily for Linux. (`SRC-01-01`)
- [DSC-01-02] Google Calendar is the first and only fully supported provider, integrated through the Google Calendar API rather than CalDAV. (`SRC-01-03`, `SRC-01-08`, `SRC-01-14`)
- [DSC-01-03] Contact suggestions come from the Google People API; Workspace directory suggestions depend on extra permissions and are not available for every account. (`SRC-01-15`, `SRC-01-16`)
- [DSC-01-04] The app talks to Google directly from the user's computer; there is no sync backend of our own. (`SRC-01-12`)
- [DSC-01-05] Calendar providers are kept separate from the app's own model of calendars and events, and provider-specific features are optional, so CalDAV or Microsoft 365 can be added later without reshaping the app. (`SRC-01-13`, `SRC-01-86`)
- [DSC-01-06] Platform-specific behaviour sits behind interfaces, so macOS and Windows can come later from the same codebase. (`SRC-01-05`, `SRC-01-07`)
- [DSC-01-07] Google's OAuth verification is needed only before a public distribution of the app. (`SRC-01-24`) — *source: `DEC-10`*
- [DSC-01-08] For now the app is for the product owner's own use; it may be shared with teammates in the same Google Workspace organization, and no public release is planned. — *source: `DEC-01`*
- [DSC-01-09] There is one kind of user: a person signed in with their own Google account, acting as organizer or attendee; the app has no admin role. — *source: `DEC-01`*
- [DSC-01-10] The MVP is built and tested on Linux only; the architecture keeps macOS and Windows possible, but nothing is built or tested for them. — *source: `DEC-02`*
- ~~[DSC-01-11] The UI toolkit is chosen by a feasibility check in the first stories: Iced first, then Slint, then Tauri if the checks fail. — *source: `DEC-03`*~~ — withdrawn, `DEC-33`
- ~~[DSC-01-12] Google access goes through a GCP project in the organization's Workspace with an Internal consent screen: only that organization's accounts sign in, and no Google verification is needed. — *source: `DEC-04`*~~ — withdrawn, `DEC-10`
- [DSC-01-13] "Everyday" calendar work — the work the app must take over from the browser — is checking upcoming meetings, accepting invitations and creating meetings with Google Meet; opening the browser for advanced operations is acceptable. (`SRC-01-04`, `SRC-01-129`) — *source: `DEC-05`*
- [DSC-01-14] Delivery goes straight to the MVP, with no separate prototype milestone and no fixed dates. — *source: `DEC-06`*
- [DSC-01-15] The product owner accepts the MVP against the PRD's acceptance criteria through daily use; support and maintenance are not in scope. — *source: `DEC-07`*
- [DSC-01-16] There is no server infrastructure: the only environment is the GCP project in the organization's Google Workspace, owned and paid for by the organization and administered by the product owner. (`SRC-02-10`, `SRC-02-11`) — *source: `DEC-09`*
- [DSC-01-17] Google access goes through an External OAuth app in production status, without Google verification: any Google account — personal or organization, several at once — can sign in; users see Google's "unverified app" warning once, and the app is capped at 100 users until it is verified before a public release. (`SRC-01-18`, `SRC-02-12`, `SRC-02-13`) — *source: `DEC-10`*
- [DSC-01-18] The UI toolkit is chosen by a feasibility check in the first stories, starting with egui; Iced is the fallback. (`SRC-02-43`) — *source: `DEC-33`*

## Contradictions

- `SRC-01` header and `SRC-01-05` name macOS and Windows as targets, `SRC-01-119` puts their releases outside the MVP "if Linux is prioritized first" → settled: Linux-only MVP, `DEC-02` (was `Q03`)
- SRC-01 §2 prefers a Rust-native UI, §10.1/§16 keep Tauri (webview) as a candidate → first settled as a feasibility check with Tauri as the last fallback (`DEC-03`), then re-ruled: egui first, Iced as the fallback, Slint and Tauri dropped, `DEC-33` (was `Q08`)
- `SRC-01-18` wants personal and Workspace accounts, `SRC-02-04`/`SRC-02-05` want team sharing without publication → first settled as Internal app, organization accounts only (`DEC-04`), then re-ruled: External app in production, unverified, personal and organization accounts, `DEC-10`

## Ambiguities

- "Everyday calendar management" (`SRC-01-04`, `SRC-01-129`): (a) everything except the §13 exclusions, (b) the few tasks the owner actually opens the browser for → (b), `DEC-05` (was `Q10`)
- Who uses the app (`SRC-01-18`, `SRC-01-79` imply users, no actor is named): (a) one personal user, (b) a team with roles → (a), possibly teammates with the same single role, `DEC-01` (was `Q12`)

## From the calls, not the documents

- `SRC-02-01` the app is just for the product owner for now
- `SRC-02-04`, `SRC-02-05` own GCP project, shareable within the organization, organization accounts only — superseded by `SRC-02-12`, `SRC-02-13` (External app, unverified, any account)
- `SRC-02-06` what the browser is opened for today
- `SRC-02-10`, `SRC-02-11` no server infrastructure; the product owner owns the GCP project and is the Workspace admin

## For the client workshop

None open — `Q03`, `Q08`, `Q10`, `Q12`, `Q23`, `Q28`, `Q32`, `Q33`, `Q35` resolved on 2026-10-22.

## Parked

- CalDAV integration (`SRC-01-109`) — a later provider adapter; the provider separation (`DSC-01-05`) keeps it possible.
- Microsoft 365 / Outlook integration (`SRC-01-110`) — same reason.
- macOS and Windows releases (`SRC-01-119`) — Linux-only MVP, `DEC-02`.
- ~~Personal Google (Gmail) accounts — until a public release only organization accounts sign in, `DEC-04`.~~ — un-parked, `DEC-10`: personal accounts are in the MVP.
- Public distribution, Google OAuth verification and the licence choice (`SRC-01-24`) — only if the app goes public, `DEC-10`, `DEC-08`.
