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

- [ ] `TC-03-08` / `US-03-05#1`, `#2`, `#4` (navigation and restart proven by `tc_03_08_*` tests; the buttons and the window are not) — run `cargo run -p calendar-buddy`: the top bar has **Today**, ‹ ›, the period title and, on the right, Day / 3 days / Week / Month / Agenda. From the week view pick **Month** (same month), ›, then **Today** (back to today's month); pick **3 days**, › twice, close the app and start it again: it opens in **3 days** starting today.
- [ ] `US-03-05#3` (no test case; marking and jumping proven by `the_mini_calendar_*` and `a_date_picked_in_the_mini_calendar_is_shown_in_the_view`) — the left panel shows a month with today in the accent colour and the days on screen highlighted (the week's 7 days, the 3 days, the whole month; nothing in Agenda); its own ‹ › browse months without moving the view; clicking a date shows it in the current view (from Agenda: the day view), and **Today** brings the mini calendar back.
