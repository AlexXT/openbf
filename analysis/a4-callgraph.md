# A4 — Call-graph reachability: true size of the RE core

Date: 2026-08-30. Tool: `scripts/callgraph.py` (streaming parser over the
Ghidra decomp exports; rerun with `python3 scripts/callgraph.py --game bf1|bf2
--outdir analysis`). Data files: `analysis/a4-{bf1,bf2}-edges.txt` (edge list,
`src C|T|L dst`), `analysis/a4-{bf1,bf2}-reachable.txt` (reachable set with
tier + dropped tags).

## Headline

| | BF1 (SWBFRONT.ELF) | BF2 (SWBFRNT2.ELF) |
|---|---|---|
| Functions (census) | 15,747 (3.68 MB text) | 17,002 (4.63 MB) |
| Direct call edges | 39,699 | 46,297 |
| Roots (bindings + entry + startup) | 485 | 771 |
| **Live estimate (linker tier)** | **13,319 fns / 3.33 MB / 90.4%** | **16,197 fns / 4.31 MB / 92.9%** |
| Dropped-region fns (seeds + library bands) | 1,260 fns / 322 KB | 1,539 fns / 380 KB |
| **RE keep-set (live − dropped)** | **12,326 fns / 3.06 MB / 83.1%** | **14,692 fns / 3.94 MB / 85.0%** |
| Core after carving dropped subtrees | 11,497 fns / 2.83 MB / 77.0% | 14,445 fns / 3.89 MB / 84.0% |

Bottom line: **the RE job is most of the binary.** Text-only call-graph
reachability from the Lua bindings + startup gives a misleadingly small
~25%, because these are C++ engines where nearly everything is dispatched
through vtables that live in `.data`/`.rodata`, invisible in decompiled C
text. Adding function pointers found in the ELF data sections as roots
(what a conservative linker would keep) raises liveness to ~90–93%.
Carving the online/peripheral libraries off leaves a core of roughly
**2.8 MB / 11.5k functions (BF1)** and **3.9 MB / 14.4k functions (BF2)**.

## Reachability tiers (each adds an edge kind)

| Tier | Edge kinds | BF1 fns (%text bytes) | BF2 fns (%text bytes) |
|---|---|---|---|
| strict | direct calls `FUN_x(...)` only | 4,018 (23.9%) | 5,353 (25.1%) |
| closure | + address-taken identifiers (`&FUN_x`, bare refs) | 4,153 (25.0%) | 5,474 (25.8%) |
| full | + hex literals equal to a fn start (thread entries, callbacks passed as constants, e.g. BF1 SetupThread arg `0x2e8b70`) | 4,547 (27.2%) | 5,911 (27.7%) |
| linker | + fns whose address sits in `.data`/`.rodata` (vtables, callback tables): 3,859 (BF1) / 4,294 (BF2) data-resident pointers | 13,319 (90.4%) | 16,197 (92.9%) |

The strict→linker jump is the vtable effect: 5,199 indirect call sites
(*(decompiler detail omitted)*) in BF1's decomp alone, none with a textual target.
Address-taken refs in C text are nearly absent (80 / 71) because Ghidra
renders data-side pointers as `PTR_FUN_<dataaddr>` labels whose names do
not encode the target.

Startup roots: BF1 `entry` (00160008) → `FUN_002e8950` (crt0: bss clear,
SetupThread, heap init) → `FUN_003928b0` = main. BF2 `entry` →
`FUN_003c1c38` (known anchor). Known BF2 anchors cross-check the graph:
memset `FUN_0051d8e8` (fan-in 431), memcpy `FUN_0051d734` (178),
snprintf `FUN_0051fbf8` (190) all land in the top fan-in table below.

## Dropped regions

Seeds = functions whose stringrefs match
GameSpy/DNAS/EENet/PPP/headset/GSM/memcard/USB patterns (142 BF1, 132
BF2). Statically linked libraries are contiguous, so seeds cluster into
address bands; bands with ≥4 seeds are treated as dropped wholesale
(`*` below). ScriptCB_* name strings are referenced only from data-side
registration tables, so the Lua registrar is not mis-tagged.

BF1:

