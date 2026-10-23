---
id: SRC-03
title: Analyst notes — discovery session 2026-10-23
kind: informal
origin: analyst — Maciej (also the product owner and the organization's Workspace admin)
date: 2026-10-23
received_via: interview, /og:discovery session 2026-10-23
supersedes:
status: read-only
---

# [SRC-03] Analyst notes — discovery session 2026-10-23

## Summary

What the product owner said during the discovery session on 2026-10-23, continuing from `SRC-02`. Covers DSC-08 (guests, invitations, RSVP and Google Meet), DSC-09 (reminders and notifications) and the discovery close.

## Items

### DSC-08 · Guests, invitations, RSVP and Google Meet

- [SRC-03-01] Responding to an invitation with a comment and proposing another time are not needed — they are rarely used. — *session 2026-10-23, DSC-08*
- [SRC-03-02] When an event with guests is changed, the app asks every time whether to send an update to the guests. — *session 2026-10-23, answer to Q30*
- [SRC-03-03] On Workspace accounts, guest suggestions also include the organization's directory. — *session 2026-10-23, answer to Q06*
- [SRC-03-04] "Open a Meet link" means joining with one click: a clear Join button on the event, also in the agenda view and in the notification before the meeting. — *session 2026-10-23, answer to Q09*
- [SRC-03-05] Invitations in a calendar disabled in the settings are not visible; disabling is the user's choice. — *session 2026-10-23, DSC-08*
- [SRC-03-06] A simple free/busy preview of guests is wanted in the MVP: busy blocks per guest for the chosen day in the event editor, conflicts marked, "no data" where not shared; advanced scheduling stays out. — *session 2026-10-23, answer to Q18*

### DSC-09 · Reminders and notifications

- [SRC-03-07] Default reminders for new events come only from Google (the calendar's defaults, changed in Google). — *session 2026-10-23, DSC-09*
- [SRC-03-08] Changing an event's reminders in the app is synchronized with Google; both forms — notification and e-mail — are editable. — *session 2026-10-23, DSC-09*
- [SRC-03-09] Closing the window hides the app to the system tray, where it keeps syncing and reminding; optional autostart at login; full exit from the tray menu. — *session 2026-10-23, answer to Q05*
- [SRC-03-10] Notifications have Dismiss and Snooze (5 / 10 min); notifications are switched globally and can be muted per calendar. — *session 2026-10-23, answer to Q21*
- [SRC-03-11] Single instance: yes. `webcal:` links: fine. `mailto:` links: probably not. No update checking was asked for. — *session 2026-10-23, answer to Q27*

### Discovery close

- [SRC-03-12] Tokens are stored through the platform's standard secret store — on Linux the Secret Service API (KWallet, GNOME Keyring, KeePassXC), Keychain and Credential Manager on other systems; when no secret store runs, sign-in stops with a message explaining what to install or start; no fallback store in the MVP. — *session 2026-10-23, close*
- [SRC-03-13] Accepted at the close: the actor is called "User"; the glossary, scope areas, non-goals, overview, goals G1–G4 (G1 measured over two consecutive working weeks), milestone, risks, stack and integrations as proposed. — *session 2026-10-23, close*


- The client is internal: the analyst is also the product owner and rules on client questions directly.

## Contradictions and unclear points

- None.
