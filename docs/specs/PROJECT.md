---
project: calendar-buddy
client: internal
language: en # language of every spec artifact (en | pl) — chosen once, at /og:analyze
status: approved
created: 2026-10-22
analyst: Maciej
---

<!-- Content in the project language. Section headings are fixed by this template — do not
     rename, reorder or add sections; the same structure is used in every Order Group project. -->

# calendar-buddy

## Overview

calendar-buddy is a lightweight, fast calendar application for Linux, connected directly to Google Calendar. It replaces opening Google Calendar in the browser for everyday calendar work. It is built for the product owner's own use, with the option of sharing it with colleagues, and runs without a server of its own: the data stays in Google and in a local cache. It can also be used without a Google account, with local calendars kept only on the device (`DEC-178`).

## Sources

The scope comes from the register in `SOURCES.md`.

- Binding: none — the project is internal; the product owner's rulings in `DECISIONS.md` settle the scope
- Informative: `SRC-01` Desktop Calendar — Product Requirements Document, v0.1 (129 items); `SRC-02` analyst notes, discovery session 2026-10-22 (49 items); `SRC-03` analyst notes, discovery session 2026-10-23 (13 items); `SRC-06` analyst notes, product owner's request 2026-10-26 (local calendars); `SRC-08` analyst notes, product owner's request 2026-10-27 (events from a description with a local model); `SRC-09` analyst notes, product owner's rulings 2026-10-27 (events from a description); `SRC-10` analyst notes, discovery session 2026-10-27 (events from a description); `SRC-11` analyst notes, requirements session 2026-10-27 (events from a description)

## Goals

- G1: For two consecutive working weeks, the product owner checks upcoming meetings, responds to invitations and creates meetings with Google Meet only in the app, without opening Google Calendar in the browser. (`SRC-01-04`, `SRC-01-129`, `DEC-05`, `DEC-48`)
- G2: All ten acceptance criteria of the PRD (§14) hold in daily use. (`SRC-01-120`…`SRC-01-129`, `DEC-07`)
- G3: Start-up, switching period or view, and scrolling the week view stay within the thresholds set after the first measurements, on a profile of 5 accounts, 50 calendars and about 20,000 cached events. (`SRC-01-96`, `DEC-14`)
- G4: The app installs from a .deb package and works on Ubuntu 26.10 with KDE Plasma, on Wayland and on X11. (`SRC-01-108`, `DEC-12`)

## Actors

| Actor | Who they are | How they sign in | Access, in short | Source |
| ----- | ------------ | ---------------- | ---------------- | ------ |
| User | A person using the app on their own Linux computer with one or more of their own Google accounts (personal or Workspace) — for now the product owner, later possibly colleagues | Google account through the system browser (OAuth); no password in the app — or no sign-in at all, with local calendars only (`DEC-179`) | Their own calendars and those shared with their accounts, within Google's permissions; on each event they act as organizer or guest | `SRC-01-18`, `SRC-01-79`, `SRC-02-01`, `DEC-01`, `DEC-10`, `DEC-48` |

## Glossary

| Term | Means exactly |
| ---- | ------------- |
| Account | A Google account (personal or Workspace) connected to the app. (`SRC-01-18`, `DEC-10`) |
| Calendar | A Google calendar accessible to a connected account, including read-only and subscribed ones, or a local calendar. (`SRC-01-25`, `DEC-22`, `DEC-178`) |
| Local calendar | A calendar kept only on the device, never synchronized with Google; local calendars are grouped under "Local". (`SRC-06-01`, `DEC-178`) |
| Primary calendar | An account's main calendar; invitations land there, and on a Workspace account only there can special event types be created. (`DEC-34`, `DEC-41`) |
| Default calendar | The calendar, chosen by the user, that new events go to. (`SRC-02-23`) |
| Hidden calendar | A calendar whose events are temporarily hidden by the quick toggle; it stays listed and keeps synchronizing. (`DEC-28`) |
| Disabled calendar | A calendar turned off in the settings; it is not listed, not synchronized and gives no reminders. (`DEC-28`) |
| Muted calendar | A calendar whose notifications are off; its events stay visible. (`DEC-45`) |
| Event | A single entry in a Google calendar or a local calendar. (`SRC-01-54`, `DEC-178`) |
| Series / occurrence | A recurring event / one of its instances. (`SRC-01-64`) |
| Edit scope | The reach of a change to a series: *this event*, *this and following*, or *all*. (`DEC-36`) |
| Special event type | *Out of office*, *focus time*, *working location* or *birthday*. (`DEC-34`) |
| Organizer / Guest | The person who invites to an event / an invited person, required or optional. (`SRC-01-72`, `SRC-01-79`) |
| Invitation | An event in which the user is a guest. (`SRC-01-75`) |
| Response | A guest's answer: *Accept*, *Decline* or *Maybe* (Google's "tentative"). (`SRC-01-76`, `SRC-01-103`) |
| Reminder | A per-user Google reminder on an event, as a notification or an e-mail, synchronized with Google; or a local event's notification, kept on the device. (`DEC-43`, `DEC-178`) |
| Notification | The desktop notification the app shows for a notification-type reminder. (`SRC-01-80`, `DEC-43`) |
| Cache | The local copy of calendar data, from one year back to two years ahead. (`SRC-01-30`, `DEC-23`) |
| Offline | No network; only what is in the cache can be browsed. (`DEC-25`) |
| Free/busy preview | Guests' busy blocks for the chosen day, shown in the event editor. (`DEC-42`) |
| Join | Opening the Google Meet meeting of an event from the event, the agenda or a notification. (`DEC-38`) |
| Description | A sentence in Polish or English from which the app fills an event to add or change, e.g. "wrzuć mi spotkanie z Anią jutro o 10". (`SRC-08-01`, `DEC-192`) |
| Local model | The language model Ollama runs on the user's own computer to read a description; nothing is sent elsewhere. (`SRC-08-02`, `DEC-193`) |