| Band | Seeds | Fns | Bytes | Tags |
|---|---|---|---|---|
| * 0018a998–0018b090 | 4 | 8 | 2,252 | dnas, ppp (UI glue) |
| * 00337898–003416e0 | 7 | 138 | 40,120 | eenet, gamespy, memcard, usb (IRX load/init glue) |
| * 003ba508–003d3670 | 13 | 493 | 100,876 | gamespy |
| * 003dc8b0–003e8578 | 13 | 191 | 46,892 | eenet |
| * 003f0830–003f9758 | 8 | 83 | 33,828 | ppp |
| * 00419ba8–00422840 | 15 | 168 | 34,792 | eenet, memcard, ppp |
| * 0046ee48–00470a58 | 10 | 22 | 7,192 | headset/gsm |
| * 004ee698–004fb340 | 63 | 148 | 51,652 | eenet, ppp, usb (drivers/dev9) |
| (8 single-seed outliers kept as seed-only tags) | | | | |

BF2:

| Band | Seeds | Fns | Bytes | Tags |
|---|---|---|---|---|
| * 003cb688–003d0100 | 5 | 67 | 17,460 | dnas, gamespy |
| * 003f74c8–00400618 | 4 | 225 | 36,428 | gamespy |
| * 0041e510–004203f0 | 5 | 48 | 10,508 | eenet, gamespy, memcard |
| * 004bd830–004f5ce0 | 40 | 911 | 222,140 | eenet, gamespy, ppp (the big online stack) |
| * 0050ed30–0051a080 | 16 | 158 | 44,936 | eenet, memcard, ppp |
| * 005ca7b0–005d5960 | 55 | 123 | 45,016 | eenet, ppp, usb |
| (7 single-seed outliers) | | | | |

