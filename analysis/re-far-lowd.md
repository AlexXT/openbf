# OPUS RE (2026-09-29) - [O] UNAUDITED — the far scene draws a model's LOWD: BF2 UTA1's sinkhole far wall

Card UTAS (lane 2). Question: at BF2 Utapau's con_cp2 the port shows sky over clear-black below the horizon where
The maintainer remembers the far cliff of the sinkhole. Does the PS2 draw a wall there, and if so what is it?

## 0. Answer

- **The PS2 draws the far wall** [V, PCSX2]: at the walkabout's own con_cp2 eye (-43.657, 3.287, 180.765), bearing 0
  (yaw 359.1, pitch -0.6), the PS2 frame is filled by the sinkhole's far cliff (`pcsx2/ref/bf2_uta1/cp4_con_cp2_b000.png`,
  pair `pairs/cp4_con_cp2_b000.png`). The port at the same pose shows sky over black.
- **It is `uta1_bldg_cliffs`'s LOWD model, drawn by the FAR SCENE** [O]. The geometry ships two models: LOD0 is only
  the near half-cylinder (local z -228..3.2), LOWD is the whole ring (z -240..242) (§4). A geometry that has a LOWD gets
  LOD-record flags 3 (§2.2); flag 1 puts it in the far queue whenever its sphere reaches past `NearSceneRange[1]·S`
  (§2.4), and flag 2 sends it down the far walker's FADE branch, which draws LOD index 3 = the LOWD slot (§2.3, §2.5).
  The far target is composited over the near scene beyond `NearSceneRange[2]` (§2.6), so from anywhere on the map the
  ring beyond 55 m is the LOWD.
- **Proved on the PS2 itself** [V]: three patched copies of the b000 state each remove the wall and leave exactly the
  port's picture (sky above, black below): LOWD slot -> the LOD0 model, LOD flags 3 -> 1, and a NOP on the far walker's
  fade-branch draw. Two unpatched reloads and a NOP on the opaque-branch draw keep it (§1.2).
- **The port never draws a LOWD** [V]: `GeomModel::best_model()` (LOD0 first, `mungefs/src/entity.rs:102`) fills
  `ManifestGeometry.model` at bake (`bake/src/map.rs:914`); `Scene::resolve_model` (`game/src/assets.rs:657–670`)
  returns it and `game/src/viewer/scene.rs:690` draws it for every world instance, in one scene with one depth buffer.
  So nothing draws the ring's far half.
- **Not an orbit limitation, not the heightfield**: UTA1 has no terrain sheet and needs none; the missing geometry is a
  model's second LOD. Of the disc's 515 LOWD-bearing geometries on the port's 32 maps, only 9 have LOWD triangles more
  than 5 m outside their LOD0's box, and UTA1's cliffs are 870,000 m² of it; the next is 1,052 m² (§4).

## 1. PS2 evidence

### 1.1 The captures (`lead_tmp/re/uta1-sinkhole/ps2exp/capture_bf2.py`)
Booted `ref/states/bf2_uta1_cp3_con_cp1_postA_spawn.p2s`; the player soldier E = 0x1c87180 (cis_inf_rifleman; class at
E+0x8 = E+0x4a4 = 0x1a3eeb0) has its world matrix at E+0x120..0x15f (right, up, forward, position rows read off the
state), E+0x4c0 = 0 standing. A thread wrote the position (E+0x150) and zeroed E+0x4c0 every 2 ms; the chase camera
followed (eye within 0.05 m of the target after one correction pass). Right stick: F turns +yaw (150 ms = +13.4 deg),
T pitches up (150 ms = +7.6 deg). Frames: `ref/bf2_uta1/cp4_con_cp2_b{000,090,180,270}.png/.json`, states kept in
`ref/states/bf2_uta1_cp4_con_cp2_b*.p2s`, port renders and pairs beside them. b000: the far wall; b270: cliff where the
port shows sky at the left edge; b090 and b180: no gap at the PS2's 4:3 FOV (the walkabout's wider 16:9 90° panel has
a sky sliver at its right edge), the PS2's distant cliff softer — the far scene's LOWD beyond 55 m.

### 1.2 The patched states (`ps2exp/mkstates.py`, `run_ps2exp.py`; sheet `ps2exp/frames/sheet_b000_variants.png`)
Copies of the b000 state, the soldier held at the capture's position, pose identical (eye y 3.25..3.40, yaw 359.11):

