---
id: VDRAW
title: Parked vehicles drawn: every pad-spawned vehicle (VPAD) drawn rigid at its sim transform from VBAKE's side models (LOD0, cross-side textures), the pad yaw checked against a PCSX2 frame — every map's vehicles become visible
milestone: M2
value: 5
phase: build
status: done
deps:
files: game/src/viewer/vehicles.rs,game/src/viewer/scene.rs,game/src/vehicle_pads.rs,game/src/walk.rs,game/src/main/{battle,headless,windowed}.rs,game/tests/vehicle_pads.rs
declares:
needs:
branch: vdraw
resume: landed 9f60ffdf 2026-09-28
created: 2026-09-26
---

## Why
VPAD (896b924) spawns every map's vehicles on tick 1 (TAT1: 16), and VBAKE (0a81fc9) baked all 75 side classes, but
nothing draws them: the battlefield has no vehicles in it. Drawing the parked ones is M2's first thing a player sees,
and it needs neither HVRE nor HOVR. Split out of VSEAT (planner 2026-09-26), whose "skiff drawn at its transform" it
generalises.

## Evidence
- `core::vehicle_pad` / `Sim::tick_vehicle_pads` (spawn at the pad, yaw = atan2(r02, r00) PROVISIONAL, height
  `ground_under` PROVISIONAL); `game::vehicle_pads::install` returns `PadBuild::classes` (nothing keeps it yet).
- `side_bf{1,2}/vehicles/<side>/vehicles.ron` + `models/` (VBAKE, rigid in bind pose); map-local classes (TAT1 skiff,
  BES1 cloud car) in the map bake. 13 chunk/LOD models name a texture only another side LVL ships (VBAKE report,
  re-vehicles §10 item 3): the lookup spans every loaded side.
- PS2 frames with parked vehicles: search `engine/artifacts/pcsx2/ref/*/` (TAT1, HOT1, GEO1 spawn poses).

## Acceptance
- Every live `Sim::vehicles` entry draws its class's LOD0 at the sim transform, each frame (a destroyed one vanishes
  with it — VDMG). Drawn through the world path (ALPHA RULE v2, the model lighting).
- Keep `PadBuild::classes` where the viewer can map `VehicleSpec::class` → model.
- One PCSX2 pair per game at a pose showing a parked vehicle: position and yaw line up (fix VPAD's yaw / height if
  not; record it). Pictures viewed.
- ci green; sim / simhash / sweep / arenas unchanged (draw only). Goldens: list the golden views that see a pad
  BEFORE building and declare exactly those.

**Note (HOVR audit, 2026-09-28):** VPAD's pad yaw is verified against PS2 RAM. All 44 quiet parked BF2 hovers within 1 m of a pad face the port's spawn heading, quat_y(-atan2(r02, r00)), to within 0.51 deg (most within 0.05 deg), and sit within 0.17 m (xz) of the pad. The port's settled heights match 45 of 47 quiet parked PS2 hovers inside the bob band (median +0.6 cm). Two spawns are wrong until HCON lands: POL1 hangerdockcon is catapulted, and the TAT1 skiff pops 5.2 m. Evidence: lead_tmp/audit-hovr/, reports/20260928_1100_audit_HOVR.md.

**Landed 2026-09-28 (main 9f60ffd; report `reports/20260928_1200_resume_VDRAW.md`).** The per-game pose check:
BF1 was checked with a PCSX2 pair: the YAV1 AAT's position and yaw line up (`lead_tmp/vdraw/pair_yav1_aat.png`,
PS2 on the left). BF2 was checked with the HOVR audit's PS2 RAM check (44 parked hovers, yaw within 0.51°) instead of a frame.
No golden view sees a battle vehicle, so there was nothing to declare. The parked flyer's height is left open: the GEO1 LAAT's skids sit in the ground (note on VFLY).
