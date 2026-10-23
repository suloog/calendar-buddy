---
id: SRC-01
title: Desktop Calendar — Product Requirements Document, version 0.1
kind: informal # working draft PRD, not a binding agreement
origin: internal — product owner (Maciej)
date: 2026-10-22 # the document is undated; date of the file in the repo
received_via: repo file docs/specs/sources/original/mvp-v0.1.md
supersedes:
status: read-only
---

# [SRC-01] Desktop Calendar — Product Requirements Document, v0.1

## Summary

A draft PRD for a lightweight, fast desktop calendar app that is Linux-first and later cross-platform, integrated with Google Calendar. It is meant to replace everyday use of Google Calendar in the browser. The PRD describes accounts, sync with a local cache and offline browsing, views, event management, guests and RSVP, notifications, a proposed Rust stack, non-functional requirements, an MVP scope with exclusions, acceptance criteria and a recommended technical prototype. It leaves open the UI toolkit, workweek/agenda views, background reminders, offline editing, packaging, distribution/Google verification and licence (§16).

Line references (*l. N*) point at `sources/original/mvp-v0.1.md`.

## Items

### 1. Product Overview

- [SRC-01-01] Build a modern, lightweight, fast desktop calendar application designed primarily for Linux. — *§1, l. 11*
- [SRC-01-02] The experience should combine the visual clarity of Apple Calendar with the modern layout and workflows of Morgen. — *§1, l. 11*
- [SRC-01-03] Provide deep Google Calendar integration. — *§1, l. 11*
- [SRC-01-04] The application should replace the need to open Google Calendar in a browser for everyday calendar management. — *§1, l. 13*
- [SRC-01-05] The architecture should allow eventual macOS and Windows support without separate implementations. — *§1, l. 13*

### 2. Product Principles

- [SRC-01-06] Linux-first: develop and test primarily on Linux, including Wayland and X11. — *§2, l. 17*
- [SRC-01-07] Cross-platform: keep platform-specific functionality behind well-defined interfaces. — *§2, l. 18*
- [SRC-01-08] Google-first: make Google Calendar the initial fully supported provider. — *§2, l. 20*
- [SRC-01-09] Desktop-native experience: support system notifications, keyboard navigation, HiDPI, window resizing, and system themes. — *§2, l. 21*
- [SRC-01-10] Performance-first: prioritize responsive navigation, low idle CPU use, modest memory consumption, and fast startup. — *§2, l. 22*
- [SRC-01-11] Local-first: cache calendar data locally for fast access and offline browsing. — *§2, l. 23*
- [SRC-01-12] Privacy-first: connect directly to Google where feasible; do not require a proprietary synchronization backend. — *§2, l. 24*
- [SRC-01-13] Extensible: keep provider integrations separate from the application domain model. — *§2, l. 25*

### 3. Integration Strategy: Google Calendar API vs. CalDAV

- [SRC-01-14] Use the Google Calendar API for the first version (chosen for feature coverage and integration quality, e.g. Google Meet creation, invitations and responses). — *§3, l. 29–31*
- [SRC-01-15] Use the Google People API for contact suggestions. — *§3, l. 33*
- [SRC-01-16] Google Workspace directory suggestions may require additional permissions and may not be available for every account. — *§3, l. 33*

### 4. Accounts and Authentication

- [SRC-01-17] Authenticate through Google OAuth 2.0 Authorization Code Flow with PKCE using the user's default browser. — *§4, l. 37*
- [SRC-01-18] Support multiple Google accounts simultaneously, including personal and Workspace accounts where permitted. — *§4, l. 38*
- [SRC-01-19] Request the minimum scopes needed for calendar operations and contact lookup. — *§4, l. 39*
- [SRC-01-20] Refresh credentials when possible and handle revoked access gracefully. — *§4, l. 40*
- [SRC-01-21] Store tokens in the operating system's credential store, not plaintext configuration files. — *§4, l. 41*
- [SRC-01-22] Allow users to add, remove, and reauthenticate accounts. — *§4, l. 42*
- [SRC-01-23] Clearly distinguish account-level errors from calendar-level synchronization errors. — *§4, l. 43*
- [SRC-01-24] Account for Google's OAuth app verification requirements before public distribution. — *§4, l. 44*

