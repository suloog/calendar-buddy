---
id: DSC-09
title: Reminders and notifications
kind: business # business (a functional area — becomes a module at /og:modules) | cross-cutting (roles and access, technology and integrations, non-functional requirements — feeds PROJECT.md, NFR.md and the constitution, never a module)
status: settled # open (not yet discussed) | discussed (discussed with the analyst, waiting on the client) | settled (no [W] question in this topic is open)
sources: [SRC-01, SRC-02, SRC-03] # SRC-NN documents whose items this topic gathers, e.g. [SRC-01, SRC-03]
module: # MOD-NN — filled by /og:modules; empty for cross-cutting topics
sessions: [2026-10-23] # dates of the discovery sessions in which the topic was discussed, YYYY-MM-DD
---

<!-- Copy to docs/specs/discovery/NN-<topic>.md. Content in the project language; headings fixed.
     This file records what we UNDERSTOOD about this topic after reading every source and talking
     to the analyst — the layer between the sources (what the client wrote) and the requirements
     (what we will build). It is alive: after every client workshop /og:discovery updates the
     findings. A finding is NOT a requirement: no acceptance criteria, no priority, no UI; one
     finding becomes 1–3 requirements in the module's REQUIREMENTS.md later. -->

# [DSC-09] Reminders and notifications

## Topic scope

Desktop notifications for upcoming events: honouring event reminders, turning notifications on and off, no duplicates, opening the event from a notification, and what happens when the app is closed.

## Sources in this topic

<Items from ALL sources that speak about this topic, side by side — so a contradiction between the
contract and the kick-off note is visible in one place, not in two files.>

| Item | Kind | Short |
| ---- | ---- | ----- |
| `SRC-01-80` | informal | Use operating-system notifications for upcoming events. |
| `SRC-01-81` | informal | Honor supported event reminders. |
| `SRC-01-82` | informal | Allow notifications to be enabled or disabled. |
| `SRC-01-83` | informal | Avoid duplicate notifications across synchronization cycles. |
| `SRC-01-84` | informal | Open the relevant event when a notification is activated, where supported by the OS. |
| `SRC-01-85` | informal | Handle events occurring while the application is closed according to documented platform limitations; backg… |
| `SRC-01-105` | informal | System event reminders while the app is running. |
| `SRC-01-127` | informal | Receive relevant desktop reminders while the app is running. |
| `SRC-02-45` | informal | Reminders are important; refine later. |
| `SRC-03-04` | informal | Join button also in the notification. |
| `SRC-03-07` | informal | Default reminders only from Google. |
| `SRC-03-08` | informal | Per-event reminder changes synced; both forms. |
| `SRC-03-09` | informal | Tray + optional autostart. |
| `SRC-03-10` | informal | Dismiss + Snooze 5/10; global switch + per-calendar mute. |
| `SRC-03-11` | informal | Single instance; webcal yes; mailto no. |

## Key findings

- [DSC-09-01] Upcoming events are shown as system notifications. (`SRC-01-80`, `SRC-01-105`, `SRC-01-127`)
- [DSC-09-02] Notifications follow the reminders set on the event in Google. (`SRC-01-81`)
- [DSC-09-03] Each reminder gives one notification; synchronization never repeats it. (`SRC-01-83`)
- [DSC-09-04] Clicking a notification opens that event in the app. (`SRC-01-84`)
- [DSC-09-05] A notification about a Google Meet meeting has a Join button. (`SRC-03-04`) — *source: `DEC-38`*
- [DSC-09-06] Calendars disabled in the settings give no reminders. (`SRC-02-38`) — *source: `DEC-28`*
- [DSC-09-07] New events get their calendar's default reminders from Google; those defaults are changed in Google, in the browser. (`SRC-03-07`) — *source: `DEC-43`*
- [DSC-09-08] An event's reminders — notifications and e-mails — can be changed in the app; the change is saved to Google, so the phone reminds the same way; an event can go back to its calendar's defaults. (`SRC-01-58`, `SRC-01-81`, `SRC-03-08`) — *source: `DEC-43`*
- [DSC-09-09] The app shows notifications for notification-type reminders; e-mail reminders are sent by Google itself. (`SRC-03-08`) — *source: `DEC-43`*
- [DSC-09-10] Closing the window hides the app to the system tray, where it keeps synchronizing and reminding; autostart at login can be turned on; the app is fully closed from the tray menu. (`SRC-01-85`, `SRC-03-09`) — *source: `DEC-44`*
- [DSC-09-11] A notification has Dismiss and Snooze (5 or 10 minutes), and Join for Meet meetings; snooze is local and does not go to Google. (`SRC-03-10`) — *source: `DEC-45`*
- [DSC-09-12] Notifications are switched on and off globally, and a chosen calendar can be muted without hiding it. (`SRC-01-82`, `SRC-03-10`) — *source: `DEC-45`*
- [DSC-09-13] Only one instance of the app runs; launching it again brings back the existing window. (`SRC-03-11`) — *source: `DEC-46`*
- [DSC-09-14] `webcal:` links opened in the system go to the app, which opens Google Calendar in the browser to add the subscription. (`SRC-03-11`) — *source: `DEC-46`*

## Contradictions

- "Background scheduling is not assumed for MVP" (`SRC-01-85`) vs. SRC-01 §16 "whether the app should run in the background" → settled: tray with optional autostart, `DEC-44` (was `Q05`)

## Ambiguities

- "Honor supported event reminders" (`SRC-01-81`): (a) local reminders, (b) Google's reminders, synchronized → (b), defaults from Google, `DEC-43` (was part of `Q21`)
- "Allow notifications to be enabled or disabled" (`SRC-01-82`): (a) global only, (b) global and per calendar → (b), `DEC-45` (was part of `Q21`)

## From the calls, not the documents

- `SRC-02-45` reminders matter (said during `DSC-07`)
- `SRC-03-04` Join button in the notification (from `DSC-08`)
- `SRC-03-07`…`SRC-03-11` answers to `Q05`, `Q21`, `Q27`

## For the client workshop

None open — `Q05`, `Q21`, `Q27` resolved on 2026-10-23.

## Parked

- Handling `mailto:` links — the app is not a mail client, `DEC-46`.
- Automatic updates and update checking — a new .deb is installed by hand, `DEC-46`.
- App-level default reminders and editing a calendar's defaults in the app — set in Google, `DEC-43`.
- Local-only reminders — reminders always synchronize with Google, `DEC-43`.
- A separate background process independent of the app — the tray is enough, `DEC-44`.
