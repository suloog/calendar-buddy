#!/usr/bin/env bash
# Thin delivery loop: one fresh, unattended Claude session per story.
#
#   scripts/loop/deliver.sh US-03-02 US-03-03 US-03-04
#
# Each story runs in its own `claude -p` session with an empty context (brief: BRIEF.md beside this
# script). The session implements the story on a local branch, runs the gates and the og review
# agents, and merges it into main locally — nothing is pushed. Per story this script records the
# wall time, turns, tokens and the change in the account's 5-hour and 7-day usage limits in
# docs/delivery/USAGE.md. It stops before a story when the limits are too high.
#
# Settings (environment, all optional):
#   LOOP_MODEL=opus        model for the session (subagents are asked for opus in the brief)
#   LOOP_FIVE_HOUR_MAX=90  pause until the 5-hour window resets at or above this %
#   LOOP_SEVEN_DAY_MAX=85  stop the run at or above this %
#   LOOP_STORY_TIMEOUT=3h  wall-clock limit per story
set -uo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
HERE="$ROOT/scripts/loop"
OG_USAGE="${OG_USAGE:-$HOME/.claude-og/plugins/cache/order-group/og/0.16.0/bin/og-usage}"
MODEL="${LOOP_MODEL:-opus}"
FIVE_MAX="${LOOP_FIVE_HOUR_MAX:-90}"
SEVEN_MAX="${LOOP_SEVEN_DAY_MAX:-85}"
TIMEOUT="${LOOP_STORY_TIMEOUT:-3h}"
RUNS="$ROOT/.git/og-loop"          # raw session output, outside the repository's history
USAGE_MD="$ROOT/docs/delivery/USAGE.md"
mkdir -p "$RUNS" "$ROOT/docs/delivery"

usage_json() { OG_USAGE_TTL=0 "$OG_USAGE" 2>/dev/null || echo '{}'; }
pct() { jq -r ".${1}.utilization // \"?\"" <<<"$2"; }
resets() { jq -r ".${1}.resets_at // \"\"" <<<"$2"; }

if [ ! -f "$USAGE_MD" ]; then
  cat > "$USAGE_MD" <<'EOF'
# Delivery usage

One row per story run by `scripts/loop/deliver.sh`. Limit columns are the account's usage
(percent of the 5-hour and 7-day windows) before → after; Δ is the story's share, and it also
includes anything else used on the account meanwhile. `~` marks a window that reset during the
story, so its Δ is not meaningful. Cost is Claude Code's own figure (notional on a subscription).

| Story | Result | Started | Wall time | Turns | Tokens in / out (cache read) | Cost (USD) | 5h before → after (Δ) | 7d before → after (Δ) |
| ----- | ------ | ------- | --------- | ----- | ---------------------------- | ---------- | --------------------- | --------------------- |
EOF
fi

for STORY in "$@"; do
  # --- limits ---------------------------------------------------------------------------------
  while :; do
    U="$(usage_json)"
    F="$(pct five_hour "$U")"; S="$(pct seven_day "$U")"
    if [ "$S" != "?" ] && [ "${S%.*}" -ge "$SEVEN_MAX" ]; then
      echo "7-day usage ${S}% ≥ ${SEVEN_MAX}% — stopping before $STORY."; exit 0
    fi
    if [ "$F" != "?" ] && [ "${F%.*}" -ge "$FIVE_MAX" ]; then
      R="$(resets five_hour "$U")"
      echo "5-hour usage ${F}% ≥ ${FIVE_MAX}% — waiting for the reset at $R."
      sleep $(( $(date -d "$R" +%s) - $(date +%s) + 60 ))
      continue
    fi
    break
  done

  # --- one fresh session --------------------------------------------------------------------
  BEFORE="$U"; START="$(date +%s)"; STARTED="$(date '+%Y-%m-%d %H:%M')"
  OUT="$RUNS/$STORY-$(date +%Y%m%dT%H%M%S).json"
  echo "▶ $STORY ($STARTED)"
  timeout "$TIMEOUT" claude -p "Deliver user story $STORY. Follow the brief you were given exactly." \
    --model "$MODEL" \
    --permission-mode auto \
    --append-system-prompt "$(cat "$HERE/BRIEF.md")" \
    --output-format json > "$OUT" 2> "$OUT.err"
  CODE=$?
  END="$(date +%s)"; AFTER="$(usage_json)"

  # --- what happened ------------------------------------------------------------------------
  RESULT_LINE="$(jq -r '.result // ""' "$OUT" 2>/dev/null | grep -E '^RESULT:' | tail -1)"
  [ -z "$RESULT_LINE" ] && RESULT_LINE="RESULT: failed $STORY session ended without a result (exit $CODE)"
  RESULT="$(awk '{print $2}' <<<"$RESULT_LINE")"
  TURNS="$(jq -r '.num_turns // "?"' "$OUT" 2>/dev/null)"
  COST="$(jq -r '.total_cost_usd // "?" | if type=="number" then (.*100|round/100) else . end' "$OUT" 2>/dev/null)"
  TOK="$(jq -r '.usage | "\(.input_tokens + (.cache_creation_input_tokens // 0)) / \(.output_tokens) (\(.cache_read_input_tokens // 0))"' "$OUT" 2>/dev/null || echo '?')"
  MIN=$(( (END - START + 59) / 60 ))
  delta() {  # window name
    local b a rb ra
    b="$(pct "$1" "$BEFORE")"; a="$(pct "$1" "$AFTER")"
    rb="$(resets "$1" "$BEFORE")"; ra="$(resets "$1" "$AFTER")"
    if [ "$b" = "?" ] || [ "$a" = "?" ]; then echo "?"; return; fi
    if [ "$rb" != "$ra" ]; then echo "${b}% → ${a}% (~)"; return; fi
    echo "${b}% → ${a}% (+$(awk -v a="$a" -v b="$b" 'BEGIN{printf "%.0f", a-b}')pp)"
  }
  printf '| %s | %s | %s | %s min | %s | %s | %s | %s | %s |\n' \
    "$STORY" "$RESULT" "$STARTED" "$MIN" "$TURNS" "$TOK" "$COST" "$(delta five_hour)" "$(delta seven_day)" >> "$USAGE_MD"
  echo "  $RESULT_LINE — ${MIN} min, 5h $(delta five_hour), 7d $(delta seven_day)"

  git add "$USAGE_MD" && git commit -q -m "docs(delivery): usage of $STORY" -- "$USAGE_MD" || true

  case "$RESULT" in
    done|review|parked) ;;
    *) echo "Stopping: $STORY did not finish ($RESULT_LINE). Session output: $OUT"; exit 1 ;;
  esac
done
echo "All stories processed. Nothing was pushed — review docs/delivery/LOG.md and push main when happy."
