# LEAD VERIFICATION (Fable lead, 2026-09-09, session 17)
**PARTIAL, and the split matters.** The CPU side is lead-verified: the EE's `f&0x2` → TEST 0x5060b agrees with `analysis/re-material-flags.md` §2's [V], and that is the half the port depends on. **The VU1 microcode reading is the AGENT's alone** — the dumps and the decoder are under `engine/artifacts/lead_tmp/s17/re/vu/`, and the lead did NOT re-read 0x800 = cull enable, 0x800000 = cull-side flip, the facing loop at 0x640–0x663 or the ADC bit at 0x393–0x3a1.
**VERDICT:** no ALPHA RULE v3 and no cull item come out of GEO1 — the wreck's "bare rafters" is NOT in the material path (the port's one `cull_mode: None` pipeline already draws everything two-sided, and the five hull segments carrying 0x02 are MASK in kind). The strongest remaining candidate is [H]: the port draws LOD0 forever while the LOWD's rafters texture is 100 % opaque, and the PS2's `modl` LOD switch rule is unread — **HOLD FOR FABLE 20**, with §4's two-station PCSX2 capture note for the maintainer.
**Lead composition [D] → queue item AT (the MASK cutoff 0.5 → 192/255):** §5 compares the port's 0.5 with the PS2's AREF 0x60 = 96/255 WITHOUT the `pal_` loader's ×129/256 CLUT scale (`analysis/re-hud-icons.md` citation 3, lead re-read). With it, the PS2 keeps a texel only at DISC alpha ≥ ~192, so the port's cutoff in disc terms is ≈192/255, not 128/255. Everything below this header is the RE agent's report, verbatim.

# GEO1 Techno Union wreck — MTRL bits 0x1/0x2/0x10000, alpha test, two-sidedness (Fable RE agent, s17, 2026-09-09)

Tags: [V] read in decomp / ELF bytes / disc / VU1 microcode / port source, [D] derived, [H] hypothesis.
D: = `analysis/SWBFRNT2.ELF-decomp.c`, B: = `analysis/SWBFRONT.ELF-decomp.c`. VU1 = local microcode addresses in `.vutext`
(BF2 ELF off 0x4935c0 len 0x61c0, BF1 0x3b23c0/0x4260; mapping per `re-texture-wrap.md` §1.3). Dumps + decoder:
`engine/artifacts/lead_tmp/s17/re/vu/{bf2,bf1}_vu1_v3.txt`, `vudis3.py` (vudis2 + full upper-op decode). segdump copy: `re/bin/segdump`.

## 1. Load-bearing citations
1. D:723411–723487 `FUN_0058ed70`: translucent = c.a≠0xff | f&4 | F&0x800000; ALPHA 0x44/0x48/0x64; TEST 0x51001 / 0x5000b (f&2) /
   0x50000; **723484–723487: `prim |= 0x800800` iff `((f&6) && !(f&0x10000)) || (c.a≠0xff && !(F&0x4000000) && !(f&4))`**;
   723489–723491: `prim &= ~0x4000` if F&0x10, no light context, or !(f&1). D:723553–723563: material qword = {x: ALPHA|prim,
   y: FIX, z-slot(rec+8): `DAT_0060c64c<<4 | TEST` = AREF 0x60, w-slot(rec+0xc): PRIM 8/0x18 |0x40 if ALPHA}.
2. D:724813–724826, 724860 `FUN_005916b0`: `seg+0x04` prim = 0x4000 if f&1, |0x100000 if COL0, |0x2000 if f&0x200&&f&1; `seg+0x18 = f`.
3. D:555875 `FUN_004979a8`: DL opcode 0x406 = header {0x406, presence bits} + per material {u64 rec+0 (x,y), u16 rec+0xc (z=PRIM),
   u32 rec+8 (w=TEST)}. D:724027–724030 + 555340–555360 `FUN_00497570`: opcode 0x427 {mask&~set, set}, set = 0x100 iff F&1.
4. VU1 BF2 0x406–0x416: material copy; **0x411–0x416 `LOI 2^31; ITOF0; ADDi; MTIR vi1,vf2.x; ISW.w vi1,-3(vi4)` → slot[6].w = x>>8**
   (ALPHA byte shifted out; 0x800→8, 0x4000→0x40, 0x800000→0x8000). 0x162 `ILW.w vi8,6(vi15)`; 0x174–0x17f lit (0x40) → light-set
   jump table; **0x1a0–0x1a3 `8 & vi8 → 0x393`**; 0x1a5–0x1ad `0x100 & mem[7].x → 0x311` (CLIP.xyz ×3 at 0x331–0x33c, FCAND 0x3ffff).