Caution on the 00337898 (BF1) and 0041e510/0050ed30 (BF2) bands: they mix
tags and include IRX-loading / device-init glue that the startup path
calls (BF1 startup calls `FUN_003416e0`, the band's last function). These
are "dropped" in the sense of being replaced by the port's platform
layer, not deletable before a replacement exists. Memcard bands likewise
implement save/load, which the port needs to stub or reimplement.

## Hub functions — early naming candidates

Flags: R = live (linker tier), D = in a dropped band. Sizes from the
census (bytes) and decomp line counts.

### BF1 top 30 by fan-in

```
004266d8 fan-in=398  192B   49ln R   <- likely memset (matches BF2 FUN_0051d8e8: same 192B/49ln)
00444b88 fan-in=395   48B   12ln R   <- matches BF2 FUN_005393c0 (48B/12ln), top fan-in there
004b31c8 fan-in=377   40B   11ln R
00394c30 fan-in=343   32B   16ln R
00394c50 fan-in=311   28B   12ln R
004b31a0 fan-in=302   40B    9ln R
004287a0 fan-in=196  128B   51ln R
00444998 fan-in=182   92B   26ln R
003a6378 fan-in=175   20B    8ln R
00196f38 fan-in=155   40B   17ln R
003b1328 fan-in=153  120B   15ln R
003e8330 fan-in=149  204B   35ln RD  (inside eenet band; shared util — check before carving)
00394c08 fan-in=141   36B   17ln R
00426d94 fan-in=141  328B   54ln R   <- matches BF2 FUN_0051dfa4 (328B/54ln)
003ec5f0 fan-in=140   84B   17ln R
003ec648 fan-in=140   36B   13ln R
001645a8 fan-in=134    8B    8ln R
003a6a00 fan-in=133   92B   22ln R
00426524 fan-in=132  176B   48ln R   <- likely memcpy (matches BF2 FUN_0051d734: 176B/48ln)
00434110 fan-in=131   28B    8ln R
00444018 fan-in=130   64B   17ln R
00507db0 fan-in=129  500B   85ln R
00444058 fan-in=128  108B   29ln R
00443fd0 fan-in=128   72B   16ln R
003b1220 fan-in=127  116B   19ln R
00196f60 fan-in=123   80B   13ln R
001916d0 fan-in=122  132B   31ln R
0018e760 fan-in=121    8B    8ln R
00441e10 fan-in=116  168B   29ln R
00504c40 SignalSema fan-in=115 16B R
```

### BF1 top 30 by fan-out

```
00173a98 fan-out=83   1620B  213ln R    00175d00 fan-out=36  1992B  228ln R
001740f0 fan-out=83   1864B  229ln R    001e3678 fan-out=36  6188B  877ln R
0033e590 fan-out=74   2444B  336ln RD   0032ed90 fan-out=36  2704B  407ln R
00205cc8 fan-out=63  11828B 1718ln R    0033b2c0 fan-out=36  3788B  518ln RD
0033f718 fan-out=63   4220B  436ln RD   00324078 fan-out=35  1936B  277ln R
001d9180 fan-out=49  10452B 1683ln R    0048b9c0 fan-out=33  2864B  342ln R
002013a0 fan-out=49   7064B 1227ln R    004cc518 fan-out=33  1132B  170ln R
00409b18 fan-out=47   8936B 1260ln R    001ea9f8 fan-out=32  3812B  669ln R
003269e8 fan-out=43   3024B  400ln R    002df2b0 fan-out=31  3136B  352ln R
00328e68 fan-out=40   2068B  293ln R    003191f8 fan-out=31  8056B  999ln R
00390af8 fan-out=39   7056B  947ln R    00325070 fan-out=31  2264B  287ln R
004a57e0 fan-out=38   6328B  738ln R    004573e8 fan-out=31  1948B  262ln R
00172ab0 fan-out=37   1556B  162ln R    0017d9d0 ScriptCB_GetPersonalStats 30 R
001f0648 fan-out=37   6332B  963ln R    001d76a0 fan-out=30  1636B  269ln R
0035fc80 fan-out=37   1928B  286ln R    001df3f0 fan-out=30  4584B  707ln R
```

Big-fan-out + big-size functions (FUN_00205cc8 11.8KB, FUN_001d9180
10.5KB) are likely game-mode / frame-update orchestrators — best early
targets after the libc-alike utilities.

### BF2 top 30 by fan-in

```
005393c0 fan-in=537   48B 12ln R        0059d848 fan-in=170  340B R
0059d428 fan-in=437   40B  9ln R        0049fde0 fan-in=168   32B R
0051d8e8 fan-in=431  192B 49ln R  = memset (known anchor)
0059d450 fan-in=401   44B 11ln R        004240f8 fan-in=160  168B R
004acdc0 fan-in=300  120B 15ln R        0052a770 fan-in=155   28B R
00539150 fan-in=277   92B 26ln R        0059d480 fan-in=149   40B R
004accc0 fan-in=229  116B 19ln R        0059d9a0 fan-in=147  100B R
0049f338 fan-in=210   20B  8ln R        003fdd28 fan-in=140  192B RD
0049faf8 fan-in=192   32B 14ln R        004e50d0 fan-in=140  204B RD
0051fbf8 fan-in=190  144B 58ln R  = snprintf (known anchor)
0051d734 fan-in=178  176B 48ln R  = memcpy (known anchor)
00164ff8 fan-in=177    8B  8ln R        0059d4a8 fan-in=138   44B R
0049f530 fan-in=172   44B 16ln R        0051dfa4 fan-in=136  328B R
00477328 fan-in=134  176B 36ln R        0049fae0 fan-in=133   24B R
003fdde8 fan-in=130  204B RD            004e9178/004e9120 fan-in=130 RD
00164ff0 fan-in=119    8B R             001a1638 fan-in=113    8B R
```

The 0059dxxx cluster (fan-in 400+) is the C++ runtime/allocator family
(operator new/delete candidates); 005393c0 is the same 48B/12ln helper
as BF1's FUN_00444b88 (called from constructors — likely a
timestamp/refcount/lock helper).

### BF2 top 30 by fan-out

```
00175258 fan-out=107  1692B  216ln R    00173d10 fan-out=45  1860B R
00240bf8 fan-out=105 23580B 3323ln R    0026cea0 fan-out=45  7968B R
00174b28 fan-out=100  1840B  207ln R    00424e10 fan-out=45  2100B -
004241b0 fan-out=98   3164B  342ln -  <- refs "ScriptInit": mission-script init dispatcher
00202d08 fan-out=90  22156B 3191ln R    00406828 fan-out=44  2336B R
00425840 fan-out=90   4300B  476ln R    001f8548 fan-out=43  5248B R
00239bb0 fan-out=69  13344B 2514ln R    003d5110 fan-out=43  2428B R
00282f78 fan-out=68  82996B 11578ln - <- 83KB monster (cf. bf2-giant.c); refs "prone","lower","%s_%s" — animation/soldier setup; only caller is FUN_004241b0
00177058 fan-out=50   2284B  376ln R    004044c8 fan-out=42  2008B R
00505cb8 fan-out=47   8936B 1260ln R    002223a0 fan-out=41  7824B R
00398000 fan-out=46   6424B  699ln R    003db578 fan-out=41  1956B R
0040eae0 fan-out=46   4932B  694ln R    00214600 fan-out=40  6488B R
004203f0 fan-out=46   4192B  557ln RD   0021b470 fan-out=40  5048B R
003eacc0 fan-out=40   3004B  364ln R    00234a68 fan-out=39  4312B R
0020fd58 fan-out=38   8424B 1138ln R    00475740/00549b08/0025f4a0 37/37/36 R
```

