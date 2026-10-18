# OPUS RE (loop lead, 2026-09-24, lane 1) - [O] UNAUDITED: no Fable pass

**Queue SPEC: what is the PS2's specular second pass?** BF2 draws it for a segment whose `MTRL` word carries 0x80
and BF1 for its Specular segment class (word bit 0x20). `analysis/re-material-flags.md` §2 named its GS record and
said a "highlight texture" came from the specular power (`FUN_00589ac0`). This document answers the entry's four
questions.

**The answer.** The pass does NOT build a texture from the power. Both games sample one shared 128×128 grey
lookup table named `specular`, stored in CORE.LVL. Its S coordinate is `power/255`. Its T coordinate is
`0.5 + 0.5·(L̂·R)`, where R is the reflection vector the VU1 program computes per vertex. The pass colour is
flat, with no lighting: the MTRL's specular colour on BF2, and on BF1 the sun colour unless the material names a
colour of its own. That colour multiplies the table texel. The result is added to the frame weighted by the
DESTINATION alpha, which is the base pass's own texture alpha: an `Specmap` sheet's alpha channel is its gloss mask.
Four quirks are faithful to the disc and matter to a port:
- **The table is symmetric.** A highlight appears when R points along the light and again when it points
  against it (§5).
- **The frames are mixed.** The light is rotated into a camera frame whose up and back axes are negated relative
  to the frame R lives in. So the effective light has its camera-right component flipped (§4).
- **BF2's translucent (0x04) variant is invisible.** Its blend weighs by the source alpha, which works out to
  1/128 (§6).
- **BF2 skips far-scene models.** A far-scene model in a mission never gets the pass (§7).

Tags: [V] read it (lines, instruction words or disc bytes cited); [D] derived, from what is said; [H] hypothesis.
`[O]` means one Opus pass plus one Opus verifier agent, with no Fable audit (§10).

Scratch, every script and every output: `engine/artifacts/lead_tmp/re/specular/`. The claim list the verifier was
given is `claims.md`, and its verdicts are `verify/VERDICTS.md`.

Conventions:
- **Decomp lines:** BF2 = `analysis/SWBFRNT2.ELF-decomp.c`, BF1 = `analysis/SWBFRONT.ELF-decomp.c`.
- **ELF offsets:** both ELFs have one PT_LOAD, file off = va − 0x15f000.
- **VU1 addresses** are local microcode addresses of the main program in `.vutext` (BF2 file 0x4935c0 len 0x61c0,
  BF1 0x3b23c0 len 0x4260). A is at 8-byte slot 2 + A + A//256, because an `{NOP, MPG}` tag sits at slot
  2 + 257k (`vu1dis.py tags`).
- **Two defects in the s12/s17 VU decoder** (`lead_tmp/s12/re/wrap_scratch/vudis2.py`, `s17/re/vu/vudis3.py`), so
  this job used a fresh `vu1dis.py`:
  - It prints the MPG tag word instead of the real instruction at every A = 0x100·k (k ≥ 1).
  - It prints `SQ`/`SQI` operands swapped: the encoding is `SQ.dest fs(15–11), imm(it 20–16)`.
  - **Consequence for `re-texture-wrap.md` §1.3:** its "0x4c9 SQ.yzw vf2,0(vi1)" is really `SQ.yzw vf1,0(vi2)`.
    Pair 3 (CLAMP_1), the lead-verified half, is symmetric (vf2/vi2) and unaffected.
- **R5900 + COP2 macro instructions** were decoded by `eedis.py`. Ghidra's `_vmulabc` listing drops the broadcast
  field; every VU0 claim below was checked against the instruction words.

## 0. The answer in one table

| question (queue SPEC) | answer | tag |
|---|---|---|
| what `FUN_00589ac0` builds | a 4×4 texture-coordinate matrix (a DL opcode-1000 packet), not a texture: S = power/255, T = 0.5 + 0.5·(L̂_C·R) (§3) | [V][O] |
| the highlight texture | the disc's `specular` `tex_` in CORE.LVL: 128×128, 8-bit grey, 1 mip, texels byte-identical on both discs, sampled bilinear with REPEAT (§5) | [V][O] |
| the pass's UVs | tex-gen from the view-space reflection vector R = 2(N'·Î)N' − Î (VU1, per vertex); the matrix dots R with the view-rotated light (§3, §4) | [V][O] |
| the pass's colour | flat per pass (the VU1 unlit path). BF2: the MTRL specular RGB × the draw's alpha factor, halved into GS units (0xff → 1.0). BF1: the SunInfo colour raw (0x80 = 1.0, so 240 ≈ 1.9×), or the material's own RGB raw (§2) | [V][O] |
| what Ad holds | the base pass's fragment alpha = the base texel alpha × ~1.0: the main target is PSMCT32 with FBMSK 0 and the base pass neither blends nor tests (§6) | [V] target, [D] Ad [O] |
| BF1's twin | the same matrix, the same VU1 R, the same table; it differs in its 4-qword packet and its Q from row z, its colour source, its record (no alpha test) and its far-scene behaviour (§8) | [V][O] |

