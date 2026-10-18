---
id: VTAI
title: Bots use seat guns: a bot pilot fires its seat's weapon and a bot may claim a gunner seat; its look/aim from brain's target through the same perception / fairness rules as on foot
milestone: M2
value: 5
phase: build
status: done
deps: VTUR
files: engine/crates/core/src/ai/gunner.rs,engine/crates/core/src/ai/ride.rs,engine/crates/core/src/sim/guns.rs,engine/crates/core/src/perception.rs,engine/crates/core/src/ai/brain.rs
declares: sweep
needs:
branch: vtai
resume: landed 744f8f8; turrets → VTAT
created: 2026-09-29
---
## Why
VAI's bots board and drive but fire nothing; VTUR (2026-09-29) arms only HUMAN riders (core::sim::guns::Sim::seat_gun returns for a bot). A vehicle full of bots that never shoot is the biggest remaining AI gap in M2.

## Evidence
VAI note on VTUR: ride::drive and a gunner seat's think must aim and press FIRE through brain::think's fire block (perception + the fairness assert in step 4); ride::claim takes the pilot seat only. re-vehicles §5 (AI boarding: BF1 FUN_00244848 → FUN_0023c418; BF2 AI commands 0x13/0x14) — the PS2's AI gunner logic is not yet read [H].

## Acceptance
A bot in a seat with a gun turns its look toward its perceived target (the aimer then follows as for a human), presses FIRE under the on-foot fairness rule (asserted where the seat shot is made), and ride::claim may take a free gunner seat of a friendly vehicle; the sweep's per-map counters show seat shots; declares sweep/arena/sim with the reason.

- VTURB (2026-09-29): BF2 UTA1's two `uta1_bldg_turret` (armedbuilding, instances Team 2) author `PilotType = self` on the body and TURRET1 — automatic turrets, not boardable, so VTURB spawns them as Turret vehicles with no boardable seat and nothing fires them. A self-piloted seat's gunner (the MountedTurret tick B1 FUN_00223850 is its acquisition, re-vehicles §9) belongs here.
