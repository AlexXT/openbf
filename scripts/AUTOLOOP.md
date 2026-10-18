# The openbf autoloop (v2, 2026-09-25)

Unattended Opus sessions run the project. The maintainer's operator page is `engine/artifacts/loop/START_HERE.md` (start, stop, what to
read); this file is how the machinery works. The v1 design (one 28 KB prompt, QUEUE.md, HANDOFF-OPUS.md, a PLAN Log entry per
landing) is archived in `engine/artifacts/briefs/v1/` and `analysis/PLAN_LOG_2026.md`; why it changed:
`engine/artifacts/restructure/PROPOSAL.md`.

## The pieces
| Piece | Path | Role |
|---|---|---|
| Launcher | `scripts/autoloop.sh` | per lane: pick + claim a job (`loopctl.py next`), run one `claude -p` session on the standing prompt + that job's orientation, record, repeat; limit probes; STOP/PAUSE |
| Standing prompt | `engine/artifacts/briefs/LOOP_PROMPT.md` | what good work is: jobs, the RE discipline, records, rules. ~10 KB |
| Bookkeeping | `scripts/loopctl.py` | cards, claims, job selection, effort, orientation, BACKLOG.md, scoreboard |
| Gates | `scripts/gates.py` + `engine/artifacts/loop/gates.toml` | ci, GPU + goldens (quantified, sheeted), sim hashes, sweep, arenas, byte-identical bake, crate map |
| Landing | `scripts/land.py` | rebase → branch gates → declared-move check → ff-merge → main gates/rebake/goldens → (session views) → commit, re-baseline, clean up |
| RE checks | `scripts/re_check.py` | enclosing function, absence search, writers/defaults, callers, ELF words |
| PCSX2 | `engine/artifacts/pcsx2/` | the reference library and its tools (README first) |
| Oracle guard | `.claude/settings.json` → `scripts/oracle_guard.py hook` | blocks a loop builder's Edit/Write/Bash writes to the oracle (below) |
| Guard | `.claude/settings.json` → `scripts/claude_guard.sh` | denies push, `add -A`, non-ff merges, writes to iso/extracted, recursive deletes of data, bare GPU tests, /tmp targets |

## State (all on disk)
- `engine/artifacts/loop/cards/<ID>.md` — the backlog; header fields id, title, milestone, value 1-5, phase
  (re|build|measure|capture|cleanup), status (todo|in-progress|parked|blocked|done), deps, files, declares, branch, resume.
- `BACKLOG.md` (generated), `STATUS.md` (planner), `INBOX.md` (the maintainer), `JOURNAL.md` (append-only), `reports/`, `audits.md`,
  `state.json` (counters for planner / cleanse / audit), `locks/` (claims: atomic mkdir + the lane pid), `gates.toml`.
- PLAN.md = goals (§Milestones), the LOCKED FEATURE SET and the specs. `analysis/` = RE. git = history.

## Job selection (`loopctl.py next`, deterministic)
STOP/PAUSE → none; DRAINED → stop. Then: resume a card in progress with no live claim → land a parked card (MAIN free; not one with `needs:`) →
planner (8 landings, 12 sessions, < 5 open cards, or no STATUS) → cleanse (15 landings) → audit (10 sessions; while lane 3's
launcher lives, `lanes/lane3.pid` naming a live `autoloop.sh` process - a thread id or a reused pid is dead - only lane 3 takes it) → the highest-value eligible card (deps done, files disjoint from
in-flight cards; `acceptance: needed` never; `defer: post-alpha` only when nothing else is eligible; at equal value
`alpha: yes` first, then lane 1 prefers build/cleanup and lane 2 re/capture/measure; lane 3 takes its re/audit/capture/
measure cards at any value, any card only when none is eligible) → planner (refill; two empty refills in a row write
DRAINED). A card taken 3 sessions in a row without its file changing is auto-blocked. After every session the launcher runs
`loopctl.py postmortem` (a claim left or a card in progress with no report → a `DIED` journal line, the claim freed). Docs
reach main through `loopctl.py commit-docs`. Effort: re/capture/audit max, planner high, the rest medium (`EFFORT=<level>`
forces one).