## 1. Which segments draw the pass [V][O]

- **BF2:** `FUN_0058ed70` 723879–723880: `if ((w & 0x100) == 0 || (no env texture)) { if ((w & 0x80) == 0) goto
  emit; … }`. So 0x80 adds the pass unless 0x100 plus an env texture wins. The pass's GS record is `0x7c57b0`
  without 0x04 and `0x7c57c0` with it (723894–723900). Its texture is `DAT_007c579c` (723908) and its matrix is
  `FUN_00589ac0(ftoi(seg+0x3c))` (723909–723910).
  - The texture word is set by `FUN_00591a30` 724932 from the name at 0x650038 = `"specular"` (ELF file 0x4f1038).
    The verifier found the stored value is the name's FNV-1a hash 0xbe73c6e0, resolved to the texture at bind.
  - This is the one per-segment draw for every `RedModel` (`re-material-flags.md` §1), soldiers included
    (`re-side-alpha.md` §3).
- **BF1:** word bit 0x20 selects the class `ps2SpecularSegment` (vtable 0x5594a0, RTTI "18ps2SpecularSegment" at
  0x5594e8; factory `FUN_004aa9f8` 568150–568156). Its draw `FUN_004b2310` (vtable+0xc) always emits two passes:
  - the segment's own texture;
  - `DAT_006f4530`, looked up from the name at 0x5594d8 = `"specular"` (`FUN_004b27a0` 573450–573451).
- **Census over every LVL on both discs** (`census_spec.py` → `census_spec.txt`, reproduced independently by the
  verifier):
  - **BF1:** 179 segment records carry 0x20. Their words are 0x31 / 0x30 / 0x71, RTYP `Specmap`.
    - Powers A ∈ {4, 5, 8, 9, 12, 15, 16, 24, 32, 45, 50}.
    - Colours B take 9 distinct values: 0, 0xffffff and seven others.
    - Maps: KAM1, KAS2, YAV1/YAV2 (the bazaar columns), RHN1, NAB2, BES2.
    - Side models: the ALL/IMP/REP/CIS weapons, speeders, `imp_inf_darktrooper`, `cis_inf_bdroid`.
  - **BF2:** 469 records carry 0x80, and **none also carries 0x100**, so the env pass never displaces it on the
    disc.
    - Powers run 1..300 (30 distinct). Every spec colour has alpha byte 0xff.
    - Maps: DEA1 (dozens), TAN1, UTA1, KAM1, FEL1, POL1, TAT2, TAT3, YAV1, COR1, GEO1, NAB2, GAL1.
    - Side models: 12 soldier segments (`0x10089`) and the first-person and third-person weapons.
    - 22 records also carry 0x04: DEA1, POL1, FEL1, GAL1, KAM1, CIS.
    - **0 carry the cutout bit 0x02**, so no spec pass sits on an alpha-tested base (counted from
      `census_spec.txt`).

## 2. The pass's GS state and colour [V][O]

- **BF2 records** (`FUN_00591a30`, instruction words 0x591afc–0x591bf0):

  | record | u64 at +0 | TEST at +8 | u16 at +0xc |
  |---|---|---|---|
  | 0x7c57b0 | 0xff00000158 | 0x5001b | 0x58 |
  | 0x7c57c0 | 0xff00000148 | 0x50000 | 0x58 |

  - **0x7c57b0:** GS ALPHA 0x58 = **Cs·Ad + Cd** (FIX 0xff unused). The test is ATE with GEQUAL and AREF 1, plus
    ZTE GEQUAL.
  - **0x7c57c0:** ALPHA 0x48 = **Cs·As + Cd**, no alpha test, ZTE GEQUAL.
  - PRIM 0x58 = IIP|TME|ABE.
  - The 0x100 in the first word is a VU flag, not a GS field. The VU1 material copy (opcode 0x406, 0x411–0x416)
    stores `word >> 8` = 1 into the pass slot's [6].w. The dispatch at 0x197–0x19e reads `& 3` = 1 and so routes
    the pass to tex-gen (§3). 0x40, the lit flag, is clear, so the colour takes the unlit path (below).
