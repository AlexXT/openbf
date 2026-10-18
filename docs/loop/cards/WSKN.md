---
id: WSKN
title: The walker drawn alive: the skinned walker GLB (VBAKE left walkers rigid) and its clips played from WALK's anim state — the idle/leftup/rightup layers, the legs' planted turn steps, the pitch/roll bone
milestone: M2
value: 5
phase: build
status: done
deps: WALK
files: engine/crates/bake/src/vehicle.rs,engine/crates/bake/src/walker.rs,engine/crates/bake/src/glb.rs,engine/crates/bake/src/skin_glb.rs,engine/crates/game/src/viewer/vehicles.rs,engine/crates/game/src/viewer/walker_pose.rs,engine/crates/game/src/assets.rs,engine/crates/core/src/vehicle/walker.rs,engine/crates/manifest/src/lib.rs,engine/crates/game/examples/walker_drive.rs
declares: rebake=side; goldens=any; leak
needs:
branch: wskn
resume:
created: 2026-09-29
alpha: yes
---
## Why
Walkers draw rigid in their skel bind pose (VBAKE). After WALK an AT-ST would slide across the ground without
stepping: its feet never plant and its turn steps are invisible.

## Evidence
- analysis/re-walker.md §5: the 11-entry clip table, the three layers (base, _leftup, _rightup) and the transition
  table DAT_005fd100.
- §6: each leg pair's leftup/rightup blend P+0x120.
- §10: the leg offset is a yaw applied to bone 0, so the model stays planted while the body frame turns. The
  pitch/roll quaternions go on bone 0x16e27226.
- The VBAKE note (card VWLK, bake/src/vehicle.rs module doc): each walker model has a same-named skel.
  rep_walk_atte carries MTRL 0x07 / 0x10003 (the 0x02 alpha test) that bake::soldier::segment_blends refuses.

## Acceptance
- bake: the skinned GLB (skel + weights) for the walker-family classes, with their banks' clips. The alpha-test
  pass is decided for rep_walk_atte (record the call).
- game: the walker is drawn skinned from WALK's anim state (clip index, direction, time; interpolated), with:
  - the leftup/rightup layer blend;
  - §10's root yaw and the pitch/roll bone.
- Viewed in the vehicle gallery and in a walkabout sheet with a walking AT-ST, whose feet plant and whose idle turn
  steps show.

## Planner note (2026-09-30, M-ALPHA)
`alpha: yes` — walkers drawn with moving legs (HOT1's AT-AT / AT-ST). Value 4 (kept). Showcase maps: BF1 NAB1, BF1 TAT1 (the port's `tat1`), BF1 YAV2, BF2 UTA1, BF2 MYG1, BF2 HOT1 (PLAN §Milestones M-ALPHA).

## Planner note (2026-09-30 13:30)
Value 4 → 5: M2's top card. HOT1 is a showcase map, and an AT-AT / AT-ST that slides rigid in its bind pose is the most
visible vehicle defect left in a 5-minute alpha video. WALK is done, so nothing blocks it.