## Scope

### In scope

- Google accounts and sign-in — several personal and Workspace accounts through the browser, refresh and revoked access, account removal with its data, first run (`discovery/03-accounts.md` · `SRC-01-17`…`SRC-01-23`, `SRC-01-97`, `SRC-01-120`; `SRC-02-24`…`SRC-02-27`)
- Calendars, synchronisation and offline cache — all accessible calendars in one view, local colours, background sync (about a minute) and on demand, cache from one year back to two years ahead, offline browsing, read-only calendars (`discovery/04-calendars-sync.md` · `SRC-01-11`, `SRC-01-25`…`SRC-01-35`, `SRC-01-98`, `SRC-01-106`, `SRC-01-121`, `SRC-01-128`; `SRC-02-22`, `SRC-02-28`…`SRC-02-33`)
- Views and navigation — day, 3 days, week, month, agenda; side panel grouped by account; hiding and disabling calendars; keyboard; regional formats and time zone from the system (`discovery/05-views-navigation.md` · `SRC-01-36`…`SRC-01-48`, `SRC-01-99`; `SRC-02-34`…`SRC-02-40`)
- Visual direction and themes — Morgen and Apple Calendar as references, light and dark themes, English interface (`discovery/06-visual-themes.md` · `SRC-01-02`, `SRC-01-49`…`SRC-01-53`, `SRC-01-107`; `SRC-02-41`, `SRC-02-42`)
- Creating and editing events — quick entry and full editor, dragging, recurrence presets with three edit scopes, special event types, event colours, attachments as links (`discovery/07-events.md` · `SRC-01-54`…`SRC-01-66`, `SRC-01-100`, `SRC-01-101`, `SRC-01-124`, `SRC-01-125`; `SRC-02-23`, `SRC-02-44`…`SRC-02-49`)
- Guests, invitations, RSVP and Google Meet — guest suggestions (contacts, Workspace directory, recent), responses, asking before notifying guests, free/busy preview, Meet with Join (`discovery/08-guests-rsvp-meet.md` · `SRC-01-60`, `SRC-01-67`…`SRC-01-79`, `SRC-01-102`…`SRC-01-104`, `SRC-01-112`, `SRC-01-122`, `SRC-01-123`, `SRC-01-126`; `SRC-03-01`…`SRC-03-06`)
- Reminders and notifications — notifications from Google reminders, editing reminders, snooze, muting calendars, tray and autostart, single instance, `webcal:` links (`discovery/09-notifications.md` · `SRC-01-80`…`SRC-01-85`, `SRC-01-105`, `SRC-01-127`; `SRC-03-07`…`SRC-03-11`)
- Local calendars — calendars and events kept only on the device, managed in the settings, grouped under "Local"; using the app without a Google account, with the welcome screen inviting to connect one (`SRC-06-01`…`SRC-06-16`, `DEC-178`, `DEC-179`)
- Events from a description — a local model through Ollama, installed by the user, fills the new-event or edit window from a description in Polish or English; the user approves it by hand; nothing leaves the computer (`SRC-08`, `SRC-09`, `DEC-192`…`DEC-200`)

### Out of scope (non-goals)