- **BF1 record** `0x6f4540` (`FUN_004b27a0`, words 0x4b27d0–0x4b27f8) = {0xff00000158, TEST **0x50000**, 0x50}:
  - ALPHA 0x58 = Cs·Ad + Cd, the same as BF2, but **with no alpha test**.
  - The base pass (pass 0) uses TEST 0x50600 (ATE off, 573377) and blends only when the draw colour's alpha is
    not 0xff (573363–573366). BF1's Specular base is therefore opaque whatever its bits 0x02/0x04 say
    (`re-material-flags.md` §0.4).
- **The pass colour is flat and unlit in both games.**
  - The VU1 colour opcode (BF2 0x43a–0x448, BF1 0x438–0x446) stores c/128 floats, so 0x80 = 1.0.
  - A pass whose slot[6].w lacks 0x40 takes the unlit branch: BF2 `IBEQ vi5,vi0 -> 0x2a3` at 0x178, BF1
    0x16e → 0x2a7.
  - That branch writes the same constant into every vertex's RGBA (BF2 0x2b3–0x2bc: `ADDi` 65536.0 packs c back
    into a byte).
  - With TEX0 MODULATE (`re-material-flags.md` §0.5): pass RGB = min(255, texel × c >> 7).
- **BF2 colour** (`FUN_0058ed70` 723882–723893 and 723955–723957; bytes 0x58fdc0–0x58ff2c):
  - **RGB:** RGB_i = ((byte(spec_i · f · 255) + 1) >> 1).
    - spec_i = the MTRL's third u32, byte 0 = R, divided by 255 (`FUN_00590328` 724210–724218 →
      `FUN_005901b0` 724071–724080 → `FUN_005916b0` 724854–724858).
    - f = *(decompiler detail omitted)*/255: 1.0 unless the draw is translucent, when it is ≈ c.a/255 (723387, 723449–723450).
    - **So a spec byte of 0xff gives a pass colour of 1.0, and the pass adds the texel itself.**
  - **Alpha:** the pass's A byte is `int(spec.a/255)`, UNSCALED: 0x58fe00 `lwc1 56(s3)` → 0x58fe64 `cvt.w.s` →
    stored at sp+147 → halved at 0x58fef0. **Its value is (1+1)>>1 = 1.** The scaled alpha is written to sp+163 and
    never read. Found by the verifier, then reproduced from the bytes.
- **BF1 colour** (573408–573425): pass 1 gets B = seg+0x40 **raw** when B ≠ 0 and (B & 0xffffff) ≠ 0xffffff.
  Otherwise it gets the four bytes of `DAT_0069c588` raw (`FUN_00499418`).
  - `DAT_0069c588` is the directional light's colour: `FUN_004930d8` forces A = 0xff (555510–555516).
  - The SunInfo scope handler `FUN_002e4dc8` sets it from `Color` (hash 0x3d7e6258 = FNV-1a of "color";
    271399–271430 → `FUN_004993f8`), clamped to 0..255 and **not halved**. The default is 80 80 80 ff = 1.0
    (555824). A weather effect `FUN_00378120` flickers it.
  - **So on BF1 the SunInfo colour reaches the GS as-is: YAV1's (240, 240, 235) is ≈ 1.88× on the table texel,
    saturating at 255.** A per-material B is raw too: `0xffc9d0` → R 0xd0 = 1.63×.
  - B and the power A come from the 12-byte MTRL {flags, A, B}, read as int, converted to float (`FUN_004aacd8`
    568288–568302) and stored back as int by the class's slot +0x1c `FUN_004b2948` (seg+0x3c = A, seg+0x40 = B;
    words 0x4b2948–0x4b2954). Every B on the disc is < 2^24, so the float round trip is exact.

## 3. The texture coordinates [V][O]

