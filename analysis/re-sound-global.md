# LEAD VERIFICATION (Fable lead, 2026-09-07, session 13)
Status: **VERIFIED** — claims A–F re-measured by a Fable verification agent from the raw files and the
decomp (`engine/artifacts/lead_tmp/s13/reports/sound_global_verify.md`, scripts under
`lead_tmp/s13/sound_verify_scratch/`): GLOBAL.LVL census 31 StreamList→Stream{info,data}, 6,523 records /
6,457 ids / 630,358,016 B music / 196,640,768 B VO, every data payload ≡ 0 mod 16384; `all_unit_vo_quick`
116 segments; `FUN_005547b0` 687649–687660 reads `info` only and fixes offsets at `data`; ≤ 6 slots
(45029, 671244); CORE.LVL `snd_` @0x3800 with 0x6a942216 at 0x8029/0x8785 and sample B75D27CD (11,060 Hz,
2,704 B) at bank record 0x78 — the lead re-read those bytes (`snd_` NAME = fnv1a("global") 1dff06ae, the
DATA row `8d39bde6 01 6a942216`, the sample record) with Python on the extracted file. **Corrections to the
text below:** (1) §0.3/§1.2 "the ELF keeps the file handle and streams from it" is WRONG in wording —
`FUN_00549058` closes the parse-time handle (`FUN_003cb148` @425323) and hands the PATH to `FUN_005490e8`,
which builds `cdrom0:\<path>;1` for the IOP `multi_streamer`; the port-relevant meaning (only `info` is
read at open; segments are addressed by absolute file offset in 16 KiB blocks) stands. (2) "99/291 luac"
is 99 of 269 luac (99 of 128 mission luac). (3) HOLD 9's "not in any extracted sound LVL" was a SCOPE miss:
BF2's CORE.LVL lives only under `lvl/DATA/_LVL_PS2/`, not under `sound/`; BF1's sound tree carries a
byte-identical CORE.LVL that would have hit; and `analysis/re-sound-banks.md` (2026-08-30) already listed
B75D27CD as the "accept wave" — what is new here is the BF2 location and the full property decode
(pitch 1.1, bus `shellfx`, `shell_menu_enter` inherits it). (4) Not re-verified: the 2,980-property join,
"0 of 45" tac names (1 of 45 checked: 0 hits in BF2 SOUND LVLs, 2 in BF1 TAT.LVL), `FUN_005543e8` field by
field. **Consequences:** Phase D's D12 must be re-pointed (BF1's tac chatter is in SOUND/CW.LVL and
GCW.LVL streams — `cw_tac_vo` 63 segs 1,130,496 B, `gcw_tac_vo` 62 segs 1,081,344 B — not the per-map
banks; BF2 ships command VO as `<side>_command_*` on the `<side>_unit_vo_quick` streams); the unified
sound plan (§7 below) is supported by the bytes and is brought to the maintainer in the PLAN Log 2026-09-07. Two ELF
facts the port mirrors: 6 stream slots; a slot = one open stream + MERGED segment tables from
AppendSegments.

# RE: BF2 SOUND/GLOBAL.LVL — layout, loading model, names, squad chatter, `shell_menu_accept`

Session 12 (2026-09-03), Fable RE agent. Analysis only; scratch + tables under
`engine/artifacts/lead_tmp/s12/re/sound_scratch/` (39 MB, no bank copies).
Tags: [V] verified on bytes/decomp, [D] derived from verified facts, [H] hypothesis.
Decomp lines are `analysis/SWBFRNT2.ELF-decomp.c` unless marked BF1. Hash = FNV-1a-32 with byte|0x20
(`fnv1a_lowercase`) — confirmed as the sound-name hash: `all_inf_com_chatter_tac_follow` → 0x75076C29 matches
BF1 TAT.LVL prop 1963420713 and `global_vo_slow` → 0xFE6EAE40 matches stream 1's name [V].

## §0 Headline

