# Delivery report — calendar-buddy

> Generated 2026-10-28 04:17 UTC by `og-delivery report`.
> Tokens are attributed to a story phase by time, from the Claude Code transcripts (subagents included). Cost is the API list-price equivalent; a subscription is billed differently.

## Summary

- Stories: 5 (done 5, failed 0, skipped 0, waiting on the owner 0, not started 0)
- Time: 11:20 h, of which waiting 5:39 h
- Claude Code sessions in the loop: 7; waiting windows: 1
- Tokens (loop): input 0.00M, cache write 5.04M, cache read 172.97M, output 1.08M
- Cost (API equivalent): loop $82.40; before the loop $5.87
- Output tokens of 5 session(s) are Claude Code's own count per model, spread over their responses by how much each wrote: the transcripts under-record output (their cache counters are exact)
- ⚠ 2 session(s) have no figure from Claude Code (run with `/loop`, not by the driver): their output tokens come from the transcripts, which under-record them — their cost here is a lower bound
- Claude Code's own figure for the 6 sessions the driver ran: $69.86 — 18% off the count above: check this script's price table against the current list prices
- Other Claude Code sessions in this repository during the run (not counted): $0.71
- Prices: API list prices per million tokens as of 2026-10-09; cache writes at 1.25× (5 min) and 2× (1 h) of input.
- Actions recorded by the hooks: 1355 (shell 1281, files written 19, agents started 36) — `ledger.jsonl` in the live state

## Open items

- US-10-01 — manual acceptance: TC-10-05
- US-10-01 — merged into the local main on the owner's word (dependency change; test run from before the collector fix)
- US-10-02 — manual acceptance: TC-10-19
- US-10-02 — merged into local main as ecdb3ac; TC-10-19 to check before push
- US-10-03 — squashed into local main 6cf68d2; review 3 rounds (B1 refused Meet turned on by a later guest, fixed); DEC-223..227
- US-10-04 — manual acceptance: TC-10-19
- US-10-04 — merged into local main 306fc08; before you push: TC-10-19 by hand against a real Ollama
- US-10-05 — squashed into local main (f973b37)
- US-10-01 · DEC-215 — a critical decision the loop made by itself (Steps of TC-10-04 and TC-10-07 that send a description wait for US-10-02): confirm it
- US-10-01 · DEC-216 — a critical decision the loop made by itself (The Ollama client in calendar-core uses ureq and url, already in the workspace): confirm it
- US-10-02 · DEC-222 — a critical decision the loop made by itself (TC-10-15: the saved event is also in the cache the views read): confirm it
- the run's settings were changed while it was live, at 2026-10-27T19:26:55Z (session 25d2fcbd): `autonomy.a_waits` null → false; `autonomy.b_c_merge` null → "owner" — if this was not you, treat everything merged after it as unreviewed

## Before you push

The run merged into the local `main` and pushed nothing. Check these, then push it:

