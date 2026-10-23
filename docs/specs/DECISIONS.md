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
- **Status:** accepted

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
