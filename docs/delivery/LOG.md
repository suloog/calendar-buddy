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

## US-02-07 — review — 2026-10-23 17:54
- What: a calendar's local colour, kept only in the app's settings (`color = <account ID> <calendar ID> #rrggbb` lines, forgotten with the account) and never sent to Google (`calendar_core::colors`, `Settings::colors`); the views are built with events recoloured by account and calendar (`Event` gained `account_id`). The side panel lists the calendars under the mini calendar (disabled ones left out), each with a **…** menu of Google's 24 calendar colours and **Reset** (`crates/app/src/calendar_list.rs`); the worker now sends the cached calendar list with the events. No SQLite schema change; the provider interface is unchanged.
- Decisions: DEC-99
- Reviews: code 1 round(s), spec 1 round(s); open notes: no blocking findings. Fixed non-blocking: "⋮" is not in egui's default fonts (now "…"), menus were keyed by row position (now by calendar), TC-02-09 now goes through the app's `pick` and `build` and its empty request-count check was dropped, removing an account with only colours no longer resends the disabled set. Updated the module's technical note (keyed by account and calendar ID). Not fixed (nits): colours of calendars gone from a list or disabled stay in the settings until their account is removed; a few key clones per frame in the side panel. The calendar list is flat until US-03-06 groups it by account (DEC-99, decided by the loop).
- Manual checks: TC-02-09, US-02-07#1, US-02-07#2, US-02-07#3
- Commits on main: be44abb feat(sync): give a calendar a local colour from the side panel

## US-03-06 — review — 2026-10-23 18:08
- What: the side panel lists each connected account by e-mail with its calendars under it (`calendar_core::side_panel`, disabled ones left out). Each calendar has Google's coloured checkbox as a quick toggle that hides its events and keeps it listed. Hidden calendars are kept as `hidden = <account> <calendar>` lines, forgotten with their account. **☰** at the far left of the top bar collapses or expands the panel, remembered as `side_panel = collapsed`; below 800 points the panel collapses on its own without saving that. Read-only, subscribed and free/busy calendars carry 🔒. A calendar's sync problem is shown under its row, which replaces the band under the top bar (`Status::problem`, `status::on_row`). With every calendar hidden or disabled, the view points to the side panel. egui's `monochrome_emoji_fonts` is now on: since egui 0.37 the default fonts lacked 🔒 and ✔, so the event locks of US-02-02 and the ticks of US-02-07 did not render. No SQLite schema or provider-interface change.
- Decisions: DEC-100, DEC-101
- Reviews: code 1 round(s), spec 1 round(s); open notes: no blocking findings. Fixed: a test now proves opening or dragging reaches the right event when a calendar is hidden (the `places` mapping), and `build` copies events only once. Not fixed (nits): with the panel collapsed, a calendar's sync problem is named nowhere, and the top bar only says "Synchronization error". `hidden` lines of calendars deleted in Google stay until their account is removed. Closing the panel by hand in a narrow window saves "collapsed". COVERAGE.md rows US-03-06#3–#5 still say "not tested yet", because there is no test case for them; unit tests cover them.
- Manual checks: TC-03-09, US-03-06#3, US-03-06#4, US-03-06#5, edge case (no calendar shown), DEC-97 row problem
- Commits on main: 07d5f70 feat(views): side panel with calendars grouped by account

## US-03-07 — review — 2026-10-23 18:14
- What: the settings get a **Calendars** section under **Accounts** that lists each account's calendars, each with a checkbox ticked while it is enabled. Unticking saves a `disabled` line, hands the set to the sync worker (`DEC-94`) and drops the calendar's events from every view in the same frame (`side_panel::shown` now filters disabled as well as hidden calendars). `side_panel::pickable` is the future calendar pickers' source (no picker exists yet, `MOD-05`). `groups` now shares `by_account` with the new lists. No SQLite schema or provider-interface change.
- Decisions: DEC-102
- Reviews: code 1 round(s), spec 1 round(s); open notes: no blocking findings. Fixed: `pickable` no longer claims to be the whole picker; edit rights are left to MOD-05 (DEC-102 reworded before merge). `groups` reuses `by_account`, and the click result is named `toggled`. Not fixed (nit): `toggle`'s parameter is still named `hidden` and typed `Hidden`, though it also serves `disabled`. The UI click wiring has no automated test, like the side panel's toggle. Check that MOD-05's pickers really use `pickable`.
- Manual checks: TC-03-10 (drawing only), US-03-07#2 pickers once MOD-05 exists
- Commits on main: 51d6ac4 feat(views): disable and enable calendars in the settings

