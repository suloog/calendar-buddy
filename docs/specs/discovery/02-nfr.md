---
id: DSC-02
title: Non-functional requirements
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

# [DSC-02] Non-functional requirements

## Topic scope

Performance, robustness, security of tokens and logs, test automation, Linux compatibility (Wayland/X11, DEs, packaging), accessibility and the desktop-native qualities. Works through the technical checklist (TQ01–TQ24) into `TECH-CHECKLIST.md`. Feeds NFR.md and the constitution; never a module.

## Sources in this topic

<Items from ALL sources that speak about this topic, side by side — so a contradiction between the
contract and the kick-off note is visible in one place, not in two files.>

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-01-06` | informal | Linux-first: develop and test primarily on Linux, including Wayland and X11. |
| `SRC-01-09` | informal | Desktop-native experience: support system notifications, keyboard navigation, HiDPI, window resizing, and s… |
| `SRC-01-10` | informal | Performance-first: prioritize responsive navigation, low idle CPU use, modest memory consumption, and fast… |
| `SRC-01-87` | informal | Responsive UI during network and synchronization operations. |
| `SRC-01-88` | informal | Low idle CPU usage and reasonable memory consumption. |
| `SRC-01-89` | informal | Fast startup and navigation even with many calendar events. |
| `SRC-01-90` | informal | Robust handling of network failures, API quotas, and account expiration. |
| `SRC-01-91` | informal | Correct recurrence handling across time zones and DST transitions. |
| `SRC-01-92` | informal | Secure token storage and minimal OAuth permissions. |
| `SRC-01-93` | informal | Clear error reporting without exposing sensitive user data in logs. |
| `SRC-01-94` | informal | Automated tests for synchronization, recurrence, event editing, and RSVP flows. |
| `SRC-01-95` | informal | Compatibility testing on mainstream Linux desktop environments, particularly Wayland and X11. |
| `SRC-01-96` | informal | Exact performance budgets should be established through early measurements rather than assumed upfront. |
| `SRC-01-108` | informal | Solid Linux support. |
| `SRC-02-14` | informal | Keyboard, contrast, HiDPI; no screen reader in the MVP. |
| `SRC-02-15` | informal | Ubuntu 26.10 + KDE Plasma, Wayland + X11 check; .deb. |
| `SRC-02-16` | informal | No telemetry; local logs only, never event contents or tokens. |
| `SRC-02-17` | informal | ≤5 accounts, ≤50 calendars, ~20k events; thresholds after measurements. |
| `SRC-02-18` | informal | Tokens in keyring; no cache encryption; no backups, resync from Google. |
| `SRC-02-19` | informal | Data stays on device and in Google; no backend, not a processor. |
| `SRC-02-20` | informal | Unit + integration tests for sync, recurrence, editing, RSVP; manual acceptance. |
| `SRC-02-21` | informal | Single-user desktop, no server: MFA, audits, SLA, web/mobile n/a. |

## Key findings

- [DSC-02-01] The app keeps responding while it synchronizes with Google or waits for the network. (`SRC-01-87`, `SRC-01-10`)
- [DSC-02-02] It uses little CPU when idle and modest memory, starts fast and switches views fast, also with many events. (`SRC-01-10`, `SRC-01-88`, `SRC-01-89`)
- [DSC-02-03] Concrete performance thresholds come from early measurements, not assumptions. (`SRC-01-96`)
- [DSC-02-04] The app copes with network failures, Google API limits, and expired or revoked account access. (`SRC-01-90`)
- [DSC-02-05] Tokens are kept in the system credential store, and the app asks Google only for the minimum permissions. (`SRC-01-92`)
- [DSC-02-06] Errors are reported clearly, and logs contain no sensitive user data. (`SRC-01-93`)
- [DSC-02-07] Recurring events behave correctly across time zones and daylight-saving changes. (`SRC-01-91`)
- [DSC-02-08] The app behaves like a native Linux application — system notifications, keyboard operation, HiDPI, window resizing, the system theme — on Wayland and X11 in mainstream desktop environments. (`SRC-01-06`, `SRC-01-09`, `SRC-01-95`, `SRC-01-108`)
- [DSC-02-09] The MVP's accessibility target is full keyboard operation, readable contrast in both themes and correct HiDPI scaling; screen-reader support is not an MVP goal. — *source: `DEC-11`*
- [DSC-02-10] The MVP is tested on Ubuntu 26.10 with KDE Plasma, mainly on Wayland with an X11 check, and ships as a .deb package; other package formats come after the MVP. — *source: `DEC-12`*
- [DSC-02-11] There is no telemetry and no remote crash reporting; logs are local, in the user's directory, with a user-set level, and never contain event contents or tokens. — *source: `DEC-13`*
- [DSC-02-12] Performance is measured on a profile of up to 5 accounts, 50 calendars and about 20,000 cached events, for start-up, switching period or view, and scrolling the week view; the product owner sets the thresholds after the first measurements. — *source: `DEC-14`*
- [DSC-02-13] The app's only network traffic is HTTPS to Google's APIs. (`SRC-01-12`, `SRC-02-19`) — *source: `DEC-16`*

## Contradictions

None between sources. A tension with the toolkit choice — screen-reader support would favour Slint or Tauri over Iced — is avoided by `DEC-11`.

## Ambiguities

- "Solid Linux support" (`SRC-01-108`) and "mainstream Linux desktop environments" (`SRC-01-95`): (a) a broad distribution/DE matrix, (b) the product owner's own setup → (b), `DEC-12` (was `Q26`)
- "Many calendar events" (`SRC-01-89`): no volume given → profile of 5 accounts / 50 calendars / ~20k events, `DEC-14` (was `Q34`)
- Accessibility (SRC-01 §10.1, prototype check only): (a) a screen-reader target, (b) keyboard and contrast → (b), `DEC-11` (was `Q11`)

## From the calls, not the documents

- `SRC-02-14`…`SRC-02-17` answers to Q11, Q26, Q29, Q34
- `SRC-02-18`…`SRC-02-21` accepted technical-checklist proposals (data at rest, recovery, personal data, test levels, what does not apply)

## For the client workshop

None open — `Q11`, `Q26`, `Q29`, `Q34` resolved on 2026-10-22. Technical checklist: 24 of 24 settled (`TECH-CHECKLIST.md`).

## Parked

- Screen-reader support — not an MVP goal, `DEC-11`.
- Flatpak, AppImage and other distributions/desktop environments as tested targets — after the MVP, `DEC-12`.
- Telemetry and remote crash reporting — not wanted, `DEC-13`.
- App-level encryption of the local cache and backups — not needed, `DEC-15`.
- Security audits, load testing, SLA, redundancy, web and mobile targets — do not apply, `DEC-17`, `DEC-18`.