5. VU1 BF2 **0x62a–0x634: `(slot8|slot17|slot26|slot35)[6].w & 0x108 → 0x640`; 0x640–0x663 facing loop**: per vertex `MTIR vi4,vf1.w`,
   clear 0x4000, transform, `OPMULA/OPMSUB` ×2 (rolling strip cross products), triple-product MAC flags, `FMAND vi3,0x20` (Sz sign),
   `vi3 = ((Sz?0x20:0) + 0x3fe0 + seed) & 0x4000` = sign XOR seed → written back into the vertex w (`ISW.w vi4,-1(vi2)`).
   **0x393–0x3a1: per output XYZ2 w: `w |= (w<<1) + (0x8000 if slot[6].w & 0x8000)`** → 0x4000 becomes ADC (bit 15); x-bit 0x800000
   flips which side (16-bit add: 0x8000+0x8000 wraps to 0). BF1 twins byte-for-byte: 0x18d, 0x393–0x3a1, 0x624–0x62a, 0x636–0x659.
6. D:724440–724459 `FUN_00590c38` (load-time packet): vertex w = 0x8000 (ADC) from STRP bit 15, |0x4000 seed while *(decompiler detail omitted)*.
   B:549452–549460: debug toggle picks 0x800800 / 0x800 (`DAT_00522a48`) / 0 (`DAT_00522a4c`) — the cull-flip/cull-off switch.
7. Port: `engine/crates/render/src/lib.rs:1493–1497` the ONLY `PrimitiveState` in engine/crates: `cull_mode: None` ("mixed winding").
   `engine/crates/bake/src/alpha.rs:190–232` rule 3 order: CUTOUT(0x02) → MASK first (221), TRANSPARENT (223), class D (229).
   `mungefs/src/entity.rs:101–108` `best_model` = LOD0 always; no LOD-distance code in game/render/bake (grep).
