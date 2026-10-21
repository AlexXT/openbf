---
id: USKN
title: Per-class skins and size: OverrideTexture / OverrideTexture2 (sharpshooter red, assault/assassin droid blue/red, urban/desert/snow rebels, snow Wookiee, the locals) and SkeletonRootScale / CollisionRootScale (Wookiee 1.15, Gungan 1.1, Jawa 0.6, Ewok 0.5) - the port reads neither
milestone: M1
value: 5
phase: build
status: done
deps:
files: engine/crates/bake/src/soldier.rs, engine/crates/mungefs/src/anim.rs, engine/crates/game/src/animator.rs, engine/crates/game/src/soldier_vis.rs, engine/crates/game/src/roster.rs, engine/crates/game/src/assets.rs
declares: rebake=side; goldens=units/bf1_all_inf_wookie.png,units/bf1_cis_inf_bdroid.png,units/bf1_cis_inf_sbdroid.png,units/bf2_all_inf_marksperson.png,units/bf2_all_inf_markspersonsnow.png,units/bf2_all_inf_wookie.png,units/bf2_cis_inf_bdroid.png,units/bf2_cis_inf_magnaguard.png,units/bf2_cis_inf_sbdroid.png
needs:
effort:
branch: uskn
resume: landed c5110943 2026-10-20
created: 2026-09-29
alpha: yes
---

## Why (RE 2026-10-16, analysis/re-soldier-skeleton-skins.md [O])
Two visible gaps on every map, one of them on every Clone Wars battle:
- **Own-skeleton classes are drawn at human bone lengths.** The PS2 poses a class that names a `SkeletonName` with that
  rig's bone LENGTHS (BF1 22 body bones, BF2 every bone but bone 0 / bone_root / hp_weapons take the rig's rest
  translation) and scales only bone_root's clip translation by `SkeletonRootScale`. The port copies the human clips'
  translations onto every bone (`bake::soldier::clip_to_tracks`). Idle head height port -> PS2: Wookiee 1.605 -> 1.852 m,
  MagnaGuard 1.643 -> 1.865 m, battle droid 1.605 -> 1.726 m (head sunk into the pack, shoulders 5-7 cm out), also the
  super battle droid and BF2's snipers (`lead_tmp/uskn/pics/close_a.png`, `close_b.png`). The clone trooper is unchanged.
- **Per-class skins** (OverrideTexture / 2) are not drawn: sharpshooter red, the droids' blue / red / orange / camo, the
  urban / desert / snow rebels, the snow Wookiee (unit-audit-bf1 / bf2 §3.3).

## Evidence
- The rule: re-soldier-skeleton-skins.md §1-§3 (pose builders, BF1 `FUN_0022f428` / BF2 `FUN_00281e68`, the setters); PS2
  RAM of 6 save states agrees for every fielded rig (§5, `lead_tmp/uskn/ram_bf1.txt`, `ram_bf2.txt`); disc rig tables
  `rigs_bf1.txt` / `rigs_bf2.txt` (§4); one rig and one scale per soldier model in both games (`census_model_rig.txt`).
- The override draw: BF1 `FUN_002013a0` (the live soldier; NOT `FUN_001be1e8`), BF2 `FUN_00239bb0`; OverrideTexture2 only
  with OverrideTexture in both games (§9). 13 of 18 override textures checked are not in the bake.
- Tools for the builder: `lead_tmp/uskn/tools/ps2rule_glb.py` (a GLB re-posed by the rule), `fk.py` (bone positions at a
  clip frame), `crc.py`, `bonerecs.py`.

## Acceptance
1. Skeleton: a bake/pose test over the baked GLBs: idle frame 0 (`human_rifle_stand_idle_emote[_full]`), head y above the
   feet = BF1 Wookiee 1.852, BF2 MagnaGuard 1.865, BF1 battle droid 1.726, BF2 battle droid 1.758 (each +-0.01), the BF1
   clone trooper unchanged at 1.605; and the listed bones' rest = the rig's JONT rest_pos (= the PS2 RAM records +0x50).
2. Root scale at run time per class: bone_root x SkeletonRootScale in the drawn pose; the movement chooser and the clip
   rate on v / s (BF1 per-axis 0.3, BF2 circular 0.09 / 0.04); a test with a 1.15 class.
3. Overrides: a census test that every fielded class with an OverrideTexture draws its override on the MNAM
   `override_texture` segment (and `override_texture2` only with the first), all those textures baked; the picture pair
   against `lead_tmp/unit-audit/pcsx2_bf2/cis_droid_skins.png` and the BF1 class-select sheets.
4. Goldens declared and viewed (the maps that show droids, Wookiees, MagnaGuards or skinned classes).

