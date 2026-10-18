---
id: AIMF
title: The soldier's magnet acquisition is the PS2's filtered cone (FUN_0025e098: sphere query, Target team/kind bits, cone score, LOS), not the MountedTurret ray the port copied — it can magnet onto friendlies
milestone: M3
value: 4
phase: build
status: done
deps:
files: engine/crates/core/src/sim/input.rs, engine/crates/core/src/sim/step.rs, engine/crates/core/src/weapon.rs, engine/crates/core/src/unit.rs, engine/crates/game/src/walk.rs (tests), class data plumbing for the soldier model box (bake::soldier aabb -> the unit)
declares:
needs:
branch: aimf
resume: landed 73632e1; follow-ups AIMBX (per-class box), AIMG (a referee that sees the magnet), RETC (reads Sim::reticle_pick)
created: 2026-09-25
---

## Why
The port's soldier magnet (`core::sim::input` `Sim::acquire_magnet`, `engine/crates/core/src/sim/input.rs:118-145`) takes
the nearest capsule the aim ray hits, ANY team — copied from `re-aim-assist.md` §3, whose function turned out to be the
MountedTurret's. On the PS2 the soldier never magnets onto friendlies and picks by a cone, not a ray. Found by VEH0's aim
RE (not an audit job, but a WRONG finding → a card, value ≥ 4). Feel: M3.

## Evidence
`analysis/re-vehicles.md` §9 [O] (verifier claims A1, A2 SUPPORTED / PARTLY-folded): FUN_0025e098 (B1:180429-180892)
— sphere query radius weaponClass+0xd0 (≤ 300, mask 0x1e), team bit (+0xf4 >> (FUN_00221788 + 1)), kind bit
(+0xf8 >> kind), the FUN_00241b98 cone score, an LOS ray (mask 0x48), stored only within LockOnRange + radius, a sticky
current-target path (B1:180600-180642). The correction banner is at the top of `re-aim-assist.md` §3.

## Do / Acceptance
Transcribe FUN_0025e098's acquisition (B1:180429-180892, incl. FUN_00221788's relation encoding and the +0xd0 radius
prop 0x8474ea2c) into `re-aim-assist.md` §3 (replacing the turret text, which moves to a "MountedTurret" subsection),
checks + one verifier; then a build: `acquire_magnet` = the PS2's filtered cone; tests: a friendly in the crosshair is
never the magnet, an enemy just off the ray is; declared `sim` / `arena` moves.

## RE done (2026-09-28, lane 2): `analysis/re-aim-assist.md` §3, [O]
Verifier: 30 SUPPORTED, 2 PARTLY (folded), 0 NOT. 23 BF1 save states were checked in RAM. The spec is §3.3 (the
algorithm), §3.4 (the score), §3.6 (a soldier's bounds and aim point) and §9 (what the port needs).

## Build plan (one card; split the plumbing off if it grows)
1. **Weapon class** (`core::weapon`):
   - Add `lock_on_angle` (rad: LockOnAngle × π/180, or AutoAimSize × 0.072), `extreme_range`, `target_team` (bits:
     enemy 1 / neutral 2 / friend 4; default 1) and `target_kind` (person 1 / animal 2 / droid 4 / vehicle 8 /
     building 0x10; default 0x1f).
   - extreme_range: the ODF's ExtremeRange; else, when OrdnanceName comes AFTER LockOnRange (every BF1 blaster), the
     ordnance's Velocity × LifeSpan (rifle 120); keep the PS2's `max(ExtremeRange, LockOnRange)` clamp where it applies.
     `PropSource` has no order, so either add one or state the rule. BF2 classes use the same rule PROVISIONALLY:
     B2 FUN_00328fb0 carries the same hash (B2:320529), but its OrdnanceName default was not read.
   - The `bf1_rifle()` / `bf2_rifle()` fixtures take the disc's values: LockOnAngle 1°, TargetVehicle 0.
2. **Unit bounds** (`core::unit` plus class data): the soldier model's box.
   - ctr = feet + box centre, r = half the box diagonal. The source is the bake's `BakedSoldier::aabb`, carried to the
     unit's class; the fallback is the stormtrooper's box, ≈ (0, 1.05, 0) with r 1.42, PROVISIONAL.
   - The aim point is feet + (0, 1.4 / 0.6 / 0.2, 0) by stance.
3. **Aim origin**: the PS2's is 2.104 m above the feet (standing), along the crosshair direction. Pick the port's
   equivalent: the camera pivot or aim frame. The muzzle's 1.425 m is not the PS2's; note the choice.
4. **`Sim::acquire_magnet`** follows §3.3 exactly:
   - the sticky target first, its cone widened by π/4 while FIRE is held; it is dropped if it leaves that cone, fails
     eligibility or loses LOS;
   - then the candidates: live units, not self, not inside the shooter's own box, whose kind bit passes;
   - the tournament: `if valid && !pass { skip }`; accept on `0 <= s < best` plus a clear LOS, where LOS is
     `perception::los(pos, aim − r·dir_to_aim)` and the score is `t + 16·d²`;
   - store the magnet only when it passes the filter and lies within LockOnRange + r.
   - The iteration order is `alive_units()` order (the PS2's list order is [H]). Relation: same team = friend, else
     enemy.
   - Also keep the reticle pick and its relation (the PS2's K+0x30 / K+0x40) in the sim, e.g. a `reticle_pick[slot]`,
     for card RETC. Do not hash it until something reads it.
5. **Order** (`sim::step` phase 2): apply the assist with last tick's magnet, THEN acquire, as on the PS2 (§3.1).
6. **Tests**:
   - a friend on the crosshair is never the magnet but is the reticle pick with rel 1;
   - an enemy 1 m off the ray at 20 m is the magnet;
   - an enemy at LockOnRange + r + 1 is picked (within ExtremeRange) but is not the magnet;
   - the π/4 sticky hold while firing;
   - the order-dependent friend shadowing, in the port's fixed order;
   - an enemy behind a wall (LOS) is not picked.
   - Update `walk.rs` `acquisition_respects_ray_and_range`: the 15 m-off trooper stays out; the 80 m one is outside
     LockOnRange + r.
   - `fire_magnetism_pulls_yaw…` must still pull.
7. **Gates**: `sim` moves (the aim replay hash, and any log with a player firing near a unit). The sweep and arena are
   AI-only and must NOT move; if they do, stop and find out why. Remove or keep the magnet soldier's pink tint in
   `game::soldier_vis`, which is not PS2 (the PS2 tints the reticle, card RETC), and say which.