## US-05-01 — review — 2026-10-23 18:46
- What: a click, a drag over empty hours (15-minute steps, primary button) or `c` opens a "New event" box with the title focused, the times and the default calendar. Enter or **Save** sends the event to Google on the sync worker, and it comes back through synchronizing its calendar. Until Google answers it shows with "· Saving…" (`calendar_core::create`). The default length is Google's `defaultEventLength`, read at each account's first sync of the run. The default calendar is picked in **Settings → Calendars** (`default_calendar` line); until then it is the first account's primary. **Calendar-provider interface changed:** `Provider::create` and `Provider::default_length`. Event gets a `saving` view flag. No SQLite schema change.
- Decisions: DEC-103, DEC-104, DEC-105
- Reviews: code 1 round(s), spec 2 round(s); open notes: no blocking findings. Fixed after code review: a selection kept across a view switch could index past the days (now `get` and dropped when the drag ends elsewhere); only the primary button selects; an event created on browsed days outside the cache comes back; a range inside a skipped DST hour lasts one step; clearer reason when the account is not signed in yet. Spec round 1: F16/U03 were mapped only to this story, so DEC-105 adds them to US-05-02/03/04. Not fixed: a POST timeout after Google stored the event says "not saved", and a retry could duplicate it (client-chosen `id` would fix it). Pending events can be clicked and dragged like real ones (id ""); US-05-02 should guard that. US-05-02/03/04 have no acceptance criterion yet for F16/U03. `keyed` at 23:xx gives 23:00. COVERAGE part 2 rows #3–#5 still say "not tested yet" (no test case); unit tests cover them.
- Manual checks: TC-05-01 (US-05-01#1, #2), US-05-01#3, US-05-01#4, US-05-01#5, edge case (offline box)
- Commits on main: 2d4c023 feat(events): create an event from the quick-entry box

## US-05-02 — review — 2026-10-23 19:23
- What: the full event editor (`calendar_core::editor`, drawn by `crates/app/src/editor.rs`). **More options** in the quick-entry box opens it with what was typed. **Edit** in an event's details opens an existing event in its own zone. It edits the title, description, start and end (typed in the system's formats; the end follows the start), all-day or multi-day, time zone, location, busy or free, visibility and calendar, grouped by account. Saving creates the event, or sends only what changed as a patch against the event's ETag, then moves it to another calendar of the same account. A change shows at once marked "Saving…" (`Saving::change`) and goes back on refusal. A 412 says the event was changed elsewhere and reloads it. **SQLite schema change:** `events` keeps description, location, show, visibility and etag; the schema is versioned by `user_version`, and an older cache is refetched in full. **Calendar-provider interface change:** `Provider::update`, `Provider::move_to`; `NewEvent` now carries a `Timing` and `About`.
- Decisions: DEC-106, DEC-107
- Reviews: code 1 round(s), spec 1 round(s); open notes: no blocking findings; spec review compliant. Fixed after code review: times left as opened keep the event's own (a twice-shown daylight-saving hour or seconds were re-sent); a move refused after a saved patch now says the other changes were saved; a location or title with spaces around is not re-sent unless changed; the migration also drops `last_synced`; a moved event leaves "saving" once it is in the new calendar even if Google kept its ETag. Not fixed (pre-existing modelling limit): an event whose end has another zone than its start gets the start's zone on its end when its times are edited. Editing an occurrence of a series is unavailable until US-05-07 (DEC-106), which must lift `REPEATING` in `crates/app/src/main.rs`. The description is plain text, and Google's HTML descriptions show raw. COVERAGE part 2 rows #1, #3, #5, #6 still say "not tested yet" (no test case); unit and engine tests cover them. Real Google's handling of the null-clearing patch and `If-Match` is proven only against the fake.
- Manual checks: TC-05-02 (drawing, real Google), US-05-02#1, US-05-02#3, US-05-02#5, US-05-02#6, edge cases (end before start, read-only, repeating, offline)
- Commits on main: 55d25fc feat(events): edit every field of an event in the full editor

## US-05-03 — review — 2026-10-23 19:39
- What: dragging and resizing in the day, 3-day and week views (`calendar_core::drag`, drawn by `crates/app/src/grid_view.rs`). An event moves by the distance dragged in 15-minute steps and whole days among those shown, keeping its length, with "start – end" shown while dragged; its bottom edge resizes the end in 15-minute steps. All-day events move by days in the strip and become timed (default length) when dropped in the hours; timed events dropped over the strip become all-day. A drop is sent like the editor's change (a times-only patch against the ETag), shown at once with "· Saving…", which all-day bars now show too. Read-only events, series occurrences and events being saved cannot be dragged. Replaces the feasibility check's in-memory `grid_view::apply`.
- Decisions: DEC-108
- Reviews: code 2 round(s), spec 1 round(s); open notes: round 1 BLOCKING fixed (the strip grew a lane mid-drag, turning small nudges into all-day drops; the preview is now an overlay). Also fixed: a zero drop no longer re-reads a fall-back hour, a sync ends a drag under way, preview times come from the same core functions as the save, clipped bars move left. Not fixed (cosmetic): with no all-day events the strip preview covers part of the day header; a sub-15-minute event's end preview shows 15 minutes when not moved; a midnight-crossing event draws only the grabbed segment moving; the end of a multi-day event's last segment cannot go above 00:15 of that day.
- Manual checks: TC-05-03 (drawing, real Google), US-05-03#2, US-05-03#3, edge cases (strip ↔ hours, not draggable, conflict)
- Commits on main: b334aec feat(events): drag events to another time and resize their end

