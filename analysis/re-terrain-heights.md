# RE: PS2 terrain height reconstruction (HGT8 + HEXP)

Answers a2-bake-formats.md §7 open questions 1 (HGT8↔HEXP), 4 (PTCH INFO,
partially) and 7 (FOLG header). All ELF evidence is from the Ghidra decomp
corpora (`SWBFRONT.ELF-decomp.c`, `SWBFRNT2.ELF-decomp.c`); all data claims
verified programmatically against `extracted/bf1/lvl/DATA/_LVL_PS2/TAT/TAT1.LVL`
(verification script: scratchpad `re-terrain/verify.py`, results reproduced
below).

Confidence key: **[V]** verified in ELF code *and* against TAT1 data,
**[E]** clear in ELF code, not separately data-verified, **[H]** hypothesis.

---

## 1. The algorithm (TL;DR) — [V]

Terrain heights are stored as one u8 per grid cell (HGT8) plus one 8-byte
record per 16×16-cell "storage patch" (HEXP). Reconstruction to an integer
height in "raw units", then to world Y:

```
raw(x, z)   = (HGT8[cellIndex(x,z)] << HEXP[p].shift) + HEXP[p].offset   // int math
worldY(x,z) = raw(x, z) * heightScale                                     // f32, heightScale from tern INFO
```

`HEXP.shift` is an unsigned byte used as a left-shift count (the "EXP" in
HEXP = exponent); `HEXP.offset` is a **signed i16** added after the shift.
`heightFloor`/`heightCeiling` from INFO do **not** participate in
reconstruction — they are only used as the terrain AABB min/max Y
(BF1 FUN_00381408 region, line ~372130: bbox built from
`DAT_005646a0`/`DAT_005646a4`).

There is no intermediate expanded buffer: collision, decal/overlay meshes and
the VU vertex packer all recompute `(h8 << shift) + offset` on the fly from
the RLE-decoded u8 buffer + the HEXP array.

## 2. Memory layout & indexing — [V]

The RLE-decoded HGT8 buffer (gridSize² bytes; 65536 for TAT1) is **not** a
row-major image. It is stored patch-major, exactly as it appears in the file
(the loader RLE-decodes straight into the buffer with no reordering):

- storage patch size `S` = 16 cells (from INFO, see §4; `Smask=15`,
  `Sshift=4`)
- patches per edge `P` = gridSize/S = 16 (TAT1) → `P² = 256` patches
- patch index `p = (x >> 4) + ((z >> 4) << 4)`   (x fastest)
- byte index  `i = (x & 15) + ((z & 15) << 4) + (p << 8)`
  (256 bytes per patch, x fastest within a patch)

Canonical form (BF1 `FUN_00382338` @ 0x00382338, decomp line 372442; the
globals are log2/mask precomputes from the INFO parse):

*(decompiler excerpt omitted from the public copy)*

Note the coordinates **wrap** (`& gridSize-1`) here; clamping to the grid is
done by the float-space callers (§5). BF2 equivalent: `FUN_00465be8`
@ 0x00465be8 (line 526631) — byte-for-byte the same algorithm, different
global addresses.

## 3. HEXP chunk format — [V]

a2 §3 called HEXP "u16 per 8×8 patch, 1024 patches". **That was wrong.**
HEXP is `(gridSize/16)²` records of **8 bytes** (256 × 8 = 2048 B on TAT1),
one per 16×16-cell storage patch, patch order x-fastest (`px + pz*16`):

```
struct HexpRecord {           // file order, little-endian
  u8  shift;        // left-shift applied to the u8 height           [V]
  u8  cutterMask;   // low nibble: per-quadrant terrain-cutter flags [E]
  i16 offset;       // signed, added after the shift                 [V]
  i16 minHeight;    // raw units; == offset in all 256 TAT1 records  [V]
  i16 maxHeight;    // raw units; true pre-quantization patch max    [V]
};
```

The loader (BF1 `FUN_004726b8` @ 0x004726b8, HEXP case at line 536636) splits
this into two parallel arrays:

- **array A** (`DAT_005647d8`): `{shift, cutterMask, offset}` — 4 B/rec, used
  by every height reconstruction site.
- **array B** (`DAT_005648bc`): `{minHeight, maxHeight}` — 4 B/rec. Stored
  once and **never read anywhere in the BF1 decomp** (single textual
  occurrence). Redundant metadata as far as the EE is concerned.