- Providers other than Google — CalDAV, Microsoft 365 / Outlook; synchronizing local calendars anywhere. (`SRC-01-109`, `SRC-01-110`, `DEC-178`)
- macOS and Windows releases; public distribution, Google verification and choosing a licence. (`DEC-02`, `DEC-08`, `DEC-10`)
- Offline editing with a change queue and conflict resolution. (`SRC-01-111`, `DEC-25`)
- Managing Google calendars — creating, renaming, deleting, sharing, administration; local calendars are managed in the app. (`SRC-01-118`, `DEC-21`, `DEC-178`)
- Event search, basic or advanced. (`SRC-01-113`, `DEC-30`)
- A workweek view; a second time zone or choosing the display time zone by hand. (`DEC-27`, `DEC-29`)
- Advanced theming and layout customization, interface translations, mock-ups. (`SRC-01-117`, `DEC-31`, `DEC-32`)
- Task management, an AI scheduling assistant beyond filling events from a description. (`SRC-01-115`, `SRC-01-116`, `DEC-192`)
- Adding Google Drive attachments; ICS import and export. (`SRC-01-114`, `DEC-37`)
- A full recurrence-rule editor; creating working-location events. (`DEC-36`, `DEC-34`)
- Advanced scheduling with guests (multi-day grid, free-slot suggestions); responding with a comment or proposing another time. (`SRC-01-112`, `DEC-42`, `SRC-03-01`)
- App-level default reminders and local-only reminders on Google events; local calendars have their own. (`DEC-43`, `DEC-178`)
- Handling `mailto:` links; update checking and automatic updates. (`DEC-46`)
- Telemetry and remote crash reports; screen-reader support; package formats other than .deb; a fallback token store when no secret store runs. (`DEC-11`, `DEC-12`, `DEC-13`, `DEC-47`)

## Stack & integrations

Rust. The interface is built with egui (eframe on wgpu), with Iced as the fallback; the choice is confirmed by a feasibility check in the first stories (`DEC-33`). Google Calendar API for calendars, events, reminders, special event types, Meet and guests' free/busy; Google People API for contacts and, on Workspace accounts, the organization's directory. Sign-in is OAuth 2.0 with PKCE through the system browser, as an External, unverified app in production status in the product owner's GCP project (`DEC-10`). Local data: an SQLite cache; tokens in the platform's standard secret store — on Linux the Secret Service API (KWallet, GNOME Keyring, KeePassXC) (`DEC-47`); desktop notifications and a tray icon on KDE Plasma. There is no backend: Google is the system of record and all logic runs in the app. The GCP project belongs to the organization; Maciej owns access on both sides (`DEC-09`). Distribution as a .deb package (`DEC-12`).

## Milestones

| Milestone | Contains | Target |
| --------- | -------- | ------ |
| M1 MVP | All eight scope areas; the UI toolkit (egui or Iced) settled in the first stories | no date (`DEC-06`) |

## Risks

| Risk | Impact | Mitigation |
| ---- | ------ | ---------- |
| egui fails the feasibility checks (dense grid, dragging, scrolling) | Moving to Iced shifts the views and the look | The first stories build the week view with dragging and scrolling before the rest of the UI (`DEC-33`) |
| Recurrence — "this and following", exceptions, DST | Wrong events in Google, for guests too | Automated sync and recurrence tests (`DEC-17`); complex rules only in the browser (`DEC-36`) |
| Unverified Google app — 100-user cap, consent warning, a growing list of sensitive scopes, possible tightening of Google's rules | Sign-in may stop working or require verification | Minimal scopes (`DSC-02-05`); the verification path is known and run before any public release (`DEC-10`) |
| Google API limits with about one-minute sync (5 accounts, 50 calendars) | Sync delays and errors | Incremental sync; disabled calendars are not synchronized; measured on the `DEC-14` profile |
| Scope beyond the PRD (several extensions) | A larger MVP cost | The estimate shows the extensions separately so they can be cut |
| Look without mock-ups | Visual rework during the build | Iterative reviews with the product owner (`DEC-32`) |
| Tested only on Ubuntu with KDE | Uncertain behaviour for colleagues on GNOME or without a secret store | Outside the MVP; revisited when the app is shared with the team (`DEC-12`, `DEC-47`) |

<Requirements are NOT listed here. They are enumerated per module in
docs/specs/modules/NN-<slug>/REQUIREMENTS.md (MOD-NN-Fnn / Unn) and, project-wide, in
docs/specs/NFR.md (NFR-nn). Source documents are registered in
docs/specs/SOURCES.md, open questions in docs/specs/QUESTIONS.md, decisions in
docs/specs/DECISIONS.md.>
