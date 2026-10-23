# Delivery log

## US-03-02 — review — 2026-10-23 14:00
- What: day and 3-day views on the shared hour grid (`GridView`, `GridViewModel`, `grid_view.rs`), with a plain Day / 3 days / Week picker; ‹ › step by the shown days.
- Decisions: DEC-69
- Reviews: code 1 round(s), spec 1 round(s); open notes: `Week::next`/`previous` now unused outside tests (kept for US-03-05); dragging an event past the grid edge moves it to an unshown day (feasibility-only drag, MOD-05).
- Manual checks: US-03-02#2, #3 (drawing of the 1- and 3-column grid; no TC)
- Commits on main: one squash commit — feat(views): add day and 3-day views

## US-03-03 — review — 2026-10-23 13:26
- What: month view (`MonthViewModel`, `month_view.rs`): whole weeks of the month, each day's events, "+N more" opening a window with the day's full list; "Month" in the plain picker, ‹ › by one month.
- Decisions: DEC-70
- Reviews: code 1 round(s), spec 1 round(s); open notes: stepping by month clamps the day (31 Jan → 28 Feb → 28 Mar), left for US-03-05; the "+N more" click is proven only at view-model level; `color`/`text_on` now shared from `grid_view`.
- Manual checks: TC-03-06 (drawing and click), US-03-03#3 (today highlight, "Busy")
- Commits on main: one squash commit — e90ea06 feat(views): add month view

## US-03-04 — review — 2026-10-23 13:31
- What: agenda view (`AgendaViewModel`, `agenda_view.rs`): events not yet ended, grouped by day from today without empty days, 30 more days loading near the end of the list, "No upcoming events" when nothing is ahead; "Agenda" in the plain picker, ‹ › disabled.
- Decisions: DEC-71
- Reviews: code 1 round(s), spec 1 round(s); open notes: fixed non-blocking "All day" label on timed events of 24 h+; with the next event years away the list loads 30 days per frame until it appears (terminates, not reachable with the fixture); scroll trigger wiring in `main.rs` proven only manually.
- Manual checks: TC-03-07, US-03-04#2, US-03-04#4
- Commits on main: one squash commit — 95ce4e7 feat(views): add agenda view

## Manual checks — 2026-10-23
- The owner went through every open manual check (US-03-02, US-03-03, US-03-04, busy blocks and today in all views): all passed. US-03-01…US-03-04 set to done.

## US-03-05 — review — 2026-10-23 13:51
- What: navigation (`calendar_core::navigation`): view switching keeps the selected date, Today and ‹ › by the view's period, a mini calendar in a left panel marking the period and jumping to a picked date; the last view saved to `~/.config/calendar-buddy/settings` (`calendar_core::settings`) by a writer thread, the app starting in it on today.
- Decisions: DEC-72
- Reviews: code 1 round(s), spec 2 round(s); open notes: fixed Today not bringing back a browsed mini calendar and the settings write on the UI thread (constitution); not fixed (non-blocking): month steps clamp the day (31 Jan → 28 Feb → 28 Mar, per DEC-72), a settings save in flight at exit can be lost (DEC-15).
- Manual checks: TC-03-08 (buttons and restart in the real app), US-03-05#3 (mini calendar drawing and clicks)
- Commits on main: one squash commit — 0d51b6d feat(views): add navigation, mini calendar and last view

## US-03-08 — review — 2026-10-23 14:49
- What: Google Calendar's keys (`calendar_core::shortcuts`: d x w m a, j/n k/p, t, g, c, ?) with the "?" list, "Go to date" and a stand-in "New event" dialog; Tab/Enter reach and open events in every view into a minimal details dialog (`calendar_core::details`).
- Decisions: DEC-73, DEC-74
- Reviews: code 1 round(s), spec 1 round(s); open notes: resumed session (earlier fixes committed as found); fixed non-blocking focus return after details, "+N more" list closing on Escape, Alt-modified keys, mini calendar reset only on `t`, all-day range year; not fixed: focus drops after dragging an event to another day; app-side key wiring has no automated test.
- Manual checks: TC-03-11, US-03-08#3, US-03-08#5
- Commits on main: one squash commit — feat(views): add keyboard shortcuts and keyboard navigation