- **The EE matrix.** `FUN_00589ac0` (BF2; instruction words 0x589ac0–0x589c5c):
  - It takes three qwords q0 q1 q2 at `DAT_0060c240` (the active camera) + 0x70/0x80/0x90, and L = the qword at
    `DAT_0060c6e0 + 0x50`.
  - It forms L_C = q0·L.x + q1·L.y + q2·L.z (`VMULAx.xyz / VMADDAy.xyz / VMADDAz.xyz / VMADDw.xyz`, words
    0x4bc219bc 0x4bc228bd 0x4bc220be 0x4bc008cb). It normalises that (VRSQRT) and sets H = −L̂_C (`VSUB.xyz vf2,
    vf0, vf1`).
  - It writes a 4×4 m with every other entry 0:

    | entry | value |
    |---|---|
    | m1, m5, m9 | s·H = 0.5·L̂_C (the three components) |
    | m12 | power/255 (constant 0x3b808081) |
    | m13 | c |
    | m14 | −1.0 |
    | m15 | 1.0 |

  - s = `DAT_0060c4f4` = 0xbf000000 (−0.5) and c = `DAT_0060c4fc` = 0x3f000000 (0.5), read from .data. No store
    to either exists (byte scan and decomp grep; the verifier's scanner also covered unaligned and pointer forms).
  - `FUN_00498be0` packs m as five qwords. qword 0.w = the flags 0x1000 (0x9000 only if the byte `DAT_0060c368`
    ≠ 0: it is 0 in .data and has no writer). qword 1.w = 0.0. Qword 4 = the w column (m3, m7, m11, m15).
  - `FUN_00497ab0` sends it as DL opcode **1000** (555951), one packet per extra pass.
- **VU1 storage and dispatch** (BF2):
  - Opcode 1000 (0x3e8–0x3fc) stores pass i's packet at mem 12+9i..16+9i, i.e. slot (8+9i)+4..+8.
  - The pass dispatch (§2) reaches 0x077.
  - 0x077–0x082 jump on `slot[4].w & 0x1800` → 0x1000 → 0x019 → **0x0ca**.
- **R per vertex** (0x0ca–0x0fe):
  - vf5..vf8 = mem[52..55] × mem[56..59] = view × model (opcodes 0x3c4 and 0x3d1; EE `FUN_00497798`,
    `FUN_00497868`).
  - N' = the model-view 3×3 · n (the stored normal, dequantised).
  - P' = model-view · (p, 1).
  - Î = P'/|P'| (`ERLENG`), the eye → vertex direction in view space.
  - **R = 2(N'·Î)N' − Î** (0x0f0–0x0f6). R is written back over the normal stream (0x0fe).
  - With the 0x8000 flag R.z would become |R+ẑ|: the sphere-map form the env pass uses. It never fires here.
- **The per-pass loop** (0x100–0x13c):
  - (S, T) = slot[7] + slot[4]·R.x + slot[5]·R.y + slot[6]·R.z.
  - q = max(R·(m3, m7, m11) + m15, 0), which is 1 here.
  - STQ = (S/(w·q), T/(w·q), 1/w).
  - The GS samples **s = power/255, t = 0.5 + 0.5·(L̂_C·R)**.
  - The verifier notes a q < 0 kill and a second sign test on the z output, which m14 = −1 always passes.
- **Texture state.** CLAMP_1 is REPEAT for every texture (`re-texture-wrap.md` §1.3). This pass's CLAMP high word
  carries the per-pass 5 at *(decompiler detail omitted)* (723391, 723958), which REPEAT ignores.
  - So **s wraps**: power 255 → s = 1.0 → column 0; power 300 → s = 1.18 → column ~22.
  - Bilinear filtering, no mips.
- **L** is the map's global directional light (`FUN_00596380` 728270–728310 picks the `lght` light of type 1 named
  as global light 1). +0x50 is its direction. [D] That is the direction of TRAVEL:
  - BF1's default is (0, −1, 0) (556595–556599);
  - BF2's shadow code tests `L.y < −0.5` as "sun high" (94970).
  - **The sign is second-order:** the table is symmetric (§5).
- **BF1** (`FUN_004a82f8`, words 0x4a8380–0x4a8440) builds the same m from the camera's +0x40/0x50/0x60 and
  L = `DAT_0069c590` (the SunInfo direction, `FUN_004931e0`/`FUN_00493128`), with the same s = −0.5 and c = 0.5.
  - Its m14 = +1.0 and m15 = 0.
  - Its packet (`FUN_0039da50`) has 4 qwords and no w column.
  - Its VU1 (0x0ca–0x101) computes the same R.
  - Its loop (0x10c–0x145) takes STQ = (M·(R,1)).xyz / w, so Q comes from row z = m14 = 1.
  - **Same s and t.**

## 4. The two camera frames — the effective light is mirrored [V] frames, [D] effect [O]

- **R's frame.** The VU1 view matrix, mem[52..55], is built by `FUN_00580830` (BF2; `neg.s` at 0x580874–0x5808a4)
  from the camera's world matrix at `cam+0x30..0x6c` (rows X, U, B, t).
  - Its columns are (X, −U, −B): **V-frame = (X·d, −U·d, −B·d)**, the PS2 screen frame with y down and z forward.
  - Its callers `FUN_005828d8` 716885–716900, `FUN_00589fa8` 720873–720888, `FUN_00587278` and `FUN_00588968` all
    pass the camera's +0x30 block.
- **L's frame.** `cam+0x70..0xac`, the rows `FUN_00589ac0` rotates L by, is written by `FUN_0057a878` (words
  0x57a888–0x57a8e8) as the **plain rigid inverse: C-frame = (X·d, U·d, B·d)**.
  - The camera's matrix setter `FUN_0057b810` calls it on every set (711620–711655).
  - The only other writer is a copy-constructor, `FUN_0057a668` 710844–710871, which copies another camera's block.
