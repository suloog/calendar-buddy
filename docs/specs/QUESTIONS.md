# Open questions — for the client

> Every assumption taken while writing the sources, requirements and stories, and every point
> deliberately left to the client, collected in one place — so the list can be sent ahead of a
> workshop instead of being reconstructed from twelve files. Written so the client can answer it
> as it stands: business language, one question per line, our proposed default where we have one.
>
> **[A]** an assumption with a default already written into the artifacts — confirm or correct it.
> **[W]** a wider question with no assumption taken — it needs a ruling before anything depending
> on it can be built.
>
> Each question names the artifact it affects and, once sent, when and how it was asked
> (*asked: YYYY-MM-DD, e-mail*). A story that depends on an unanswered question names it under
> "Open questions" and carries `status: blocked`. When a question is answered, `/og:decide`
> records the ruling as a `DEC-nn` and moves the question to **Resolved**. Numbers are never
> reused.

## General


## Resolved

- **Q03** Is Linux definitely prioritised first, so that production macOS/Windows releases are out of the MVP? Should the MVP still build and run on macOS/Windows (e.g. in CI), or only keep the architecture open? — Linux only for the MVP; architecture kept open for macOS/Windows, 2026-10-22 (`DEC-02`)
- **Q08** Is a webview-based toolkit (Tauri) acceptable, given the stated preference for a Rust-native UI? Is the toolkit choice settled by the prototype alone? — feasibility check in the first stories; Iced first, then Slint, then Tauri, 2026-10-22 (`DEC-03`) — re-ruled the same day: egui first, Iced as the fallback (`DEC-33`)
- **Q10** Which tasks count as "everyday", so that the browser is no longer needed? Are basic search, viewing existing attachments and viewing shared calendars expected despite the §13 exclusions? — everyday = check upcoming meetings, accept invitations, create meetings with Meet; browser allowed for advanced operations, 2026-10-22 (`DEC-05`)
- **Q12** Is there only one kind of user (a person with their own Google accounts, acting as organizer or attendee), with no admin or support role in the app? — one kind of user, the product owner (possibly teammates in the organization); no admin role, 2026-10-22 (`DEC-01`)
- **Q23** Who owns the Google Cloud project and OAuth client (and its quotas), and when must Google verification be done relative to the first release? — product owner's GCP project in the organization's Workspace, Internal OAuth app, org accounts only, no verification, 2026-10-22 (`DEC-04`) — re-ruled the same day: External app in production, unverified, personal and organization accounts (`DEC-10`)
- **Q28** What licence and distribution model will the app use (open source or not, which licence)? — no licence until a public release; repo private, 2026-10-22 (`DEC-08`)
- **Q32** Who accepts the MVP and how? Are the §14 acceptance criteria the full acceptance procedure? — the product owner accepts against §14 through daily use, 2026-10-22 (`DEC-07`)
- **Q33** What are the milestones and target dates? Is the technical prototype (§15) a separate milestone with its own go/no-go? — straight to MVP, no prototype milestone, no dates, 2026-10-22 (`DEC-06`)
- **Q35** Is support or maintenance after the first release part of the scope? — no support or maintenance in scope, 2026-10-22 (`DEC-07`)
- **Q11** Is there an accessibility target (screen reader support on Linux, contrast, full keyboard operation), or is accessibility only a prototype check? — keyboard, contrast and HiDPI; no screen reader in the MVP, 2026-10-22 (`DEC-11`)
- **Q26** Which Linux packaging formats come first (Flatpak, AppImage, distro packages), and which distributions/desktop environments are the support and test matrix? — Ubuntu 26.10 + KDE Plasma (Wayland, X11 check), .deb; Flatpak/AppImage after MVP, 2026-10-22 (`DEC-12`)
- **Q29** Is crash reporting or any telemetry allowed, given the privacy-first principle? Which log levels and log locations are expected? — no telemetry, no remote crash reports; local logs only, 2026-10-22 (`DEC-13`)
- **Q34** Who sets the performance budgets after the early measurements, and what volume is "many calendar events" (number of accounts, calendars, events)? — ≤5 accounts, ≤50 calendars, ~20k events; the product owner sets thresholds after measurements, 2026-10-22 (`DEC-14`)
- **Q24** What should the user see on first run with no account, when offline at first start, and when a calendar loses access? — welcome screen with one action; no network at first run: message and retry; later starts offline: cache with offline status. The calendar-loses-access part moved to `Q36`. 2026-10-22 (`DEC-19`)
- **Q04** Is the "first usable release" (MVP) the same as the "first public release"? Does offline editing stay out of both? — offline = browsing only in the MVP; offline change queue after the MVP, 2026-10-22 (`DEC-25`)
- **Q13** Can the user create, rename or delete calendars in the app, or only use existing ones? — existing calendars only; calendar management in the browser, 2026-10-22 (`DEC-21`)
- **Q14** How should read-only and subscribed calendars (holidays, birthdays, other people's calendars) appear and behave? — shown like the others, not editable; free/busy-only as blocks, 2026-10-22 (`DEC-22`)
- **Q22** Local cache: how far back and forward events are kept, any size limit, purge when an account is removed, and encryption at rest? — removal purge `DEC-20`; window one year back, two years ahead, no size limit; encryption `DEC-15`, 2026-10-22 (`DEC-23`)
- **Q36** When a calendar loses access (unshared, deleted, permission removed) while the account still works, what happens to it and its cached events in the app? — removed with its cached events, user told once, 2026-10-22 (`DEC-24`)
- **Q01** Are the workweek and agenda views part of the first release, or only if inexpensive? The PRD lists them as main views but also as optional and as an open decision. — agenda yes, workweek no; also a 3-day view, 2026-10-22 (`DEC-27`)
- **Q02** In "day, week, and month views", which week is meant: the full 7-day week, the workweek, or both? — the full week, 2026-10-22 (`DEC-27`)
- **Q17** "Advanced" search is out. Is any basic event search (by title) in the MVP? — no search in the MVP, 2026-10-22 (`DEC-30`)
- **Q19** Which user settings are needed: week start day, date/time format, display time zone, secondary time zone, default reminders, default calendar for new events? — week start, formats and time zone follow the system; default calendar chosen by the user (`SRC-02-23`, DSC-07); default reminders handled with `Q21`, 2026-10-22 (`DEC-29`)
- **Q20** Which UI language(s) must the app support, and should formats follow the system locale? — English only in the MVP; texts kept in one place for later translations, 2026-10-22 (`DEC-31`)
- **Q31** Is a UX/UI design (mock-ups) part of the delivery, or do the Morgen/Apple references guide implementation directly? — no mock-ups; refined iteratively on the working app, 2026-10-22 (`DEC-32`)
- **Q07** What do "basic recurring event management" and "common recurring event cases" include: which recurrence patterns can be created, and which edit scopes (this event / this and following / all)? — presets + this / this and following / all; complex rules edited in the browser, 2026-10-22 (`DEC-36`)
- **Q15** Should Google-specific event types (out-of-office, focus time, working location) and per-event colours be shown and editable? — types shown distinctly; out of office and focus time creatable where allowed; event colours shown and editable, 2026-10-22 (`DEC-34`, `DEC-35`)
- **Q16** Drive attachments are out. Should existing attachments on an event still be visible (e.g. as links)? — existing attachments shown as links; no adding, 2026-10-22 (`DEC-37`)
- **Q25** Is ICS import/export, or any migration from another calendar app, expected? — no ICS import or export in the MVP, 2026-10-22 (`DEC-37`)
- **Q06** Is Google Workspace directory search for attendees in the MVP, or only Google Contacts and recent addresses? — directory suggestions on Workspace accounts, 2026-10-23 (`DEC-39`)
- **Q09** What does "open a Google Meet link" mean: join an existing meeting from the event (open in the browser), or something more? — one-click Join button on the event, in the agenda and in the notification, 2026-10-23 (`DEC-38`)
- **Q18** "Advanced" free/busy scheduling is out. Is any basic attendee availability view in the MVP? — simple free/busy preview in the event editor; advanced scheduling out, 2026-10-23 (`DEC-42`)
- **Q30** When an event with guests changes, should the app always notify them, ask each time, or let the user choose a default? — ask every time, 2026-10-23 (`DEC-40`)
- **Q05** Should the app keep running in the background (tray, autostart) to deliver reminders when its window is closed, or only while it is running? — tray + optional autostart; reminders while the window is closed, 2026-10-23 (`DEC-44`)
- **Q21** Notification behaviour: default reminders for new events, snooze/dismiss, which Google reminder types are honoured (popup only, or e-mail too), and on/off per account or per calendar as well as globally? — defaults from Google, per-event changes synced, both forms; Dismiss + Snooze 5/10; global switch + per-calendar mute, 2026-10-23 (`DEC-43`, `DEC-45`)
- **Q27** Are auto-updates, single-instance behaviour and deep links (e.g. from notifications or `webcal:`/`mailto:` handlers) expected? — single instance; webcal: handled via the browser; no mailto:, no update checking, 2026-10-23 (`DEC-46`)
- **Q37** How are accounts managed in the app? — confirmed as proposed: settings list, confirmation on removal, unverified-app heads-up, 2026-10-23 (`DEC-51`)
- **Q38** How do account edge cases behave? — confirmed as proposed, 2026-10-23 (`DEC-52`)
- **Q39** How do the calendar and cache details behave? — confirmed as proposed, 2026-10-23 (`DEC-54`)
- **Q40** How do the remaining view details behave? — confirmed as proposed, 2026-10-23 (`DEC-56`)
- **Q41** How is "compact" and "restrained" measured? — confirmed as proposed, 2026-10-23 (`DEC-58`)