- `US-10-01` — criteria only a human or an unavailable environment can evidence: #5 README tutorial on Ollama and gemma4:e4b — MANUAL: TC-10-05 is manual-only (README.md section "Events from a description (optional)"); #4 Default address: no request beyond Google and this computer — BLOCKED: every step of TC-10-04 sends a description and opens "New event", which is US-10-02 (DEC-215). This story's own traffic is proven by ollama::tests::the_check_ignores_a_proxy_set_in_the_environment (with HTTP_PROXY, HTTPS_PROXY and ALL_PROXY set and NO_PROXY removed, the proxy receives nothing and Ollama gets only GET /api/version and GET /api/tags; fails without the fix) and description::tests::ollama_is_asked_only_for_its_version_and_models_at_the_address_set; #8 Ollama starts later: box available without a restart — BLOCKED: TC-10-07 steps 1-3 pass (Ollama stops: greyed box, text kept; Ollama starts: a click makes the box available, no restart); step 4 needs US-10-02 (DEC-215)
- `US-10-01` — acceptance tests: manual-only to run by hand: TC-10-05
- `US-10-01` — manual acceptance tests — a human runs them: TC-10-05
- `US-10-02` — criteria only a human or an unavailable environment can evidence: #8 Test set thresholds, fake model and real Ollama — TC-10-18 pass (fake); MANUAL: TC-10-19 is manual-only (real Ollama with gemma4:e4b)
- `US-10-02` — acceptance tests: manual-only to run by hand: TC-10-19
- `US-10-02` — manual acceptance tests — a human runs them: TC-10-19
- `US-10-03` — acceptance tests: manual-only to run by hand: none
- `US-10-04` — criteria only a human or an unavailable environment can evidence: #4 test set gains at least 10 change sentences in pl and en and passes 95 % / 80 % — TC-10-18 tc_10_18_the_test_set_passes_its_thresholds_against_the_fake_model, steps 1-2 pass (44 sentences, recorded answers now include the live "same" adds); MANUAL: TC-10-19 is manual-only (run here by hand against gemma4:e4b on b58f11f: date 98 %, every field 93 %)
- `US-10-04` — acceptance tests: manual-only to run by hand: TC-10-19
- `US-10-04` — manual acceptance tests — a human runs them: TC-10-19

## Stories

| Story | Status | PR | Dev | Review (rounds) | Tests | Merge | Waiting | Total | Output tok. | Cost | Sessions | Tests p/f/b/m | Merged by | Model · effort |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `US-10-01` | done | — | 0:17 | 0:11 (2) | 0:00 | 0:00 | 0:20 | 0:50 | 0.17M | $11.57 | 2 | 4/0/2/1 | the loop | claude-opus-5-5 · medium |
| `US-10-02` | done | — | 0:18 | 0:15 (2) | 0:00 | 0:00 | 0:00 | 0:34 | 0.22M | $13.12 | 1 | 17/0/0/1 | the loop | claude-opus-5-5 · medium |
| `US-10-03` | done | — | 0:16 | 0:36 (3) | 0:00 | 0:00 | 5:39 | 6:34 | 0.22M | $20.18 | 1 | 6/0/0/0 | the loop | claude-opus-5-5 · medium |
| `US-10-04` | done | — | 0:19 | 0:35 (2) | 0:00 | 0:00 | 0:00 | 0:55 | 0.21M | $12.83 | 1 | 7/0/0/1 | the loop | claude-opus-5-5 · medium |
| `US-10-05` | done | local | 0:17 | 0:20 (3) | 0:00 | 0:00 | 0:00 | 0:38 | 0.20M | $11.76 | 1 | 4/0/0/0 | the loop | claude-opus-5-5 · medium |

## Cost by model

| Model | Responses | Input | Cache write | Cache read | Output | Cost |
| --- | --- | --- | --- | --- | --- | --- |
| `claude-opus-5-5` | 1306 | 0.00M | 4.41M | 167.47M | 0.95M | $78.97 |
| `claude-sonnet-5-5` | 175 | 0.00M | 0.63M | 5.50M | 0.13M | $3.43 |

## Cost by agent

| Agent | Responses | Output | Cost | Share | Answered by |
| --- | --- | --- | --- | --- | --- |
| og:story-implementer | 689 | 0.49M | $38.04 | 46% | `claude-opus-5-5` |
| orchestrator | 295 | 0.19M | $26.68 | 32% | `claude-opus-5-5` |
| og:spec-compliance-reviewer | 322 | 0.27M | $14.24 | 17% | `claude-opus-5-5` |
| og:code-reviewer | 175 | 0.13M | $3.43 | 4% | `claude-sonnet-5-5` |

## Merges

