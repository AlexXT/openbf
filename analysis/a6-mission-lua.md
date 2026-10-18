# A6 — Mission Lua extraction + PC mod-tools cross-check

Tooling: `scripts/extract_lua.py` (walks the ucfb container, finds `scr_` NAME/INFO/BODY
and `mcfg` chunks anywhere in the tree, dumps them, parses Lua dump headers, extracts
string constants, and FNV-matches 4-byte hashes). Outputs live under `extracted/bf1/lua/`
and `extracted/bf2/lua/` (gitignored), organized per source LVL:
`mission/`, `common/`, `shell/` (+ bf2 `ingame/`, `inshell/`).

## Container layout (as found)

- Both games: `ucfb` → chunks `[4-byte tag][u32 LE size][payload][pad4]`.
- **BF1 MISSION.LVL**: `ucfb` → 44× `lvl_`; each `lvl_` payload is
  `[u32 hash][u32 inner_size]` then a `scr_` chunk (`NAME` cstring, `INFO` u8=1,
  `BODY` Lua dump). The `lvl_` prefix hash is **FNV-1a-32 of the lowercased script
  name** (basis 0x811C9DC5, prime 0x1000193) — verified: fnv1a("bes1a") = 0x8618BD6B.
- **BF2 MISSION.LVL**: `ucfb` → 18× top-level `scr_` (shared logic scripts) +
  1 top-level `mcfg` + 107× `lvl_` wrappers, each containing one or more `scr_`
  and/or `mcfg` chunks. **BF2 `scr_` NAMEs are plain strings** (e.g. `cor1c_con`) —
  the "hashed names" premise applies only to `mcfg` chunks, whose `NAME` is a 4-byte
  hash. The BF2 `lvl_` prefix hash is *not* FNV-1a of the contained script name
  (algorithm unresolved; FNV seed back-computation yields nothing recognizable).

## Census

| LVL | scr_ | mcfg | Lua version (header) |
|---|---|---|---|
| bf1 MISSION.LVL | 44 | 0 | all 4.0 (0x40) |
| bf1 COMMON/ENG/COMMON.LVL | 39 | 0 | all 4.0 |
| bf1 SHELL/ENG/SHELL.LVL | 64 | 1 | all 4.0 |
| bf1 CORE.LVL | 0 | 0 | — |
| bf2 MISSION.LVL | 132 (128 unique) | 19 | all 5.0 (0x50) |
| bf2 COMMON.LVL | 42 | 0 | all 5.0 |
| bf2 SHELL.LVL | 92 | 2 | all 5.0 |
| bf2 INGAME.LVL | 7 | 0 | all 5.0 |
| bf2 INSHELL.LVL | 0 | 1 | — |
| bf2 SHELLPS2.LVL / CORE.LVL | 0 | 0 | — |

Every single script BODY is **precompiled bytecode** — zero plain-source scripts.
(BF1 COMMON/SHELL exist only in ENG; the other language dirs are empty.)

Header details (identical across every script in each game):

- BF1: `1B 4C 75 61 40` — Lua **4.0** dump, little-endian, sizeof(int)=4,
  sizeof(size_t)=4, 32-bit Instructions, **sizeof(Number)=4 (float!)**,
  test number 3.14159e8. Matches the engine's embedded Lua 4.0.1.
