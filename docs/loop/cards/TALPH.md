---
id: TALPH
title: Terrain texture LAYERS: the port draws only LTEX[0]; the PS2 blends every painted layer by ALPH's 4-bit over-alphas — BF2 KAS2's sand beaches, BF1 KAS1's grass, KAS2's pale slopes (KTEX merged)
milestone: M1
value: 5
phase: build
status: done
deps:
files: engine/crates/mungefs/src/terrain.rs,engine/crates/bake/src/terrain_glb.rs,engine/crates/bake/src/map.rs,engine/crates/manifest/src/lib.rs,engine/crates/game/src/assets.rs,engine/crates/game/src/viewer/scene.rs,engine/crates/render/src/draw.rs,engine/crates/render/src/shader.wgsl,engine/CRATES.md
declares: goldens=maps/bf1_bes2.png,maps/bf1_end1.png,maps/bf1_geo1.png,maps/bf1_hot1.png,maps/bf1_kam1.png,maps/bf1_kas1.png,maps/bf1_kas2.png,maps/bf1_nab1.png,maps/bf1_nab2.png,maps/bf1_rhn1.png,maps/bf1_rhn2.png,maps/bf1_yav1.png,maps/bf1_yav2.png,maps/bf2_dag1.png,maps/bf2_end1.png,maps/bf2_fel1.png,maps/bf2_geo1.png,maps/bf2_hot1.png,maps/bf2_kas2.png,maps/bf2_nab2.png,maps/bf2_pol1.png,maps/bf2_yav1.png,maps/tat1.png,maps/tat2.png,tat1_overview.png,tat1_spawn.png,tat1_hud.png; rebake=tat1,tat2,maps
needs:
branch: talph
resume: LANDED 23238e1 (2026-09-28 lane 1, report reports/20260928_1320_land_TALPH.md): 28 goldens viewed and committed; PS2-pose checks improved on RHN1/2, HOT1 and BF2 KAS2 shore.
created: 2026-09-25
---

## Build (RE done 2026-09-28, lane 2): `analysis/re-terrain-layers.md` [O], §8
**Why.** The PS2 draws every painted layer. The port draws LTEX[0] only, so it misses:
- BF1 KAS2's pale slope (F6): PS2 (127,114,86), the layer model (116,103,81), the port (46,45,27);
- KAS1's green ground (F17): PS2 (59,47,26), the model (62,50,26);
- BF2 KAS2's beaches: kas2_main_2 along every shore (`lead_tmp/re/talph/topdown_bf2_kas2.png`).

