# Manual checks

What only a person can verify, collected by the delivery loop per story.

## US-03-02

- [ ] `US-03-02#2`, `#3` (no test case; content proven by view-model tests) — run `cargo run -p calendar-buddy`, pick **Day** and **3 days** in the top bar: the grid draws one / three full-width columns with the all-day strip, overlapping events side by side, the red current-time line on today, today's header highlighted and free/busy events as "Busy" blocks, the same as in **Week**; ‹ › move by one / three days.

## US-03-03

- [ ] `TC-03-06` / `US-03-03#1`, `#2` (content and "+N more" count proven by `tc_03_06_*` view-model tests; the drawing and the click are not) — run `cargo run -p calendar-buddy`, pick **Month**: the month's whole weeks are drawn, the neighbouring months' days dimmed, each day with its events (all-day chips first, then "HH:MM title" with the calendar's dot); a full day ends in "+N more", and clicking it opens a window listing all of that day's events, which closes with its ×; ‹ › move by one month.
- [ ] `US-03-03#3` (no test case; proven by `today_is_highlighted_and_free_busy_events_are_busy_without_details`) — in **Month**, today's date has a highlighted circle, and events of the free/busy calendar read "Busy" with no title.
