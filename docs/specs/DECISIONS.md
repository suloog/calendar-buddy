<!-- Copy to docs/specs/DECISIONS.md. Content in the project language; headings fixed. -->

# Decisions — the project's decision log

> Every ruling that shapes the scope lands here, whoever made it: the client at a workshop or by
> e-mail, the analyst taking an assumption, the tech lead choosing between two readings, a
> developer who found a requirement impossible. Not only departures — every decision, so the log
> reads as the full history of how the scope got to where it is.
>
> **Append-only.** An entry is never edited away or renumbered. A ruling that changes later is a
> new entry whose **Status** on the old one becomes `superseded by DEC-nn`. `/og:analyze`,
> `/og:discovery`, `/og:modules`, `/og:requirements`, `/og:stories`, `/og:decide` and `/og:start-story`
> all append here as they work.
>
> **Decided by** — `client (<name or role>)` · `analyst` · `tech lead` · `developer`.
> **Source** — where the ruling comes from: `workshop YYYY-MM-DD` · `e-mail from … YYYY-MM-DD` ·
> `call YYYY-MM-DD` · `assumption` (the analyst's default, awaiting the client) ·
> `implementation` (found while building) — with the `SRC-NN` holding the notes when there is one.
> **Scope effect** — `within scope` (interprets the sources) · `extension` (goes beyond every
> formal source — commercial follow-up with the client) · `reduction` (drops or narrows a formal
> source item — the client consciously gives it up).
> **Status** — `accepted` · `to confirm with the client` · `rejected` · `superseded by DEC-nn`.
>
> An entry that answers a question names it under **Resolves**; the question moves to the
> "Resolved" section of QUESTIONS.md in the same run.


## [DEC-01] Audience: the product owner, possibly teammates in the organization; one kind of user

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-01`, `SRC-02-04`, `SRC-02`)
- **Affects:** `DSC-01-08`, `DSC-01-09`, PROJECT.md Actors
- **Decision:** The app is for the product owner's own use for now and may be shared with teammates in the same Google Workspace organization; no public release is planned. There is one kind of user — a person signed in with their own Google account, acting as organizer or attendee; there is no admin role in the app.
- **Why:** The app is a personal tool first; sharing within the team is enough.
- **Scope effect:** within scope
- **Resolves:** `Q12`
- **Status:** accepted

## [DEC-02] MVP is built and tested on Linux only

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-02`, `SRC-02`)
- **Affects:** `SRC-01-05`, `SRC-01-119`, `DSC-01-10`
- **Decision:** The MVP is built and tested on Linux only. The architecture keeps macOS and Windows possible (platform code behind interfaces), but nothing is built or tested for them in the MVP.
- **Why:** Linux is the product owner's platform; the PRD already makes macOS/Windows releases conditional on Linux going first.
- **Scope effect:** within scope
- **Resolves:** `Q03`
- **Status:** accepted

## [DEC-03] UI toolkit chosen by a feasibility check, Iced first

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-03`, `SRC-02`)
- **Affects:** `DSC-01-11`, SRC-01 *Noted* §10.1, §16
- **Decision:** The UI toolkit is chosen by a feasibility check in the first stories: Iced first; Slint and then Tauri (webview) are allowed if Iced fails the checks the PRD lists (smooth scrolling, overlaps, resizing, drag-and-drop, accessibility).
- **Why:** Keeps the Rust-native preference while not betting the product on one toolkit.
- **Scope effect:** within scope
- **Resolves:** `Q08`
- **Status:** superseded by DEC-33

## [DEC-04] Google access through an Internal OAuth app in the organization's Workspace

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-04`, `SRC-02-05`, `SRC-02`)
- **Affects:** `SRC-01-18`, `SRC-01-24`, `DSC-01-12`, `DSC-03`
- **Decision:** Google access goes through a GCP project owned by the product owner inside the organization's Google Workspace, with an Internal OAuth consent screen. Only accounts of that organization can sign in; no Google verification is needed. Personal Google accounts — and so "multiple accounts" beyond the one organization — are parked until a public release.
- **Why:** Sharing with the team needs no publication; an External app would need either Google verification or a weekly re-login (refresh tokens of unverified apps in testing mode expire after 7 days).
- **Scope effect:** reduction
- **Resolves:** `Q23`
- **Status:** superseded by DEC-10

## [DEC-05] What "everyday calendar tasks" means

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-06`, `SRC-02`)
- **Affects:** `SRC-01-04`, `SRC-01-129`, `DSC-01-13`
- **Decision:** "Everyday" means checking upcoming meetings, accepting invitations, and creating meetings with Google Meet. Opening Google Calendar in the browser for advanced operations is acceptable.
- **Why:** These are the reasons the product owner opens the browser today.
- **Scope effect:** within scope
- **Resolves:** `Q10`
- **Status:** accepted

## [DEC-06] Straight to the MVP, no prototype milestone, no dates

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-07`, `SRC-02`)
- **Affects:** SRC-01 *Noted* §15, `DSC-01-14`
- **Decision:** Delivery goes straight to the MVP; the §15 technical prototype is not a separate milestone (its risks are covered by the first stories), and there are no fixed dates.
- **Why:** Personal project; no external deadline.
- **Scope effect:** within scope
- **Resolves:** `Q33`
- **Status:** accepted

## [DEC-07] Acceptance by the product owner against §14; no support

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-08`, `SRC-02`)
- **Affects:** `SRC-01-120`…`SRC-01-129`, `DSC-01-15`
- **Decision:** The product owner accepts the MVP against the §14 acceptance criteria through day-to-day use. Support and maintenance after release are not in scope; fixes are ordinary backlog work.
- **Why:** Internal tool with one user.
- **Scope effect:** within scope
- **Resolves:** `Q32`, `Q35`
- **Status:** accepted

## [DEC-08] Licence deferred until a public release

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-09`, `SRC-02`)
- **Affects:** SRC-01 *Noted* §16
- **Decision:** No licence is chosen until the app goes public; the repository stays private until then.
- **Why:** No public release is planned (`DEC-01`).
- **Scope effect:** within scope
- **Resolves:** `Q28`
- **Status:** accepted

## [DEC-09] Owner of permissions and access (TQ18)

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-10`, `SRC-02-11`, `SRC-02`)
- **Affects:** TQ17, TQ18, PROJECT.md Stack & integrations
- **Decision:** Maciej owns the GCP project and the OAuth client on our side; the counterpart on the organization's side — the Workspace admin who allows the Internal app — is also Maciej.
- **Why:** There is no server infrastructure; the GCP project is the only environment.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-10] Google access through an External OAuth app in production, unverified

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-12`, `SRC-02-13`, `SRC-02`)
- **Affects:** `SRC-01-18`, `SRC-01-24`, `DSC-01-07`, `DSC-01-12`, `DSC-01-17`, `DSC-03`
- **Decision:** Google access goes through an OAuth app with user type External and publishing status "In production", without Google verification, in the product owner's GCP project. Any Google account can sign in — personal and organization accounts, several at once — as `SRC-01-18` states. Each user sees Google's "unverified app" warning once, and the app is capped at 100 users while unverified. Verification happens only before a public release. Replaces `DEC-04`.
- **Why:** Multiple accounts (personal and organization) are wanted in the MVP. Unlike testing mode, production status does not expire refresh tokens after 7 days, and for a personal/team tool the warning and the cap are acceptable.
- **Scope effect:** within scope
- **Resolves:** `Q23` (re-ruled)
- **Status:** accepted

## [DEC-11] Accessibility target: keyboard, contrast, HiDPI; no screen reader in the MVP

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-14`, `SRC-02`)
- **Affects:** `DSC-02-09`, `DEC-03`
- **Decision:** The MVP's accessibility target is full keyboard operation, readable contrast in both themes and correct HiDPI scaling. Screen-reader support is not an MVP goal, so it does not change the toolkit order of `DEC-03` (Iced first).
- **Why:** Personal tool; a screen-reader requirement would push the toolkit away from Iced.
- **Scope effect:** within scope
- **Resolves:** `Q11`
- **Status:** accepted

## [DEC-12] Linux test matrix and MVP package

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-15`, `SRC-02`)
- **Affects:** `SRC-01-95`, `SRC-01-108`, `DSC-02-10`
- **Decision:** The MVP is tested on Ubuntu 26.10 with KDE Plasma — mainly Wayland, with an X11 check — and ships as a .deb package. Flatpak and AppImage come after the MVP.
- **Why:** This is the product owner's own setup.
- **Scope effect:** within scope
- **Resolves:** `Q26`
- **Status:** accepted

## [DEC-13] No telemetry; local logs only

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-16`, `SRC-02`)
- **Affects:** `SRC-01-12`, `SRC-01-93`, `DSC-02-11`, TQ15
- **Decision:** No telemetry and no remote crash reporting. Logs are written locally in the user's directory, with a level the user sets, and never contain event contents or tokens.
- **Why:** Privacy-first; no backend to receive reports.
- **Scope effect:** within scope
- **Resolves:** `Q29`
- **Status:** accepted

## [DEC-14] Data scale for performance measurement

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-17`, `SRC-02`)
- **Affects:** `SRC-01-89`, `SRC-01-96`, `DSC-02-12`, TQ05
- **Decision:** Performance is measured on a profile of up to 5 accounts, up to 50 calendars and about 20,000 cached events. The product owner sets the thresholds after the first measurements, for three flows: start to first view, switching period or view, scrolling the week view.
- **Why:** A personal user with team calendars, with headroom.
- **Scope effect:** within scope
- **Resolves:** `Q34`
- **Status:** accepted

## [DEC-15] Data at rest and recovery

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-18`, `SRC-02`)
- **Affects:** `SRC-01-21`, `SRC-01-30`, TQ01, TQ10
- **Decision:** Tokens live in the OS credential store. The local SQLite cache has no app-level encryption; the user's disk encryption covers it. There are no backups: Google is the system of record and the cache is rebuilt by resynchronization; losing local settings is acceptable.
- **Why:** Nothing in the cache is unique; encryption would add key management for no gain on an encrypted disk.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-16] Personal data and regulations

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-19`, `SRC-02`)
- **Affects:** TQ11, TQ12
- **Decision:** Calendar data, including attendees' e-mail addresses, stays on the user's device and in Google. There is no backend and no processing by us, so the app has no regulatory duties beyond secure token storage and logs free of personal data. Cache retention is settled in `DSC-04` (`Q22`).
- **Why:** Single-user tool with a direct connection to Google.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-17] Required test levels

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-20`, `SRC-02`)
- **Affects:** `SRC-01-94`, TQ16, constitution.md
- **Decision:** Automated unit and integration tests cover synchronization, recurrence, event editing and RSVP. Acceptance is manual, by the product owner (`DEC-07`). No load testing.
- **Why:** As the PRD asks; the scale profile is small.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-18] Single-user desktop app with no server — what does not apply

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-21`, `SRC-02`)
- **Affects:** TQ03, TQ04, TQ06–TQ09, TQ19–TQ24
- **Decision:** The app is a single-user desktop application with no server of ours. Multi-factor authentication is Google's, security audits are not planned, and throughput, scalability, SLA, redundancy, web browsers and resolutions, iOS, Android, tablets and e-readers do not apply.
- **Why:** Follows from `DEC-01`, `DEC-02` and `SRC-01-12`.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-19] First run and start without network

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-24`, `SRC-02-25`, `SRC-02`)
- **Affects:** `DSC-03-06`, `DSC-03-07`, `DSC-04`
- **Decision:** First run with no account shows a welcome screen with one action, connect a Google account; the calendar appears after the first sync. On a first run without network, sign-in is impossible: the app says so and offers a retry when the network returns. A later start without network shows the calendar from the cache with a visible offline status (`DSC-04`).
- **Why:** Simplest path to a working calendar; offline browsing is in the MVP (`SRC-01-31`).
- **Scope effect:** within scope
- **Resolves:** `Q24`
- **Status:** accepted

## [DEC-20] Removing an account purges its local data

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-26`, `SRC-02`)
- **Affects:** `SRC-01-22`, `DSC-03-08`, `Q22`
- **Decision:** Removing an account deletes at once its tokens from the credential store, its cached calendars and events, and its local settings (calendar colours, default calendar). The Google account itself is untouched. The rest of `Q22` (cache window and size) stays open in `DSC-04`.
- **Why:** Nothing in the cache is unique (`DEC-15`); leftover data of a removed account has no use.
- **Scope effect:** within scope
- **Resolves:** `Q22` (part)
- **Status:** accepted

## [DEC-21] Existing calendars only — no calendar management in the app

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-28`, `SRC-02`)
- **Affects:** `SRC-01-25`, `SRC-01-118`, `DSC-04-09`
- **Decision:** The app works with calendars that already exist in Google. Creating, renaming and deleting calendars is not in the MVP; it is done in the browser.
- **Why:** An advanced operation in the sense of `DEC-05`.
- **Scope effect:** within scope
- **Resolves:** `Q13`
- **Status:** accepted

## [DEC-22] Read-only and subscribed calendars are shown, not editable

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-29`, `SRC-02`)
- **Affects:** `SRC-01-25`, `SRC-01-61`, `DSC-04-10`
- **Decision:** Read-only and subscribed calendars (holidays, birthdays, colleagues' calendars) are shown like the others, with visibility and colour; their events cannot be edited, and the app makes that clear. Calendars shared as free/busy only show busy blocks without details.
- **Why:** "All accessible calendars" (`SRC-01-25`) includes them; edits are refused by Google anyway.
- **Scope effect:** within scope
- **Resolves:** `Q14`
- **Status:** accepted

## [DEC-23] Cache window: one year back, two years ahead

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-30`, `SRC-02`)
- **Affects:** `SRC-01-30`, `SRC-01-31`, `DSC-04-11`
- **Decision:** The local cache holds events from one year back to two years ahead, with no separate size limit (the scale profile of `DEC-14` fits). Periods outside the window are fetched from Google when browsed online and are not available offline.
- **Why:** Covers everyday use offline while keeping the first sync and the cache small.
- **Scope effect:** within scope
- **Resolves:** `Q22` (rest)
- **Status:** accepted

## [DEC-24] A calendar that loses access is removed with a notice

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-31`, `SRC-02`)
- **Affects:** `SRC-01-23`, `SRC-01-25`, `DSC-04-12`
- **Decision:** When a calendar becomes inaccessible while its account still works (unshared, deleted, permission removed), it disappears from the app, its cached events are deleted, and the user is told once that the calendar is no longer available.
- **Why:** Stale data of a calendar the user no longer has is misleading.
- **Scope effect:** within scope
- **Resolves:** `Q36`
- **Status:** accepted

## [DEC-25] Offline means browsing only in the MVP

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-32`, `SRC-02`)
- **Affects:** `SRC-01-35`, `SRC-01-111`, `DSC-04-13`
- **Decision:** Without network the MVP allows browsing only; creating, editing and responding to invitations are unavailable and the app says why. A queue of offline changes comes after the MVP.
- **Why:** As the PRD's MVP offline boundary states; no public release is planned (`DEC-01`).
- **Scope effect:** within scope
- **Resolves:** `Q04`
- **Status:** accepted

## [DEC-26] Changes from elsewhere appear within about a minute

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-33`, `SRC-02`)
- **Affects:** `SRC-01-28`, `DSC-04-14`
- **Decision:** While the app is open, changes made elsewhere appear within about a minute, and right after the computer wakes or the network returns; manual refresh covers the rest. Push notifications from Google are not used — they would need a server of ours (`DSC-01-04`).
- **Why:** Fast enough for everyday use without a backend.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** superseded by DEC-53

