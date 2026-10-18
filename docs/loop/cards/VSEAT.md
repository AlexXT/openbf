---
id: VSEAT
title: Enter, drive, exit — the first drivable vehicle: ACTION + the PS2's aim-cone entry, the driver seat's pad drives the skiff, the vehicle camera, the skiff drawn at its transform, exit at the PS2's 9 points (TAT1)
milestone: M2
value: 5
phase: build
status: done
deps: HOVR,VTEAM,VDRAW,HCON
files: engine/crates/core/src/sim/seats.rs,engine/crates/core/src/sim/seats/tests.rs,engine/crates/core/src/sim/step.rs,engine/crates/core/src/vehicle.rs,engine/crates/core/src/vehicle/hover.rs,engine/crates/game/src/walk.rs,engine/crates/game/src/ride.rs,engine/crates/game/src/vehicle_pads.rs,engine/crates/game/src/vehicle_class.rs,engine/crates/game/src/main/headless.rs,engine/crates/game/examples/vehicle_ride.rs,engine/CRATES.md
declares:
needs:
branch: vseat
resume: landed 2c5521fb 2026-09-28
created: 2026-09-25
---

## Why
M2's first player-visible result: walk up to a vehicle, get in, drive it, get out.

## Evidence
`analysis/re-vehicles.md` §5: the use input gathers objects within the soldier's radius + 0.5 and takes the best score
t + 16·perp² inside the aim cone (0 ≤ t ≤ 5.0 + target radius, perp² ≤ (radius + t)²); the first free PilotType-2 seat
in ring order (the driver first); instant enter and exit; exit placement = 9 candidates at the hull radius (behind,
left, right, diagonals, front, centre), each with a clear path, a 5 m ray up and a 10 m (BF1) / 15 m (BF2) ground ray.
§3.2 item 9: throttle / strafe / turn inputs.

## Acceptance
`buttons::ACTION` enters and exits by §5's rules (a player takes over an AI-held seat); the mounted soldier is hidden
(or posed at PilotPosition) and frozen; the driver's pad drives HOVR's integrator; the vehicle camera from the seat's
TrackCenter / TrackOffset / EyePointOffset / TiltValue / ThirdPersonFOV (PS2 values, Hor+); the skiff drawn rigid at
its transform. A skiff placed at a TAT1 pad: a headless replay walks the player to it, enters, drives 200 m, exits at a
§5 point; a windowed smoke run. FEEL_NOTES lines for each feel choice (stick mapping, camera). Replays and the sweep
unchanged (no bot enters).

**Planner note (2026-09-26):** drawing the vehicles at their transforms is split out as VDRAW (value 5, no deps) and their
solid hulls as VHULL; VSEAT adds VDRAW to its deps and keeps enter / drive / exit / the vehicle camera. CanEnter's rules
(VTEAM report: neutral → anyone; else an AI needs the same team, a player a friendly relation, B1 FUN_0020b1b0) are VSEAT's.

**Note (HOVR audit, 2026-09-28):** when filling HoverInput for a player, follow FUN_002b0548. The full-stick timer E+0x360 tests the raw +-1-clamped right stick (turn^2 + pitch^2 >= 0.9) BEFORE FUN_002b4598's aim shaping and the unit-circle normalisation, and FUN_002f4a50 zeroes it while nobody human drives. The port's ramp_stick_timer tests the post-shaping HoverInput and keeps a stale timer. FUN_003eef38 is the mouse flag: 0 for a pad, so a pad player always takes the timer branch. See HCON for the contact / spawn-pop fixes.
