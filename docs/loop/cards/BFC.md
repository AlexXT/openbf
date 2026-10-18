---
id: BFC
title: The port draws every face double-sided; the PS2 culls BF1's back faces — a camera inside or behind a closed mesh sees a dark wall in the port and nothing on the
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
created: 2026-09-25 (from QUEUE.md)
defer: post-alpha
acceptance: 
---

## Why / Evidence / Acceptance
See the original entry below; the first session to work this card rewrites this section into short Why / Evidence / Acceptance lines and fills `files:` and `declares:`.

## Original QUEUE.md entry (verbatim, 2026-09-25)
BFC. [TODO — **NOTE 2026-09-25 (PXC batch 3, FINDINGS F33): BF1 BES2 `cp0_cp4_postA_v2`'s port frame is one flat colour (21 distinct RGB values) where the PS2 sees a room — the camera sits inside a mesh (`engine/artifacts/pcsx2/ref/bf1_bes2/pairs/cp0_cp4_postA_v2.png`).** ADDED 2026-09-25 (lane 1) by PXC batch 2 (FINDINGS F10, F23); **[RE] FIRST**; FREE; the build touches `render/src/pipelines.rs` (shared with every render entry — build it when none is PARKED or IN PROGRESS) and maybe the bake's winding] **The port draws every face double-sided; the PS2 culls BF1's back faces — a camera inside or behind a closed mesh sees a dark wall in the port and nothing on the PS2** [RE] — evidence `engine/artifacts/pcsx2/ref/bf1_kam1/pairs/cp0_cp2_beta1_spawn.png` (~75 % of the port frame one dark surface; the PS2 sees the CP floor), `…/bf1_kas2/pairs/cp3_cp3_cityhall_v3.png`. The port: `render/src/pipelines.rs` 528–529 `cull_mode: None` ("No culling for v0: baked map geometry has mixed winding"), the only `cull_mode` under `engine/crates`. The disc: `analysis/re-material-flags.md` 125 — BF2's segment flag 0x10000 suppresses prim 0x800800 (BF2 decomp 723485–723487), [H] "two-sided/no-cull", on every BF2 word and no BF1 word. RE (§5): the PS2's cull rule per game (the VU1 microcode or GS setup that culls, what 0x800800 is, BF1's side), the disc strips' winding, which meshes are authored two-sided; then measure the port's "mixed winding" (how many submeshes flip). GATE: a dated § in `analysis/re-material-flags.md` (§5, [O], verifier) + a build entry (cull per draw; winding fixed in the bake if needed) with the two pairs to beat and its golden list. Files: `analysis/re-material-flags.md`, `analysis/RE_LOG.md`, scratch `engine/artifacts/lead_tmp/re/bfc/`.

## Note (PXWA, 2026-09-29, lane 2): two more cases from the PCSX2 walkabout
- BF1 YAV1 `tflank` b090/b270: the chase camera pulled 2.1-2.6 m in; the port frame at that eye is one flat olive fill
  (inside a mesh) where the PS2 sees out — `engine/artifacts/pcsx2/ref/bf1_yav1/walk/pairs/cp4_tflank__sheet.png`.
- BF1 BES2 cp7 b000/b090: through the corridor's end the PS2 sees the fog colour (52,53,62) past
  `bes2_bldg_16m_outerwall`'s back face; the port draws the back face brown (66,57,49) —
  `bf1_bes2/walk/pairs/cp3_cp7__sheet.png`.

## Planner note (2026-09-30, M-ALPHA)
`defer: post-alpha` — not needed for the alpha's six showcase maps or its checklist. Value 3 (kept).

`acceptance: needed` (planner 2026-09-30): no measurable acceptance yet (a parity delta, a leak-census count, a sim replay hash or a pixel test). Write one before the card is picked.

## Acceptance (planner 2026-09-30 17:30)
At the three recorded poses — BF1 BES2 `cp0_cp4_postA_v2`, BF1 YAV1 `cp4_tflank` b090/b270, BF1 BES2 `cp3_cp7` b000/b090 — the port frame is no longer one flat fill (≥ 1,000 distinct RGB values where the PS2's has them) and the named box (BES2 cp7's corridor end: PS2 fog (52,53,62), port (66,57,49)) lands within ±8 levels per channel of the PS2's; BF1 parity mean block error not worse. KAMR's floor census may add cases.

## Note (KAMR RE, 2026-09-30): a stored pose that shows it
BF1 KAM1 `cp0_cp2_beta1_spawn` (eye (-3.849, 63.229, 424.033), yaw 0): the PS2 camera sits inside a `kam_prop_pillar`
and sees through it; the port fills 80 % of the frame with the pillar's back faces (block error 39.4, the map's worst
pair). `lead_tmp/kamr/sbs/bf1_kam1/cp0_cp2_beta1_spawn.png`.

## Note (card B1DK's RE, 2026-10-05)
`ref/bf1_kam1/cp0_cp2_beta1_spawn` is the best PS2 referee pose for BF1's post light (a friendly post 18 m ahead, the
deck washed green; knock-out frames in `lead_tmp/b1dk/ps2/frames/k1__*.png`) and the port cannot use it: about 85 % of
its frame there is the inside of a `kam_prop_pillar` the PS2 culls. Fixing this card gives card B1LL its referee.