## US-03-09 — done — 2026-10-23 15:01
- What: the week start, date and time formats come from the system locale (`calendar_core::regional`, glibc `LC_TIME` data) in every view, the mini calendar and the dialogs; the views follow a change of system time zone (checked every 5 s off the UI thread); event details add the time in the event's own zone.
- Decisions: DEC-75, DEC-76
- Reviews: code 2 round(s), spec 1 round(s); open notes: fixed a crash on `g` in locales with chrono-unsupported date formats, unreadable pre-filled dates and year-first locales written month first; not fixed: zone aliases (e.g. `Poland`) count as another zone; the app-side zone watcher has no automated test; AC1's "day and 3-day views start on Sunday" read as the week start (they start on the selected date).
- Manual checks: none
- Commits on main: one squash commit — feat(views): follow the system locale and time zone

## Manual checks — 2026-10-23 (2)
- The owner checked US-03-05 and US-03-08: all passed. Both set to done.

## US-01-01 — review — 2026-10-23 15:37
- What: welcome screen with one action, "Connect Google account" (`crates/app/src/welcome.rs`); new `calendar-google` crate with the OAuth installed-app sign-in (PKCE S256, loopback redirect on 127.0.0.1, 5-min wait with Cancel), checks before the browser (client file per DEC-77, secret store, network), the "unverified app" heads-up, the refresh token saved to the Secret Service behind `calendar_core::platform`; first-run failures explained with "Try again". Tests run against a fake Google, fake browser and fake store.
- Decisions: DEC-78, DEC-79, DEC-80
- Reviews: code 1 round(s), spec 1 round(s); open notes: fixed non-blocking: a different account picked on the Workspace directory screen was mixed in, the offline check ignored proxies (now HTTPS through ureq), Cancel during the token exchange still connected, check-order test. Not fixed: closing the directory-only tab leaves a Workspace user waiting up to 5 min (Cancel drops the whole connection); the loopback read has a per-read, not total, deadline; the offline test re-binds a freed port (rare flake). The account isn't remembered across restarts (US-01-03, DEC-80). Spec review flags edge case "admin block / user cap" as at risk if Google never redirects back.
- Manual checks: TC-01-01, US-01-01#4, US-01-01#5, US-01-01#6, edge cases (cancel, client file, admin block)
- Commits on main: one squash commit — ad9bcc4 feat(accounts): connect a first Google account from the welcome screen

## US-01-02 — review — 2026-10-23 15:42
- What: `Accounts` list in core (keyed by Google account ID, refreshed in place on a repeat sign-in, states "Connected" / "Needs sign-in", no cap); a Settings window from the top bar listing accounts with disabled "Sign in again" / "Remove" and "Add account", which reuses the welcome screen's connect flow (`welcome.rs` → `connect.rs`); Google's account chooser on the first consent screen so a second account can be picked. Tests against the fake Google (now with a second, personal account).
- Decisions: DEC-81, DEC-82
- Reviews: code 1 round(s), spec 1 round(s); open notes: fixed non-blocking: a stale error or heads-up stayed in Settings after closing it, a wrong test comment, a test name that promised the real store's de-duplication. Not fixed: `Accounts::add`'s return value is used only by tests; the real Secret Service replacing a repeat sign-in's token is checked only by hand. No code sets "Needs sign-in" yet (US-01-03).
- Manual checks: TC-01-02, US-01-02#3, edge case (repeat sign-in, one KWallet item)
- Commits on main: one squash commit — 1ec28bb feat(accounts): connect more accounts and list them in the settings

## US-01-03 — review — 2026-10-23 15:56
- What: connected accounts are remembered in `~/.config/calendar-buddy/accounts` (no secrets) and checked off the UI thread by refreshing each one's token from the secret store (at start, every 10 min, 1 min after a failure); `invalid_grant` or a missing token marks only that account "Needs sign-in", told above the calendar and on its Settings row; "Sign in again" re-runs the connect flow with `login_hint` and restores the account in place. A locked/missing store or client file leaves accounts as they were, with a notice and "Try again".
- Decisions: DEC-83, DEC-84
- Reviews: code 1 round(s), spec 1 round(s); open notes: fixed non-blocking: checks paused by a store problem now resume after a sign-in, the accounts file is written atomically, the check/sign-in race is tested, nits. Not fixed: a periodic check may prompt to unlock a keyring that re-locked itself (KeePassXC, GNOME Keyring on screen lock); Google errors other than `invalid_grant` (e.g. a replaced OAuth client) are retried silently; any store failure is worded as "locked"; access tokens are not kept until MOD-02.
- Manual checks: TC-01-03, US-01-03#2, US-01-03#3, US-01-03#4, edge cases (locked store, no network)
- Commits on main: one squash commit — feat(accounts): stay signed in and sign in again

