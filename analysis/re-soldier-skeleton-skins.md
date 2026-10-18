# OPUS RE (2026-10-16) - [O] UNAUDITED — a soldier class's own skeleton, its size and its skins: SkeletonName, SkeletonRootScale, CollisionRootScale, OverrideTexture (card USKN, BF1 and BF2)

Card USKN (lane 2). Question (UAUD1 / UAUD2 §3.3): what does a class's `SkeletonRootScale` do to the mesh, the collision,
the camera and the speed; what does `CollisionRootScale` do; BF2's twin of the override-texture draw. Builds on
`unit-audit-bf1.md` / `unit-audit-bf2.md` §3.3 (corrected in §0.7 and §0.3 below), `re-anim-keys.md` (the rig registry,
§1.1, §2.3) and `re-softskin.md` (bone-space vertices). Tools, tables, RAM scans and pictures:
`engine/artifacts/lead_tmp/uskn/` (`tools/`, `rigs_bf1.txt`, `rigs_bf2.txt`, `ram_bf1.txt`, `ram_bf2.txt`, `fk_idle.txt`,
`census_model_rig.txt`, `census_bf2_keys.txt`, `pics/`, `checks/`).

## 0. Answer

1. **`SkeletonRootScale` does not scale a soldier's mesh.** Both games build a bone's local transform from the pose quaternion and ONE
   translation: the pose skeleton's rest translation (bone record +0x50), or the clip's when bit 0 of record +0x67 is set;
   when bit 1 is set that translation's xyz is multiplied by the float at +0x68 (BF1 `FUN_00440928` / `FUN_00440ed0`, BF2
   the five builders `FUN_00533d38`…`FUN_00534a70`; the instruction is `VMULx.xyz`, the matrix has no scale term) (§1) [O].
   `SkeletonRootScale` is that float on `bone_root` only: it scales the CLIP's root translation (hip height and bob), so the
   human clips put a longer- or shorter-legged skeleton's feet on the ground (§3) [O].
2. **A class with its own `SkeletonName` keeps its skeleton's bone lengths** (§2) [O]. The skeleton is the `zaf_` rig the
   class names (`re-anim-keys.md` §1.1, §2.3), and when it is set:
   - BF1 `FUN_0022f428` clears the clip-translation bit on 22 named body bones (pelvis, spine, ribcage, neck, head,
     clavicles, arms, hands, legs, feet, toes); DummyRoot, bone_root, hp_weapons and any other bone keep the clip's;
   - BF2 `FUN_00281e68` clears it on every bone but bone 0 and sets it back on bone_root and hp_weapons (the droideka and
     the Geonosian get their own short lists).
   Soldier vertices are stored in bone space (`re-softskin.md`), so the PS2 draws the mesh where the RIG's lengths put the
   bones; the model's own `skel` never enters the draw.
3. **The port draws every own-skeleton class at HUMAN bone lengths** (§6) [V code, measured]. `bake::soldier::clip_to_tracks`
   copies every clip track's translations, and an untranslated track holds the human bank's stored base (`decode_clip`), so
   the body bones get the human rig's offsets. Idle frame 0, head height above the feet (port → PS2 rule, `fk_idle.txt`):
   - Wookiee (BF1 smuggler; BF2 `all_inf_wookiee`, 35 scripts): 1.605 → 1.852 m (+15 %), bone_root 1.015 → 1.168;
   - MagnaGuard (BF2 `cis_inf_officer`, every CW battle incl. UTA1 / MYG1): 1.643 → 1.865 m, bone_root ×1.24;
   - battle droid (every CW battle of both games): head 1.605 → 1.726 m (BF1), 1.643 → 1.758 m (BF2), shoulder joint 5–7 cm
     further in, the head forward of the neck instead of sunk into the pack (`pics/close_a.png`);
   - also the super battle droid, BF2's rebel snipers (0.95), and the locals and heroes the port does not field yet.
   The clone trooper (human rig) is unchanged to the pixel - the control (`pics/close_a.png`). The UAUD1 reading "the
   Wookiee is drawn 15 % small because the scale is missing" is right in effect but wrong in mechanism: the missing piece is
   the rig's bone lengths; the scale alone would lift a human-sized Wookiee off the ground.
