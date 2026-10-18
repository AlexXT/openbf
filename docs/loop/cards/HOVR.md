---
id: HOVR
title: core::vehicle::hover — the BF2 hover integrator (spring points, lift, gravity, velocity-matching thrust, filtered yaw, world contact) on every hover/treaded class; the TAT1 skiff on a scripted pad
milestone: M2
value: 5
phase: build
status: done
deps: VENT,HVRE,VCLS
files: core/src/vehicle/hover.rs,core/src/vehicle.rs,core/src/sim/vehicles.rs,core/src/sim/step.rs,core/src/sim/state.rs,core/src/collision/walls.rs,core/src/fmath.rs,game/src/vehicle_class.rs,game/src/vehicle_pads.rs,game/examples/hover_drive.rs
declares: sweep
needs:
branch: hovr
resume: LANDED dae70c5 on 2026-09-28 (report reports/20260928_0923_land_HOVR.md). Follow-ups: HOVX (hull prims, water float, boost, bike jump), HCPW (contact sidedness / cp4 wedge).
created: 2026-09-25
---

## Why
The first vehicle physics: HVRE's transcription of the BF2 hover integrator as `core` code, deterministic, on every
hover/treaded class (PLAN locks BF2 ground handling; BF1-disc classes lose only their stick-based turn blend and the
strafe→turn fold — FEEL_NOTES line).

## Evidence
`analysis/re-hover.md` (HVRE) and `analysis/re-vehicles.md` §3.

## Acceptance
`core::vehicle::hover` stepped from `Sim::step` for every live hover (none exists in a battle yet); unit tests pin each
equation against hand numbers from re-hover.md (gravity weighting with / without springs, lift, the spring force,
accelStep / tractionStep, the speed-based turn blend, boost off when BoostSpeed = 0); a scratch example drives the TAT1
skiff on a scripted pad for 20 s: it settles at its spring height, reaches ForwardSpeed 16 m/s, turns at the rate
re-hover.md predicts, stays on the terrain and against walls through `CollisionWorld`; two runs hash equal;
`--sim-hash 100`, the replays and the sweep unchanged. Constants not yet in re-hover.md are PROVISIONAL.

**Note (VPAD, 2026-09-26):** pads now spawn every class into `Sim::vehicles` (tick stage 7, `core::vehicle_pad`); `VehicleSpec::class` indexes `game::vehicle_pads::PadBuild::classes`, which `install` returns and nothing keeps yet — keep it (e.g. on `WalkSim`) to find each vehicle's `HoverClass`. Spawn height is PROVISIONAL `ground_under` at the pad.

**Note (HVRE, 2026-09-26): `analysis/re-hover.md` [O] is the spec. Build from its §6 / §4 / §7 pseudo-code, not from
re-vehicles §3.2.**
1. **Order** per 1/30 tick: input → drive (§6) → spring points (§4) → contacts (§7). g reaches the drive as
   1 − dt = 0.9667 (PS2 RAM agrees).
2. **Only the spring branch (§6.5) is needed.** Every shipped hover has 3-6 points; the no-spring branch can stay
   PROVISIONAL.
3. **Tilt feedback.** Build q from the TILTED matrix and re-apply the new tilt on top (§6.11-6.16). The authored angles
   (~0.01) are tuned for that.
4. **The yaw is a slew:** ω.yaw steps by TurnFilter·dt to its target (not re-vehicles' filter). A human's rate blends
   from PitchRate by the full-stick timer (§5.2: +dt while turn² + pitch² ≥ 0.9, else 0, cap 1).
5. **Springs compress only from contacts:** c = min(depth·(n·up), len)/len, when n·up > 0.5 and n.y > 0.5.
   Depth ≤ len leaves the contact to the spring; deeper → damage, push-out (depth − len) and the rebound (§7 K5).
   - Hull contacts: friction 0.25, restitution 0.125, push-out, **no damage** (K6-K9).
   - `Sim::impact_vehicle` (no caller yet) only for vehicle-vs-vehicle contacts and bottomed spring spheres.
6. The water float (§4 S3), boost + energy, the bike jump (§6.9) and the tread scroll are class-driven. They may come in
   later commits.
7. Also fix `vehicle_class.rs`'s comment "BF2's entry init … not opened line by line [H]": it is FUN_00224368
   (B2:138140-138183), the same shape.
8. FEEL_NOTES line: BF1-disc classes get BF2's speed blend and slew (§8).

**Test numbers** (re-hover §9; a numeric model in `lead_tmp/hvre/skiff_sim.py` agrees):
- skiff at rest: origin 1.94 m above flat ground, a ±0.07 m bob (1.98 m without jitter);
- 0 → 16.0 m/s in 3.31 s; coast-down 7.73 m/s²;
- turn at speed 1.0 rad/s: AI after 0.5 s; pad peaks at 1.2 at 0.6 s, then 1.0 from 1.0 s;
- turning circle radius 16 m at 16 m/s.
