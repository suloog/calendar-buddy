<!-- Written to docs/delivery/RUNBOOK.md by /og:deliver. Content in the project language; headings fixed.
     The machine-readable settings are docs/delivery/delivery.json (the snapshot; the loop works from its
     live copy, `og-delivery config`) — this file says the same in words,
     for the owner and for any later session. Change settings with /og:deliver, not by hand. -->

# Delivery runbook — calendar-buddy

> The autonomous delivery loop for this repo. It runs one user story at a time — implementation,
> tests, review, merge — started with `og-loop up` (*How the loop runs and how to start it*).
> The procedure is the og plugin's (`/og:deliver-next`); this file holds this project's settings
> and the owner's standing authorisations.
>
> Armed: 2026-10-27 by Maciej Sułek (Slack: Maciej Sułek). The live state — settings, event log,
> implementer results — is in the clone's git directory (`og-delivery where`), outside every
> branch; `og-delivery status` shows it, `og-delivery report` builds the report. When the loop
> ends, the log and the report are committed here as `log.jsonl` and `REPORT.md`.

## Input

optimal — every story is `ready`, every acceptance criterion has a test case, no open question; no warning.

## Order

| # | Story | Title | Depends on | Test cases |
| --- | --- | --- | --- | --- |
| 1 | `US-10-01` | As a User, I can turn on events from a description and point the app at my Ollama and model, so that the description box works on my computer | — | `TC-10-01`…`TC-10-07` (`TC-10-05` manual-only) |
| 2 | `US-10-02` | As a User, I can add an event by describing it in a sentence in Polish or English, so that it is in my calendar in seconds | `US-10-01` | `TC-10-08`…`TC-10-19` (`TC-10-19` manual-only) |
| 3 | `US-10-03` | As a User, I can name guests and ask for Meet in the description, so that a meeting with people is set up in one sentence | `US-10-02` | `TC-10-20`…`TC-10-25` |
| 4 | `US-10-04` | As a User, I can change an event by describing the change, so that moving a meeting is as quick as adding one | `US-10-02` | `TC-10-18`, `TC-10-19` (manual-only), `TC-10-26`…`TC-10-31` |
| 5 | `US-10-05` | As a User, I am asked for what my description left out and can name the calendar in it, so that a short sentence still gives the right event | `US-10-02` | `TC-10-32`…`TC-10-35` |

The next story is the first in this order that is neither done, failed, skipped nor waiting,
and whose dependencies are met. Stories waiting on the owner never stop the others.

## Settings