### 5. Calendar Synchronization

- [SRC-01-25] Retrieve all accessible calendars from connected Google accounts. — *§5, l. 48*
- [SRC-01-26] Show calendars from multiple accounts in unified views. — *§5, l. 49*
- [SRC-01-27] Let users toggle calendar visibility and preserve provider-defined colors. — *§5, l. 50*
- [SRC-01-28] Synchronize changes automatically and offer manual refresh. — *§5, l. 51*
- [SRC-01-29] Use incremental synchronization (`syncToken`) where supported by the Google Calendar API. — *§5, l. 52*
- [SRC-01-30] Cache events and relevant calendar metadata in a local SQLite database. — *§5, l. 53*
- [SRC-01-31] Support offline browsing of previously synchronized events. — *§5, l. 54*
- [SRC-01-32] Recover from connectivity loss, expired synchronization tokens, rate limits, and transient API errors. — *§5, l. 55*
- [SRC-01-33] Represent the synchronization state clearly without blocking the user interface. — *§5, l. 56*
- [SRC-01-34] Handle time zones, daylight-saving changes, and recurring-event exceptions correctly. — *§5, l. 57*
- [SRC-01-35] MVP offline boundary: offline browsing is included; offline creation/editing with a durable outgoing operation queue can be added later. — *§5, l. 59*

### 6.1. Main Views

- [SRC-01-36] Day view. — *§6.1, l. 65*
- [SRC-01-37] Workweek view. — *§6.1, l. 66*
- [SRC-01-38] Full-week view. — *§6.1, l. 67*
- [SRC-01-39] Month view. — *§6.1, l. 68*
- [SRC-01-40] Agenda / upcoming events view. — *§6.1, l. 69*

### 6.2. Navigation and Layout

- [SRC-01-41] Navigate quickly between days, weeks, and months. — *§6.2, l. 73*
- [SRC-01-42] Jump back to Today. — *§6.2, l. 74*
- [SRC-01-43] Provide a mini calendar and sidebar listing accounts and calendars. — *§6.2, l. 75*
- [SRC-01-44] Allow the sidebar to be collapsed. — *§6.2, l. 76*
- [SRC-01-45] Highlight the current day and current-time indicator. — *§6.2, l. 77*
- [SRC-01-46] Support keyboard shortcuts and keyboard-first navigation. — *§6.2, l. 78*
- [SRC-01-47] Scroll smoothly and handle overlapping events legibly. — *§6.2, l. 79*
- [SRC-01-48] Resize gracefully across different window dimensions and DPI scales. — *§6.2, l. 80*

### 6.3. Visual Direction

- [SRC-01-49] Inspired by Morgen for layout density, clear scheduling grids, and modern aesthetics. — *§6.3, l. 84*
- [SRC-01-50] Inspired by Apple Calendar for simplicity and intuitive interactions. — *§6.3, l. 85*
- [SRC-01-51] Provide light and dark themes, optionally following the system setting. — *§6.3, l. 86*
- [SRC-01-52] Preserve useful calendar color coding while keeping the interface restrained. — *§6.3, l. 87*
- [SRC-01-53] Avoid unnecessary animations and unrelated productivity features in the MVP. — *§6.3, l. 88*

### 7.1. Creating Events