1. **Layout [V]:** GLOBAL.LVL (827,344,896 B) is a plain `ucfb` of **31 top-level `StreamList` (0x5BB97F21) → `Stream` (0x5F6F6D65, bytes `emo_`) {info, data}** chunks, `_pad`-aligned to 0x800. **No SampleBank, no `snd_`, no `lvl_`** — it is a pure stream file; that is exactly why the port has not read it (`bake sounds` walks SampleBanks only, `sounds.rs:7` "v0 bakes the SampleBanks only"; `is_sound_wrapper` treats StreamList/Stream as leaves).
2. **Contents [V]:** 6,523 segment records / 6,457 unique ids (58 ids appear in 2–3 streams). 826,998,784 data bytes = **630.4 MB music** (`cw_music` 308.4 MB + `gcw_music` 321.9 MB, stereo SPU2-ADPCM 44100–48002 Hz) + **196.6 MB VO** (29 mono streams, 22050–22128 Hz). Format field = 1 (SPU2 ADPCM) everywhere; 16 KiB block size; no aliases.
3. **Loading model [V]:** never loaded whole and never resident. Each **mission Lua** (99/291 luac) opens it by name at map start: `OpenAudioStream("sound\\global.lvl","all_unit_vo_slow")` + `AudioStreamAppendSegments(... "imp_unit_vo_slow"/"global_vo_slow")`, the same for `*_vo_quick`, and `OpenAudioStream("sound\\global.lvl","gcw_music"|"cw_music")`. The ELF keeps the file handle, parses only the `info` chunk, adds the file position of `data` to each segment offset (FUN_005547b0 @687649–687660) and streams 16 KiB blocks; ≤6 stream slots (AudioStreamAppendSegments @44991 *(decompiler detail omitted)*). The shell never opens it (uses `sound\shell.lvl`).
4. **Names:** streams **31/31** named (from Lua). Properties that reference GLOBAL segments (in the per-map `snd_` scripts): **1,564 / 2,980 named** (1,266 Direct + 298 Derived, 52 %). Segment ids: **0 / 6,457** — not preimageable from any harvested string (1–3-char suffix brute force and a 48×18×111 prefix/suffix/number dictionary over all 1,564 names: no convention). The runtime does not need them: a property (named) → its segment ids (offsets) is the whole chain.
5. **Squad chatter [V]:** BF2 has **no** `*_inf_com_chatter_tac_*` (0 of 45 BF1 names in any BF2 `snd_`). BF2's own set is `<side>_command_{follow,stopFollow,stopVehicle,getIn,getOut}` + `<side>_response_*` on the four `<side>_unit_vo_quick` streams (ODF `VOSound` events `SC_Follow … SC_GetOutResponse`, bf2_rep.ron:5546–5555). BF1's 9-command set lives in **SOUND/GCW.LVL `gcw_tac_vo` (62 segs, 1.08 MB) / CW.LVL `cw_tac_vo` (63 segs, 1.13 MB)** as streams referenced by per-map `soundstreamproperties` — **not baked today** (tat1 sounds.ron: `samples: []`).
6. **`shell_menu_accept` (HOLD 9) — FOUND [V]:** it is a `soundproperties` in **`lvl/DATA/_LVL_PS2/CORE.LVL`'s `snd_` @0x3800** (BF2; BF1 @0x3000), sample 0xB75D27CD (2,704 B, 11,060 Hz, 0.43 s) in CORE.LVL's `global` SampleBank @0x10. Byte-grep of every extracted LVL: only the two CORE.LVLs. HOLD 9 searched the sound tree only.
7. **Recommendation:** bake **once**, not per map: `assets_baked/audio/global_bf2/` = VO streams re-encoded (195 MB ADPCM → ~683 MB PCM → OGG/Opus est. 40–70 MB; or keep ADPCM 195 MB and decode at runtime) + `global_names.tsv`; keep the 612 MB music as **streamed** (decode from the LVL or bake to OGG ≈ 90–120 MB). Also bake CW.LVL+GCW.LVL (18.4 MB, BF1 VO + tac chatter) and BF2 CORE.LVL's `global` bank (11.5 KB, all shell/load sounds). Per-map banks and GLOBAL are disjoint (5 shared ids, all `shell_auditionVO`).

## §1 Layout and loading model

### 1.1 Chunk layout [V]
`scripts/ucfb_tree.py GLOBAL.LVL --max-depth 2` and `sound_scratch/streams.py`: 31 × (`StreamList` → one `Stream` → `info`(0x0FB40705) + `data`(0xD872E2A5)) + 30 `_pad`. Full table: `sound_scratch/global_streams.tsv`.

