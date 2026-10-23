# Delivery usage

One row per story run by `scripts/loop/deliver.sh`. Limit columns are the account's usage
(percent of the 5-hour and 7-day windows) before → after; Δ is the story's share, and it also
includes anything else used on the account meanwhile. `~` marks a window that reset during the
story, so its Δ is not meaningful. Cost is Claude Code's own figure (notional on a subscription).

| Story | Result | Started | Wall time | Turns | Tokens in / out (cache read) | Cost (USD) | 5h before → after (Δ) | 7d before → after (Δ) |
| ----- | ------ | ------- | --------- | ----- | ---------------------------- | ---------- | --------------------- | --------------------- |
| US-03-02 | review | 2026-10-23 13:18 | 4 min | 27 | 71113 / 18017 (1351524) | 1.73 | 43.0% → 52.0% (~) | 37.0% → 38.0% (~) |
| US-03-03 | review | 2026-10-23 13:22 | 5 min | 5 | 5880 / 2702 (451917) | 1.93 | 52.0% → 62.0% (~) | 38.0% → 39.0% (~) |
| US-03-04 | review | 2026-10-23 13:26 | 5 min | 35 | 76985 / 21788 (1833919) | 2 | 62.0% → 71.0% (~) | 39.0% → 40.0% (~) |