`cutterMask` usage (BF1 `FUN_00474d98` @ 0x00474d98, line 537988): the low
nibble is a 2×2 grid of flags over the patch's 8×8-cell quadrants; bit index
= `((x >> 3) & 1) | ((z >> 2) & 2)`. A set bit means "this quadrant may be
inside a terrain cutter (CUTS/CUTR) — run the plane tests"; the height query
wrapper `FUN_00473db0` @ 0x00473db0 returns **-300.0** for a cut point.
TAT1 mask values observed: {0, 4, 8}, nonzero only near the CUTS volumes.

Data verification (TAT1, all 256 records):
- `minHeight == offset` in **256/256** records → the munger picks
  `offset = patch min` and `shift = ceil(log2((max-min)/255))` (observed
  shifts 0–2, `maxHeight - offset ≤ 255 << shift` in 256/256).
- `maxHeight - max_over_patch((h8<<shift)+offset)` ∈ {0..3} (histogram
  0:185, 1:51, 2:12, 3:8) — i.e. `maxHeight` is the *pre-quantization* max;
  quantized values round down by at most `(1<<shift)-1 = 3`.
- Global reconstructed raw range: **[-10, 1057]** → ×0.06 =
  [-0.6, 63.42]. INFO heightFloor = **-0.6 exactly**; heightCeiling 63.54 =
  0.06 × 1059 where **1059 = max over records of `maxHeight`** — the INFO
  floor/ceiling are derived from HEXP min/max fields, confirming field roles.

## 4. How the tern INFO fields enter — [V]

INFO parse in `FUN_004726b8` (line 536741):

| file field (TAT1 value) | global | use |
|---|---|---|
| f32 gridUnitSize (8.0) | DAT_00564694 | cell → world scale; also DAT_005647bc = 1/gridUnit |
| f32 heightScale (0.06) | DAT_005647c0 | worldY = raw * heightScale |
| f32 heightFloor (-0.6) | DAT_005646a0 | terrain AABB min Y only |
| f32 heightCeiling (63.54) | DAT_005646a4 | terrain AABB max Y only |
| u16 gridSize (256) | DAT_00564690 | grid dim; HGT8 decodes to gridSize² |
| u16 geoPatchSize (8) | DAT_0056485c | **render/geometry** patch edge in cells → 32×32 = 1024 PTCH, (8+1)² verts |
| u16 hexpPatchSize (16) | DAT_00564860 | **height storage** patch edge in cells → 16×16 = 256 HEXP records |
| u16 textureCount (6) | DAT_00564864 | LTEX/DTEX/SCAL/AXIS/ROTN loop count |
| u16 maxTextureLayers (16) | DAT_00564858 | layer cap |
| u16 decalCount (0) | DAT_0056486c | DCAL record count (0x14 B each); a2 called this "pad" |

a2 §3's names "heightPatches(8)/texturePatches(16)" are misleading: both are
patch **sizes in cells**, 8 for the render-patch grid (PCHS/PTCH), 16 for
the height-storage grid (HEXP/HGT8 ordering). Derived precomputes:
halfExtent `DAT_00564698 = gridUnit*gridSize*0.5` (1024), and
`DAT_0056469c = halfExtent - gridUnit` (1016).

## 5. World ↔ grid mapping and interpolation — [V code, E axis semantics]

Height query at a world point (BF1 `FUN_00473710` @ 0x00473710, line 536997;
BF2 `FUN_0055ebc0` @ 0x0055ebc0):

```
fx_cell = clamp(worldX + 1024.0, 0, (gridSize-1)*8.0) / 8.0    // +halfExtent
fz_cell = clamp(worldZ + 1016.0, 0, (gridSize-1)*8.0) / 8.0    // +halfExtent - gridUnit  (!)
cx = (int)fx_cell; fx = frac;  cz = (int)fz_cell; fz = frac
``*(decompiler detail omitted)*FUN_00473db0`/`FUN_00474d98`: vec[0] gets +1024, vec[2]
gets +1016) is offset by `halfExtent - gridUnit`. Inverse (cell → world),
used verbatim by the renderer (`FUN_00482298` line 546530):
`worldX = cx*8 - 1024`, `worldZ = cz*8 - 1016`.

Interpolation is triangle-exact, not bilinear, with the split diagonal
alternating by **z-row parity** (relevant if the bake tool wants
collision-identical meshes):

