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

## US-02-03 — review — 2026-10-23 17:02
- What: ongoing sync after the first one: per-calendar adaptive polling (`calendar_core::schedule`: editable 1 min, read-only 15 min; minimized 5 / 30 min), at once on **Refresh** (new top-bar button), window activation (calendars not synced in the last 10 s), opening an event (its calendar), and wake/network return (logind `PrepareForSleep(false)`, NetworkManager `StateChanged(70)` behind the new `Wakeups` platform interface, `crates/app/src/platform.rs`). Incremental sync with `syncToken` (`calendar_core::sync::sync_calendar`), full resync on 410 Gone or once the window moves; one signed-in `calendar_google::Session` per account per run, access token renewed after 45 min or on 401. The worker (`crates/app/src/sync.rs`) became an `Engine` driven by jobs and the schedule's next due time. Call-outs per the constitution: new SQLite table `sync_tokens` (token + window per calendar), and calendar-provider interface change (`Provider::events` returns `Fetched`, new `Provider::changes`). Fake Google available to the app's tests via the `fake` feature of `calendar-google`.
- Decisions: DEC-90, DEC-91, DEC-92
- Reviews: code 2 round(s), spec 1 round(s); open notes: fixed blocking: sync tokens outlived a moved window (events entering it never fetched) and the worker's window never moved (DEC-92); fixed non-blocking: first request after sleep failed on an expired access token, Refresh/wake did not retry an account whose first sync failed, an account turned away after a retried first sync kept being polled. Not fixed: any failed round counts as "changed" (views rebuilt every minute while offline); Refresh and wake-ups have no minimum gap (a burst of clicks spends a full round each); minimize may not be reported on Wayland, so polling stays at the shown rate there; the D-Bus watcher has no automated test; a moved window shows its events at once but fetches the new days at each calendar's next turn; no back-off on 403/429 and no sync state shown (`US-02-06`); the calendar list is refreshed only at each account's first sync of the run (`US-02-04`); the module's technical note on one HTTP batch per round was changed to one request per calendar (DEC-91).
- Manual checks: TC-02-05, US-02-03#2, US-02-03#3, US-02-03#4, US-02-03#5, US-02-03#6, US-02-03#7, US-02-03#8
- Commits on main: one squash commit — feat(sync): keep calendars current with adaptive polling

## US-02-04 — review — 2026-10-23 17:14
- What: every sync round (and each account's first sync of a run) now refreshes the account's calendar list first (`calendar_core::sync::sync_calendar_list`): new calendars are synced at once and polled (`Schedule::update_account`), changed access takes effect, and calendars no longer listed go with their events and are told of once in a top banner ("The calendar “…” is no longer available.", ✕ closes it), including those deleted while the app was closed or offline. Calendars disabled in the settings (`disabled = <account> <calendar>` lines in `~/.config/calendar-buddy/settings`, forgotten with their account) stay listed, are never fetched, polled or cached, and are synced in full at once when enabled (`Synchronization::set_disabled`; the switch itself is US-03-07). No SQLite schema or provider-interface change.
- Decisions: DEC-93, DEC-94
- Reviews: code 1 round(s), spec 1 round(s); open notes: fixed non-blocking: the worker kept a removed account's disabled set, an unreadable cached list stopped every sync (now rewritten); added a test for a calendar list Google does not send. Not fixed (nits): Refresh fetches a recovering account's list twice; a calendar unshared from two connected accounts is named twice in the banner. An account whose every calendar is disabled refreshes its list only at the next start (DEC-93).
- Manual checks: TC-02-06, US-02-04#1, US-02-04#3, US-02-04#4, edge case (deleted while offline)
- Commits on main: 9c5639c feat(sync): keep the calendar list current and honour disabled ones

## US-02-05 — review — 2026-10-23 17:27
- What: the app knows it is offline from NetworkManager (new `Network` platform interface, `SystemNetwork`, `Online` flag); offline, a band under the top bar says so and that creating, editing, deleting and answering invitations are unavailable, the new-event dialog and an editable event's details give the reason (`calendar_core::offline`), and events cannot be dragged. Days on screen outside the cache window (`Navigation::on_screen`, `Window::outside`) are fetched from Google while browsed online (`fetch_events`, padded by a month so moved occurrences are not ghosted; only the latest browsing is fetched), shown with the cached events and never cached; offline they are empty with a note. Cached events at start were already shown before sync (US-02-01). No SQLite schema change; the provider interface is unchanged.
- Decisions: DEC-95
- Reviews: code 1 round(s), spec 1 round(s); open notes: no blocking findings. Fixed non-blocking: browsing jobs queued while the worker was busy are coalesced; edge-crossing events are de-duplicated by calendar and event ID (a newer cached version no longer shows twice); an occurrence moved out of the browsed days by up to a month no longer shows at its old time; a drag ending offline is not applied; TC-02-07 now checks `offline::indicator`. Not fixed: "Refresh" while browsing outside the window builds the event list twice; removing an account refetches the browsed days instead of filtering; NetworkManager is watched by two subscribers (`SystemWakeups`, `SystemNetwork`); the band and dialogs have no automated UI test; delete and RSVP actions do not exist yet (MOD-05/06 must use `offline::refused`); an occurrence moved more than a month away from the browsed days still shows at its old time.
- Manual checks: TC-02-07, US-02-05#3, US-02-05#4
- Commits on main: 1595118 feat(sync): browse calendars offline and fetch days outside the cache

## US-02-06 — review — 2026-10-23 17:45
- What: a synchronization indicator in the top bar (Synchronizing… / Up to date / Offline / Synchronization delayed / Synchronization error, plus "Last synchronized at HH:MM", kept in the cache across restarts — `calendar_core::status`), a band naming each calendar that is not up to date (failed, delayed by a rate limit, or not synchronizing because its account needs sign-in), and per-calendar recovery: `sync_calendars` now returns a result per calendar, and a failing calendar waits its interval and then doubles, up to 30 min, plus up to 25% jitter (`Schedule::failed/succeeded`, `back_off`), while the other calendars keep their turns. The Google adapter tells rate limits (429; 403 with `rateLimitExceeded`/`userRateLimitExceeded`) apart. The fake Google can rate-limit a calendar, disconnect and recover. The damaged-cache edge case was already in place (`a_damaged_cache_file_is_started_anew`). **SQLite schema change:** a new `last_synced` table. The provider interface is unchanged, but `sync_calendars` now returns a per-calendar result.
- Decisions: DEC-96, DEC-97, DEC-98
- Reviews: code 2 round(s), spec 1 round(s); open notes: both blocking findings fixed. Round 1: "Up to date" was shown before any sync of the run, which led to DEC-98. Round 2: accounts saved as needing sign-in were never marked at start. Also fixed: "Synchronizing…" no longer flashes for jobs that don't reach Google; a simplification in `schedule.rs`. Not fixed: an account whose first sync fails is still retried by the account check, Refresh and wake-ups, not with back-off (DEC-96); jitter can lift a capped wait to 37.5 min; the last-sync time survives removing every account until the next sync; no test sends a plain 403 that is not a rate limit; the band can get tall when many calendars fail; the "accounts cannot be checked → error" override in `main.rs` has no automated test; COVERAGE.md rows US-02-06#2 and #4 still say "not tested yet" (no test case exists for them); DEC-96–98 were decided by the loop, and the product owner may want to confirm the band as the meaning of "shown on that calendar" until US-03-06.
- Manual checks: TC-02-08, US-02-06#2, US-02-06#3, US-02-06#4, edge case "account needs sign-in"
- Commits on main: 47d1a57 feat(sync): show sync status and recover from errors on its own
