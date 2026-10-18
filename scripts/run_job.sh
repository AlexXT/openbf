#!/usr/bin/env bash
# run_job.sh — run ONE loop session by hand, outside autoloop.sh: no lane loop, no picker, no STOP handling.
#
#   scripts/run_job.sh audit [lane]            an AUDIT session (the briefing lists its targets by risk)
#   scripts/run_job.sh card <ID> [lane]        a session on one card, in the card's own phase (re / build / ...)
#   scripts/run_job.sh --dry audit|card <ID>   build the prompt and print what would run; claim nothing, start nothing
#
# It does what one iteration of autoloop.sh does: claims the job through scripts/loopctl.py (the same lock files, so a loop
# or another run cannot take it), builds the prompt (LOOP_PROMPT.md + the orientation + the workflow block for an audit or a
# research phase), runs `claude -p` with the loop's environment and effort, and runs the postmortem. Run it in the background
# (nohup) with a log: the claim is held by THIS script's pid, so it must stay alive for the whole session.
# Env as autoloop.sh: MODEL (opus), MAX_TURNS (400), ITER_TIMEOUT (4h), PERM_MODE (auto), WORKFLOWS (1), WF_EFFORT (medium).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DRY=0; [ "${1:-}" = "--dry" ] && { DRY=1; shift; }
kind="${1:-}"; cid=""
case "$kind" in
  audit) lane="${2:-1}" ;;
  card)  cid="${2:-}"; lane="${3:-2}"; [ -n "$cid" ] || { echo "usage: run_job.sh card <ID> [lane]"; exit 1; } ;;
  *) sed -n 2,8p "$0"; exit 1 ;;
esac

MODEL="${MODEL:-opus}"; MAX_TURNS="${MAX_TURNS:-400}"; ITER_TIMEOUT="${ITER_TIMEOUT:-4h}"; PERM_MODE="${PERM_MODE:-auto}"
WORKFLOWS="${WORKFLOWS:-1}"; WF_EFFORT="${WF_EFFORT:-medium}"
PROMPT_FILE="$ROOT/engine/artifacts/briefs/LOOP_PROMPT.md"
WF_BLOCK="$ROOT/engine/artifacts/briefs/WORKFLOW_BLOCK.md"
LOOP_DIR="$ROOT/engine/artifacts/loop"; RUNS_DIR="$LOOP_DIR/runs"; JOURNAL="$LOOP_DIR/JOURNAL.md"
mkdir -p "$RUNS_DIR"

# Launched from inside a Claude session, inherited CLAUDE* variables confuse the child session.
for v in $(env | grep -o '^CLAUDE[A-Z_]*'); do unset "$v"; done

if [ "$DRY" = 0 ] && { pgrep -f "^bash scripts/autoloop.sh" >/dev/null || pgrep -f "^bash $ROOT/scripts/autoloop.sh" >/dev/null; }; then
  echo "an autoloop is running - stop it first (touch STOP and wait)"; exit 1
fi

# The job, as loopctl's picker would have built it.
if [ "$kind" = audit ]; then
  claim="AUDIT"; phase=""
else
  claim="$cid"
  phase="$(python3 - "$cid" <<'PY'
import sys; sys.path.insert(0, "scripts")
import loopctl as L
c = L.cards().get(sys.argv[1])
print(c.get("phase", "") if c else "NOCARD")
PY
)"
  [ "$phase" = "NOCARD" ] && { echo "no card $cid"; exit 1; }
fi
effort="$(python3 - "$kind" "$phase" "$cid" <<'PY'
import sys; sys.path.insert(0, "scripts")
import loopctl as L
kind, phase, cid = sys.argv[1:4]
eff = L.EFFORT.get("audit" if kind == "audit" else phase, "medium")
if cid:
    c = L.cards().get(cid, {})
    if c.get("effort") in ("low", "medium", "high", "xhigh", "max"): eff = c["effort"]