## US-01-04 — review — 2026-10-23 16:06
- What: **Remove** in Settings asks inside the settings window (what is deleted locally, Google account untouched), then off the UI thread deletes the account's Secret Service items (`SecretStore::take`, which still deletes an unreadable item) and forgets it in the accounts file; the refresh token is then revoked at Google's revocation endpoint, best effort and never waited for (`calendar_google::revoke`, `crates/app/src/remove.rs`). A store that won't delete keeps the account with the reason and **Try again**; removing the last account closes Settings and shows the welcome screen. Removal and connecting/signing in again never run at once.
- Decisions: DEC-85
- Reviews: code 2 round(s), spec 1 round(s); open notes: fixed non-blocking: an unreadable credential blocked removal, removal and sign-in could run at once, a failure was hidden when Settings closed, DEC-85 wording on order. Not fixed: the real store's "unreadable item" path has no test (fakes cannot model it); if a later of several items fails to delete, the token already read is not revoked; a "not removed" message can linger after signing in again to that account; the app wiring for the last account → welcome screen is checked only by hand. No calendar/event cache or per-account settings exist yet, so F08 covers the accounts file and the credentials; later stories must delete their per-account data in the same step (DEC-85).
- Manual checks: TC-01-04, US-01-04#1, US-01-04#3, US-01-04#4, US-01-04#5, US-01-04#6, edge case (locked store)
- Commits on main: one squash commit — feat(accounts): remove an account with its local data

## US-02-01 — review — 2026-10-23 16:33
- What: first full synchronization of every connected account: Google's calendar list (colours, access) and each calendar's events from one year back to two years ahead (`calendar_google::sync_account`, behind `calendar_core::sync::Provider`) into a new SQLite cache (`calendar_core::cache`, `~/.cache/calendar-buddy/cache.sqlite3`); series kept as master plus exceptions and expanded with `rrule` (`calendar_core::recurrence`); one worker thread in the app (`crates/app/src/sync.rs`) shows the cache at start, syncs each account once its sign-in check works, and deletes a removed account's data. The generated 20,000-event fixture is gone from the app. New SQLite schema and calendar-provider interface (constitution review rule).
- Decisions: DEC-86, DEC-87, DEC-88
- Reviews: code 2 round(s), spec 1 round(s); open notes: fixed blocking: a timed series' `UNTIL` as a date or local time was rejected by `rrule` (series vanished), an all-day series with a UTC `UNTIL` lost its last day west of UTC; fixed non-blocking: the cache file was deleted on any open error, unstable calendar order, weak tests, `Sync` name shadowing. Not fixed: removing an account waits for a full sync already running before its events leave the screen and the cache; a stale failure can cause one extra full sync; one calendar Google always refuses makes its account re-sync at every 10-minute check until `US-02-03`; sync errors are not shown (`US-02-04`/`F17`); a moved occurrence moved out of the window shows at its original time; a series starting at an ambiguous local time shows only its first occurrence (DEC-88); the window moves only at the next start. Spec review notes partly-delivered neighbours: free/busy events busy-only (`US-02-02`), cached calendars shown at start (`F13`, `US-02-05`), damaged cache rebuilt; `rebuild()` of 20k events still runs on the UI thread after each update.
- Manual checks: TC-02-01, TC-02-02, US-02-01#3, US-02-01#5, edge cases (window edge, old series), account removal
- Commits on main: one squash commit — feat(sync): show all accounts' calendars from a first full sync

## US-02-02 — review — 2026-10-23 16:40
- What: events of calendars the account cannot change (Google `reader`/`freeBusyReader`) carry `read_only` from the calendar's access at every sync (`Access::can_edit`, `calendar_core::recurrence::expand`) into the grid, month, agenda and details view models; the app draws a 🔒 before their title (or "Busy") and the details say "Read-only: this event can't be changed". Free/busy events were already busy-only (US-02-01). The calendars' half of `MOD-02-U01` moves to the side panel: US-03-06 gains criterion #5, COVERAGE and BACKLOG updated. No SQLite schema or provider-interface change.
- Decisions: DEC-89
- Reviews: code 1 round(s), spec 2 round(s); open notes: fixed: spec review round 1 found the calendars' half of U01 owned by no story (now US-03-06#5); code review nit: the read-only text moved from core to the app next to `event_label`. Not fixed: read-only events can still be dragged in memory by the US-03-01 feasibility drag (MOD-05-F17 / US-05-05); US-03-06#5 has no test case yet.
- Manual checks: TC-02-03, US-02-02#1, edge case (access dropped to read-only)
- Commits on main: one squash commit — feat(sync): mark events of read-only calendars