| # | name hash | name [V] | ch | segs | bytes | Hz |
|---|---|---|---|---|---|---|
| 1 | FE6EAE40 | global_vo_slow | 1 | 471 | 11,337,728 | 22050–44119 |
| 2 | 0C6A2014 | global_vo_quick | 1 | 1 | 32,768 | 44120 |
| 3–4 | F56DE5DB / C812E73D | cis_unit_vo_slow / _quick | 1 | 670 / 45 | 17,399,808 / 851,968 | 22050–22106 |
| 5–6 | 24188F2D / 8AA6A4FF | rep_unit_vo_slow / _quick | 1 | 779 / 51 | 21,741,568 / 933,888 | 22050–22128 |
| 7–8 | 17D0BF91 / D022593B | all_unit_vo_slow / _quick | 1 | 1564 / 116 | 38,387,712 / 2,048,000 | 22050–22128 (one 617400) |
| 9–10 | 2F3F07CA / 2366685E | imp_unit_vo_slow / _quick | 1 | 1617 / 67 | 44,367,872 / 1,523,712 | 22050–22128 |
| 11–14 | BAA4DB7D 36485014 42A641F5 D533A41E | wok/des/gam/gun_unit_vo_slow | 1 | 51/27/12/13 | 1.18/0.56/0.20/0.36 MB | 22050–22108 |
| 15 | 0E191803 | cw_music | 2 | 82 | 308,412,416 | 44100–48002 |
| 16 | 2AA7C1B4 | gcw_music | 2 | 85 | 321,945,600 | 44100–48002 |
| 17–31 | 118DD7E9 … 0153D31D | {cor,dea,fel,geo,hot,kam,kas,mus,myg,nab,pol,spa1,tan,uta,yav}_objective_vo_slow | 1 | 17–328 | 1.7–14.0 MB (Σ 55.7 MB) | 22050–22086 |

Stream `info` = (u32 field-hash, u32 value) pairs, exactly as `analysis/re-sound-banks.md` §Stream, now confirmed
against the ELF reader: header `name`(8D39BDE6) `format`(=1) `numchannels`(7816084B) 182FD58D(=0x4000 block)
`numsegments`(40FBDEBD) `size`(23A0D95C) 7AAF1A1C(=1) 740FDB0C(=0x4000); then per segment: marker (first B969BE96,
then 2E789FB4; value = hash "sample"), `id`(37386AE0), `frequency`(2FB31C01), `size`, 1D48FEEF(frames = (size+pad)/16·28),
`padding`(809608B6), optional `alias`(7D268157). FUN_005547b0 @687354 reads the header fields (name at 687438, block
0x182fd58d, numsegments 0x40fbdebd, …); FUN_005543e8 @687187 reads the segment fields (0x2fb31c01 freq, 0x37386ae0 id,
0x23a0d95c size, 0x1d48feef, 0x809608b6 pad, 0x7d268157 alias resolved against earlier records, 0x2e789fb4 ends a
record) into 0x24-byte records `{+8 float freq, +0xc size, +0x10 id, +0x14 offset, +0x18 frames, +0x20 pad}`; offsets
accumulate `size+padding` [V]. Σ(size+pad) == header `size` == data payload for all 31 streams [V]. GLOBAL has 0 aliases.

Differences from the per-map format: per-map SOUND/<planet>.LVL = 5 StreamLists + `lvl_` wrappers holding
SampleBank + `snd_`; GLOBAL has no wrappers, no bank, no script. Every property that plays a GLOBAL segment lives in
the **per-map** `snd_` scripts (`soundstreamproperties` with `stream(<hash>)` + `segmentlist{segment(id,w)}`), so a
map's script names both its local samples and its GLOBAL lines.

