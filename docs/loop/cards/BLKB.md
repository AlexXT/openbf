---
id: BLKB
title: Playtest black box (dev mode): the windowed game keeps the last 60 s of every unit's state in memory and writes a snapshot file on F9, on exit and on a panic, so the maintainer's bug reports come with the bots' goals, legs, steer picks and positions, and the maintainer's own state
milestone: M0
value: 5
phase: build
status: done
deps:
files: engine/crates/game/src/blackbox.rs,engine/crates/game/src/unit_probe.rs,engine/crates/game/src/main.rs,engine/crates/game/src/main/windowed.rs,engine/crates/game/src/assets.rs,engine/crates/game/src/lib.rs,engine/crates/core/src/ai.rs,engine/crates/core/src/ai/steer.rs,engine/crates/core/src/collision/walls.rs,engine/crates/game/examples/blackbox_read.rs,engine/crates/game/examples/round_to_outcome.rs,engine/CRATES.md,CLAUDE.md
declares:
needs:
effort:
branch: blkb
resume: landed 7b64fcd2 2026-10-02
created: 2026-10-02
alpha: yes
first: 1
---

## Why
The maintainer, 2026-10-02: he now plays the port himself and each time comes back with screenshots and a description. A snapshot of
the game's own state (every bot's task, position, route leg, steer pick; the maintainer's own position and input) lets Claude find
the cause from the file instead of guessing from a picture. DEV ONLY: it goes away (off by default, compiled in but
inert) when the game is playable for real. The maintainer's flow: he plays, sees something wrong, presses F9 (or just closes the
game), comes to Claude with a screenshot and a line; Claude reads the snapshot, writes the finding into a card for the
loop, the maintainer restarts the loop and plays again. The loop's own sessions may also use it (a headless run can write one).

## What to build
1. **Switch.** On when `OPENBF_DEV=1` (the existing dev mode, `game/src/dev.rs`) - nothing in normal play, no cost then.
   `OPENBF_BLACKBOX=0` turns it off inside dev mode.