4. **The PS2's RAM agrees** (§5) [V RAM]: in six save states (BF1 GEO1, END1, NAB1, KAS1; BF2 MYG1, HOT1) every rig a
   fielded class uses carries exactly its game's bit pattern, and bone_root carries flags 3 with its class's scale: Wookiee
   1.15 (BF1 END1, KAS1; BF2 HOT1), Ewok 0.5, Gungan 1.1 (NAB1), MagnaGuard 1.24 (MYG1), rebel sniper 0.95, Vader 1.25
   (HOT1). A rig no fielded class uses keeps the loader's all-set default.
5. **The scale's other readers** (§7) [O unless marked]: the movement-clip chooser divides the velocity by it before the
   idle test and the clip choice (BF1 `FUN_0022e990`, BF2 `FUN_0027ed18`), so a 1.15 Wookiee plays its walk / run at
   v / 1.15 (BF1's idle test is per axis, |v/s| < 0.3; BF2's is circular, |v/s|² < 0.09, or 0.04 while animator +0x20c0
   is set, and BF2 also raises a 10.0 limit to 10/s when s < 1); BF2's not-moving test is dt × scale × 0.2 / 0.3 (`FUN_0024f350`); BF1 hands 2 × scale to `FUN_0047c870`
   [H: the ground shadow]; BF2's AI multiplies a point on a soldier target by the target's scale (`FUN_00335920`,
   `FUN_00337b80`) [H: which point]; while soldier +0xc14 ≥ 0 BF1 / BF2 run the animator step with the scale reset to 1
   [H: what sets that state]. **No camera code reads it** (the readers are listed in §7, [receipt:3f15c787]); the camera is the
   class's own `CAMERASECTION` (card CAMS). **No speed reads it**: MaxSpeed is a class key of its own.
6. **`CollisionRootScale` scales the soldier's collision primitives** (§8) [O]: BF1 `FUN_0020e2e0` (its only reader past
   the constructor, copy and setter, [receipt:3f15c787]) builds the three stance sets with s = the key: `coll_body` length
   1.55 s, `coll_head` at 1.65 s (r 0.17 max(s,1)), `coll_body_upper` at 1.9 s − 0.42, the feet sphere fixed; BF2
   `FUN_002517e8` likewise (and its main capsule 1.65 s), with three more BF2 readers. The port's soldier capsule is one
   shape for every class (r 0.5, h 1.9 / 1.6 / 0.55, `core::unit::capsule`) → card **USKC**.
7. **Override textures** (§9) [O]: the live soldier's draw is BF1 `FUN_002013a0` (two model draws) and BF2
   `FUN_00239bb0`: class OverrideTexture → model group `override_texture` (0xdc5803e9) and, ONLY inside that branch,
   OverrideTexture2 → `override_texture2` (0xb98e11c1). UAUD1's `FUN_001be1e8` is another object's draw (its animator
   is built with scale 1.0). 13 of 18 override textures checked are not in the port's bake.

## 1. The pose builder (both games) [O]