## [DEC-27] MVP views: day, 3 days, full week, month, agenda

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-34`, `SRC-02-37`, `SRC-02`)
- **Affects:** `SRC-01-36`…`SRC-01-40`, `SRC-01-99`, `DSC-05-01`, `DSC-05-08`, `DSC-05-09`
- **Decision:** The MVP has day, 3-day, full-week, month and agenda views. The 3-day view is chosen by hand like any other view; there is no automatic switch in a narrow window. "Week" in `SRC-01-99` is the full week. The workweek view is not in the MVP.
- **Why:** The week view is the main one, the 3-day view serves smaller screens, and the agenda covers "checking upcoming meetings" (`DEC-05`).
- **Scope effect:** extension
- **Resolves:** `Q01`, `Q02`
- **Status:** accepted

## [DEC-28] Calendars grouped by account; two levels of hiding; disabled calendars are not synchronized

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-35`, `SRC-02-36`, `SRC-02-38`, `SRC-02`)
- **Affects:** `SRC-01-27`, `DSC-04-01`, `DSC-04-03`, `DSC-05-10`, `DSC-05-11`, `DSC-05-12`, `DSC-09`
- **Decision:** Calendars are always grouped by account, in the side panel and in the calendar picker. A quick toggle in the side panel hides a calendar's events but keeps it listed. Disabling a calendar in the app settings removes it from the side panel, the calendar picker and the views; it is re-enabled in the settings. A disabled calendar is not synchronized or cached and gives no reminders; the list of calendars itself is always fetched, so it can be re-enabled.
- **Why:** Many calendars; quick lists must stay short.
- **Scope effect:** extension
- **Resolves:** —
- **Status:** accepted

## [DEC-29] Regional formats and time zone follow the system

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-40`, `SRC-02`)
- **Affects:** `SRC-01-34`, `SRC-01-56`, `DSC-05-13`, `DSC-05-14`
- **Decision:** Week start and date/time formats follow the system's regional settings; there are no app settings for them. The calendar is shown in the system time zone; an event stored in another time zone also shows its own time in its details. A second time zone and choosing the display time zone by hand come after the MVP.
- **Why:** Simplest behaviour that is correct for a single user.
- **Scope effect:** within scope
- **Resolves:** `Q19` (with `SRC-02-23` for the default calendar; default reminders go to `Q21`)
- **Status:** accepted

## [DEC-30] No search in the MVP

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-39`, `SRC-02`)
- **Affects:** `SRC-01-113`
- **Decision:** There is no event search in the MVP, basic or advanced; searching is done in the browser.
- **Why:** An advanced operation in the sense of `DEC-05`.
- **Scope effect:** within scope
- **Resolves:** `Q17`
- **Status:** accepted

## [DEC-31] Interface in English only

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-41`, `SRC-02`)
- **Affects:** `DSC-06-05`
- **Decision:** The MVP interface is in English only. Texts are kept in one place so translations can be added later; no Polish version now. Dates and times follow the system (`DEC-29`).
- **Why:** Single user; translations add work on every screen.
- **Scope effect:** within scope
- **Resolves:** `Q20`
- **Status:** accepted

## [DEC-32] No mock-ups; the look is refined on the working app

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-42`, `SRC-02`)
- **Affects:** `SRC-01-02`, `SRC-01-49`, `SRC-01-50`, `DSC-06-06`
- **Decision:** There is no UI design deliverable. Morgen and Apple Calendar are the references; the look is refined iteratively on the working app and reviewed by the product owner.
- **Why:** Personal tool; the product owner is the only reviewer.
- **Scope effect:** within scope
- **Resolves:** `Q31`
- **Status:** accepted

## [DEC-33] UI toolkit check starts with egui, Iced as the fallback

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-43`, `SRC-02`)
- **Affects:** `DSC-01-11`, SRC-01 *Noted* §10.1, §16, `DEC-11`
- **Decision:** The UI toolkit is chosen by a feasibility check in the first stories, starting with egui (eframe on wgpu); Iced is the fallback. Slint and Tauri are no longer candidates. The checks are the PRD's: smooth scrolling, overlapping events, resizing, drag-and-drop, plus the accessibility target of `DEC-11`. Replaces `DEC-03`.
- **Why:** egui is proven on dense, complex Rust desktop interfaces (WordCraft renders its UI with egui on wgpu), drawing a custom week grid with drag-and-drop is straightforward in it, and it supports AccessKit. It stays Rust-native, as SRC-01 §2 prefers.
- **Scope effect:** within scope
- **Resolves:** `Q08` (re-ruled)
- **Status:** accepted

## [DEC-34] Google event types: shown distinctly; out of office and focus time can be created

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-44`, `SRC-02-46`, `SRC-02`)
- **Affects:** `SRC-01-56`, `DSC-07-09`, `DSC-07-10`
- **Decision:** Google's special event types — out of office, focus time, working location, birthdays — are shown distinctly in the views. Out-of-office and focus-time events can be created where the account allows it (Google offers them only on the primary calendar of Workspace accounts). Working location and birthdays are display-only.
- **Why:** Out of office matters in everyday work; Google limits these types to Workspace primary calendars.
- **Scope effect:** extension
- **Resolves:** `Q15` (types)
- **Status:** accepted

## [DEC-35] Event colours are shown and editable

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-47`, `SRC-02`)
- **Affects:** `SRC-01-52`, `DSC-07-11`
- **Decision:** An event's own Google colour is shown and can be changed in the editor; the change is saved to Google. Calendar colours remain local overrides (`DSC-04-03`).
- **Why:** Keeps colour coding consistent with the browser.
- **Scope effect:** within scope
- **Resolves:** `Q15` (colours)
- **Status:** accepted

## [DEC-36] Recurrence in the MVP: presets and three edit scopes

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-48`, `SRC-02`)
- **Affects:** `SRC-01-59`, `SRC-01-64`, `SRC-01-101`, `SRC-01-125`, `DSC-07-12`, `DSC-07-13`
- **Decision:** Recurring events are created from presets: daily, weekdays, weekly on chosen days, monthly, yearly; the series ends on a date, after a count, or never. Changing or deleting an occurrence offers three scopes: this event, this and following, all. Series with more complex rules created in Google are shown correctly; their rule is edited in the browser.
- **Why:** Defines "basic" and "common cases" of the PRD.
- **Scope effect:** within scope
- **Resolves:** `Q07`
- **Status:** accepted

## [DEC-37] Attachments as links; no ICS import or export

- **Date:** 2026-10-22
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-22 (`SRC-02-49`, `SRC-02`)
- **Affects:** `SRC-01-114`, `DSC-07-14`
- **Decision:** Existing attachments of an event are shown in its details as links that open in the browser; adding attachments is not in the MVP. There is no ICS import or export in the MVP.
- **Why:** Viewing is enough for everyday use; the rest is done in the browser (`DEC-05`).
- **Scope effect:** within scope
- **Resolves:** `Q16`, `Q25`
- **Status:** accepted

## [DEC-38] Join Google Meet with one click

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-23 (`SRC-03-04`, `SRC-03`)
- **Affects:** `SRC-01-126`, `DSC-08-10`, `DSC-09`
- **Decision:** An event with a Google Meet link has a clear Join button that opens the meeting in the browser; the same button is in the agenda view and in the notification before the meeting.
- **Why:** "Open a Meet link" (`SRC-01-126`) means joining fast.
- **Scope effect:** within scope
- **Resolves:** `Q09`
- **Status:** accepted

## [DEC-39] Guest suggestions include the Workspace directory

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-23 (`SRC-03-03`, `SRC-03`)
- **Affects:** `SRC-01-16`, `SRC-01-69`, `DSC-08-11`
- **Decision:** On Workspace accounts, guest suggestions also include the organization's directory (an extra read-only permission). On personal accounts: Google Contacts and recently used addresses only.
- **Why:** Colleagues are often not in personal contacts.
- **Scope effect:** within scope
- **Resolves:** `Q06`
- **Status:** accepted

## [DEC-40] Ask every time whether to notify guests

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-23 (`SRC-03-02`, `SRC-03`)
- **Affects:** `SRC-01-65`, `SRC-01-74`, `DSC-08-12`, `DSC-07-08`
- **Decision:** When a change to an event with guests is saved — including by dragging — the app asks once whether to send an update to the guests (yes / no).
- **Why:** Same behaviour as in the browser; the user decides case by case.
- **Scope effect:** within scope
- **Resolves:** `Q30`
- **Status:** accepted

## [DEC-41] Invitations in disabled calendars are not shown

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-23 (`SRC-03-05`, `SRC-03`)
- **Affects:** `DEC-28`, `DSC-05-12`, `DSC-08-13`
- **Decision:** Invitations that arrive in a calendar disabled in the settings are not visible; disabling is the user's choice. Invitations normally land in the primary calendar of the account.
- **Why:** Follows from `DEC-28`.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-42] Simple free/busy preview of guests in the MVP

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-23 (`SRC-03-06`, `SRC-03`)
- **Affects:** `SRC-01-112`, `DSC-08-14`
- **Decision:** The event editor shows, for the chosen day, the busy blocks of each guest and marks a conflict with the proposed time; guests who do not share availability show "no data"; not available offline. Advanced scheduling — a multi-day grid, dragging on it, suggesting free slots — stays outside the MVP.
- **Why:** Low cost (Google's free/busy query, no new permission) for a frequent need when inviting colleagues.
- **Scope effect:** extension
- **Resolves:** `Q18`
- **Status:** accepted

## [DEC-43] Reminders: defaults from Google, per-event changes synchronized, both forms

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-23 (`SRC-03-07`, `SRC-03-08`, `SRC-03`)
- **Affects:** `SRC-01-58`, `SRC-01-81`, `DSC-09-07`, `DSC-09-08`, `DSC-09-09`
- **Decision:** New events use their calendar's default reminders from Google (the event is created with "use defaults"); those defaults are changed in Google, not in the app. An event's own reminders can be changed in the app — both notification and e-mail forms, within Google's limit — and the change is saved to Google, so other devices follow; an event can go back to the calendar defaults. The app shows notifications for notification-type reminders; e-mail reminders are sent by Google.
- **Why:** One source of truth for reminders across devices.
- **Scope effect:** within scope
- **Resolves:** `Q21` (default reminders, reminder types)
- **Status:** accepted

## [DEC-44] Tray, autostart and reminders while the window is closed

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-23 (`SRC-03-09`, `SRC-03`)
- **Affects:** `SRC-01-85`, `SRC-01-105`, `SRC-01-127`, `DSC-09-10`
- **Decision:** Closing the window hides the app to the system tray, where it keeps synchronizing and delivering reminders; autostart at login is optional; the app is fully closed from the tray menu. No separate background process.
- **Why:** Reminders matter (`SRC-02-45`); the tray covers the closed-window case cheaply on KDE.
- **Scope effect:** extension
- **Resolves:** `Q05`
- **Status:** accepted

## [DEC-45] Snooze, dismiss and notification switches

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-23 (`SRC-03-10`, `SRC-03`)
- **Affects:** `SRC-01-82`, `DSC-09-11`, `DSC-09-12`
- **Decision:** A notification offers Dismiss and Snooze (5 or 10 minutes), plus Join for Meet events (`DEC-38`); snooze is local and not written to Google. Notifications are switched on and off globally, and can be muted for a chosen calendar without hiding it.
- **Why:** Avoids noise from holidays, birthdays and colleagues' calendars.
- **Scope effect:** within scope
- **Resolves:** `Q21` (snooze, switches)
- **Status:** accepted

## [DEC-46] Single instance and webcal links; no mailto, no update checking

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery session 2026-10-23 (`SRC-03-11`, `SRC-03`)
- **Affects:** `DSC-09-13`, `DSC-09-14`, TQ14
- **Decision:** Only one instance of the app runs; launching it again brings back the existing window. `webcal:` links opened in the system are handled by the app, which opens Google Calendar in the browser to add the subscription (the Calendar API cannot add URL subscriptions). The app does not handle `mailto:` links and has no update checking or automatic updates; a new .deb is installed by hand.
- **Why:** Cheap system integration that fits `DEC-21` (calendar management in the browser).
- **Scope effect:** within scope
- **Resolves:** `Q27`
- **Status:** accepted

## [DEC-47] Tokens in the platform's standard secret store

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery close 2026-10-23 (`SRC-03-12`, `SRC-03`)
- **Affects:** `SRC-01-21`, `DSC-02-05`, `DEC-15`, TQ01, constitution.md
- **Decision:** Tokens are stored through the platform's standard secret store behind one interface: on Linux the Secret Service API (KWallet, GNOME Keyring, KeePassXC, the Flatpak secret portal), Keychain on macOS and Credential Manager on Windows when those come. When no secret store is running, sign-in stops with a message explaining what to install or start; there is no fallback store in the MVP (an encrypted file can be added when the app is shared with the team).
- **Why:** The standard interface keeps other distributions, desktop environments and platforms open; plaintext storage is ruled out by `SRC-01-21`.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-48] Discovery close: actor, glossary and goals fixed

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** discovery close 2026-10-23 (`SRC-03-13`, `SRC-03`)
- **Affects:** PROJECT.md (Actors, Glossary, Scope, Goals, Milestones, Risks, Stack & integrations)
- **Decision:** The single actor is named "User"; organizer and guest are roles on an event, not actors. The glossary terms are fixed as written in PROJECT.md (among them hidden vs. disabled vs. muted calendar, and Maybe meaning Google's "tentative"). Goal G1 is measured over two consecutive working weeks of using only the app for upcoming meetings, invitations and meetings with Meet.
- **Why:** Names and terms are frozen at the close so modules and stories use them verbatim; the PRD names no actor and no measurable goal.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-49] Module split: one module per business topic, plus a deferred-scope module

- **Date:** 2026-10-23
- **Decided by:** tech lead (Maciej)
- **Source:** /og:modules 2026-10-23
- **Affects:** `MOD-01`…`MOD-08`, `DSC-03`…`DSC-09`
- **Decision:** Each business topic becomes one module, numbered by dependency: `MOD-01` accounts (`DSC-03`), `MOD-02` calendars and sync (`DSC-04`), `MOD-03` views and navigation (`DSC-05`), `MOD-04` appearance and themes (`DSC-06`, kept separate rather than merged into views and moved before events), `MOD-05` events (`DSC-07`), `MOD-06` guests, RSVP and Meet (`DSC-08`), `MOD-07` reminders and notifications (`DSC-09`). `MOD-08` holds the deferred scope — the items parked in discovery. An item mentioned by two topics belongs to the module that delivers it (`SRC-02-35` → `MOD-03`, `SRC-02-23` → `MOD-05`); partly delivered items stay with the delivering module (`SRC-01-112` → `MOD-06`, `SRC-01-114` → `MOD-05`). The week-view feasibility story of `MOD-03` may start in parallel with `MOD-01`/`MOD-02` on fixture data.
- **Why:** Merging `DSC-06` into views would exceed the module size limit; dependency order with the UI-toolkit risk addressed early.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-50] No company boilerplate: a Rust desktop stack

- **Date:** 2026-10-23
- **Decided by:** tech lead (Maciej)
- **Source:** /og:modules 2026-10-23
- **Affects:** constitution.md (Stack)
- **Decision:** The project uses no company boilerplate. The stack is Rust with egui (eframe on wgpu), Iced as the fallback (`DEC-33`), SQLite and the platform secret store, packaged as .deb.
- **Why:** The company boilerplates are web stacks; this is a native Linux desktop application (`DSC-01-01`), and the PRD prefers Rust.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-51] Account management in the settings

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-14`)
- **Affects:** `MOD-01-U01`, `MOD-01-U02`, `MOD-01-U03`
- **Decision:** Accounts are managed in one place in the settings, listing each account by e-mail address with its state and offering add, sign in again and remove; removal asks for confirmation and says what is deleted locally; before the browser opens, the app warns about Google's "unverified app" screen.
- **Why:** Confirms the default proposed for `Q37`.
- **Scope effect:** within scope
- **Resolves:** `Q37`
- **Status:** accepted

## [DEC-52] Account edge cases

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-14`)
- **Affects:** `MOD-01-F09`, `MOD-01-F14`, MOD-01 Constraints and edge cases
- **Decision:** Removing an account also withdraws the app's access at Google (the local deletion happens even offline); removing the last account returns to the welcome screen; with partial consent the account connects only if calendar access was granted, and features needing a missing permission show as unavailable; with the secret store locked and not unlocked, cached calendars are shown but nothing synchronizes or changes.
- **Why:** Confirms the default proposed for `Q38`.
- **Scope effect:** within scope
- **Resolves:** `Q38`
- **Status:** accepted

