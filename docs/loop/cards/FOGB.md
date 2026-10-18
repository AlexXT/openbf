---
id: FOGB
title: BF1's near models took no fog on the PS2 (KAS2, patched 0..200 m ramp: bamboo 10-40 m and the soldier unchanged, only the far water fogged) - re-fog §0.3 reads every opaque segment fogged in both scenes; and the PS2 fogs by planar view depth, the port by radial distance
milestone: M1
value: 3
phase: re
status: todo
deps:
files:
declares:
needs:
branch:
resume:
created: 2026-09-28
defer: post-alpha
---

## Why
`analysis/re-fog.md` §0.3 / §2.2 read BF1's segment draw `FUN_004d0108` as fogging every opaque model segment in BOTH
scenes (no layer test), and the port fogs every draw from its near distance. The PS2 itself disagrees on BF1 KAS2. With the
fog ramp patched to 0..200 m in EE RAM:
- the bamboo stalks (10–40 m, f = 0.05–0.2) changed by exactly (0,0,0);
- the soldier's box changed by −0.13 (noise);
- only the far-water band fogged.

On BF1 maps with short ramps (END1/YAV1 90..350, BES1 80..430, BES2 80.., HOT1 100..1000), the port may be fogging near
models the PS2 leaves clear. Separately, the PS2's f uses PLANAR view depth (the fog matrix in RAM = fwd/(far−near)),
while the port's `view_dist` is radial (`shader.wgsl:143`). That puts more fog at the screen edges in the port.

## Evidence
- `analysis/re-fog.md` §9.3 ([O]).
- Frames: `engine/artifacts/lead_tmp/re/kas2-fog/ps2exp/frames/bf1_kas2_v1sea__{ctl,r200,r400,r800}.png`.
- States: `ps2exp/mkstates.py` (keeps the recipe; the .p2s copies were deleted, regenerate them from
  `pcsx2/ref/states/bf1_kas2_cp0_cp2_beachhead_v1_sea.p2s`).
- RAM matrix: 0x6ea8b0 in that state.

## Acceptance
- Name the gate that skips near models in BF1: the opacity test (*(decompiler detail omitted)*, `seg+0x2c`, 0x400000), a near-scene
  rule, the fog matrix per scene, or the VU program. Use RE plus one more patched-state pair on a map whose near props
  are plainly opaque (e.g. BF1 HOT1 or END1, whose short ramps make the test easy).
- Then a build card (or a recorded no-change) for the port's BF1 near-model fog, and for radial vs planar distance.
- BF2's twin question is card FOGN.

## Notes
Created 2026-09-28 by KASF (lane 2). FOGN (value 3) is the BF2 half: far-scene-only fog. Consider merging if the answer is
"far scene only" in both games.

## Note (REDB, 2026-09-29, lane 2)
REDB settles HOW MUCH fog (planar view depth, the disc's `fogspread` table, a lerp on the stored bytes —
`analysis/re-fog-amount.md` §8). This card keeps WHICH draws are fogged in BF1's near scene. The segment draw's fog pass binds
the same texture (B1 594618).

## Planner note (2026-09-30, M-ALPHA)
`defer: post-alpha` — not needed for the alpha's six showcase maps or its checklist. Value 3 (kept).
