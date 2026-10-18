---
id: VHULL
title: Parked hulls are solid: a spawned vehicle's coll/prim bodies join CollisionWorld at its transform — soldiers stop at it, bolts hit it (VDMG's hull pool goes live), AI rays see it; a declared sim/sweep move
milestone: M2
value: 5
phase: build
status: done
deps: VDRAW
files: core/src/sim/vehicles.rs,core/src/collision/world.rs,game/src/vehicle_pads.rs
declares: sweep
needs:
branch: vhull
resume: landed a8da834f 2026-09-29
created: 2026-09-26
---

## Why
After VDRAW, a parked vehicle is visible but soldiers walk through it and bolts fly through it; VDMG's hull damage
path is reached only by tests. On the PS2 a vehicle is a solid body that bolts hit.

## Evidence
VDMG report (`reports/20260926_1225_build_VDMG.md`): "Hulls join the bolt ray when HOVR/VSEAT put them in the
collision world". VBAKE bakes each class's `coll` mesh + `prim` bodies (3 undecoded prim kinds dropped: both AT-TEs, BF2
oneman AT-ST). `core/src/collision/world.rs` (`move_body` already moves door bodies).

## Acceptance
- A spawned vehicle's bodies enter `CollisionWorld` at its transform (with `Hit::Target(Target::Vehicle)`); removed
  when it is destroyed or despawned; moved by `move_body` when HOVR moves it.
- Tests: a soldier walking into a TAT1 pad vehicle stops; a bolt at it damages the hull (VehicleScale); a destroyed
  hull no longer blocks.
- Declared moves: sim/sweep (bots' paths and stray bolts now meet hulls) — measure the sweep: no new stuck maps.
  Which layers the PS2 tests a vehicle against for soldiers / ordnance: from re-vehicles §3/§6 and the collision-layer
  lists VBAKE wrote; unread → PROVISIONAL.

- VTURB (2026-09-29): building turrets are now `VehicleKind::Turret` vehicles whose hull pool (MaxHealth, HealthType building) nothing can hit: the building's collision mesh is a world body with no link to the vehicle. Give a turret's pool the ray through its instance's body; the same link lets its OWN seat's bolts and the exit's path test ignore that body (today the exit skips the path test for turrets — PROVISIONAL in `core::sim::seats` — because HOT1's origin sits inside its mesh).

## Notes
- Planner 2026-09-29 19:05: value 4 → 5. M2 had no value-5 card after WALK. PLAN's M2 is ground vehicles first, and soldiers and bolts passing through every parked vehicle is the most visible ground-vehicle gap left; it also takes VDMG's hull pool live. Reverse: set value=4.
