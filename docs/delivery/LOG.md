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