- **BF1 is identical:** `FUN_004999e8` (the negations), `FUN_004901d8` (the plain inverse at cam+0x40, words
  0x4901e4–0x490234), the setter `FUN_00490f80`, and copy-constructor `FUN_0048ffd8`.
- **Effect [D].** The pass dots L_C with R_V, where V = diag(1,−1,−1)·C:
  - **d_disc = (X·L)(X·R) − (U·L)(U·R) − (B·L)(B·R) = 2(X·L)(X·R) − L·R**, in world terms, with X = the camera's
    screen-right axis.
  - With the table's |d| symmetry that is the lobe of the light **with its camera-right component negated**:
    L − 2(X·L)X.
  - A sun straight ahead or behind is unaffected. A sun off to one side lights highlights as if it stood at the
    mirrored azimuth, and the effective light swings as the camera turns.
  - The consistent version would be d = L·R. Whether the difference is visible on the PS2 needs an in-map PCSX2
    capture (HOLD 7), which the rig cannot reach yet.

## 5. The table — `specular` in CORE.LVL [V][O]

- **Format:** `tex_` at BF2 CORE.LVL 0x2ad598 and BF1 CORE.LVL 0x103a10, both 0xb1c bytes. INFO: 128×128,
  format 8, mipCount 1. 256 grey palette entries with A 0xff; entries 252–255 are 0 but unused.
- **The two discs' decoded texels are byte-identical** (`dectex.py`; the verifier's own decoder agrees). Rows run
  top to bottom, so t = 0 is row 0.
- **Shape:** column 0 is linear (255 → 128 at rows 63–64 → 255 at row 127), i.e. 0.5 + 0.5·|d|. For x ≥ 1 each
  column is bright at both ends and 0 in the middle, narrowing as x grows.
  - Row 0 falls 255 → 143 and row 127 255 → 244 across x.
  - It is roughly |d|^(2x): fitted exponent ≈ 2.4 at x = 1 and ≈ 31 at x = 16. The generator is unknown and
    irrelevant: **sample the disc's table**.
- **Measured through GS-style bilinear + REPEAT** (`lead_tmp/re/specular/`, the sampling snippet in the report):

  | power | half-width of the + lobe | T at d = 0 | note |
  |---|---|---|---|
  | 4 | 36° | 0 | |
  | 15 | 20° | 0 | |
  | 50 | 14° | 0 | |
  | 100 | 12° | 0 | |
  | 1 | — | 128 | **column 0: a +50 % glow over the whole surface** |
  | 255 | — | 64 | REPEAT wraps s = 1.0 to column 0 |

  - T(d = ±1) ≈ 210–255 at every power: the rows next to the t = 0/1 seam are bright, so the lobe never gets
    narrower than ~11°.
  - The lobe at d → −1 is as bright as at +1: **two highlights, one of them "anti-sun".**

## 6. What Ad holds — and BF2's invisible translucent variant [V] target, [D] reading [O]

- **The target.** The main render target's FRAME word is 0x80080: FBP 0x80, FBW 8, **PSMCT32**, **FBMSK 0**.
  It is stored to 0x7c5120 by `FUN_0058d510` 722641 (words `lui s2,0x8; ori s2,s2,0x80` at 0x58daf0/0x58daf4,
  `sd` at 0x58db28), the only store.