## [DEC-53] Adaptive background sync and sync on focus

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-15`)
- **Affects:** `DSC-04-14`, `MOD-02-F07`, `MOD-02-F23`, `MOD-02-NFR-01`
- **Decision:** Background synchronization is adaptive: calendars the user can edit are synchronized about every minute, read-only and subscribed calendars about every 15 minutes, and less often while the window is hidden in the tray. The app also synchronizes immediately when its window becomes active and before an event is opened for editing, as well as after waking and when the network returns. Push notifications from Google stay out (they need a server). Replaces `DEC-26`.
- **Why:** "Every calendar every minute" spends most requests on calendars that rarely change (holidays, birthdays, colleagues'); adaptive polling cuts the request volume about fourfold while keeping data fresh where and when the user looks.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-54] Calendar and cache details

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-16`)
- **Affects:** `MOD-02-F04`, `MOD-02-F07`, `MOD-02-F12`, `MOD-02-U03`, `MOD-02-U04`, MOD-02 Constraints and edge cases
- **Decision:** While the window is hidden in the tray, editable calendars are refreshed at least every 5 minutes. A local calendar colour is chosen from the calendar's menu in the side panel and can be reset to the Google colour. Events fetched outside the cache window while browsing online are not kept for offline use. The synchronization indicator shows the time of the last successful sync. An event crossing the window edge is kept whole. A damaged cache is discarded and rebuilt from Google.
- **Why:** Confirms the defaults proposed for `Q39`.
- **Scope effect:** within scope
- **Resolves:** `Q39`
- **Status:** accepted

## [DEC-55] Start view, agenda range and Google-style shortcuts

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** /og:requirements MOD-03 2026-10-23 (`SRC-03-17`, `SRC-03-18`, `SRC-03-19`)
- **Affects:** `MOD-03-F05`, `MOD-03-F15`, `MOD-03-F23`, `MOD-03-U04`
- **Decision:** The app starts in the view used last, on today's date. The agenda lists events from now on, grouped by day, skipping empty days and loading more while scrolling. Keyboard shortcuts follow Google Calendar's keys (views, next and previous period, today, go to date, new event, shortcut list).
- **Why:** Matches how the product owner works and keeps Google Calendar's muscle memory.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-56] View details

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-20`)
- **Affects:** `MOD-03-F11`, `MOD-03-F20`, `MOD-03-F21`, `MOD-03-U03`, `MOD-03-U05`, MOD-03 Constraints and edge cases
- **Decision:** The side panel remembers whether it was collapsed and collapses on its own when the window gets too narrow. A month-view day with too many events shows how many more there are and opens the day's list. All-day and multi-day events sit in a strip above the hours. The day, 3-day and week views open scrolled to the current time. With no calendar visible the view is empty and points to the side panel. An agenda with nothing ahead says there are no upcoming events.
- **Why:** Confirms the defaults proposed for `Q40`.
- **Scope effect:** within scope
- **Resolves:** `Q40`
- **Status:** accepted

## [DEC-57] Theme switching, accent colour and grid density

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** /og:requirements MOD-04 2026-10-23 (`SRC-03-21`, `SRC-03-22`, `SRC-03-23`)
- **Affects:** `MOD-04-F02`, `MOD-04-F03`, `MOD-04-U01`, `MOD-04-U03`
- **Decision:** The theme follows the system's light or dark setting by default and switches live; the settings offer System, Light and Dark. The app has one fixed, restrained accent colour of its own (not taken from the system), so calendar colours stay the only loud colours. The day and week grids have one compact density, like Morgen, with no density setting.
- **Why:** Restrained interface (`SRC-01-52`) and Morgen's density (`SRC-01-49`); advanced customization is outside the MVP (`SRC-01-117`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-58] Measures of a compact, restrained look

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-24`)
- **Affects:** `MOD-04-U01`, `MOD-04-U02`, `MOD-04-U05`, `MOD-04-NFR-01`, MOD-04 Constraints and edge cases
- **Decision:** In a 1080-pixel-high window at 100% scale the week view shows 8:00–18:00 without scrolling. Event blocks in the grid show only title and time. There are no transition animations when changing views or periods. Text contrast is at least 4.5:1 (WCAG 2.1 AA) in both themes. When the desktop reports no light/dark preference, the light theme is used.
- **Why:** Confirms the defaults proposed for `Q41`.
- **Scope effect:** within scope
- **Resolves:** `Q41`
- **Status:** accepted

## [DEC-59] Quick entry, default length, delete and undo

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** /og:requirements MOD-05 2026-10-23 (`SRC-03-25`…`SRC-03-28`)
- **Affects:** `MOD-05-F02`, `MOD-05-F03`, `MOD-05-F09`, `MOD-05-F27`, `MOD-05-F28`
- **Decision:** Clicking or selecting a range opens a small quick-entry box with the title, the time already filled in and the default calendar; Enter saves, "More options" opens the full editor with what was typed. A new event's default length comes from the user's Google Calendar settings. An event without guests that is not part of a series is deleted without a question, with Undo available for a few seconds. The last change to an event can be undone with Ctrl+Z or the notice's Undo; there is no full history.
- **Why:** Matches Google Calendar's behaviour the product owner is used to; undo replaces confirmation dialogs.
- **Scope effect:** extension
- **Resolves:** —
- **Status:** accepted

