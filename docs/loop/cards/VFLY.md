---
id: VFLY
title: The flyer, transcribed: the 5-state machine (landed/takeoff/flying/landing/4) and the Min/Mid/Max speed law, BF1 FUN_001d9180 and BF2 FUN_00202d08 — then a build card (BF1 flyers, faithful per map)
milestone: M2
value: 3
phase: re
status: todo
deps:
files:
declares:
needs:
branch:
resume:
created: 2026-09-25
defer: post-alpha
value_pre_alpha: 4
---

## Why
15 flyer classes; PLAN keeps flyers faithful per source map (BF1's flyable craft on BF1 maps).

## Evidence
`analysis/re-vehicles.md` §4.2: B1 FUN_001d9180 / B2 FUN_00202d08 — 5 states (landed / takeoff with a 1024 m ray and
fraction += dt/TakeoffTime / flying / landing, touchdown normal·up > 0.9375 / 4), target speed Mid ± t·(Max|Min − Mid)
clamped to ±Acceleraton·dt of the current forward velocity, first-order rate filters, the Lua fly-height band
(±200 m default); BF2 caps and boost; carrier = flyer + cargo.

## Do / Acceptance
Transcribe the BF1 integrator (and the BF2 differences) into `analysis/re-flyer.md` with the §RE checks and one
verifier — resolve Ghidra's case fall-through with the jump table, what state 4 is, and the terrain contact; then write
the build card.

**Planner note (2026-09-26):** raised 3 → 4 — VPAD now spawns X-wings / TIEs on TAT1 and every flyer pad; after HVRE this integrator is next.

**Note (VDRAW, 2026-09-28):** with parked vehicles now drawn, the BF1 GEO1 LAAT sits with its skids in the ground
(`lead_tmp/vdraw/geo1_v2.png`): VPAD's spawn height is `ground_under` (PROVISIONAL) at the model origin. The flyer's
landed rest height (landing gear / the landed state's ground ray) belongs in this card's transcription.

## Planner note (2026-09-30, M-ALPHA)
`defer: post-alpha` — flyers come right after the alpha (ground vehicles meet 'drivable vehicles'); first M2 card to raise then. Value 4 → 3 until the alpha (restore from `value_pre_alpha`).

## Note (VREST RE, 2026-10-07): the landed state, LandedHeight, and two lines for the transcription
`analysis/re-vehicle-rest.md` §2-§3 [O]: a flyer's constructor (BF2's `EntityFlyerInitAsLanded` body) adds LandedHeight
(authored, else minus the model's vis-box min y; table of every class in §2) to its spawn point; landed, its velocity
is zeroed every tick and its body is a BUILDING-layer object (takeoff moves it to the moving layer). The instance keeps
its own copy (B1 entity+0x628, B2 +0x66c), which takeoff and landing read: BF1's takeoff too holds the lift fraction
until the height is over LandedHeight + TakeoffHeight (B1:85668-85684; re-vehicles §4.2 said BF2 only), and the landing
ends where the origin's height first drops under LandedHeight with the body upright (B1:85703-85793) - no snap. Those
two readings are the lead's, not verifier-checked. Card VREST builds the parked height; this card starts from it.