- BF1 `FUN_00440928` (B1 500703) and its twin `FUN_00440ed0` (501107); BF2 `FUN_00533d38` (663382), `FUN_00534150`
  (663676), `FUN_00534430` (663905), `FUN_00534708` (664131), `FUN_00534a70` (664374). Per bone (0x70-byte record R,
  pose entry P = pose + i·0x20 + 0x10):
  - a bone whose name hash (+0x60) is that of `DummyRoot` (string 0x5576f8) copies its parent's matrix;
  - else T = R+0x50 (the rest translation) or, when bit 0 of R+0x67 is set, P+0x10 (the clip's) (B1 500781-500784);
  - when bit 1 of R+0x67 is set, T.xyz × R+0x68 (500786-500791); ELF `0x440a20` / `0x440fc8` (B1) and `0x533e0c` (B2) =
    `0x4bc20858` = `VMULx.xyz vf1, vf1, vf2.x`, vf2.x = the +0x68 float;
  - the local matrix = `FUN_00446110` (B1) / `FUN_0053ac20` (B2): the rotation of P's quaternion (normalised by 2/|q|²)
    with T in the fourth row, w = 1 - no scale; world = parent × local; recursion over child (+0x66) and sibling (+0x65).
- So the +0x68 float moves ONE bone's origin relative to its parent; every child keeps its own length.
- The flag setters: BF1 `FUN_00441ca8(scale, skel, hash)` (502037; 5 call sites, ELF `jal` count 5) / BF2
  `FUN_00535658(scale, skel, index)` (665074): bit 1 set when scale ≠ 1.0 (cleared at 1.0), scale stored at +0x68 (B1
  502055-502061). BF1 `FUN_00441c38` / BF2 `FUN_00535618` set or clear bit 0 only. The walker's pair writes the same
  fields (BF2 `FUN_0025f050`, undone by `FUN_0025f210` 179322-179324); the loaders write the byte as 1 (B1 `FUN_00441538`
  501629, B2 `FUN_00534f70`) and the record copies carry it (B1 501678, B2 `FUN_005356f0`). The flag lives in the
  SKELETON, shared by every soldier on that rig.

## 2. Which bones keep the clip's translation [O]

- The skeleton loader BF1 `FUN_00441538` sets R+0x67 = 1 (the clip's translation) on every bone (B1 501629).
- **BF1 `FUN_0022f428`** (145362) clears bit 0 on 22 names (strings 0x5352a0-0x535400): bone_pelvis, bone_a_spine,
  bone_b_spine, bone_ribcage, bone_neck, bone_head, bone_l/r_clavicle, _upperarm, _forearm, _hand, bone_l/r_thigh, _calf,
  _foot, _toe. Called from the SkeletonName case (0x7012f6cd) of the class setter `FUN_0020f2e0` (123351-123361), from
  `FUN_0020e2e0` (122382) and from `FUN_0022d828` (144320). A low-res rig gets an 11-bone list instead (`FUN_00230168`,
  145975, from `FUN_0022fd28`).
- **BF2 `FUN_00281e68`** (204167): bit 0 cleared on every bone index ≥ 1 (`FUN_00535618(skel, i, 0)`), then set on one
  list: 0x620a90 = {bone_root 0x16e27226, hp_weapons 0x2b960099}; 0x620aa0 = {bone_root, bone_head, the four wrists}
  when the rig has `bone_abdomen` 0x93e93819 (the droideka); 0x620ac0 = {bone_root} when it has `bone_r_finger3`
  0x7a89c91f (the Geonosian). Callers: the SkeletonName case of `FUN_00252bb0` (171413), `FUN_002517e8` (170132),
  `FUN_00281228` (203646), `FUN_001765d0` (17742). Both games' functions also run on the default rig's load (BF1
  `FUN_0022d828`, `FUN_0020e2e0`; BF2 `FUN_00281228`, `FUN_002517e8`, `FUN_001765d0`), so the default human rig gets the
  same treatment. Hash names by the engine CRC (`tools/crc.py`; names found in the BF2 side LVLs).
- For a human rig both rules change nothing: the human rig's rest = the human clip bank's base (`rigs_*.txt`).

## 3. The class keys (setter → field → use) [O]

| key (fnv1a-lowercase) | BF1 field (`FUN_0020f2e0`) | BF2 field (`FUN_00252bb0`) | use |
|---|---|---|---|
| SkeletonName 0x7012f6cd | +0x1d80 (rig), 123351 | +0x1148, 171863 → 0x002539c4 | §2's bone rule on that rig |
| SkeletonRootScale 0x4cacc3d0 | +0x1e5c, 123186-123194 | +0x1220, 172070-172080 | bone_root's +0x68 (when SkeletonName is set and not the default rig and ≠ 1); §7 |
| SkeletonRootScaleLowRes 0x26239cd4 | +0x1e64, 122969-122982 | +0x1228, 171680-171692 | the same scale on **`bone_pelvis`** (BF1 string 0x532b10; BF2 CRC 0xe6e47876) of the LOW-RES rig (+0x1d98 / +0x1154), when that is set and not the default low-res rig (BF1 *(DAT_00517cc0+0x54), BF2 *(DAT_005fde18+0x50)) |
| CollisionRootScale 0xc64a53ab | +0x1e60, 123571-123573 | +0x1224, 170980-170981 | §8 |
| OverrideTexture / 2 | +0x18d0 / +0x18d4 (UAUD1) | +0x844 / +0x848 (UAUD2) | §9 |

Defaults 1.0 (BF1 constructor `FUN_0020d238` 122003-122006; copied by `FUN_0020d940`). The bone is found by the CRC of
`bone_root` (0x16e27226; string 0x532a40). BF1 `FUN_0020e2e0` (122369-122395) has an easter egg: when `DAT_005178cc`
(written only by `FUN_0034d678` 337437, a profile-name hash test) is set, every class gets the `tat_inf_jawa` rig and
scale 0.6 [V; the name is not read].

## 4. The disc: each class's rig against the human clip [V disc]

`tools/uskn_rigs.rs` (a scratch mungefs example, run from a detached worktree; output `rigs_bf1.txt`, `rigs_bf2.txt`, `tools/summ.py`):
per bone the human rig's rest, the class rig's rest, the model `skel`'s bind and the clip's track.
- Human clips (`human_rifle_stand_walkforward`, both games) translate only DummyRoot and bone_root; every body track is
  untranslated (the bank's base = the human rig's rest).
- Ratio class-rig length / human, BF1: Wookiee 1.16-1.33 (bone_root 1.346 vs 1.159), battle droid head (0.219, −0.124)
  vs 0.131 and upper arm 0.025 vs 0.240, super battle droid forearm 1.62, Gungan 0.64-1.43 (root 1.274), Jawa 0.38-0.86
  (root 0.786), Ewok (root 0.548), Vader (rig `imp_darthvader`, root 1.307), Geonosian. BF2: MagnaGuard 1.13-1.56 (root
  1.439), marksperson 0.61-1.04 (root 1.097), Wookiee 1.17-1.33, both droids as BF1, Vader (root 1.456).
- The model `skel` equals its rig for every BF1 class but Vader (17 bones differ) and the battle droid's hp_weapons; BF2's
  `all_inf_wookie` model differs from its `wookie` rig on 22 bones (bind root 1.400 vs 1.346) - the RIG is what the PS2
  poses with. Same-named rigs in different side LVLs are equal (checked: wookie ×3, bdroid ×5, magnaguard ×2, marksperson ×3).
- One rig and one scale per soldier MODEL in both games (`census_model_rig.txt`: 10 BF1 and 22 BF2 own-rig models, none mixed).

## 5. The PS2's RAM [V RAM]

`tools/bonerecs.py <state>`: scans EE RAM for runs of 0x70-byte records whose +0x60 is a bone CRC and prints, per
skeleton, the rest lengths, which bones have bit 0, and any bit-1 scale. Output `ram_bf1.txt` (GEO1, END1, NAB1, KAS1),
`ram_bf2.txt` (MYG1, HOT1).
- "In use" is read from the missions' class lists (`census_model_rig.txt`, UAUD's census), not from RAM [D]; the scan
  itself reports each skeleton's bits.
- BF1, every rig a fielded class uses: bit 0 only on DummyRoot, bone_root, hp_weapons (+ hp_camera, extra bones) - the
  22-bone rule (the human rig too). Wookiee rig: flags 3, 1.15 on END1 and KAS1, all bits set on GEO1 and NAB1 (CW maps;
  the census fields the Wookiee classes on GCW maps and as KAS locals) [D]; Ewok 0.5 (END1); Gungan 1.1 (NAB1); the droid
  rigs follow the rule on the CW maps and keep all bits on END1 (a GCW map: loaded, not fielded) [D]; the droid and human
  rigs carry no scale.
- BF2: bit 0 only on DummyRoot, bone_root, hp_weapons for every soldier rig (human rigs included); MagnaGuard flags 3,
  1.24 (MYG1); marksperson 0.95, wookie 1.15, Vader 1.25 (HOT1); the droideka rig carries the 0x620aa0 list exactly.
  One human-sized rig on MYG1 (0x1854460) keeps all bits [D: a rig not fielded there].

## 6. The port today [V code, measured]

- `bake/src/soldier.rs` `clip_to_tracks`: every track's `translations` go into the GLB clip (DummyRoot's replaced by its
  rest); `mungefs/src/anim.rs` `decode_clip`: an untranslated track = `vec![t.base; frames]`; `game/src/skin.rs`
  `Clip::sample` uses a track's keys whenever it exists. So every soldier is posed at the human bank's lengths.
