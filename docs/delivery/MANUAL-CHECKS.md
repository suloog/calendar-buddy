# Manual checks

What only a person can verify, collected by the delivery loop per story.

## US-03-02

- [ ] `US-03-02#2`, `#3` (no test case; content proven by view-model tests) — run `cargo run -p calendar-buddy`, pick **Day** and **3 days** in the top bar: the grid draws one / three full-width columns with the all-day strip, overlapping events side by side, the red current-time line on today, today's header highlighted and free/busy events as "Busy" blocks, the same as in **Week**; ‹ › move by one / three days.
