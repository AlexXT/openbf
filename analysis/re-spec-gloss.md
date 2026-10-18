# OPUS RE (2026-09-29, lane 2) - [O] UNAUDITED: the specular pass's gloss mask — why the port's droids are chrome

> **AUDIT 2026-09-29 (loop lane 1, OK — `engine/artifacts/loop/audits.md`).** Re-derived before reading this doc, from
> the B1/B2 decomp, the ELF instructions, the disc (the audit's own `tex_` and `segm` readers) and PCSX2 patched states
> (the audit's own patcher, masks and analysis). Evidence: `engine/artifacts/lead_tmp/audit-dshn/`. **Everything held.**
> - **§1 chain.**
>   - B1: `FUN_004aa9f8` makes the Specular class (vtable 0x5594a0, '18ps2SpecularSegment') from word 0x20, before any
>     other class bit. Its record is set by `FUN_004b27a0`: 0xff00000158, TEST 0x50000, PRIM 0x50, over `specular`.
>     The same values sit in BF1 KAM1 EE RAM.
>   - B2: `FUN_00591a30` sets 0x7c57b0 = 0xff00000158 and, for words with 0x04, 0x7c57c0 = 0xff00000148.
>   - Both games: the base colour is halved to alpha 0x80. TEX0 is TCC 1 / MODULATE in both of B1 `FUN_0049c5c8`'s
>     branches. The palette alpha is (a·0x81)>>8: B1 `FUN_004aebf8`/`FUN_004aecb0`, B2 `FUN_00593558`/`FUN_005935d8`.
> - **§5 census, to the sheet.**
>   - 151 pass sheets (40 BF1 + 111 BF2); 9 more BF2 sheets are 0x04 words, excluded.
>   - 68 lost: 51 drawn by a baked `spec_pass` material, plus 17 first-person weapon sheets baked at 255 with none.
>   - SBD alpha: mean 34.07/255, 0.04 % ≥ 128, max 247.
> - **§2 on the PS2: BF1 KAS2 Dock Two SBD**, luma mean / p95 / p99:
>   - as-is 68.0 / 135.2 / 188.2 (ctl2 67.1 / 131.5 / 187.8);
>   - weight 1.0: 105.7 / 220.1 / 243.9;
>   - no pass: 61.3 / 122.2 / 158.4;
>   - per-pixel weight median 0.127.
> - **New: §5's world sheets, measured on the PS2** [D: a distribution match, not a per-texel registration].
>   - Where: BF1 KAM1, the cp1_cp10 post 2 spawn, the Kamino deck.
>   - The weight (ctl − none)/(full − none): median 0.274, 16.1 % above 0.7, median of the rest 0.233. The rim strips
>     read ≈ 1, the plates scratched mid-grey.
>   - `kam1_bldg_plat1`'s disc alpha: median 0.271, 13.9 %, 0.231.
>   - Picture: `audit-dshn/kam1post2_deck_crop.png`.
> - **§8's BF2 alpha test is inert.**
>   - The pass's vertex alpha is (trunc(spec a × 1/255) + 1) >> 1 (0x58fe00–0x58ff14), so a spec alpha below 255
>     would fail TEST 0x5001b's As ≥ 1.
>   - All 469 BF2 0x80 segments carry spec alpha 255, and the `specular` table's alpha is 255 on both discs.
> - **§6:** CAMB's goldens `86d23b5` touched 0 unit sheets.
> - **§4's BF1 prototype, reproduced** (this doc's scratch renderer; the disc alpha restored and the pixels measured by
>   the audit's own scripts). Luma mean / p95 / p99:
>   - now 67.6 / 214.3 / 242.8;
>   - prototype 32.8 / 104.3 / 137.8;
>   - no pass 27.1 / 70.7 / 119.0.
>   The add is +5.73 (the doc's +5.5); the base does not move. Not redone: BF2's +4.9.
> - **Side finding, outside this doc [D]:** BF1's ShieldEffect (attributed by its RTTI neighbourhood: the reader's
>   caller `FUN_0037bb18` installs vtable 0x549428 among the ShieldEffect RTTI) has its own `specular` records, set by
>   `FUN_00389338` and read by `FUN_003879a8` / `FUN_00388ee8`:
>   - 0x148 = Cs·As + Cd;
>   - 0x4000000068 = Cs/2 + Cd;
>   - 0x8000000168 = Cs + Cd.
>   It is noted on DEKA and FAMB.
> - **Verifier** (one Opus agent, foreground): 7 SUPPORTED, 2 PARTLY (claim wording; BF2 `FUN_0058ed70`'s other gates),
>   0 NOT SUPPORTED.

**Card DSHN** (the maintainer's play session, 2026-09-29): the port draws the super battle droid and the other droids as polished
chrome with hard white bands, where the PS2's are matte grey. Is it the specular second pass, and if so what does the PS2
do differently?

**The answer.** The pass is right and so is its colour; its WEIGHT is wrong. The PS2 adds the specular table texel ×
colour to the frame weighted by the destination alpha Ad (GS ALPHA `Cs·Ad + Cd`, `analysis/re-specular-pass.md` §2).
Ad is what the segment's own base pass wrote into the frame's alpha: the texel's alpha (the palette alpha, rescaled
to GS units at load) × the base colour's alpha 0x80 >> 7, i.e. the texture's alpha channel, which on every droid sheet is
a dark GLOSS MASK (the SBD's averages 34/255). The port bakes those sheets with alpha 255, because its side, vehicle and
map bakes keep a texture's alpha only for a cutout (0x02) or blended (0x04) draw, so `fs_spec` weights the add by 1.0.
Patching the PS2's own record to weight the pass by 1.0 reproduces the port's chrome on the PS2 (§2). The defect is
general: 68 of the 151 textures a specular segment draws lose a real gloss mask in the bake: droids, the dark trooper,
BF2 rebels and AT-AT crew, every first-person weapon sheet, a speeder, and 29 world sheets (§5).

Tags: [V] read (lines / ELF words / disc bytes cited) or measured (files cited); [D] derived; [H] hypothesis. `[O]` = one
Opus pass plus one Opus verifier, no Fable audit. Scratch, scripts, frames and outputs: `engine/artifacts/lead_tmp/dshn/`
(below: `dshn/`). The verifier's claims and verdicts: `dshn/verify/claims.md`, `dshn/verify/VERDICTS.md`.

## 0. The answer in one table

| question | answer | tag |
|---|---|---|
| does the PS2 give droid bodies a specular pass? | yes: BF1 SBD and battle droid are the Specular class (word 0x31, power 4, colour = the sun); BF2 SBD 0x10089 power 50, battle droid power 4, magnaguard 15, colour white. The droideka has none (0x1 / 0x10001) | [V][O] |
| what weights the add | Ad = the base pass's frame alpha = the texel's gloss alpha (disc a → GS (a·129)>>8), not 1.0 | [V][O] |
| what the port does | the sheets are baked at alpha 255 → weight 1.0 → the PS2's own "weight 1" frame, chrome | [V][O] |
| colour, power, table, tex-gen | as `re-specular-pass.md` (not the defect: the prototype with only the alpha restored matches the PS2's add) | [V] measured |
| CAMB's part | none: the unit sheets went chrome with SPEC2 (`c231edb`, 2026-09-25); CAMB's goldens (`86d23b5`) touched no unit sheet | [V] git |
| how much | BF1 KAS2 SBD, one mask: PS2 luma p95/p99 131/189, PS2 at weight 1.0 222/244, port now 217/244, port prototype 104/138 (§2, §4) | [V] measured |

## 1. The chain on the PS2 [V][O]

1. **BF1 base colour.** `FUN_004b2310` (the Specular-class draw, B1 573294) builds *(decompiler detail omitted)* = seg+0x34..0x37 ×
   the draw colour / 255 + 0.5 per byte (573379–573386) and hands `FUN_0039c4f8` two colours: the FIRST (the base
   pass) with every byte halved `(b + 1) >> 1`, alpha included (573415–573416, 573422–573423), the SECOND (the pass)
   raw: seg+0x40, or the sun colour `FUN_00499418` writes (573409–573425). In the sun branch the first colour is taken
   from *(decompiler detail omitted)* (573410–573413) before `FUN_00499418` overwrites *(decompiler detail omitted)* (573414). seg+0x34 =
   `_DAT_0069c420` at load (`FUN_004abde0` 568926) = ff ff ff ff (`FUN_00492598` 555048/555049/555070/555071, the only
   writers — the verifier's whole-ELF store scan agrees, and EE RAM holds ff ff ff ff in all 10 BF1 states and on all 19
   live Specular segments). The machine code truncates (`cvt.w.s`): 255·255/255 + 0.5 → 255, halved → **0x80**.
2. **BF1 pass record.** `DAT_006f4540` = 0xff00000158 (`FUN_004b27a0` 573452; ELF 0x4b27d0 `ori a0,zero,0xff00`, 0x4b27d4
   `dsll a0,a0,24`, 0x4b27d8 `ori a0,a0,0x158`, 0x4b27e4 `sd a0,0x4540(a3)`), passed by address on every draw (573407).
   GS ALPHA 0x58 = A 0 (Cs), B 2 (0), C 1 (**Ad**), D 1 (Cd): `Cs·Ad >> 7 + Cd`. TEST 0x50000 (no alpha test).
3. **BF2 base colour.** `FUN_0058ed70` (B2 723257): *(decompiler detail omitted)* halves every byte (723536–723538), alpha from *(decompiler detail omitted)*
   = 0xff (723387). Its only other assignment, *(decompiler detail omitted)* (723449–723450), is reached for a translucent
   draw (*(decompiler detail omitted)*, 723410–723413: c.a ≠ 0xff, word & 4, or F & 0x800000) and for word & 2 under F & 0x20 — those
   draws get at most 0x7f. **A droid's word 0x10089 has none of 0x02/0x04/0x10, so its base alpha is 0x80.** Exception
   (verifier, machine code 0x58f344–0x58f370): a word with 0x10 and without 0x04 while `DAT_007c5754` < 1 (the pause
   dim) gets base colour 0xff404040, alpha 0xff: its pass weight doubles while paused (one pass word has 0x10: BF2
   0x10098 `fel1_bldg_bladder1`).
4. **BF2 pass record.** `_DAT_007c57b0` = 0xff00000158 (`FUN_00591a30` 724936; ELF 0x591af4 `lui s5,0x7c`, 0x591afc–
   0x591b04 build it, 0x591b0c `sd a0,0x57b0(s5)`), TEST 0x5001b, chosen when the word lacks 0x04 (723894–723898) and
   copied into the pass list on every draw that reaches the specular block (723900–723906). The same `Cs·Ad + Cd`.
   The passes a BF2 segment may run between its base and specular passes (projected 0x7c57f0/0x7c57e0, detail
   0x7c5800) write no frame alpha (TEST ATST NEVER, AFAIL RGB_ONLY — verifier), so the pass reads the base's As.
5. **The texture function.** TEX0 TCC=1, TFX=0 MODULATE for every LVL texture the uploaders register (B2 716966 /
   717180–717187, B1 562033–562038; the verifier found TCC=1 TFX=0 CPSM=PSMCT32 on every live texture-table entry:
   B1 103/103, B2 147/147 and 93/93), so the base fragment's alpha is **As = At·Av >> 7 = At**, written to the PSMCT32
   frame (FBMSK 0, `re-specular-pass.md` §6).
6. **At in GS units.** The palettes sit in EE RAM (CSM1 order) with alpha = (disc a · 129) >> 8, 0..128 — 256/256
   entries match for the SBD, battle-droid and skin sheets in both games, the raw disc value only 47–164/256
   (verifier; `analysis/re-hud-icons.md` citation 3 read the loader). **So the pass weight Ad/128 ≈ disc alpha / 255.**

## 2. Measured on the PS2 — the record patched in a save state [V]

`dshn/ps2/mkstates.py` (BF1) and `mkstates_bf2.py` (BF2) copy a state and patch EE RAM; `run.py` / `run2.py` boot
each copy in PCSX2 (isolated, under ci.lock) and keep the frame. Variants: `fix80` = record 0x8000000168 (ALPHA C = FIX,
FIX 0x80 = weight 1.0: the port's model); `nospec` = 0x0000000168 (weight 0); `hide` = the draw's vtable slot → a
`jr ra; nop` stub (BF1 Specular class 0x5594ac = `FUN_004b2310` → `FUN_00164598`; BF2 soldier render slot 0x61eb0c =
`FUN_00255db8` → `FUN_00164fe8`, `re-side-alpha.md` §2); `ctl`, `ctl2` unpatched. The verifier found the patched
values still in the post-run states, so the patches were live.

- **BF1 KAS2**, player = the SBD (`ref/states/bf1_kas2_cw_cis_cp4_cp11_dock_two_spawn.p2s`). One mask (the union over
  variants of |V − hide| > 30 in the droid's box, eroded 1 px; 27,491 px; `measure_union.py`):

  | variant | luma mean | p50 | p95 | p99 | ≥ 230 |
  |---|---|---|---|---|---|
  | ctl / ctl2 | 60.1 / 60.1 | 51.6 | 131.0 / 131.8 | 189.1 / 190.0 | 0 % |
  | fix80 (weight 1.0) | 100.1 | 86.5 | 221.7 | 243.9 | 3.72 % |
  | nospec | 54.2 | 46.5 | 113.5 | 161.7 | 0 % |

- **BF2 KAM1**, player = the engineer droid (the `cis_inf_bdroid` body, skin `cis_inf_battledroid_orange`, alpha mean
  27.6), `ps2/states2/bf2_eng_base.p2s` from UAUD2's class-select state: ctl 52.0/80.8/100.5 (mean/p95/p99), fix80
  86.7/199.2/226.4, nospec 47.5/75.1/98.3 (frame a; b agrees). Rain makes the mask choice matter by a few levels
  (the verifier's rain-rejecting mask: ctl 65.2/89.4/104.0, fix80 100.9/204.5/226.3); the conclusion holds in every mask.
  (A first BF2 try on the SBD itself was uninformative: its power-50 lobe added nothing in view and lightning
  flashes broke two frames — `sheet_bf2_kam1_all5.png`.)
- **Visually** `fix80` is chrome in both games and `ctl` matte: `ps2/sheet_bf1_dock2_all5.png`, `sheet_bf1_dock1_all5.png`
  (a second BF1 pose), `sheet_bf2_eng_all5.png`, and the summary `dshn/DSHN_pairs.png`.
- **The weight per pixel.** r = (ctl − nospec)/(fix80 − nospec) where the GS add is unsaturated is Ad/128 itself
  (`ratio.py`): BF1 mean 0.141, p50 0.098, p95 0.529 (verifier 0.151; least-squares slope 0.123–0.128), and ctl2's r
  correlates 0.963 with ctl's; BF2 0.124–0.128 (verifier 0.127–0.148). The map (`ps2/ratio_bf1_dock2.png`) is textured
  — bright on the upper back plate, dark elsewhere — so the weight follows the texture, and its level matches the
  sheets' gloss (0.134 SBD, 0.108 orange skin): it is not 1.0 (the port), not 0.27 (an unconverted CLUT alpha) and not 0.

## 3. The disc: every droid sheet's alpha is a gloss mask [V]

Mip 0, the CLUT alpha (`dshn/droid_alpha.py`, `tex_alpha_all.py` → `census_spec_droid.txt`; the verifier's own reader
agrees): `cis_inf_sbdroid` (both discs) mean 34.1, 0.04 % ≥ 128, max 247 (the red eye); `cis_inf_battledroid` 40.5; BF1
skins `_blue`/`_red` 37.9; BF2 skins `_blue/_camo/_green/_orange/_red` 26.1–27.6; `cis_inf_magnaguard` 25.6;
`imp_inf_darktrooper` 42.2, `_pack` 51.9; `cis_inf_droideka` 255 everywhere (and no pass). Each SBD model is one
segment: BF1 MTRL {0x31, A 4, B 0x00ffffff} RTYP Specmap (colour = the sun), BF2 0x10089, spec 0xffffffff, power 50.
The sheet and its alpha: `dshn/bf1_sbd_rgb_alpha.png`.

## 4. The port [V]

- The baked sheets have alpha 255 at every texel: `side_bf{1,2}/textures/{cis_inf_sbdroid, cis_inf_battledroid,
  imp_inf_darktrooper, cis_inf_magnaguard}.ktx2`, and BF2's `.tier2.ktx2` too (`ktx2alpha.py`, verifier).
- Why: `bake::soldier::bake_textures_flagged` (soldier.rs 412–416) calls `make_opaque` unless the OR of the naming
  segments' words has `ALPHA_IS_MEANINGFUL` = 0x02 | 0x04 (detail.rs 37): soldiers and weapons. Vehicles pass
  `AlphaUsage::needs_alpha` (vehicle.rs 321–333). Maps and common decide by `alpha::keep_alpha` (alpha.rs 459–469, map.rs
  1245–1310): a MASK/BLEND draw, an unnamed texture or a named consumer. No path asks whether a specular segment reads
  the alpha: `spec_pass` only writes `extras.spec_pass` (verifier's search).
- `fs_spec` weights the add by `base.a` = texel.a × vertex a (shader.wgsl 316, 738) — correct for a sheet that keeps
  its alpha, 1.0 for a forced-opaque one. `fs_main` (OPAQUE) never reads alpha, and no pipeline blends by the
  destination alpha (grep: no `DstAlpha`), so a kept gloss alpha changes nothing but the pass and the target's alpha
  plane (the goldens store it: `render::encode_png`).
- **Measured at the PS2 poses** (scratch example `dshn/tools/dshn_soldier.rs`: the map, the PS2 camera, the soldier at
  the PS2 soldier's feet facing along the camera in `UNIT_POSES[0]`, base + pass exactly as `SoldierVisuals::draws`;
  `proto_tex*/` = the baked sheet with the disc alpha written back by `tools/restore_alpha.py`; `measure_port.py`):

  | | now (alpha 255) | prototype (disc gloss) | no pass | PS2 |
  |---|---|---|---|---|
  | BF1 KAS2 SBD mean / p95 / p99 | 67.2 / 217.1 / 243.6 (3.0 % ≥ 230) | 31.6 / 104.4 / 138.0 | 26.1 / 68.1 / 116.3 | 60.1 / 131.0 / 189.1 |
  | BF2 KAM1 battle droid | 70.0 / 171.6 / 248.5 (3.1 %) | 43.5 / 73.6 / 93.4 | 38.6 / 65.6 / 78.2 | 52.0 / 80.8 / 100.5 |

  The add (with − without the pass): prototype +5.5 / +4.9, PS2 +5.9 / +4.5. The port's pose is not the PS2's
  animation, and the BF2 port droid wears the base sheet (40.5), not the engineer's orange skin (the port has no
  override skins — card USKN), so these are like-for-like in camera and lighting only. The renderer's adapter was not
  recorded (lane 1 saw a lost NVIDIA adapter at ~14:20; the frames show no speckle).

## 5. Scope: the census [V]

`dshn/world_census.py` → `world_census.txt` (reproduced exactly by the verifier): over every LVL on both discs, 151
(game, texture) rows are drawn by a pass segment (BF1 word & 0x20; BF2 & 0x80 without 0x04); **68 are LOST** (disc
alpha mean < 250, baked at 255 somewhere):
- side soldiers: BF1 SBD, battle droid, dark trooper (+pack); BF2 SBD, battle droid, magnaguard, dark trooper (+pack),
  AT-AT commander (24.5, its power-1 wash), AT-AT pilot, rebels (`all_inf_soldierjungle` 3.3, `markspersonjungle` 2.4,
  `vanguarddesert` 2.3);
- every first-person weapon sheet on both discs (38–82); BF1 `all_hover_rebel_attack_speeder` (91.6); common
  `com_weap_probedroid`, `com_1st_weap_inf_torpedo`;
- 29 world sheets: BF1 KAM1 (`kam1_bldg_plat1/2`, `kam_bldg_floor/domesides/railpost/rails/walkwaysides/antenna`,
  `kam_crate` — SPEC's "broad deck glow" at 51.7 % of the golden), KAS2 `kas2_tech`, RHN1 `rhn1_prop_tunnel_1`; BF2 DEA1
  (4), FEL1 `fel1_bldg_bladder1`, HOT1/TAN1 `hoth_prop_crate`, KAM1 (same sheets as BF1), NAB2 `nab2_prop_fountain`,
  UTA1 `uta1_bldg_generator(_lowrez)`.
Kept already (a BLEND/MASK draw needs it): e.g. `yav2_bldg_details` (5.8), `all_inf_wookie` (30.7). 62 of the 68 are
also drawn by non-pass segments (mostly LODs; some world sheets by 0x04/0x10 words): the port decides those draws' modes
from their own words, so keeping the alpha must leave them as they are (an OPAQUE draw never reads it).

## 6. History — SPEC2, not CAMB [V]

The SBD unit golden turns from matte to chrome at `c231edb` (SPEC2's goldens, 2026-09-25; SPEC2's report: "BF1's white
B … turns the droids and dark trooper chrome", accepted then as the disc's look). It stays chrome through `0b6fd3d`
(UGAL, 2026-09-29). CAMB's goldens `86d23b5` (2026-09-28) changed 10 map sheets and `walkabout_stats.ron`, no unit
sheet (CAMB lights BF1 soldiers only on maps that set `CharacterAmbientColor`), so CAMB is not the cause.

## 7. What the port owes — build DSHN (PROVISIONAL, [O])

1. **Bake: a texture that a specular-pass segment draws keeps its alpha** — the per-game predicate
   `bake::spec::spec_pass(seg).is_some()` over every segment naming it, at the three decisions: `soldier::
   bake_textures_flagged` (soldiers AND weapons — without touching the weapon materials' `legacy_per_texture_or`, which
   must not see the new keep), the vehicle bake's `needs_alpha`, and the map/common bake's `keep_alpha` / `AlphaUsage`.
   The `.tier2.ktx2` takes the PS2 mask (`take_alpha_from`), as it already does for a kept alpha. The material modes do
   not change. Rewrite the soldier.rs test that pins the droid sheet opaque (`textures_no_material_masks_are_baked_opaque`,
   ~681–716).
2. **Render: nothing** — `fs_spec` already weights by `base.a` (the GS's At·Av). A GPU referee should pin it: a sheet
   with alpha 0.25 gets a quarter of the add.
3. **Goldens / gates:** every unit sheet with a pass (28), map goldens showing a LOST world sheet (BF1 KAM1 first; KAS2,
   RHN1, BF2 DEA1/FEL1/HOT1/TAN1/KAM1/NAB2/UTA1 where in view) move in RGB, and wherever those sheets are drawn the
   golden's alpha plane moves too; the walkabout moves; `rebake=side,maps,common` (+ tat groups if a LOST sheet is
   there). Sim untouched.
4. **Later, USKN:** the override skins (`cis_inf_battledroid_{blue,red,…}`) must get the same keep when they are baked.
5. **Acceptance** (the card's): PS2 | port pairs per droid class at a PCSX2 pose, p95/p99 on the droid's pixels. The
   PS2 numbers above are the targets for the BF1 SBD and a BF2 battle-droid body.

## 8. Other findings

- **The port's soldier base is darker than the PS2's** at the same poses (no pass): BF1 KAS2 SBD mean 26.1 vs 54.2, BF2
  KAM1 battle droid 38.6 vs 47.5 (§4). Not the specular pass; a card (SBLT) for the lighting.
- BF2's pass also alpha-tests its own As (TEST 0x5001b, GEQUAL AREF 1; As = 1 always passes): not modelled, no effect.
- BF2's pause dim doubles `fel1_bldg_bladder1`'s pass weight while paused (§1.3): not worth modelling.

## 9. Checks run, the verifier, and what stays [H]

- **Self-check 1** (`re_check.py fn`, `dshn/checks/check1_fn.txt`): all 27 cited lines are in the named functions.
- **Self-check 2** (absences): `writers` of `DAT_0069c420..423`, `DAT_006f4540`, `_DAT_007c57b0` and `absent` of the
  record addresses (`check2_5_writers_absent.txt`); the verifier's whole-ELF store scans (`verify/scan_*.txt`) and its
  port search for a spec-aware alpha keep (none).
- **Self-check 3** (twins): BF1 `FUN_004b2310` / `FUN_004b27a0` and BF2 `FUN_0058ed70` / `FUN_00591a30` each read; both
  games measured on the PS2.
- **Self-check 4** (bytes): the record builders (BF1 0x4b27d0–0x4b27e4; BF2 0x591afc–0x591b0c,
  `check4_bf2_record_bytes.txt`), the vtable slots and stubs (0x5594ac, 0x164598; 0x61eb0c, 0x164fe8), and the live RAM
  values in the states (`mkstates*.py` assert them).
- **Self-check 5** (defaults / writers): *(decompiler detail omitted)*'s two stores and *(decompiler detail omitted)*'s three conditions (verifier, machine
  code); `DAT_0069c420`'s initialiser.
- **The verifier** (one Opus agent, foreground, claims only): **11 SUPPORTED, 2 PARTLY (V4, V12), 0 NOT SUPPORTED**.
  Folded in: V4's 0x7f cases and the pause-dim override (§1.3); V12's wording (§6); the palette rescale verified in RAM
  (§1.6); the BF2 in-between passes (§1.4); the engineer's orange skin (§2, §4); the scope notes (§5, §7).
- **Still [H]:** alias writes to seg+0x34 through every callee were not traced (RAM agrees in the 10 states); the BF1
  frame buffer's FBA (assumed 0, as `re-specular-pass.md`), which the measured r makes moot for these draws.

## 10. Reproduction

`dshn/`: `droid_alpha.py`, `tex_alpha_all.py`, `world_census.py`, `ktx2alpha.py` (data); `ps2/mkstates.py`,
`mkstates_bf2.py`, `mkbase_bf2.sh`, `mkbase_bf2_eng.sh`, `run.py`, `run2.py` (PCSX2; under ci.lock), `ps2/measure.py`,
`measure_union.py`, `ratio.py` (PS2 numbers); `tools/dshn_soldier.rs` (a scratch example — build it in a detached
worktree: `<repo parent>/openbf-wt/dshn-scratch`, target `lead_tmp/dshn_target`), `tools/restore_alpha.py`,
`measure_port.py` (port numbers); `verify/` (the verifier's own readers).
