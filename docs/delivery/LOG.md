# Delivery log

## US-03-02 — review — 2026-10-23 14:00
- What: day and 3-day views on the shared hour grid (`GridView`, `GridViewModel`, `grid_view.rs`), with a plain Day / 3 days / Week picker; ‹ › step by the shown days.
- Decisions: DEC-69
- Reviews: code 1 round(s), spec 1 round(s); open notes: `Week::next`/`previous` now unused outside tests (kept for US-03-05); dragging an event past the grid edge moves it to an unshown day (feasibility-only drag, MOD-05).
- Manual checks: US-03-02#2, #3 (drawing of the 1- and 3-column grid; no TC)
- Commits on main: one squash commit — feat(views): add day and 3-day views
