---
id: VTUR2
title: BF2 building turrets: bake SIDE/TUR.LVL's classes (tur_bldg_laser, _tower, _chaingun_*, _recoilless_*, _hoth_*, _beam, _mortar, tur_weap_built_gunturret — 333 instances on BF2 maps) with their models and skel nodes, so vehicle_pads::add_turrets spawns them as VTURB turrets
milestone: M2
value: 5
phase: build
status: done
deps: VTURB
files: engine/crates/bake/src/map.rs engine/crates/bake/src/side_classes.rs engine/crates/manifest/src/lib.rs engine/crates/game/src/assets.rs engine/crates/game/src/vehicle_pads.rs scripts/rebake.sh
declares: goldens=maps/bf2_cor1.png,maps/bf2_end1.png,maps/bf2_geo1.png,maps/bf2_hot1.png,maps/bf2_kam1.png,maps/bf2_kas2.png,maps/bf2_myg1.png,maps/bf2_nab2.png,maps/bf2_yav1.png,maps/tat2.png; rebake=maps,classes,sounds; sweep; leak; arena
needs:
branch: vtur2
resume: landed a1bf3e87 2026-09-30
created: 2026-09-29
alpha: yes
---
## Why
VTURB (2026-09-29) made every armed-building instance a seat-only vehicle, but BF2's building turrets resolve to no class: `tur_*` classes live in `extracted/bf2/lvl/DATA/_LVL_PS2/SIDE/TUR.LVL`, which no class bake reads (`assets_baked/classes/bf2_*.ron` = all, cis, imp, ingame, rep). Instance counts (BF2 manifests): tur_bldg_laser 122, _tower 35, _chaingun_tripod 31, _recoilless_lg 26, _chaingun_roof 25, _hoth_dishturret 24, tur_weap_built_gunturret 27, _hoth_lasermortar 18, _beam 8, others ≤ 5.

## Evidence
re-vehicles §8 (BF2 armedbuilding FUN_001e8860 → FUN_001e8948 → FUN_00270770; portableturret / defensegridturret are armedbuildingdynamic). BuildingSection hash in B2 FUN_001e8948 B2:97129. Report `engine/artifacts/loop/reports/20260929_*_card_VTURB.md`.

## Acceptance
`bake classes` (and the node bake, `bake/src/hardpoints.rs`) cover TUR.LVL; `turret_ride bf2_<map> <n> rep cis` enters, fires and exits a BF2 tur_bldg_laser; check PilotType (a `self` turret is VTAI's, not boardable); declare the sweep move (new entities on BF2 maps).

- AUDIT VTURB (2026-09-29): the port's census spawns 0 turrets on all 16 baked BF2 maps. The conquest missions load
  `SIDE\tur.lvl` with named classes:
  - kam1c_con: tur_bldg_chaingun_roof, tur_weap_built_gunturret
  - geo1c_con: tur_bldg_geoturret
  - yav1c_con: tur_bldg_laser, tur_bldg_tower
  - nab2g_con, cor1c_con, end1g_con: tur_bldg_laser
  - kas2c_con: tur_bldg_beam, tur_bldg_recoilless_kas
  - myg1c_con: tur_bldg_recoilless_lg
  - tat2g_con: tur_bldg_tat_barge, tur_bldg_laser
  - (strings of `extracted/bf2/lua/mission/*.luac`)
- BF2 ODFs name `portableturret` (10 classes) and `defensegridturret` (3) as BASE. Both are engine labels in the B2 ELF
  beside `armedbuildingdynamic`, which no ODF uses, so `vehicle_class::TURRET_BASES` must list the two disc names.
  All 13 author PilotType self (VTAI's).

## Notes
- 2026-09-29 (PXWB, lane 2; FINDINGS F52): the PS2 DRAWS these — BF2 HOT1's defence towers stand beside cp6 on the PS2 and are absent from the port's frame (`pcsx2/ref/bf2_hot1/walk/pairs/cp1_cp6__sheet.png` b180: `tur_bldg_hoth_lasermortar` turet11, 21.9 m at yaw 155; b270: turet12, 22.1 m at yaw 282; `cp2_cp3__sheet.png` b270 more on the plain). The bake has no model for any `tur_bldg_*` instance, so today they are invisible as well as unboardable.

## Planner note (2026-09-30, M-ALPHA)
`alpha: yes` — BF2 HOT1's building turrets. Value 4 (kept). Showcase maps: BF1 NAB1, BF1 TAT1 (the port's `tat1`), BF1 YAV2, BF2 UTA1, BF2 MYG1, BF2 HOT1 (PLAN §Milestones M-ALPHA).
- Planner 2026-09-30 08:30: value 4 -> 5, so M2 has a value-5 card beside M1's effects chain (interleave). 0 of 333 BF2 building turrets spawn; showcase HOT1 and UTA1 carry them.