## US-05-04 — review — 2026-10-23 19:58
- What: **Delete** beside **Edit** in an event's details, and Delete/Backspace with the details open, deletes an event at once with no question. It is sent to Google against its ETag on the sync worker, hidden at once (`Saving::delete`) and back if Google refuses. After Google takes a deletion, move, resize or edit, "Event deleted." / "Event saved." with **Undo** shows for 5 s. Ctrl+Z, or the notice's Undo, undoes that last change until the next one (`calendar_core::undo::LastChange`): a deletion is restored at Google (PATCH `status: confirmed`), and a change is patched back against its new ETag and moved back. An event changed elsewhere refuses the undo with a message. **Calendar-provider interface change:** `Provider::delete`, `Provider::restore`. No SQLite schema change.
- Decisions: DEC-109
- Reviews: code 1 round(s), spec 1 round(s); open notes: no blocking findings; the spec review found it compliant. Fixed after code review: undoing a move-only change is now sent against the version (title patched as it is); Ctrl+Shift+Z no longer undoes; removing an account drops its undo; a misplaced doc comment. Not fixed (recorded in DEC-109): a restore is sent without a version, so it is never refused as "changed elsewhere"; an undo Google refuses for a passing reason (network, not signed in) is not offered again, though the message says "try again"; a 410 on delete (already deleted elsewhere) counts as a deletion the user can undo. An event with guests is deleted without a question until MOD-06-F12. The tests repeat `main.rs`'s wiring of `Saving`/`LastChange` rather than drive `CalendarApp`. Real Google's undelete is proven only against the fake.
- Manual checks: TC-05-04 (drawing, real Google), TC-05-05 (keys, real Google), US-05-04#1, edge case (undo refused), guests deleted without a question (accept)
- Commits on main: b6c096e feat(events): delete events and undo the last change

## US-05-05 — review — 2026-10-23 20:14
- What: Events the user may not change — of a calendar the account cannot change, or a guest's copy whose organizer does not let guests change it (Google's `organizer.self` / `guestsCanModify`) — cannot be dragged, resized, edited or deleted. A drag on one emits `grid_view::Action::Unmovable` and the notice bar says why once; the details show the same reason with no Edit/Delete (decided on the freshly synchronized event). A change Google refuses shows Google's reason and the event returns as it was (path from US-05-02/03, now tested). **SQLite schema change:** `events.organizer_only`, version 2 (old caches refetch). **Calendar-provider interface change:** `SyncedEvent::organizer_only`. The fake Google now refuses a guest's patch with 403 `forbiddenForNonOrganizer`.
- Decisions: DEC-110
- Reviews: code 1 round(s), spec 1 round(s); open notes: no blocking findings, spec review compliant. Fixed after code review: the details decide from the fresh event, and a tested seam (`tell_unmovable`) carries the drag notice. Kept: a guest's copy cannot be edited at all, though Google lets guests change their own busy/free, colour and reminders (story's criterion, DEC-110). The details dialog and resize-by-edge refusal are proven in code, not by a test. The read-only details text changed from DEC-89's (TC-02-03's manual check updated). One squash commit: one logical change.
- Manual checks: TC-05-06 (drawing, real window), US-05-05#1 (real guest's copy), US-05-05#2 (real Google's refusal)
- Commits on main: e6e8b96 feat(events): guard events the user may not change

## US-05-06 — review — 2026-10-23 20:29
- What: The full editor of a new event gets **Repeats** (Daily, Every weekday, Weekly on chosen days, Monthly, Annually) and **Series ends** (Never, On a date, After N). New `core::repeat` turns them into Google's RRULEs (`UNTIL` is the last second of the day in the event's zone, in UTC, or a date for all-day events). The series is sent to Google with `recurrence`. The "saving" overlay now clears once the series' occurrences (`<id>_<start>`) are synchronized. **Calendar-provider interface change:** `NewEvent::recurrence`. `Save::Create` now boxes its `NewEvent`.
- Decisions: DEC-111, DEC-112
- Reviews: code 1 round(s), spec 1 round(s); open notes: no blocking findings, spec review compliant. Fixed after code review: a weekly series' single day follows the start, Weekly can no longer end up with no days, the `UNTIL` conversion reuses `recurrence::until_in_utc`, and the series row is labelled "Series ends". The owner should ratify DEC-111 (repeat for new events only). One squash commit: one logical change.
- Manual checks: TC-05-12, US-05-06#1 (real window), US-05-06#2 (real Google)
- Commits on main: 57f8914 feat(events): make a new event repeat with Google's presets [US-05-06]