- [SRC-01-54] Create an event by clicking or selecting a time range. — *§7.1, l. 94*
- [SRC-01-55] Offer quick entry plus a full event editor. — *§7.1, l. 95*
- [SRC-01-56] Set title, description, start/end, time zone, and calendar. — *§7.1, l. 96*
- [SRC-01-57] Support all-day and multi-day events. — *§7.1, l. 97*
- [SRC-01-58] Set location, reminders, availability (busy/free), and visibility where supported. — *§7.1, l. 98*
- [SRC-01-59] Create recurring events. — *§7.1, l. 99*
- [SRC-01-60] Add a Google Meet link through Google Calendar API where permitted. — *§7.1, l. 100*

### 7.2. Editing and Deleting

- [SRC-01-61] Edit and delete events subject to user permissions. — *§7.2, l. 104*
- [SRC-01-62] Drag events to a different time/date and resize their duration. — *§7.2, l. 105*
- [SRC-01-63] Move events between calendars where the provider permits it. — *§7.2, l. 106*
- [SRC-01-64] Handle recurring event instances and entire series; support changes to future occurrences where technically feasible. — *§7.2, l. 107*
- [SRC-01-65] Warn users when changes affect invited participants or an entire series. — *§7.2, l. 108*
- [SRC-01-66] Display clear failures if an operation is disallowed by Google or the organizer's permissions. — *§7.2, l. 109*

### 8.1. Inviting Guests

- [SRC-01-67] Add attendees using email addresses. — *§8.1, l. 115*
- [SRC-01-68] Suggest people from Google Contacts through Google People API. — *§8.1, l. 116*
- [SRC-01-69] Optionally include Google Workspace directory search when authorized. — *§8.1, l. 117*
- [SRC-01-70] Offer suggestions from recently used attendee email addresses. — *§8.1, l. 118*
- [SRC-01-71] Show attendee names, addresses, and invitation status. — *§8.1, l. 119*
- [SRC-01-72] Support required and optional attendees to the extent exposed by Google Calendar API. — *§8.1, l. 120*
- [SRC-01-73] Add or remove attendees when editing an event. — *§8.1, l. 121*
- [SRC-01-74] Send appropriate event updates through Google Calendar API. — *§8.1, l. 122*

### 8.2. Responding to Invitations

- [SRC-01-75] Identify invitations awaiting a response. — *§8.2, l. 126*
- [SRC-01-76] Support Accept, Decline, and Maybe responses. — *§8.2, l. 127*
- [SRC-01-77] Display attendee response statuses. — *§8.2, l. 128*
- [SRC-01-78] Synchronize RSVP changes with Google Calendar. — *§8.2, l. 129*
- [SRC-01-79] Respect permissions and organizer/attendee distinctions. — *§8.2, l. 130*

### 9. Notifications

- [SRC-01-80] Use operating-system notifications for upcoming events. — *§9, l. 134*
- [SRC-01-81] Honor supported event reminders. — *§9, l. 135*
- [SRC-01-82] Allow notifications to be enabled or disabled. — *§9, l. 136*
- [SRC-01-83] Avoid duplicate notifications across synchronization cycles. — *§9, l. 137*
- [SRC-01-84] Open the relevant event when a notification is activated, where supported by the OS. — *§9, l. 138*
- [SRC-01-85] Handle events occurring while the application is closed according to documented platform limitations; background scheduling is not assumed for MVP. — *§9, l. 139*

### 10.2. Logical Components

- [SRC-01-86] Future provider adapters could include CalDAV and Microsoft Graph / Microsoft 365; provider-specific functions must remain optional rather than forcing all providers into the same feature set. — *§10.2, l. 169*

### 11. Non-Functional Requirements