## Method

1. Two-pass stream parse of the decomp C (functions delimited by
   `/* ==== addr name ==== */`). Pass 1 builds name→addr for every
   defined symbol (FUN_*, thunk_FUN_*, entry, renamed Lua bindings from
   `analysis/*-bindings.txt`); thunks get an implicit edge to their
   target. Pass 2 tokenizes whole identifiers (so
   `PTR_ScriptCB_Foo_005f914c` does not match `ScriptCB_Foo`): known
   name followed by `(` → call edge (C); otherwise address-taken (T);
   hex literal equal to a census function start → literal ref (L).
2. BFS from roots = all binding functions + `entry` + the startup anchor
   (BF1 `FUN_002e8950` found from `entry`; BF2 `FUN_003c1c38` known),
   over C, then C+T, then C+T+L.
3. Linker tier: scan the actual ELF's non-executable PROGBITS sections
   (.data, .rodata, level, .libinit, .libupd, .gcc_except_table) for
   aligned 32-bit words equal to a function start; those functions
   become additional roots (vtable slots, callback tables).
4. Dropped tagging from `analysis/*-stringrefs.txt` string patterns,
   plus band clustering (gap < 0x8000, bands with ≥4 seeds dropped
   wholesale).

## Limits (read before trusting a specific function's status)

- **Linker tier is still a lower bound on live code and an upper bound
  on nothing.** Proof by counterexample: BF2 `FUN_004241b0`
  ("ScriptInit" dispatcher) and its 83KB callee `FUN_00282f78` are
  certainly live but have zero textual or data-section references — the
  pointer is presumably computed at runtime (split %hi/%lo constants,
  or written into bss). Unreached ≠ dead; treat census − linker (BF1:
  2,428 fns / 9.6%, BF2: 805 fns / 7.1%) as *undecidable from this
  evidence* (percentages are byte shares of .text), not deletable.
  Adding just these two as extra roots (`--roots`) lifts BF2's linker
  tier from 92.9% to 97.3% of text bytes — the undecidable slice is
  mostly live code, not dead code.
- **Data-pointer scan false positives:** ~16k (BF1) / ~22k (BF2) data
  words fall in the text range; ~40% are exact function starts (vs 1.6%
  chance rate), so most are genuine, but ~150–200 coincidental words are
  expected — a few percent of the 3,859/4,294 data-referenced functions
  may be accidental, each dragging in its subtree. Effect on totals is
  small; effect on any single function's status can be total.
- **Text-tier under-counting:** Ghidra renders vtable stores as
  `&DAT_xxxx`/`PTR_FUN_<dataaddr>` (target not in the name), and 5,199
  BF1 indirect call sites have no textual target, which is exactly why
  strict reachability is only ~24%.
- **VU overlays:** the `.DVP.overlay.*` sections (VU microprograms) are
  outside the EE call graph entirely; nothing here says which VU code is
  live.
- **Census mismatch:** decomp export has 15,770/17,070 headers vs
  15,747/17,002 census rows (duplicate exports / census-run drift);
  sizes for non-census functions are counted as 0 bytes.
- **Bands over-capture:** address bands include every census function
  between first and last seed; interleaved keep-code (e.g. shared
  low-level socket utils used by both the dropped online stack and
  nothing else — fine; but also IRX/memcard init the port must replace,
  see band notes) is tagged dropped. The seed-only count (142/132) and
  the band count (1,260/1,539) bracket the truth.
- Lua-side reachability (which bindings the shipped scripts actually
  call) is not considered; all bindings are roots, which is the
  conservative choice for a port.
