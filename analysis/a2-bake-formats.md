# A2 — Bake-format recon: PS2 `ucfb` LVL containers (BF1 2004 / BF2 2005)

Audit date: 2026-08-30. Tool: `scripts/ucfb_tree.py` (chunk-tree walker with
recursion heuristic, FNV-hash resolution, census mode). All offsets below are
absolute file offsets of the chunk *header* (tag), payload starts at +8.

Local evidence roots (read-only):
- `extracted/bf1/lvl/DATA/_LVL_PS2/` (abbrev. **B1/**)
- `extracted/bf2/lvl/DATA/_LVL_PS2/` (abbrev. **B2/**)

Web evidence: [Ben1138/LibSWBF2](https://github.com/Ben1138/LibSWBF2) (**LibSWBF2:**)
and [SleepKiller/swbf-unmunge](https://github.com/SleepKiller/swbf-unmunge)
(**unmunge:**), paths relative to each repo root. Local caches of the fetched
sources are in the session scratchpad (not committed).

---

## 1. Container format (confirmed)

`[4-byte tag][u32 LE payload size][payload][pad to 4-byte alignment]`, root
chunk `ucfb`, chunks nest freely. Identical to PC
(LibSWBF2: `src/Chunks/BaseChunk.cpp` `GetAlignedSize()`; unmunge:
`src/ucfb_reader.cpp` `read_child()` + `align_head()`).

**Tags come in two flavors:**
1. 4 ASCII chars (`tex_`, `NAME`, `tern`, ...).
2. A 32-bit **FNV-1a hash** of a name (basis 2166136261, prime 16777619,
   chars folded to lowercase before XOR). Verified locally: the chunk tagged
   `0x9A644D15` in B1/CORE.LVL @0x10 contains children tagged
   `fnv("INFO")=0x0FB40705`... resolved as `INFO`/`DATA` hashes, and the
   `lvl_` child names in B1/MISSION.LVL resolve against the mission script
   names (`fnv("bes1a") = 0x...` matches the chunk at @0x10 whose inner
   `scr_ NAME` is `"bes1a"`). Property-name hashes crack the same way:
   `0x8D39BDE6=name, 0x07A640F6=layer, 0xA2FD7D0C=team, 0xF69717FD=label,
   0xC6C161F9=captureregion, 0x805C1F8A=controlregion, 0xDDE0B14F=spawnpath,
   0x0DBA4CB3=radius`.
   Same scheme as PC (LibSWBF2: `src/Hashing.cpp` `FNV::Hash`; unmunge:
   `src/swbf_fnv_hashes.hpp` `fnv_1a_hash` with `c |= 0x20`; unmunge ships a
   ~103 KB reverse dictionary in `src/swbf_fnv_hashes.cpp` worth vendoring
   for the bake tool).

**`lvl_` wrapper**: payload = one chunk whose *tag* is the FNV hash of the
sub-level name, containing the actual content (LibSWBF2:
`src/Chunks/LVL/lvl_.cpp` reads `u32 nameHash; u32 sizeLeft`, same bytes,
different framing — the hash+size pair *is* a chunk header). Used by:
BF1/BF2 MISSION.LVL (per-mission scripts), SIDE/*.LVL (per-unit sections),
BF2 LOAD screens (per-image), BF2 map LVLs (per-game-mode world layers,
e.g. B2/TAT/TAT2.LVL @0x335D14 `lvl_` → `0xE2AB7AC8` → `lght` + `wrld` with
extra `regn`/`inst` for one game mode).

---

## 2. Chunk-tag census

Aggregated over 6 representative files per game
(CORE.LVL, COMMON[.LVL|/ENG/COMMON.LVL], MISSION.LVL, TAT map, SIDE/ALL.LVL,
LOAD screen). Bytes are summed payload sizes (children double-count inside
parents). `<hashed>` = all non-ASCII (FNV) tags combined. Full per-file
censuses were produced with `ucfb_tree.py --census`.

### BF1 (top tags)

| tag | count | bytes | seen in |
|---|---|---|---|
| tex_ | 528 | 7,455,528 | common,core,load,side,tat1 |
| lvl_ | 63 | 6,451,432 | mission,side |
| `<hashed>` | 37 | 6,299,628 | core,mission,side |
| SCOP | 6,386 | 4,291,744 | common,core,tat1 |
| modl / segm | 263 / 483 | 2.9M / 2.7M | common,load,side,tat1 |
| fx__ | 117 | 931,856 | common,tat1 |
| BIN_ (anim) | 33 | 920,388 | common,side,tat1 |
| Locl | 6 | 904,036 | core |
| zaa_ / zaf_ | 15 / 18 | 837,696 / 83,936 | common,side,tat1 |
| tern | 1 | 499,748 | tat1 |
| scr_ | 83 | 484,652 | common,mission |
| coll / TREE | 70 | 407,140 | common,load,side,tat1 |
| plan | 1 | 272,456 | tat1 |
| pal_ | 523 | 189,552 | everywhere tex_ is |
| wrld | 1 | 139,332 | tat1 |
| entc/wpnc/expc/ordc | 98/92/…/… | ~250K | common,side,tat1 |
| shdw / SHDI | 40 / 40 | 120K / 84K | side |
| font / FTEX / FBOD | 5 | 130,376 | core |
| snd_ | 1 | 16,644 | core |

Segment sub-tags (BF1+BF2): `INFO MTRL RTYP TNAM _PS2 STRP POSI NORM TEX0
[COL0] [BNAM] [WGHT BIND BONE]`. Terrain sub-tags: `NAME INFO LTEX DTEX DTLX
SCAL AXIS ROTN HEXP HGT8 CUTS FOLG LUMI PCHS/PTCH [LPTC LOWR COLR ALPH DCAL
BMAP]`.

### BF2 (top tags)

| tag | count | bytes | seen in |
|---|---|---|---|
| `<hashed>` | 162 | 12,573,320 | core,load,mission,side,tat2 |
| lvl_ | 164 | 12,557,984 | load,mission,side,tat2 |
| tex_ | 308 | 7,909,188 | common,core,load,side,tat2 |
| modl / segm | 327 / 604 | 4.3M / 4.2M | side,tat2 |
| Locl | 6 | 2,686,336 | core |
| scr_ | 174 | 1,480,292 | common,mission |
| entc/ordc/wpnc/expc | 202/301/363/162 | ~850K | side,tat2 |
| zaa_ / zaf_ / BIN_ | 13 / 27 / 40 | 1.7M | side |
| wrld | 13 | 269,892 | tat2 (1 base + 12 layer wrlds) |
| mcfg | 19 | 20,220 | mission |
| tern | 1 | 12,036 | tat2 (64×64 grid — small) |
| CLTH (cloth) | 11 | 53,256 | side |
| plan | 1 | 140,524 | tat2 |
| pal_ | 308 | 219,660 | everywhere tex_ is |

Notable BF1↔BF2 shifts: BF2 wraps nearly everything in hashed `lvl_`
sections; BF2 map files embed per-game-mode `wrld` layers; BF2 adds `mcfg`
(hash-keyed mission config), `CLTH`; BF2 COMMON.LVL is small (scripts +
textures only) while BF1's COMMON/ENG/COMMON.LVL carries big UI/fx configs
(`SCOP`/`DATA` trees) and models.

---

## 3. Tree excerpts (map LVLs)

B1/TAT/TAT1.LVL (3,377,880 bytes) top-level shape:

```
ucfb
  fx__  (NAME=hash)                     @0x8
  tex_  tat_skiff01_lowrez  NAME/INFO/pal_(INFO,BODY)/BODY×mips   @0x1C
  ...   (46 tex_ total)
  skel  all_hover_skiff  INFO/NAME/PRNT/XFRM        @0x98B0
  modl  all_hover_skiff  NAME/NODE/INFO/segm×8      @0x9D0C
  gmod  all_hover_skiff  NAME/INFO/LOD0/LOWD        @0x13918
  coll  all_hover_skiff  NAME/NODE/INFO/POSI/TREE   @0x13984
  zaa_  tat_hover_skiff  NAME/BIN_(SMNA/MINA/TNJA/TADA) @0x1CA14
  entc  hover → tat_hover_skiff  BASE/TYPE/PROP×n   @0x1CFC4
  ...   (repeats per model/entity)
  tern  tat1                                        @0xC1E1C
  sky_  (hashed config)                             @0x16F55C
  wrld  tat1  NAME/TNAM/SNAM/INFO/regn×15/inst×249/BARR/Hint  @0x313274
  plan  ...                                         (path planning)
  path  ...×13
```

B2/TAT/TAT2.LVL is the same plus: 6 top-level `lvl_` game-mode layers at the
end (each `lght`+`wrld`, e.g. @0x335D14), `lght` config chunk @0x37174, and
BF2-flavored segm/MTRL (below).

`tern` subtree (B1 @0xC1E1C):

```
tern  size=499748
  NAME "tat1"
  INFO 28 B   = f32 gridUnitSize(8.0), f32 heightScale(0.06), f32 heightFloor(-0.6),
                f32 heightCeiling(63.54), u16 gridSize(256), u16 heightPatches(8),
                u16 texturePatches(16), u16 textureCount(6), u16 maxTextureLayers(16), u16 0
  LTEX 76 B   = 6 nul-terminated layer texture names ("tat_main_2", ...)
  DTEX 28 B   / DTLX ("tat1_detail_1") / SCAL f32[16] / AXIS u8[16] / ROTN f32[16]
  HEXP 2048 B — **ERRATUM 2026-08-30 (re-terrain-heights.md §3): NOT u16 per
  8×8 patch.** It is one 8-byte record per 16×16-cell patch (256 records on a
  256 grid): {u8 shift, u8 cutterQuadMask, i16 offset, i16 min, i16 max}.
  HGT8 63953 B → RLE-decodes to exactly 65536 B = 256×256 u8 heights
  CUTS → CUTR (INFO 20B + PLNS f32 planes) — terrain cutters
  FOLG 131076 B = 4 B header + 256×256 u16 foliage map
  LUMI 65536 B = 256×256 u8 lightmap (uncompressed)
  PCHS → 1024 × PTCH { INFO 14 B }  (+ 64 LPTC, LOWR, COLR, ALPH, DCAL, BMAP)
```

BF2 tern identical layout, gridSize 64 (HGT8 134→4096 B, LUMI 4096 B,
64 PTCH, no FOLG on TAT2). The tern INFO layout **exactly matches PC**
(unmunge: `struct Terrain_info`, `static_assert(sizeof == 28)` in
`src/handle_terrain.cpp`).

---

## 4. Bake-v0 relevant tags: structure + confidence

Confidence key: **[V]** verified programmatically on local data,
**[X]** cross-confirmed by unmunge source, **[H]** hypothesis.

### Textures `tex_` → `NAME, INFO, pal_(INFO,BODY), BODY×mips` — [V][X]
- `INFO` 14 B: `u16 w, u16 h, u16 format (4|8 = bits per palette index),
  u16 detailCompressed, f32 mipDistance (-1 or 7/15/20/150), u16 mipCount`.
  Verified across 90 textures in both TAT maps; mipCount always equals the
  number of `BODY` chunks. Matches unmunge `src/handle_texture_ps2.cpp`
  exactly (its `Ps2_format` also allows 32 = true-color; not seen in the
  files sampled here — every sampled texture is paletted).
- `pal_ INFO` 4 B: `u16 colorCount (16|256), u16 bitsPerColor (32)`.
- **RLE codec [V]**: control byte `c`: if `c < 0x80` copy `(c&0x7F)+1`
  literal *elements*; else repeat the next element `(c&0x7F)+1` times.
  Element = 1 byte for `BODY` texel data and `HGT8`; element = 4 bytes
  (one RGBA color) for `pal_ BODY`. Verified: 13/13 tex BODYs in TAT1
  decode to exactly `w*h*bpp/8`; 107/107 palettes across TAT1/TAT2/CORE
  decode to `colorCount*4`; HGT8 decodes to `gridSize²` in both games.
  Identical to unmunge `decompress_body` (4-bit indices packed two per
  byte, **high nibble first** per `promote_4bit_refs`; colors are
  byte-order R,G,B,A per `resolve_texels`).
- `detailCompressed != 0` means colors must be modulated ×2 against the
  sibling `*_dtl` texture (unmunge `resolve_detail_compression`) — the bake
  tool should bake this in exactly as unmunge does.
- Bake target: decode indices → palette lookup → RGBA8 → KTX2. Mip chain is
  present per-BODY (each mip RLE'd separately, halving w/h).

### Models `modl` → `NAME, NODE, INFO, segm×n` — [V][X]
- `modl INFO` 68 B (both games, = PC-SWBF1 size): 3×u32, then
  `f32[3] vertexBoxMin, f32[3] vertexBoxMax, f32[3] visBoxMin,
  f32[3] visBoxMax, u32 faceCount` (evidence B1/TAT1 @0x9D40; PC BF2 has 72 B
  with 4 leading u32 — PS2 BF2 keeps 68 B).
  **ERRATUM 2026-08-30:** PS2 BF2 modl INFO is 72 B (4 leading u32, boxes at
  +16); see RE_LOG §Erratum. TAT2 renders were shifted until fixed.
  **ERRATUM 2026-09-08 (lead-verified, session 15; queue item 8, `mungefs/examples/segdump.rs`):**
  the layout above adds up to 64 B of a 68-B chunk, so it is one word short, and
  the trailing word is NOT a face count. Measured on the discs: the u32 at +60
  (the one `mungefs::model::Model::face_count` reads) is 1..14 on models of up
  to 3,758 triangles — unidentified, and `face_count` has no pipeline consumer;
  the LAST u32 of INFO (+64 in BF1's 68 B, +68 in BF2's 72 B) EQUALS the sum of
  the segments' `segm INFO indexCount` on every model — 445/445 over seven LVLs
  (agent), re-run by the lead on BF1 BES1 (37), BF1 HOT1 (55), BF2 TAT2 (90),
  BF2 HOT1 (101): 0 mismatches. So the last word is the model's TOTAL INDEX
  COUNT, and `idx != @tail` in `segdump` means the parse lost indices.
- `segm` children (per-vertex counts verified over 303 segms, both games):
  - `INFO` 8 B: `u32 vertexCount, u32 indexCount` (no topology field —
    always PS2 triangle strips; unmunge `read_segment_info_ps2`).
  - `MTRL`: **BF1 = 4 B** (u32 flags only); **BF2 = 25 B**
    (u32 flags, u32 diffuse, u32 specular, u32 specExponent, u32 params[2],
    nul string) — the PC-BF2 24-B layout + attached-light string
    (unmunge `Material_info`).
  - `RTYP`: **BF1 = string** ("Normal"); **BF2 = u32 enum** (unmunge notes
    the u32 form for PS2).
  - `TNAM`: `u32 slot + nul name` (up to 4 in BF2, 1 in BF1 samples).
  - `_PS2`: u32 (e.g. 0x168) — PS2-only, not parsed by any community tool.
    Likely VU/DMA buffer size. [H]
  - `STRP`: `indexCount × u16` strip indices; a strip restart is marked by
    the **high bit (0x8000) set on the first two indices** of the new strip
    (evidence B1 @0x9DE4: `1a 81 0c 81 08 01 ...`).
  - `POSI`: `vertexCount × 3×u16`, dequantized into `modl INFO` vertexBox
    (`min + v * (max-min)/65535`; unmunge `read_positions_buffer`).
  - `NORM`: `vertexCount × 3×i8`, ÷127.
  - `TEX0`: `vertexCount × 2×i16`, ÷2048.
  - `COL0` (optional): `vertexCount × u32` RGBA (unmunge unpacks `.bgra`).
  - `BNAM` hard-skin bone name; `BBOX` 6×f32 (BF2 only); softskin adds
    `WGHT`/`BIND`/`BONE` (side LVLs).
- `gmod` (B1 @0x13918): `NAME, INFO (2×f32), LOD0/LOD1/LOD2/LOWD` entries of
  `[model name][u32]` — LOD table binding entity models to `modl` chunks. [H]
- `skel`: `INFO (20 B), NAME (packed strings), PRNT, XFRM (48 B per bone =
  3×3 f32 rot + f32[3] pos)` — same as PC (LibSWBF2 `src/Chunks/LVL/skel/`).
- `coll`: `NAME, NODE, INFO (4×u32 counts + 2×f32[3] bbox), POSI (f32×3
  verts — full floats, unlike render mesh), TREE(NODE/LEAF)` — same as PC
  (LibSWBF2 `src/Chunks/LVL/coll/`). Verified 292 verts × 12 B @B1 0x139E8.

### Terrain `tern` — [V] header, [H] height reconstruction
Layout in §3. Unlike PC there are **no per-patch VBUF/IBUF** — PS2 stores
`HGT8` (RLE u8 heightfield) + `HEXP` (8-B record per 16×16 patch — see the
ERRATUM in §3; the original "u16 per 8×8 patch" was wrong) + `LUMI` (u8
lightmap) + `PTCH INFO` (14 B: `u32 textureMask?, u16, u16, u16, u32` —
layer assignment per patch [H]) and generates geometry on the VU. Bake
hypothesis: `worldHeight = f(HGT8[x,z], HEXP[patch], heightScale,
heightFloor/Ceiling)` — the exact per-patch scale/offset combination needs
confirmation from the ELF decomp (see open questions). Texture layering:
`LTEX` names + per-patch `PTCH INFO` + `LPTC`/`ALPH`/`COLR` blend data [H].
**ERRATUM 2026-08-31:** the `PCHS`-level `COLR` is not blend data — it is
LUMI in colour (RE_LOG §"Terrain `COLR` cracked"); only the per-patch
`PTCH -> COLR/ALPH` overrides and `ALPH` remain open.

### World `wrld` — [V][X]
`NAME, TNAM (terrain), SNAM (sky), INFO (u32 regnCount, u32 instCount)`,
then `regn` and `inst` chunks:
- `regn`: `INFO{TYPE ("box"/"sphere"/"cylinder"), NAME, XFRM 48 B, SIZE
  3×f32} + PROP*`.
- `inst`: `INFO{TYPE (entity class), NAME, XFRM 48 B} + PROP*`.
- `XFRM` = 3×3 f32 rotation + f32[3] translation (PC-identical; unmunge
  `struct Transform` sizeof==48).
- `PROP` = `u32 fnv1a(propertyName) + nul-terminated value string`
  (PC-identical). BARR barriers (`INFO/…/FLAG`) and `Hint` nodes also
  present, PC-shaped.
BF2: base `wrld` + one `wrld` per game mode inside trailing `lvl_` layers.

### Localization `Locl` — [V][X]
`NAME (language) + BODY`; BODY = packed entries
`[u32 fnv1a(keyPath)][u16 entrySize incl. 6-B header][UTF-16LE chars,
nul-padded]`. Verified: 2057 entries parse gap-free over the whole 144,300-B
english BODY in B1/CORE.LVL @0x26E94. PC-identical
(LibSWBF2 `src/Types/LocalizeEntry.cpp`).

### Scripts `scr_` — [V]
`NAME, INFO (u8 count=1), BODY` = precompiled Lua bytecode.
**BF1 = Lua 4.0** (`\x1bLua\x40`, B1/MISSION.LVL @0x3C) —
**BF2 = Lua 5.0** (`\x1bLua\x50`, B2/MISSION.LVL @0x1A0C). Both little-endian,
size_t 4. Bake: keep as-is (RON metadata + blob) or decompile offline; the
runtime will need matching Lua VM behavior anyway.

### Config chunks (`sky_ lght fx__ snd_ mus_ bnd_ prp_ path hud_ mcfg ...`) — [V][X]
All share one compiled-config format:
`NAME (u32 hash of config name)`, then a tree of `DATA`/`SCOP`:
`DATA = [u32 fnv1a(functionName)][u8 argCount][args: 4 B each — f32, u32
hash, or [u32 strLen][chars] for strings][4-B zero terminator]`; a following
`SCOP` scopes child statements. Evidence: B1/TAT1 `sky_` @0x16F55C,
B2/TAT2 `lght` @0x37174, B1/CORE `snd_` @0x3000, B2/MISSION `mcfg` @0x8.
PC-identical (LibSWBF2 `src/Chunks/LVL/config/DATA.cpp` documents the same
layout, listing chunks `lght fx__ sky_ bnd_ prp_ path comb snd_ mus_ ffx_
tsr_ hud_`; unmunge `src/handle_config.cpp`). Bake: mechanical → RON.

### Animation `zaa_`/`zaf_` → `NAME + BIN_` — [X]
`BIN_` holds `SMNA` header (u32 version=8, numBones, dataBufferLength,
u16 numAnims), then `MINA`/`TNJA`/`TADA` quantized keyframe streams keyed by
Zero-engine CRC-32 of bone/anim names (evidence B1/TAT1 @0x1CA34
`SMNA...MINA`; parser: LibSWBF2 `src/Chunks/LVL/zaa_/BIN_.cpp`, `MINA.cpp`,
`TNJA.cpp`, `TADA.cpp`; the CRC lives in `src/Hashing.cpp`
`CRC::CalcLowerCRC`). Appears byte-identical to PC. Not needed for bake v0
(static geometry) but needed for v1 characters.

### Fonts `font` → `NAME, HEAD, FTEX(NAME + tex_), FBOD` — [H]
PS2-specific glyph data in FBOD; tex_ inside is the standard PS2 paletted
texture. CORE.LVL only. Defer past bake v0.

### Entity classes `entc/ordc/wpnc/expc` — [V][X]
`BASE (parent class string), TYPE (odf name), PROP*
([u32 keyHash][value string])` — PC-identical (LibSWBF2
`src/Chunks/LVL/common/GenericClass.cpp`). Bake: → RON dictionaries.

---

## 5. PS2 vs PC differences (vs LibSWBF2 / swbf-unmunge)

| area | PC (LibSWBF2 / unmunge) | PS2 (observed) |
|---|---|---|
| endianness / container | LE, 4-byte aligned chunks | identical |
| hashing | FNV-1a lowercase for lvl_/PROP/config/Locl; CRC-32 for anims | identical (verified locally) |
| textures | `tex_ → INFO(D3DFORMAT list) → FMT_ → FACE → LVL_ → BODY`, raw D3D pixels (`src/Chunks/LVL/tex_/*`, `src/handle_texture.cpp`) | `tex_ → INFO(14 B w/h/bpp) → pal_ → BODY×mips`; paletted 4/8-bit, plus 32-bpp direct-colour effect textures in COMMON.LVL; RLE-compressed; detail-modulation baked via sibling `_dtl` texture. Handled only by unmunge `src/handle_texture_ps2.cpp` — LibSWBF2 has **no** PS2 support at all |
| model vertex data | interleaved `VBUF` w/ flag bits (0x2 pos … 0x8000 uv-compressed) + `IBUF` (`src/vbuf_reader.cpp`) | planar `STRP/POSI/NORM/TEX0/COL0/BONE` quantized buffers + `_PS2` chunk; no topology in segm INFO (`read_segment_info_ps2`, `src/handle_model.cpp`) |
| segm MTRL | BF2-PC 24 B + string; BF1-PC variable | BF1-PS2 just 4 B flags; BF2-PS2 = 25 B (PC-BF2 form) |
| segm RTYP | string both games | BF1-PS2 string, BF2-PS2 u32 enum |
| modl INFO | 68 B BF1 / 72 B BF2 | **ERRATUM 2026-08-30: same as PC — 68 B BF1, 72 B BF2.** The original "68 B in both" shifted every BF2 model into a box (RE_LOG §ERRATUM modl INFO). |
| terrain | per-patch `VBUF`(pos/normal/color 28 B)+`IBUF`+texture vbufs (`src/handle_terrain.cpp`, `src/Chunks/LVL/tern/PTCH.VBUF.cpp`) | same INFO/LTEX/SCAL header, but heightfield as RLE `HGT8` + `HEXP` + `LUMI` + tiny 14-B `PTCH INFO`s; extra `FOLG LPTC LOWR COLR ALPH DCAL HEXP` chunks; `WATR` is rare on PS2: absent from the TAT maps, present as a stub-`tern` child on BF2 DEA1 **ERRATUM 2026-08-31:** wrong — 14 `WATR` chunks ship across 12 of the 34 ground maps and on BF2 DEA1/KAM1 the water is the map's only ground; decoded in RE_LOG §"`tern -> WATR` cracked" and `mungefs::water`. |
| shadows | `SHDW` vertex data inside modl (PC) | separate top-level `shdw`/`SHDI` chunks in SIDE LVLs |
| Locl / wrld / coll / prim / config / entity classes / zaa_ | — | byte-identical layouts to PC |
| scripts | Lua 5.0 (BF2 PC) | BF1: Lua 4.0; BF2: Lua 5.0 |
| CORE.LVL top chunks | named chunks | two hash-tagged config chunks (`0x5E262256` → `0x9A644D15` → INFO/DATA) + hashed sound chunks (unmunge/LibSWBF2 `KNOWN_SOUND_HEADERS`: hashes of "StreamList", "SampleBank", "Info", "Data", ...) |

Community-tool bottom line: **unmunge is the reference for PS2 textures and
models** (`-platform ps2`); **LibSWBF2 is the reference for everything
platform-neutral** (wrld, config, Locl, zaa_, coll, plan, skel). Neither
tool parses PS2 terrain heights (`HGT8/HEXP`), `_PS2`, `shdw/SHDI`, `LPTC`,
`FOLG`, or the `gmod` LOD table — those need the ELF decomp as oracle.

---

## 6. Recommended P1 parsing order (one map: terrain + models + textures)

Target: `B1/TAT/TAT1.LVL` first (larger terrain, string RTYP, simplest
MTRL), then generalize to `B2/TAT/TAT2.LVL` (adds layer lvl_, BF2 MTRL/RTYP).

1. **Chunk walker + FNV** (done: `scripts/ucfb_tree.py`; port to the bake
   tool as a zero-copy reader). Vendor unmunge's `swbf_fnv_hashes.cpp`
   dictionary.
2. **RLE decoder** (element size 1 and 4) — small, already verified; unit
   test = decoded size assertions from §4.
3. **tex_ → KTX2**: decode → de-palettize → RGBA8, apply `_dtl` modulation
   when `detailCompressed`, emit mip chain. (Resolved: texels are linear
   row-major after RLE, and 256-colour CLUTs are linear — no unswizzle.)
4. **modl/segm → glTF**: dequantize POSI/NORM/TEX0/COL0 against modl INFO
   vertexBox, unstitch STRP on 0x8000 markers → indexed triangles, material
   from MTRL/TNAM, hard-skin via BNAM + skel.
5. **coll → glTF collision mesh or RON** (float verts, trivial).
6. **tern → glTF terrain mesh**: decode HGT8, resolve HEXP scaling against
   the ELF decomp, apply gridUnitSize/heightScale; LUMI as vertex color or
   lightmap texture; layer splat from LTEX + PTCH INFO (can ship v0 with
   only base layer + LUMI).
7. **wrld → RON scene**: instances (TYPE/NAME/XFRM/PROPs), regions,
   barriers; hash-resolve PROP keys.
8. **entc/… + config chunks → RON**; **Locl → RON string tables**;
   `scr_` blobs passed through.
9. Defer: zaa_ animation, fonts, plan/path AI data, shdw, sound/mus.

---

## 7. Open questions

1. **ANSWERED (P1, re-terrain-heights.md §3).** HGT8 ↔ HEXP: HEXP is one
   8-B record per 16×16 patch carrying a per-patch shift and offset; the
   reconstruction is implemented in `mungefs::terrain` and verified
   empirically (floor −0.6 exact).
2. **ANSWERED (P1, RE_LOG §LVL textures).** Texels are LINEAR row-major after
   RLE — no GS swizzle. Separately, 256-colour CLUTs are LINEAR too, not
   CSM1-swizzled (RE_LOG §ERRATUM — 256-color CLUTs are LINEAR).
3. **`_PS2` u32 in segm** — meaning (VU chain/DMA size?); harmless to ignore
   for bake, useful for validation.
4. **PARTLY ANSWERED.** `PTCH INFO` field 0 is a u16 LAYER BITMASK + u16
   flags, verified on all 50 shipped terrains, and the base layer (LTEX[0],
   tiled by SCAL[0]) renders (RE_LOG §Terrain layer texturing). STILL OPEN:
   how the layers a patch lists are BLENDED — `ALPH` and the per-patch
   `PTCH -> COLR/ALPH` overrides.
5. **CORE.LVL hashed chunks** `0x5E262256`/`0x9A644D15` (12 KB) — names not
   cracked (not in ELF strings); likely a boot/console config. Also the six
   BF2 LOAD `lvl_` names (`0xA9EE4593`+n·0x01000193 — same base name with a
   trailing digit 1–6, base string unknown).
6. **BF2 SIDE/shell `lvl_` section names** — hashes unresolved; harvest
   candidate names from BF2 Lua bytecode strings (`ReadDataFile` args embed
   `side\all.lvl;section` names) to crack them.
7. **FOLG** 4-B header meaning; foliage map semantics (u16 per cell).
8. **Sound**: `snd_`/hashed Stream/SampleBank chunks reference stream banks
   that live in the excluded movie/sound files; bake OGG pipeline needs the
   A3 audio recon first.