- `cz` even: diagonal (0,0)–(1,1). `fx < fz` → triangle
  {(cx,cz),(cx,cz+1),(cx+1,cz+1)}, else {(cx,cz),(cx+1,cz),(cx+1,cz+1)}.
- `cz` odd: diagonal (0,1)–(1,0). `fx < 1-fz` → triangle
  {(cx,cz),(cx,cz+1),(cx+1,cz)}, else {(cx,cz+1),(cx+1,cz+1),(cx+1,cz)}.

Each corner sample is `FUN_00382338(cx,cz) * heightScale`; if the terrain
flag `DAT_005229a8` is unset every sample is -1000.0.

## 6. Renderer / VU path (why the second quantization exists) — [E]

The TerrainRenderable PCHS loader (BF1 `FUN_00481608` @ 0x00481608, PTCH
case line 546273) builds a 0x18-byte record per 8×8-cell render patch.
The VU vertex packer (`FUN_00482298` @ 0x00482298, line 546345; BF2
`FUN_005700a8`/`FUN_0056c9d8`) re-quantizes each vertex height to u16 for
the GS/VU stream:

```
h16 = (raw(x,z) - ptch.heightMin) << ptch.heightShift      // fits u16
// VU dequant range set up as floats:
yMin = ptch.heightMin * heightScale
yMax = (ptch.heightMin + (0x10000 >> ptch.heightShift)) * heightScale
```

i.e. per-render-patch u16 heights over [min, min + 65536>>shift] raw units.
X/Z within the patch are packed as `cell * 0x2000` (0xffff at cell 8).
This second quantization is render-only; it round-trips exactly (shift is
chosen so `(raw-min)<<shift ≤ 0xffff`), so the bake tool can ignore it.

## 7. PTCH INFO (14 B) field meanings — secondary goal

Read order in `FUN_00481608` (stored into the 0x18-B record):

| # | type | → rec | meaning | conf |
|---|---|---|---|---|
| 1 | u16 | +0x08 | texture-layer bitmask; layer count via `FUN_00481328` (popcount-ish) drives the ALPH stream stride `(8+1)*(8/2+1)*nLayers` | [E] |
| 2 | u16 | +0x0a | per-patch entry count, prefix-summed into +0x0c (running start index into some global per-patch resource; consumer not identified — possibly foliage/detail instances) | [H] |
| 3 | i16 | +0x0e | patch min height, raw units — VU quantization base; also patch AABB minY (×heightScale) | [E] |
| 4 | i16 | +0x10 | patch max height, raw units — patch AABB maxY (×heightScale), culling (`FUN_00487ad8` line 549244 builds the bounding volume from +0x0e/+0x10) | [E] |
| 5 | u16 | +0x12 | height quantization shift for the VU u16 packing (§6) | [E] |
| 6 | u32 | +0x14 flag | nonzero → patch has replacement cutter-clipped geometry: sub-chunks INFO/POSI (f32? /6 stride check)/STRP/COLR + one more (tag in `a local`-relative compare) parsed into a 0x18-B side struct; renderer uses it instead of the procedural 8×8 grid (`FUN_00487158` line 548316: `if (rec[+0x14] == 0) build procedural`) | [E] |

## 8. FOLG (bonus) — [E]

> **SUPERSEDED 2026-09-26 by `analysis/re-foliage.md` §0–§2 ([O], verifier-checked, placement reproduced from PS2
> RAM).** The loader and nibble reading below hold, but the "2× terrain grid resolution" line is wrong for BF1: its
> FOLG is always 512² at a fixed 16 m cell (`DAT_006270cc` = 16.0, FUN_00343b08), so a 256 × 8 m map uses cells
> 192..320; BF2's FOLG is (gridSize/2)² at 2 × the grid unit. A nibble is four layer bits (the map's `prp_` layers),
> not a type index.

Loader case `0x474c4f46` (line 536700): reads a **u32 = byte length of the
payload** (131072), allocates exactly that, reads the rest raw into
`DAT_005229b8`. So the "4-byte header" is just the data size.
Consumer `FUN_00344d98` @ 0x00344d98 (line 331498):

*(decompiler excerpt omitted from the public copy)*

FOLG is a **512×512, 4-bit-per-cell** foliage map at 2× terrain grid
resolution (131072 = 512·512/2), not "u16 per 256×256 cell" as a2 guessed.

