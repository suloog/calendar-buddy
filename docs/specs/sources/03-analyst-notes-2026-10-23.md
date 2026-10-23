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

### MOD-01 · Google accounts and sign-in (requirements review)

- [SRC-03-14] Accepted the MOD-01 register as drafted, including the defaults of Q37 (account management in settings, confirmation on removal, a heads-up about the unverified-app screen) and Q38 (revoke access at Google on removal, back to the welcome screen after the last account, partial consent, locked secret store). — *session 2026-10-23, /og:requirements MOD-01*

### MOD-02 · Calendars, synchronisation and offline cache (requirements review)

- [SRC-03-15] Background sync is adaptive instead of every minute for everything: calendars the user can edit about every minute, read-only and subscribed calendars about every 15 minutes, slower while the window is hidden in the tray; plus an immediate sync when the window becomes active and before editing. — *session 2026-10-23, /og:requirements MOD-02*
- [SRC-03-16] Accepted the MOD-02 register as drafted, including the defaults of Q39 (tray interval of at most 5 minutes for editable calendars, resettable local colour chosen from the side panel, no offline copy outside the window, last-sync time, events across the window edge kept whole, damaged cache rebuilt). — *session 2026-10-23, /og:requirements MOD-02*

### MOD-03 · Views and navigation (requirements)

- [SRC-03-17] The app starts in the view used last, on today's date. — *session 2026-10-23, /og:requirements MOD-03*
- [SRC-03-18] The agenda lists events from now on, grouped by day, skipping empty days, loading more while scrolling. — *session 2026-10-23, /og:requirements MOD-03*
- [SRC-03-19] Keyboard shortcuts follow Google Calendar's keys, so muscle memory works from day one. — *session 2026-10-23, /og:requirements MOD-03*
- [SRC-03-20] Accepted the MOD-03 register as drafted, including the defaults of Q40. — *session 2026-10-23, /og:requirements MOD-03*

### MOD-04 · Appearance and themes (requirements)

- [SRC-03-21] Theme: follows the system's light or dark setting by default and switches live; the settings offer System / Light / Dark. — *session 2026-10-23, /og:requirements MOD-04*
- [SRC-03-22] One fixed, restrained accent colour of the app's own; calendar colours stay the only loud colours. — *session 2026-10-23, /og:requirements MOD-04*
- [SRC-03-23] One compact grid density, like Morgen, chosen so a working day is visible in a typical window; no density setting. — *session 2026-10-23, /og:requirements MOD-04*
- [SRC-03-24] Accepted the MOD-04 register as drafted, including the defaults of Q41. — *session 2026-10-23, /og:requirements MOD-04*

### MOD-05 · Creating and editing events (requirements)

- [SRC-03-25] Clicking or selecting a range opens a small quick-entry box (title, the time already filled in, the default calendar); Enter saves, "More options" opens the full editor with what was typed. — *session 2026-10-23, /og:requirements MOD-05*
- [SRC-03-26] The default length of a new event comes from the user's Google Calendar settings. — *session 2026-10-23, /og:requirements MOD-05*
- [SRC-03-27] Deleting an event without guests happens without a question, with Undo available for a few seconds. — *session 2026-10-23, /og:requirements MOD-05*
- [SRC-03-28] The last change to an event (drag, resize, edit) can be undone with Ctrl+Z or the notice's Undo; no full history. — *session 2026-10-23, /og:requirements MOD-05*
- [SRC-03-29] Accepted the MOD-05 register as drafted, including the defaults of Q42. — *session 2026-10-23, /og:requirements MOD-05*

### MOD-06 · Guests, invitations, RSVP and Google Meet (requirements)

- [SRC-03-30] Invitations awaiting a response are marked in the views and counted in the side panel, the counter opening a list to respond from. — *session 2026-10-23, /og:requirements MOD-06*
- [SRC-03-31] Declined events stay visible, struck through and faded. — *session 2026-10-23, /og:requirements MOD-06*
- [SRC-03-32] The editor has an "Add Google Meet" switch that turns on by itself when the first guest is added; without guests it is off by default. — *session 2026-10-23, /og:requirements MOD-06*
- [SRC-03-33] Responding to an occurrence of a recurring invitation asks: this event or all events. — *session 2026-10-23, /og:requirements MOD-06*
- [SRC-03-34] Don't worry about the suggestion-speed NFR — it is a test app; we will tune it later if needed. — *session 2026-10-23, MOD-06 review*
- [SRC-03-35] Declined events visible by default, and it should be easy to switch them on and off. — *session 2026-10-23, MOD-06 review*
- [SRC-03-36] An organizer splitting a series is normal; tested in another project and it is fine. — *session 2026-10-23, MOD-06 review*
- [SRC-03-37] As a rule we do not try to be better than Google; we want a comfortable Linux interface to what Google built. — *session 2026-10-23, MOD-06 review*
- [SRC-03-38] Accepted the MOD-06 register with these changes, including the defaults of Q43. — *session 2026-10-23, /og:requirements MOD-06*

### MOD-07 · Reminders and notifications (requirements)

- [SRC-03-39] Accepted the MOD-07 register as drafted, including the defaults of Q44. — *session 2026-10-23, /og:requirements MOD-07*

## Noted but not scope

- The client is internal: the analyst is also the product owner and rules on client questions directly.

## Contradictions and unclear points

- None.
