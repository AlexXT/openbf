# RE: Soldier animation banks (`zaa_`) — frame counts for gameplay timings

Purpose: pin the exact frame counts behind the provisional timings in
`re-stance-roll.md` §3.4/§3.6/§4.5 (roll duration, prone/getup transition
locks). Those durations are computed at runtime from LVL animation data, not
the ELF; this note cracks the bank format far enough to read the counts
straight off the disc files.

Sources: `analysis/SWBFRONT.ELF-decomp.c` (BF1), `analysis/SWBFRNT2.ELF-decomp.c`
(BF2), raw ELFs in `extracted/`, LVLs in `extracted/{bf1,bf2}/lvl/`.
Parser used for every number below: scratchpad script (logic reproduced in §2;
16-byte MINA entries, CRC-32 name hash). **[V]** = verified against decompiled
code and real bytes; **[H]** = hypothesis.

---

## 1. Where the soldier banks live

| game | file | banks |
|------|------|-------|
| BF1 | `DATA/_LVL_PS2/COMMON/ENG/COMMON.LVL` | `human_animation_0..4`, `human_animation_fp`, `human_animation_alert`, `human_animation_lowrez` |
| BF2 | `DATA/_LVL_PS2/INGAME.LVL` | `human_0..human_4`, `humanfp`, `humanlz` |
| BF2 | `DATA/_LVL_PS2/INSHELL.LVL` | `human_0..human_4` (byte-identical MINA values to INGAME) |

No `human*` banks exist in any BF1 SIDE/planet LVL (those `zaa_` chunks are
vehicles/props — full scan done). BF2 SIDE LVLs carry hero/droid banks
(`human_sabre`, `bdroid`, …) but the trooper set is INGAME.LVL. Only the ENG
COMMON.LVL is extracted locally; other languages assumed identical **[H]**.

The engine builds one global runtime bank object at startup:

- **BF1** `FUN_0022d478` → `FUN_0022d828(alloc 0x16a0)` → `DAT_00517ae8`.
  It looks up six sub-banks **by NAME hash** in the global anim registry
  (hash table at 0x6ff2d8, filled by the `zaa_` NAME parser):
  `human_animation_0..4` + `human_animation_alert` (strings at ELF
  0x535410..0x535488, hashed by `FUN_0022f900`). `_fp`/`_lowrez` are separate
  (1st-person rig / LOD). **[V]**
- **BF2** `FUN_00281228` → `DAT_005fdd48`; base name `"human"` (ELF 0x620a80),
  sub-banks `human_0..4`. **[V]**

## 2. Chunk format (as far as needed)

### 2.1 `zaa_` container **[V]**

Standard ucfb chunk (`[tag][u32 size][payload]`, pad to 4). Children
(BF1 loader `FUN_004be900`, dispatched from the `lvl_` walker at the
`0x5f61617a` case; BF2 equivalent path at decomp line ~18379 → `FUN_005aacb0`):

- `NAME` — bank name, NUL-terminated string. Hash of this name (see §2.3)
  registers the bank in the global registry.
- `BIN_` — the munged animation data (below).
- `EVNT` (BF1 `FUN_004beae0`) — optional per-clip event lists:
  `u16 clipCount`, then per clip { cstring clipName, `u16 eventCount`, per
  event { `u16 frame`, cstring eventName } }. Runtime stores each event time
  as `frame / clipFrames` — the clip's frame count is fetched via the same
  record used below. **[V]** (structure; not needed for timings)

`zaf_` chunks (skeletons, `0x5f66617a` → BF1 `FUN_004bec78`) have the same
NAME/BIN_ shell; not needed for frame counts.

### 2.2 `BIN_` payload = SMNA blob **[V]**

Parsed by BF1 `FUN_004cdf28` (header) + `FUN_0043d528`/`FUN_0043d590`
(section walk) + `FUN_00446c20` (per-clip record build):