- The port reads `SkeletonName` only to pick BF2's animation family (`roster.rs`, `anim_sets/bf2.rs`) and
  `SkeletonRootScale` only for walkers; `OverrideTexture`, `OverrideTexture2`, `CollisionRootScale`: no reader
  [receipt:7a84241b].
- `tools/ps2rule_glb.py` rewrites a baked GLB's clips by §1-§3 (listed bones → the node's bind = the rig for these models,
  bone_root × scale); `tools/fk.py` gives the bone positions; `pose_preview` renders both (`pics/close_a.png` clone trooper
  + battle droid, `pics/close_b.png` Wookiee + MagnaGuard, `pics/skeleton_rule_sheet.png` five models, idle + walk, front
  + side). The feet stay on the ground under the rule (Wookiee toe y 0.034 → 0.038; MagnaGuard 0.034 → 0.068).
- PS2 frames: BF1 class select (`lead_tmp/unit-audit/pcsx2/03_cls_r2.png`: droid heads forward on a long neck) and BF2
  (`pcsx2_bf2/cis_sheet.png`: the locked MagnaGuard's silhouette towers over the droids).
- Walkers: BF2's walker hook `FUN_0025f050` (179247-179256) writes the same bit 1 / +0x68 on bone_root, so a walker's
  `SkeletonRootScale` is also a translation scale; the port's `viewer/walker_pose.rs` scales the whole subtree
  (`Mat4::from_scale`). No class in the port's bake authors the key on a walker: the key's hash (1286390736) has 17 hits
  in `assets_baked/classes`, every one a soldier class [receipt:2927b672], so the difference never shows; noted, no card.

## 7. The scale's other readers [O unless marked]

- BF1 class +0x1e5c is read in `FUN_002013a0` (113533, 113557, 113777, 113892, 113901, 114192, 114216), `FUN_0020e2e0`
  (122388) and the setter / constructor / copy ([receipt:3f15c787]); BF2 +0x1220 in `FUN_00239bb0`, `FUN_0024f350`,
  `FUN_00335920`, `FUN_00337b80`, `FUN_002517e8`, the setter / constructor / copy.
- **Clip choice:** BF1 `FUN_0022aa10` passes the animator's copy (+0x20c4, stored by `FUN_00229fc8` 141846 from class
  +0x1e5c, 113777) to `FUN_0022e990` (142626), which multiplies both velocity components by 1/scale (144992-144995) before
  the idle test and the clip choice. BF2 `FUN_0027ed18` the same with the animator's +0x60 (202177-202181, stored by
  `FUN_0027b710` from +0x1220, 153729).