2. **A rolling in-memory ring of the last 60 s of game time** (a ring of frames, fixed capacity, no allocation per tick
   once warm; core stays 0-alloc - the recorder lives in `game`, reads the sim by reference, and is never hashed):
   - **Every 0.5 s of game time, for every alive unit (bots and the maintainer):** handle/id, team, class, name, position, yaw,
     floor/level, stance, hp, speed, on-ground, in-vehicle/turret id; for a bot its whole brain: objective (post / hold /
     follow / attack) and why, route leg (from hub -> to hub, destination post), next waypoint and its distance, `SteerPick`
     and whether a wall / barrier / ramp / steep sweep hit and on which collision body, hold state, perceived enemy and range,
     the stuck / no-net-progress counters, the locus (hub / arc / Nearest), recycle and lost state, the skill tier.
   - **The maintainer, every tick for the last 5 s and every 0.5 s before that:** position, camera yaw / pitch, input state (stick,
     buttons), weapon and ammo, hp, what the reticle is on (`Sim::reticle_pick`), the vehicle or turret he is in.
   - **World, every 2 s:** post owners and capture progress, tickets and bleed, every vehicle and turret (position, team,
     occupants, hp), door states, plus the **event log with timestamps** (kills with killer, victim, weapon and place;
     captures; recycles; lost deaths; spawns; the announcer's lines; stuck-run starts).
   - **Per-bot trail:** each bot's last 10 s of positions at 4 Hz, so a bot walking into a wall shows as a pile of points.
3. **The file.** One header and the ring, as JSON (small enough to read: bots' brain fields in short keys are fine, a
   `schema` key and a key legend in the header). Header: git commit and `dirty` flag, build time, map, mode, seed,
   difficulty, window size, tick, game-time, the frame rate, the human's unit id, `OPENBF_*` env, and the reason it was
   written (`hotkey`, `exit`, `panic`, `manual`). A `focus` block: the unit the human is looking at or aimed at when F9
   was pressed (its id) and the world point under the reticle, so "this guy" in a bug report is findable.
   Written with write-to-temp-then-rename, so the file is never half written.
   **Location:** `engine/artifacts/playtest/` - `last.json` is always the newest; keep the previous 5 as `prev1.json`..
   `prev5.json` (rotate on each write), plus `last.png` (a screenshot of the frame, if the renderer can read it back
   cheaply; if not, skip it and say so). Add the folder to `.gitignore`.
   Triggers: **F9** (writes now, shows a short "snapshot saved" line in the HUD's message box), **window close / quit**,
   **a panic** (a panic hook that writes before the process dies), and a **stuck alarm** (a snapshot when the human's
   unit has not moved for 20 s while not dead - often a bug - at most once per 2 minutes).
4. **A reader**: `engine/crates/game/examples/blackbox_read.rs` (a probe: runs under the memory cap) printing a digest
   Claude reads first: the header, the human's last 10 s, the focus unit's brain and trail, every bot with a
   no-progress or wall-slide run over 3 s (who, where, which body, goal), every bot within 15 m of the human, post
   ownership and events of the last 30 s, and `--unit <id>` / `--near x,y,z,r` / `--events` views. Include a
   `--replay-spot` option that prints the spot as a line for `STUCK_CENSUS` / `sweep_seeds.py` style follow-up.
5. **A headless writer**: `round_to_outcome` (and the sweep's example) can write the same file at a tick or at an event
   (`BLACKBOX_AT=<tick>` or `BLACKBOX_ON=lost|recycle|stuck`), so a loop session can snapshot a bug it is chasing.
6. **Make it known (acceptance item, not optional):** write a `CLAUDE.md` section titled "Playtest black box" at the repo
   root `CLAUDE.md` (create it if absent; the planner's note below is the text), and add one line to
   `engine/artifacts/briefs/LOOP_PROMPT.md` and `OPUS_LOOP_PROMPT.md`: "the maintainer's playtest snapshots are in
   `engine/artifacts/playtest/` - read `last.json` (via `blackbox_read`) when a card or an INBOX note cites one."
   Commit the tracked-doc edits with the card's landing (a dirty tracked doc blocks the ff-merge).

## Stretch (a second card if this one lands cleanly - do not hold this card for it)
**Input recording + replay:** record the human's input stream and the RNG seed from the start of the round into the
snapshot; `blackbox_read --replay <tick>` runs the sim headlessly to that tick and drops into the usual probes. This only
works if a round with a human in it is bit-deterministic: first measure that (record a 5-minute round, replay it, compare
the sim hash every second) and card the result either way.

## Acceptance
- A windowed run in dev mode (under the repo's GUI isolation: unset WAYLAND_DISPLAY, Xvfb, `QT_QPA_PLATFORM=xcb` - see the
  machine notes; never open a window on the maintainer's desktop) with F9 pressed writes `last.json`, rotates `prev1.json`, and
  `blackbox_read` prints its digest. A kill -SEGV or a forced panic still writes a file.
- Overhead: frame time with the ring on vs off, mean and p99, on BF2 KAM1 with 32 units (no visible hitch at the write,
  the write off the main thread or under 20 ms); memory of the ring, in MB, reported. Sim hashes, goldens, parity and the
  sweep are unchanged (the recorder only reads) - say so with the gate output.
- The file size for 60 s on a 64-unit map is reported; under about 8 MB, or say why not and what was cut.
- A test: the ring's wrap (60 s fills, the oldest frame drops), the header's fields, the atomic write, the `OPENBF_DEV`
  switch off = no file and no ring.
- `CLAUDE.md` and the two prompts name the tool (item 6).

## Notes
- Related: SQMK (the dev overlay, `OPENBF_DEV`), OBST (the first card this tool should serve: snapshot the maintainer's
  wall-walking bots), the stuck census (`STUCK_CENSUS`), `UNIT_SAMPLES` in `round_to_outcome` (its per-unit second samples
  hold most of the bot fields already: reuse the field list and the code that builds them; do not write a second one).
- Run any new probe or example binary under the memory cap (INBOX 2026-10-01 22:00).
- It is DEV scaffolding: keep it in its own module (`game/src/blackbox.rs`) so removing it later is one delete.

## Built (2026-10-02, lane 2) - what landed and the checks that were run
`game/src/blackbox.rs` (the ring, the file, the triggers), `game/src/unit_probe.rs` (the unit row, shared with the sweep's
`UNIT_SAMPLES`), `examples/blackbox_read.rs`, `round_to_outcome`'s `BLACKBOX_AT` / `BLACKBOX_ON` / `BLACKBOX_TIMING`,
two read-only core diagnostics (`CollisionWorld::wall_sweep_body`, `ai::way_ahead`), root `CLAUDE.md`, the line in
`briefs/LOOP_PROMPT.md` (and the archived `briefs/v1/OPUS_LOOP_PROMPT.md`). `engine/artifacts/` was already gitignored.
Acceptance, each RUN (evidence `engine/artifacts/lead_tmp/blkb/`):
- Windowed, dev mode, own Xvfb display (`win_run.sh`: WAYLAND_DISPLAY unset, private runtime dir, scratch profile, no
  sound device; F9 by a raw-X11 XTEST client `xkey.py`): BF2 KAM1 16 v 16, F9 twice then the window closed -> `last.json`
  (exit), `prev1.json`, `prev2.json` (hotkey), and `blackbox_read` prints the digest (`win1/`). A forced panic
  (`OPENBF_BLACKBOX_PANIC=450`) wrote `last.json` with reason `panic: ... forced at tick 450`, exit 101 (`win2/`).
  kill -SEGV is NOT covered (no signal handler; a Rust panic is).
- Overhead, BF2 KAM1, 32 units, 9000 ticks headless (`timing/`): the recorder costs mean 63 us per tick, p99 1.0 ms, max
  1.5 ms (the 0.5 s unit frame); the hotkey's share of a frame is a 0.13 ms copy, the encode + write 4.1 ms on its own
  thread. Windowed with 65 units: frame mean 19.1 / p99 21.8 / max 22.0 ms before the F9 write, 19.1 / 21.8 / 22.1 ms
  after it (no hitch). Ring memory 1.19 MB at 32 units, 2.00 MB at 65.
- File size, 65 units x 60 s (BF2 KAM1 `--ai 32 --ai-cap 64`): 4.6 MB (under 8 MB).
- The recorder only reads: the sweep's stdout with the black box on is byte-identical to off (result + hash, 3 maps at
  3000 ticks and KAM1 at 9000), the `UNIT_SAMPLES` CSVs byte-identical to the pre-change binary's on 3 maps; land.py's
  gates below.
- Tests (`game --lib blackbox`): the wrap at 60 s, the header + read-back, the atomic rotating write, off = no ring and
  no folder, the stuck alarm. Mutation-checked: a wheel that does not advance fails the wrap test.
Cut / different from the card: no `last.png` (the windowed renderer cannot read its frame back); the stuck alarm wants
the stick PUSHED for 60 % of the 20 s (standing still on purpose is not a bug, and an alarm rotates the files); a headless
snapshot goes to `playtest/headless/<map>_s<seed>.json`, never over the maintainer's `last.json`; the "wall / barrier / ramp /
steep hit" is asked by the probe along the straight line to the leg's aim point at each 0.5 s sample, not carried by the
brain (core's state and hash are untouched). Stretch (input recording + replay): not started - a second card if wanted.