print(eff)
PY
)"
job_json="$(python3 - "$kind" "$cid" "$phase" "$effort" "$claim" "$lane" <<'PY'
import json, sys
kind, cid, phase, effort, claim, lane = sys.argv[1:7]
print(json.dumps({"job": kind, "card": cid or None, "phase": phase or None, "effort": effort, "claim": claim,
                  "lane": int(lane), "reason": "run by hand (scripts/run_job.sh, the maintainer 2026-10-07: the workflow trial)"}))
PY
)"
export OPENBF_LOOP=1 OPENBF_FABLE_MAX=0 OPENBF_LANE="$lane" OPENBF_LANES=2 OPENBF_LANE_PID="$$" OPENBF_EFFORT="$effort" OPENBF_JOB_JSON="$job_json"

stamp="$(date +%Y%m%d_%H%M%S)"
out="$RUNS_DIR/run_${stamp}_manual_${claim}.json"
promptf="$RUNS_DIR/prompt_${stamp}_manual_${claim}.md"
{ cat "$PROMPT_FILE"; printf '\n\n---\n\n'; python3 "$ROOT/scripts/loopctl.py" orient; } > "$promptf"
wf=0
if [ "$WORKFLOWS" != "0" ] && [ -f "$WF_BLOCK" ] && { [ "$kind" = audit ] || { [ "$kind" = card ] && [ "$phase" = re ]; }; }; then
  wf=1
  { printf '\n\n---\n\n'; sed "s/@WF_EFFORT@/$WF_EFFORT/g" "$WF_BLOCK"; } >> "$promptf"
fi
echo "job: $job_json"
echo "effort=$effort wf=$wf  prompt: $promptf ($(wc -c < "$promptf") bytes)"
if [ "$DRY" = 1 ]; then echo "dry run: nothing claimed, nothing started"; exit 0; fi

# Claim it (the same lock directories and counters loopctl's picker uses), or stop.
python3 - "$claim" "$$" "$kind" <<'PY' || { echo "the claim $claim is held by a live session - not started"; exit 1; }
import sys; sys.path.insert(0, "scripts")
import loopctl as L
name, pid, kind = sys.argv[1], int(sys.argv[2]), sys.argv[3]
if not L.claim(name, pid): sys.exit(1)
st = L.state()
st["last_job"] = kind
st["sessions_since_planner"] = st.get("sessions_since_planner", 0) + 1
st["sessions_since_audit"] = 0 if kind == "audit" else st.get("sessions_since_audit", 0) + 1
L.save_state(st)
PY

printf '\n## manual run lane %s started %s (model %s, effort %s, wf=%s) job %s\n' "$lane" "$(date '+%F %T')" "$MODEL" "$effort" "$wf" "$job_json" >> "$JOURNAL"
started="$(date +%s)"
timeout "$ITER_TIMEOUT" claude -p --model "$MODEL" --max-turns "$MAX_TURNS" --permission-mode "$PERM_MODE" --effort "$effort" \
  --output-format json < "$promptf" > "$out" 2> "${out%.json}.stderr"
rc=$?
read -r turns cost <<<"$(python3 - "$out" <<'PY'
import json, sys
try: d = json.load(open(sys.argv[1])); print(d.get("num_turns", ""), d.get("total_cost_usd", ""))
except Exception: print(" ")
PY
)"
echo "done: rc=$rc turns=$turns cost=\$$cost  result: $out"
printf -- '- ended %s (manual run, lane %s): rc=%s turns=%s est_cost=$%s wf=%s result=%s\n' "$(date '+%F %T')" "$lane" "$rc" "$turns" "$cost" "$wf" "$out" >> "$JOURNAL"
python3 "$ROOT/scripts/loopctl.py" postmortem --since "$started" --pid "$$" --lane "$lane" 2>&1 | sed 's/^/postmortem: /'
exit 0