- **BF2 not moving:** `FUN_0024f350` 168670-168700: moved² < (dt · scale · 0.2|0.3)² → not moving.
- **BF1 2 × scale** into `FUN_0047c870` (113531-113533, 113557, 114190-114192, 114216) [H: a ground shadow; not opened].
- **BF2 AI:** `FUN_00335920` (328375-328389), `FUN_00337b80` (329766-329781): a point on a target that IsA
  `DAT_00689828` is multiplied by the target's +0x1220 [H: which point; for card USKC to read].
- **The reset bracket:** BF1 `FUN_002013a0` 113889-113902 / BF2 `FUN_00239bb0` 153977-153994: while soldier +0xc14
  (BF2 argument 2[0x2a7]) ≥ 0 the animator step runs with bone_root's scale at 1.0, then restores it. +0xc14 is −1 by
  default (111439-111441) and written by `FUN_0020c880` (121294-121306) [H: its meaning].

## 8. CollisionRootScale [O]

- BF1 `FUN_0020e2e0` 122400: s = class +0x1e60; three stance sets at +0x1b60 / +0x1c14 / +0x1cc8 of `main_body`,
  `coll_body`, `coll_head`, `coll_body_lower`, `coll_body_upper` (strings 0x532a68-0x532aa8). Set 1: `main_body` capsule
  (0.57, 1.65) at y 0.9 unscaled; `coll_body` (0.35, 1.55 s) at y 0.8 s + 0.17 (max(s,1) − 1); `coll_head` r 0.17 max(s,1)
  at (−0.09, 1.65 s, 0.16); `coll_body_lower` r 0.57 at (0, 0.57, 0.15); `coll_body_upper` r 0.57 at (0, 1.9 s − 0.42,
  0.15). Set 2 uses 1.2 s / 1.6 s for head / upper. (re-soldier-walls §audit and re-soldier-ground §1.5 read the s = 1 case.)