### 1.2 Who opens it, and how [V]
- Lua: `sound\global.lvl` occurs in 99 luac, all under `extracted/bf2/lua/mission/` (none in shell/common/ingame).
  Constant order in `mission/yav1g_con.luac` (strings 136–147): `OpenAudioStream, sound\global.lvl, all_unit_vo_slow,
  AudioStreamAppendSegments, imp_unit_vo_slow, global_vo_slow, voiceQuick, all_unit_vo_quick, imp_unit_vo_quick,
  gcw_music, sound\yav.lvl, yav1, yav1_emt` — i.e. one slow VO stream (map's two sides + global), one quick VO stream,
  one era music stream, then the planet's own streams [V by constant order; call shapes [D]]. Only the two playing
  sides' streams are opened: a GCW ground map touches ≈ 38.4+44.4+11.3 MB slow + 2.0+1.5 MB quick + 322 MB music.
- ELF `OpenAudioStream` @44961 → FUN_0019eb70 @44870: splits `"%s;%s"` (0x616a88, sub-level syntax), opens the file
  (FUN_003cb070 @425258 = file open), picks a ring buffer of 0x10000 for cw/gcw music (0x0e191803 / 0x2aa7c1b4) and
  0x20000 for shell_music (0xf6885c99), else default; FUN_00549058 @679448 → FUN_005547b0 parses `info` only; on the
  `data` chunk (0xd872e2a5 @687649) it adds the current file position (FUN_003cb650) to every segment's +0x14 offset
  and returns — **no sample data is read at open**. FUN_005490e8 @679463 allocates the streaming buffers from pools
  0x741e40/0x741e10/0x741e00. `AudioStreamAppendSegments` @44991 merges another stream's segment table into slot
  `DAT_0060b658 + n*0x180`, n < 6 [V].
- `ReadDataFile` path: the sound-chunk dispatcher FUN_0053cf00 @671079 handles `snd_`(0x5f646e73), SoundBankList
  (0x5e262256)/SampleBank(0x9a644d15 → FUN_0053e7e0) and **skips StreamList 0x5bb97f21** (0x0053d214) — a stream
  file loaded as data would load nothing [V].
- Resident global sound = **CORE.LVL** (`core.lvl` 0x60fdc8, FUN_00174b28): `global` SampleBank (7 samples, 11,568 B)
  + `snd_` (68 defs incl. 15 buses). BF2 has no resident per-side VO; BF1 uses the same model with SOUND/CW.LVL and
  GCW.LVL (2 streams each: `cw_vo`/`gcw_vo` 314/284 segs 8 MB, `cw_tac_vo`/`gcw_tac_vo` 63/62 segs 1.1 MB) opened
  from mission Lua (`OpenAudioStream("sound\\gcw.lvl","gcw_vo")` + `gcw_tac_vo`, e.g. bf1 `mission/yav1i.luac`) [V].

## §2 Names

Sources hashed (`sound_scratch/cand_all2.txt`, 17,263 strings): all `names.ron` (1,349), printable runs of every
BF1+BF2 luac (11,256), both ELF stringrefs (3,697), every quoted string in `assets_baked/classes/*.ron` split on
whitespace (3,718 — the split is what unlocked the ODF `VOSound` lists, whose values are `"<propname> <Event>
[+modifier…]"`). PC install: no `.snd/.sfx/.asfx` (only munged `_lvl_pc/sound/*.lvl`, `global.lvl` 739 MB); community_mods:
only `ifs_opt_sound.lua`. Derived pass: side-token swap over Direct stems (SIDES + `<side>1`).

| target set | N | named | how |
|---|---|---|---|
| stream names | 31 | 31 | Lua strings (`*_unit_vo_slow/quick`, `*_objective_vo_slow`, `cw/gcw_music`, `global_vo_*`) |
| props referencing GLOBAL segs (BF2 per-map + SHELL `snd_`) | 2,980 | 1,564 (1,266 D + 298 Derived) | table `global_names.tsv` (hash, name, source, stream, segment ids) |
| segment ids | 6,457 | 0 | see below |
| ids referenced by ≥1 prop | 5,498 | — | 5,345 VO ids = 173.5 MB; 21.8 MB of VO is referenced by nothing in any BF2 script (H: cut lines) |
| ids reachable from a *named* prop | 2,795 | — | VO 2,646 ids = 80.4 MB; music 145/162 ids = 572 MB |

Per-stream prop coverage: quick streams 10/10 (rep, cis, imp, all — all five commands + five responses), wok 10/10,
rep_slow 186/234, cis_slow 164/214, imp_slow 156/445, all_slow 138/490, global_vo_slow 42/75, cw_music 252/308,
gcw_music 212/267, objective VO 9–135 per planet (spa1 135/316), des/gun 0.

Segment-id preimage: ids are NOT `<propname>+suffix`: brute force of every 1–3-char suffix over the 1,564 named props
(`suffix_brute2.py`, 16 s) and of prefix×suffix×number dictionaries (`dict_brute.py`, 4m49s; hits were the expected
~2^-32 collisions such as `seg_COR_obj_2583_quick`) found nothing. Family clustering by stripping one trailing
digit (`families.py`) puts 6,255/6,457 ids into 10,860 candidate `<stem><digit>` families, so ids are numbered
takes of an unknown stem (likely the source .wav basenames [H]); none of 17k stems match.

Naming families found (Direct unless noted; see `global_names.tsv`):
- Battle chatter: `<side>1_inf_chatter_<Event>[_Above][_IsSniper][_VOUnitNum<N>][_InHover|_InWalker]`, Event ∈
  {AcquiredTarget, NotShootingCriticalHit, RepairStart, RepairEnd, KillingSpree4/8, HeadshotKill, Grenade,
  MissileIncoming, FriendlyFire, RebelsShootDeadBody, NearbyEnemySlaughter, NearbyFriendlySlaughter} — 312 named,
  on `<side>_unit_vo_slow`; ODF key 388384185 = fnv("VOSound") [V]; consumer FUN_002e5340 reads `VOUnitNum` (0x624e58).
- Squad commands: `<side>_command_{follow,stopFollow,stopVehicle,getIn,getOut}`, `<side>_response_*` (rep/cis Direct;
  imp/all/wok1 Derived) on `<side>_unit_vo_quick`.
- Announcer/officer: `<side>_off_com_report_{us_overwhelmed,enemy_losing}`, `<side>_off_{victory,defeat}_im`,
  `<side>_off_response`, `<side>leaving`, `<side>_<X>Flag_{take,drop,returned,score}_<Y>`, `common_objComplete`.
- Heroes: `hero_<name>_{spawn,exhausted,AcquiredTarget,KillingSpree4}` (global_vo_slow).
- Objective VO: `<PLANET>_obj_NN`, `<PLANET>_inf_NN`, `KAS_Yoda_NN`, `KAM_boba_NN`, `AOSMP_obj_NN` (spa1).
- Music (stereo): `<side>_<planet>_amb_{start,middle,end,victory,defeat,hunt,…}`, `<side>_<planet>_objComplete_NN`,
  `<side>_hero_<name>_lp`, `imp_yav_immVict_01`.
- Shell: `shell_auditionVO` (5 segs), `shell_auditionSFX` — these 6 ids are also segs 0–5 of global_vo_slow and of
  both SHELL.LVL `shell_vo` streams.
- ODF `pc_com` set (`<side>1_inf_pc_com_{transport,mechanic,hostile,defend,clear_area,bacta,backup,ammo}[_inVehicle]`,
  events NeedPickupVO…AttackPositionVO, 32 names): present in ODFs, **absent from every BF2 `snd_`** — unshipped [V].

## §3 Squad chatter, shell sounds, announcer

**Squad-command chatter.** BF1: per-map `soundstreamproperties` `<side>_inf_com_chatter_tac_<cmd>` (+ `ai_` twins,
9 cmds × all/imp/rep/cis/wok) → `stream(gcw_tac_vo|cw_tac_vo)` + 2–3 segments, e.g. TAT.LVL
`all_inf_com_chatter_tac_follow` → stream ACC55D97 segs C045C279, CB45D3CA, CC45D55D (inherit B68DDDF7); all 62
`gcw_tac_vo` segments are referenced from TAT.LVL [V]; ODF keys carrying them: 30 hashed keys in bf1_all.ron:3321–3326
(names not cracked). BF2: the equivalent is the `command/response` set above (5 commands, matching the ELF AI verbs
`getintovehicle, getoutofvehicle, waitforpickup, followchr, stopfollowchr` @stringrefs 841–845 / FUN_001926d8) —
there is no hold/moveout/spreadout/allclear/yes/no/needgunner VO in BF2 [V]. Sizes: rep 0.93 MB, cis 0.85 MB,
imp 1.52 MB, all 2.05 MB (quick streams, incl. responses).

**`shell_menu_accept` 0x6A942216.** In both CORE.LVLs' `snd_` script `global` (0x1DFF06AE):
`soundproperties{ name 6A942216, gain 1.0, pitch 1.1, looping 0, priority 0.5, bus 82C7F467 (= "shellfx", gain 0.4),
samplelist{ sample(B75D27CD, 1.0) } }`; `shell_menu_enter` (25AB7690) inherits it. Bank `global` (BF2 CORE.LVL @0x10,
data @0x800): B62DC1E9 ui_load_lp 720 B/3605 Hz; **B75D27CD 2,704 B/11,060 Hz/0.43 s (accept)**; B21A7DEA 592 B
(shell_select_change, pitch 1.5); C0D52579 896 B (shell_keyboard_key 1.5 / _backspace); EBAA708C 2,352 B
(shell_menu_exit, pitch 0.8); 1C2ECDEE ui_cancel 2,720 B (shell_menu_cancel/_error); D6F2E828 1,584 B/22,210 Hz
(BF2-only; three looping props 77A05D56/53D98429/A09B48E0, gain 0/0/0.5, names uncracked). BF1's bank is the same
minus the 7th. Players: FUN_0017ca60 @22128 (`FUN_0043d230(0x66af50,0x6a942216,1,0)`) and the cheat matcher
FUN_0038c1d8 @386286; FUN_0043d230 @499006 resolves a hash through the soundproperties/layered/stream tables
(FUN_0054d910/FUN_0054e958/FUN_00550b40) — CORE.LVL's script is what satisfies it [V]. `load_transition`'s wave
E502F23B is still in no bank (unchanged from re-sound-banks.md).

**Announcer/HUD lines the port names today** (`*_off_com_report_*`, `*_off_defeat/victory_im`, `*leaving`,
`*Flag_*`, `hero_*_spawn`, `*_amb_*`): all are GLOBAL streams in BF2 (or CW/GCW streams in BF1) — none of them has a
local sample. Every `AmbientMusic`/`VictoryMusic`/`DefeatMusic` name in mission Lua resolves to a `gcw_music`/`cw_music`
segment (85/82 segments, 5–6.5 MB each ≈ 60–75 s stereo).

## §4 Per-map vs global split

- Baked per-map sample ids (32 maps, 1,554 unique) ∩ GLOBAL ids = **5** (E4466DF0…EB4678F5 = `shell_auditionVO`,
  which BF1 also stores as bank samples in bes/end/hot/kas). Otherwise **disjoint**: nothing baked today duplicates
  GLOBAL, and nothing in GLOBAL is a bank sample [V].
- Of the names the port resolved per BF2 map, roughly **45 % are GLOBAL-streamed properties** the bake silently leaves
  empty (`samples: []`): yav1 143/321, cor1 145/300, tat2 132/292, myg1 151/341, end1 73/198 (full list in
  §Reproduction output). BF1: 150/408 (tat1) of the resolved names stream from GCW/CW.LVL, 54 of them tac chatter.
- Per-map SOUND/<planet>.LVL streams (map music `yav1`, ambience `yav1_emt`, per-map VO `gal_vo`) are likewise unbaked.

## §5 Recommendation (bake)

Bake once → `assets_baked/audio/global_bf2/`: (a) walk GLOBAL.LVL's 31 streams with the existing pair parser
(add `alias` + first-marker B969BE96 handling; `find_banks_deep` is not applicable — add `find_streams`), (b) decode
VO segments referenced by any BF2 `snd_` (5,345 ids, 173.5 MB ADPCM → ≈ 607 MB s16 PCM) and encode to OGG/Opus
(est. 40–70 MB at speech quality; keep native rate 22 kHz mono); emit `global.ron` = {stream name, segment id →
file, freq, frames} and ship `global_names.tsv` as the readable index; (c) treat `cw_music`/`gcw_music` (612 MB
ADPCM, 162 segments) as **streamed** assets: either decode on demand from the LVL (offset table above, 16 KiB blocks)
or bake to OGG ≈ 90–120 MB — do not put them in the per-map dirs. (d) Same walker over BF1 SOUND/CW.LVL + GCW.LVL
(18.4 MB → all BF1 VO incl. the 9 tactical commands; 125 segments) and over BF2 `lvl/CORE.LVL` (11.5 KB bank +
`snd_` → every shell/load sound incl. `shell_menu_accept`). Then extend `bake sounds` so `soundstreamproperties`
carry `(stream, segments)` and `game::sounds` can resolve `all_off_com_report_us_overwhelmed` etc. to the global
files. Universal chatter per the maintainer's rule: BF1's `*_inf_com_chatter_tac_*` set (from GCW/CW.LVL) stays the universal
9-command set; BF2's global bank only has the 5-command `command/response` set, so "replace with BF2's" cannot be
literal — BF2 lines can only replace follow/stop/get-in/get-out.

## §Reproduction (all from repo root; scratch = engine/artifacts/lead_tmp/s12/re/sound_scratch)

```
ls -laS extracted/bf2/sound/DATA/_LVL_PS2/SOUND/            # GLOBAL.LVL 827344896
python3 scripts/ucfb_tree.py extracted/bf2/sound/DATA/_LVL_PS2/SOUND/GLOBAL.LVL --max-depth 2 --max-lines 120
cd engine/artifacts/lead_tmp/s12/re/sound_scratch
python3 streams.py ../../../../../../extracted/bf2/sound/DATA/_LVL_PS2/SOUND/GLOBAL.LVL   # -> GLOBAL.summary.txt, GLOBAL.LVL.streams.json (31 streams, 6523 recs)
python3 streams.py <bf1 SOUND/CW.LVL|GCW.LVL>; python3 streams.py <bf2 SOUND/SHELL.LVL>; python3 streams.py <bf2|bf1 lvl/CORE.LVL>
for f in bf2 SOUND/*.LVL (not GLOBAL) and bf1 SOUND/*.LVL: python3 snd_parse.py $f bf{1,2}_<PLANET>.snd.json
python3 snd_parse.py extracted/bf2/lvl/DATA/_LVL_PS2/CORE.LVL bf2_CORE.snd.json     # shell_menu_accept = name 1788092950
grep -a -r -o -i -E "sound[\\/][a-z0-9_.;]+" extracted/bf2/lua | awk -F: '{print $2}' | sort | uniq -c | sort -rn  # 99× sound\global.lvl
strings -n 3 extracted/bf2/lua/mission/yav1g_con.luac | sed -n 130,150p
cd extracted && for f in $(find bf1 bf2 -iname '*.lvl' ! -name GLOBAL.LVL); do n=$(LC_ALL=C grep -c -a -F $'\x16\x22\x94\x6a' "$f"); [ "$n" != 0 ] && echo "$n $f"; done   # 3 CORE.LVL hits
python3 join.py            # prop->segment join, per-stream coverage (needs cand_all2.txt; see harvest cmds in shell history: cand_lua/cand_elf/cand_classes_tok)
python3 families.py; python3 suffix_brute2.py; python3 dict_brute.py   # negative preimage results
sed -n 44870,45060p analysis/SWBFRNT2.ELF-decomp.c   # FUN_0019eb70, OpenAudioStream, AudioStreamAppendSegments
sed -n 687354,687660p analysis/SWBFRNT2.ELF-decomp.c # FUN_005547b0 info reader; data-chunk offset fixup at 687649
sed -n 687187,687353p analysis/SWBFRNT2.ELF-decomp.c # FUN_005543e8 segment-record reader
sed -n 671079,671205p analysis/SWBFRNT2.ELF-decomp.c # FUN_0053cf00 sound-chunk dispatcher (StreamList skipped)
grep -n "0x6a942216" analysis/SWBFRNT2.ELF-decomp.c  # 22134 (FUN_0017ca60), 386286 (FUN_0038c1d8)
```
Outputs kept: `global_streams.tsv` (31 rows), `global_names.tsv` (2,980 rows: hash, name, Direct/Derived/?, stream,
n, segment ids), `seg_to_props.json`, `propnames.json`, `*.snd.json`, `*.streams.json`, `GLOBAL.summary.txt`.

## §Not closed
- Segment-id names (0/6,457) and the 1,416 unnamed props (mostly `all/imp_unit_vo_slow` chatter variants and
  `spa1_objective` lines); the stems are probably source-wav basenames — needs the BF2 mod-tools `.stm/.asfx` lists,
  not on any disc here [H].
- Ring-buffer / block streaming details beyond "16 KiB blocks from the kept file handle" (FUN_005490e8 pool sizes
  0x741e40/0x741e10) not traced; the 0x10000/0x20000 values are per-stream buffer sizes [D].
- BF1 ODF key names for the tac-chatter properties (30 hashed keys) uncracked; the `ai_` twins' trigger path untraced.
- Runtime confirmation (emulator) not done; every claim is from bytes, Lua constants and the decomp.
- The 617,400 Hz frequency on one shared segment (all/wok_unit_vo_slow) — looks like an authoring error; unverified.
