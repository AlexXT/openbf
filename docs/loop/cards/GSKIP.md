---
id: GSKIP
title: The selective gates skip the golden step for any diff under engine/crates/core/, but core decides golden pixels: the gallery / walkabout vantage on the six stub-terrain maps is floored by core::collision ray_down, grass / foliage / leaves sit on core::heightfield, doors pose by core::door — GRND's whole file set would be skipped while it changes ray_down
milestone: M0
value: 5
phase: build
status: done
deps:
files: engine/goldens/deps.toml,scripts/gates_selftest.py,scripts/gates.py,engine/goldens/README.md
declares:
needs:
effort:
branch: gskip
resume: landed bc6a047c 2026-10-01
created: 2026-09-30
alpha: yes
---
## Why
The maintainer's INBOX (2026-09-30, speed-up round): "the first landings after this are the real test — the planner should read
their GATES_FOR_REPORT.md for anything odd (a wrong skip, a reused row that should have re-run) and card it value 5."
The planner read BPNT's, STHOP's and CLNC's. The reused rows and the single sweep/arena/leak runs are right. The skip
rule is not: `engine/goldens/deps.toml` lists `engine/crates/core/` under `skip` ("the goldens are static frames and none
of these draws"), but `core` code decides golden pixels in at least four places.

## Evidence (read 2026-09-30 21:45, main f0da5ad; none of it run — step 1 below runs it)
- `engine/crates/game/src/gallery.rs:232-240`: on a map with a stub `tern` (COR1, DEA1, MYG1, TAN1, UTA1, BF2 KAM1 — two
  are showcase maps) the gallery / walkabout vantage is floored by `scene.collision_world().ray_down(…)` =
  `core/src/collision/rays.rs:57`. A changed ray answer moves the camera, so the whole sheet.
- `game/src/viewer/grass.rs:217`, `foliage.rs:59`, `leaf.rs:265`: placement and tilt from
  `core::heightfield::Heightfield` (`height_at`, `normal_at`).
- `game/src/viewer/scene.rs:140`: door leaves posed by `core::door::sample`.
- `game/src/viewer/vehicles.rs:273-294`, `walker_pose.rs:28`: `core::vehicle::walker` (only if a golden draws a walker).
- CLNC (de2541b) was the first landing skipped under the rule: `gpu | goldens skipped (deps.toml) | no golden can move:
  16 changed file(s)`, with 11 of them under `core/src`. Its core changes were comments and one unused function, so nothing
  was missed THAT time.
- The next case is real: **GRND**'s `files:` are `core/src/collision/rays.rs`, `core/src/sim/state.rs` and an example —
  every one under a skipped prefix — and it changes `ray_down_with`. SRAD, VCOL, VSUB and LEAK3 edit `core/src/collision` too.
- The safety net (every 10th crate landing regenerates everything) would show the move up to nine landings late, on
  another card's landing, which then has to explain a golden it did not move.

## Do
1. Prove the dependency: in a scratch worktree add 1.0 to `ray_down`'s returned height, run `scripts/goldens.sh gallery
   walkabout` there (overlay dir — memory: lane picture outputs need redirecting) and list which goldens move. Then the same
   for `Heightfield::normal_at` (e.g. always +Y).
2. Replace the blanket `engine/crates/core/` skip by what step 1 shows: either move the draw-feeding files under `always`
   (`core/src/collision/`, `core/src/collision.rs`, `core/src/heightfield.rs`, `core/src/door.rs`,
   `core/src/vehicle/walker*`, plus anything step 1 finds), or skip only the core subtrees that provably cannot draw
   (`core/src/ai/`, `core/src/sim/`, `battle.rs`, `squad.rs`, `weapon.rs`, tests). Prefer the second: a new core file is
   then NOT skipped by default.
3. `scripts/gates_selftest.py`: cases for both directions. README §"Which goldens a change regenerates" updated.

## Acceptance
- `python3 scripts/gates_selftest.py` passes with new cases: a diff of only `engine/crates/core/src/collision/rays.rs` →
  every golden group runs; only `engine/crates/core/src/heightfield.rs` → runs; only `engine/crates/core/src/ai/steer.rs`
  → skipped; only `engine/crates/core/src/soldier.rs` → skipped (or runs, if step 1 shows a path — say which).
- Step 1's mutation result pasted in the report: the list of goldens the +1.0 m `ray_down` moved (expected: the six
  stub-terrain map sheets and `walkabout_stats.ron`; if NONE moves, close this card as "rule sound" with that output).
- The history check of the README re-run with the new rule: still 0 golden-moving commits skipped; report the new skip
  share of the other commits (55 % today).
- `land.py --self-test` PASS. Every gate PASS, nothing declared.

## Notes
- memory `land-runs-mains-gates-script`: land.py runs main's `gates.py`, so this lands as its own card before the
  cards that rely on it; if `gates.py` itself needs no edit (deps.toml + selftest only), say so.
- Until this lands: GRND, SRAD, VCOL, VSUB, LEAK3 run `gates.py … --golden-groups all` (notes on those cards).
## Outcome (2026-10-01, landed bc6a047; report `reports/20261001_2035_card_GSKIP.md`)
- Mutations of main ce66fe2: `ray_down` + 1.0 m moved maps/bf2_cor1.png, maps/bf2_uta1.png, walkabout_stats.ron;
  `normal_at` = +Y moved bf1_end1, bf1_rhn2, bf1_yav2, bf2_end1, bf2_fel1 and the stats; a panic in `Sim::empty` /
  `Sim::new` moved nothing.
- Rule: only core's sim*, ai*, aim_assist, perception, skill, squad, tests and examples are skipped. `soldier.rs` RUNS.
- `gates.py` WAS edited: `core_link_check` fails the gpu gate when deps.toml skips a core module a golden test links.
- History (`scripts/gates_history.py`): 47 golden-moving landings, 0 skipped; 57 % of the other commits skip (65 % before).
