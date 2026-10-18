#!/usr/bin/env bash
# start_loop.sh — start the openbf loop in the background with one command. Stop it with `touch STOP` (repo root).
#
#   scripts/start_loop.sh            # 2 lanes, runs until STOP
#   scripts/start_loop.sh 1          # 1 lane
#   scripts/start_loop.sh pilot      # 1 lane, 3 sessions one after another, then it stops by itself
#   scripts/start_loop.sh status     # is it running, what is each lane doing
#
# It refuses if a loop is already running (a second copy would run beside it), clears old STOP/PAUSE files, checks the
# subscription answers (the probe), then starts scripts/autoloop.sh with nohup so closing the terminal does not stop it.
set -u
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
LOG="$ROOT/engine/artifacts/loop/autoloop.log"

running() { pgrep -f "^bash scripts/autoloop.sh" >/dev/null || pgrep -f "^bash $ROOT/scripts/autoloop.sh" >/dev/null; }

if [ "${1:-}" = "status" ]; then
  if running; then echo "loop: RUNNING"; else echo "loop: not running"; fi
  [ -f STOP ] && echo "STOP file present — lanes exit after their current session"
  [ -f PAUSE ] && echo "PAUSE file present"
  [ -f engine/artifacts/loop/DRAINED ] && echo "DRAINED (the loop stopped itself): $(cat engine/artifacts/loop/DRAINED)"
  python3 scripts/loop_cpu.py status
  echo "claims: $(ls engine/artifacts/loop/locks 2>/dev/null | tr '\n' ' ')"
  grep -E '^## autoloop lane|^- ended' engine/artifacts/loop/JOURNAL.md | tail -4 | cut -c1-200
  exit 0
fi

case "${1:-2}" in
  pilot) export LANES=1 MAX_ITERS=3; what="PILOT: 1 lane, 3 sessions one after another, then it stops" ;;
  1)     export LANES=1;              what="1 lane, until STOP" ;;
  2)     export LANES=2;              what="2 lanes, until STOP" ;;
  *)     echo "usage: scripts/start_loop.sh [1|2|pilot|status]"; exit 1 ;;
esac

if running; then
  echo "A loop is already running (scripts/start_loop.sh status). Stop it first: touch STOP, and wait for it to exit."
  exit 1
fi

# Launched from inside a Claude session, inherited CLAUDE* variables confuse the child sessions — drop them.
for v in $(env | grep -o '^CLAUDE[A-Z_]*'); do unset "$v"; done

rm -f STOP PAUSE engine/artifacts/loop/DRAINED

probe_out="$(LANES=1 MAX_ITERS=0 scripts/autoloop.sh --probe 2>&1)"   # LANES=1: with LANES=2 the launcher would run the probe as two lanes
echo "probe: $(printf '%s' "$probe_out" | tail -1)"
if ! printf '%s' "$probe_out" | grep -q "probe OK"; then
  echo "the subscription did not answer (limit hit or not logged in) — nothing started; try again later"; exit 1
fi

# setsid: its own session, so a Ctrl-C or a closed terminal can never reach it; nohup: ignore hangups too. Stop = touch STOP.
# Its own systemd slice (2026-10-11): openbf.slice has a low CPU / IO weight, so the maintainer's own programs win a contended core,
# and scripts/loop_cpu.py (started beside the loop, in the same scope) keeps the loop on the efficiency cores while he is
# at the machine and gives it every core when the lid is closed, the screen is locked or he has been idle 5 minutes.
# OPENBF_CPU_WEIGHT (default 10; an ordinary program is 100), OPENBF_POLITE=0 (no core limit), OPENBF_IDLE_SECS (300).
W="${OPENBF_CPU_WEIGHT:-10}"
if /usr/bin/systemd-run --user --scope --quiet --collect --slice=openbf.slice true 2>/dev/null; then
  systemctl --user set-property --runtime openbf.slice CPUWeight="$W" IOWeight="$W" 2>/dev/null
  setsid nohup /usr/bin/systemd-run --user --scope --quiet --collect --slice=openbf.slice --unit="openbf-loop-$(date +%s)" \
    bash -c 'python3 scripts/loop_cpu.py watch >> engine/artifacts/loop/cpu_governor.log 2>&1 & exec scripts/autoloop.sh' \
    > "$LOG" 2>&1 < /dev/null &
  how="in openbf.slice (CPU weight $W; scripts/loop_cpu.py status)"
else
  setsid nohup scripts/autoloop.sh > "$LOG" 2>&1 < /dev/null &
  how="NOT in a systemd slice (systemd-run --user failed): it competes with your programs as before"
fi
sleep 2
echo "started: $what (pid $!)"
echo "cpu:     $how"
echo "watch:   tail -f $LOG        status: scripts/start_loop.sh status"
echo "stop:    touch $ROOT/STOP    (each lane finishes its current session, then exits)"