## US-05-07 — review — 2026-10-23 21:01
- What: Occurrences of a series can now be dragged, resized, edited and deleted. Each asks "Edit/Delete recurring event": **This event**, **This and following events**, **All events** or **Cancel** (`calendar_core::series`, `Dialog::Scope`).
  - **This event** changes or deletes the occurrence by Google's instance ID. An unchanged one has no version, so it is sent without `If-Match`.
  - **All events** patches or deletes the master. New times keep the series' first day.
  - **This and following** puts `UNTIL` on the master the second before the occurrence, against its version. A change then copies the series whole at Google (guests, reminders, colour, conference) with the new fields. The rule's `COUNT` loses the occurrences before; deleted occurrences stay excluded (`EXDATE`). Changed occurrences from there on are deleted. On the first occurrence it is all events.
  - Following/all cannot move to another day (a rule change, `DEC-111`); only all events can change calendar.
  - The editor of an occurrence offers **Open in Google Calendar** (`eventedit/<eid>?authuser=`) for the repetition, and a complex rule is never rewritten but for a split's end.
  - **Calendar-provider interface change:** `Patch::recurrence`, `Provider::copy`, an empty version sends none. **Model change:** `Event::series`, and unchanged occurrences have an empty `etag`. No SQLite schema change. The fake Google now handles occurrence IDs, `events.get`, and deleting a series' occurrences with it.
- Decisions: DEC-113, DEC-114
- Reviews: code 2 round(s), spec 1 round(s).
  - Fixed after round 1: the split's new series dropped guests, reminders and colour (now `Provider::copy`); an occurrence changed on its own survived a following-delete or change (now deleted with the split); Cancel lost the editor's edits; a misleading message for invitations to one occurrence; two fake deviations.
  - Fixed after round 2: changed occurrences are deleted without a version and all are tried; Esc and close also return to the editor.
  - Spec review compliant.
  - Not fixed:
    - a drop replaces another open dialog;
    - deleted occurrences beyond the cache window (24 months) reappear after a split of a never-ending series;
    - the `eid` uses the full calendar ID (manual check);
    - an unparseable `EXDATE` is dropped from the new series;
    - changed occurrences after a split lose their own changes;
    - series changes cannot be undone (`DEC-114`).
  - The owner should ratify DEC-113 (no day change for following/all) and DEC-114.
  - One squash commit: one logical change.
- Manual checks: TC-05-08, US-05-07#2, US-05-07#3, US-05-07#4, edge case (copy keeps guests, no e-mail)
- Commits on main: e7b3aa4 feat(events): change or delete occurrences by scope [US-05-07]

## US-05-08 — review — 2026-10-23 21:16
- What: Google's event types read, cached and shown (out of office hatched, a mark per type, working locations and birthdays as read-only labels in the all-day strip); out of office and focus time creatable on a Workspace primary calendar, all day as midnight to midnight, with Google Calendar's auto-decline defaults.
- Decisions: DEC-115, DEC-116, DEC-117
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - Code review found nothing blocking. Fixed: dragging out of office/focus time into the strip stays timed; the editor no longer offers to move a special event to another calendar (DEC-117).
  - Not fixed: `Editor::kinds(&mut self)` also resets the type (a getter with a side effect); an all-day out of office on a spring-forward day (23 h) is drawn in the hours, not the strip; "always busy" is enforced both in the editor and in the Google adapter.
  - Spec review compliant. The SQLite cache schema changed (`kind` column, VERSION 3: the cache refetches in full).
  - The owner should ratify DEC-115 (hatching only in the hour grid) and DEC-116 (auto-decline only new invitations).
  - One squash commit: one logical change.
- Manual checks: US-05-08#1, TC-05-09 (US-05-08#2 against real Google), US-05-08#3
- Commits on main: 2fbf2ce feat(events): show and create special event types [US-05-08]

## US-05-09 — review — 2026-10-23 21:28
- What: an event's own Google colour (`colorId` 1–11) is read, cached (cache schema VERSION 4, `events.color`) and shown over its calendar's colour in every view; the editor's **Colour** row picks one of Google Calendar's eleven or "Calendar colour", sent as a patch (`null` takes it away), also on create, undo and "this and following".
- Decisions: DEC-118
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - Code review found nothing blocking. Fixed anyway: keyboard focus ring and accessible names on the colour swatches, a direct colour table instead of a name lookup, and the module's Integrations line (built-in palette, DEC-118).
  - Spec review compliant. The SQLite cache schema changed (`color` column, VERSION 4: the cache refetches in full); `About.color` and `Patch.color` now travel through the calendar-provider calls (trait signatures unchanged).
  - The owner should ratify DEC-118 (built-in palette, not the API's `colors` endpoint; an event's own colour shown over a local calendar colour).
  - One squash commit: one logical change.
- Manual checks: US-05-09#1, TC-05-10 (US-05-09#2 against real Google)
- Commits on main: 172fa69 feat(events): show and change an event's own colour [US-05-09]