- **Ad.** An opaque 0x80 base pass (TEST 0x50000, no ABE; §2) writes As = At·Av>>7 into the frame's alpha, and
  the pass right after it in the same batch reads that back as Ad.
  - Av = 0x80 for an opaque draw colour, so **Ad ≈ the base texel's alpha**: disc alpha a → (a·0x81)>>8 in GS
    units (`re-hud-icons.md` citation 3).
  - A `Specmap` sheet's alpha is therefore its gloss mask. BF1 YAV1's `yav2_bldg_details` is 96 % alpha 0, so the
    bazaar strips glint only on their opaque bits.
  - FBA is assumed 0 (the GS reset value) [H]: no writer of FBA_1 was searched for.
- **BF2's translucent spec variant.** A word with 0x04|0x80 uses record 0x7c57c0 = Cs·As + Cd, and As = At·Av>>7
  with At = 0x80 and Av = 1 (§2).
  - **As = 1/128, so that pass adds at most 2/255: invisible on the disc.** The non-0x04 record only alpha-tests
    As ≥ 1, which Av = 1 passes, and blends by Ad.
  - A port can skip BF2's 0x04 spec pass outright.

## 7. When the pass is drawn [V][O]

- **Order.** It is appended to the base segment's multi-pass batch, in the same VU1 kick, right after the base
  and any bump/light pass. ZTST is GEQUAL (nearer or equal) on the base's own depth.
  - [D] So a port pass drawn after all opaque draws, with depth test ≤ and the base texture's alpha re-sampled as
    the mask, is pixel-equivalent. A later nearer draw overwrote the PS2's highlight exactly where it now fails
    the port's depth test.
- **BF2 far scene.** A far-scene non-sky draw (`(flags & 0x20020) == 0x20000`) with `DAT_0060c2c7` ≠ 0 jumps to
  the emit (*(decompiler detail omitted)*, 723621–723687) before the 0x80 test.
  - It gets the fogspread pass there instead, when fog is on.
  - `DAT_0060c2c7` is 1 in .data and is set to 1 for mission loads (`FUN_00174b28`).
  - Three sky/effect draws clear it around themselves (`FUN_003bf060`, `FUN_003c9258`, `FUN_005a3d40`).
  - **So in a mission BF2's far-scene models never get the pass.** BF2 fog is a far-scene-only pass
    (`re-fog.md` §0), so the two never meet on BF2.