## [DEC-60] Event-editing details

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-29`)
- **Affects:** `MOD-05-F10`, `MOD-05-F29`, `MOD-05-U01`…`MOD-05-U04`, MOD-05 Constraints and edge cases
- **Decision:** Until the user picks one, the default calendar is the primary calendar of the first connected account. For a series with a complex rule the app offers to open it in Google Calendar to change the rule. Out-of-office events are hatched, focus-time events carry an icon, working-location and birthday events are labels in the all-day strip. Dragging and resizing move in 15-minute steps. A changed event is marked "saving" until Google confirms. The Undo notice stays 5 seconds. Dragging between the all-day strip and the hours converts the event. An event changed elsewhere meanwhile is reloaded and the user applies the change again.
- **Why:** Confirms the defaults proposed for `Q42`.
- **Scope effect:** within scope
- **Resolves:** `Q42`
- **Status:** accepted

## [DEC-61] Invitations, declined events, Meet switch and RSVP to series

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** /og:requirements MOD-06 2026-10-23 (`SRC-03-30`…`SRC-03-33`)
- **Affects:** `MOD-06-F09`, `MOD-06-F10`, `MOD-06-F13`, `MOD-06-F14`, `MOD-06-F17`
- **Decision:** Invitations awaiting a response are marked in the views and counted in the side panel; the counter opens a list to respond from. Declined events stay visible, struck through and faded. The event editor has an "Add Google Meet" switch that turns on by itself when the first guest is added and is off by default without guests. Responding to an occurrence of a recurring invitation asks whether the answer applies to this event or to all events.
- **Why:** Keeps invitations from being missed and Meet one step away for meetings with guests.
- **Scope effect:** extension
- **Resolves:** —
- **Status:** accepted

## [DEC-62] Guiding principle: a comfortable Linux interface to Google Calendar, not a better Google

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-37`)
- **Affects:** every module; `MOD-06-NFR-01`, `MOD-06-F22`
- **Decision:** Where a behaviour is open, the app follows what Google Calendar does rather than inventing its own; the goal is a comfortable Linux interface to Google's features, not to improve on them. Performance targets beyond the project-wide NFRs are tuned later, if needed (`MOD-06-NFR-01` withdrawn). Declined events are visible by default and can be shown or hidden with one easy switch.
- **Why:** The app is a personal/test tool; matching Google keeps it predictable and the scope small.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-63] Guest and invitation details

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-38`)
- **Affects:** `MOD-06-U01`…`MOD-06-U04`, MOD-06 Constraints and edge cases, Out of scope
- **Decision:** The guest list shows responses as icons; the availability preview is one horizontal strip per guest with the chosen time highlighted; an invitation's details open with Accept, Maybe and Decline at the top; an invitation awaiting a response has a dashed outline and a light fill; the guest list collapses beyond 10 guests and the preview shows the first 20; guest permissions are left at Google's defaults.
- **Why:** Confirms the defaults proposed for `Q43` (the suggestion-speed part withdrawn by `DEC-62`).
- **Scope effect:** within scope
- **Resolves:** `Q43`
- **Status:** accepted

## [DEC-64] Notification details

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** requirements review 2026-10-23 (`SRC-03-39`)
- **Affects:** `MOD-07-F16`, `MOD-07-F19`, `MOD-07-F20`, `MOD-07-U01`…`MOD-07-U03`, MOD-07 Constraints and edge cases
- **Decision:** Declined events produce no notifications. Reminders missed while the app was not running are not shown later; those that fell during sleep are shown on waking if the event has not started. Starting at login is off until turned on. A notification shows title, start time and location. Clicking the tray icon shows or hides the window. Muting is in the calendar's side-panel menu. Without notification buttons, Join and Snooze are reached by opening the event.
- **Why:** Confirms the defaults proposed for `Q44`, which follow Google's behaviour (`DEC-62`).
- **Scope effect:** within scope
- **Resolves:** `Q44`
- **Status:** accepted

## [DEC-65] Test cases on a "desktop" layer, smoke depth, without /og:run-tests

- **Date:** 2026-10-23
- **Decided by:** tech lead (Maciej)
- **Source:** /og:tests MOD-01 2026-10-23
- **Affects:** `docs/specs/modules/*/tests/`, `TESTPLAN.md`, `COVERAGE.md` part 2
- **Decision:** Test cases use `layer: desktop`, outside the template's admin/web/api/mobile set, because the app is a native desktop application that Playwright and Maestro cannot drive. There is no `.claude/og-test-targets.json` and `/og:run-tests` is not used. `automation: auto` means an automated test in the project's own suite (`cargo test`, against a fake Google API and a test secret store); `manual-only` means the product owner checks it by hand as part of acceptance (`DEC-07`) — the real Google consent screen, the browser, desktop notifications, the tray, the look. Depth is smoke (the happy path of every story) for now; criteria without a test carry a note in `COVERAGE.md` and full depth can be added per module later.
- **Why:** Follows from the desktop delivery form and test levels (`DEC-12`, `DEC-17`) and the web/mobile targets being n/a (`DEC-18`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-66] NFR-03 thresholds stay qualitative; the first measurement is the product owner's observation

- **Date:** 2026-10-23
- **Decided by:** tech lead (Maciej)
- **Source:** implementation (`/og:start-story US-03-01`)
- **Affects:** `NFR-03`, `MOD-03-NFR-01`, `US-03-01#5`, `TC-03-04`
- **Decision:** No numeric performance thresholds are set for now. The "first measurement" of `US-03-01#5` is the product owner's observation that the week view on the ~20,000-event profile scrolls and switches weeks without visible stutter on the target desktop (`DEC-12`). Numeric thresholds are added later only if a problem shows up.
- **Why:** The developer chose manual observation over an instrumented measuring mode; consistent with `DEC-62` (no performance targets beyond need for a personal tool).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-67] Daylight-saving days follow the clock

- **Date:** 2026-10-23
- **Decided by:** client (product owner — Maciej)
- **Source:** implementation (`/og:start-story US-03-01`, spec-compliance review)
- **Affects:** MOD-03 Constraints and edge cases, `US-03-01` edge cases, `MOD-03-F17`
- **Decision:** On a day with a daylight-saving change the hour grid stays 0:00–24:00 and events sit on their wall-clock hours, as Google Calendar draws them: the skipped hour stays empty and the two occurrences of the repeated hour share it. Replaces the edge case "the hours are drawn as they are on that day" (23 or 25 hours).
- **Why:** One hour grid with shared labels for the whole week can only be right on the clock; follows `DEC-62` (do as Google does).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-68] egui passes the feasibility check

- **Date:** 2026-10-23
- **Decided by:** tech lead (Maciej)
- **Source:** implementation — manual check of `US-03-01` (`TC-03-03`, `TC-03-04`)
- **Affects:** `DEC-33`, `US-03-01#4`…`#6`, `NFR-03`, constitution.md (Stack)
- **Decision:** egui (eframe 0.37 on wgpu) stays the UI toolkit; the Iced fallback is not needed. The week view on the ~20,000-event profile — scrolling, switching weeks, overlapping events, resizing and screen scaling, dragging an event — was judged fine by the product owner on Ubuntu 26.10 with KDE Plasma (Wayland). This observation is also the first NFR-03 measurement (`DEC-66`).
- **Why:** The product owner's check found no problem ("jest spoko").
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-69] Day and 3-day views: the 3 days start at the selected date; a plain picker until `US-03-05`

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-02
- **Affects:** `US-03-02`, `MOD-03-F01`, `MOD-03-F04`, `MOD-03-F07`
- **Decision:** The 3-day view shows the selected date and the two days after it, as Google Calendar's custom view does; the week view still shows the week containing the selected date. Previous and next move by the view's length (1, 3 or 7 days). Until `US-03-05`, the views are picked with three plain buttons (Day, 3 days, Week) in the bar above the grid, keeping the selected date; the app opens in the week view on today.
- **Why:** Follows Google Calendar (`DEC-62`) and the pass criterion of `TC-03-05`; the simplest way for the user to pick a view before the full navigation of `US-03-05` exists.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-70] Month view: whole weeks, an event on each day it covers, "+N more" opens a day list

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-03
- **Affects:** `US-03-03`, `MOD-03-F03`, `MOD-03-F20`, `MOD-03-F07`
- **Decision:** The month view shows the whole weeks covering the month (four to six rows, Monday first until `US-03-09`), the days of the neighbouring months dimmed, as Google Calendar does. Each day lists its events: all-day and multi-day first, then timed ones by start with their time; an all-day, multi-day or overnight event is listed on every day it covers, as a separate entry per day rather than one bar across the days. When not all fit, the day's last row shows "+N more", and choosing it opens a small window listing all of that day's events. ‹ › move by one month, and "Month" joins the plain view picker of `DEC-69`.
- **Why:** Follows Google Calendar (`DEC-62`) where it is cheap; one entry per day keeps "how many more" exact per day without lining bars up across a week, which the spec does not ask for.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-71] Agenda: events not yet ended, from today, 30 more days at a time

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-04
- **Affects:** `US-03-04`, `MOD-03-F05`, `MOD-03-F14`, `MOD-03-F22`
- **Decision:** "From now on" means every event that has not ended yet: a timed event whose end is after now (so one in progress stays), an all-day event that still covers today. Each event is listed on every day from today it covers, like the month view (`DEC-70`); within a day all-day and multi-day events come first, then timed ones by start with their start and end time. The agenda opens with the next 30 days and loads 30 more each time scrolling comes near the end of the list (or while the list is shorter than the window), until no event lies further ahead; only then does an empty agenda say "No upcoming events". Today's date is highlighted as in the other views; free/busy events read "Busy" with their time. Until `US-03-05`, "Agenda" joins the plain view picker of `DEC-69`; ‹ › are disabled in it, since the agenda always starts now.
- **Why:** The simplest reading of `MOD-03-F05` and `TC-03-07`; listing an ongoing meeting and showing multi-day events per day follows Google Calendar's schedule view (`DEC-62`). Loading by a day horizon keeps each step cheap on the ~20,000-event profile.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-72] Navigation: Today and ‹ › in the bar, a mini calendar in the side panel, the last view in a settings file

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-05
- **Affects:** `US-03-05`, `MOD-03-F06`, `MOD-03-F07`, `MOD-03-F08`, `MOD-03-F09`, `MOD-03-F15`
- **Decision:** The bar above the view holds "Today", ‹ ›, the period's title and, on the right, the view picker (Day, 3 days, Week, Month, Agenda), as Google Calendar lays it out; it replaces the plain picker of `DEC-69`. Switching views keeps the selected date; ‹ › move by the view's period (1, 3 or 7 days, or a month keeping the day of the month, the last day of a shorter month when it does not exist) and stay disabled in the agenda (`DEC-71`). The mini calendar sits at the top of a left side panel: one month in whole weeks, Monday first until `US-03-09`, today in the accent colour, the days on screen marked (the grid's days; the whole month in the month view; none in the agenda, which has no period). Its own ‹ › browse months without moving the view, and it follows the view again after any navigation. Picking a date shows it in the current view; in the agenda, which always starts now, it opens the day view of that date. The view used last is saved at every view change to `$XDG_CONFIG_HOME/calendar-buddy/settings` (`~/.config/…` when unset) as `key = value` lines; a missing or unreadable file means the week view, and the app always starts on today.
- **Why:** Follows Google Calendar (`DEC-62`) where the spec is silent; a plain text file is the simplest home for the app's settings until a settings screen exists, and losing it is acceptable (`DEC-15`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-73] Keyboard: Google's single keys, one dialog at a time, Tab through panel and events

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-08
- **Affects:** `US-03-08`, `MOD-03-F23`, `MOD-03-F24`, `MOD-03-U04`
- **Decision:** The shortcuts are the typed characters of `MOD-03` *Technical notes*, exactly as typed (`d` but not `D`), ignored while a text field has the keyboard. `t` also brings back a browsed mini calendar, like the Today button. Like Google Calendar, one dialog is open at a time over the view and Escape closes it: `?` lists every key with what it does, one row per key; `g` asks for a date in a field pre-filled with the selected date as `YYYY-MM-DD` (other formats wait for `US-03-09`), Enter or "Go" shows it as a date picked in the mini calendar does (`DEC-72`); `c` opens a "New event" dialog naming the selected day and saying creating events is not available yet, until `MOD-05`'s editor replaces it. Without a mouse, Tab and Shift+Tab (and egui's arrow-key focus moves) go through the bar, the side panel and the events on screen in every view, the focused one ringed and scrolled into sight; Enter or Space opens an event (as a click does) into a minimal details dialog with its title ("Busy" for free/busy calendars) and its time in the system's zone, which `US-03-09` and `MOD-06` extend. The agenda lets the keyboard reach the events within a screen of the visible part, so Tab scrolls on through the list.
- **Why:** Follows Google Calendar (`DEC-62`) where the spec is silent; opening an event needs something to open before `US-03-09`, and the smallest details dialog is enough for `MOD-03-F24`.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-74] Keyboard details: modifiers, Escape and the "+N more" list

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-08
- **Affects:** `US-03-08`, `MOD-03-F23`, `MOD-03-F24`
- **Decision:** Shortcut keys pressed with Alt (or Ctrl) do nothing. Closing an event's details gives the keyboard back to the event it was opened from. The month view's "+N more" list closes on Escape and when one of its events is opened, so one thing is open at a time (`DEC-73`). Only `t` brings back a browsed mini calendar; the other keys move it only when the date changes, as their buttons do.
- **Why:** Follows Google Calendar (`DEC-62`); keeps the user's place when moving without a mouse.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-75] Regional settings: the locale's `LC_TIME` data, read at start

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-09
- **Affects:** `US-03-09`, `MOD-03-F16`
- **Decision:** The locale of dates and times is the one the system sets for them, `LC_ALL`, else `LC_TIME`, else `LANG` (Plasma's Formats settings set `LC_TIME`); an unset or unknown locale is POSIX. Its data is glibc's, built into the app (`pure-rust-locales`): the first day of the week as glibc counts it (`week` and `first_weekday`), a 12-hour clock when the locale's time format is one (`3:00 PM`, otherwise `15:00`), always without seconds, the month before the day when the locale's numeric date runs month, day, year (`Thu, Nov 12, 2026`), the day first otherwise, also for locales writing the year first (`Thursday 12 November 2026`), and the locale's names of days and months, with a month on its own written in its standalone form (`listopad 2026`). The `g` dialog pre-fills and reads the locale's numeric date (`11/12/2026`) and still takes `YYYY-MM-DD` (`DEC-73`); a locale whose numeric date cannot be read back (written with names, or in forms the date library does not know) uses `YYYY-MM-DD` there. The settings are read when the app starts; a change of locale shows after a restart, as Plasma asks for a new session for it.
- **Why:** `DEC-29` gives no app settings, so the system locale is the only source; glibc's data is what the rest of the desktop uses, and reading it at start is the simplest correct behaviour.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-76] A change of time zone is noticed within seconds; another zone is a second line

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-09
- **Affects:** `US-03-09`, `MOD-03-F17`, `MOD-03-F18`
- **Decision:** The app looks up the system's time zone every 5 seconds off the UI thread and redraws the views in the new zone when it changes, keeping the selected date. An event's details show its time in the system's zone and, when the event is stored in another IANA zone (even one with the same offset), a second line with its time in that zone followed by the zone's name, as `3:00 PM – 4:00 PM (Europe/Warsaw)`. All-day events have no zone and show no second line.
- **Why:** The system does not tell an app that its zone changed, so it is looked up; 5 seconds is quick enough to follow and costs nothing when idle (`NFR-02`). Naming the zone by its IANA name is the simplest unambiguous label.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-77] The Google OAuth client is read from a file in the user's config directory

- **Date:** 2026-10-23
- **Decided by:** tech lead (Maciej)
- **Source:** setup before the MOD-01 delivery run
- **Affects:** `MOD-01-F01`, MOD-01 Constraints and edge cases, MOD-01 Technical notes, `US-01-01`
- **Decision:** The app reads its Google OAuth client (type Desktop app, in the owner's GCP project — `DEC-10`) from `$XDG_CONFIG_HOME/calendar-buddy/google-oauth-client.json` (default `~/.config/calendar-buddy/google-oauth-client.json`), exactly as downloaded from Google Cloud Console: `{"installed": {"client_id", "client_secret", "redirect_uris", …}}`. The file lives outside the repository; it is neither committed, nor an environment-file entry, nor built into the binary. When it is missing or unreadable, connecting an account stops and the app says which file to put where. Automated tests point the app at a test client file and the fake Google, never at the real one.
- **Why:** An app installed from a .deb has no environment file to read; Google treats a desktop client's secret as not confidential, but it has no place in git; one file the owner drops in once.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-78] The permissions asked when connecting; the directory in a second step on Workspace accounts

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-01-01
- **Affects:** `US-01-01`, `MOD-01-F03`, MOD-01 Technical notes, `NFR-05`
- **Decision:** Connecting asks every account for `openid`, `email`, `profile`, Google Calendar (`https://www.googleapis.com/auth/calendar`, needed to edit events, RSVP and subscribe to calendars later), contacts (`contacts.readonly`) and other contacts (`contacts.other.readonly`). Whether an account is a Workspace account is known only after it signs in (Google's `hd` claim), so a Workspace account is then sent to Google's consent screen a second time, for the directory (`directory.readonly`) alone, with the account pre-selected and earlier grants kept (`login_hint`, `include_granted_scopes`); a personal account never sees the directory permission. The account connects only when calendar access was granted; contact suggestions count as available only when both contacts permissions were granted. Declining or cancelling the second screen still connects the account, without directory suggestions; "Cancel" in the app cancels the whole connection. Every consent asks for offline access with `prompt=consent`, so Google always returns a refresh token.
- **Why:** One consent screen cannot differ between personal and Workspace accounts; a second, directory-only screen keeps `MOD-01-F03` ("only … and, on Workspace accounts, the directory") exact. The exact scope set will be narrowed by `MOD-02`, `MOD-05` and `MOD-06` if a smaller one suffices.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-79] Sign-in checks, waiting for the browser and the first-run messages

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-01-01
- **Affects:** `US-01-01`, `MOD-01-F10`, `MOD-01-F13`, `MOD-01-U03`, MOD-01 Constraints and edge cases
- **Decision:** "Connect Google account" first checks, off the UI thread, the OAuth client file, that a secret store runs (a Secret Service session can be opened) and that Google's token service can be reached (no connection within 5 seconds = offline); any failure is explained on the welcome screen with a "Try again" button that runs the checks again. Only then the "unverified app" heads-up is shown, with "Continue in the browser" and "Cancel". While the browser has the sign-in the app listens on the loopback port for up to 5 minutes and offers "Cancel"; a request that does not carry this sign-in's `state` (a favicon, a stray visit) is ignored. Google's `access_denied` counts as the user cancelling; any other error from Google — on the redirect or from its token service — is shown as Google's reason. The refresh token is saved in the secret store's default collection (unlocking it if needed), labelled with the e-mail address and keyed by Google's account ID.
- **Why:** Checking before the heads-up means the user does not read about Google's warning only to hit an error; 5 minutes leaves time to read Google's screens while not listening forever; Google gives no signal when the browser is closed, so the app's own "Cancel" covers it.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-80] Until US-01-02 and US-01-03, a connected account lives for the session and is announced in a notice

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-01-01
- **Affects:** `US-01-01`, `US-01-02`, `US-01-03`
- **Decision:** The connected account is kept in memory: the app does not yet remember it across restarts (that is `US-01-03`), so every start shows the welcome screen until then. After connecting, the calendar appears with a closable notice "Connected <e-mail>." that also names the features unavailable for the account because a permission was not granted; the account list in the settings comes with `US-01-02`. The calendar still shows the generated test events until synchronization exists (`MOD-02`).
- **Why:** Keeps the story to its scope (staying signed in and the account list are separate stories) while the account visibly "appears in the app".
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-81] The account list in the settings, its actions before US-01-03/04, and no cap on accounts

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-01-02
- **Affects:** `US-01-02`, `MOD-01-F02`, `MOD-01-U01`, `MOD-01-NFR-01`
- **Decision:** The settings open from a **Settings** button at the right end of the top bar, as a window whose "Accounts" section lists every account, in the order connected, by e-mail address with its state ("Connected" / "Needs sign-in"), and below the list **Add account**, which runs the same steps as the welcome screen (checks, the "unverified app" heads-up, the browser). Each row shows **Sign in again** and **Remove**, disabled with "Not available yet." until `US-01-03` and `US-01-04` give them their behaviour. A connection finishing while the settings are closed still adds the account, with the "Connected <e-mail>." notice. Signing in to an account already connected (same Google account ID) replaces its entry in place — new e-mail address and permissions, state back to "Connected" — and the secret store keeps only its newest refresh token. The app sets no maximum on accounts: `MOD-01-NFR-01`'s five is the profile it must support (`DEC-14`), like Google Calendar, which has no such cap.
- **Why:** One place for accounts as `DEC-51` asks, without building ahead the actions owned by the next stories; a cap would add a refusal nobody asked for.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-82] Google's account chooser on every new sign-in

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-01-02
- **Affects:** `US-01-02`, `MOD-01-F02`, `DEC-78`
- **Decision:** The first consent screen of a sign-in asks Google with `prompt=select_account consent`, so Google shows its account chooser even when the browser is already signed in to a connected account, and the user can pick or add another; the Workspace directory screen keeps `prompt=consent` with `login_hint`, as the account is already known.
- **Why:** Without the chooser Google may reuse the browser's current account, and a second account could not be connected; this is what Google's own sign-in offers apps that support several accounts.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-83] Accounts are remembered in a file and checked with Google at start and every 10 minutes

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-01-03
- **Affects:** `US-01-03`, `MOD-01-F04`, `MOD-01-F05`, MOD-01 Constraints and edge cases, `DEC-80`
- **Decision:** The connected accounts — Google's account ID, e-mail address, what was granted and the state ("connected" / "needs sign-in"), nothing secret — are kept in `$XDG_CONFIG_HOME/calendar-buddy/accounts` (default `~/.config/…`), so the app starts on the calendar, not the welcome screen, without the secret store or the network; this ends the session-only accounts of `DEC-80`. Off the UI thread the app reads every connected account's refresh token from the secret store (unlocking it if needed, once for all accounts) and asks Google's token service for an access token: at start, then every 10 minutes, and 1 minute after Google could not be reached. Google's `invalid_grant`, or no sign-in for the account in the secret store, makes the account "needs sign-in"; it is then not asked about again until the user signs in to it again. Any other failure (no network, Google's other errors) leaves the account connected. A missing client file, no running secret store or a store the user did not unlock leaves every account as it was, with a notice above the calendar that nothing synchronizes or changes until it is fixed, and "Try again"; the app does not ask to unlock the store again on its own. The access tokens are not kept until synchronization (`MOD-02`) needs them.
- **Why:** The account list must be there when the store is locked (`DEC-52`); a token refresh is how Google tells an installed app that its access was revoked, and checking regularly notices it while the app runs before synchronization exists; asking every 10 minutes costs nothing, and not re-prompting avoids a password dialog popping up over and over.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-84] "Needs sign-in" is told above the calendar and on the account; "Sign in again" asks Google for that account

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-01-03
- **Affects:** `US-01-03`, `MOD-01-F05`, `MOD-01-F06`, `MOD-01-F11`, `MOD-01-U01`, `DEC-81`
- **Decision:** While an account needs sign-in, a line above the calendar says "<e-mail> needs sign-in: Google ended Calendar Buddy's access to it. Your other accounts keep working." with **Sign in again**; in the settings the account's row shows "Needs sign-in" and **Sign in again** (only on such rows; **Remove** stays disabled until `US-01-04`). **Sign in again** opens the settings and runs the steps of adding an account — checks, the "unverified app" heads-up, the browser — with Google asked for that account (`login_hint`, `prompt=consent`); offline it says signing in is not possible offline and offers "Try again". The account comes back "Connected" in its place under the same Google account ID, so whatever the app keeps for it stays. If another account is picked in Google's screen, that one is connected as an added account and the first still needs sign-in.
- **Why:** The account's problem is shown on the account and apart from any calendar's problem (`MOD-01-F11`), and the user is told without opening the settings; Google's own sign-in prompts pre-select the account that needs it.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-85] Removing an account: asked in the settings, the credentials first, Google afterwards

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-01-04
- **Affects:** `US-01-04`, `MOD-01-F07`, `MOD-01-F08`, `MOD-01-F09`, `MOD-01-F14`, `MOD-01-U02`, `DEC-20`, `DEC-52`, `DEC-84`
- **Decision:** **Remove** on an account's row in the settings asks inside the settings window: "Remove <e-mail>? Its calendars, events, credentials and local settings will be deleted from this computer, and Calendar Buddy's access to it will be withdrawn at Google. The Google account itself stays untouched: its calendars and events stay in Google." with **Remove** and **Cancel**. Confirmed, the app deletes the account's items from the secret store (unlocking it if needed) off the UI thread, keeping the refresh token in memory if it can still be read (an unreadable item is deleted all the same); once they are gone, the app forgets the account in `~/.config/calendar-buddy/accounts` while the refresh token is sent to Google's token revocation endpoint, once, which nothing waits for and nothing retries, so offline the removal is complete locally and Google's grant stays until the user withdraws it in the Google account. If the secret store is not running or does not delete the items (e.g. it stays locked), the account stays listed and the settings say why — also when they were closed meanwhile — with **Try again**. One removal at a time, and never while an account is being connected or signed in again: while one runs or is asked about, **Add account** and **Sign in again** are not offered, and while connecting, **Remove** is disabled. Removing the last account closes the settings and shows the welcome screen. The app keeps no calendar or event cache and no per-account settings yet (synchronization is `MOD-02`; the events shown are test data of no account), so today an account's local data is its line in the accounts file and its credentials; whatever later stories keep per account (cache, calendar colours, default calendar) is deleted in the same step.
- **Why:** Deleting the credentials before telling Google means the local deletion never waits for the network (`DEC-52`); a store that refuses to delete would otherwise leave a credential behind for an account the app no longer lists, which `MOD-01-F08` rules out. A question inside the settings keeps one window open at a time (`DEC-73`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-86] The calendar list as Google Calendar lists it

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-01
- **Affects:** `US-02-01`, `MOD-02-F01`, `MOD-02-F02`, `MOD-02-F03`, `MOD-02-F20`
- **Decision:** Each account's calendars are its Google calendar list (`calendarList.list`, Google's defaults): its own, shared and subscribed calendars, of every access level, free/busy ones included; a calendar the user hid from the list in Google ("Hide from list") stays out, as in Google Calendar. A calendar is named as Google shows it (the user's own name for a shared calendar, `summaryOverride`, before its owner's `summary`) and drawn in the list entry's `backgroundColor`. A calendar on the lists of two accounts (e.g. the same holiday calendar subscribed in both) is kept and shown once per account, each copy under its account. Events of a free/busy calendar are marked busy-only, since Google sends them without details; an event without a title is shown as "(No title)", as Google Calendar shows it.
- **Why:** Google Calendar is the reference (`DEC-62`); a hidden calendar is one the user chose not to see. Merging copies across accounts would need a rule Google does not have, as each account is a separate Google Calendar there.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-87] First full synchronization: when it runs, the cache file and a removed account's data

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-01
- **Affects:** `US-02-01`, `MOD-02-F11`, `MOD-02-F13`, `MOD-02-NFR-02`, `NFR-01`, `NFR-08`, `DEC-23`, `DEC-83`, `DEC-85`
- **Decision:** The cache is an SQLite file, `$XDG_CACHE_HOME/calendar-buddy/cache.sqlite3` (default `~/.cache/…`), holding each account's calendars and their events as Google keeps them (series as master plus changed or cancelled occurrences); a damaged file is deleted and started anew. One worker thread owns it: at start it shows what the cache holds and drops the data of accounts no longer in the accounts file; each account's first full synchronization runs once per app run, as soon as Google has confirmed the account's sign-in in the account check of `DEC-83` (at start, and right after connecting or signing in again), so the secret store is unlocked once. It fetches the calendar list and every calendar's events from one year back to two years ahead of today in the system's time zone (`timeMin`/`timeMax`), one calendar after another, and replaces what the cache held for the account; calendars gone from the list go with their events. A calendar Google refuses keeps its cached events and does not stop the others; an account whose synchronization did not finish keeps its cached data and is synchronized again after its next successful check. Removing an account deletes its cached calendars and events right after its credentials (`DEC-85`). Until `US-02-03`, nothing else refreshes the cache: the window moves, and changes made elsewhere arrive, at the next start.
- **Why:** The UI never waits on network or disk (`NFR-01`); synchronizing only accounts Google has just accepted avoids a second unlock prompt and requests bound to fail. A cache rebuilt from Google belongs in the XDG cache directory (`NFR-17`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-88] Expanding series into occurrences

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-01
- **Affects:** `US-02-01`, `MOD-02-F22`
- **Decision:** A series is expanded by the `rrule` library from the master's start in the master's IANA zone and Google's `RRULE`, `EXDATE` and `RDATE` lines, into the occurrences that overlap the cache window; each lasts as long as the master. An occurrence Google sends on its own (moved or otherwise changed) takes the place of the one with the same original start, and a cancelled one removes it; an occurrence sent without its series (an invitation to one occurrence) is shown on its own. Occurrences get Google's IDs (`<series>_<UTC start>` or `<series>_<date>`). A timed series' end (`UNTIL`) kept by Google as a date or a local time is read in the series' zone (a date as the end of that day) and handed to the library in UTC, which is the only form it takes; an all-day series' end is read as its date alone, whatever time comes with it. A series whose rule the library does still not accept, or whose first start is an ambiguous local time (the repeated hour of a daylight-saving change), shows only its first occurrence. An occurrence moved from inside the window to outside it is not sent by Google, so its original time is shown until it is synchronized in a later story.
- **Why:** The constitution requires a tested library for recurrence and times kept as UTC plus the IANA zone; Google's IDs keep occurrences addressable for editing in `MOD-05`.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-89] How events that cannot be changed are marked

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-02
- **Affects:** `US-02-02`, `MOD-02-F19`, `MOD-02-F20`, `MOD-02-U01`, `DEC-22`, `DEC-86`
- **Decision:** An event is read-only when Google gives the account `reader` or `freeBusyReader` access to its calendar (`owner` and `writer` may change it); the access is taken from the calendar list at every synchronization, so a calendar whose access dropped is marked from the next one on. Read-only events look like the others, in their calendar's colour, with a lock (🔒) before the title (or before "Busy" for a free/busy event) in the day, 3-day, week, month and agenda views; opening one shows "🔒 Read-only: this event can't be changed" under its time. A free/busy event shows "Busy" and nothing of its own, also when opened. Calendars get no mark of their own until a view lists them: the calendars' half of `MOD-02-U01` moves to the side panel (`US-03-06`, which now covers it with its own criterion).
- **Why:** `MOD-02-U01` asks for a visible mark; Google Calendar shows such events in their colour and only leaves out the edit controls when one is opened, so the app keeps the events as Google draws them and adds the smallest mark that is visible in every view (`DEC-62`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-90] Ongoing synchronization: cadence, triggers and sessions

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-03
- **Affects:** `US-02-03`, `MOD-02-F07`, `MOD-02-F08`, `MOD-02-F09`, `MOD-02-F23`, `MOD-02-NFR-01`, `DEC-53`, `DEC-54`, `DEC-87`
- **Decision:** After an account's first synchronization of the run (`DEC-87`), each of its calendars is polled on its own: `owner`/`writer` every minute, `reader`/`freeBusyReader` every 15 minutes; while the window is hidden, every 5 and every 30 minutes. Until the tray exists (`US-07-05`), a minimized window counts as hidden. "Refresh" (a button in the top bar), waking from sleep and the network's return synchronize every calendar at once; activating the window synchronizes every calendar not synchronized in the last 10 seconds; opening an event synchronizes its calendar alone — until the editor of `MOD-05` exists, opening an event's details counts as opening it for editing. A calendar that fails is tried again at its next turn; an account whose sign-in Google turns away is no longer polled until a check finds it working. Each account signs in once per run (the secret store is read once) and renews its access token after 45 minutes or when Google refuses it. The views are rebuilt only when a round changed something.
- **Why:** The cadences are those of `DEC-53`/`DEC-54`; 30 minutes for hidden read-only calendars is the "less often while hidden" of `DEC-53` applied to them too. The 10-second gap keeps window switching from spending requests: even with the window activated every second, 50 editable calendars on one account stay under half of Google's 600 requests per minute per user (`MOD-02-NFR-01`). Syncing only the opened event's calendar gives the editor fresh data without a whole round per click.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-91] Incremental synchronization and wake-up signals

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-03
- **Affects:** `US-02-03`, `MOD-02-F08`, `MOD-02-F10`, `MOD-02-NFR-01`, MOD-02 *Technical notes*
- **Decision:** Each calendar's `nextSyncToken` from its full synchronization is kept in the cache (a new `sync_tokens` table, dropped with its calendar or account). Later synchronizations, including the first of each run, ask Google only for changes (`syncToken`, no time range); changed events inside the window, series and changed or cancelled occurrences are kept, events deleted or moved out of the window are removed (a deleted series with its occurrences). When Google answers 410 Gone, that calendar is fetched in full again within the window and its new token kept. Each calendar is asked for in its own request rather than in one HTTP batch per round: Google counts every request inside a batch against the same quotas, so a batch would not help `MOD-02-NFR-01`, and incremental answers are small. Waking and the network's return come from systemd-logind (`PrepareForSleep(false)`) and NetworkManager (`StateChanged` to connected-global) on the system bus, behind the `Wakeups` platform interface; without a system bus nothing is watched.
- **Why:** `MOD-02-F10` and the module's technical notes; one request per calendar is the simplest form that keeps within Google's limits.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-92] A sync token holds only for the window it was taken in

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-03
- **Affects:** `US-02-03`, `MOD-02-F10`, `MOD-02-F11`, `DEC-90`, `DEC-91`
- **Decision:** Refines `DEC-91`: a calendar's `syncToken` is kept together with the window its events were fetched in, and is used only while the window is the same. Once the window moves (the day changes, or the time zone), the calendar's next synchronization is a full one within the new window, which also brings events that entered the window unchanged and drops those that left it. The worker follows the window the views compute, so polling rounds use today's. Refines `DEC-90`: "Refresh", waking and the network's return also try again the first synchronization of an account that did not get through (no network, a locked store), and an access token Google refuses is renewed and the request repeated once, so the first round after sleep does not fail.
- **Why:** Changes alone never bring an unchanged event that the moving window reaches; one full fetch per calendar per day keeps the cache as `MOD-02-F11` describes at little cost to `MOD-02-NFR-01`. Found in code review.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-93] The calendar list is refreshed with each polling round; a calendar gone is told of in the app

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-04
- **Affects:** `US-02-04`, `MOD-02-F05`, `MOD-02-F21`, `DEC-24`, `DEC-90`
- **Decision:** Every synchronization round (polling, "Refresh", activating the window, opening an event, waking, the network's return) first fetches the calendar list of each account it synchronizes calendars of, one request per account, as does each account's first synchronization of a run. A calendar new on the list is synchronized in full at once and polled from then on; a calendar whose access changed is polled at its new rate; a calendar no longer on the list goes from the cache with its events, whatever the reason (deleted, unshared, access removed, or hidden from the list in Google, which Google's list does not tell apart), disabled ones included. The user is told once, in a banner at the top of the window ("The calendar “…” is no longer available.", closed with ✕), not with a desktop notification; a calendar gone while the app was closed or offline is told of at the account's next synchronization. A list Google does not send leaves the calendars as they were. An account whose every calendar is disabled has no rounds, so its list is refreshed at the next start.
- **Why:** The account's primary calendar is always editable and polled every minute, so the list stays as current as the editable calendars at one extra request per account per round, well within `MOD-02-NFR-01`. An in-app notice is the simplest form of "the user is told once" (`DEC-24`); desktop notifications are reminders (`MOD-07`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-94] Disabled calendars are kept in the app's settings, per account

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-04
- **Affects:** `US-02-04`, `US-03-07`, `MOD-02-F06`, `MOD-03-F13`, `DEC-28`, `DEC-85`
- **Decision:** The calendars disabled in the settings are kept in the app's settings file (`~/.config/calendar-buddy/settings`), one `disabled = <account ID> <calendar ID>` line each, so the same calendar on two accounts is disabled on each separately, like its place in the side panel under its account. They are forgotten when their account is removed. A disabled calendar stays in the cache's calendar list, its events and sync token are deleted (at start and when it is disabled) and it is not polled; enabling it synchronizes it in full at once. The synchronization worker takes the set at start and on every change (`Synchronization::set_disabled`); the switch that changes it is `US-03-07`, until which the line can be added to the settings file by hand.
- **Why:** The settings are where the user's own choices live (as local colours will), and the cache can be discarded and rebuilt from Google at any time, which must not enable calendars again.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-95] Offline: how the app knows, what it says, and days outside the cache

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-05
- **Affects:** `US-02-05`, `MOD-02-F12`, `MOD-02-F13`, `MOD-02-F14`, `MOD-02-F15`, `MOD-02-U02`, `DEC-19`, `DEC-23`, `DEC-25`, `DEC-54`
- **Decision:** The app is offline when NetworkManager's state on the system bus is below full connectivity (`NM_STATE_CONNECTED_GLOBAL`), behind the `Network` platform interface; an unknown state, no system bus and the moments before NetworkManager answers count as online. Offline, a band under the top bar says "Offline — your calendars are shown as last synchronized." and "Creating, editing and deleting events and answering invitations are not available offline."; the new-event dialog and the details of an editable event give the same reason, and events cannot be dragged. Requests to Google go on as before and fail quietly. The days on screen (the grid's days, the month's whole weeks, the agenda's days from today) that the cache window does not hold are, online, fetched from the enabled calendars of the signed-in accounts as one span when browsed (asking Google for a month more on each side, so an occurrence moved out of the span does not show at its old time; of the spans browsed while the worker was busy only the last is fetched), again after "Refresh" and after an account's first synchronization, shown with the cached events (an event crossing the window edge once) and dropped when browsing away; nothing of them is written to the cache. Offline, those days are empty with the note "Days more than a year back or two years ahead are not available offline." above the view.
- **Why:** NetworkManager is how KDE Plasma on Ubuntu knows the network, and its connectivity check tells a captive portal from the internet; a quiet failure of the background requests is what Google Calendar does offline (`DEC-62`). One span per screen is the simplest reading of "fetched as needed" in `MOD-02-F12`.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-96] Recovering from failures: growing waits with jitter, rate limits told apart

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-06
- **Affects:** `US-02-06`, `MOD-02-F16`, `MOD-02-F18`, MOD-02 *Technical notes*, `DEC-90`
- **Decision:** A calendar whose synchronization fails, for any reason (no connection, a rate limit, an error from Google), keeps what the cache holds and is tried again on its own: after its first failure at its usual interval, then each wait doubles, up to 30 minutes, and each wait gets a random extra of up to a quarter so calendars that failed together do not retry together. Once it gets through it is polled at its interval again. Activating the window does not cut a calendar's wait short; "Refresh", waking from sleep, the network's return and opening one of its events try it at once. The other calendars keep their own turns. Google's answer is a rate limit when it is 429, or 403 with the reason `rateLimitExceeded` or `userRateLimitExceeded`. An account whose first synchronization of the run fails is tried again as before (`DEC-90`).
- **Why:** The module's technical notes ask for exponential back-off with jitter on rate limits; one rule for every failure is the simplest. Starting from the calendar's own interval means a single blip costs nothing extra, and the 30-minute cap is the longest polling interval, so a calendar is never left longer than a hidden read-only one. Activation would otherwise undo the back-off at every window switch.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-97] The synchronization indicator and each calendar's problem

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-06
- **Affects:** `US-02-06`, `MOD-02-F17`, `MOD-02-F18`, `MOD-02-U03`, `US-03-06`, `DEC-54`, `DEC-95`
- **Decision:** The top bar, left of "Refresh", says "Synchronizing…", "Up to date", "Offline", "Synchronization delayed: Google asked to slow down" or "Synchronization error" (the last two in the warning colour), and "Last synchronized at 14:05" (with the date when not today; "Not synchronized yet" before the first). Offline comes first, then a synchronization under way, then an error, then a delay. The last successful synchronization is the last time any calendar's synchronization got through; it is kept in the cache (a new `last_synced` table) so it shows at the next start, offline too. Until the side panel lists the calendars (`US-03-06`), each calendar that is not up to date is named in a band under the top bar: "“Work” could not be synchronized; trying again.", "“Work” is synchronized later: Google asked to slow down." or, for the calendars of an account that needs sign-in, "“Work” is not synchronizing: its account needs sign-in." — and `US-03-06` shows the same on the calendar's row. A calendar of an account whose first synchronization of the run did not get through counts as failed; a calendar not synchronizing counts as an error. Offline only the calendars not synchronizing are named, as other failures are expected then (`DEC-95`). Disabled calendars are never named.
- **Why:** The four states are those of `MOD-02-F17`; the delayed one is the rate-limit edge case. "Last synchronized" by any calendar keeps the time meaningful when one calendar fails for good, since that calendar is named on its own. Naming the calendar in the band is the closest the app can come to "shown on that calendar" before it lists calendars anywhere.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-98] The indicator before an account's first synchronization of the run

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-06
- **Affects:** `US-02-06`, `MOD-02-F17`, `DEC-52`, `DEC-90`, `DEC-97`
- **Decision:** Until every connected account with cached calendars has had its first synchronization of the run (`DEC-90`), the indicator says "Synchronizing…", never "Up to date" (a calendar in error or delayed is said instead). An account saved as needing sign-in in an earlier run is not checked (`MOD-01`), so at start its cached calendars are marked as not synchronizing, as when Google turns it away during the run. When Google could not be asked whether such an account's sign-in still works, its calendars count as failed (named in the band, "Synchronization error"), and "Refresh" tries it again. While the accounts cannot be checked at all (no client file, no secret store, a locked one — `DEC-52`), the indicator says "Synchronization error" when online.
- **Why:** The code review of `US-02-06` found the indicator saying "Up to date" when nothing had been synchronized in the run; "so that I can trust what I see" rules that out.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-99] Local calendar colours: per account, Google's palette, a list in the side panel until US-03-06

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-02-07
- **Affects:** `US-02-07`, `MOD-02-F04`, `MOD-02-U04`, MOD-02 *Technical notes*, `DEC-54`, `DEC-85`, `DEC-94`
- **Decision:** A local colour is kept in the app's settings file (`~/.config/calendar-buddy/settings`), one `color = <account ID> <calendar ID> #rrggbb` line per calendar, so the same calendar on two accounts has a colour on each, like its Google colour (one per account's calendar list) and like `disabled` (`DEC-94`); the lines go when their account is removed (`DEC-85`). Nothing about it is sent to Google. The side panel gets, under the mini calendar, a list of the connected accounts' calendars (those disabled in the settings left out), each with its colour and a **…** menu (Google's **⋮**, which the default fonts lack) holding Google Calendar's 24 calendar colours (Radicchio … Birch, named on hover, the chosen one ticked) and **Reset**, enabled while the calendar has a local colour; picking or resetting recolours the calendar and its events in every view at once. Google's custom-colour picker ("+") is not offered. Grouping the list by account, hiding calendars and collapsing the panel stay with `US-03-06`.
- **Why:** The module's technical note keys colours by calendar ID; adding the account follows Google, where a calendar's colour belongs to each account's list entry (`DEC-62`), and lets removing an account forget them. The side panel is `MOD-03`, but `MOD-02-U04` needs a calendar's menu in it, so the smallest list that carries that menu is added now. Google's palette is what its calendar menu offers.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-100] The side panel: its button, collapsing in a narrow window, hidden calendars in the settings

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-06
- **Affects:** `US-03-06`, `MOD-03-F10`, `MOD-03-F11`, `MOD-03-F12`, `MOD-03-U03`, MOD-03 *Constraints and edge cases*, `DEC-28`, `DEC-56`, `DEC-72`
- **Decision:** Like Google Calendar's main menu button, a **☰** button at the far left of the top bar collapses and expands the side panel. The user's choice is kept in the settings file as `side_panel = collapsed` (no line when expanded) and the app starts that way. Below a window width of 800 points the panel collapses on its own. This is not saved. **☰** still opens it while the window is narrow, and once the window is wide again the panel is back as the user left it. Under the mini calendar the panel lists each connected account by its e-mail address, in the order the accounts were connected. Under each account come its calendars in Google's order, those disabled in the settings left out. Each calendar's quick toggle is Google's checkbox in the calendar's colour: filled and ticked while its events are shown, an empty box while they are hidden. It is reached and used from the keyboard like a button. The calendars hidden by the toggle are kept as `hidden = <account ID> <calendar ID>` lines and forgotten with their account. When the calendar list is known and every calendar is hidden or disabled, the view says "No calendars are shown. Choose calendars in the side panel." above its empty grid.
- **Why:** Google Calendar puts the drawer toggle top left and lets its checkboxes hide calendars (`DEC-62`). Not saving the automatic collapse keeps "remembers which" (`MOD-03-F11`) about what the user chose. 800 points still leaves a readable week next to the panel, and the window's minimum width is 480. Before the first synchronization there are no calendars, so an empty list is not a reason to point to the panel.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-101] A calendar's row: the lock, its synchronization problem, and the emoji fonts

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-06
- **Affects:** `US-03-06`, `MOD-02-U01`, `MOD-02-F18`, `DEC-89`, `DEC-97`, `DEC-99`
- **Decision:** A calendar the account cannot change (`reader` or `freeBusyReader`) has the lock 🔒 before its name in the side panel, as its events have before their titles (`DEC-89`), with "Read-only: this calendar can't be changed" on hover. A calendar that is not up to date says so in small warning-coloured text under its row: "Could not be synchronized; trying again.", "Synchronized later: Google asked to slow down." or "Not synchronizing: its account needs sign-in." The same offline rule as before applies: offline only the last is shown. The band under the top bar that named these calendars (`DEC-97`) is gone. The app turns on egui's `monochrome_emoji_fonts`. Since egui 0.37 its default fonts leave out the emoji fonts, so 🔒, ✔ and ☰ were not drawn. A test now checks that the app's fonts have them.
- **Why:** `DEC-97` moves each calendar's problem from the band to its row once the panel lists calendars, and the name is already on the row. Without the emoji fonts the lock of `MOD-02-U01` would not be visible anywhere, neither in the panel nor on events.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-102] Disabling calendars: a checkbox list in the settings, at once in the views, one source for the pickers

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-03-07
- **Affects:** `US-03-07`, `MOD-03-F13`, `MOD-05-F10`, `DEC-28`, `DEC-94`
- **Decision:** The settings get a **Calendars** section under **Accounts**: each connected account's e-mail address in bold, in the order the accounts were connected, and under it all its calendars in Google's order, disabled ones included, each with a checkbox ticked while it is enabled. Unticking disables the calendar, ticking enables it again; hovering says "A disabled calendar is not shown in the side panel or the views, and is not synchronized". The list scrolls once it is taller than 320 points. The change is saved as the `disabled` lines of `DEC-94` and handed to the synchronization worker. The calendar's events leave the views in the same frame, filtered by the settings like hidden calendars, without waiting for the worker to delete them. No calendar picker exists yet. The new event's and the default calendar's (`MOD-05-F10`) are to start from `side_panel::pickable`, which groups the calendars by account and leaves out those disabled; which of those a picker offers, such as only the calendars the account can change, is left to `MOD-05`.
- **Why:** Google Calendar has no disabled calendars; its settings list every calendar, and a checkbox is the smallest control that disables one and enables it again (`DEC-62`). Filtering in the views keeps a disabled calendar out of them at once, also offline, when the worker cannot reach Google. One function for the pickers keeps `MOD-05` from listing a disabled calendar.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-103] The quick-entry box: 15-minute steps, the `c` key at the next full hour, the default length read once per run, "Saving…"

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-01
- **Affects:** `US-05-01`, `MOD-05-F01`, `MOD-05-F02`, `MOD-05-F09`, `MOD-05-F16`, `MOD-05-U03`, `DEC-59`, `DEC-60`
- **Decision:** A click in the empty hours of the day, 3-day or week view starts the event at the 15-minute step clicked. Dragging over them selects a range in that day in 15-minute steps, at least one step, and the selection is drawn while it is made. The `c` key starts the event on the selected day at the next full hour of the clock, 23:00 at the latest. The quick-entry box is a small dialog titled "New event". It has the title field with the keyboard ("Add title"), the date and times as the event details write them, the calendar picker and **Save**; Enter in the title saves too. An empty title is saved as Google saves one, and shown as "(No title)". The picker offers only the calendars the account can change, under their accounts, those disabled left out (`DEC-102`). The default length is Google's `defaultEventLength` setting of the default calendar's account. It is read with the account's first synchronization of the run, and 60 minutes stands in until then or when it cannot be read. A saved event is shown at once with "· Saving…" after its title until Google answers, then without the mark until synchronization brings it. If Google refuses, the event leaves the views and a notice under the top bar says "The event could not be saved: <Google's reason>." Offline, or with no calendar that can take events, the box says why and **Save** is unavailable. The calendar-provider interface gets `create` and `default_length`.
- **Why:** 15-minute steps match the drag steps of `DEC-60`. Google Calendar's quick box takes a title, shows the time and has a calendar picker (`DEC-62`). A setting that rarely changes does not need a request every round. Confirmation and synchronization are separate steps, so the event never blinks out between Google's answer and the next synchronization.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-104] The default calendar: chosen in the settings, else the first account's primary, else the first calendar that can take events

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-01
- **Affects:** `US-05-01`, `MOD-05-F10`, `DEC-60`, `DEC-94`, `DEC-102`
- **Decision:** The settings get a **Default calendar** picker under **Calendars**, with the same choices as the quick-entry box. The choice is kept as a `default_calendar = <account ID> <calendar ID>` line and forgotten with its account. Until the user chooses, new events go to the primary calendar of the first connected account. That is the calendar whose ID is the account's e-mail address, as Google names it, so the cache needs no new column. A chosen or primary calendar that is disabled, gone or no longer changeable gives way to the next in that order. Failing both, the first calendar that can take events in the pickers' order is used.
- **Why:** `DEC-60` settles the default until the user chooses; the fallback only keeps creating possible when that calendar cannot take events, instead of refusing it.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-105] Saving to Google and the "saving" mark are delivered by every story that changes events

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-01
- **Affects:** `MOD-05-F16`, `MOD-05-U03`, `US-05-01`, `US-05-02`, `US-05-03`, `US-05-04`, MOD-05 `COVERAGE.md`
- **Decision:** `US-05-01` delivers `MOD-05-F16` and `MOD-05-U03` for new events. `US-05-02` (editing) and `US-05-03` (dragging and resizing) now also cover both. `US-05-04` (deleting and undo) now also covers `MOD-05-F16`. Their frontmatter, the BACKLOG rows and `COVERAGE.md` say so. The requirements themselves are unchanged.
- **Why:** The spec-compliance review found that only `US-05-01` covered both requirements, while changes and deletions come in later stories. Without this, nothing would deliver saving changes and deletions to Google. The traceability is fixed without weakening either `must`.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-106] The full editor: how its fields are typed and what it opens from

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-02
- **Affects:** `US-05-02`, `MOD-05-F03`…`MOD-05-F08`, `MOD-05-F11`, `MOD-05-F15`
- **Decision:** The quick-entry box gets **More options**, which opens the full editor with the title, times and calendar typed there. An event's details get **Edit** for events the account can change. Dates and times are typed in the system's numeric date format (or `YYYY-MM-DD`) and on its clock (`15:00`, or `3:00 PM` with the locale's AM/PM). Changing the start moves the end with it, so the event keeps its length. Changing the time zone keeps the times on the clock. An event may end when it starts, as in Google Calendar; one ending before it starts cannot be saved, and the editor says why. A time the zone skips at a daylight-saving change moves to the first time that exists. An all-day event made timed starts at 9:00 and ends at 10:00 on its first and last days. A timed event made all-day keeps its days. The description is edited as plain text. An existing event offers only its own account's calendars (`MOD-05-F15`). Saving sends only the fields that changed, as a patch against the version opened (its ETag), and then a move when another calendar was picked. Times left as they were opened keep the event's own, so a twice-shown hour of a daylight-saving change or seconds are not lost. A move Google refuses after the other changes were saved says "it could not be moved to the other calendar (…); its other changes were saved". An event changed elsewhere meanwhile is refused with the notice "The event could not be saved: it was changed elsewhere in the meantime; it is shown as it is now, make your change again." Its calendars are synchronized with that. **Edit** is unavailable for an event still being saved, and for an occurrence of a series until `US-05-07` brings the three-way question.
- **Why:** The spec names the fields but not how they are typed. These are the simplest choices that follow Google Calendar (`DEC-62`) and the regional formats the app already uses (`DEC-29`). Changing an occurrence without the "this / following / all" question of `MOD-05-F20` would guess the scope, so it waits for the story that asks it.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-107] The cache keeps an event's description, location, busy or free, visibility and version

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-02
- **Affects:** `US-05-02`, `MOD-05-F11`, `DEC-23`, the SQLite cache, the calendar-provider interface
- **Decision:** **SQLite schema change:** the `events` table gets `description`, `location`, `show`, `visibility` and `etag`. The schema is versioned with SQLite's `user_version`. A cache made by an earlier version drops its events and synchronization states when opened, and they are fetched again in full; its calendars stay. **Calendar-provider interface change:** `Provider::update` (a patch sent against the event's version, refused as "changed elsewhere" on Google's 412) and `Provider::move_to`. Google's `confidential` visibility is read as private.
- **Why:** The editor opens on what the views already hold, without waiting for Google (`NFR-01`), and a change is sent against the version it was made to (MOD-05 *Technical notes*). The cache is rebuilt from Google anyway (`NFR-17`), so dropping an old one's events is the simplest migration.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-108] Dragging and resizing: the distance in 15-minute steps, the end's edge, the strip and the hours

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-03
- **Affects:** `US-05-03`, `MOD-05-F13`, `MOD-05-F14`, `MOD-05-F16`, `MOD-05-U02`, `MOD-05-U03`, `DEC-60`, `DEC-103`, `DEC-106`
- **Decision:** In the day, 3-day and week views, an event moves by the distance dragged, rounded to the nearest 15 minutes and to whole days among the days shown; an event starting off the step (10:07) keeps its offset. Times move on the clock of the system's time zone the views show, so across a daylight-saving change the event keeps the hour it was shown at; it keeps its length and its own time zone. The end is grabbed at the bottom 5 points of the block where the event ends, and moves in 15-minute steps, leaving the event a step long at least and ending by midnight of that day. While a block is dragged it shows its new times as "start – end". An all-day event moves by whole days in the strip; dropped in the hours it becomes timed from the 15-minute step under the pointer, of the default length of its account (`DEC-103`), in the system's time zone, and while it is dragged there a one-step block with its start time shows where. A timed event dropped above the hours, over the strip or the day header, becomes an all-day event of the day under the pointer; while it is dragged there, it is drawn over the strip's last lane in that day, the layout staying still. An all-day event shown from before the first day can be dragged until its last day shown reaches the first. A timed event lasting a day or longer, shown in the strip, moves by whole days only. An event dropped where it was is not sent. A drag under way ends when synchronization changes the events shown, so it cannot pass to another event. Events of calendars the account cannot change, events being saved and occurrences of a series cannot be dragged (the explanation is `US-05-05`, the scope question `US-05-07`); offline nothing can (`MOD-02-F15`). A drop is saved like the editor's change (`DEC-106`): the times as a patch against the event's version, shown at once with "· Saving…", which all-day bars now show too, and refused with the same notice when the event was changed elsewhere.
- **Why:** The spec fixes the 15-minute steps and the conversions (`DEC-60`) but not how far a drop goes, where the end is grabbed or how long a converted event is. These follow Google Calendar (`DEC-62`), which moves an event by the distance dragged and converts between the strip and the hours the same way; reusing the editor's change keeps one path to Google.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-109] Deleting and undoing: Delete in the details and on the keyboard, restore at Google, Ctrl+Z until the next change

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-04
- **Affects:** `US-05-04`, `MOD-05-F12`, `MOD-05-F16`, `MOD-05-F27`, `MOD-05-F28`, `MOD-05-U04`, `DEC-59`, `DEC-60`, `DEC-106`, `DEC-108`, the calendar-provider interface
- **Decision:** An event's details get **Delete** beside **Edit**. It is available under the same conditions: an event the account can change, online, not still being saved, and not an occurrence of a series until `US-05-07`. With the details open, Delete or Backspace deletes the event too, as in Google Calendar. No deletion asks a question. The app does not keep an event's guests yet, so every single event counts as one without guests until `MOD-06` brings the question about notifying them (`MOD-06-F12`). A deletion is sent against the version shown (its ETag). It is refused with "The event could not be deleted: it was changed elsewhere in the meantime; …" when the event changed elsewhere. A deleted event disappears at once and comes back if Google refuses. After Google takes a deletion, a move, a resize or an edit, a notice "Event deleted." or "Event saved." with **Undo** shows for 5 seconds. Ctrl+Z undoes the same last change, also after the notice has gone, until the user creates, changes or deletes another event. An undone change cannot be undone again (there is no redo). Ctrl+Z does nothing while a text field has the keyboard, where it undoes typing, and does nothing offline. Undoing a deletion restores the event at Google by setting its status back to confirmed. Google keeps a deleted event, cancelled, with everything it had, so the event keeps its ID and every field, including those the app does not hold. Undoing a change patches the fields it changed back to their old values, against the version Google gave the change, and moves a moved event back to its calendar. A change that only moved the event is undone with its title patched as it is, so that it is sent against the version too. An event changed elsewhere meanwhile refuses the undo: "The change could not be undone: it was changed elsewhere in the meantime; …", and it is shown as it is now. A restore is sent without a version, because Google answers a deletion without one; restoring an event someone else restored meanwhile changes nothing. An undo Google refuses is not offered again, and an account removed takes its undo with it. **Calendar-provider interface change:** `Provider::delete` (with the ETag) and `Provider::restore`.
- **Why:** The spec fixes the Undo notice, its 5 seconds and Ctrl+Z (`DEC-59`, `DEC-60`), but not where Delete is, how long Ctrl+Z works or how a deletion is undone. Google Calendar (`DEC-62`) deletes with Delete or Backspace from an event's details and offers Undo after a deletion and a save. Keeping the undo until the next change is the simplest reading of "the last change". Restoring the deleted event, rather than re-creating it, loses nothing the app does not cache, such as reminders, guests or attachments. Sending both the deletion and the undo against the event's version gives the "changed elsewhere" refusal of the edge case.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-110] Protected events: read-only calendars and guests' copies, the reason on trying, Google's refusal as it says it

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-05
- **Affects:** `US-05-05`, `MOD-05-F17`, `MOD-05-F22`, `DEC-22`, `DEC-89`, `DEC-106`, `DEC-108`, `DEC-109`, the SQLite cache, the calendar-provider interface
- **Decision:** An event the user may not change is one of a calendar the account cannot change (not owner or writer, `DEC-22`), or a guest's copy of someone else's event whose organizer does not let guests change it: Google gives it an `organizer` that is not the calendar itself (`self`) and no `guestsCanModify`. Such an event cannot be dragged, resized, edited or deleted, a guest's copy included, although Google lets a guest take an invitation off their calendar: what a guest may still do comes with `MOD-06`. A drag started on it leaves it where it is and the notice bar says why, once however often it is tried: "🔒 This event can't be changed: its calendar is read-only." or "This event can't be changed: only its organizer can change it." Its details say the same, in place of the "🔒 Read-only: this event can't be changed" of `DEC-89`, and offer neither **Edit** nor **Delete**, and Delete or Backspace do nothing there. Only an event of a read-only calendar carries the lock in the views (`MOD-02-U01`); a guest's copy looks like the user's other events, as in Google Calendar. A change Google refuses anyway, because the calendar or the event changed before the app learned of it, is told as Google words it — "The event could not be saved: Google refused (You need to have writer access to this calendar.)" or "(Shared properties can only be changed by the organizer of the event.)" — and the event is shown as it was; the synchronization after it brings the new rights, so it is protected from then on. One changed elsewhere meanwhile is reloaded as `DEC-106` says. **SQLite schema change:** the `events` table gets `organizer_only`; the schema version goes to 2, so an earlier cache drops its events and fetches them again (`DEC-107`). **Calendar-provider interface change:** `SyncedEvent::organizer_only`.
- **Why:** The spec names guests without edit rights (`US-05-05`) but not how they are told apart; `organizer.self` and `guestsCanModify` are how Google tells a guest's copy from the organizer's event, and Google Calendar does not let a guest move or edit such a copy (`DEC-62`). The acceptance criterion forbids deleting as well, and responding or removing an invitation belongs to `MOD-06`. Google's own message names the reason, contains no event contents (`NFR-07`), and needs no table of Google's error codes.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-111] Repeating is chosen in the editor of a new event; an existing event's repetition is not changed in the app

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-06
- **Affects:** `US-05-06`, `US-05-07`, `MOD-05-F18`, `MOD-05-F19`, `DEC-36`, the calendar-provider interface
- **Decision:** The **Repeats** and **Ends** fields of the full editor are shown for a new event only: the event is created as a series, with Google's rule in `recurrence` and its times in their own zone. The editor of an existing event, a series' occurrence included, has no repeat fields; making an existing single event repeat, or changing a series' rule, is done in Google Calendar in the browser. A new series is shown marked "saving" until Google confirms it, and then until synchronization brings its occurrences, which Google names by the series' ID and their start. **Calendar-provider interface change:** `NewEvent::recurrence`.
- **Why:** The criteria and test cases of `US-05-06` create a series (`TC-05-07`, `TC-05-12`). Turning an existing event into a series, or changing a series' rule, is a change to a series, which comes with the scopes and undo of `US-05-07` (`MOD-05-F20`, `MOD-05-F28`); undoing it would have to turn a series back into a single event, which the spec does not describe. Complex rules are edited in the browser already (`DEC-36`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-112] Defaults and checks of the repeat fields, as Google Calendar's

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-06
- **Affects:** `US-05-06`, `MOD-05-F18`, `MOD-05-F19`, `DEC-36`, `DEC-62`
- **Decision:** A new event does not repeat until a preset is chosen: "Daily", "Every weekday (Monday to Friday)", "Weekly", "Monthly", "Annually". "Weekly" starts with the day of the week the event starts; further days are toggled by their letters, from the first day of the user's week, and the last chosen day cannot be taken away. "Monthly" repeats on the same day of the month and "Annually" on the same date (`FREQ=MONTHLY`, `FREQ=YEARLY`). The series ends "Never" until another end is chosen; "On" a date starts with the event's start date and moves with the start as the end does; "After" a number of occurrences starts with 13, Google Calendar's default. A date ending a timed series is written as Google writes it: the last second of that day in the event's zone, in UTC (`UNTIL=20261216T225959Z`); for an all-day series, the date. A series ending before the day it starts, an end date that cannot be read, or a number of occurrences that is not a whole number of at least 1 cannot be saved, and the editor says why: "The series ends before it starts.", "The end date of the series can't be read.", "The number of occurrences can't be read."
- **Why:** The spec names the presets, the three ends and their rules (`MOD-05` *Technical notes*), not the defaults or what an impossible end does; these follow Google Calendar (`DEC-62`) and the editor's existing "can't be read" and end-before-start checks (`MOD-05` *Constraints and edge cases*).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-113] The three scopes of a series: what each asks of Google, and which a change can take

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-07
- **Affects:** `US-05-07`, `MOD-05-F20`, `MOD-05-F21`, `MOD-05-F29`, `DEC-36`, `DEC-111`, the calendar-provider interface
- **Decision:** **This event** changes or deletes the occurrence alone, by the ID Google gives it (the series' ID and the occurrence's start); an occurrence not changed before has no version of its own, so it is sent without one. **All events** changes or deletes the series' master, against its version: new times keep the series' first day and take the occurrence's new times of day, length and zone; an all-day series takes the new length. **This and following** ends the series the second before the occurrence's start by its rule (`UNTIL` in UTC; the day before, as a date, for an all-day series), against its version, dropping a `COUNT`; for a change it then creates a new series from that occurrence as a copy of the series with everything Google keeps of it (guests, reminders, colour, conference, attachments; guests are not notified until `MOD-06-F12`) and the changes made, the series' rule kept but for its count, which loses the occurrences before (`COUNT=10` from the 5th is `COUNT=6`), and its deleted occurrences from there on kept deleted at the new times (`EXDATE`). `RDATE` lines stay with the old series. The occurrences from there on changed on their own, the one chosen included, are deleted, as they belong to the old series, without a version, as ending the series may have changed them at Google; the occurrences changed on their own of a series whose times change for all events are left as Google keeps them. On the first occurrence, this and following is all events (MOD-05 *Constraints and edge cases*). A series' rule is never rewritten otherwise, so one the presets cannot express keeps it when other fields change (`MOD-05-F29`). Moving the occurrence to another day, or between all-day and timed, can only be done to this event: for the following or all events it changes how the series repeats, which is done in Google Calendar (`DEC-111`), and the question says so. Moving it to another calendar can only be done to all events, as Google moves a series whole. Deleting can take any scope. **Calendar-provider interface change:** `Patch::recurrence` (a series' rule lines, replaced whole); `Provider::copy` (a copy of an event with some fields in place of its own); `Provider::update` and `Provider::delete` with an empty version send none. **Model change:** `Event::series` (the series' ID and where its rule starts the occurrence); an unchanged occurrence's `etag` is empty. No SQLite schema change.
- **Why:** The spec fixes the three scopes and the split (`UNTIL` on the old master, a new series from the occurrence; MOD-05 *Technical notes*) but not how far the old series runs, what the new one's count is, or how new times apply to all events. These follow how Google names and changes occurrences and how Google Calendar's "all events" keeps a series' start day (`DEC-62`). Moving a series to other days would rewrite its rule (`BYDAY`), which `DEC-111` leaves to Google Calendar.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-114] The scope question, the series' repetition in Google Calendar, and undo of a series change

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-07
- **Affects:** `US-05-07`, `MOD-05-F20`, `MOD-05-F21`, `MOD-05-F28`, `MOD-05-F29`, `MOD-05-U03`, `DEC-106`, `DEC-108`, `DEC-109`, `DEC-111`
- **Decision:** An occurrence of a series can now be dragged, resized, edited and deleted (lifting `DEC-106`, `DEC-108`, `DEC-109`'s exclusion). Saving a change from the editor, dropping a drag or resize, and Delete (the button, Delete or Backspace) on an occurrence open "Edit recurring event" or "Delete recurring event" with **This event**, **This and following events**, **All events** and **Cancel**, as Google Calendar asks; a scope the change cannot take is unavailable, with the reason below. Nothing is sent until a scope is chosen; a dragged occurrence stays where it was meanwhile, and Cancel, Esc or closing the question go back to the editor with the change as it was made. The editor of an occurrence shows "Repeats: Changed in Google Calendar" with **Open in Google Calendar**, which opens the series in Google Calendar's editor in the browser, as the event's account (`https://calendar.google.com/calendar/r/eventedit/<eid>?authuser=<e-mail>`, `eid` being the series' ID and its calendar's in base64url); for every series, as the app changes no series' repetition (`DEC-111`). A change to this event is shown and undone like any change (`DEC-109`). A change to or deletion of the following or all events is shown at once on the occurrence, marked "saving" until Google confirms it and until the next synchronization brings the series; it cannot be undone, and like creating an event it ends the undo of the change before. If the new series of a split cannot be created after the old one was ended, the user is told: "The event could not be saved: the series now ends before this event, but the following events could not be made a series of their own (…)."
- **Why:** The spec asks the three-way question for the editor, dragging and resizing (`MOD-05-F20`) and deleting (`MOD-05-F21`), and to offer Google Calendar for changing a rule (`DEC-60`); the dialog follows Google Calendar's (`DEC-62`). Undoing a split would have to merge two series back, which the spec does not describe; `MOD-05-F28` is delivered by `US-05-04` for single changes.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-115] How special event types look, and that working locations and birthdays are not changed in the app

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-08
- **Affects:** `US-05-08`, `MOD-05-F23`, `MOD-05-U01`, `MOD-05-F17`, `DEC-34`, `DEC-60`
- **Decision:** Every special type carries a mark before its title in every view (day, 3-day, week, month, agenda): out of office 🌴, focus time 🎧, working location 📍, birthday 🎂. In the hour grid, out-of-office events are also drawn hatched (diagonal stripes over their colour), in the hours or in the all-day strip. Working-location and birthday events always go to the all-day strip, also a timed working location (e.g. 9:00–17:00), drawn as a label: outlined in their colour, not filled. Google's other types (`default`, `fromGmail`) are ordinary events. Working-location and birthday events cannot be dragged, resized, edited or deleted in the app; trying says "This event can't be changed here: working locations and birthdays are changed in Google."
- **Why:** `MOD-05-U01` names hatching for out of office and an icon for focus time but not how the month and agenda views make the types stand out (`MOD-05-F23`); one mark per type, in all views, is the simplest consistent answer. Google Calendar shows working locations as labels at the top of the day whatever their hours. The technical notes make working location and birthday read-only (`DEC-34`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-116] Creating out of office and focus time: where, all day, busy, and Google Calendar's auto-decline defaults

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-08
- **Affects:** `US-05-08`, `MOD-05-F24`, `DEC-34`
- **Decision:** The editor of a new event shows "Type: Event | Out of office | Focus time" only when the calendar picked is the primary calendar of a Workspace account (its ID is the account's e-mail address, the account owns it, and the account signed in with Google's `hd` claim); with another calendar picked, the row is gone and the type goes back to an ordinary event. An existing event keeps its type, shown in the editor but not changeable (Google does not change an event's type). As Google takes these types only as timed events, "All day" makes them run from midnight of the first day to midnight after the last in the editor's time zone, as Google Calendar does; they are always busy, so "Show as" is not offered. They are sent with `eventType` and Google Calendar's defaults: out of office with `autoDeclineMode: declineOnlyNewConflictingInvitations` and the message "Declined because I am out of office"; focus time with the same mode, the message "Declined because I am in focus time" and chat set to do not disturb. Declining and chat settings are not editable in the app.
- **Why:** `MOD-05-F24` limits the types to Workspace primary calendars without saying how the app knows one or what the other required fields are; Google's API refuses all-day and transparent out-of-office and focus-time events and needs their properties. Declining only new invitations is the gentler of Google's options and leaves existing meetings alone; editing these settings is not asked for (`DEC-62`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-117] Out of office and focus time stay timed when dragged, and stay in their calendar

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-08
- **Affects:** `US-05-08`, `MOD-05-F13`, `MOD-05-F15`, `DEC-116`
- **Decision:** An out-of-office or focus-time event dragged into the all-day strip runs from midnight to midnight of that day in the views' time zone, instead of becoming an all-day event. The editor of an existing out-of-office, focus-time, working-location or birthday event offers only its own calendar.
- **Why:** Google keeps out of office and focus time only as timed events (`DEC-116`), and moves only ordinary events between calendars (`events.move`); offering either would end in Google's refusal.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-118] Event colours: Google Calendar's eleven, built in, shown over the calendar's colour

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-09
- **Affects:** `US-05-09`, `MOD-05-F25`, `MOD-05` *Integrations* (colours), `DEC-35`, `DEC-99`
- **Decision:** The event colours are Google's eleven `colorId`s (1–11), built into the app with the names and colours Google Calendar shows them in (Lavender, Sage, Grape, Flamingo, Banana, Tangerine, Peacock, Graphite, Blueberry, Basil, Tomato), instead of being read from the Calendar API's `colors` endpoint. An event's own colour is shown instead of its calendar's colour, a local calendar colour included. The editor offers "Calendar colour" (none of its own) and the eleven colours, for new and existing events; taking the colour away sends `colorId: null`. A `colorId` outside the eleven is shown in the calendar's colour. "This and following" gives the new series the series' colour, or the one picked.
- **Why:** The `colors` endpoint still answers with Google's older, paler palette, which Google Calendar in the browser no longer shows; the IDs are fixed by the API, so the built-in palette matches what the user sees in Google (`DEC-62`). Google Calendar shows an event's own colour over its calendar's, and offers the calendar's colour first in the event colour menu.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-119] Attachments: listed by Google's title, opened only when they are web pages

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-05-10
- **Affects:** `US-05-10`, `MOD-05-F26`, `DEC-37`
- **Decision:** An event's details list each of Google's `attachments` by its `title`, in Google's order, and choosing one opens its `fileUrl` in the default browser. An attachment without a title is listed by its link. Only `https://` links are listed: an attachment without one, or with a link of another kind, is left out. An occurrence of a series shows its series' attachments unless it was changed on its own, in which case it shows its own, as Google keeps them.
- **Why:** Google Calendar shows attachments by their file name and opens them in the browser (`DEC-62`). The link is set by whoever made the event, possibly someone else, and the desktop's URL handler would also start other programs for other kinds of links; Google Drive links are always `https://`.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-120] Guest suggestions: what matches, in what order, and what Enter adds

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-01
- **Affects:** `US-06-01`, `MOD-06-F01`, `MOD-06-F02`, `MOD-06-F03`, `MOD-06-F04`, `DEC-39`, `DEC-52`
- **Decision:** What is typed in the guest field suggests the people whose name, a word of their name or whose address starts with it, ignoring case: first those Google finds (People API `searchContacts`, then `otherContacts.search`, both warmed up once per run with an empty search, then `searchDirectoryPeople` with the domain profiles, only on an account that granted the directory at sign-in), then the guests of the account's cached events, those of the latest events first. Each address is suggested once, never one already invited or one of the user's own (the account's address and the calendar's ID), and at most 8 are shown. Enter adds the address typed when it is a valid one, otherwise the first person suggested; a click adds the person clicked. An address is valid with one `@`, something before it, a domain of at least two non-empty dot-separated parts after it, and no spaces; otherwise it is not added and the field says "This is not a valid e-mail address." A Google search that fails, e.g. offline, adds nobody; the recent guests are still suggested.
- **Why:** Google Calendar suggests people by the start of their name or address as one list, and its Enter picks the highlighted (first) suggestion (`DEC-62`); keeping an address typed in full avoids replacing it with a longer one that merely starts the same way. The cache holds no "invited by me" mark, so the guests of the account's events stand for the addresses it has invited.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-121] Guests are sent whole with their answers; invitations e-mailed for new events only, until the notify question

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-01
- **Affects:** `US-06-01`, `US-06-03`, `MOD-06-F06`, `MOD-06-F07`, `MOD-06-F08`, `MOD-06-F12`
- **Decision:** A new event with guests is created with `sendUpdates=all`, so Google e-mails the invitations. A change to an existing event's guests patches Google's `attendees` whole, each guest with their address, name, `optional` when marked optional, and the `responseStatus` they gave, and without `sendUpdates`, so Google e-mails nobody, until `US-06-03` asks the user (`MOD-06-F12`). The account's own entry (Google's `self`) is kept in the list sent but not shown among the guests in the editor. Out of office and focus time take no guests, as Google allows none there. A guest is required unless marked optional, as in Google.
- **Why:** The story leaves the notify question to `US-06-03`; sending nothing until then is the API's default, and e-mailing everyone on any change would be the surprise. Resending each guest's answer, and the account's own entry, keeps the answers and the organizer as Google had them when the list is replaced.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-122] A guest change keeps what Google keeps of each guest

- **Date:** 2026-10-23
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-01
- **Affects:** `US-06-01`, `MOD-06-F06`, `MOD-06-F07`, `MOD-05-F20`, `DEC-120`, `DEC-121`
- **Decision:** Refines `DEC-121`. Before the guests of an existing event are sent, the event is read from Google, and each guest Google already has keeps their entry as Google keeps it: their answer, comment, additional guests, room mark and organizer mark. Only whether they are optional, and their name where the app knows one, are set by the app. Guests new to the event are sent unanswered. The same applies when "this and following" copies the series' guests to the new series, and when a change to one occurrence is applied to all events. Refines `DEC-120`: the contact searches' warm-up is tried again at the next search until both warm-ups get through.
- **Why:** Google replaces the attendee list whole, and the app does not model every attendee field. Taking the answers from the occurrence opened would copy one occurrence's answers to the whole series. Without a warm-up, Google's contact searches may find nobody.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-123] How an event's details list its guests and their answers

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-02
- **Affects:** `US-06-02`, `MOD-06-F05`, `MOD-06-U01`, `DEC-63`
- **Decision:** Below the attachments, the details say how many guests there are ("N guests") and list everyone Google lists as an attendee, the user included, in Google's order. Each row is the answer's icon (✔ accepted, ✖ declined, ❓ maybe, ⏳ awaiting; hovering names it), then the guest's name when Google gives one, then their address. With more than 10 guests the first 10 are listed and a "Show N more" link expands the list ("Show fewer" collapses it again); each opening of an event starts collapsed. The open details follow the event as last synchronized, so an answer changes there after the next synchronization.
- **Why:** As in Google Calendar, which lists the organizer and every guest with a small answer mark and a count of guests; the icons are in the app's built-in fonts. Google's order is kept rather than re-sorted, the simplest choice consistent with `DEC-62`.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-124] When and how the app asks whether to notify guests

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-03
- **Affects:** `US-06-03`, `MOD-06-F08`, `MOD-06-F12`, `MOD-05-F20`, `MOD-05-F21`, `MOD-05-F28`, `DEC-40`, `DEC-121`
- **Decision:** The question is asked when the event has a guest other than the account itself, before or after the change, so inviting the first guest or removing the last one asks too; it comes after the scope question for an occurrence of a series. A change asks "Send update e-mails to the guests?", a deletion "Send cancellation e-mails to the guests?", with the buttons Back, Don't send and Send; Back (or Escape) saves nothing and returns to the editor the change was made in, and a dragged event stays where it was. The answer goes to Google as `sendUpdates=all` or `none` on every request the change makes: the patch, the move to another calendar, the series' end, its copy and the deletions of a split, the deletion and the restore. A change or deletion of an event without guests asks nothing and is sent with `sendUpdates=none`. An undo asks nothing and tells the guests as the change it undoes did. Refines `DEC-121`: changes to existing events are no longer sent without `sendUpdates`; a new event with guests is still created with `sendUpdates=all`.
- **Why:** As in Google Calendar, which asks after the scope with Back / Don't send / Send and words the question for an update or a cancellation (`DEC-62`). Applying one answer to every request keeps the guests told, or not, of the whole change; an undo that reverts a change the guests were told of has to reach them too.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-125] Which invitations the side panel counts and lists

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-04
- **Affects:** `US-06-04`, `MOD-06-F09`, `MOD-06-F10`, `MOD-06-F21`, `DEC-41`, `DEC-61`
- **Decision:** An invitation awaits an answer when the account's own entry among the guests (Google's `self` attendee) has `responseStatus: needsAction`. The side panel counts, and its list shows, those not over yet (an all-day one until its last day ends), a recurring invitation once, by its next occurrence, earliest first; those of calendars disabled in the settings are left out, while those of calendars only hidden by the quick toggle are still counted and listed (as `MOD-06-F21` names only disabled calendars), though the views do not show them. The counter, "N invitations awaiting an answer", sits under the mini calendar and is shown only while at least one invitation awaits; choosing it opens the list "Invitations", each row its title and when, which opens the invitation's details, where it is answered (`US-06-05`).
- **Why:** The simplest reading of `DEC-61` that keeps the counter useful: past invitations can no longer be attended, and counting every occurrence of a weekly series would drown the rest. Hiding a calendar for a moment should not hide that someone is waiting for an answer.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-126] How an invitation awaiting an answer is drawn

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-04
- **Affects:** `US-06-04`, `MOD-06-U04`, `DEC-63`
- **Decision:** In the day, 3-day, week and month views, an invitation awaiting an answer is filled with its colour at a fifth of its strength over the view's background, inside a 1-point dashed outline in its full colour, its text in the view's text colour. In the month view's rows of timed events and in the agenda, which draw no box, the row gets the same fill and dashed outline, its coloured dot kept.
- **Why:** `MOD-06-U04` asks for a dashed outline and a light fill without fixing their look; this mirrors how Google Calendar draws unanswered invitations (an outline in the calendar's colour over a pale fill) and reads in light and dark themes alike.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-127] Only the account's own unanswered invitations are counted

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-04
- **Affects:** `US-06-04`, `MOD-06-F09`, `MOD-06-F10`, `DEC-125`
- **Decision:** Google marks as `self` the guest whose calendar the event's copy is on, which in a colleague's calendar shared with the user is the colleague. The views draw an event dashed whenever that guest has not answered, also in a colleague's calendar; the side panel counts and lists an invitation only when that guest's address is the address of the account the calendar is listed under, so a colleague's unanswered invitations, and a second copy of the user's own in a shared calendar, are not counted. Refines `DEC-125`.
- **Why:** As in Google Calendar, which shows other people's calendars with their owners' answers (`DEC-62`); only the user's own invitations wait for the user's answer.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-128] An answer lets the organizer know and asks nothing first

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-05
- **Affects:** `US-06-05`, `MOD-06-F11`, `MOD-06-F12`, `MOD-06-F13`
- **Decision:** An answer (Accept, Maybe, Decline) is sent at once with `sendUpdates=all`, so Google lets the organizer know, without the question of `MOD-06-F12`, which stays for changes and deletions; the organizer's own answer to their event is sent with `sendUpdates=none`, e-mailing nobody. It patches the user's own attendee entry (`self`) alone, the other guests' entries sent back as Google keeps them, and a list Google left guests out of (`attendeesOmitted`) stays marked so. An occurrence of a series asks only "This event" or "All events" (`MOD-06-F13`), not "This and following events". An answer is not undone with Ctrl+Z and shows no "Event saved." notice; it is changed by answering again.
- **Why:** In Google Calendar answering an invitation e-mails the organizer and asks nothing (`DEC-62`); `MOD-06-F12` speaks of changing and deleting events, and `MOD-06-F13` names the two scopes.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-129] Which events can be answered, and where

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-05
- **Affects:** `US-06-05`, `MOD-06-F11`, `MOD-06-F15`, `MOD-06-U03`
- **Decision:** Accept, Maybe and Decline are offered for every event whose guests include the guest whose calendar it is on (Google's `self`), in a calendar the account can change — also when only the organizer may change the event itself, and also an event the user organizes. They sit in a "Going?" row at the top of the event's details, the current answer marked, and in each row of the invitations list. Offline, or while the event is being saved, they are disabled and the reason is shown.
- **Why:** As in Google Calendar, whose event details offer "Going?" whenever the user is a guest (`DEC-62`); the simplest rule that needs nothing from Google beyond the guest list the app already keeps.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-130] Which events are declined, how they look, and the switch

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-06
- **Affects:** `US-06-06`, `MOD-06-F14`, `MOD-06-F22`
- **Decision:** An event is declined when the guest whose calendar it is on (Google's `self`) answered `declined`, as for an invitation awaiting an answer (`DEC-127`); so a colleague's declined event in their shared calendar is drawn declined too. In the day/3-day/week, month and agenda views its text (title, and in the hour grid also its time) is struck through and the whole event is faded under a half-transparent veil of the view's background, on top of whatever else marks it. The switch is a "Show declined events" checkbox in the view bar, on by default; turned off, declined events leave every view. The choice is kept in the app's settings and survives a restart.
- **Why:** As in Google Calendar, which strikes through declined events and keeps "Show declined events" as a lasting view option (`DEC-62`); the simplest rule that reuses the guest list the app already keeps.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-131] How the "Add Google Meet" switch behaves

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-07
- **Affects:** `US-06-07`, `MOD-06-F16`, `MOD-06-F17`
- **Decision:** The editor shows an "Add Google Meet" checkbox under the guests, for a new and an existing event, except for out of office and focus time, which take neither guests nor a call. For a new event it starts off. For an existing one it shows whether the event has a Google Meet call. It turns on by itself whenever a guest is added to an event that has no other guest at that moment. Otherwise it stays as the user left it: removing the guests does not turn it off, and switched off it stays off for the guests added after the first. Saved on, a new event asks Google to make a call (`conferenceDataVersion=1`, `createRequest` of type `hangoutsMeet`). Switched on for an existing event, it adds a call. Switched off, it takes the call away. "This and following events" keeps the series' call for the new series unless the switch changed. Undo puts the switch back as it was.
- **Why:** As in Google Calendar, where Meet comes with the first guest and is added or removed in the editor (`DEC-62`). Turning on only for the first guest keeps the user's own choice once made.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-132] An event Google cannot make a Meet call for

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-07
- **Affects:** `US-06-07`, `MOD-06-F16`, MOD-06 *Constraints and edge cases*
- **Decision:** When Google refuses a new or changed event sent with a Meet call to make, as it does for an account that may not use Google Meet, the event is sent again without the call, so it is saved. After saving, the app synchronizes the event's calendar. If the event comes back without a Meet link, the app says "The event was saved without Google Meet: Google could not add a Meet call for this account." among its notices. This also covers Google accepting the request but failing to make the call. It is not said after "this and following events" makes a new series. It is also said, wrongly, in two rare cases. One is a call Google is still making when the event comes back (`pending`); that call gets its Join at the next synchronization. The other is a change saved with Meet whose synchronization right after it fails.
- **Why:** The edge case asks that the event is saved without the call and that the app says so. Checking the event as it comes back covers both ways Google can fail. The provider's methods stay as they are: only the event and patch they take carry the switch (`NewEvent.meet`, `Patch.meet`), and the event they give carries the link (`About.meet`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-133] Which calls have Join, and where it sits

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-07
- **Affects:** `US-06-07`, `MOD-06-F18`
- **Decision:** Only a Google Meet conference (`conferenceSolution` of type `hangoutsMeet`) gives an event a Join button. Join opens the conference's video entry point, and only when it is an `https://` page. Other conferences, e.g. from add-ons, get no Join. The event's details have a "Join with Google Meet" button under its time. In the agenda, an event's row has a "Join" button at its right end. Both open the link with the desktop's URL handler. Events of calendars shared as free/busy only show no Join. The link is kept in the local cache with the event.
- **Why:** The requirement names Google Meet. Google Calendar labels the button "Join with Google Meet" (`DEC-62`). Only web pages are handed to the URL handler, as for attachments (`MOD-05-F26`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-134] What the guests' availability preview shows, and when

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-08
- **Affects:** `US-06-08`, `MOD-06-F19`, `MOD-06-F20`, `MOD-06-U02`
- **Decision:** The editor shows an "Availability" row under the time zone, for a timed event with guests, online. The preview covers the day the event starts, midnight to midnight in the editor's time zone. It has one strip per guest the editor lists (the account itself is not one), for the first 20. Each strip fills in the guest's busy times and is followed by the guest's name, or address when there is none. A guest busy at any moment of the chosen time is marked "Conflict"; busy times that only touch it are not. A guest whose calendar does not share free/busy, or whom Google does not know, shows "No data". The chosen time is one band across all strips. Google is asked with `freeBusy.query` through the account of the calendar picked, again whenever the day, the guests or the calendar change and each time the editor opens. Until Google answers, and when the query fails, no preview is shown. An all-day event shows no preview, nor do out of office and focus time, which take no guests.
- **Why:** The requirements say "the chosen day" and "the chosen time", which an all-day event does not have. The editor's own time zone is the one its times are typed in. Hiding the preview when Google does not answer is the same as offline (`MOD-06-F20`).
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-135] An existing event's own time is not a conflict

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-06-08
- **Affects:** `US-06-08`, `MOD-06-F19`
- **Decision:** In the editor of an existing timed event shown as busy, the time it was saved at is taken out of the busy times of each guest it invites who has not declined. Google's free/busy counts the event itself for them, and each would otherwise show a conflict with it. The cost: another event of such a guest at exactly that saved time is not shown there either.
- **Why:** Free/busy cannot tell the event apart from others. Without this, the conflict mark would be wrong for every saved meeting whose time is not changed.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-136] When a reminder is shown, and when it is not

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-07-01
- **Affects:** `US-07-01`, `MOD-07-F01`, `MOD-07-F07`, `MOD-07-F20`, MOD-07 *Technical notes*
- **Decision:** Reminders are scheduled by the synchronization worker from the events the cache holds, so they keep coming while the window is minimized. "While the app was not running" means before this run of the app started: a reminder whose time is earlier is never shown. A reminder whose time has come is shown at once while its event has not started, or within a minute of its time. This covers reminders that fell while the computer slept (the worker wakes with the system's wake signal, and at least once a minute to follow the clock), and also those whose event reached the app by synchronization only after their time. A reminder is told apart by its account, calendar, event (an occurrence's own ID), its event's start and its minutes. The record of reminders shown is kept in memory for the run. That suffices: a reminder from before the run is not shown anyway. A moved event's reminder is a new one; a deleted event's goes. Calendars hidden by the side panel's quick toggle still remind, as only disabling (`MOD-07-F13`) and muting (`MOD-07-F12`) silence a calendar. Events of free/busy-only calendars remind of nothing (Google gives no reminders with them). The same calendar on two accounts' lists reminds once per account, as each account's own reminders are separate in Google.
- **Why:** `MOD-07-F20` separates "not running" from "asleep" but does not say what "not running" is measured against. The start of the run is the simplest clear line. Showing a late-synchronized reminder while its event has not started follows the rule for sleep. The minute's grace lets a reminder set at the event's start be shown at all.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-137] All-day events' reminders

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-07-01
- **Affects:** `US-07-01`, `MOD-07-F02`
- **Decision:** An all-day event's own reminders count back from midnight of its first day in the system's time zone, as Google Calendar does ("the day before at 17:00" is 420 minutes). An all-day event that uses its calendar's defaults gives no notification. Google keeps separate defaults for all-day events but does not give them through the Calendar API (`defaultReminders` are those of timed events). Using the timed defaults instead would remind e.g. 10 minutes before midnight.
- **Why:** The app can only follow reminders Google gives it (`MOD-07-F02`). No reminder is better than a wrong one.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted

## [DEC-138] What a notification says, and what clicking it does

- **Date:** 2026-10-24
- **Decided by:** delivery loop (Claude)
- **Source:** implementation US-07-01
- **Affects:** `US-07-01`, `MOD-07-F08`, `MOD-07-U01`
- **Decision:** The notification's title is the event's title ("(No title)" when it has none). Its text gives the start time in the system's formats ("10:00"). An event on another day than today gets its day before the time ("Friday 13 November, 10:00"), and an all-day event the day alone. The location, if any, is on a second line. It comes from "Calendar Buddy" with the `x-office-calendar` icon, and stays as long as the desktop keeps notifications. Clicking it brings the window forward, un-minimized, and opens the event's details. The view moves to the event's day (the agenda stays as it is). While the quick entry, the editor or one of their questions is open, the click only brings the window forward, so nothing typed is lost. The notifications go through `org.freedesktop.Notifications` (the body's "default" action), behind the `Notifier` platform interface. **SQLite schema change:** `calendars.reminders` (the calendar's default reminders) and `events.reminders` (an event's own; none when it uses the defaults). Cache version 8 drops the calendars too, so the list is fetched again.
- **Why:** `MOD-07-U01` names the contents but not their form. Google Calendar's notifications show the title over the time and place. Replacing an open editor would lose the user's work.
- **Scope effect:** within scope
- **Resolves:** —
- **Status:** accepted