## 9. Function address map

| role | BF1 (SWBFRONT.ELF) | line | BF2 (SWBFRNT2.ELF) | line |
|---|---|---|---|---|
| top-level chunk dispatch ('tern') | FUN_00175d00 | 17308 | (caller of loader) | 18303 |
| tern loader (INFO/HEXP/HGT8-RLE/FOLG/LUMI/CUTS) | FUN_004726b8 @0x4726b8 | 536566 | FUN_0055db58 @0x55db58 | 694491 |
| raw height sample (u8+HEXP → int) | FUN_00382338 @0x382338 | 372442 | FUN_00465be8 @0x465be8 | 526631 |
| world-space height query (triangle interp) | FUN_00473710 @0x473710 | 536997 | FUN_0055ebc0 @0x55ebc0 | 695031 |
| height query + cutter (-300 if cut) | FUN_00473db0 @0x473db0 | 537211 | — (not chased) | |
| cutter point test (uses HEXP.cutterMask) | FUN_00474d98 @0x474d98 | 537988 | analog inline | |
| PCHS/PTCH loader + ALPH/COLR streams | FUN_00481608 @0x481608 | 545918 | ~FUN_0056c9d8 | 703647 |
| VU patch vertex packer (u16 requant) | FUN_00482298 @0x482298 | 546345 | FUN_005700a8 @0x5700a8 | 705604 |
| patch culling (PTCH min/max Y) | FUN_00487ad8 @0x487ad8 | 549068 | | |
| foliage nibble lookup | FUN_00344d98 @0x344d98 | 331498 | | |

RLE codec (HGT8 case in the loader, line 536667) is exactly the a2 §4 codec:
control byte `c`; `c < 0x80` → copy `(c&0x7F)+1` literals; else repeat next
byte `(c&0x7F)+1` times; decode until gridSize² bytes are written.

## 10. Worked examples (unit-test vectors, TAT1.LVL)

TAT1 tern @ file offset 0xC1E1C; HEXP payload @ 0xC1F98; HGT8 payload
@ 0xC27A0 (63953 B → 65536 B). INFO: gridUnit=8.0, heightScale=0.06
(f32 0x3D75C28F ≈ 0.05999999865889549), gridSize=256, S=16, P=16.

**Example A** — patch p=0 (px=0,pz=0), cell (x=0,z=0):
- HGT8 decoded[0] = **28**; HEXP[0] = {shift=2, mask=0, offset=177,
  min=177, max=885}
- raw = (28 << 2) + 177 = **289**; worldY = 289 × 0.06 = **17.34**
  (f32: 17.339999612420797)
- grid (0,0) → world (x,z) = (0·8−1024, 0·8−1016) = **(−1024, −1016)**

**Example B** — patch p=17 (px=1,pz=1), cell (cx=5,cz=3) → grid (21,19):
- byte index = 17·256 + 3·16 + 5 = **4405** → HGT8 = **96**;
  HEXP[17] = {shift=2, offset=111}
- raw = (96 << 2) + 111 = **495**; worldY = 495 × 0.06 = **29.7**
  (f32: 29.69999933615327)
- world (x,z) = (21·8−1024, 19·8−1016) = **(−856, −864)**

**Global invariants** (good regression checks for the Rust bake tool):
- min over all cells of raw = −10 → ×0.06 = heightFloor (−0.6) exactly
- max over all HEXP.maxHeight = 1059 → ×0.06 = heightCeiling (63.54)
- HEXP.minHeight == HEXP.offset for all 256 records
- 0 ≤ maxHeight − max_cell(raw) ≤ (1<<shift)−1 per patch

## 11. Confidence summary

- Reconstruction formula, HGT8 patch-major layout, HEXP record layout
  (shift/offset/min/max), heightScale role, floor/ceiling non-participation:
  **[V]** — matched in two independent ELF sites each (collision + VU
  packer, BF1 + BF2) and reproduced numerically against TAT1 (floor exact,
  offset==min 256/256).
- cutterMask nibble semantics, world-mapping z-offset (−1016), triangle
  split parity, VU u16 requant, PTCH fields 1/3/4/5/6, FOLG: **[E]** —
  single clear ELF reading, not data-cross-checked.
- PTCH field 2 (+0x0a prefix-sum count): **[H]** — consumer not traced.