## Lanes and safety
Claims are atomic; the launcher sweeps claims whose pid is dead and its own leftovers. Every ci/gpu/rebake/goldens/PCSX2 run
holds `flock engine/artifacts/loop/ci.lock`. Main changes only under the `MAIN` claim through land.py, fast-forward only.
Worktrees: `<repo parent>/openbf-wt/<branch>`, target dirs `engine/artifacts/lead_tmp/<branch>_target`.

## Hard stop and clean-up
```
pkill -f 'scripts/autoloop.sh'      # the launcher and its lanes
pgrep -af 'claude -p'               # a live session, if any — let it finish, or kill it (its card is resumed next start)
rm -f STOP PAUSE
python3 scripts/loopctl.py check    # cards valid
```
Never `rm -rf` a worktree: land it, or `git worktree remove` it after unlinking its five symlinks.

## Absence receipts and metrics (2026-10-08)
`re_check.py absent b2 TERM… [--card ID] [--src PATH] [--elf]` greps both decomp corpora (plus raw bytes of any `--src` file/dir,
e.g. `engine/assets_baked`, and with `--elf` both ELFs) and ALWAYS writes a receipt, `engine/artifacts/loop/receipts/<card>_<id>.json`
(terms, sources, files/bytes scanned, hits per source, timestamp, tool version), printing `[receipt:<id>]`. An RE doc must put that
tag on (or within 3 lines under) every absence claim ("no class authors", "absent", "never authored", "not present", "no callers"…).
`loopctl.py check-absence analysis/re-x.md [--warn] [--base REF]` fails (exit 1) on a claim with no tag or a tag naming no receipt;
`--warn` only reports. `commit-docs` runs it on every `analysis/*.md` it commits, `--base HEAD`, so only lines ADDED by that commit
are judged — the 88 existing docs are never failed retroactively. Receipts are plain untracked files; keep them. Fenced code
blocks (pasted tool output) are skipped. `loopctl.py metrics [--csv] [--out F]` prints a short summary (landings/day from the cards'
`landed` stamps, cards by milestone, follow-up ratio, audit verdicts, cost per lane from `runs/run_*.json`, tests from gates.toml)
and writes the tidy CSV `engine/artifacts/loop/metrics.csv`. Tests: `python3 -m unittest discover -s scripts/tests`.

