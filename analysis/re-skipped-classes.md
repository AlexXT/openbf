# OPUS RE (2026-10-15) - [O] UNAUDITED

# RE: the instances the port skips as "no drawable model" - which the PS2 draws, and why (card SKIPC)

Corpora: B1 = `analysis/SWBFRONT.ELF-decomp.c`, B2 = `analysis/SWBFRNT2.ELF-decomp.c`; the ELFs; the extracted discs;
two PCSX2 captures of 2026-10-15; main **efa2d56**. Scratch, tools and every output: `engine/artifacts/lead_tmp/re/skipc/`.

## 0. Answer

1. **On today's main a battle draws the classes the card was opened for** [V][O]: the command-post console
   (`com_bldg_controlzone` and its major / invisible / untakeable variants), the three recharge droids
   (`com_item_healthrecharge`, `_weaponrecharge`, `_vehiclerecharge`), BF1's `com_weap_gunturret` and the
   `com_inv_col_*` walls all resolve a model once the COMMON bake is merged, which a battle does (`main/battle.rs` 29-37,
   `scene.merge_common`). The vines and the grass are drawn by their own passes (LEAF, FOLGG). **What the card's evidence
   showed is the STILL path**: `pose_render` without `POSE_COMMON=1` (the parity frames, `pcsx2/port.py`, the PXWA / PXWB
   walkabout pairs) and the `walkabout` example (the golden ground views) never merge the common bake, so every one of
   those classes is missing there (consoles 191, recharge droids 584, gun turrets and built-turret pads 166 instances; section 1). That is card
   STLC's cause; this card's build is that fix (section 3).