```
+0x00  char[4] "SMNA"
+0x04  u32     0
+0x08  u32     version        (must be 8 — checked in FUN_004cdf28)
+0x0c  u32     totalBoneTracks (sum of bones over all clips; TNJA entry count)
+0x10  u32     tadaBytes      (size of compressed keyframe payload)
+0x14  u16     animCount
+0x18  chunk "MINA": size = animCount*0x10, entries back-to-back, NO header:
        +0x00  u64  boneFlagMask   (per-bone bit; bit i tested for track i in FUN_00446c20)
        +0x08  u32  nameCrc        (hash of full clip name, e.g. "human_rifle_diveforward")
        +0x0c  u16  FRAMES         <-- the number the game divides by 30
        +0x0e  u16  bones
… chunk "TNJA": totalBoneTracks entries x 0x30 bytes (per-track quantization/offset info)
… chunk "TADA": tadaBytes of bit-packed keyframe data
```

The runtime chain that proves `frames` is the u16 at MINA entry +0xc:
`FUN_0043d590` walks MINA (`0x414e494d`), TNJA (`0x414a4e54`), TADA
(`0x41444154`), calling `FUN_00446c20(rec, minaEntry, …)` which stores
`rec+4 = u32 entry+8` (crc), **`rec+8 = u16 entry+0xc` (frames)**,
`rec+10 = u16 entry+0xe` (bones), and inserts `rec` into the bank's hash
table keyed by crc (`FUN_004449f8`, probed by `FUN_0043d718`). Duration
getters then read `*(u16*)(rec+8)`:

- **BF1** `FUN_0022f3d8(DAT_00517ae8, id)` → `frames * 0.033333335` (= F/30).
- **BF2** `FUN_00278398(wrapper)` → raw u16; every use site computes
  `(frames − 1) * 0.033333335` (e.g. roll exit at decomp line ~160681,
  getup lock ~161902). A missing clip returns frames = 0 → duration
  `(0−1)/30 = −0.033 s`, i.e. the lock expires immediately. **[V]**

### 2.3 Name hash = CRC-32, not FNV **[V]**

Anim clip names and bank NAMEs use the engine's **CRC-32**
(BF1 `FUN_00444bb8`, table `DAT_00521ca8`): MSB-first, polynomial
**0x04C11DB7**, init 0xFFFFFFFF, final complement, with `A-Z` lowercased
before hashing (i.e. CRC-32/MPEG-2 of the lowercased string, complemented).
This is distinct from the FNV-1a-`|0x20` hash used for ODF keys/config chunks.
Table start dumped from the ELF: `{0, 0x04C11DB7, 0x09823B6E, …}`.

### 2.4 Special-anim id → clip name **[V for BF1; H for BF2 set binding]**

BF1 `FUN_0022d828` fills the id table at `DAT_00517ae8+0xcc` by formatting
**`"human_rifle_%s"`** (ELF 0x535248) with the name from the pointer table at
**0x00517af0** (id 0x14 = `diveforward`, 0x16 = `stand_getdown_prone`,
0x17 = `prone_getup_stand`, 0x18 = `crouch_getdown_prone`,
0x10 = `stand_getupfront`, …), CRC-hashing it, and searching the six
sub-banks in order (`FUN_0022f888`). So **every BF1 soldier's special-anim
durations come from the `human_rifle_*` clips**, regardless of carried weapon.

BF2's name table at **0x005fde48** (dumped): 0x00 `sprint`, 0x01 `jump`,
0x06 `landsoft`, 0x07 `landhard`, **0x14 `stand_getupfront`**,
0x15 `stand_getupback`, **0x18 `diveforward`**, **0x1b `prone_getup_stand`**,
**0x1c `crouch_getdown_prone`**, 0x1d `prone_getup_crouch`, … 0x27.
(Correction to re-stance-roll §4.1, which lists getupfront at 0x11 — the
dump says 0x14; the diveforward/prone ids there are confirmed.)
`FUN_00281510(bank, id, set)` reads `bank + 0x28 + id*8 + set*0x4b8`; all
stance-machine call sites pass `set = 0`. The set→weapon-prefix fill happens
inside the 83 KB anim-set giant `FUN_00282f78` and was **not** fully traced;
set 0 = `human_rifle` is inferred from BF1 parity and from the clip inventory
(the only humanoid `diveforward` clips shipped are `human_rifle_` = 35 fr and
`human_tool_` = 49 fr, in both games). **[H on the binding, V on inventory]**

