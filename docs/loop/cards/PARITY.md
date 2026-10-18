---
id: PARITY
title: A PS2 parity score: re-render the port at every stored PS2 pose and score each pair (16×16 block error, mean luma difference, p99 luma, HUD/minimap/own-body masked) → loop/parity/latest.csv + per-map summary + history; the scoreboard line; a gate for visual landings
milestone: M0
value: 5
phase: build
status: done
deps:
files: scripts/parity.py,scripts/gates.py,scripts/land.py,scripts/loopctl.py,engine/crates/game/examples/pose_render.rs,engine/CRATES.md,engine/artifacts/pcsx2/port.py
declares:
needs:
branch: parity
resume: landed 144b2f0 2026-09-30; baseline history.csv 12.000
created: 2026-09-30
---

## Why
The maintainer's INBOX note 2026-09-29 22:40 (approved loop improvements), item 2. The PS2 reference library has about 700 comparable pairs: the PXWA/PXWB walkabouts (187 sheets, 4
bearings each) and the earlier PXC sets. Every gate says "nothing broke"; nothing says "we got closer". The PXC census
metric (16×16 block error, 18.3 → 11.2 mean on 15 fogged maps) and p99 luma (which caught the droid shine) already
exist as scratch scripts.

## Do
- **Renderer.** `pcsx2/port.py` runs a scratch pose renderer (`tools/pxc_pose.rs`, binary under
  `lead_tmp/pxc-render_target`).
  - Make it a committed example, e.g. `engine/crates/game/examples/pose_render.rs`, so parity always renders main.
  - Keep `port.py` as the caller.
- **`scripts/parity.py`.**
  - Render every stored pose (`pcsx2/ref/<map>[/walk]/*.json`) under ci.lock, into a scratch dir. Never overwrite
    `ref/*/port/`.
  - Score each pair: block error, mean luma difference and p99 luma.
  - Mask the HUD and minimap regions and the soldier's own body. Use `walk_sheet.py`'s masks if they exist; otherwise
    one fixed mask per game.
  - Write `engine/artifacts/loop/parity/latest.csv` (map, spot, bearing, three metrics), a per-map summary and an
    appended `history.csv` (date, main sha, per-map mean).
- **Scoreboard.** One line: `parity: mean block error X (was Y), worst 5 posts …`. The planner reads the worst posts
  as a ranking input.
- **Gate.** `gates.py` gets a `parity` gate, run for a landing whose `declares:` has `goldens=`. It scores the affected
  maps on the branch and on main, and FAILS a regression past a threshold unless the card states a reason
  (`declares: parity=<maps>`, like the other declared moves).

## Acceptance
- One timed run over all pairs, with the GPU lock held; the time goes in the report.
- The first baseline in `history.csv`.
- A re-run across a known fix shows the expected movement. Render DSHN's parent (DSHN's RE says the SBD's add fell
  +40.6 → +5.7), or REDB's, and main; the p99 on the affected pairs moves as expected.
- The gate is proven both ways: a scratch change that darkens one map FAILS, and main vs main PASSES.