- [SRC-01-87] Responsive UI during network and synchronization operations. — *§11, l. 173*
- [SRC-01-88] Low idle CPU usage and reasonable memory consumption. — *§11, l. 174*
- [SRC-01-89] Fast startup and navigation even with many calendar events. — *§11, l. 175*
- [SRC-01-90] Robust handling of network failures, API quotas, and account expiration. — *§11, l. 176*
- [SRC-01-91] Correct recurrence handling across time zones and DST transitions. — *§11, l. 177*
- [SRC-01-92] Secure token storage and minimal OAuth permissions. — *§11, l. 178*
- [SRC-01-93] Clear error reporting without exposing sensitive user data in logs. — *§11, l. 179*
- [SRC-01-94] Automated tests for synchronization, recurrence, event editing, and RSVP flows. — *§11, l. 180*
- [SRC-01-95] Compatibility testing on mainstream Linux desktop environments, particularly Wayland and X11. — *§11, l. 181*
- [SRC-01-96] Exact performance budgets should be established through early measurements rather than assumed upfront. — *§11, l. 183*

### 12. MVP Scope

"The first usable release should include:" — *§12, l. 187*

- [SRC-01-97] Google sign-in and multiple account support. — *§12.1, l. 189*
- [SRC-01-98] Multiple calendars displayed in a unified interface. — *§12.2, l. 190*
- [SRC-01-99] Day, week, and month views (workweek and agenda may be included if inexpensive). — *§12.3, l. 191*
- [SRC-01-100] Create, edit, delete, and reschedule events. — *§12.4, l. 192*
- [SRC-01-101] Basic recurring event management. — *§12.5, l. 193*
- [SRC-01-102] Add attendees with Google Contacts suggestions. — *§12.6, l. 194*
- [SRC-01-103] Accept, decline, or tentatively accept invitations. — *§12.7, l. 195*
- [SRC-01-104] Google Meet link creation where supported. — *§12.8, l. 196*
- [SRC-01-105] System event reminders while the app is running. — *§12.9, l. 197*
- [SRC-01-106] Local synchronization cache and offline browsing. — *§12.10, l. 198*
- [SRC-01-107] Light/dark themes and keyboard shortcuts. — *§12.11, l. 199*
- [SRC-01-108] Solid Linux support. — *§12.12, l. 200*

### 13. Outside Initial MVP

- [SRC-01-109] CalDAV integration. — *§13, l. 204*
- [SRC-01-110] Microsoft 365 / Outlook integration. — *§13, l. 205*
- [SRC-01-111] Full offline editing with conflict resolution and queued writes. — *§13, l. 206*
- [SRC-01-112] Advanced attendee free/busy scheduling. — *§13, l. 207*
- [SRC-01-113] Advanced calendar search. — *§13, l. 208*
- [SRC-01-114] Google Drive attachments. — *§13, l. 209*
- [SRC-01-115] Natural-language event creation. — *§13, l. 210*
- [SRC-01-116] Task management or AI scheduling assistant. — *§13, l. 211*
- [SRC-01-117] Advanced theming and layout customization. — *§13, l. 212*
- [SRC-01-118] Calendar sharing and administrative management. — *§13, l. 213*
- [SRC-01-119] Production-quality macOS and Windows releases if Linux is prioritized first. — *§13, l. 214*

### 14. Acceptance Criteria

"A user should be able to:" — *§14, l. 218*

- [SRC-01-120] Connect at least one Google account without manually entering server settings. — *§14.1, l. 220*
- [SRC-01-121] See events from selected calendars across accounts. — *§14.2, l. 221*
- [SRC-01-122] Create a meeting and invite attendees using contact suggestions. — *§14.3, l. 222*
- [SRC-01-123] Receive an invitation and respond Accept, Decline, or Maybe. — *§14.4, l. 223*
- [SRC-01-124] Reschedule, edit, and cancel events within their permissions. — *§14.5, l. 224*
- [SRC-01-125] Manage common recurring event cases. — *§14.6, l. 225*
- [SRC-01-126] Create or open a Google Meet link. — *§14.7, l. 226*
- [SRC-01-127] Receive relevant desktop reminders while the app is running. — *§14.8, l. 227*
- [SRC-01-128] View cached events when temporarily offline. — *§14.9, l. 228*
- [SRC-01-129] Complete everyday calendar tasks without opening Google Calendar in the browser. — *§14.10, l. 229*

