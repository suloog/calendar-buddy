# Desktop Calendar — Product Requirements Document

**Version:** 0.1
**Primary platform:** Linux  
**Target platforms:** Linux, macOS, Windows  
**Preferred technology:** Rust  
**Initial integration:** Google Calendar

## 1. Product Overview

Build a modern, lightweight, fast desktop calendar application designed primarily for Linux. The experience should combine the visual clarity of Apple Calendar with the modern layout and workflows of Morgen, while providing deep Google Calendar integration.

The application should replace the need to open Google Calendar in a browser for everyday calendar management. Its architecture should allow eventual macOS and Windows support without separate implementations.

## 2. Product Principles

- **Linux-first:** Develop and test primarily on Linux, including Wayland and X11.
- **Cross-platform:** Keep platform-specific functionality behind well-defined interfaces.
- **Rust-first:** Implement application logic in Rust and prefer a Rust-native UI toolkit.
- **Google-first:** Make Google Calendar the initial fully supported provider.
- **Desktop-native experience:** Support system notifications, keyboard navigation, HiDPI, window resizing, and system themes.
- **Performance-first:** Prioritize responsive navigation, low idle CPU use, modest memory consumption, and fast startup.
- **Local-first:** Cache calendar data locally for fast access and offline browsing.
- **Privacy-first:** Connect directly to Google where feasible; do not require a proprietary synchronization backend.
- **Extensible:** Keep provider integrations separate from the application domain model.

## 3. Integration Strategy: Google Calendar API vs. CalDAV

Use **Google Calendar API** for the first version. CalDAV supports many standard calendar operations, including events, recurrence, and some scheduling functionality, but behavior around invitations and responses depends on the provider and implementation. Google-specific features, such as Google Meet creation, are more straightforward through Google's APIs.

Direct account sign-in and CalDAV are not mutually exclusive: Google CalDAV can also use OAuth 2.0. The decision to use the Google Calendar API is about feature coverage and integration quality, not simply authentication.

Use **Google People API** for contact suggestions. Google Workspace directory suggestions may require additional permissions and may not be available for every account.

## 4. Accounts and Authentication

- Authenticate through Google OAuth 2.0 Authorization Code Flow with PKCE using the user's default browser.
- Support multiple Google accounts simultaneously, including personal and Workspace accounts where permitted.
- Request the minimum scopes needed for calendar operations and contact lookup.
- Refresh credentials when possible and handle revoked access gracefully.
- Store tokens in the operating system's credential store, not plaintext configuration files.
- Allow users to add, remove, and reauthenticate accounts.
- Clearly distinguish account-level errors from calendar-level synchronization errors.
- Account for Google's OAuth app verification requirements before public distribution.

## 5. Calendar Synchronization

- Retrieve all accessible calendars from connected Google accounts.
- Show calendars from multiple accounts in unified views.
- Let users toggle calendar visibility and preserve provider-defined colors.
- Synchronize changes automatically and offer manual refresh.
- Use incremental synchronization (`syncToken`) where supported by the Google Calendar API.
- Cache events and relevant calendar metadata in a local SQLite database.
- Support offline browsing of previously synchronized events.
- Recover from connectivity loss, expired synchronization tokens, rate limits, and transient API errors.
- Represent the synchronization state clearly without blocking the user interface.
- Handle time zones, daylight-saving changes, and recurring-event exceptions correctly.

**MVP offline boundary:** Offline browsing is included. Offline creation/editing with a durable outgoing operation queue can be added later.

## 6. User Interface and Navigation

### 6.1. Main Views

- Day view.
- Workweek view.
- Full-week view.
- Month view.
- Agenda / upcoming events view.

### 6.2. Navigation and Layout

- Navigate quickly between days, weeks, and months.
- Jump back to **Today**.
- Provide a mini calendar and sidebar listing accounts and calendars.
- Allow the sidebar to be collapsed.
- Highlight the current day and current-time indicator.
- Support keyboard shortcuts and keyboard-first navigation.
- Scroll smoothly and handle overlapping events legibly.
- Resize gracefully across different window dimensions and DPI scales.

### 6.3. Visual Direction

- Inspired by **Morgen** for layout density, clear scheduling grids, and modern aesthetics.
- Inspired by **Apple Calendar** for simplicity and intuitive interactions.
- Provide light and dark themes, optionally following the system setting.
- Preserve useful calendar color coding while keeping the interface restrained.
- Avoid unnecessary animations and unrelated productivity features in the MVP.

## 7. Event Management

### 7.1. Creating Events

- Create an event by clicking or selecting a time range.
- Offer quick entry plus a full event editor.
- Set title, description, start/end, time zone, and calendar.
- Support all-day and multi-day events.
- Set location, reminders, availability (busy/free), and visibility where supported.
- Create recurring events.
- Add a Google Meet link through Google Calendar API where permitted.

### 7.2. Editing and Deleting

- Edit and delete events subject to user permissions.
- Drag events to a different time/date and resize their duration.
- Move events between calendars where the provider permits it.
- Handle recurring event instances and entire series; support changes to future occurrences where technically feasible.
- Warn users when changes affect invited participants or an entire series.
- Display clear failures if an operation is disallowed by Google or the organizer's permissions.

## 8. Guests, Invitations, and RSVP

### 8.1. Inviting Guests