| variant | patch (asserted old value) | far wall |
|---|---|---|
| ctl1, ctl2 | none | present |
| lowd2lod0 | gmod 0x1ab2f20 +0x2c (LOWD slot) 0x1ab2e20 -> 0x1ab2d20 (the LOD0 model) | GONE: sky above, black below |
| flag1 | LOD-record flags byte 0x1ab2f68 (gmod +0x48) 3 -> 1 | GONE |
| nofade | jalr at 0x5b0640 (far walker, fade branch) 0x0040f809 -> 0 | GONE |
| noopaque | jalr at 0x5b0678 (far walker, opaque branch) 0x0040f809 -> 0 | present |

## 2. The chain (BF2, `analysis/SWBFRNT2.ELF-decomp.c`)

### 2.1 The gmod loader — `FUN_001c59e8` (72214–72424), object 0x7c bytes from `FUN_001c55c0` (72034–72057)
- NAME -> +0 (hash). LOD0 -> +0x20 and +0x1c, tri count +0xc (72322–72331); LOD1 -> +0x24, count +0x10 (72332–72341);
  LOD2 -> +0x28, count +0x14 (72412–72417, after the `0x32444f4c` break at 72320); **LOWD -> +0x2c, count +0x18, only
  `if (DAT_005fc5b9 == '\0')`** (72362–72370); LOD1 also sets +0x1c when +0x20 is 0 (72339–72340), LOD2 when +0x20 and
  +0x24 are 0 (72418–72422). The trailers are ints (`FUN_003cb2e0`, stored with `sw`) — triangle counts. [V]
- At the chunk end (72252–72316): the 8-word LOD record at +0x30 = `FUN_001b5000(rec, 1, +0x20, +0x24, +0x28, +0x2c)`,
  unless the string last read into the loader's sp+0 buffer (NAME, or INFO's last cloth name when INFO follows with a
  count > 0) equals "coll_" (va 0x6187d0) case-insensitively over 5 chars (`FUN_00523848` = strnicmp) -> the empty
  `FUN_001b5180` record; no BF2 disc gmod matches (verifier census: 2,989 gmods). THEN empty LOD0/1/2 slots are filled
  from each other and last from the LOWD (72273–72316). [V]
- `DAT_005fc5b9`: ELF byte 0 (`re_check word b2 0x5fc5b8` = 0); one text hit (the read, 72364), no text writer; the
  verifier's instruction scan finds only the `lbu` at 0x1c5cd8; 0 in RAM in six BF2 states incl. UTA1 b000 -> the LOWD
  is loaded in the shipped game. [V]
- `+0x78` = vptr `&DAT_006187d8` (72042); its only entry, slot +8 (0x6187e0), is `FUN_001c5610` (ELF word). [V]

### 2.2 The LOD record — `FUN_001b5000` (59950–60004)
`rec[0..3]` = each present model's `+0xa8` (its triangle count: 588 / 1200 for the cliffs, the gmod trailers);
`rec[4]` mask |1 LOD0, |2 LOD1, |4 LOD2 (the LOWD sets no mask bit); `rec[5]` = argument 2 (1); **byte `rec+0x18` |= 3 iff
a LOWD model is present** (59995–59997), |= 4 when `FUN_0057e610(model)`; `rec[7]` = 1000.0. [V]

### 2.3 The draw by LOD index — `FUN_001c5610` (72059–72123)
model = `*(gmod + 0x20 + 4·lod)`, falling back to +0x20, +0x24, +0x28 when 0 (72073–72077); drawn through the model's
`+0xc0` vtable slot +0xc with the colour alpha scaled by the float argument when < 1.0. **lod 3 = +0x2c = the LOWD.** [V]
Entities reach it through their renderable interface at E+0xd0 (thunks *(decompiler detail omitted)*, e.g. `FUN_001d4208` ->
`FUN_001d1c28`, 80493–80540: *(decompiler detail omitted)*). [V]

### 2.4 The traversal — `FUN_005af6b0` (745980–746180), per object, sphere at obj+0x30 (centre) / +0x3c (radius)
- Far pass (*(decompiler detail omitted)*, 746034): after the frustum test (`FUN_005aef98` != -1, 746017–746022), the 0x200000
  gate (746031) and a not-full heap (746035), **only when the LOD record's `+0x18 & 1`** (746038), with
  d = dot(centre − eye, forward): `(d + r) − near·S >= 0` (746058–746060) and `FarSceneRange > near·S` (746063) ->
  pushed on the far heap `DAT_0060c88c`; otherwise not queued (746083–746085). The only other `DAT_0060c88c` users
  allocate it (`FUN_005b00d8`, 512 entries) and clear it (`FUN_005b0250`). [V]
- Near pass: a `&1` object with `&2` skips the "entirely beyond near·S" cull (746107–746110); with `&1` alone it is
  dropped from the near scene when `near·S <= −z − r` (746124) if FarSceneRange > near (746112–746117). `FUN_001b5000`
  only ever sets |3, so every LOWD-bearing gmod is in BOTH scenes when it straddles. [V]