## Noted but not scope

- Document header: version 0.1; primary platform Linux; target platforms Linux, macOS, Windows; preferred technology Rust; initial integration Google Calendar. — *l. 3–7*
- Rust-first: implement application logic in Rust and prefer a Rust-native UI toolkit. — *§2, l. 19*
- §3 rationale: CalDAV supports standard operations, but invitation/response behaviour depends on the provider. Google CalDAV can also use OAuth 2.0, and the choice of the Calendar API is about feature coverage, not authentication. — *§3, l. 29–31*
- Proposed stack: Rust; UI: evaluate Iced first, consider Slint or Tauri; OAuth 2.0 + PKCE via system browser; Google Calendar API; Google People API; SQLite; async Rust HTTP client and runtime; OS credential store / keychain; OS-specific notification adapters. — *§10.1, l. 145–155*
- The UI framework is not finalized. A prototype should verify smooth scrolling, event overlap layout, resizing, drag-and-drop, accessibility and cross-platform behaviour before committing. — *§10.1, l. 157*
- Logical components: UI layer; domain layer (accounts, calendars, events, attendees, recurrence, RSVP state); provider interface with capability flags; Google adapter; sync engine; local storage; platform adapters (credential storage, desktop notifications, deep links, window behaviour). — *§10.2, l. 161–167*
- Recommended first technical prototype, before the full product: (1) Google OAuth and token persistence; (2) calendar listing and incremental event sync; (3) a smooth, responsive week view in Iced with overlaps and scrolling. It should inform the final UI technology choice. — *§15, l. 233–239*
- Open decisions: final GUI toolkit (Iced / Slint / Tauri); workweek and agenda in the first release; background running for reminders; offline editing in the first public release; Linux packaging (Flatpak, AppImage, distro packages); distribution, OAuth consent screen and Google verification strategy; licence and open-source model. — *§16, l. 243–249*
- No dates, milestones, budget or team are stated.

## Contradictions and unclear points

- Workweek and agenda are "main views" (`SRC-01-37`, `-40`) but "may be included if inexpensive" (`SRC-01-99`) and are an open decision (§16). → `Q01`
- "Week" in `SRC-01-99` can mean the full week (`SRC-01-38`) or the workweek (`SRC-01-37`). → `Q02`
- Target platforms include macOS and Windows (header, `SRC-01-05`), but production releases for them are outside the MVP "if Linux is prioritized first" (`SRC-01-119`). The condition is not settled. → `Q03`
- Offline editing "can be added later" (`SRC-01-35`) and is outside the initial MVP (`SRC-01-111`), yet whether it belongs in the "first public release" is open (§16). Is the "first usable release" (§12) the same as the "first public release"? → `Q04`
- Background reminders are "not assumed for MVP" (`SRC-01-85`, `-105`) but remain an open decision (§16). → `Q05`
- Workspace directory search is "optional when authorized" (`SRC-01-69`, `-16`) and absent from the MVP list (`SRC-01-102`). → `Q06`
- Recurrence depth differs: "future occurrences where technically feasible" (`SRC-01-64`), "basic" (`SRC-01-101`), "common cases" (`SRC-01-125`). → `Q07`
- A Rust-native UI toolkit is preferred (§2, Noted), while Tauri, which is webview-based, is a candidate (§10.1, §16). → `Q08`
- "Open a Google Meet link" is an acceptance criterion (`SRC-01-126`) with no matching feature in §7 (`SRC-01-60` covers adding a link only). → `Q09`
- "Everyday calendar tasks without the browser" (`SRC-01-04`, `-129`) sits next to exclusions that may be everyday for some users (search, attachments, sharing: `SRC-01-113`, `-114`, `-118`). → `Q10`
- Accessibility appears only as a prototype check (§10.1, Noted), with no requirement or target level. → `Q11`