## US-05-10 — review — 2026-10-23 21:35
- What: Google's event attachments are read (title, `fileUrl`), cached (new `attachments` column) and listed in the event's details as 📎 links that open in the default browser through the `Browser` platform interface.
- Decisions: DEC-119
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - Code review found nothing blocking. Fixed anyway: a comment that said Google, not the app's copy of the event, carries attachments to a split series.
  - Spec review compliant. The SQLite cache schema changed (`attachments` column, VERSION 5: the cache refetches in full); `calendar-core` now depends on `serde_json` to store them. The calendar-provider interface is unchanged.
  - The owner should ratify DEC-119 (untitled attachments listed by their link; only `https://` links listed and opened).
  - One squash commit: one logical change.
- Manual checks: TC-05-11 (US-05-10#1, US-05-10#2 against real Google)
- Commits on main: 944ce00 feat(events): list an event's attachments as links [US-05-10]

## US-06-01 — review — 2026-10-23 21:56
- What: Guests can be added to an event in the editor by address or from suggestions, marked optional, and removed. Suggestions come from Google Contacts and other contacts (People API, with a warm-up), from the directory on Workspace accounts that granted it, and from recent guests of the account's cached events. A new event with guests is created with `sendUpdates=all`. A guest change merges into Google's attendee entries.
- Decisions: DEC-120, DEC-121, DEC-122
- Reviews: code 2 round(s), spec 1 round(s); open notes:
  - No blocking findings.
  - Fixed after round 1:
    - one occurrence's answers were copied to the whole series;
    - attendee fields the app does not model (comment, +1s, rooms) were lost;
    - a failed warm-up was never retried;
    - the recent-guests dedup grew with the square of the guest count;
    - a search was not sent again when the editor's account changed.
  - Fixed after round 2: guests Google has no entry for are sent unanswered; one case rule for addresses.
  - Left as NITs:
    - an unchanged occurrence is patched without If-Match;
    - the warm-up is retried on every search while the People API keeps refusing;
    - searches share the sync worker with saves.
  - Spec review: compliant.
  - Changes the owner should know of:
    - the SQLite schema has a new `guests` column, and its VERSION went from 5 to 6, so the cache is fetched again in full;
    - the calendar-provider interface gained an optional `Provider::people`.
  - Until `US-06-03`, a guest added to an existing event gets no e-mail.
  - One squash commit: one logical change.
- Manual checks: TC-06-13, TC-06-01, TC-06-02, US-06-01#4, US-06-01#5
- Commits on main: cb8807f feat(guests): invite guests with suggestions [US-06-01]

## US-06-02 — review — 2026-10-24 03:50
- What: event details list everyone invited with name, address and an answer icon (✔ ✖ ❓ ⏳), collapsed beyond 10 with "Show N more"; open details follow the latest synchronization. Guest data and its sync were already on main from US-06-01.
- Decisions: DEC-123
- Reviews: code 1 round(s), spec 1 round(s); open notes: the open dialog's refresh is not unit-tested (UI, covered by a manual check); COVERAGE row `US-06-02#3` has no TC (`DEC-65`), unit-tested only
- Manual checks: TC-06-03 (US-06-02#1, #2), US-06-02#3
- Commits on main: 67ee24c feat(guests): show guests and their answers in details [US-06-02] (squash, one logical change)

## US-06-03 — review — 2026-10-24 04:05
- What: changing (drag or editor) or deleting an event with guests asks once "Send update/cancellation e-mails to the guests?" (Back / Don't send / Send), after the scope question for occurrences; the answer goes to Google as `sendUpdates=all|none` on every write the change makes, undo reusing it.
- Decisions: DEC-124
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - the calendar-provider interface changed: `Provider::update/copy/move_to/delete/restore` take `notify`;
  - fixed after review: a drag that asks nothing no longer closes an open dialog;
  - not done: turning the answer into `Edit.notify` / the delete flag lives in the UI's `saved` and is not unit-tested;
  - not done: deleting an invitation the user may modify as a guest also asks (not checked against Google);
  - Back after a deletion closes the details.
  - One squash commit: one logical change.
- Manual checks: TC-06-04 (US-06-03#1), US-06-03#2
- Commits on main: d659348 feat(guests): ask whether to notify guests of a change [US-06-03]

## US-06-04 — review — 2026-10-24 04:15
- What: invitations whose own guest entry is `needsAction` get a pale fill and a dashed outline in the day/3-day/week, month and agenda views; the side panel shows "N invitations awaiting an answer" (only when N > 0), opening an "Invitations" list whose rows open the details; disabled calendars' invitations are neither shown nor counted.
- Decisions: DEC-125, DEC-126, DEC-127
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - fixed after review: Google's `self` is the owner of the calendar the copy is on, so only entries with the account's own address are counted (`DEC-127`); a test now pins that hidden calendars stay counted; the fill is square like its dashes;
  - not done: COVERAGE row `US-06-04#3` still says "not tested yet", though `invitations_of_a_calendar_disabled_in_the_settings_are_neither_shown_nor_counted` covers it (COVERAGE is written by /og:tests, left as earlier stories did);
  - not done: TC-06-05's preconditions name a fake Google with contacts; the tests use fixtures, and the Google mapping is tested separately;
  - not done: the drawing and the counter → list → details clicks are UI, not unit-tested (manual checks);
  - no SQLite schema or provider-interface change.
  - One squash commit: one logical change.
- Manual checks: TC-06-05 (US-06-04#1, #2), US-06-04#3
- Commits on main: 535fa1f feat(guests): mark and list invitations awaiting an answer [US-06-04]

## US-06-05 — review — 2026-10-24 04:27
- What: Accept / Maybe / Decline in a "Going?" row at the top of an invitation's details and in each row of the invitations list; the answer patches the account's own attendee entry (`Patch.response`), an occurrence asking "This event" (instance) or "All events" (master); a guest's copy only its organizer may change can be answered but not dragged or edited.
- Decisions: DEC-128, DEC-129
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - fixed after review: the organizer's own answer is sent with `sendUpdates=none`; Google's `attendeesOmitted` is kept on an answer; list rows without an answerable entry show no buttons; a test now proves other guests' answers stay;
  - not done: Ctrl+Z after an edit and then an answer to the same event within 5 s reverts against an old version and reports "could not be undone" (harmless, left);
  - not done: no automated test drives the answer buttons offline (UI wiring of `offline::refused`; manual check);
  - the provider interface's `Patch` gained `response` (calendar-provider interface change, constitution review rule); no SQLite schema change.
  - One squash commit: one logical change.
- Manual checks: TC-06-06 (US-06-05#1, #2), TC-06-07 (US-06-05#3), TC-06-08 (US-06-05#4)
- Commits on main: f5cf06d feat(guests): accept, decline or maybe for one or all events [US-06-05]

## US-06-06 — review — 2026-10-24 04:34
- What: an event whose own guest entry (Google's `self`) is `declined` stays in the day/3-day/week, month and agenda views with its text struck through and the event faded; a "Show declined events" checkbox in the view bar (on by default, kept in the settings as `declined = hidden`) leaves declined events out of every view and brings them back.
- Decisions: DEC-130
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - not done: the drag previews (an all-day event dragged into the hours, a timed one over the all-day strip) are drawn unstruck and unfaded until dropped (cosmetic, left);
  - not done: the strike-through, the fading and the checkbox are UI, not unit-tested (manual checks);
  - settings gained the `declined` key; no SQLite schema or provider-interface change.
  - One squash commit: one logical change.
- Manual checks: TC-06-09 (US-06-06#1, #2)
- Commits on main: 3e9a3b3 feat(guests): strike through and hide declined events [US-06-06]

## US-06-07 — review — 2026-10-24 04:51
- What: the editor has an "Add Google Meet" switch. It turns on with the first guest; a saved event with it on asks Google for a Meet call, and switching it adds or removes the call on an existing event. The Meet link is synchronized and cached, and opens from "Join with Google Meet" in the details and "Join" in the agenda. If Google refuses the call (400), the event is saved without it and the user is told.
- Decisions: DEC-131, DEC-132, DEC-133
- Reviews: code 2 round(s), spec 1 round(s); open notes:
  - fixed in round 1: Join in the agenda moved egui's layout cursor (`ui.put` → `ui.place`); the retry without Meet is now limited to Google's 400; the "without Meet" notice now also appears when a move fails after the patch;
  - left: "saved without Google Meet" is shown wrongly for a call Google is still making (`pending`), or when the synchronization right after a change fails (both rare, written down in DEC-132); every 400 from Google now reads "Google refused it as invalid (…)"; a long agenda title runs under the Join button; no notice after a "this and following" split;
  - SQLite schema change: cache VERSION 6 → 7, new `events.meet` column (events are fetched again);
  - calendar-provider interface: method signatures unchanged; new fields in the data it carries: `NewEvent.meet`, `Patch.meet`, `About.meet`;
  - one squash commit: one logical change.
- Manual checks: TC-06-10 (US-06-07#2 against real Google), TC-06-11 (US-06-07#3), the no-Meet edge case
- Commits on main: 26fe73d feat(meet): add Google Meet switch and Join button [US-06-07]

## US-06-08 — review — 2026-10-24 05:05
- What: the editor shows an "Availability" row under the time: one strip per guest (first 20) with their busy blocks for the event's day, from Google's `freeBusy.query` run off the UI thread, the chosen time as a band across the strips, "Conflict" for a guest busy then and "No data" for one who does not share; no preview offline, when Google does not answer, for all-day events, out of office and focus time.
- Decisions: DEC-134, DEC-135
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - fixed after code review: no preview for out of office / focus time; an existing event's saved time is cut only for invited, not-declined guests of an event shown as busy;
  - left: a failed free/busy query is not retried until the day, guests or calendar change or the editor reopens; the same query is sent again after the time field passes through an unreadable state; the row disappears until a changed query is answered; the strips are UI, not unit-tested (manual checks);
  - calendar-provider interface change: new `Provider::busy` method (default answers "no data" for everyone); no SQLite schema change;
  - one squash commit: one logical change.
- Manual checks: TC-06-12 (US-06-08#1, #2 against real Google), US-06-08#3, US-06-08#5
- Commits on main: 15a45d2 feat(guests): show guests' availability in the editor [US-06-08]

## US-07-01 — review — 2026-10-24 05:25
- What: reminders set in Google (an event's own, or its calendar's defaults) are read, cached and shown as KDE notifications through `org.freedesktop.Notifications`. Each shows once, with title, start time and location, from the sync worker, so they also come while the window is minimized. E-mail reminders, declined events and disabled calendars give none. Reminders from before the app started are skipped; those that fell during sleep show on waking if the event has not started. Clicking a notification brings the window forward and opens the event on its day.
- Decisions: DEC-136, DEC-137, DEC-138
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - fixed after code review: reminders are checked before the jobs a wake-up starts, so a slow sync after sleep does not drop them; `Job::Remind` no longer shows "synchronizing"; D-Bus signals are filtered by sender;
  - left:
    - on Wayland, un-minimizing and focusing from a click may be refused (no xdg-activation token), to be seen in TC-07-07;
    - a reminder whose time comes while a long job (e.g. a full sync) runs waits for it;
    - the body is always markup-escaped;
    - timed working-location events using the defaults remind like other events;
    - the worker loop's reminder wake-up has no loop-level test;
  - **SQLite schema change:** `calendars.reminders` and `events.reminders`; cache version 8 also drops the calendars, so they are fetched again. No calendar-provider interface change; `Calendar` gains `default_reminders`, `About` gains `reminders`;
  - one squash commit: one logical change.
- Manual checks: TC-07-07 (US-07-01#3), US-07-01#1, #2 on real KDE, US-07-01#7 (real sleep)
- Commits on main: 4257d1e feat(reminders): show desktop notifications for reminders [US-07-01]

## US-07-02 — review — 2026-10-24 05:39
- What: reminder notifications offer Join (Meet), Snooze 5/10 minutes and Dismiss over D-Bus; snooze is local and follows its event; without notification buttons the event's details offer Snooze.
- Decisions: DEC-139
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - no blocking findings; spec review found nothing unmet;
  - fixed after code review: a snooze counts from when it was chosen, not when the worker gets to it; an unknown action key keeps the notification's callback; only 5 and 10 are accepted as snooze keys; the Dismiss and Snooze 10 test was split in two;
  - left:
    - the Join-to-browser and Snooze-to-worker routing in `remind_with` uses `SystemBrowser` directly and has no test;
    - the notification callback holds a `jobs` sender, so the worker no longer sees the channel close (matters only at exit);
    - `buttons()` caches its first answer for the run, also a failure;
    - GNOME shows at most 3 buttons (outside the MVP);
  - `Notifier` (platform interface) changed: `show` takes `Fn(Action)` and `buttons()` was added. No SQLite schema change and no calendar-provider interface change;
  - one squash commit: one logical change.
- Manual checks: TC-07-07 (US-07-02#3), US-07-02#1, #2 on real KDE, US-07-02 no-buttons edge case
- Commits on main: 33923dd feat(reminders): snooze, dismiss and join from notifications [US-07-02]

## US-07-03 — review — 2026-10-24 05:52
- What: a "Desktop notifications" switch in the settings and "Mute notifications" in a calendar's side-panel menu; silenced reminders are skipped in the sync worker, nothing goes to Google, and the calendar's events stay shown.
- Decisions: DEC-140
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - the code review found nothing blocking. Its should-fix was a late silence change while the worker was busy; fixed by sharing the state with the worker (`Arc<Mutex<Silenced>>`) instead of queuing a job, which also passes the startup state in `Synchronization::start` like `disabled`;
  - the UI wiring (the settings checkbox, the menu item's click) has no automated test, like the rest of `main.rs`;
  - no SQLite schema change, no calendar-provider or platform interface change; two settings-file keys were added (`notifications`, `muted`);
  - one squash commit: one logical change.
- Manual checks: TC-07-04 (US-07-03#2), US-07-03#1, US-07-03#3 on real KDE and Google
- Commits on main: aeb9140 feat(reminders): turn notifications off and mute a calendar [US-07-03]

## US-07-04 — review — 2026-10-24 06:08
- What: the editor's "Notifications" section: a new event shows and keeps its calendar's defaults; notification/e-mail reminders in minutes, hours or days are added, changed and removed (at most 5, at most 4 weeks before) and sent to Google; "Use calendar defaults" resets (`useDefault` with empty overrides). The fake Google now refuses defaults with overrides, and more than five, as Google does.
- Decisions: DEC-141
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - the code review found nothing blocking. Fixed: a test for reminders in "this and following"; the number field no longer clamps silently when the unit changes; the fake records a patch only once it is taken;
  - the code review also suggested not asking about guest e-mails for a reminders-only change. It was tried and reverted: it would narrow the `must` MOD-06-F12, which is the owner's call. Today, saving reminders of an event with guests still asks "Send update e-mails to the guests?" (Google Calendar does not ask);
  - a guest's copy that only the organizer may change stays read-only in the editor, so a guest cannot set their own reminders there, though Google allows it. This is outside this story;
  - spec review: compliant. Added a core check of the 5-reminder cap (`an_event_takes_five_reminders_and_no_more`). The TC-07-05 test skips the fake notifier and the clock in the TC's preconditions, as none of its steps uses them;
  - no SQLite schema change. The calendar-provider interface is unchanged in signature; `Patch` gained a `reminders` field;
  - one squash commit: one logical change.
- Manual checks: US-07-04#1, US-07-04#2/#3, US-07-04#4, duplicate-reminder edge case (real Google)
- Commits on main: 86cf4e6 feat(reminders): edit an event's reminders in the editor [US-07-04]

## US-07-05 — review — 2026-10-24 06:35
- What: closing the window leaves the app in the KDE tray (ksni StatusNotifierItem), where it keeps synchronizing and reminding. Clicking the icon shows or hides the window; the menu has "Open Calendar Buddy" and "Quit". "Start at login" writes an XDG autostart entry that starts the app with `--in-tray`. A single instance holds the D-Bus name `pl.ordergroup.CalendarBuddy` and is reached through `org.freedesktop.Application.Activate`. On Wayland a window cannot be hidden, so it is closed and reopened (`run_native_ext` with one shared egui context), and the app's state is kept.
- Decisions: DEC-142, DEC-143, DEC-144, DEC-145
- Reviews: code 2 round(s), spec 1 round(s); open notes:
  - round 1 found nothing blocking. Fixed:
    - "bring forward" did nothing on Wayland; the window is now closed and reopened;
    - an autostart race with Plasma's tray at login; the app now waits for the tray;
    - time-zone changes and the sync window are now applied while in the tray;
    - "Start at login" changes are now written in order by one writer thread;
    - `%` in the Exec path is doubled, and " (deleted)" is stripped from the program's path.
  - round 2 fixed: "Start at login" is hidden when there is no tray; a reopened window keeps its size and maximized state.
  - left as is:
    - a double "Open" may reopen the window twice (one extra flicker);
    - each reopen adds one font provider to the shared egui context;
    - the single-instance test needs `dbus-daemon` installed;
    - the reopen loop in `main()` has no automated test.
  - follow-up outside this story: `open_notified` (US-07-01) still uses `Minimized(false)`/`Focus`, which do nothing on Wayland. A notification clicked while the window is open but behind other windows does not bring it forward.
  - smoke run on real KDE without accounts:
    - the start in the tray and the second launch were confirmed;
    - hide and re-show could not be watched, because the screen was off (DPMS).
  - no SQLite schema change and no calendar-provider change. The platform interfaces gained `Tray`, `Autostart` and `Instance`, and there is a new dependency, `ksni` 0.3.7.
  - one squash commit: one logical change.
- Manual checks: TC-07-08 (US-07-05#1, #3, #4), US-07-05#2, TC-07-06 on real KDE (US-07-05#5)
- Commits on main: 08ccb64 feat(reminders): keep reminding from the tray, start at login [US-07-05]

## US-07-06 — review — 2026-10-24 06:41
- What: a `webcal:` link launches the app, which opens Google Calendar's "add calendar from URL" page (`/calendar/r?cid=…`) in the browser and ends. The running app registers `~/.local/share/applications/calendar-buddy-webcal.desktop` (`x-scheme-handler/webcal`) behind a new `LinkHandler` platform interface.
- Decisions: DEC-146
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - none blocking;
  - `webcals:` is not handled (not in the spec);
  - a failed browser launch is silent, because there is no window to report it in;
  - no `update-desktop-database` (KDE reads the entries directly);
  - the app does not change the user's default `webcal:` program.
- Manual checks: TC-07-09 (US-07-06#1)
- Commits on main: 4a14c9f feat(reminders): open webcal links in Google Calendar [US-07-06] (one squash commit: one logical change)

## US-04-01 — review — 2026-10-24 06:52
- What: a System / Light / Dark theme under Settings → Appearance, kept as `theme = light|dark`. With System, the app follows the desktop's preference live. The preference comes from the XDG portal's `color-scheme` (`ReadOne` + `SettingChanged`) behind a new `Appearance` platform interface, and light is used when the desktop states none. The themes are egui's default light and dark looks until US-04-03.
- Decisions: DEC-147
- Reviews: code 1 round(s), spec 1 round(s); open notes:
  - none blocking;
  - the theme preference itself is now compared (fixed);
  - the platform header was re-wrapped (fixed);
  - the wiring and the radio buttons are covered only by manual checks;
  - until the portal answers, the app is light, so a dark desktop may show a light first frame.
- Manual checks: TC-04-01 (US-04-01#1), TC-04-02 (US-04-01#2, #3)
- Commits on main: 2190680 feat(themes): follow the desktop's light or dark theme [US-04-01] (one squash commit: one logical change)