- BF2 `FUN_002517e8` 170167: the same shapes with s (head 1.65 s, upper 1.9 s − r, and the main capsule 1.65 s at 0.9 s);
  BF2 also reads +0x1224 in `FUN_0024f0b0` (168302), `FUN_0024f350` (168821) and `FUN_00234a68` (149713) [H: stance
  rebuilds].
- Fielded values: Wookiee 1.15 (BF1 smuggler + locals; BF2 `all_inf_wookiee` / snow, Chewbacca, the Wookiee locals),
  BF2 snipers and Leia 0.95, Jawa 0.6, Grievous 1.10, Yoda 0.42.

## 9. Override textures [O]

- BF1 `FUN_002013a0` (the live soldier; one caller `FUN_00210fb0`): `if (class+0x18d0 == 0)` plain draw, else
  `FUN_004985f0` override list, group 0xdc5803e9 ← +0x18d0, and when +0x18d4 ≠ 0 group 0xb98e11c1 ← +0x18d4 (113711-113740
  for model argument 2[0x2fd], 114234-114256 for argument 2[0x90]). BF2 `FUN_00239bb0` 155274-155300 with +0x844 / +0x848.
- So OverrideTexture2 needs OverrideTexture in both games (UAUD2 said it of BF2 only). The segment's group is its `MNAM`
  (UAUD1 F5); the slot tables are unit-audit-bf1 / bf2 §3.3.
- In the port's bake (`assets_baked/side_bf*/textures`): `rebel_face01`, `rebel_face02`, `all_inf_soldier_hoth`,
  `all_inf_markspersonjungle`, `all_inf_vanguarddesert` present; `rep_inf_sharpshooter`, `cis_inf_battledroid_blue / red /
  orange / camo`, `all_inf_trooperurban / desert`, `all_inf_soldierdesert / urban`, `all_inf_vanguard_hoth`, `wookie_snow`,
  `wookieface02_snow`, `all_inf_wookie_snow` not among the files (`checks/tex_listing.txt`).

## 10. For the build (card USKN, phase build)