- **BF1 far scene.** `FUN_004b2310` has no far-scene variant: no 0x20000 test in 573294–573460 (grep). BF1 draws
  the pass in both scenes. Whether BF1 fogs a Specular segment at all is not read [H] (the fog gate cited in
  `re-fog.md` is the Normal class's).

## 8. BF1 against BF2 — the differences

| | BF2 (word bit 0x80) | BF1 (class bit 0x20) |
|---|---|---|
| matrix packet | 5 qwords (w column); Q from m15 = 1; m14 = −1 | 4 qwords; Q from row z = m14 = 1; m15 = 0 |
| colour | MTRL spec RGB × f, halved (0xff → 1.0) | B raw, or the SunInfo colour raw (0x80 → 1.0) |
| pass alpha / test | A = 1; AREF 1 passes; the 0x04 variant uses As → invisible | no alpha test; always Cs·Ad + Cd |
| base pass | the normal rule for the word | always opaque, no test |
| far scene | skipped in missions | drawn |
| light | `lght` global light 1 (+0x50) | `SunInfo` direction `DAT_0069c590` |

The table, s, t, R and the frame mismatch are identical in both.

## 9. What the port owes (the build entries SPEC and SPEC2) — PROVISIONAL, [O]

**Corrected 2026-09-29 (card DSHN, `analysis/re-spec-gloss.md`, [O]):** the list below misses one bake step. The pass
is weighted by Ad (§6), the base texel's alpha, so a texture that a pass segment draws must KEEP its alpha. The port's
bakes forced 68 such sheets to alpha 255 (every droid, every 1st-person weapon, 29 world sheets). The pass then ran at
weight 1.0, and the droids went chrome. PCSX2 patched states confirm Ad on the PS2 itself in both games.

1. **Bake.**
   - Parse the rest of the MTRL in `mungefs::model`: BF2 bytes 8..15 = spec RGBA + power; BF1 12-byte MTRL bytes
     4..11 = A, B.
   - Write a per-segment `extras.spec_pass` for: BF2 0x80 without 0x04 (the 0x04 variant is invisible, §6); BF1
     class 0x20 on every word. It carries `{power, rgb}`:
     - BF2 rgb = ((b+1)>>1)/128 per channel;
     - BF1 rgb = B/128 raw when B ≠ 0 and B & 0xffffff ≠ 0xffffff, else the map's SunInfo colour/128 (so ≤ 2.0).
   - Bake the `specular` table once from CORE.LVL (either disc).
2. **Draw.** A second draw of each such segment, additive (One, One), depth-tested ≤ with no depth write, after
   the opaque pass.
   - Vertex: Î = normalize(p − eye), R = 2(N·Î)N − Î (world space), d = 2(X_cam·L)(X_cam·R) − L·R,
     uv = (power/255, 0.5 + 0.5·d), with a REPEAT sampler.
   - Fragment: min(1, T·rgb) × the base texel's alpha at the base UV.
   - L = the sun's travel direction.
   - BF2: near scene only. BF1: everywhere.
   - A flag can switch d to the consistent L·R (§4).
3. **Soldiers and weapons** (BF2 12 soldier segments + weapons, BF1 4 soldier segments + weapons) are the same
   pass on the skinned path: a follow-up entry (`bake::soldier`, the skinned renderer).
4. **Goldens.** Any golden showing a spec model moves. Candidates: BF1 KAM1/KAS2/YAV1/YAV2/RHN1/NAB2/BES2 and BF2
   DEA1/TAN1/UTA1/KAM1/FEL1/POL1/TAT2/TAT3/YAV1/COR1/GEO1/NAB2.

## 10. How this was verified (`OPUS_LOOP_PROMPT.md` §5)

- **Self-check 1 (the enclosing function):** every cited decomp line had its `/* ==== FUN_ ==== */` header
  printed by `enclosing.py`. Two wrong guesses were corrected before the claims went out:
  - 716900 is in `FUN_005828d8`, not `FUN_00587278`;
  - 720888 is in `FUN_00589fa8`.
- **Self-check 2 (absence claims searched):**
  - no store to s, c or the 0x8000 flag bytes, in either game (`immscan.py`; the verifier's `cscan.py` also covers
    lwl/lwr/sdl/sdr and pointer words);
  - no BF2 0x80 word with 0x100 (`census_spec.py`, 0/469);
  - only opcodes 0x3b7/0x3c4/0x3d1 write VU mem 48..59;
  - the far-scene goto skips the 0x80 block (brace nesting plus the verifier's byte reading);
  - no 0x20000 test in BF1's `FUN_004b2310`.
  - The one absence the verifier falsified, other writers of the camera's inverse block, found only
    copy-constructors that preserve it.
- **Self-check 3 (twins opened both ways):** BF1's builder, draw, packet, VU1 R and loop, view builder, camera
  inverse, setter and record were each read in their own function. None was assumed from BF2.
- **Self-check 4 (constants from bytes):**
  - s, c, 1/255 and the records;
  - the FRAME word;
  - `DAT_0060c2c7` (.data word 0x60c2c4 = 0x01000000);
  - the VU LOI constants (1/128 = 0x3c000000; 65536 = 0x47800000; 2^33 = 0x50000000; 2^22 = 0x4a800000;
    2^31 = 0x4f000000);
  - the table (disc bytes).
- **The verifier** (one Opus agent, foreground, claims and citations only): **23 SUPPORTED, 7 PARTLY, 0 NOT
  SUPPORTED** (`verify/VERDICTS.md`). Folded in above:
  - C4: the pass alpha is 1, which is what makes BF2's 0x04 variant invisible;
  - C5: `DAT_0060c2c7` is a runtime flag;
  - C18: five callers, not two;
  - C19/C27: the copy-constructors;
  - C23: the other writers of the light colour;
  - C28: palette 252–255, column 0's end value;
  - C30: B's value count.
- **Still [H]:**
  - FBA = 0;
  - COLCLAMP on (the sum saturating at 255);
  - BF1 fog on Specular segments;
  - how the frame mismatch LOOKS in play (needs an in-map PCSX2 capture);
  - which named `lght` entry is "global light 1" on each BF2 map (the port's `bake::lighting` key light is taken
    as it).