8. Disc + GLB (measured): `segdump GEO1.LVL geo_bldg_technounion_hull` (table below); `assets_baked/bf2_geo1/models/geo_bldg_technounion_hull.glb`
   extras: SIX MASK primitives (0,1,2,3,7 rule 3 via 0x02; 6 via class D), 4/5 OPAQUE. Baked KTX2 mip 0 (vkFormat 43, raw RGBA8):
   `geo_bldg_wrecked_02` a<96 0.2318, a<128 0.2453, band[96,127] **0.0134**; `geo_bldg_rafters` 0.2783/0.2920/**0.0137**;
   `geo_bldg_rafters_lowrez` and `geo_bldg_wrecked` **0.0000** (fully opaque); `gmod geo_bldg_technounion_hull` @0x335F98 INFO=(0.0,1.0),
   LOD0 trailer 0xa55=2645, LOWD 0x37f=895 (= index counts, NOT distances).

## 2. The three answers
**Q1 — which word is which, and what 0x1 / 0x2 / 0x10000 do** [V unless noted]
- Three words. (a) MTRL `f` (disc u32 → `seg+0x18`); (b) draw flags `F` (argument 5 of `FUN_0058ed70`, the caller's word — soldiers
  0x80001, sky 0x20031 …); (c) prim `p` (`seg+0x04`, built at load from `f`, patched at draw, OR'd with ALPHA into material qword x).
- `f&0x1` LIT → `p|=0x4000` → VU1 slot[6].w 0x40 → per-vertex lighting path (0x174–0x17f). No GS effect. `f&0x2` → GS TEST_1 =
  0x5000b|0x600 = **0x5060b** (ATE, GEQUAL, AREF 0x60=96/255, AFAIL KEEP, ZTE GEQUAL, Z written); no ABE. `f&0x10000` → suppresses
  `p|=0x800800` for 0x02/0x04 segments = **VU1 backface culling OFF (two-sided)**; it has no other reader (grep 712000–726500).
- `F&0x1` → opcode 0x427 → VU1 `mem[7].x ^= 0x100` (0x427–0x438: state = (old&mask) XOR set) → the 0x311 draw path with CLIP/FCAND
  [D: near/frustum CLIPPING for things that reach the camera — soldiers, weapons, UI, sky; buildings never get it]. Not a GS state.
- `F&0x10000` (with 0x20000/0x200000 clear) → `FUN_00592c38(model+0xb8)` (D:712963, 725798–): a matrix-chained child list gated by
  `DAT_0060c66e` (`FUN_0045f540` argument 2) — sub-models [H attached parts]; no material or GS effect.
- `p` bits the VU1 reads (after >>8): 0x4000 lit, 0x2000 vc-lit, 0x100000 COL0, 0x100 UV-matrix program, **0x800 = cull enable,
  0x800000 = cull-side flip**; the EE always sets the pair, so "0x800800" = cull one facing. `f&0x10000` and BF2's near-universal
  0x10000 make BF2 cutouts/blends two-sided; plain opaque segments (0x1, 0x10001) NEVER get 0x800800 → two-sided in both games.
  Which facing 0x800800 culls (CW/CCW in world terms) depends on the seed parity convention — not derived here [D].

**Q2 — are the MASK segments alpha-tested on the PS2? YES for the ones that matter** [V]
| seg | tris | MTRL | texture | PS2 TEST_1 / cull | port GLB (rule) |
|---|---|---|---|---|---|
| 0 | 19 | 0x3 | geo_bldg_wrecked_02 | 0x5060b test@96; **0x800800: one side culled** | MASK 0.5 (3, 0x02) two-sided |
| 1 | 342 | 0x10003 | geo_bldg_rafters | 0x5060b test@96; no cull | MASK 0.5 (3, 0x02) |
| 2 | 190 | 0x10003 | geo_bldg_wrecked_02 | 0x5060b; no cull | MASK 0.5 (3, 0x02) |
| 3 | 88 | 0x10003 | geo_bldg_rafters | 0x5060b; no cull | MASK 0.5 (3, 0x02) |
| 4 | 459 | 0x10001 | geo_bldg_wrecked | 0x50600 opaque; no cull | OPAQUE (3) |
| 5 | 319 | 0x10001 | geo_bldg_technounion | 0x50600 opaque; no cull | OPAQUE (3) |
| 6 | 5 | 0x10001 | geo_bldg_wrecked_02 | **0x50600 OPAQUE, alpha ignored**; no cull | **MASK (3, class D)** — disagrees |
| 7 | 87 | 0x10003 | geo_bldg_wrecked_02 | 0x5060b; no cull | MASK 0.5 (3, 0x02) |
- The brief's premise is off: the MASK verdicts are not "four by the unflagged class-D clause" — five segments CARRY 0x02 and are MASK by
  rule 3's first clause, which AGREES with the PS2 in kind (cutoff 0.5 vs 96/255: the [96,127] band is 1.3 % of texels on both textures,
  so the port discards ≈1.3 % more of the plating than the PS2 — invisible at golden distance). The class-D clause fires on ONE segment,
  seg 6, 5 triangles, which the PS2 draws opaque (the standing s12 disagreement 6(b), tiny here). **Rule 3 is not what makes the wreck
  read as bare rafters; no ALPHA RULE v3 is decided by GEO1.** The only PS2-vs-port alpha delta on this model is 1.3 % of texels + 5 tris.

**Q3 — two-sided? PS2 draws segs 1–7 two-sided; the port draws EVERYTHING two-sided** [V]
- The GS has no cull; VU1 culls by converting a per-vertex facing flag to ADC, and does so only when the material x carries 0x800
  (citations 4–5). The EE sets 0x800800 only for 0x02/0x04 segments lacking `f&0x10000` and for colour-faded non-0x04 segments
  (723484–723487). Segs 1–7 carry 0x10000 → no cull → two-sided. Seg 0 (0x3, 19 tris) is one-side culled on the PS2.
- The port's single pipeline has `cull_mode: None` (render/lib.rs:1496) and nothing reads a GLB `doubleSided` (grep: none written, none
  read) → the port already draws both faces of every segment. It draws MORE of the wreck than the PS2 (seg 0's back faces), not less.
  **Two-sidedness cannot be the cause; no per-segment cull item is warranted for the wreck** (a cull flag would only remove seg 0's 19 tris).
- Not settled by the CPU side, and not needed for the wreck: which facing 0x800800 culls (seed parity at D:724440–724459 vs the Sz sign).

## 3. Draft RE_LOG.md entry (≤ 20 lines)
### Backface culling is VU1-side, per material record, and OFF for plain opaque segments (2026-09-09, Fable s17) — GEO1 wreck: not a material bug
Full note: `engine/artifacts/lead_tmp/s17/re/geo1_wreck.md` (VU1 dumps + decoder under `re/vu/`). Answers HOLD 10(g)'s two-sided half
and re-reads `re-material-flags.md` §2's [H] on 0x800800/0x10000 as [V].
- [V] The material qword's x word (ALPHA | `seg+0x04` prim bits) is converted by VU1 to `slot[6].w = x>>8` (BF2 0x411–0x416); the bits
  it tests are 0x4000 lit, 0x2000 vc-lit, 0x100000 COL0, 0x100 UV-matrix, **0x800 = cull enable, 0x800000 = cull-side flip**. With 0x800
  on any of a batch's materials VU1 runs a facing loop (0x640–0x663: rolling OPMULA/OPMSUB over the strip, FMAND Sz, XOR a load-time seed
  bit in the vertex w) and the material's draw (0x393–0x3a1) turns that bit into the GIF ADC bit (`w |= (w<<1) + flip`). BF1 identical
  (0x18d / 0x393 / 0x624 / 0x636). The GS never culls.
- [V] The EE sets 0x800800 only for `(f&6) && !(f&0x10000)` and for colour-faded non-0x04 segments (D:723484–723487; B:594369 twin:
  translucent && mode 0; B:549452–549460 is a debug cull-flip/off switch). So **MTRL 0x10000 = "two-sided cutout/blend"**, and every plain
  opaque world segment (0x1, 0x10001) is drawn two-sided on both consoles. Draw-flag 0x1 → opcode 0x427 → VU1 state 0x100 → the CLIP/FCAND
  path [D clipping]; draw-flag 0x10000 → `FUN_00592c38` sub-model list, not material.
- [V] GEO1 `geo_bldg_technounion_hull`: five segments carry 0x02 (0x3, 0x10003 ×4) → PS2 TEST 0x5060b (test at 96/255) = the port's
  MASK in kind ([96,127] band = 1.3 % of texels on `geo_bldg_wrecked_02`/`geo_bldg_rafters`); segs 1–7 two-sided on the PS2; the port
  draws everything two-sided (`render/lib.rs:1496`). Class D fires on one 5-triangle segment (PS2 opaque). The wreck's "bare rafters" is
  NOT explained by the material path. Port always draws LOD0 (`entity.rs:101`); the PS2 LOD switch rule is unread — the LOWD's rafters
  texture (`geo_bldg_rafters_lowrez`) is 100 % opaque, LOD0's is 28 % holes, so at the golden's ~475 m eye the two consoles may be
  drawing different models. Decision: no ALPHA RULE v3, no cull item from GEO1; PROVISIONAL capture note instead.

## 4. PROVISIONAL note (no queue item) — the PCSX2 capture that settles the wreck
- Not decided by this RE: what the PS2 actually shows of the wreck at the golden's framing. Everything the material path controls is
  now [V] and matches the port to within 1.3 % of texels, 5 triangles, and 19 back faces the port draws extra.
- **Capture (the maintainer's rig, HOLD 7):** BF2 GEO1 (Geonosis, either side), PCSX2 GS dump (Debug → GS dump, or a savestate + GSdx dump) at two
  camera stations: (1) standing ~30 m from `geo_bldg_technounion_hull` (the crashed core-ship hull; the golden's left panel lower-left
  wreck; ~x −21..37 m, y −11..14 m, z −14..14 m in model space) so LOD0 is drawn; (2) the golden's vantage: eye ≈ centre +
  (0.75r, 0.55r, 0.75r), r ≈ 398 m (`viewer::overview_camera`), ≈475 m from centre, so the far LOD is drawn.
  In the dump, on the hull's draws: TEST_1 (expect 0x5060b on the 0x02 segments, 0x50600 on the 0x1 ones), ALPHA_1's low byte (expect 0:
  no blend), PRIM (0x18/0x58; no ABE), the XYZ2 ADC bits (both facings present on segs 1–7; one facing missing on seg 0), and which
  `modl` (2645 or 895 indices) is kicked at station (2). Compare the frame to `engine/goldens/maps/bf2_geo1.png` left panel.
- If station (2) kicks the 895-index LOWD, the "bare rafters" is the port's LOD0-forever policy (LOWD's rafters texture is solid) and
  the fix is a LOD-distance rule (needs the ELF's switch rule: `FUN_0057cf60` far/fade 712650–712828 reads only the sphere; the gmod
  INFO floats are (0.0, 1.0) and the per-LOD trailer is the index count — the distance is elsewhere, unread).

## 5. Still open
- The PS2 LOD switch rule (LOD0 ↔ LOWD distance/screen-size) — not in `gmod`; a separate RE (`FUN_00592c38`'s *(decompiler detail omitted)*
  at D:712968 is a distance compare on the sub-model list — the same shape may pick the LOD; not traced).
- Which facing 0x800800 culls (seed parity at D:724440–724459 vs `FMAND 0x20`) — cosmetic for seg 0 only.
- The x-bit 0x10000 (`slot[6].w & 0x100`) that also enters the 0x62a gate and the 0x192/0x197 UV jump table: no EE model path sets it
  (terrain/particle records `DAT_0076c0f0/0x100` = 0x800800/0x800944 don't either) — dead or another program's bit.
- ALPHA RULE v2 rule 3's class-D clause still contradicts the ELF (s12 item 6(b)); GEO1 adds one 5-triangle case, not a decision.
- The port's alphaCutoff 0.5 vs the PS2's 96/255 (AREF 0x60): 1.3 % of texels here; a global AREF change is a v3 question, not GEO1's.