### 2.5 Worked example (real bytes) **[V]**

`crc32("human_rifle_diveforward") = 0xDC2308EB`.

BF1 `COMMON/ENG/COMMON.LVL`: `zaa_` at file 0x201060, `NAME` @0x201068 =
`human_animation_2`, `BIN_` payload @0x20108C =
`SMNA, 0, ver 8, 1430 tracks, 61151 TADA bytes, 55 anims`; `MINA` @0x2010A4
(size 0x370 = 55×16). Entry #9 @ **file 0x20113C**:

```
05 00 00 00 00 00 00 00  eb 08 23 dc  23 00  1a 00
boneMask=0x5            crc=DC2308EB  fr=35  bones=26
```

BF2 `INGAME.LVL`: `zaa_` @0x1ACEAC, `NAME` = `human_1`, `MINA` @0x1ACEE4,
entry #0 @ **file 0x1ACEEC** — byte-identical: `fr=35, bones=26`. The two
games ship the same diveforward clip.

---

## 3. Frame counts and derived durations

All values read from MINA entries and matched by CRC of the full clip name;
"bank" gives the sub-bank (and therefore file) each number came from.

### 3.1 BF1 — `COMMON/ENG/COMMON.LVL` (dur = F/30) **[V]**

| clip (human_rifle_…) | bank | frames | duration | gameplay use (re-stance-roll §) |
|---|---|---:|---:|---|
| `diveforward` | human_animation_2 | **35** | 1.1667 s | roll exits at 0.75×F/30 = **0.8750 s** (§3.6) |
| `stand_getdown_prone` | human_animation_4 | **40** | **1.3333 s** | stand→prone lock, id 0x16 (§3.4) |
| `crouch_getdown_prone` | human_animation_2 | **40** | **1.3333 s** | crouch→prone lock, id 0x18 (§3.4) |
| `prone_getup_stand` | human_animation_4 | **45** | **1.5000 s** | prone→stand lock, id 0x17 (§3.4) |
| `prone_getup_crouch` | human_animation_4 | 44 | 1.4667 s | id 0x19 (unused by lock writers found) |
| `stand_getupfront` | human_animation_4 | **58** | **1.9333 s** | getting-up state 9 timer, id 0x10 (§3.4) |
| `stand_getupback` | human_animation_4 | 53 | 1.7667 s | id 0x11 |
| `landsoft` | human_animation_2 | 17 | 0.5667 s | (landing locks are hardcoded 10.6/0.5, not clip-based) |
| `landhard` | human_animation_2 | 34 | 1.1333 s | |
| `jump` / `fall` | anim_2 / anim_4 | 1 / 1 | — | pose holds |