- `near·S`: `FUN_00565120` = `DAT_0074601c` = NearSceneRange[1] (`FUN_00564f50` 698733–698735, unless
  `DAT_0060c120` — ELF 0, no store; the console variable `rendering.nsFadeStart` (`FUN_005654e0` 699056–699058),
  settable only through `ScriptCB_DoConsoleCmd`'s forced "debugmenu." prefix, so 0 in retail); `FUN_00565140` =
  `DAT_00746024` = FarSceneRange (`FUN_00565070` 698768); S = cam+0x140 when `DAT_00746028` (`FUN_00565160`).
  UTA1 RAM: 40 / 55 / 400, S 1. [V]

### 2.5 The far walker — `FUN_005b0330` (746527–746668; disassembly 0x5b0560–0x5b0684)
For each popped object: d = dot(centre − `DAT_007ec470`, `DAT_007ec4e0`) — the frustum record's eye and forward
(`FUN_005ae940` 745241–745254: near/far at [0x21]/[0x20], eye = cam+0x60, +0x70 = vf0 − cam+0x50 on xyz =
−back); cull when `r²/(d+r)² < k` (k = `DAT_0060c858`² / 208000, `rendering.minScreenSize` 5.0, 746563/746569) or
`d >= FarSceneRange` (746649). Then (746650):
- **`rec+0x18 & 2` (FADE branch)**: fade = min(1, ((d + r) − near·S) · 0.05) (746656; 0x5b0604–0x5b0624); if fade > 0,
  draw slot +0x54 with **lod 3**, f12 = fade, flags `obj[0x11] | 0x20000` (0x5b0634–0x5b0644; the decomp's printed call
  at 746659 drops the float and the flags — `a2` is set in the branch's delay slot 0x5b0600). [V]
- opaque branch: lod 3, 1.0, `| 0x20000` only when `near·S < d − r` (746651–746652; 0x5b0650–0x5b067c). [V]

### 2.6 The composite (prior RE, not re-read here)
Near-scene pixels beyond `NearSceneRange[2]` (or empty) show the far target, a ramp b..c between
(`re-fog.md` §2.5, `re-sky-pass.md` §2.2 [V]). The PS2 camera's near-scene `far` in every UTA1 pose json is 55.0.

### 2.7 Context — the near scene's LOD choice is a BUDGET, not a distance (`FUN_005ace40` 743899–744005)
(gmods queue on 1 — `FUN_005acd68` indexes by rec+0x14 = 1; `DAT_0060c85d` = 0 `rendering.showMissingLOD`,
`DAT_0060c854` = 1.0 `maxVertexDensity`, console variables with no store.)
Objects leave the near heap largest-on-screen first (gate 743964); each takes the first level of its mask (743970) whose
running triangle total still fits the queue's per-level budget (743983), skipping a level whose triangles per screen
area exceed `DAT_0060c854·208000` when `rec+0x18 & 1` or `DAT_0060c85d` (743972–743976); none fits -> lod 3 if `&1`
(743991–743993), else not drawn. The cliffs' mask is LOD0 only. [V lines; D reading; the budgets' values unread — H]

## 3. RAM (b000 state, `ref/states/bf2_uta1_cp4_con_cp2_b000.p2s`)
- gmod `uta1_bldg_cliffs` = 0x1ab2f20 (vptr 0x6187d8 at +0x78; the only gmod with counts 588/0/0/1200): +0x1c..+0x28 =
  0x1ab2d20 (LOD0; LOD1/LOD2 filled from it), +0x2c = 0x1ab2e20 (LOWD); record at +0x30 = 588, 0, 0, 1200, mask 1,
  1, flags 3, 1000.0. [V]
- its entity 0x1ad7cc0 (E+0x160 = the gmod; matrix (−1,0,0)(0,1,0)(0,0,−1) at (−20, 1, −30)): sphere centre
  (−27.97, −307.82, 82.35), r = 869.14. Far record 0x7ec470: eye (−43.657, 3.249, 180.765), forward (0.0155, −0.0112,
  −0.9998), far 400, near 40. -> d = 102.1, r²/(d+r)² = 0.80 (k = 1.2e-4), d < 400, fade = min(1, (971.3 − 40)·0.05)
  = 1; the opaque rule d − r > 40 is false. [V]

## 4. The disc and the census
- UTA1.LVL: gmod `uta1_bldg_cliffs` LOD0 `uta1_bldg_cliffs` (588 tris), LOWD `uta1_bldg_cliffsLOWD` (1200), INFO
  (0, 1, 0) [V bytes]. The bake's GLBs: LOD0 local x −219..235, y −832..214, z −228..3.2; LOWD −240..240, −832..214,
  −240..242 — the ring. One instance, at (−20, 1, −30) yawed 180°. [V]
- `lead_tmp/re/uta1-sinkhole/lowd_census.py` over all 32 bakes: 515 geometries have a LOWD; 9 have LOWD triangle area
  whose centroid lies > 5 m outside their LOD0's box: UTA1 cliffs 869,971 m² (50.8 %), BF1 KAS2
  barrier_stairs 1,052 / dock_ender_right 981 / left 928, UTA1 rocks 728, BF1 END1 end_prop_tree01 238 (a billboard-
  style LOWD), UTA1 hangers ≤ 142. [V measured]

## 5. BF1
`FUN_001a91e0` (B1 51375–51430) is the byte-for-byte twin of `FUN_001b5000` (LOWD -> flags |3). BF1's gmod loader
`FUN_001ace78` loads LOWD with no gate (54126–54133). The far walker `FUN_004c62b0` (587277–587407) has the same two
branches (flags & 2 -> fade, lod 3; else `near·S < d − r`, 587389–587400) and is called only in the 0x40000 pass of
`FUN_004c6608`; it has no minScreenSize skip, applies S unconditionally (`*(FUN_0049c3d0()+0x90)`) and pops ≤ 200.
BF1's per-object far-queue insertion was NOT read — [H] same as BF2.

## 6. For the port (PROVISIONAL shape for the build — card UTAS phase build)
The port's single scene draws LOD0 everywhere, the modern choice (the PS2's LOD switch is a budget, §2.7); what it
loses is LOWD geometry that LOD0 does not have. `extension_split.py` splits `uta1_bldg_cliffsLOWD` in world space
(degenerate strip triangles dropped): **360 of its 731 triangles (677,178 m²) lie within 2 m of the cliffs' own LOD0,
none lie on any other placed instance's LOD0, and 371 (1,036,060 m²) are the extension** — the whole far half plus 81
near-side triangles (120,684 m², many 20–100+ m off LOD0, y down to −831: the pit's lower walls). At a 5 / 10 / 20 m
threshold the extension keeps 357 / 351 / 333 triangles. Build shape:
1. bake: per geometry with a LOWD, the LOWD triangles whose centroid is > ~2 m from the geometry's LOD0 surface, kept
   only when their area passes a threshold well above the census's next entry (1,052 m²) — UTA1's cliffs alone today;
   written as an extra model on the geometry (`ManifestGeometry`), skipped when empty so other manifests do not move;
2. draw: that model per instance, depth-tested, fragments nearer than `NearSceneRange[1]·S` (40 m on UTA1) discarded —
   the far scene's reach (§2.4); the PS2's b..c crossfade is optional (nothing of it is near a playable spot);
3. look at the seam where the extension meets LOD0 (the ring's two ends), the walkabout's UTA1 sheets and
   `maps/bf2_uta1.png`'s vantage panel, and the pit looking down from the platforms.
Not this card: drawing LOWD instead of LOD0 beyond the near scene for every object (card LOD), and the BF1 cases.

## 7. Checks run (full output: `engine/artifacts/lead_tmp/re/uta1-sinkhole/checks.txt`)
*(decompiler excerpt omitted from the public copy)*

## 8. Still [H]
- (2026-10-16, card LOD: BF1's far insertion — `FUN_004c5808` 586920–586941, the twin — and the near-scene budgets'
  values are now read: `re-lod.md` §2–§4 [O].)
- BF1's per-object far-queue insertion (the `FUN_005af6b0` twin) was not read; the BF1 map cases the census flags (KAS2
  barrier/docks, END1 `end_prop_tree01`) have no PCSX2 frame.
- The near-scene budgets' values (`DAT_007ec448` queues) — the cliffs take LOD0 near (mask 1); not needed here.
- How the far target shares Z with the near scene in detail (prior RE) — irrelevant for the ring (nothing near covers it).

## 9. Verifier (one opus agent, foreground, claims + citations only; 2026-09-29)
15 claims: 13 SUPPORTED, C2 PARTLY (the "coll_" compare reads the loader's last string buffer, not always NAME — fixed
in §2.1; no BF2 gmod matches either way), C15 SUPPORTED with a wrong citation (fixed in §0). Completeness notes taken
into §2.1, §2.4, §2.5, §2.7, §5. Side finding: `mungefs::entity::GeomLod.param` (and the comments at entity.rs:7–8,
82–83) read the per-LOD trailer as an f32; the ELF reads an int — the model's triangle count (588 / 1200 here).
