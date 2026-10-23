# Manual checks

What only a person can verify, collected by the delivery loop per story.

## US-03-02

- [x] `US-03-02#2`, `#3` (no test case; content proven by view-model tests) — run `cargo run -p calendar-buddy`, pick **Day** and **3 days** in the top bar: the grid draws one / three full-width columns with the all-day strip, overlapping events side by side, the red current-time line on today, today's header highlighted and free/busy events as "Busy" blocks, the same as in **Week**; ‹ › move by one / three days.

## US-03-03

- [x] `TC-03-06` / `US-03-03#1`, `#2` (content and "+N more" count proven by `tc_03_06_*` view-model tests; the drawing and the click are not) — run `cargo run -p calendar-buddy`, pick **Month**: the month's whole weeks are drawn, the neighbouring months' days dimmed, each day with its events (all-day chips first, then "HH:MM title" with the calendar's dot); a full day ends in "+N more", and clicking it opens a window listing all of that day's events, which closes with its ×; ‹ › move by one month.
- [x] `US-03-03#3` (no test case; proven by `today_is_highlighted_and_free_busy_events_are_busy_without_details`) — in **Month**, today's date has a highlighted circle, and events of the free/busy calendar read "Busy" with no title.

## US-03-04

- [x] `TC-03-07` / `US-03-04#1`, `#3` (content proven by `tc_03_07_*` view-model tests; the drawing is not) — run `cargo run -p calendar-buddy`, pick **Agenda**: the list starts at today, one group per day with events (date on the left, then each event's dot, "HH:MM – HH:MM" or "All day", and title), with no empty days; ‹ › are disabled. The "No upcoming events" text needs a store with nothing ahead (not reachable with the fixture profile until sync exists).
- [x] `US-03-04#2` (no test case; proven by `events_past_the_horizon_load_with_a_longer_horizon` and `more_days_load_only_when_scrolled_near_the_end`) — in **Agenda**, scroll down past the first month: more days keep appearing near the end, and scrolling stays smooth on the ~20,000-event profile.
- [x] `US-03-04#4` (no test case; proven by `today_is_highlighted_and_free_busy_events_are_busy_without_details`) — in **Agenda**, today's date has a highlighted circle, and events of the free/busy calendar read "Busy" with their time and no title.

## US-03-05

- [x] `TC-03-08` / `US-03-05#1`, `#2`, `#4` (navigation and restart proven by `tc_03_08_*` tests; the buttons and the window are not) — run `cargo run -p calendar-buddy`: the top bar has **Today**, ‹ ›, the period title and, on the right, Day / 3 days / Week / Month / Agenda. From the week view pick **Month** (same month), ›, then **Today** (back to today's month); pick **3 days**, › twice, close the app and start it again: it opens in **3 days** starting today.
- [x] `US-03-05#3` (no test case; marking and jumping proven by `the_mini_calendar_*` and `a_date_picked_in_the_mini_calendar_is_shown_in_the_view`) — the left panel shows a month with today in the accent colour and the days on screen highlighted (the week's 7 days, the 3 days, the whole month; nothing in Agenda); its own ‹ › browse months without moving the view; clicking a date shows it in the current view (from Agenda: the day view), and **Today** brings the mini calendar back.

## US-03-08

- [x] `TC-03-11` / `US-03-08#1`, `#2`, `#4` (keys, periods and the list proven by `tc_03_11_*` tests; the wiring in the window is not) — run `cargo run -p calendar-buddy`: `d` `x` `w` `m` `a` switch views, `j`/`n` and `k`/`p` move a period, `t` comes back to today (and brings back a browsed mini calendar); `g` opens "Go to date" with the field focused and pre-filled, Enter or **Go** shows the date, a wrong date keeps the field; `?` lists all 13 keys; Escape closes each dialog; keys typed in the date field or with Alt do nothing.
- [x] `US-03-08#3` (no test case; key mapping proven by `g_asks_for_a_date_and_c_starts_a_new_event_without_moving_the_view`) — `c` opens "New event" naming the selected day (the editor itself is `MOD-05`).
- [x] `US-03-08#5` (no test case) — without the mouse, Tab / Shift+Tab move through the top bar, the side panel and the events on screen in Day, 3 days, Week, Month (incl. the "+N more" list) and Agenda; the focused event is ringed and scrolled into sight; Enter or Space opens its details (title or "Busy", time); Escape closes them with focus back on the event; the "+N more" list closes on Escape.

## US-01-01

Needs the real OAuth client file at `~/.config/calendar-buddy/google-oauth-client.json` (`DEC-77`), a real Google account and the KDE session. The app does not yet remember the account across restarts (`DEC-80`), so every start shows the welcome screen.

- [ ] `TC-01-01` / `US-01-01#1`, `#2`, `#3` (the flow against the fake Google proven by `a_personal_account_is_connected_and_its_refresh_token_kept_in_the_secret_store` and `a_workspace_account_is_asked_for_the_directory_on_its_own`; the screens, the real browser and real Google are not) — run `cargo run -p calendar-buddy`: the welcome screen shows one button, **Connect Google account**. Click it: before the browser opens, the app explains Google's "unverified app" warning ("Advanced" → "Go to … (unsafe)"). **Continue in the browser**, pick the test account, get past the warning, consent: the browser tab says it can be closed, and the app shows the calendar with "Connected <e-mail>." at the top — nothing was typed into the app. KWallet (Wallet Manager) now holds an item "Calendar Buddy: <e-mail>". Close the notice with ✕.
- [ ] `US-01-01#4` (no test case; the scopes asked are proven by `the_authorization_request_asks_for_calendar_and_contacts_with_pkce` and the two tests above) — on Google's consent screen for a **personal** account: only calendar and contacts (plus name/e-mail), no directory. With a **Workspace** account: the same first screen, then the browser opens a second time asking only for the organization's directory. Unticking contacts (or the directory) still connects the account, and the notice names "contact suggestions" (or "colleague suggestions") as not available; unticking calendar does not connect it and the welcome screen says why.
- [ ] `US-01-01#5` (no test case; the check proven by `offline_signing_in_is_refused_and_works_again_once_the_network_is_back`) — with the network off (e.g. `nmcli networking off`), click **Connect Google account**: the app says signing in is not possible offline and offers **Try again**; turn the network on, **Try again** goes on to the "unverified app" heads-up.
- [ ] `US-01-01#6` (no test case; proven by `without_a_secret_store_connecting_stops_before_the_browser` and `the_failures_of_a_first_run_explain_themselves`) — with no secret store answering (e.g. disable KWallet in System Settings → KDE Wallet and stop `kwalletd6`; or run the app with `DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent`), **Connect Google account** stops before the browser and says to start or install KWallet, GNOME Keyring or KeePassXC.
- [ ] Edge cases (proven against the fake Google by `cancelling_on_googles_screen_adds_no_account`, `a_closed_browser_ends_in_a_time_out_or_the_users_cancel`, `a_missing_client_file_stops_connecting_before_anything_else`) — in the real app: **Cancel** on Google's screen, or closing the browser tab and clicking **Cancel** in the app, says no account was connected and offers **Try again**; with the client file renamed away, **Connect Google account** names the full path where to put it.