- BF2: `1B 4C 75 61 50` — Lua **5.0** dump, little-endian, int=4, size_t=4,
  Instruction=4 (OP:6 A:8 B:9 C:9 — stock layout), **sizeof(Number)=4 (float)**,
  test number 3.14159e7. Matches Lua 5.0.2 (Gametoast mirrors the exact interpreter
  as [Gametoast/Lua-5.0.2](https://github.com/Gametoast/Lua-5.0.2)).
- Takeaway for the port: PS2 Lua was built with `LUA_NUMBER = float`, so all
  numeric constants in mission data are single-precision.

### BF1 missions (44)

25 playable missions + 19 `_h` historical-campaign variants:
`bes1a[_h] bes1r bes2a[_h] bes2r end1a[_h] geo1r[_h] hot1i[_h] kam1c[_h]
kas1c[_h] kas1i kas2c[_h] kas2i nab1c[_h] nab1i nab2a nab2c[_h] rhn1i rhn1r[_h]
rhn2a[_h] rhn2c tat1i[_h] tat1r tat2i[_h] tat2r yav1c yav1i[_h] yav2i[_h] yav2r`.
Each is a self-contained `ScriptInit()` doing team/unit/objective/sound setup inline.

### BF2 missions (128 unique names)

Worlds: cor dag dea end fel gal geo hot kam kas mus myg nab pol spa(1-4,6-9) tan
tat(2,3) uta yav + test1{c,g}. Mode suffixes: `_con` (conquest), `_ctf` (2-flag),
`_1flag`, `_hunt`, `_eli` (hero assault), `_ass` (space assault), `_cmn` (space
common), `_c` (campaign — the big 16–34 KB scripts). Shared logic ships as
top-level scripts: `setup_teams`, `Objective` + `ObjectiveConquest/CTF/OneFlagCTF/
TDM/Assault/SpaceAssault/Goto`, `SoundEvent_ctf`, `MultiObjectiveContainer`,
`LinkedTurrets/Shields/Destroyables`, `ambush`, `gametype_conquest/capture`,
`PlayMovieWithTransition`. BF2 mission scripts are object-oriented: mission files
instantiate `ObjectiveX:New{...}` from the shared scripts (unlike BF1's inline style).

### mcfg = music config, not missions

The `mcfg` DATA property hashes resolve by FNV-1a against ELF strings to
`Name`, `Segment`, `SegmentList` — these chunks are munged **music/sound-segment
configuration** (one per world-group in MISSION.LVL, plus shell ones). Their 4-byte
NAME hashes did **not** resolve (0/21 unique) against FNV-1a of mission names, world bases,
music-flavored candidates (`*_music`, `*gcw`, ...), or the full ELF string dumps.
Notably `mcfg 12d97623` appears in *both* BF1 SHELL.LVL and BF2 SHELL.LVL (same
name in both games), and BF2 MISSION.LVL's top-level `mcfg ca9a17f0` is identical
in name to INSHELL.LVL's. Since the actual mission scripts all carry plain-text
names, hash reversal is only needed for the music configs — low priority.

## PC mod-tools cross-check

**Reference repo: [Gametoast/Documentation](https://github.com/Gametoast/Documentation)**
(the Gametoast community's mirror of the 2004/2005 LucasArts/Pandemic mod-tools
assets), branch `main`:

- BF1: `StarWarsBattlefront/SourceCode/Common/Scripts/` (127 .lua) +
  `.../Shell/Scripts/` (74 .lua)
- BF2: `StarWarsBattlefrontII/SourceCode/Common/scripts/` (247 .lua) +
  `.../Shell/scripts/` (133 .lua)

(Also relevant: [SWBF2-Lua-API](https://github.com/Gametoast/SWBF2-Lua-API) for API
docs; [BAD-AL/BF1LuaDC](https://github.com/BAD-AL/BF1LuaDC) BF1 Lua-4 decompiler;
[SWBFSpy/ZeroFront](https://github.com/SWBFSpy/ZeroFront) carries modded BF1 mission
sources. Comparison was done locally against downloaded PC sources; no game data
was uploaded anywhere.)

### Name coverage (PS2 script → PC .lua of same name)

| set | covered | missing |
|---|---|---|
| bf1 mission (44) | 44/44 | — |
| bf1 common (39) | 39/39 | — |
| bf1 shell (64) | 63/64 | `popup_loadsave` (PS2 memory-card popup) |
| bf2 mission (128) | 127/128 | `gal1` |
| bf2 common (42) | 41/42 | `controller_presets` |
| bf2 shell (92) | 91/92 | `ifs_trailer` |
| bf2 ingame (7) | 7/7 | — |

### Content similarity (string-constant overlap, PS2 bytecode vs PC source)

- **bf1 tat1r: 134/134 (100%)**, **bf1 bes1a: 140/140 (100%)** — every string
  constant in the PS2 bytecode appears verbatim in the PC source; call order in the
  bytecode matches the source's statement order.
- bf2 tat2g_con: 128/153 (84%), cor1c_con: 163/191 (85%), setup_teams: 23/28 —
  and on inspection every "PS2-only" token except one is a table key / method name
  (`CommandPost`, `New`, `SetupTeams`, `ATT`, `name`, `units`, ...) that the
  quick PC-side literal harvest simply didn't capture; they are present in the PC
  sources as identifiers. The one genuine delta found: PS2 `tat2g_con` loads
  `imp_walk_atst`, absent from the PC `tat2g_con.lua` — small per-platform tuning.

**Verdict: the PC mod-tools Lua sources ARE the mission scripts** — the PS2 LVLs
contain the same scripts compiled (comments stripped by luac), with occasional
minor PS2 tuning (unit lists, memory-pool sizes). Use
`Gametoast/Documentation/.../SourceCode/*/scripts` as the readable reference for
all mission/mode logic instead of decompiling bytecode.

## Implications for RON mission data (no Lua VM in v1)

1. **Mission scripts are mostly declarative.** `ScriptInit` is a linear sequence of
   engine calls (`ReadDataFile`, `AddUnitClass`, `SetUnitCount`,
   `SetReinforcementCount`, `AddMissionObjective`, `OpenAudioStream`,
   `SetBleedingVoiceOver`, `SetMemoryPoolSize`, ...). This transcribes directly to a
   RON `Mission { teams, unit_classes, objectives, sound_streams, ... }` record.
   Memory-pool hints can be dropped for the port.
2. **BF2's shared objective scripts are engine logic, not data.** `ObjectiveConquest`
   / `CTF` / `1flag` / `hunt` / `eli` / space-assault should be ported once each as
   native Rust game-mode systems; per-mission RON then only parameterizes them
   (command posts, flags, spawn paths, thresholds). BF1 inlines conquest logic
   per-mission but it's formulaic — the same native modes cover it.
3. **Authoring flow**: generate RON from the PC .lua sources (trivially parseable
   call sequences), then **validate against the PS2 bytecode string dump**
   (`extract_lua.py --strings`) to catch PS2 deltas like `imp_walk_atst`.
   Where they differ, PS2 wins (it's the target).
4. **Shell/UI scripts (ifs_* / ifelem_*) are a full Lua UI framework** — out of
   scope for RON mission data; v1 should use native menus.
5. Numbers are 4-byte floats in both games' Lua — no double-precision data exists.

## Open questions

1. **Numeric constants weren't compared** (string-level diff can't see unit counts /
   reinforcement totals). Before trusting PC sources verbatim for RON numbers,
   spot-check 2–3 missions with a real decompiler (unluac for Lua 5.0, luadec/
   BF1LuaDC for 4.0) or extend `extract_lua.py` to walk the constant tables for
   numbers — the dump format is simple and the header is already parsed.
2. **mcfg NAME hashes (21 unique) unresolved** — needed only if music-segment
   config becomes RON data. Best lead: find the mcfg loader in the ELF decomp
   (search for the FNV constants near music-segment code) to learn what string is
   hashed; or brute-force with BAD-AL/ReverseHash-style search.
3. **BF2 `lvl_` wrapper prefix hash** — not FNV-1a of the script name (BF1's is).
   Harmless for extraction (names are inside), but worth resolving for repacking.
4. **Four PS2-only scripts have no PC source** (`popup_loadsave`, `gal1`,
   `controller_presets`, `ifs_trailer`) — bytecode-only; decompile if their logic
   matters (all look shell/platform-related, `gal1` is likely PS2 galactic-conquest
   glue).
