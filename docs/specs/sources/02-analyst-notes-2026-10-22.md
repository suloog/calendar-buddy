---
id: SRC-02
title: Analyst notes — discovery session 2026-10-22
kind: informal
origin: analyst — Maciej (also the product owner and the organization's Workspace admin)
date: 2026-10-22
received_via: interview, /og:discovery session 2026-10-22
supersedes:
status: read-only
---

# [SRC-02] Analyst notes — discovery session 2026-10-22

## Summary

What the product owner said during the discovery session on DSC-01 (product, roles, platform and integrations), in answer to the open points of `SRC-01`. It covers the audience, platforms, the UI toolkit choice, Google access, what "everyday" means, delivery staging, acceptance, licence and who owns access.

## Items

### DSC-01 · Product, roles, platform and integrations

- [SRC-02-01] The app is just for me (the product owner) for now. — *session 2026-10-22, DSC-01*
- [SRC-02-02] Platforms: the MVP is Linux only, with the architecture kept open for macOS and Windows. — *session 2026-10-22, answer to Q03*
- [SRC-02-03] UI toolkit: the prototype decides, starting with Iced; Slint and then Tauri are allowed if Iced fails the checks. — *session 2026-10-22, answer to Q08*
- [SRC-02-04] The Google Cloud project is mine, and it should be possible to share the app with my team. No publication is needed, only the organization. — *session 2026-10-22, answer to Q23*
- [SRC-02-05] Only accounts of the organization are needed (Internal OAuth app, no verification). Personal Google accounts are not needed now. — *session 2026-10-22, follow-up to Q23*
- [SRC-02-06] Why I open Google Calendar in the browser today: to check upcoming meetings, accept invites, and create meetings with Meet. Opening the browser for advanced operations is allowed. — *session 2026-10-22, answer to Q10*
- [SRC-02-07] Go straight to the MVP, with no separate prototype milestone and no dates. — *session 2026-10-22, answer to Q33*
- [SRC-02-08] I accept the MVP against the §14 acceptance criteria by using it day to day. There is no support or maintenance in scope. — *session 2026-10-22, answer to Q32, Q35*
- [SRC-02-09] The licence is decided only before a public release; the repo stays private until then. — *session 2026-10-22, answer to Q28*
- [SRC-02-10] There is no server infrastructure. The only environment is the GCP project in the organization's Google Workspace, owned and paid for by the organization. — *session 2026-10-22, TQ17 (accepted proposal)*
- [SRC-02-11] I own the GCP project and OAuth client, and I am also the organization's Workspace admin. — *session 2026-10-22, TQ18*
- [SRC-02-12] Supporting more than one account only needs the Google app switched to External and "In production", without Google verification. Personal and organization accounts, several at once, are wanted in the MVP. — *session 2026-10-22, revisiting Q23*
- [SRC-02-13] It is acceptable that every user sees Google's "unverified app" warning once and that an unverified app is capped at 100 users. Verification happens only before a public release. — *session 2026-10-22, revisiting Q23*

### DSC-02 · Non-functional requirements

- [SRC-02-14] Accessibility target: full keyboard operation, readable contrast in both themes, HiDPI; no screen reader in the MVP. — *session 2026-10-22, answer to Q11*
- [SRC-02-15] Test and distribution: Ubuntu 26.10 with KDE Plasma, mainly Wayland plus an X11 check; the MVP ships as a .deb package; Flatpak and AppImage after the MVP. — *session 2026-10-22, answer to Q26*
- [SRC-02-16] No telemetry and no remote crash reports; local logs only, level set by the user, never event contents or tokens. — *session 2026-10-22, answer to Q29*
- [SRC-02-17] Data scale for performance: up to 5 accounts, up to 50 calendars, about 20,000 cached events; I set the thresholds after the first measurements. — *session 2026-10-22, answer to Q34*
- [SRC-02-18] Accepted: tokens in the OS keyring; no app-level encryption of the local cache (disk encryption covers it); no backups — Google is the system of record and the cache is rebuilt by resync; loss of local settings is acceptable. — *session 2026-10-22, TQ01, TQ10*
- [SRC-02-19] Accepted: data stays on the device and in Google; there is no backend and we are not a processor; no regulatory duties beyond secure tokens and clean logs. — *session 2026-10-22, TQ11, TQ12*
- [SRC-02-20] Accepted: automated unit and integration tests for sync, recurrence, event editing and RSVP; acceptance is manual by the product owner; no load testing. — *session 2026-10-22, TQ16*
- [SRC-02-21] Accepted: single-user desktop app with no server — MFA, security audits, throughput, scalability, SLA, redundancy, monitoring, web and mobile targets do not apply. — *session 2026-10-22, TQ03–TQ09, TQ15, TQ19–TQ24*

### DSC-03 · Google accounts and sign-in

- [SRC-02-22] Colours should be settable locally: each calendar starts with its Google colour and can be overridden in the app; the override is not written back to Google. — *session 2026-10-22, DSC-03 (belongs to DSC-04)*
- [SRC-02-23] The user picks which calendar is the default for creating new events. — *session 2026-10-22, DSC-03 (belongs to DSC-07)*
- [SRC-02-24] First run with no account: a welcome screen with one action, connect a Google account; the calendar appears after the first sync. — *session 2026-10-22, answer to Q24*
- [SRC-02-25] Start without network: the calendar is shown from the cache with a visible "offline" status; on a first run without network, sign-in is impossible, so a message and a retry once the network is back. — *session 2026-10-22, answer to Q24*
- [SRC-02-26] Removing an account deletes at once its tokens, its cached calendars and events, and its local settings (colours, default calendar); the Google account itself is untouched. — *session 2026-10-22, answer to Q22 (part)*
- [SRC-02-27] Out of the MVP for accounts: providers other than Google; the user choosing permission scopes by hand. — *session 2026-10-22, DSC-03 parked (accepted proposal)*

### DSC-04 · Calendars, synchronisation and offline cache

- [SRC-02-28] The app uses only calendars that already exist in Google; creating, renaming and deleting calendars is done in the browser. — *session 2026-10-22, answer to Q13*
- [SRC-02-29] Read-only and subscribed calendars are shown like the others, without editing; free/busy-only calendars show blocks without details. — *session 2026-10-22, answer to Q14*
- [SRC-02-30] The cache keeps events from one year back to two years ahead, with no separate size limit; older periods are fetched from Google when browsed online. — *session 2026-10-22, answer to Q22*
- [SRC-02-31] A calendar that loses access while the account works is removed with its cached events, and the user is told once. — *session 2026-10-22, answer to Q36*
- [SRC-02-32] Offline, the MVP allows browsing only; creating, editing and RSVP are unavailable, with a clear reason; an offline change queue comes after the MVP. — *session 2026-10-22, answer to Q04*
- [SRC-02-33] While the app is open, changes made elsewhere appear within about a minute, and right after the computer wakes or the network returns. — *session 2026-10-22, sync latency*

### DSC-05 · Views and navigation

- [SRC-02-34] I look at the calendar mostly in the week view, sometimes in a 3-day view on a smaller screen. — *session 2026-10-22, DSC-05*
- [SRC-02-35] Calendars should always be grouped by account, also when choosing a calendar while creating an event. — *session 2026-10-22, DSC-05*
- [SRC-02-36] Hidden calendars must not clutter quick lists — I have many calendars; turning them back on can be in the app settings. — *session 2026-10-22, DSC-05*
- [SRC-02-37] Views in the MVP besides day, full week and month: 3 days (chosen by hand) and agenda; no workweek view. — *session 2026-10-22, answer to Q01, Q02*
- [SRC-02-38] Two levels of hiding: a quick toggle in the side panel hides only the events; disabling in settings removes the calendar from the panel, the calendar picker and the views. A disabled calendar is not synchronized or cached and gives no reminders. — *session 2026-10-22, DSC-05*
- [SRC-02-39] No search in the MVP. — *session 2026-10-22, answer to Q17*
- [SRC-02-40] Week start and date/time formats follow the system; the calendar is shown in the system time zone; no second time zone in the MVP. — *session 2026-10-22, answer to Q19*

### DSC-06 · Visual direction and themes

- [SRC-02-41] The interface is in English only in the MVP; texts kept in one place so translations can be added later. — *session 2026-10-22, answer to Q20*
- [SRC-02-42] No mock-ups before coding; Morgen and Apple Calendar are the references and the look is refined iteratively on the working app, reviewed by the product owner. — *session 2026-10-22, answer to Q31*
- [SRC-02-43] Applications like WordCraft and GridCraft work very well; WordCraft's interface is egui on wgpu. The toolkit check starts with egui, Iced is the fallback; Slint and Tauri are dropped. — *session 2026-10-22, revisiting Q08*

### DSC-07 · Creating and editing events

- [SRC-02-44] It is important to be able to choose the event type where possible, e.g. out of office, and to make the types stand out in the views. — *session 2026-10-22, DSC-07*
- [SRC-02-45] Reminders are important too, but they can be refined later. — *session 2026-10-22, DSC-07 (belongs to DSC-09)*
- [SRC-02-46] Event types: out of office and focus time can be created where the account allows; working location and birthdays are shown only. — *session 2026-10-22, answer to Q15*
- [SRC-02-47] Event colours from Google are shown and can be changed in the editor. — *session 2026-10-22, answer to Q15*
- [SRC-02-48] Recurrence: presets daily, weekdays, weekly on chosen days, monthly, yearly; end by date, by count or never; edit scopes this / this and following / all; complex rules shown, edited in the browser. — *session 2026-10-22, answer to Q07*
- [SRC-02-49] Existing attachments shown as links; no ICS import or export in the MVP. — *session 2026-10-22, answer to Q16, Q25*

## Noted but not scope

- The client is internal: the analyst is also the product owner and rules on client questions directly.

## Contradictions and unclear points

- `SRC-02-05` narrows `SRC-01-18` ("personal and Workspace accounts") to organization accounts only → `DEC-04`, superseded the same day by `SRC-02-12` → `DEC-10` (personal and organization accounts restored)
