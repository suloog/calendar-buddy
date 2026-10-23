# Delivery

Stories are delivered by a thin local loop: `scripts/loop/deliver.sh <US-…> …`.

- One fresh, unattended Claude session per story (brief: `scripts/loop/BRIEF.md`) — local branch
  `story/<ID>`, gates, `og:code-reviewer` and `og:spec-compliance-reviewer`, then a merge into
  `main` (squash or a few logical commits). Nothing is pushed; the owner pushes `main`.
- The loop decides on its own and records every decision as a `DEC-nn` in
  `docs/specs/DECISIONS.md`; what it would have to ask the owner goes to `PARKED.md`.
- `LOG.md` — one entry per story. `USAGE.md` — wall time, tokens and the change in the account's
  5-hour / 7-day limits per story. `MANUAL-CHECKS.md` — what only a person can verify (real Google
  sign-in, KDE notifications, the look), to go through in one sitting.
- Limits: the loop waits when the 5-hour window is full (100%) and stops when the 7-day window is full
  (`LOOP_FIVE_HOUR_MAX`, `LOOP_SEVEN_DAY_MAX`). Model: Opus (`LOOP_MODEL`), effort high (`LOOP_EFFORT`).