## Fidelity gates and oracle protection (card FIDL, 2026-10-08)
Why: the OBST and FGHT audits found landings that regressed things no gate named (Tatooine rock gap, Naboo column, Bespin
edge; FGHT's contact bars missed), and nothing stopped a landing from changing its own yardstick.

**Fidelity gate** (`gates.py` group `fidelity`; rides on `sweep`, so every landing with a core/game change runs it, ~4 min,
`sweep_seeds.py --botq` seeds 1-8). `gates.toml [fidelity.metrics.<name>]` holds, per metric, `dir` (down = lower is better,
up = higher, track = recorded only), `tol_agg_abs`, `tol_map_abs`, the all-map value `agg` and a `maps` table. Metrics: finished_pct,
stuck_max_med, recycled, noprog, world_death_pct, wip_pct (walking in place), offgraph_pct, still_pct (standing still),
mate_3_pct / mate_1p5_pct (a teammate within 3 / 1.5 m), contact20_pct, rev_mean / rev6_pct (reversals), pinball_pct, idle20_pct,
stand20_pct, teamkill_pct; deaths_per_min is tracked only. A value worse than baseline by more than its tolerance FAILS (`land.py`
refuses, "fidelity FAILED"); better is PASS and listed in `<out>/fidelity/fidelity.md`. A card that moves one on purpose says so:
`declares: sweep; fidelity=mate_3_pct,still_pct,contact20_pct` (tokens: `metric`, `metric@map`, `metric@agg`, `@map`, `all`) - a
declared worse value is MOVED, not FAIL. The card is found from `OPENBF_JOB_JSON` (its committed header in the main checkout),
else by `branch:`. Per-map tolerances are wide (chaos between like builds is up to ~10 points); the `agg` row is the sharp one.
Re-baseline after a declared (or wanted) move: `python3 scripts/gates.py --rebaseline-fidelity <out>/main/fidelity/fidelity_new.json
[--only m1,m2] --source "<run dir, commit>"` (rewrites only the `# ---- fidelity` block). OWED in land.py: call it from `rebaseline()`
when the card declared `fidelity=` (until then run it by hand after `--commit`). Tolerances and directions are hand-set in gates.toml.
Run alone: `gates.py --out D --groups fidelity [--fidelity-dir <finished sweep_seeds --botq dir>]`. Self-test:
`python3 scripts/gates.py --selftest-fidelity`.

**Oracle protection** (`scripts/oracle_guard.py`; classes goldens, pcsx2 data, parity cache, baselines = gates.toml + loop_baselines,
psref = bot_quality_ps2.json, guards = gates.py / land.py / oracle_guard.py / claude_guard.sh / .claude/settings.json).
- Hook (PreToolUse on Edit|Write|MultiEdit|NotebookEdit|Bash, `.claude/settings.json`): in a loop session (`OPENBF_LOOP=1`, set by
  autoloop.sh) it denies a write to a protected path, in the main checkout or any `openbf-wt/` worktree, with a message. Exempt:
  job `planner` or `land`; `OPENBF_ORACLE_EDIT=1` in the harness environment (not a variable typed into a command: that is refused);
  a `capture`-phase card for pcsx2 data; a card whose COMMITTED header declares `oracle=<class,...|any>` (`goldens=` frees goldens).
  Interactive sessions (no `OPENBF_LOOP`) are never blocked. Off switch: `OPENBF_ORACLE_GUARD=0`, or remove the second hook entry.
  Test: `python3 scripts/oracle_guard.py selftest`; by hand: `echo '{"tool_name":"Edit","tool_input":{"file_path":"x/engine/goldens/a.png"}}' |
  OPENBF_LOOP=1 python3 scripts/oracle_guard.py hook`; `oracle_guard.py check <path>` names a path's class.
- Gate (`gates.py` group `protect`, rides on `map`): `git diff main...HEAD` over the same paths; a build card that changed one without
  declaring it FAILS ("UNDECLARED change of the yardstick"), declared = MOVED. This is what catches a script-made or committed change.
- Not covered: a script a session runs that writes protected files itself (`goldens.sh`, a python file it wrote earlier), a command built
  from variables or a `bash -c` string, writes under any other path then copied by a tool the hook does not know, editing the hook's own
  settings via a tool when the session has no loop env, and anything in a worktree created before `.claude/settings.json` was committed
  (sessions start in the main checkout, so the main copy applies). The hook is a tripwire for a hasty session, not a sandbox.

## CPU: the loop yields to the maintainer's own work (2026-10-11)

`scripts/start_loop.sh` starts the loop in the systemd user slice `openbf.slice` and starts `scripts/loop_cpu.py watch`
beside it. Two things keep the loop out of the way:

- **The slice's weight** (CPUWeight / IOWeight 10; an ordinary program is 100, KDE's own session 500): on a contended
  core the maintainer's programs win. `OPENBF_CPU_WEIGHT` changes it.
- **Core affinity, by presence** (`loop_cpu.py`): while the maintainer is at the machine every thread in the slice is held to the
  efficiency cores (`/sys/devices/cpu_atom/cpus`, 16-31 here), which leaves the 8 performance cores to him; when the lid
  is closed, the screen is locked or there has been no keyboard / touchpad / mouse interrupt for 5 minutes
  (`OPENBF_IDLE_SECS`) the slice gets every core. It checks every 5 s. PCSX2 processes always keep every core (a capture
  presses keys on the wall clock and must run at full speed). `OPENBF_POLITE=0` turns the affinity off.
  Why affinity and not only the weight: measured 2026-10-11, 32 busy threads at ANY weight slow 8 of the maintainer's jobs 1.7x
  (0.56 -> 0.95 s: shared physical cores and lost turbo); held to the efficiency cores they cost nothing (0.53 s).

`scripts/loopbin/systemd-run` is first in the loop's PATH (autoloop.sh): a session's own `systemd-run --user --scope`
gets `--slice=openbf.slice` added, so its capped scopes stay in the slice. `scripts/start_loop.sh status` prints the
mode (`engine/artifacts/loop/cpu_mode`); transitions are logged in `engine/artifacts/loop/cpu_governor.log`.
Not covered: a process a session starts some other way outside the slice keeps the affinity it was born with; idle is
read from /proc/interrupts (KDE on Wayland gives no idle time), so an input device on an unusual interrupt line is not
seen - the screen lock (15 min) and the lid still are.