| Story | PR | Merged by | Why it waited for a person |
| --- | --- | --- | --- |
| `US-10-01` | — | the loop | dependency manifests changed (Cargo.lock, crates/app/Cargo.toml, crates/core/Cargo.toml — ureq and url added to calendar-core, no new crate in the lock); the test run does not record which build it tested (unknown version, not 36cafb1); to check: manual TC-10-05 (README tutorial), #4 and #8 blocked until US-10-02 (DEC-215) |
| `US-10-02` | — | the loop | — |
| `US-10-03` | — | the loop | — |
| `US-10-04` | — | the loop | — |
| `US-10-05` | local | the loop | — |

## Review rounds

| Story | Round | Reviewers | Blocking | Non-blocking | Security |
| --- | --- | --- | --- | --- | --- |
| `US-10-01` | 1 | code, spec | 2 | 18 | 2 |
| `US-10-01` | 2 | code, spec | 0 | 7 | 0 |
| `US-10-02` | 1 | code, spec | 3 | 18 | 0 |
| `US-10-02` | 2 | code, spec | 0 | 10 | 0 |
| `US-10-03` | 1 | code, spec | 1 | 15 | 0 |
| `US-10-03` | 2 | code, spec | 2 | 7 | 0 |
| `US-10-03` | 3 | code, spec | 0 | 3 | 0 |
| `US-10-04` | 1 | code, spec | 2 | 16 | 0 |
| `US-10-04` | 2 | code, spec | 0 | 8 | 0 |
| `US-10-05` | 1 | code, spec | 1 | 13 | 0 |
| `US-10-05` | 2 | code, spec | 2 | 10 | 0 |
| `US-10-05` | 3 | code, spec | 0 | 7 | 0 |

## Decisions

| DEC | Story | Class | What | The owner's answer |
| --- | --- | --- | --- | --- |
| `DEC-215` | `US-10-01` | auto ⚠ | Steps of TC-10-04 and TC-10-07 that send a description wait for US-10-02 | — |
| `DEC-216` | `US-10-01` | auto ⚠ | The Ollama client in calendar-core uses ureq and url, already in the workspace | — |
| `DEC-217` | `US-10-01` | auto | When Ollama is checked, and the fake Ollama on a free local port | — |
| `DEC-218` | `US-10-02` | auto | The model's answer says whether it is an event, and leaves the length empty when none is given | — |
| `DEC-219` | `US-10-02` | auto | Until US-10-03 and US-10-05, guests, Meet and a missing time are read but not acted on | — |
| `DEC-220` | `US-10-02` | auto | While the model reads, the keyboard goes back to the views | — |
| `DEC-221` | `US-10-02` | auto | The test set holds the 30 adding sentences | — |
| `DEC-222` | `US-10-02` | auto ⚠ | TC-10-15: the saved event is also in the cache the views read | — |
| `DEC-223` | `US-10-03` | auto | The model's answer says Meet asked for, refused or not mentioned | — |
| `DEC-224` | `US-10-03` | auto | Guests named in a description are looked up before the full editor opens | — |
| `DEC-225` | `US-10-03` | auto | A name not certain waits under the guest field with its suggestions | — |
| `DEC-226` | `US-10-03` | auto | A calendar that cannot add Meet behaves as in the editor today | — |
| `DEC-227` | `US-10-03` | auto | The description's Meet wins over the first-guest rule | — |
| `DEC-228` | `US-10-04` | auto | A described change: the words naming the event, and only what changes | — |
| `DEC-229` | `US-10-04` | auto | An event fits the words naming it word by word, Polish endings aside | — |
| `DEC-230` | `US-10-04` | auto | A change of an event the editor is not offered for opens its details, as today | — |
| `DEC-231` | `US-10-04` | auto | The model's answer is taken in while an event's details are open | — |
| `DEC-232` | `US-10-05` | auto | A day is asked for only when the description names neither a day nor a part of the day | — |
| `DEC-233` | `US-10-05` | auto | A named calendar is read by the model and matched among the calendars that can take the event | — |
| `DEC-234` | `US-10-05` | auto | The "New event" window asks for a missing day or time in fields of its own | — |

## Waiting

- 2026-10-27 20:08 → 2026-10-28 01:48 UTC (5:39 h)