- Add attendees using email addresses.
- Suggest people from Google Contacts through Google People API.
- Optionally include Google Workspace directory search when authorized.
- Offer suggestions from recently used attendee email addresses.
- Show attendee names, addresses, and invitation status.
- Support required and optional attendees to the extent exposed by Google Calendar API.
- Add or remove attendees when editing an event.
- Send appropriate event updates through Google Calendar API.

### 8.2. Responding to Invitations

- Identify invitations awaiting a response.
- Support **Accept**, **Decline**, and **Maybe** responses.
- Display attendee response statuses.
- Synchronize RSVP changes with Google Calendar.
- Respect permissions and organizer/attendee distinctions.

## 9. Notifications

- Use operating-system notifications for upcoming events.
- Honor supported event reminders.
- Allow notifications to be enabled or disabled.
- Avoid duplicate notifications across synchronization cycles.
- Open the relevant event when a notification is activated, where supported by the OS.
- Handle events occurring while the application is closed according to documented platform limitations; background scheduling is not assumed for MVP.

## 10. Technical Architecture

### 10.1. Proposed Stack

| Area | Proposed approach |
|---|---|
| Main language | Rust |
| UI | Evaluate **Iced** first; consider **Slint** or **Tauri** if UI complexity warrants it |
| Google authentication | OAuth 2.0 + PKCE through system browser |
| Calendar integration | Google Calendar API |
| Contact suggestions | Google People API |
| Local database | SQLite |
| Networking | Asynchronous Rust HTTP client and runtime |
| Secrets | OS credential store / keychain |
| Notifications | OS-specific adapters |

The UI framework is **not finalized**. A prototype should verify smooth scrolling, event overlap layout, resizing, drag-and-drop, accessibility, and cross-platform behavior before committing.

### 10.2. Logical Components

- **UI layer:** Views, interactions, event editor, notifications.
- **Domain layer:** Accounts, calendars, events, attendees, recurrence, RSVP state.
- **Provider interface:** Calendar-provider abstraction with capability flags.
- **Google adapter:** Google Calendar API and People API integration.
- **Sync engine:** Incremental fetch, local persistence, error recovery.
- **Local storage:** SQLite cache and application settings.
- **Platform adapters:** Credential storage, desktop notifications, deep links, window behavior.

Future provider adapters could include CalDAV and Microsoft Graph / Microsoft 365. Provider-specific functions must remain optional rather than forcing all providers into the same feature set.

## 11. Non-Functional Requirements

- Responsive UI during network and synchronization operations.
- Low idle CPU usage and reasonable memory consumption.
- Fast startup and navigation even with many calendar events.
- Robust handling of network failures, API quotas, and account expiration.
- Correct recurrence handling across time zones and DST transitions.
- Secure token storage and minimal OAuth permissions.
- Clear error reporting without exposing sensitive user data in logs.
- Automated tests for synchronization, recurrence, event editing, and RSVP flows.
- Compatibility testing on mainstream Linux desktop environments, particularly Wayland and X11.

Exact performance budgets should be established through early measurements rather than assumed upfront.

## 12. MVP Scope

The first usable release should include:

1. Google sign-in and multiple account support.
2. Multiple calendars displayed in a unified interface.
3. Day, week, and month views (workweek and agenda may be included if inexpensive).
4. Create, edit, delete, and reschedule events.
5. Basic recurring event management.
6. Add attendees with Google Contacts suggestions.
7. Accept, decline, or tentatively accept invitations.
8. Google Meet link creation where supported.
9. System event reminders while the app is running.
10. Local synchronization cache and offline browsing.
11. Light/dark themes and keyboard shortcuts.
12. Solid Linux support.

## 13. Outside Initial MVP

- CalDAV integration.
- Microsoft 365 / Outlook integration.
- Full offline editing with conflict resolution and queued writes.
- Advanced attendee free/busy scheduling.
- Advanced calendar search.
- Google Drive attachments.
- Natural-language event creation.
- Task management or AI scheduling assistant.
- Advanced theming and layout customization.
- Calendar sharing and administrative management.
- Production-quality macOS and Windows releases if Linux is prioritized first.

## 14. Acceptance Criteria

A user should be able to:

1. Connect at least one Google account without manually entering server settings.
2. See events from selected calendars across accounts.
3. Create a meeting and invite attendees using contact suggestions.
4. Receive an invitation and respond Accept, Decline, or Maybe.
5. Reschedule, edit, and cancel events within their permissions.
6. Manage common recurring event cases.
7. Create or open a Google Meet link.
8. Receive relevant desktop reminders while the app is running.
9. View cached events when temporarily offline.
10. Complete everyday calendar tasks without opening Google Calendar in the browser.

## 15. Recommended First Technical Prototype

Before implementing the full product, build a small proof of concept that validates the main risks:

1. Google OAuth authentication and token persistence.
2. Calendar listing and incremental event synchronization.
3. A smooth, responsive **week view** implemented in Iced, including event overlaps and scrolling.

The prototype should inform the final UI technology choice and expose integration constraints early.

## 16. Open Decisions

- Final GUI toolkit: Iced vs. Slint vs. Tauri.
- Whether workweek and agenda views are required for the first release.
- Whether the app should run in the background to deliver reminders when its window is closed.
- Whether offline editing belongs in the first public release.
- Linux packaging priorities: Flatpak, AppImage, distro-specific packages.
- Distribution, OAuth consent screen, and Google verification strategy.
- License and open-source distribution model.