2. **Three things the PS2 draws that no port path draws** [V][O]:
   - **Godrays** (section 4): BASE `godray`, a map-FX-enabled manager of light shafts - BF1 Endor's 135 placed clusters
     (1,080 rays, read live in RAM) and BF2 Endor's 134, Yavin 1's one cluster per game, and props that attach a godray
     class (Geonosis, Felucia, Yavin's leaf tree). A RAM knock-out on BF1 Endor removes vertical light shafts from the
     PS2's frame. The port has no godray code. Not on a showcase map (YAV2 enables the effect but has no godray class).
   - **BF1 Geonosis cp2's building, `rep_bldg_forwardcenter`** (section 5): a `commandpost` class whose model, skeleton
     and collision live in `SIDE/REP.LVL`; the conquest mission loads it by name; the port bakes no side building, so cp2
     plays as a post with no building drawn and no body to collide with. Two PS2 frames show a large hull and tower
     where the port shows open desert.
   - The `dusteffect` volumes (mist, dust, cloud; 8 classes, 324 instances on 11 maps) are card **DUSTE**'s.
3. The rest are not drawn by the PS2 either, or not in the battle the port plays (section 6): sound and rumble markers,
   vehicle spawns, Hoth's `com_holo_controlzone` placed 157.5 m under the terrain, BF1 Theed's gian speeder
   (campaign-only), and the two VBIND names (`all_weap_inf_autoturret` is a weapon class, card VSELF).

## 1. The census (today's main, both render paths)

Instrument: `skipc_census.rs` (scratch example; mirrors `viewer::scene::upload_scene`'s loop - world instances +
foliage path props, `Scene::resolve_model` - for the map + its conquest mode layer, then again after
`merge_common`). Output `census_main_efa2d56.txt`. **Control** [V][O]: the game binary's own printout for BF1 KAS2
(`game --map assets_baked/bf1_kas2 --headless ...`, `kas2_cp5_battle.log`: "2 skipped classes: 68x com_snd_amb_static,
11x com_item_vehicle_spawn") equals the census's battle column for KAS2 row for row.

| class (base) | instances, maps | still path | battle | where the model / effect lives | PS2 draws it? | owner |
|---|---|---|---|---|---|---|
| `com_bldg_controlzone` + major / invisible (`commandpost`) | 191, 32 | skipped | **drawn** | COMMON bake (`common_bf?/models`) | yes (PXWA/PXWB sheets; F12) | **this card** (still path) |
| `com_item_healthrecharge`, `_weaponrecharge`, `_vehiclerecharge` (`powerupstation`) | 584, 32 | skipped | **drawn** | COMMON | yes (F50-F64 sheets) | **this card** |
| `com_weap_gunturret` (`armedbuilding`), BF2 `tur_bldg_laser` / `_beam` (via build zone) | 166, 17 | skipped | **drawn** | COMMON / its zone class | yes | **this card** |
| `com_inv_col_8/16/32/64` (`prop`) | 242, 12 | skipped | resolves | COMMON | no - invisible walls (6.3) | none |
| `com_item_powerup_health100`, `com_item_healthrecharge_unbuilt` | 2, 2 | skipped | drawn | COMMON | yes | this card |
| `rep_bldg_forwardcenter` (`commandpost`) | 1 (BF1 GEO1 cp2) | skipped | **skipped** | `SIDE/REP.LVL` (no bake) | **yes** (5) | new card FWDC |
| `end_prop_godraycluster`, `yav_prop_godraycluster` (`godray`) | 271, 4 | skipped | **skipped** | no geometry: an effect (4) | **yes** (4) | new card GODR |
| `*_prop_mist`, `*_prop_dust`, `rhn2_prop_cloud` (`dusteffect`, 8 classes) | 324, 11 | skipped | skipped | no geometry: an effect | yes (DUSTE) | DUSTE |
| `nab2_prop_vine1/2`, `cor1_prop_leafpatch_tall`, `yav_prop_bush(2)` (`leafpatch`) | 120, 4 | skipped by `upload_scene` | drawn by `viewer/leaf.rs` (259) | sprites (re-leafpatch) | yes | LEAF (done) |
| `nab_prop_flowers`, `yav_prop_grass(_tall)`, `nab1_prop_grass_tall` (`grasspatch`) | 149, 6 | skipped by `upload_scene` | drawn by `viewer/grass.rs` (285) | sprites (re-foliage 7) | yes | FOLGG (done) |
| `com_item_vehicle_spawn` (`vehiclespawn`) | 215, 23 | skipped | skipped | no geometry; its vehicle is drawn by the vehicle path | no (the pad spawns) | - |
| `com_snd_amb_static` / `_streaming` (`SoundAmbience*`) | 599, 14 | skipped | skipped | none (editor speakers) | no | - |
| `hot_rum_caves`, `uta_rum_*`, `mus1_rum_*` (`rumbleeffect`) | 10, 4 | skipped | skipped | none (camera shake) | no | - |
| `com_holo_controlzone` (`hologram`) | 2 (both HOT1) | skipped | skipped | placed at y -162.6 | no - under the world (6.2) | - |

Showcase maps, battle path [V][O] (census): NAB1 grass 4 (FOLGG); TAT1 dust 66 (DUSTE); YAV2 leaf 3, grass 12 (drawn by
their passes), mist 9 (DUSTE); UTA1 rumble 3; MYG1 nothing; HOT1 dust / mist 59 (DUSTE), rumble 2, the buried holo.
Vehicle spawns and sound markers on all. **None of the console / droid / vine classes** - in the battle. In the still
path all of them are skipped on all six.

## 2. The battle path draws the card's classes (the control picture)

`kas2_cp5_battle_pair.png` [V][O]: `game --headless --cam 5,18.315,-7,{0,180},0 --capture-frame` on BF1 KAS2 (the PXWA
cp1_cp5 stand) draws the tall recharge droid (b000) and the GNK box (b180); PXWA's sheet
`pcsx2/ref/bf1_kas2/walk/pairs/cp1_cp5__sheet.png` shows both on the PS2 and neither in its port column (the still
path). The headless capture uses `load_and_upload` = `main/battle.rs` (`run_headless`, `main/headless.rs` 335).

## 3. The still path - this card's build (card STLC folded in)

- `main/battle.rs` 29-37 and `run_headless` merge `common_<game>` [V][O]; `examples/pose_render.rs` merges it only under
  `POSE_COMMON=1` (40-46) [V][O]; `examples/walkabout.rs` has no `load_common` / `merge_common` [V][O] (grep). `pcsx2/port.py`
  runs `pose_render` without it (port.py 39-42) [V][O].
- So the parity score's port frames, every PXWA / PXWB / PXC port column and the golden ground views lack the console,
  the droids, the turrets and BF2's built turret pads; CPHOL's hologram stands over nothing in them (STLC's evidence).
- Build: merge the common bake in the still renderers exactly as a battle does (one shared helper, e.g.
  `assets::load_scene_as_battle(dir, mode)` used by `battle.rs`, `pose_render` and `walkabout`), drop `POSE_COMMON`.
  Moves: goldens with a post / droid / gun turret in view, every parity map's score. Acceptance: `PXC_SKIPPED=1` on the
  six showcase maps lists none of the console / droid / turret classes; KAS2 cp1_cp5, YAV1 cp6_reflectingpool and BF2
  COR1 cp4_cp4 re-rendered with `port.py <map>/walk` show the droids and the console in the PS2's boxes.

## 4. Godrays (BASE `godray`) - [O] for the class, the seeds and the switch; the draw stays [H]

**The class** [V][O]. B1 `FUN_00370fc0` (the SetProperty, B1 362122-362190) stores MinLifetime +0x4c, MaxLifetime +0x50,
Radius +0x54, MinAlpha +0x58, MaxAlpha +0x5c, Color +0x60 (four bytes, each 0xff unless given, stored only when at least
three values parse - its own format 0x5485c8), NumRays +0x64 (int, format 0x5485d8), SpreadRadius +0x68; the
keys are the FNV-1a hashes of the eight names the BF1 mod-tools ODF guide lists for `godray` (odfguide.md 1560-1569;
SpreadRadius 0x82a0379c, Color 0x3d7e6258, Radius 0x0dba4cb3, MinAlpha 0x7db59929, MaxAlpha 0xb1e439af, MaxLifetime
0x9d399b92, MinLifetime 0xbd9397ec, NumRays 0xf357466a). The constructor `FUN_00370a30` (361926-361945): MinLifetime
2.0, MaxLifetime 5.0, MinAlpha 0.5, MaxAlpha 1.0, Radius 1.0, NumRays 1, SpreadRadius 0. The disc's classes all
author Radius 3, MinLifetime 3.5, MaxLifetime 5, MinAlpha 0.15, MaxAlpha 0.8, Color 202 201 171; the `*cluster`
classes add NumRays 8, SpreadRadius 5 (the baked `classes.ron` of END1 / YAV1, both games).

**The seeds** [V][O]. `FUN_00370d18` (362054-362119), reached only through the class vtable (data word 0x548608 =
0x00370d18; 0x548618 = the setter; `re_check callers`: 0 direct calls), adds NumRays rays for a placed instance, each at
the instance's position (*(decompiler detail omitted)*) plus (u - 0.5) x 2 x SpreadRadius on each axis (the LCG at DAT_0066eec8),
through `FUN_0036e538(DAT_00519498, ...)`. `FUN_00370c60` (362014-362051) adds ONE ray for an attached godray object and
keeps its handle (the object is re-placed through `FUN_0036edb0`, 362436). `FUN_0036e538` (360197-360234) takes a
0x50-byte record off the free list (+0x2c) onto the active list (+0x30, count +0x28; `FUN_0036e430`), writes the
position, -1.0 in word 8, 1 in word 9 and the six class values in words 10-15. The explicit-segment twin
`FUN_0036e5f8` (360236) has no caller: no decomp call, no `jal` 0x0c0db97e and no data word 0x0036e5f8 in either ELF
[receipt:eb8e0013].

**The switch: the map's FX file** [V][O]. The manager (`DAT_00519498`, created by `FUN_0036dbe0`, cleared by
`FUN_0036dc20` - its only two writers) starts DISABLED: `FUN_0036dc60` 359747 and `FUN_0036dd78` 359781 call
*(decompiler detail omitted)* (the enable flag is +0x1c, 358273-358290). Its reader of the map's `Effect("Godray")`
section, `FUN_0036dee8` (359845-), turns it on by the `Enable` key (0xaf8bb8ce, 359907) and reads MaxGodraysInWorld
(0x74e6240d -> +0x34), 0xd760c55d (-> +0x38, the view list's size), MaxViewDistance (0xda62178f -> +0x44),
FadeViewDistance (0xf300705d -> +0x48), MaxLength (0x5d6614a1 -> +0x4c; the clamp `FUN_0036e5f8` applies),
Texture (0x3c6468f4 -> +0x54), OffsetAngle (0x34904329 -> +0xa0, x 0.017453294: degrees to radians),
two segment counts clamped 1..8 (0x7c0daa5d -> +0x3c, 0xf21598cf -> +0x40), TextureScale (0x6d9b436e, two floats -> +0x5c, +0x58),
0x3f7364cd -> +0x50, 0xef3fab75 -> +0x70, TextureVelocity (0x209f61af -> +0x60..+0x6c). Defaults (`FUN_0036dd78`
359776-359812): 300 rays, 8 view entries, 80 / 70 m, length 40, +0x50 2.0, +0x58 = +0x5c 1.5, segments 6 / 3, texture
"fx_godray" (string at 0x5485b0; fnv 0xac3632be).

**Which maps enable it** [D][O] (`fxgodray.py`, every LVL of both discs, `fx_godray_census.txt`): Enable 1 on BF1 END1, GEO1,
RHN1, RHN2, YAV1, YAV2 (+ YAV2_H) and BF2 DAG1, END1, FEL1, GEO1, TAT3, YAV1; Enable 0 on both HOT1; no Enable record on
BF1 BES1, BES2, NAB2, TAT1, TAT2. Seeds by map (the baked classes): BF1/BF2 END1 135 / 134 placed clusters plus four
tree classes that attach `end_prop_godray`; YAV1 (both) one cluster plus `yav_prop_leaftree`'s attachment; GEO1 (both)
and FEL1 attachments only; **YAV2, RHN1, RHN2 and BF2 DAG1 have no godray class: nothing seeds a ray there.**

**Live RAM** [V][O] (`ps2/run_end1.log`, `run_geo1.log`; BF1, one unit a side): END1's manager 0x107ad00: enable 1,
**1,080 active rays = 135 clusters x 8**, max 5000, view 50, MaxViewDistance **130** (the section carries two
MaxViewDistance records, 150 then 130: the last read wins), fade 70 (default), length 40 (default), texture 0xac3632be;
3-19 rays in the view list per bearing. GEO1: 4 active, length 20, angle -0.698 rad (= -40 deg), segments 12 -> 8 and 6.
[H] END1's 471 tree attachments put no ray in the active list (1,080 is the clusters alone) - why is open.

**The PS2 draws them** [V][O]: `end1_b090_godray_quad.png` / `end1_godray_knock_sheet.png` - the cp2 stand, pitch +25, four
bearings; each grab repeated with the active list emptied (+0x28 = +0x30 = +0x78 = 0, restored after). The signed diff
shows tapering vertical light shafts (b090: two bands, +5 to +7 levels of mean luminance across them) that exist only
with the list live; the port at the same pose (`cp2up_b090_port.png`) draws none.

**The port** [V][O]: no godray code - `re_check absent godray Godray GodRay --src engine/crates` finds three comment lines
(bake `detail.rs`, `leafpatch.rs`) [receipt:6b5fee9b]; `bake::leafpatch` drops a godray attachment.

**[H], for card GODR's RE:** the ray's shape and direction (the -1.0 length word, the sun, the +0xa0 angle), the
segments (`FUN_00370228`), the line-of-sight cut (`FUN_0036fdc0` calls `FUN_00161d68` with the length +0x4c), the view
selection and fade (`FUN_0036f678`: +0x44 / +0x48), the lifetime / alpha cycle, the texture scroll and the blend.

**BF2 is the same machine** [V][O]: setter `FUN_00455760` (B2 516343-516413, the same eight hashes at +0x6c..+0x88),
spawner `FUN_004554f0` (516275-516341: NumRays +0x84, 2 x SpreadRadius +0x88, `FUN_00453878(DAT_00601290, ...)`)
through its vtable (0x639920 = 0x004554f0, 0x639930 = the setter), the attached form `FUN_00455410`
(516230-516273: one add, the handle at +0x80 of a 0x90-byte object - B1's keeps it at +0xb0 of 0xc0 bytes), the
manager `DAT_00601290` (written only by `FUN_00453018` / `FUN_00453058`) built DISABLED by
`FUN_00453098` (514732: *(decompiler detail omitted)*), its FX reader at 514919 (MaxGodraysInWorld).

## 5. BF1 Geonosis cp2: `rep_bldg_forwardcenter`

- [V][O] The class: `entc`, BASE `commandpost`, GeometryName `rep_bldg_forwardcenter`, destroyed geometry
  `rep_bldg_forwardcenter_dest`, explosion `_exp`, collision prims `p_vehiclecylinder1/2` + CollisionMesh
  (`assets_baked/classes/bf1_rep.ron`, the side class table). Its `skel`, `modl`, `gmod`, `coll` and `prim` are in
  `SIDE/REP.LVL` of both discs (`lvlgrep.py`: 25 hits each); BF1 `GEO/GEO1.LVL` places it once (`inst`) - the port's
  manifest names it `cp2` at (38.275, -62.817, 258.848).
- [V][O] The PS2 loads it in the battle the port plays: the mission `geo1r` (and `geo1r_h`) calls
  `ReadDataFile("SIDE\\rep.lvl", "rep_bldg_forwardcenter", "rep_fly_assault_dome", ...)` (MISSION.LVL strings at
  0xc9ef / 0xe4b3, `mstr.py`).
- [V][O] The PS2 draws it: `geo1_forwardcenter_pairs.png` - two stands 20 m north (b000) and west (b270) of it: a large
  hull with a tower and a ramp on the PS2; open desert in the port's battle path at the same eyes (`POSE_COMMON=1`,
  which lists it skipped).
- [V][O] Why the port skips it: no bake path takes a side-LVL `commandpost` class - the vehicle bake keeps `VEHICLE_BASES`
  only (`bake/src/vehicle.rs` 45-63, 248), the soldier / weapon bakes and `map.rs` `pull_side_models` take named models
  (soldiers, weapons, sky objects), `side_classes.rs` reads only `SIDE/TUR.LVL` - so the bake holds no model of it: the
  name occurs only in the class tables and GEO1's instance row, in no GLB of the side, common or map bakes
  [receipt:9a4fc899]; and the battle's scene has no class row for it (`merge_common` brings only the COMMON bake).
  The same miss drops its collision body: the sim's world is built from `resolve_class` too. cp2 is still a post -
  posts are found by the CaptureRegion property (`game/src/layout.rs` 312-314), not by the class.
- [D] It is the only placed instance on the port's 32 conquest layers whose class a battle cannot find (census:
  `NOCLASS` in the battle column appears once).

## 6. The rest

1. `com_item_vehicle_spawn`, the `SoundAmbience*` speakers and the `rumbleeffect` markers have no geometry key; the
   ODF guide calls the latter two editor speakers and "special, used in Hoth hangar" (odfguide.md 76-88) [V][O]. Nothing
   to draw.
2. Hoth's `com_holo_controlzone` (both games, the same placement) stands at (-131.726, **-162.562**, 24.808); the terrain
   there is -5.03 (`terrain_h.py`) - 157.5 m under the ground [V][O]. A leftover; the posts' holograms are CPHOL's.
3. `com_inv_col_*` [V][O]: BF1's model is a 64 / 32 / 16 / 8 m box (12 triangles) with the texture `com_inv_col`, whose
   alpha is 0 everywhere, on a BLEND material; BF2's is two zero-area slivers (six vertices, 1/16 mm apart) on an OPAQUE
   material. The port draws BF1's in the Blend pass, which writes no depth (`render/src/pipelines.rs` 174-183): nothing
   shows. Nothing to do.
4. BF1 Theed's gian speeder (VBIND's note) [V][O]: the pad key names `nab_hover_gianspeeder`, whose class and model are
   only in `NAB/NAB2_H.LVL`; the Instant Action missions `nab2a` / `nab2c` read `NAB\nab2.lvl`, only the campaign's
   `nab2c_h` reads `nab2_h.lvl` (MISSION.LVL strings, `mstr.py`). In conquest the PS2 has no such class, so the pad
   offers it to nobody. Not a gap of the battle the port plays.
5. `all_weap_inf_autoturret` (VBIND's note) is a `wpnc` (a weapon class, `classes/bf2_all.ron`); its dropped turret is
   `all_bldg_inf_autoturret` with geometry `com_weap_inf_dropturret`, in the BF2 COMMON bake. Card VSELF.

## 7. Checks run

The `re_check.py` outputs (fn, callers, word, writers, and the three receipts eb8e0013, 6b5fee9b, 9a4fc899):
`engine/artifacts/lead_tmp/re/skipc/re_checks.txt`. Verifier results: section 8.

## 8. Verification (workflow `skipc-re-verify`, 7 agents, 2026-10-15)

The claims and their citations only (no reasoning), in five batches (A the class and seeds, B the manager and its FX
reader, C the BF2 twin, D the disc and census data, E the port's code); every absence claim (B6, B7, D3, D5, E2, E3, E7)
and every twin claim (C1-C6) went to a second, separate verifier. 47 verdicts: **46 SUPPORTED, 1 PARTLY, 0 NOT
SUPPORTED**; the second verifiers overturned none. The PARTLY (A1): the Color key reads four ints with its own format
(0x5485c8) and is stored only when three or more parse - fixed in section 4. Additions the verifiers made, folded in:
`FUN_0036c528` has 26 calls in B1 = 26 `jal 0x0c0db14a` words, three of them inside the godray module (0, 0, and the
Enable value); B2's `FUN_00451678` stores the flag as a byte at +0x1c; the B2 FX reader is `FUN_004532f0` (header
514828); the godray ctor also sets +0x48 = &DAT_005485e0 and +0x60 = _DAT_0069c420; the side bake's four paths are the
ones listed in section 5. An indirect call of `FUN_0036e5f8` through a computed address cannot be excluded (none was
found: no `lui`/`addiu` pair forms 0x0036e5f8). Stays [H]: the draw (section 4's list) and why END1's tree attachments
add no ray.