## Original card (2026-09-29)
### Why (2026-09-29)
UAUD1 (analysis/unit-audit-bf1.md §3.3). BF1's soldier-class setter `FUN_0020f2e0` stores OverrideTexture at +0x18d0 (B1 122890) and
OverrideTexture2 at +0x18d4 (122865), and scales the skeleton's `bone_root` by SkeletonRootScale (123186-123195, when
`SkeletonName` is set and is not the human default); CollisionRootScale goes to +0x1e60 (123572). Nothing under
`engine/crates` reads any of them. The PS2 class-select frames show the sharpshooter's red and the assault / assassin
droids' blue / red markings (`lead_tmp/unit-audit/pcsx2/cis_classes_sheet.png`, `rep_classes_sheet.png`); the port
draws them as a plain clone and plain tan battle droids, the urban / desert rebels as the default trooper, and the
Wookiee Smuggler 15 % small on 13 port GCW missions.

### Evidence (2026-09-29)
- 16 BF1 IA classes author an override. Two classes' pairs change nothing (the ARC trooper's, the smuggler's), and
  neither does the snow soldier's body override - the table in analysis/unit-audit-bf1.md §3.3. The override
  textures sit in the class's own side sublevel.
- ANSWERED by UAUD1 (analysis/unit-audit-bf1.md §3.3, [V] + the second verifier's F5): which segment an override
  replaces.
  - BF1's soldier draw `FUN_001be1e8` (B1 65890-65907) reads class+0x18d0. When it is set, the draw sets the model
    group fnv("override_texture") = 0xdc5803e9 to OverrideTexture, and group fnv("override_texture2") = 0xb98e11c1 to
    OverrideTexture2 when +0x18d4 is set. It then draws with that override list.
  - Each disc model segment names its group in its `MNAM` chunk
    (`engine/artifacts/lead_tmp/unit-audit/verify2/modl.py SIDE/REP.LVL rep_inf_trooper …`):
    - `rep_inf_trooper`: its one segment;
    - `cis_inf_bdroid`: its one segment;
    - `all_inf_soldier`: the `All_Inf_trooper` body, not the face or the packs;
    - `all_inf_snowtrooper`: body `override_texture`, face `override_texture2`;
    - `rep_inf_arctrooper`: body `override_texture`, pack `override_texture2`.
  - The port's soldier GLBs keep only the texture names, not the MNAM slot.

### Acceptance (2026-09-29)
- RE ([O]): what the bone_root scale does to the mesh, the collision, the camera and the speed (CollisionRootScale
  +0x1e60's reader); BF2's twin of the override draw.
- Then build:
  - the soldier bake keeps each segment's MNAM slot and bakes every override texture the missions' classes name;
  - the draw swaps them per class;
  - the root scale goes on the skeleton;
  - goldens declared and viewed.

## Notes
- UAUD2 (2026-09-29, analysis/unit-audit-bf2.md §3.3): BF2's draw is the twin `FUN_00239bb0` (B2 155278-155296, class at entity+0x4a4):
  OverrideTexture (setter 171503, class+0x844) -> group fnv("override_texture"); OverrideTexture2 (+0x848) only INSIDE
  that branch. 9 BF2 classes change their look: the CIS assault / engineer / assassin droids red / orange / camo on all
  16 CW scripts (PS2: `lead_tmp/unit-audit/pcsx2_bf2/cis_droid_skins.png`), `all_inf_rifleman` desert tan on 7 GCW
  scripts (PS2: `pcsx2_bf2/cor1g_sheet.png` vs `end_sheet.png`), `all_inf_rifleman_urban`, hot1's snow rebel faces,
  snow vanguard and snow Wookiee, geo1's `rep_inf_ep2_sniper` (red sharpshooter). SkeletonRootScale: the MagnaGuard
  (`cis_inf_officer`, every CW battle) 1.24, the Wookiee 1.15 (+ CollisionRootScale), the rebel snipers 0.95. Table in
  analysis/unit-audit-bf2.md §3.3.

## Planner note (2026-09-30, M-ALPHA)
`alpha: yes` — per-class skins: snow troops on HOT1, the sharpshooter's red, on every map. Value 4 (kept). Showcase maps: BF1 NAB1, BF1 TAT1 (the port's `tat1`), BF1 YAV2, BF2 UTA1, BF2 MYG1, BF2 HOT1 (PLAN §Milestones M-ALPHA).
- planner 2026-10-19 16:25: value 4 -> 5 (the RE's suggestion): every Clone Wars battle's battle droids are distorted and the Wookiee / MagnaGuard crushed to human size - the main CIS infantry, seen in a minute of play.
