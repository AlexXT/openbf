---
id: VTHUL
title: A building turret's hull pool is hit through its building: the turret instance's world bodies carry the turret vehicle as owner (CollisionWorld Body.owner, VHULL), so a bolt at the building damages the pool and the turret's own seat's bolts, aim ray and exit path skip it
milestone: M2
value: 3
phase: build
status: todo
deps: VHULL
files: engine/crates/game/src/assets.rs,engine/crates/game/src/vehicle_pads.rs,engine/crates/core/src/collision/world.rs,engine/crates/core/src/sim/seats.rs
declares:
needs:
effort:
branch:
resume:
created: 2026-09-29
defer: post-alpha
value_pre_alpha: 4
---
## Why
Split from VHULL (its VTURB note). VTURB's turrets are VehicleKind::Turret vehicles whose pool (MaxHealth, HealthType building) nothing can hit: the building's collision is a world body with no link to the vehicle.

## Evidence
VHULL (report reports/*_card_VHULL.md) gave every Body an owner: Option<VehicleHandle> and HullFilter / ray_with (the owner a ray hit). Turrets get no hull parts (their building is already the world's body). core::sim::seats skips the exit path test for turrets (PROVISIONAL) because HOT1's origin sits inside its mesh.

## Acceptance
- Scene::collision_and_doors records each body's instance; vehicle_pads::install sets owner = the turret handle on its instance's bodies (a CollisionWorld::set_owner); a destroyed turret clears them (the building stays solid, a bolt at it is Hit::World).
- A bolt at a HOT1 turret building damages the pool; the gunner's own bolts and aim ray pass out of it; the exit keeps its path test (HullFilter::except skips the own building) — drop the turret skip if the HOT1 turret_ride exit still lands outside.
- Declared moves: sim/sweep (bots' bolts now hit turret pools).

## Planner note (2026-09-30, M-ALPHA)
`defer: post-alpha` — not needed for the alpha's six showcase maps or its checklist. Value 4 → 3 until the alpha (restore from `value_pre_alpha`).