| Area | Setting |
| --- | --- |
| Base branch | `main`, local — every story is squashed into the local `main` once review and tests are green; no push, no PR, no CI. The owner checks the report's *Before you push* and pushes the batch; the final report goes straight to the local `main` |
| Stories the loop starts | only ready ones — `status: ready`, no open question, a test case for every criterion; the rest stay in the order, reported with what they lack, and start once the base says they are ready (`strict`) |
| Foundation | none — the project had its foundation (the gates `cargo fmt`, `cargo clippy`, `cargo test` exist and are green) |
| Pilot | none |
| Stop | when the scope is done — and after 3 failed stories |
| A failed story | one more attempt on the retry's model and effort, then skipped with its dependents |
| Decisions | every story is mode A: decided on the recommendation and recorded; critical ones flagged in the log and the thread (the run's default, semi-automatic, applies only to a story without a mode) |
| Code review | standard — code reviewer and spec-compliance reviewer, up to 3 rounds, blocking findings fixed, the rest listed |
| Merge | every story is mode A: merged once review is clean and gates green — except a story with manual-only criteria (`US-10-01`: `TC-10-05`; `US-10-02` and `US-10-04`: `TC-10-19`) or whose diff trips a signal (dependencies, …): those wait for the owner |
| Approving a waiting merge | the owner's word — in the thread, on the page or in the loop's session |
| Protected paths | only the built-in signals · size ceiling none |
| Dependents of an unmerged story | wait |
| Unit tests | as the constitution says — test-first for recurrence, time zones, synchronization and Google mapping; unit tests first elsewhere, without a TDD mandate |
| End-to-end | each story's test cases, as tests in `cargo test` against the fake Ollama and fake Google (`DEC-65`) · regression at the end |
| Questions for the owner | the session, the thread or the terminal |
| Test results a merge relies on | the agents' own runs (local target) |
| Models and effort | Balanced — orchestrator `claude-opus-5-5` · medium · implementer `claude-opus-5-5` · medium · its retry `claude-opus-5-5` · high · code review `claude-sonnet-5-5` · high · spec review `claude-opus-5-5` · high · commit messages and summaries `claude-haiku-5-5`. Provisional: the profiles rest on vendor data as of 2026-10-09, not yet on this team's measurements; the report names the model that really answered for each role |
| Weekly limit | pause at 90% |
| 5-hour limit | ignored |
| Extra credits | not used |
| After a limit pause | resumes by itself |
| When usage cannot be read | carries on, says once that the limit cannot be enforced; the account's hard limit still stops it |
| Environment files | `.env.example` (key names only); no other `.env*` |
| Variables the loop needs | none |
| Reports | Slack thread https://ordergroup.slack.com/archives/DNYFHBZDK/p1793119659108229 — milestones; the thread also carries the owner's answers and every command |
| Backup of the run's state | after every iteration to the branch `og-delivery/state` on `origin` (run `20261027T164845Z`) — a commit on top of the last, never the base, never a PR; a new clone starts from it, and `og-delivery status` says when the last one was |
| Where the loop runs | on this machine, in the owner's checkout — `og-loop up` starts the driver detached (`/loop /og:deliver-next` the fallback); nothing isolates it: its sessions are the owner's account in auto mode |

## Critical decisions

What counts as critical (they are flagged — every story is mode A): deviations from a `must`
requirement or an NFR, scope changes, new dependencies or paid services, irreversible data or
schema changes, authentication / permissions / secrets / personal data, contracts other modules
rely on, refactors beyond the story.

## Authorisations

The owner's standing consent, in plain words — what the loop may do without asking:

- squash each story into the local `main` when review is clean, the gates are green and every criterion has automatic evidence — stories with a manual-only test case or a dependency change wait for the owner's word;
- decide implementation questions itself, critical ones included, record each as a DEC entry and flag the critical ones in the thread;
- push its state backup to the branch `og-delivery/state` on `origin` after every iteration;
- commit the `chore(delivery)` log and report to the local `main` when the loop ends;
- run `cargo` commands (allowlisted in `.claude/settings.json`).

## How the loop runs and how to start it

The loop has three layers. **The driver** (`og-deliver-run`) starts a fresh Claude Code session for
every iteration (`/og:deliver-next once`), so the context does not grow from story to story, and a
session past its clock, its turns or its budget is stopped by the driver. **The state** — settings,
event log, implementer results, the DEC register — is in the clone's git directory (`og-delivery
where`), outside every branch: a new session loses nothing. **The owner's word** — from the page
(`og-loop view`), the Slack thread or a terminal — is written into that log, and a driver
that waits picks it up within about half a minute.

- **On this machine.** `og-loop up` in this checkout starts the driver detached — it goes on after
  the terminal and the Claude Code session that started it have ended. Nothing isolates it: its
  sessions run as the owner, with `--permission-mode auto` — their SSH keys, `~/.config`, other
  repositories. The owner chose this on 2026-10-27 by choosing the local `main` as the target
  ("Lokalny main"), which runs on the host only; the shell that starts it holds nothing for production.
- **`/loop /og:deliver-next` in a session — the fallback.** One long session you can watch and
  talk to: `claude --permission-mode auto --model claude-opus-5-5 --effort medium`,
  then `/loop /og:deliver-next`. Its context grows with every story; between stories it waits for
  the owner's word in the background (`og-delivery await-word`) and wakes within half a minute of
  it.

Only one driver or session drives a run at a time: `og-loop up` refuses while a `/loop` session
waits on it, and a `/loop` session ends its loop while a driver runs (`og-delivery driver` says
which holds the run).

**Start and watch:** `og-loop up --max-iterations 1` — one trial iteration; `og-loop up
[--budget-usd N]` — the run. `og-loop status` — who drives it and where it stands; `og-loop logs
-f` — a line per iteration; `og-loop view` (or `/og:loop view` in a session) — the page. **Pause,
continue, stop:** on the page, in the thread, or in the loop's session;
`og-loop stop` ends the driver as Ctrl-C would — the session in progress stops, the state stays,
and `og-loop up` carries on. **After a reboot** — of the machine or Claude Code — nothing
comes back by itself: `og-loop up` again (or `/loop /og:deliver-next` in a new session); the loop
carries on from its state.

## Steering

Use the page (`og-loop view`), write in the loop's session — or in the Slack thread, which takes every command:
`pause` · `continue` · `stop` · `status` ·
`skip US-xx-yy` · `retry US-xx-yy` (`… fresh` for a new attempt on a new branch) ·
`DEC-nn: <answer>` · `merge US-xx-yy` · `fix US-xx-yy: <what>` · `take over` / `fresh` / `skip US-xx-yy` (when
the loop finds someone else's work on a story). Only the owner's messages are commands.

People may keep working alongside the loop: a story set to `in-progress` on the base is left to
them (its dependents wait for their merge), and a story with someone's branch or tagged
commits is never started without asking. More stories join with `/og:deliver add MOD-nn`.

Nobody can answer a session the driver started: the page and `og-delivery status` show what
waits; a decision is answered on the page, or from a terminal in the clone with
`og-delivery answer <DEC-nn> --option <a|b|…> --by <owner> --via terminal` — or, in words,
`echo "…" | og-delivery answer <DEC-nn> --by <owner> --via terminal` — and the loop takes it up
within half a minute (a driver that has ended takes it up when it is started again).

`pause` stops new work, but the loop keeps listening — a `continue` from the page, the thread, the
loop's session or a terminal (`og-delivery continue --by <owner> --via terminal`) brings it back
within half a minute. `stop` ends it: the driver ends, a `/loop` session ends its wake-ups. To carry
on: *Start* on the page — or `og-delivery set status=active`, then `og-loop up`.

## Preflight — 2026-10-27

```
✓ tree clean; local base main exists; live state writable (.git/og-delivery); one worktree, no stashes, no other local branches
✓ gates green on main: cargo fmt --check, cargo clippy -D warnings, cargo test (718 passed)
✓ assess: 5 stories runnable under strict readiness, none blocked, no foreign work, no contracts; DECISIONS.md present
✓ Slack thread readable ("Pętla test calendar", DM DNYFHBZDK)
✓ usage readable: 5-hour 7 %, weekly 3 %
✓ Claude Code 2.1.311; CLAUDE_CODE_EFFORT_LEVEL unset
✓ the starting shell holds nothing for production (owner's answer)
✓ backup branch og-delivery/state not on origin yet — this run starts it
⚠ no .claude/og-test-targets.json — by design (DEC-65): tests are cargo test against fakes
⚠ the app start was not checked — a desktop GUI; the preview pictures and cargo test stand in for it
⚠ gh is not installed — not needed for a local target; the backup goes by git push
⚠ manual-only test cases TC-10-05 (US-10-01) and TC-10-19 (US-10-02, US-10-04): those merges wait for the owner, and their dependents with them
⚠ auto mode: the loop needs a new session with --permission-mode auto — the driver starts it; the repo default is plan mode
```

## What the loop never does

- Changes the specification beyond story statuses, `BACKLOG.md` rows and DEC entries.
- Edits a test case, relaxes an assertion or adds a retry to make a test pass.
- Touches environment files beyond the ones named above, prints a credential or asks for one.
- Pushes anything to `origin` — the owner pushes the batch; only the backup of its state goes to a branch of its own.
- Mentions AI in commits.
- Keeps state anywhere but this clone (the live state directory and the repo).
