---
id: CLN10
title: Small leftovers: the `missions_root()` / `extracted_root()` literals, the era/sound/HUD-icon fallbacks, the TAT1 `--spawn` pin (delete `HISTORIC_WALK_START_MAP`)
milestone: M0
value: 1
phase: cleanup
status: todo
deps:
files:
declares:
branch:
resume:
created: 2026-09-25
---

## Evidence
analysis/PLAN_LOG_2026.md §ARCHIVED 2026-09-25 HANDOFF-OPUS.md §Standing backlog 9. `game/src/walk.rs:198` `HISTORIC_WALK_START_MAP`; pinning `--spawn -140,230` needs an edit to
`scripts/sim_gates.sh` — gate-script edits have been classifier-denied before (gpu_tests.sh); if denied, leave that half
for the maintainer in the report.

## Acceptance
Behaviour-neutral: every gate PASS, the sim hashes unmoved.

## Note (cleanse CLNR, 2026-10-02) - three more small leftovers, outside the render area
- `ui/src/hud.rs` `TEAM_COLORS` (used by the egui lines and `spawn.rs`) has no doc: say where the two colours come from, or label them PROVISIONAL.
- `ui/src/hud_draw.rs` `anchor_shift`: the 1/3 and 2/3 thresholds are tagged `[ours]` only; PROVISIONAL is the ledger keyword.
- `game/examples/arena.rs` (`SWEEP_SLOT0`'s doc) cites "PLAN §WHERE THINGS STAND", a heading PLAN.md no longer has.
