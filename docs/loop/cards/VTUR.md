---
id: VTUR
title: Gunner seats and vehicle weapons: MountedTurret sections, aimers (rate-limited, clamped, aimed at the camera ray's hit), fire points; building turrets as seat-only vehicles; the in-vehicle aim/lock-on call
milestone: M2
value: 5
phase: build
status: done
deps: VSEAT
files: engine/crates/bake/src/hardpoints.rs,engine/crates/core/src/vehicle/gun.rs,engine/crates/core/src/sim/guns.rs,engine/crates/game/src/vehicle_pads.rs,engine/crates/game/src/vehicle_class.rs,engine/crates/game/src/walk.rs,engine/crates/manifest/src/lib.rs
declares: rebake=tat1,tat2,common,side,maps,sounds,global-sound
needs:
branch: vtur
resume: landed e69e73a 2026-09-29; building turrets -> VTURB, bot gunners -> VTAI, lock-on -> VLOCK
created: 2026-09-25
---

## Why
Vehicle weapons and gunner seats (the skiff's three guns, the AT-ST's chin and head guns), and every building turret
on the maps (`com_weap_gunturret` and BF2's `tur_*`), use one seat/aimer machinery.

## Evidence
`analysis/re-vehicles.md` §5 (TURRET1..8 sections = MountedTurret seats with up to 4 weapons each, props routed turret →
camera → aimer → seat), §8 (armedbuilding = the same sections, no pilot seat → a seat-only vehicle), §9 (aimers: defaults
pitch ±45°, yaw ±30°, PitchRate 1.0, YawRate 2.0 rad/s; rate-limited toward the 512 m camera ray's hit, clamped to the
limits; BF1 magnetism runs in any seat whose weapon has LockOnRange > 0; BF2's lock-on is one mechanism on foot and in
every vehicle kind — its button and acquisition loop not yet located).

## Acceptance
Gunner seats enterable by §5's rules; each seat's weapon fires from its FirePointName through the existing weapon code;
aimers per §9; building turrets become seat-only vehicles spawned from their map instances; the in-vehicle aim call
taken and recorded (PLAN §Vehicles & space: the maintainer's "likely" BF2 lock-on for vehicles/flyers — take it as the reversible
default, FEEL_NOTES + STATUS "Calls you may want to reverse"); tests per seat kind.

**Note (planner 2026-09-28):** raised 4 -> 5 so M2 interleaves with the new M1 value-5 cards (M2 done-criterion: drivable with its weapons and turrets). The maintainer's 09-28 coverage note asked for building turrets (NAB1 ships `com_weap_gunturret` instances; PLAN §Vehicles item (5)): they are THIS card's scope (seat-only vehicles, re-vehicles §8) — do not drop them; split them into their own card if the session runs long.

**Note (VAI 2026-09-29):** VAI's bots board pilot seats and drive (`core::ai::ride`), but fire nothing — VAI left the
AI's side of the seat weapons to this card: once a seat has a weapon, `ride::drive` (the pilot) and a gunner seat's
think (today `drive` returns an idle pad for any non-pilot seat) must aim and press FIRE through the same perception /
fairness rules as on foot (`brain::think`'s fire block), and a bot may then also claim gunner seats (`ride::claim`
takes the pilot seat only).
