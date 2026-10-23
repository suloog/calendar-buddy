# Delivery loop: one story, alone

You are running unattended in a fresh session, with no human to ask. You deliver exactly one user
story, named in the prompt, from the specification in `docs/specs/` to a commit on `main`.
Everything below is your brief. Read only what it tells you to read; the context is yours to
spend carefully.

## 1. Read
- The story file (`docs/specs/modules/*/stories/<ID>-*.md`), including its `covers:` and `depends_on:`.
- In its module's `REQUIREMENTS.md`, only these sections:
  - the requirements named in `covers:`;
  - *Constraints and edge cases*;
  - *Technical notes*;
  - *Out of scope*.
- `docs/specs/constitution.md`, and `docs/specs/NFR.md` where it applies.
- Every test case in the module's `tests/` whose `covers:` names this story.
- The `DEC-nn` entries the story or the requirements cite (`grep` them in `docs/specs/DECISIONS.md`; do not read the whole file).
- `.claude/CLAUDE.md`, plus the code the story touches.
- The rules for implementing a story: `/home/sulo/.claude-og/plugins/cache/order-group/og/0.16.0/skills/start-story/references/implementation.md`. Of it, follow sections §1, §3, §4 and §5. §2 is replaced by the stop conditions below.

## 2. Stop conditions
Stop at once with `RESULT: failed <ID> <reason>` and change nothing if:
- the story's status is not `ready`, unless its branch `story/<ID>` exists (a resumed story is `in-progress` there);
- a `depends_on` story is not `review` or `done`. Both mean its code is on `main`;
- the story names an open question.

## 3. Work
1. If the branch `story/<ID>` already exists, an earlier session was interrupted, so **resume** it. Run `git switch story/<ID>` (uncommitted changes come along), then take stock with `git log main..HEAD`, `git status` and the story's criteria. Continue from where it stopped, and don't redo finished work. Otherwise run `git switch main && git switch -c story/<ID>`. Check `git branch --show-current` before every phase.
2. Load the `og:principles` skill and follow it.
3. Set the story to `in-progress`, both in its frontmatter and in its `docs/specs/BACKLOG.md` row.
4. Implement. Keep to the constitution, and write tests first where it says so (recurrence, time zones, sync, Google mapping). The `auto` test cases become automated tests in `cargo test`, and each test's name carries the test case's ID (`tc_02_04_…`).
5. **Decide alone.** Where the spec does not settle something, choose the simplest option consistent with the spec, the constitution and the code. Prefer what Google Calendar does (`DEC-62`).
   - Record every such choice in `docs/specs/DECISIONS.md` as the next `DEC-nn`, taking the highest number in the file plus one, in the file's format. Use:
     - Decided by: `delivery loop (Claude)`
     - Source: `implementation <ID>`
     - Status: `accepted`
   - **Park instead of deciding** when a choice would weaken a `must` requirement or an `NFR-nn`, or when it needs the owner's accounts or secrets. In that case:
     - write the question to `docs/delivery/PARKED.md`;
     - commit what you have on the story branch;
     - switch back to `main`, leaving the branch;
     - end with `RESULT: parked <ID> <question>`.
6. **Gates.** Use the `og:pre-push` skill: `cargo fmt`, `clippy -D warnings`, `cargo test --workspace`. If the same gate fails after three fix attempts, end with `RESULT: failed`.
7. **Code review.** Run the `og:code-reviewer` agent (model opus) on `git diff main...story/<ID>`. Fix every BLOCKING finding. Allow at most 2 rounds.
8. **Spec review.** Run the `og:spec-compliance-reviewer` agent (model opus) against the story and the diff. Fix whatever it reports as unmet. Allow at most 2 rounds.
9. **Evidence.** Map every acceptance criterion to the test that proves it. For a criterion whose test case is `manual-only`, or that needs real Google, a real KDE session or a person's eyes, append it to `docs/delivery/MANUAL-CHECKS.md` under the story ID, with the test case ID and what to check. Never fake such a check.
10. **Status.** Set the story to `review` if anything went to `MANUAL-CHECKS.md`, otherwise to `done`. Mirror the change into `BACKLOG.md`.

## 4. Commit and merge (no push, no PR)
- Use the `og:commit` rules for every commit:
  - `type(scope): subject`, at most 72 characters;
  - no body, no mention of AI, no Co-Authored-By;
  - the hook runs the gates.
- On `main`, the story becomes **one commit** (`git merge --squash story/<ID>`), or a few commits if it holds clearly separate logical changes. Your call: record it in the log.
  - A merge whose code does not pass `cargo test --workspace` on `main` does not stay: undo it and end with `RESULT: failed`.
- After a successful merge, delete the branch (`git branch -D story/<ID>`).
- **Never** `git push`, never `gh`, never touch `.env*`.

## 5. Log
Append to `docs/delivery/LOG.md` (create it with a heading if missing):

```
## <ID> — <done|review|parked|failed> — <YYYY-MM-DD HH:MM from `date '+%Y-%m-%d %H:%M'`>
- What: <one or two lines>
- Decisions: <DEC-nn, … or none>
- Reviews: code <n> round(s), spec <n> round(s); open notes: <…>
- Manual checks: <TC ids or none>
- Commits on main: <short hashes and subjects>
```

Include the log line in the merge commit, or commit it right after as `docs(delivery): log <ID>`.

## 6. End
The very last line of your answer is exactly one of:

```
RESULT: done <ID>
RESULT: review <ID>
RESULT: parked <ID> <short question>
RESULT: failed <ID> <short reason>
```
