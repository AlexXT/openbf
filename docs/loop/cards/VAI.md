---
id: VAI
title: Vehicle AI: bots board (the PS2's enter request), drive with RE_LOG's vehicle nav constants, fire the seat weapons, dismount
milestone: M2
value: 5
phase: build
status: done
deps: VSEAT,VPAD
files: engine/crates/core/src/ai/ride.rs,engine/crates/core/src/ai/brain.rs,engine/crates/core/src/ai/route.rs,engine/crates/core/src/ai/state.rs,engine/crates/core/src/sim/seats.rs,engine/crates/core/src/vehicle.rs,engine/crates/game/src/vehicle_class.rs
declares: sweep
needs:
branch: vai
resume: landed f56e5ed9 2026-09-29
created: 2026-09-25
---

## Why
M2 is done only when AI drives every vehicle class a map ships.

## Evidence
`analysis/re-vehicles.md` §5 (AI boarding: B1 FUN_00244848 → FUN_0023c418 queues an enter request consumed through the
soldier's Enter slot; BF2 AI commands 0x13/0x14 in FUN_0027bae0 / FUN_0027caf0), §10 (RE_LOG's vehicle nav constants:
hover arrival radius ForwardSpeed × 0.5, others 4.0 m; look-ahead |velocity| for hovers, 20 m for walkers; flyers 3-D;
AISizeType masks), §9 (the AI's Target filter setter FUN_0023d1f8). Design: PLAN P7 — `ai::think` dispatches on
`Unit.mount`; boarding decisions belong to openbf's own AI (PLAN §AI).

## Acceptance
Bots board free vehicles near their objective, drive them along the planning graph with the vehicle masks, fire the
seat weapons at targets, dismount at the objective / when the vehicle dies; TAT1 and one BF2 map show AI-driven hovers in
a headless battle; the sweep's footer and the stuck census reported (declared `sweep`).

**Note (planner 2026-09-28):** raised 4 -> 5 so M2 keeps two value-5 cards beside VTUR (M2 done-criterion: AI-driven). VSEAT landed (`Sim::enter_vehicle`); HOVX left the AI boost branch (re-hover §5.1 I5) to this card.

**Landed (VAI 2026-09-29, main f56e5ed; report `reports/20260929_0134_card_VAI.md`):** bots board (a claim, then the
instant Enter through CanEnter's pilot seat in step 1c), drive their route as the vehicle's AISizeType with the hover
arrival radius ForwardSpeed x 0.5, and get out at the post, when stuck (3 windows), when their vehicle has no route, or
in an undrivable class. Riders die with the vehicle (VSEAT/VDMG). TAT1: 74 boards / 10,267 drive-ticks; BF2 KAS2: 156 /
51,201 (60,000-tick sweep). NOT done here: **firing the seat weapons** — no seat has a weapon until VTUR, so the AI side
of it is written into VTUR's card (note 2026-09-29). Flyers and walkers are not driven (VFLY / VWLK have no integrator).