(`human_tool_diveforward` = 49 fr and a full `human_tool_*` prone set also
exist in anim_4 — relevant only if a unit's anim set binds the `tool` class.)

### 3.2 BF2 — `INGAME.LVL` (dur = (F−1)/30) **[V]**

| clip (human_rifle_…) | bank | frames | duration | gameplay use |
|---|---|---:|---:|---|
| `diveforward` | human_1 | **35** | 1.1333 s | roll exits at 0.75×(F−1)/30 = **0.8500 s** (§4.5) |
| `stand_getupfront` | human_0 | **58** | **1.9000 s** | getup lock, id 0x14 (`FUN_00246850` states 9/0xa/0xb) |
| `stand_getupback` | human_0 | 53 | 1.7333 s | id 0x15 |
| `stand_getdown_prone` | — | **absent** | — | not even in the id table (prone removed) |
| `crouch_getdown_prone` | — | **absent** (clip not shipped) | (0−1)/30 = −0.033 s | id 0x1c lookup returns null → no lock |
| `prone_getup_stand` | — | **absent** (clip not shipped) | −0.033 s | id 0x1b, used in `FUN_00246850` case 2 → lock expires instantly |
| `landsoft` | human_1 | 23 | 0.7333 s | |
| `landhard` | human_1 | 36 | 1.1667 s | |
| `jump` | human_1 | 31 | 1.0000 s | (BF1's jump clip was 1 frame) |

### 3.3 The headline numbers

- **BF1 roll duration = 0.75 × 35/30 = 0.875 s** (then returns to previous stance).
- **BF2 roll duration = 0.75 × (35−1)/30 = 0.85 s** (then stand + 0.3 s soft lock).
- **BF1 prone locks: down 1.3333 s (both stand→ and crouch→), up 1.5 s;
  getup-after-tumble 1.9333 s.**
- BF2 shipped no prone transition clips at all — corroborates that prone was
  cut, and shows its residual prone code would run lock-free if re-enabled.

---

## 4. Confidence & open questions

Verified **[V]**:
- MINA entry layout (u64 mask / u32 crc / u16 frames / u16 bones) — proven by
  the runtime builder `FUN_00446c20` field-for-field, and by byte dumps.
- Name hash = lowercased CRC-32 poly 0x04C11DB7 (init −1, final ~) — proven by
  reading `FUN_00444bb8` + its table, and by ~90 exact name matches across
  both games with structured names (`human_<class>_<stance>_<action>`);
  BF1↔BF2 shared clips carry identical CRCs and identical frame counts,
  which double-confirms both hash and field decode.
- BF1 duration = F/30 (`FUN_0022f3d8`), BF2 = (F−1)/30 (call sites of
  `FUN_00278398`); roll exit fraction 0.75 in both.
- BF1 special-anim naming `human_rifle_%s` and the six-bank search order.

Open / hypotheses **[H]**:
(2026-10-05: item 1 is ANSWERED in `re-soldier-anim-layers.md` 6 - set 0 is the (`human`, `rifle`) set in every
battle state read from PS2 memory, and its `diveforward` lower slot is `human_rifle_diveforward`, 35 frames.)
1. **BF2 set-0 → `human_rifle` binding** is inferred, not traced through
   `FUN_00282f78`. Risk is low (BF1 parity, clip inventory), but a PCSX2 peek
   at `[[DAT_005fdd48]+0x28+0x18*8]` → `[ptr]+8` (u16) confirming 35 would
   close it. Unit classes that bind the `tool` anim class (engineer?) may
   roll on 49 frames = 1.2 s (BF2: 0.75×48/30) — check ODF `AnimationName`.
2. Non-ENG COMMON.LVL copies assumed to carry identical banks (not extracted
   locally).
3. `human_rifle_sprint` is absent from BF2 INGAME.LVL (`human_bazooka_sprint`
   and `human_tool_sprint` exist, 15 fr) — sprint pose for rifles must come
   through another path (blend/alert set?). No gameplay timing depends on it
   (sprint uses ODF `SprintAccelerateTime`), noted as a curiosity.
4. TNJA (0x30/track) and TADA (bit-packed keyframes) internals were not
   decoded — not needed for timings; needed only for actual pose playback.
5. Doc nit: re-stance-roll §4.1 lists BF2 `stand_getupfront` at id 0x11;
   the ELF table at 0x5fde48 says **0x14** (0x11 is `thrown_bouncebacksoft`).

Reproduce: parse any `zaa_` chunk as §2.2; hash names as §2.3. The scan
script logic (16-line parser + CRC) is embedded above; ~10 s over all LVLs.
