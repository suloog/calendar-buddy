# Delivery usage

One row per story run by `scripts/loop/deliver.sh`. Limit columns are the account's usage
(percent of the 5-hour and 7-day windows) before → after; Δ is the story's share, and it also
includes anything else used on the account meanwhile. `~` marks a window that reset during the
story (usage dropped), so its Δ is not meaningful. Cost is Claude Code's own figure for the whole session including
its subagents (notional on a subscription); the token columns count the top-level session only.

| Story | Result | Started | Wall time | Turns | Tokens in / out (cache read) | Cost (USD) | 5h before → after (Δ) | 7d before → after (Δ) |
| ----- | ------ | ------- | --------- | ----- | ---------------------------- | ---------- | --------------------- | --------------------- |
| US-03-02 | review | 2026-10-23 13:18 | 4 min | 27 | 71113 / 18017 (1351524) | 1.73 | 43.0% → 52.0% (+9pp) | 37.0% → 38.0% (+1pp) |
| US-03-03 | review | 2026-10-23 13:22 | 5 min | 5 | 5880 / 2702 (451917) | 1.93 | 52.0% → 62.0% (+10pp) | 38.0% → 39.0% (+1pp) |
| US-03-04 | review | 2026-10-23 13:26 | 5 min | 35 | 76985 / 21788 (1833919) | 2 | 62.0% → 71.0% (+9pp) | 39.0% → 40.0% (+1pp) |
| US-03-05 | review | 2026-10-23 13:46 | 6 min | 33 | 82095 / 24853 (1916692) | 2.14 | 75.0% → 84.0% (+9pp) | 41.0% → 42.0% (+1pp) |
| US-03-06 | failed | 2026-10-23 13:51 | 1 min | 3 | 15754 / 585 (72124) | 0.15 | 84.0% → 85.0% (+1pp) | 42.0% → 42.0% (+0pp) |