**What to build** (the doc's §1–§4 are the spec; the scripts in `lead_tmp/re/talph/` are the reference decoders):
1. `mungefs::terrain` decodes the tern-level `ALPH`.
   - Walk it per PTCH in file order; a popcount-2+ patch takes layers × 45 decoded bytes; 1-layer patches → 0xee.
   - Decode the PTCH-level `ALPH` / `COLR` too.
   - Test on the census: the walk ends at the chunk end on all 27 terrains; 122 replacement patches decode to their sizes.
2. **Bake**, per render patch, 81 own vertices: the per-patch weights differ across 0–3.3 % of shared edges.
   - Per layer: alpha = table[nibble] / 128.
   - The triangles that survive the skip rule (any 0xf or all-0 → dropped). BF2's top layer keeps all of them, with
     0xf → alpha 0; BF1 applies the rule to every layer.
   - The PS2 diagonal: (x,z)–(x+1,z+1) on even z, the other on odd z.
   - Replacement patches use their own nibble / COLR per `POSI` vertex, and treat AXIS ≥ 3 as top-down.
   - Keep the n×n grid block that `game::assets::terrain_grid_size` reads, or move the sim's source. The sim must not
     move.
3. **Render.** Layers in ascending LTEX index, each T_k × COLR (TERL's `LightPath::Terrain`), "over" the previous by its
   interpolated alpha.
   - S/T per layer: k = round(gridUnit / SCAL · 2048) / 2048 repeats per GRID cell.
   - AXIS: ≥ 6 negates S, then ≥ 3 swaps S/T, then 1 = T from the height, 2 = S from the height.
   - x / z are grid indices: world x / gu + n/2, world z / gu + n/2 − 1.
   - ROTN is unused.
4. **Where the PS2 draws no terrain** — mask-0 patches and vertices whose layers cover < 1 — keep today's LTEX[0] × COLR
   as a documented backstop. Card TMSK removes it after REFL.

**Acceptance.** Each pair box reads like the PS2 after the build, measured by rerendering the port at the pose:
- `bf1_kas2/pairs/cp4_cp11_dock_two_spawn` (540,110,630,150): PS2 (127,114,86);
- `bf1_kas1/pairs/cp2_cp2_eastharbor_spawn` (0,260,150,330): PS2 (59,47,26);
- a BF2 KAS2 beach pose captured with the PCSX2 scripts (none of the five library poses shows the shore);
- UV orientation checked on kas2_cobbles_02 (AXIS 0) and on a KAS1 AXIS 6 layer.
- Everything else in the declared goldens moves by the layers only; view every sheet.
- Hoth.
  - BF2 HOT1's TERL boxes paint hoth_main_1 alone, so they must not move beyond the triangulation.
  - BF1 HOT1's spawn boxes show hoth_main_4 (mean 214) over hoth_main_1. The model predicts 214 against today's 228 and
    the PS2's 197 / 191; the rest of that gap is open.

## Planner re-scope (2026-09-26) — value 2 → 5, KTEX merged in
**Why now:** the maintainer (INBOX 2026-09-25) asked whether BF2 Kashyyyk's sand beaches are the terrain layers. They are [D]:
the bake keeps ONE layer (`mungefs/src/terrain.rs` "LTEX[0] as its single base layer"; `terrain_kas2.glb` has one
material, `kas2_main_1`), and the planner's census of `PCHS -> PTCH -> INFO` masks (script below;
`reports/20260926_1100_planner.md`) shows what the port throws away:
- **BF2 KAS2:** LTEX = [kas2_main_1 (mean RGB 79,80,63 — grass/dirt), kas2_main_3 (103,92,70 — sandy brown),
  **kas2_main_2 (135,131,120 — pale sand)**]; layer masks: layer 0 in 174 of 256 patches, layer 1 in 160, **layer 2
  (the pale sand) in 90**. Script: `engine/artifacts/lead_tmp/planner/layer_census_20260926.py`.
- **BF1 KAS1:** LTEX[0] = kas1_main_2 (140,133,114 — the grey sand the port draws everywhere, F17), then kas1_main_3,
  **kas1_main_1 (66,88,58 — green)** in 51 of 73 live patches, kas1_main_4, kas1_main_7 (both green).
- **BF1 KAS2:** kas2_main_1 (56,60,36 — the port's dark grass) in 555 of 621 patches; kas2_main_2 (124,110,88 — pale
  sand, F6's slope behind Dock Two) in 60; kas2_cobbles_02 in 18; kas2_main_3 in 102.
So F6, F17, most of F14 / F23's ground colour, the maintainer's beaches and TERL's Hoth residual are ONE missing system.
Walk-through rule (the maintainer): the layers are decoration on the same heightfield — no collision change.

## Acceptance (planner 2026-09-26; replaces the one below)
1. **RE, bounded (one session):** the ALPH RLE stream's format (FUN_0056e898 / BF1 twin), layer draw order, the
   per-layer UV projection (SCAL / AXIS / ROTN — AXIS non-zero on layers ≥ 2 of nine maps), and the `PTCH`
   replacement's ALPH / COLR. TERL §6 [O] already has the nibble table and the skip rule; add the rest to
   `analysis/re-terrain-lighting.md` (or a new `re-terrain-layers.md`) through the §5 verifier.
2. **KOC first check:** does the ALPH skip rule (a weight 0xf, or all three 0) remove BF1 KAM1's y = 0 sea terrain?
   If so, KOC is solved by this card's build (note it on KOC).
3. **Build card** (or this card → build): bake every layer's texture + per-vertex weights; draw them blended
   (ALPHA 0x44 in the RE) under TERL's `LightPath::Terrain`. Pairs that must read like the PS2 after it:
   `bf1_kas2/pairs/cp4_cp11_dock_two_spawn.png` box (540,110,630,150) PS2 (126,114,85) (port 46,45,27);
   `bf1_kas1/pairs/cp2_cp2_eastharbor_*` (green ground, not grey sand); BF2 KAS2's shore (capture a beach pose if
   none of `bf2_kas2/pairs/` shows it — they are all from one platform); BF2 HOT1 spawn L210 (TERL today 223).

## Why (original)
The terrain's authored alpha (texture-layer blend weights or transparency) is unread; wherever the disc uses it the port's
ground differs. May bear on KTEX (KAS2's pale sand slope).

## Evidence
analysis/PLAN_LOG_2026.md §ARCHIVED 2026-09-25 HANDOFF-OPUS.md HOLD 10(d); `analysis/a2-bake-formats.md` (terrain).
- **NOTE 2026-09-25 (TERL RE, `analysis/re-terrain-lighting.md` §6 [O]).** `ALPH` holds the LAYER WEIGHTS, and the ELF's
  path is now known:
  - The PCHS parser FUN_0056bbf0 stores ALPH's per-patch pointer at patch entry +0 (703310 / 703323). It is an RLE
    stream walked for (grid+1)·(grid/2+1)·layers bytes.
  - FUN_00572c98 decompresses it (FUN_0056e898). FUN_0056c9d8 reads each vertex's weight as a 4-bit nibble, 2 per byte.
  - DAT_0060c1a0 (BF1 DAT_00522a20, same 16 bytes) maps the nibble to the GS vertex alpha:
    {0, 9, 18, 27, 36, 45, 54, 64, 73, 82, 91, 100, 109, 118, 128, 0}. A triangle is skipped when any weight is 0xf or
    all three are 0.
  - A single-layer patch uses 0xee (opaque). Layers are blended with ALPHA 0x44.
  - A patch with a `PTCH` replacement (entry +0x10) takes its own ALPH / COLR (FUN_0056dcd8; *(decompiler detail omitted)* / *(decompiler detail omitted)*).
- **Why it matters:** the Hoth residual after TERL is this blend. At both Hoth spawns the patches are masks 0x5 / 0x7;
  hoth_main_2 has mean 177 and hoth_main_4 214, against hoth_main_1's 225.

## Acceptance
The `ALPH` layout decoded against the ELF's terrain loader ([O]), a census of maps that author it, and a build card.
