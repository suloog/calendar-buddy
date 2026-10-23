---
id: MOD-07
title: Reminders and notifications
ticket: # optional YouTrack ID, filled by /og:sync-youtrack
status: draft # draft | ready | in-progress | done
depends_on: [MOD-02] # other module IDs
sources: [SRC-01, SRC-02, SRC-03] # SRC-NN documents this module draws on, e.g. [SRC-01, SRC-03]
---

<!-- Copy to docs/specs/modules/NN-<slug>/REQUIREMENTS.md. Content in the project language; headings
     fixed by this template. Delete sections that have no content; never write "none". -->

# [MOD-07] Reminders and notifications

## Context

Being reminded: desktop notifications driven by the reminders set in Google, editing an event's reminders, snooze and dismiss, muting a calendar, running from the system tray with optional autostart, a single instance, and handling `webcal:` links. Answers for the scope area "Reminders and notifications"; comes from `DSC-09` (`discovery/09-notifications.md`), drawing on `SRC-01`, `SRC-02` and `SRC-03`. The Join button in notifications comes from `MOD-06`.

## Scope coverage

| Source item | Kind | Delivered by |
| ----------- | ---- | ------------ |
| `SRC-01-80` Use operating-system notifications for upcoming events. | informal | `MOD-07-F01` |
| `SRC-01-81` Honor supported event reminders. | informal | `MOD-07-F02` |
| `SRC-01-82` Allow notifications to be enabled or disabled. | informal | `MOD-07-F11`, `MOD-07-F12` |
| `SRC-01-83` Avoid duplicate notifications across synchronization cycles. | informal | `MOD-07-F07` |
| `SRC-01-84` Open the relevant event when a notification is activated, where supported by the OS. | informal | `MOD-07-F08` |
| `SRC-01-85` Handle events occurring while the application is closed according to documented platfor… | informal | `MOD-07-F14`, `MOD-07-F20` |
| `SRC-01-105` System event reminders while the app is running. | informal | `MOD-07-F01`, `MOD-07-F14` |
| `SRC-01-127` Receive relevant desktop reminders while the app is running. | informal | `MOD-07-F01` |
| `SRC-02-45` Reminders are important too, but they can be refined later. | informal | `MOD-07-F01`…`MOD-07-F06` |
| `SRC-03-07` Default reminders for new events come only from Google (the calendar's defaults, change… | informal | `MOD-07-F03` |
| `SRC-03-08` Changing an event's reminders in the app is synchronized with Google; both forms — noti… | informal | `MOD-07-F04`, `MOD-07-F05`, `MOD-07-F06` |
| `SRC-03-09` Closing the window hides the app to the system tray, where it keeps syncing and remindi… | informal | `MOD-07-F14`, `MOD-07-F15`, `MOD-07-F16` |
| `SRC-03-10` Notifications have Dismiss and Snooze (5 / 10 min); notifications are switched globally… | informal | `MOD-07-F10`, `MOD-07-F11`, `MOD-07-F12` |
| `SRC-03-11` Single instance: yes. `webcal:` links: fine. `mailto:` links: probably not. No update c… | informal | `MOD-07-F17`, `MOD-07-F18`; no `mailto:`, no update checking (`DEC-46`) |

## Functional requirements

Every requirement in this module is priority **must** unless marked otherwise.

### Reminders

- [MOD-07-F01] Each notification-type reminder of an upcoming event produces a desktop notification at the reminder's time.
- [MOD-07-F02] Reminder times follow the reminders set on the event in Google.
- [MOD-07-F03] A new event gets its calendar's default reminders from Google. — *source: `DEC-43`*
- [MOD-07-F04] In the event editor the user can add and remove reminders — as a notification or an e-mail, a chosen number of minutes, hours or days before — up to Google's limit; the change is saved to Google. — *source: `DEC-43`*
- [MOD-07-F05] The user can set an event back to its calendar's default reminders. — *source: `DEC-43`*
- [MOD-07-F06] E-mail reminders produce no desktop notification; Google sends them. — *source: `DEC-43`*
- [MOD-07-F07] Each reminder produces one notification; synchronization never repeats it.
- [MOD-07-F19] Events the user declined produce no notifications. — *source: `DEC-64`*
- [MOD-07-F20] Reminders whose time passed while the app was not running are not shown afterwards; reminders that fell while the computer slept are shown on waking if their event has not started yet. — *source: `DEC-64`*

### Notifications

- [MOD-07-F08] Clicking a notification opens its event in the app.
- [MOD-07-F09] A notification for an event with Google Meet has a Join button that opens the meeting in the browser. — *source: `DEC-38`*
- [MOD-07-F10] A notification offers Dismiss and Snooze for 5 or 10 minutes; snoozing does not change anything in Google. — *source: `DEC-45`*
- [MOD-07-F11] In the settings the user turns all notifications on or off. — *source: `DEC-45`*
- [MOD-07-F12] The user can mute a calendar's notifications without hiding the calendar. — *source: `DEC-45`*
- [MOD-07-F13] Calendars disabled in the settings produce no notifications. — *source: `DEC-28`*

### Running in the background

- [MOD-07-F14] Closing the window hides the app to the system tray, where it keeps synchronizing and notifying. — *source: `DEC-44`*
- [MOD-07-F15] The tray icon's menu opens the window and quits the app completely. — *source: `DEC-44`*
- [MOD-07-F16] In the settings the user can make the app start at login; it is off until the user turns it on. — *source: `DEC-44`, `DEC-64`*
- [MOD-07-F17] Only one instance of the app runs; launching it again brings the existing window forward. — *source: `DEC-46`*
- [MOD-07-F18] A `webcal:` link opened anywhere in the system opens Google Calendar in the browser, ready to add that subscription. — *source: `DEC-46`*

## UI/UX requirements

- [MOD-07-U01] A notification shows the event's title, start time and, if any, location. — *source: `DEC-64`*
- [MOD-07-U02] Clicking the tray icon shows or hides the window. — *source: `DEC-64`*
- [MOD-07-U03] A calendar's notifications are muted from the calendar's menu in the side panel. — *source: `DEC-64`*

## Constraints and edge cases

- An event is moved or deleted before its reminder → the reminder moves with it or is dropped.
- Several reminders fall in the same minute → each gets its own notification.
- The desktop's "do not disturb" mode is on → the desktop decides what is shown; the app does not work around it.
- The desktop's notification service does not support buttons → clicking the notification still opens the event, and Join and Snooze are available there. — *source: `DEC-64`*
- The system time zone changes → reminders follow the events' real times.
- Notifications are off globally → nothing is shown, but the reminders themselves stay unchanged in Google.

## Integrations

- **Google Calendar API — events** (read and write): each event's reminders.
- **Google Calendar API — calendar list** (read): each calendar's default reminders.
- **Desktop notification service** (write): showing notifications with buttons, receiving clicks.
- **System tray, autostart and URL handlers of the desktop** (write): the tray icon, starting at login, handling `webcal:` links.

## Technical notes

- Reminders are scheduled from the local store (`MOD-02`), so they work offline for cached events; overrides on the event win over the calendar's `defaultReminders` when `useDefault` is false.
- Notifications go through `org.freedesktop.Notifications` over D-Bus with actions (Join, Snooze 5, Snooze 10, Dismiss); a delivered-reminder record (event, occurrence start, reminder offset) prevents repeats across sync cycles.
- Tray: StatusNotifierItem (KDE); autostart: an XDG autostart `.desktop` entry; single instance: a well-known D-Bus name; `webcal:` handler registered as `x-scheme-handler/webcal`, opening `https://calendar.google.com/calendar/r?cid=…` in the browser. All behind the platform interface (constitution).
- Sleep and wake signals come from systemd-logind (shared with `MOD-02`).

## Out of scope / what not to worry about

- Handling `mailto:` links, update checking and automatic updates — `DEC-46`.
- Changing a calendar's default reminders in the app — done in Google (`DEC-43`).
- App-level default reminders and local-only reminders — `DEC-43`.
- A separate background service independent of the app — `DEC-44`.
- Delivering reminders while the app is completely closed — `SRC-01-85`.
- Performance targets beyond the project-wide ones — `DEC-62`.

## Acceptance criteria

- [ ] An event with a 10-minute notification reminder produces one desktop notification 10 minutes before, even after several synchronizations.
- [ ] A new event gets the calendar's default reminders; changing them in the app shows in Google Calendar in the browser.
- [ ] Snoozing a notification for 5 minutes shows it again 5 minutes later; Join opens the Meet meeting.
- [ ] With the window closed to the tray, reminders still arrive; quitting from the tray stops them.
- [ ] A muted calendar shows its events but produces no notifications; turning notifications off globally stops all of them.
- [ ] Launching the app twice brings the existing window forward; a `webcal:` link opens Google Calendar's "add calendar" page.