1. **Bake the pose rig's lengths** (`bake/src/soldier.rs` `clip_to_tracks`, the soldier bake's side inputs): per soldier
   model, the pose rig = the `zaf_` rig its classes' `SkeletonName` names (one per model), else the human rig
   (`human_animation_0` / `human_0`); for the game's listed bones (§2) write the rig's JONT `rest_pos` as the translation
   (keys or the node), not the clip's. Keep the GLB's inverse bind = the model `skel` (bone-space vertices → model space).
2. **SkeletonRootScale at run time, per class**: bone_root's translation × s in the drawn pose; the movement chooser and
   the clip rate on v / s (`animator.rs` `choose`, the 0.95 v / v_clip rate); BF2's not-moving threshold × s; the foley
   stride table follows the clip (`foley.rs`) - check it moves with v / s.
3. **Overrides**: the bake keeps each soldier segment's MNAM slot (`override_texture` / `override_texture2`) in the GLB and
   bakes every override texture the missions' classes name (from the class's own sublevel); the draw swaps the slot's
   texture per class; OverrideTexture2 only with OverrideTexture.
4. Not in this card: `GeometryLowRes` / `SkeletonLowRes` / `SkeletonRootScaleLowRes` and the 11-bone low-res list (when
   the port draws `_low1`, card LOD); CollisionRootScale (card USKC); the walker form (§6, unexercised).
5. **Acceptance**: a bake / pose test that the idle-frame head of the Wookiee (BF1) is 1.852 ± 0.01 m, the MagnaGuard 1.865,
   the BF1 battle droid 1.726, the clone trooper unchanged at 1.605 (`tools/fk.py` numbers); the rig rests equal the RAM
   records (`ram_*.txt`); goldens declared and viewed.

## 11. Checks run

- `re_check.py fn` on every cited line: `checks/fn.txt` (59 lines, each in the function named here).
- `re_check.py absent`: the port's keys [receipt:7a84241b]; the fields +0x1e5c, +0x1e60, +0x20c4, the bone flag bit 1,
  +0x1220, +0x1224 [receipt:3f15c787].
- Twins: `re_check.py callers` for both games' pose builders, flag functions, choosers and soldier functions
  (`checks/callers_b1.txt`, `callers_b2.txt`).
- `re_check.py word`: the `VMULx.xyz` words, `bone_root` / `DummyRoot` strings, the BF2 bone lists (`checks/word_writers.txt`).
- `re_check.py writers`: `DAT_005178cc` (one writer); soldier +0xc14 by `grep` (`FUN_0020c880` + two initialisers).
- Verifier: workflow `uskn-re-verify` (5 agents, effort medium; `checks/verifier_wf.json`). Three batch readers took 18
  claims and two data re-runs (C1-C18, D1-D2); two independent readers each took the 4 absence and 4 twin claims. 36
  verdicts on 28 claims: 29 SUPPORTED, 7 PARTLY, 0 NOT SUPPORTED. The second reader overturned nothing (A1 was PARTLY for both, the rest
  SUPPORTED for both). Folded in:
  - the low-res key's bone is `bone_pelvis` (C3);
  - `FUN_001765d0` is a fourth caller of `FUN_00281e68`, and both games' bone rules also run on the default rig (C7, T2);
  - BF2's idle test is circular and adds a 10/s limit (C9);
  - the bit-1 writers include the walker's undo `FUN_0025f210`, the loaders and the record copies (A1);
  - "in use" in §5 is read from the census, not from RAM (D2).
  D1's command reproduces the Wookiee and droid rows; the saved table came from a run with more side LVLs.

## 12. What stayed [H]

- What `FUN_0047c870` draws with 2 × scale (a shadow?); which point BF2's AI scales; what state soldier +0xc14 marks.
- Which name the BF1 Jawa easter egg tests.
- BF1's pose for a bone with bit 0 set and no clip track (hp_camera): not drawn by any vertex we checked; not read.
