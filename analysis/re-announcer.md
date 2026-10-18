# OPUS RE (2026-09-30) - [O] UNAUDITED

> **AUDIT 2026-10-02 (loop lane 3, PARTLY — `engine/artifacts/loop/audits.md`, report
> `engine/artifacts/loop/reports/20261002_0135_audit_ANNC.md`, evidence `engine/artifacts/lead_tmp/audit-annc/`).**
> Audited with the landing ANNC (feb04a8). Re-derived before this doc was read; one Opus verifier, 13 / 5 / 0 of 18,
> corrections folded in.
> - **Held.** §2.0-2.3 on both ELFs: the post table from both setters (40 of the 80 `VO_<L>_<S><Event>` names exist
>   in each game, one linear layout), the bleeding start and the own-team repeat, the single low-reinforcement latch.
>   §3's mission lines against the bytecode with independent Lua 4.0 / 5.0 readers (57 RONs, 0 differ). The posts'
>   tables against a raw scan of the world LVLs (2,932 rows). §4 rows 7-14, re-read from the raw log.
> - **Corrected in §0.** T2: `VO_<L>_<enemy>Lost` IS set — 18 properties on the discs, 10 of them on capture posts
>   (BF1 BES2 cp5, GEO1 cp6, YAV1 temple). T8: `common_objComplete` is not a conquest line.
> - **Missing from §2.2 and §6 item 4 (card ANNC2).** On BF2 "bleeding" is the rate the script last handed to
>   `SetBleedRate`, and the script's own bleed timer zeroes it at its first elapse (3.0 s) that finds alive >=
>   reinforcements (`objectiveconquest.luac` function 7.3.0); only the next post event sets it again (7.0, called
>   from 7.2 at Start and at every finished capture / neutralisation / post kill / post respawn). So in a team's last
>   stand every post event starts a new 3 s bleed and the line is said again. Live, BF2 UTA1 with one team's tickets
>   held at 3: 5 post events, 5 lines, the rate back at 0 after 3.003 s (4 of 4). The port feeds the announcer its
>   own per-tick clamped rate and is silent there: 77 lines missing in 23 of 31 BF2 battles, none extra. BF1's engine
>   gates its rate on alive < tickets every update (B1 238700-238702), so BF1 maps are unaffected.
> - **Built (card ANNC2, 2026-10-05).** `core::announcer::ScriptBleed` is the script's rate (set at Start and at every
>   owner change to the unclamped rule, zeroed at a timer elapse with alive >= reinforcements) and feeds the manager on
>   BF2 maps; BF1 maps keep the sim's per-tick clamped rate. Over 31 BF2 missions (seed 42) the port asks 205 bleeding
>   lines where the probe's own model of the PS2 rule asks 205 (the old feed on the same sims: 135, 70 missing in 21
>   battles). The mission's lines are keyed by the listener's FACTION, not the script's team number
>   (`game::announce::battle_lines`): the PS2 cannot seat an army in the other team's slot, the port's shell can
>   [ours]. Post kill / respawn events are still not modelled (no destroyable posts).
> - **Audit of ANNC2 (2026-10-05, PARTLY — `engine/artifacts/loop/audits.md`, report
>   `engine/artifacts/loop/reports/20261005_1150_audit_ANNC2.md`, evidence `engine/artifacts/lead_tmp/audit-annc2/`;
>   one Opus verifier, 16 / 2 / 0 of 18).** The built rule holds: bytecode 7.0 / 7.3.0 beside the mod-tools source,
>   `SetBleedRate` -> team+0x2c (B2 367639) -> the manager's latch (371155), every capturable BF2 conquest post worth
>   10 to all four armies, the sim's posts = the scripts' lists in 31 of 31 missions. The census's "205 = 205" sets the
>   port beside a second copy of its own rule: it shows the wiring, not the PS2; the PS2 side is the one captured
>   scene. Two things this doc did not say:
>   1. **On BF1 a line's NAME does not fix its audio.** A planet's SOUND LVL defines the post lines once per
>      sub-level under one name, each with its era's stream: `imp_off_com_report_lost_commandpost` is gcw_vo segment
>      1c18b3b0 in `kas1gcw` / `kas2gcw` and cw_vo segment bdf175ac in `kas1cw` / `kas2cw` **[V]** (raw bytes of
>      KAS.LVL); the PS2 holds the one sub-level its mission names (`sound\kas.lvl;kas2gcw`) **[V]**, and in 160 of
>      160 BF1 battle save states the six post-line names sit beside their own era's stream **[L]**. The port keeps
>      one definition per name for the whole LVL, the last sub-level's (`game::sounds::merge_props`): in a GCW
>      battle on a two-era BF1 planet the `all_` / `imp_` post lines play Clone Wars takes (11 of the 18 commander
>      lines of a default Kashyyyk Docks round). Card SNDE. BF2's commander lines sit in per-army streams and are
>      clean.
>   2. **§3's lines are per era, and the shell starts one era.** 0 of the 21 BF1 Clone Wars scripts set a
>      low-reinforcement line (1 in each of the 23 GCW ones); BF2's `fel1c_con` / `kas2c_con` set none and
>      `yav1c_con` sets no commander line at all; `kam1c_con` / `mus1c_con` swap the CIS listener's two low lines.
>      The shell starts `<map>_con.ron` for every pairing, so Republic against CIS on a two-era map hears the GCW
>      script's roles with the Republic's prefix. Card ERAM.
> - **Not re-derived.** §1's queue code (only its data rows), §2.4 (card BNDY), §2.6-2.7. §1.6 does not say that
>   BF2's `allleaving` (PlayInterval 10) and `repleaving` (takes `(id, 1, 1, 0)`) carry locks.

# RE: the commander's voice-overs (BF1 + BF2) — triggers, lines, the voice queue

Card ANNC (loop lane 2, max effort). Corpora: `analysis/SWBFRONT.ELF-decomp.c` (B1), `analysis/SWBFRNT2.ELF-decomp.c`
(B2); the mission bytecode (`extracted/bf{1,2}/lua/mission/*.luac`); the world and sound LVLs; and two live PCSX2
sessions (BF1 NAB1 Clone Wars, BF2 UTA1 Clone Wars) read through `/dev/shm`. Tags: [V] read (lines cited, each found
INSIDE the named function by `lead_tmp/annc/cite.py`), [L] measured live on the PS2, [D] derived, [H] hypothesis.
`[O]` marks the load-bearing claims. Scratch, tools and logs: `engine/artifacts/lead_tmp/annc/`.

## 0. Headline

The PS2 has no "announcer" object. A commander line is an ordinary 2-D `soundstreamproperties` on the battle's voice
stream, and five places ask for one. Each call goes through the sound handle's play function with the QUEUE flag set,
so a line waits for the voice stream to be free and never cuts another line off.

| # | trigger (both games unless marked) | who hears which line | rule | evidence |
|---|---|---|---|---|
| T1 | a command post is **captured** (neutral -> team S) | the listener L: the post's `VO_<L>_<S>Capture` | at the capture tick | [V][L][O] §2.1 |
| T2 | a post is **neutralised** (owner S -> neutral) | `VO_<L>_<S>Lost` — the generic posts set it only for L == S; 10 entries on three BF1 capture posts (BES2 cp5, GEO1 cp6, YAV1 temple) set it for the enemy too (audit 2026-10-02) | at the tick | [V][L][O] §2.1 |
| T3 | a neutralisation starts **contested** / is **saved** | `VO_<L>_<S>InDispute` / `...Saved` — no retail post sets either (silent) | first contested frame / recovery | [V][O] §2.1 |
| T4 | a team starts to **bleed** | L == S: `<L>_off_com_report_us_overwhelmed`; else `<L>_off_com_report_enemy_losing` | once at the start; **repeats every 30 s only for the listener's own team** | [V][L][O] §2.2 |
| T5 | a team's reinforcements fall to **10 % of its start count** | L == S: `<L>_off_defeat_im`; else `<L>_off_victory_im` | **once per battle** (one latch for both teams) | [V][L][O] §2.3 |
| T6 | the player **leaves the battlefield** | his team's `<side>leaving` | BF2: when the HUD warning appears, 6.5 s before death; BF1: at the start of a 10 s countdown | [V][O] §2.4 |
| T7 | BF1 only: **team 1** loses its last post (no neutral post left) | the T5 line again (no latch) | first frame of the 20 s defeat countdown | [V][O] §2.3 |
| T8 | BF2 only: script `BroadcastVoiceOver(name[, team])` | the local player, if his team's bit is in the mask | NOT in conquest: `Objective.Complete` broadcasts `common_objComplete` only `if not self.multiplayerRules` (function 9, pc 32-38) and every `*_con` script sets that true (audit 2026-10-02; this row said "conquest uses it once") | [V] engine, [V] script §2.5 |
| T9 | battle over | no voice line: the listener's victory or defeat MUSIC; new stream lines are refused | — | [V][L] §2.5 |

Not commander lines, recorded for their cards: BF2 hero lines (§2.6, card HERO), BF1 Galactic Conquest bonus and
reserves lines (§2.7, out of the port's scope), the squad/tactical lines (card SQACK) and unit chatter (card USND) that
share the voice stream.

**Found on the way, and it corrects a landed claim: BF1's conquest bleed is LIVE on the PS2** (§5).
`analysis/re-tickets-spawn.md` §2.4 said the engine bleed "is provably inert with retail data"; it missed a `qsort`.

## 1. The voice stream and its queue [O]

Everything below is the path of `FUN_00354fe0(handle, pos, vel, 0, 1)` (B1) / `FUN_0043ceb0(handle, pos, vel, 0, 1)`
(B2), the call every trigger makes. The last argument is the queue flag.

1. **Gate** [V]. B1 `FUN_00354ee8` (342640-342646) / B2 `FUN_0043cd88` (498808-498816): no sound when the sound
   system is off; and **a stream-kind handle (kind 3) is refused while the game-over flag is set** (B1 `DAT_00519084`,
   B2 `DAT_00600b3c`; set by the battle-end code, B1 writers 328420-328726 + cleared 328143/328880).
2. **Kind 3 = stream properties** -> B1 `FUN_0045ec68(props, emitter, queue)` (342738/342751 -> 522509-522554), B2
   `FUN_005519c8` (498913/498926 -> 685320-685390): find the stream SLOT that has the property's `Stream` open
   (`FUN_00449480(props+0x80)` / `FUN_0053d348(props+0x88)`), pick the take (`FUN_0046b290` / `FUN_0055bf60` on the
   segment list), compute the line's key (below); key <= 0 -> nothing. With the queue flag: **enqueue** on that slot
   (`FUN_0045d638` 522554 / `FUN_00550528` 685390). Without it (no trigger here does that): start at once — B1 always
   (a playing slot is restarted on the new line, 522241-522242); B2 only if the slot is idle, has no line recorded,
   or the playing line's key <= the new key (685337-685345).
3. **The queue** [V] (B1 `FUN_0045d638` 521586-521670; B2 `FUN_00550528` 684452-684536, the same code):
   - every pending entry of the slot is re-keyed; an entry whose key is now <= 0 is dropped (521586-521589);
   - the new entry is inserted **in descending key order, behind equal keys** — the per-slot "sorted" bit is set for
     all six slots at init (B1 `FUN_0045d468` 521460; B2 `FUN_00550340` 684325);
   - **at most 4 pending** (`sltiu v0,v0,5` at 0x45d8e4 / 0x5507d4; 521650 / 684516): the 5th drops the tail (lowest
     key, newest among equals). If the tail IS the new entry: a 2-D line (Mode3D 0) takes the place of the entry before
     it; a 3-D line is dropped itself (521661-521670).
   - 24 queue nodes in all (521517).
4. **Playback** [V]: once per frame, for each of the 6 slots: if the slot's voice state is 4 (stopped) and its queue is
   not empty, pop the head and start it (B1 `FUN_0045eec8` 522610-522612; B2 `FUN_00551d08` 685446-685449). So lines
   on one slot play strictly one after another.
5. **The key** [V]: `key = level * (1 - Bias) + Bias * Priority` (B1 `FUN_0044e4e8` 511369, called with 0 as its
   first argument so the trailing factor is 1), `level = Gain * busGain` for a 2-D property; for a Mode3D one the
   emitter's distance level takes the Gain's place (B1 `FUN_0044e570` 511387-511401, `FUN_0045e670` 522286-522302).
   `Priority` = property hash 0x94e4e309 -> props+0, `Bias` = 0xba467ec4 -> props+4 (B1 `FUN_0044e648`
   511413-511420); defaults Priority 1.0, Bias 0.5 (B1 `FUN_0045e768` 522343-522344; `lui 0x3f80` / `lui 0x3f00` at
   0x45e840 / 0x45e848).
6. **The data** [V] (`lead_tmp/annc/sndq.py`, from the SOUND LVLs' `snd_` scripts and CORE.LVL): every commander line
   inherits template 0xC01B9EF5 (CORE.LVL): Gain 1, bus `voiceover` (0x3B13AA4B), Mode3D 0, Looping 0 — and in BF2
   also `Bias 1, Priority 1`. Unit chatter inherits 0xCFFF9324: Gain 0.6, Mode3D 1, MinDistance 16, MaxDistance 150,
   **Bias 0, Priority 0.6** (bus soundfx in BF1); BF2's CORE.LVL has two more chatter templates (0x3E2F5FE1: Bias
   0.7, Priority 0.8; 0xFA6489AD: Bias 0, Priority 0.6).
   [D] A BF2 commander line's key is exactly 1.0 (Bias 1: the key IS the Priority); BF1's is 0.5 + 0.5 x the
   voiceover bus gain. A chatter line (Bias 0) has key = its 3-D distance level x bus gain [H: at most 1]. So a
   commander line is never queued behind a chatter line asked for later, and [L] in 4 of 4 BF1 cases it played NEXT,
   ahead of 3-4 pending chatter lines (§4). Between commander lines the order is first come, first played.
7. **Slots** [V][L]: a slot = one `OpenAudioStream` (+ its `AudioStreamAppendSegments`). BF2 conquest scripts open one
   slow voice slot holding both sides' `<side>_unit_vo_slow` + `global_vo_slow`, and one quick slot (`*_unit_vo_quick`
   + `global_vo_quick`); BF1 opens `gcw_vo` or `cw_vo` (commander + chatter) and `gcw_tac_vo` / `cw_tac_vo`. Live:
   BF1 NAB1 slot 0 music, slot 1 `cw_vo`; BF2 UTA1 slot 0 voice slow, slot 1 voice quick, slot 2 music.
   **All commander lines and all unit chatter of a battle share ONE slot** (BF2 `common_objComplete` alone is on the
   quick slot).

## 2. The triggers

### 2.0 The manager, the listener, the gates [O]

B1 `DAT_005644a8` (class name string "16VoiceOverManager", ctor `FUN_002b0ef8`), B2 `DAT_00661360` (ctor
`FUN_003737c8`). Its per-frame update is vtable slot +0xc: B1 0x53f14c -> `FUN_002b1320`, B2 0x62d26c ->
`FUN_00373b80` (`re_check.py word`). The same object also owns the ambient music (not this card).

| field | B1 | B2 |
|---|---|---|
| bleed timer [2] (float) | +0x18 | +0x18 |
| bleed repeat time (float, ctor 30.0) | +0x20 | +0x20 |
| handles A[listener][team] (SetBleedingVoiceOver, 4th arg != 0) | +0x24 + L*0x10 + T*8 | same |
| handles B[listener][team] (4th arg == 0; never used by a retail script) | +0x44 + ... | same |
| handles C[listener][team] (SetLowReinforcementsVoiceOver) | +0x64 + ... | same |
| low-reinforcement threshold [2] (int, ctor 20) | +0x84 | +0x84 |
| low-reinforcement latch | +0x8c (int) | +0x8c (byte) |
| bleeding latch [2] | +0x90 (int) | +0x8d (byte) |
| attacking team index (SetAttackingTeam) | +0xfc | +0xf4 |
| enabled | +0x100 (int) | +0xf8 (byte) |

- **Listener team index** [V] (B1 `FUN_002b1720` 241416-241425; B2 `FUN_00373fa0` 371304-371313): 0 in the demo
  (`DAT_0051905c` / `DAT_00600b15`, written only by `ScriptCB_LaunchDemo`); else local player 0's team - 1 (1 if
  there is no player 0); if a second local player is on the OTHER team, the attacking team index instead
  (`SetAttackingTeam`, B1 32463 / B2 45494). One listener per machine, not per split-screen view.
- **Enabled** [V]: 0 after construction in an offline game (ctor 241135-241140 / 371027-371032 set it only under the
  `DAT_00518be4/be8` / `DAT_00600548/549` session flags [H: a network session]); set to 1 by the unit-spawn function
  when a human-controlled member spawns: B1 `FUN_002966f0` (221659-221675) — the single local player himself, or a
  human whose team - 1 is the attacking team index; B2 `FUN_0034e1f0` (345188-345192) — the single local player
  himself, or any human once there are two local players. Never cleared.
  **Nothing in §2.1-2.3 plays before the player's first spawn.**
- **Game over** [V]: the update, the CP line function and both play helpers return while the game-over flag is set
  (B1 241252, 227789, 241484; B2 371146-371147, 351345, 371332).

### 2.1 Command-post lines (T1-T3) [O]

- The post's tick enqueues event 4 (captured; team = the NEW owner), 5 (neutralised; team = the OLD owner), 6 (first
  contested frame of a neutralisation; team = owner), 7 (saved; team = owner): B1 `FUN_0029f0d8` 228351 / 228552 /
  228524 / 228717; B2 `FUN_00357440` 352126 / 352725 / 352691 / 352609. Offline the event is consumed in the same
  call (B1 `FUN_00307940` 291968; B2 `FUN_003e0218` 438862). The consumer maps 4->entry 0, 5->1, 6->2, 7->3 with the
  subject = the event team's faction (B1 `FUN_00307a00` 292055 / 292078 / 292089; B2 `FUN_003e0320` 438955 /
  438965). Entry 4 is asked for by the tick itself, every frame, for the owner's faction while the post is alive
  (B1 228756; B2 352886).
- The line: B1 `FUN_0029edd0(cp, subject, entry, credited)`: handle = `cp + 0xd4 + L*0xf0 + S*0x28 + entry*8`, L =
  the faction (`team+0x64`) of the listener's team (227798-227799), played queued (227842); returns first when game
  over (227789) or the manager is not enabled (227794). B2 `FUN_003570b8`: `cp + 0x120 + L*0x140 + S*0x28 + entry*8`
  (351356), L = `team+0x60`, **only if `DAT_005fed28` (ScriptCB_EnableCommandPostVO) and the manager's enabled flag**
  (351355). Entry 0 / 1 also play the class's CapturedSound / LostSound at the post and, when a player is credited,
  the faction's capture / lost music (B1 227873-227915; B2 351431-351475) — not this card.
- **Names** [V]: the instance property `VO_<L>_<S><Event>`, Event = `Capture`, `Lost`, `InDispute`, `Saved`, `Info`
  -> entries 0-4; factions 1 All, 2 Imp, 3 Rep, 4 CIS. `VO_All_AllInDispute` = 0x22bd3453 (the entry-2 name
  `analysis/re-cp-capture.md` §9.2 left uncracked). [L] the table of a live BF1 NAB1 post (cp[1]) read back from
  RAM holds exactly the world file's twelve properties at [L][S][entry].
- **`ScriptCB_EnableCommandPostVO`** [V]: `DAT_005fed28`, default 1 (`FUN_00174b28` 16806), written only by the
  binding (45917), read only by `FUN_003570b8` (`re_check.py absent/writers`). 17 BF2 scripts call it with 0 (18
  calls) — all campaign (`*_c`) scripts; no `_con` script and no BF1 script does (bytecode census +
  `/usr/bin/grep -a`).
- **Retail data** [V] (`lead_tmp/annc/cpvo.py`, every `inst` of every world LVL): BF1 129 of 132 posts, BF2 416 of
  451 carry VO properties. The generic set is 12 properties, 3 per listener faction:

  | property | BF1 line | BF2 line |
  |---|---|---|
  | `VO_<L>_<L>Capture` (we took it) | `<L>_off_com_report_captured_commandpost` (CIS: `cis_off_com_chatter_goodjob2`) | `<L>_off_com_report_captured_commandpost` |
  | `VO_<L>_<L>Lost` (ours was neutralised) | `<L>_off_com_report_lost_commandpost3` | `<L>_off_com_report_lost_commandpost` |
  | `VO_<L>_<enemy>Capture` (they took one) | `<L>_off_com_report_lost_commandpost` | `<L>_off_com_report_enemyCaptured_commandpost` |

  with <enemy> = the other faction of the era (All<->Imp, Rep<->CIS). **No generic `VO_<L>_<enemy>Lost`: nothing is
  said when the enemy's post is neutralised.** `InDispute` and `Saved` are set on no post in either game; `Info` by
  17 BF1 properties (END1 2, GEO1 11, HOT1 4) and 4 BF2 properties (HOT1). Named posts carry their own lines (BF1
  BES2 carbon freezing, YAV1 temple, GEO1 spire / techno union, HOT1 shield generator, END1 shield bunker, KAM1
  cloning facility; BF2 HOT1 shield generator; 14 BF2 posts keep BF1's names) — full list
  `lead_tmp/annc/cpvo_bf{1,2}.txt`.

### 2.2 Bleeding (T4) [O]

- Setter: `SetBleedingVoiceOver(L, T, sound, flag)` -> table A (flag != 0) or B (flag == 0) at [L-1][T-1] (B1 32204
  -> `FUN_002b1600` 241357 / 241360; B2 45206 -> `FUN_00373e80` 371254 / 371257). Every retail call passes 1 (408 of
  408 calls in the bytecode census), so B is empty.
- Update, per team i in (0, 1), every frame while enabled (B1 `FUN_002b1320` 241257-241274; B2 `FUN_00373b80`
  371155-371174): bleed rate (B1 `team+0x1c`, B2 `team+0x2c`) <= 0 -> if the latch is set: clear it, timer = 0. Rate
  > 0 -> `timer += dt`; latch clear -> set it, **STARTED**, timer = 0; else `timer > repeat` -> **REPEAT**, timer = 0.
- A line is asked for when `STARTED(i)`, or `REPEAT(i)` **and i == the listener's team** (241287-241296;
  371183-371192).
- Which: listener L; if `L == i` or the listener's own team is not bleeding -> A[L][i]; else B[L][i] (empty) (B1
  `FUN_002b1848` 241484-241492; B2 `FUN_00374038` 371332-371340). So "enemy losing" is said once, when the enemy's
  bleed starts, and not at all if the listener's own team is bleeding at that moment; "we are overwhelmed" is said at
  the start and then every `repeat` seconds while the bleed lasts. Stopping says nothing.
- `repeat` = 30.0 (ctor: B1 241134, `lui 0x41f0` at 0x2b10b8; B2 371026, 0x37391c; [L] 30.0 in RAM in both games).
  `ScriptCB_SetBleedingRepeatTime` can change it (B1 32219, B2 45221); **no script on either disc calls it** (0 of
  every `.luac` under `extracted/bf{1,2}/lua`, `/usr/bin/grep -rlaF`; the same search finds the VO setters in 44 +
  61 files).
- What "bleeding" is: BF2 — the rate the conquest script hands to `SetBleedRate` (1/3 when the enemy holds a strict
  majority of the posts' bleed value, `analysis/re-tickets-spawn.md` §1.3); BF1 — the engine's threshold rate (§5).

### 2.3 Low reinforcements (T5, T7) [O]

- Setter: `SetLowReinforcementsVoiceOver(L, T, sound, x, flag)` -> C[L-1][T-1], and threshold[L-1] = `(int)(x *
  tickets of team L at the call)` when a 5th argument is given and non-zero, else `(int)x` (B1 32250-32254 ->
  `FUN_002b1668` 241371-241372; B2 45252-45256 -> `FUN_00373ef0` 371268-371269). Retail: x = 0.1, flag 1, after the
  reinforcement count is set -> 10 % of the start count ([L] 20 of 200 on BF1 NAB1, 15 of 150 on BF2 UTA1).
- Rule (B1 241299-241301; B2 371195-371197): for team i = 0 then 1: `if latch == 0 and tickets(i) <= threshold[i]`
  -> play C[listener][i] (B1 `FUN_002b1918` 241510; B2 `FUN_00374108` 371358), `latch = 1`. **One latch: the first
  team to reach its threshold speaks, the other never does.**
- T7, BF1 only [V]: offline, the team update `FUN_002ad830` calls `FUN_002b1918(mgr, 0)` on the first frame of the
  20 s no-post countdown (the team's post value is 0 and the countdown still reads 20.0), and only for team index 1
  (238807-238824; `slti v0,a1,2` at 0x2adc9c). No latch. BF2 has no such call (`FUN_00374108` has one caller).
- Data [V]: **none of BF1's 21 Clone Wars scripts calls the setter** — no "defeat / victory imminent" in a BF1 CW
  battle ([L]: the latch flipped at 20 tickets on NAB1 CW and no line played). BF2: `fel1c_con`, `kas2c_con` and
  `yav1c_con` have none; `kam1c_con` and `mus1c_con` hand CIS the two lines swapped.

### 2.4 Leaving the battlefield (T6) [O]

- Setter: `SetOutOfBoundsVoiceOver(team, sound)` -> boundary object + 4 + (team-1)*8 (B1 32857 -> `FUN_00295788`;
  B2 45981 -> `FUN_0034c600`). Names: `allleaving`, `impleaving`, `repleaving`, `cisleaving`.
- BF1 `FUN_00297e60(dt, member)` (222656-222701): the member's entities that are inside the boundary
  (`FUN_002956d8` == 0) are ignored; none outside -> `timer(member+0xe4) = -1`. Otherwise: `timer < 0` -> `timer =
  10.0` (`lui 0x4120` at 0x297f44) and, if the member is a local human (`FUN_002ff158(member+0x9c) >= 0`), **the
  line** (`FUN_00295750` 220554); `timer -= dt`; `< 0` -> the outside entities are killed.
- BF2 `FUN_00350660` (346687-346735): the same shape; `timer(+0x184) = 6.5` s, or 2.1667 s when the unit's vehicle
  class has bit 1 of `class+0x1208` [H: a flyer]; kill at 0. The line is played by the HUD's "leaving the
  battlefield" element (`FUN_00391fe0` 389580-389610): when the timer is >= 0 and the element is not yet shown it
  shows itself, restarts its alpha pulse and plays the team's line — once per excursion. (A soldier flag, bit 7 of
  soldier+0x660, keeps the unit counted as outside and the warning up regardless of the timer — B2 346692,
  389591-389593; what sets it was not traced [H].)
- HUD text key `game.tooltips.leaving_battlefield` (B1 string 0x540ab0 in `FUN_002d88a8`; B2 0x62ec30 in
  `FUN_00391ed8`) — card EVMSG.
- Data: BF1 `rhn1r`, `rhn1r_h`, `nab2c_h`, `yav2i`, `yav2i_h` set no line; every BF2 `_con` script sets both.

### 2.5 Battle end (T8, T9)

- BF1 [V]: `FUN_0033f718` (328724-328746) -> `FUN_002be838`: mutes the soundfx bus, stops every sound
  (`FUN_00356000`), plays the listener team's music handle: the DefeatMusic slot when the listener team's bit is in
  the mask `FUN_002acae0` returns (the teams with no reinforcements left), else the VictoryMusic slot
  (249769-249776); the call waits out a 2 s countdown after a team is out. [L] BF1 NAB1: game-over flag at t,
  `cis_nab_amb_defeat` on the music slot one frame later; the voice slot went on playing the chatter already queued.
  No voice handle is played there [H: an absence read from one function, not a search].
- BF2 [V]: `BroadcastVoiceOver(name[, team])` (34653-34655) -> event 9 -> `FUN_003e2708` (440562) -> `FUN_00374170`
  (371374-371382): resolve the name; play it queued if `mask >> (local player 0's team) & 1` (mask = -1 with no team
  argument). BF1 has no such binding (`analysis/bf1-bindings.txt`: 0 hits). [D] In conquest the only caller is the
  shared `Objective` script: `BroadcastVoiceOver("common_objComplete", winningTeam)` (bytecode census; the script's
  condition was not decompiled). `common_objComplete`: stream `global_vo_quick`, Gain 0.8, one 0.96 s take.

### 2.6 Heroes (BF2) — for card HERO [D]/[H]

`ScriptCB_EnableHeroVO` / `EnableHeroMusic` -> `DAT_005fd083` / `DAT_005fd082`, default 1, each read only in the
soldier spawn function `FUN_002360c8` (150299-150309): hero music handle on spawn, and a class sound at class+0x1274
(property 0x99257468 — in no shipped class [H]). The hero classes' four line properties (class+0x1254 / +0x1258 =
0xe4dee4ee `hero_<n>_spawn` / +0x125c = 0xd3a93472 and +0x1260 = 0xef9e29f7 `hero_<n>_exhausted`; parser
`FUN_00252bb0` 170821-171034) are resolved per team by the hero HUD (`FUN_00383208` 381077-381083) and played by the
hero manager `FUN_003e6cf8` through `FUN_00382580` / `FUN_003825c8` (spawn line at the hero + a HUD message, 380603)
/ `FUN_00382880` (exhausted, 380684). When each fires was not traced [H]. Only `tat2g_eli` switches them off.

### 2.7 BF1 Galactic Conquest bonus and reserves lines — out of scope [V]

`SetPlanetaryBonusVoiceOver` / `SetReservesVO` fill `DAT_00518350`; `FUN_002a4650` plays one bonus line at a time when
the voice slot is idle, and the reserves line when the reserves bonus tops a team up. BF2's two bindings are empty
stubs (45172-45188). The port has no Galactic Conquest.

## 3. Lines per faction and era [V]

Per mission, from the PS2 bytecode (`lead_tmp/annc/luacalls.py` -> `bf1_calls.txt`, `bf2_calls.txt`): 4 bleeding,
4 low-reinforcement and 2 out-of-bounds calls per conquest script, with the names of §0 for the script's two
factions. Exceptions are listed in §2.3 / §2.4, plus BF1's three historical CW scripts using `cis_..._generic` names.
Team numbers differ per script (BF2 CW: CIS is team 1 in cor1c, kam1c, nab2c, pol1c, uta1c, Republic in the rest) —
read the script, never assume.

Streams and takes (`lead_tmp/annc/linetab.py`; BF1 TAT, BF2 UTA; seconds = size / 16 x 28 / frequency):

| line | BF1 (stream `gcw_vo` / `cw_vo`) | BF2 (stream `<side>_unit_vo_slow`) |
|---|---|---|
| `<L>_off_com_report_us_overwhelmed` | 4-6 takes, 1.5-2.5 s | 1-4 takes, 1.2-2.5 s |
| `<L>_off_com_report_enemy_losing` | 6 takes, 1.4-2.4 s | 3 takes, 1.6-3.0 s |
| `<L>_off_defeat_im` / `_victory_im` | 1 take, 1.1-1.6 s (GCW scripts only) | 3-6 takes, 1.0-3.6 s |
| `<side>leaving` | 2 takes, 1.3-2.8 s | 3-6 takes, 1.1-3.2 s |
| CP lines (§2.1) | 1 take, 1.3-2.5 s | 2-4 takes, 1.4-3.1 s |

The take is drawn from the property's weighted segment list when the line is enqueued (§1 step 2).

## 4. PS2 live measurements [L]

Tool: `lead_tmp/annc/annc_live.py` (reads the six stream slots, the queue counts, the manager, the teams and every
post from a running PCSX2 every ~10 ms; a played segment id is named through the map's `snd_` scripts). Logs:
`run1_nab1_natural.csv`, `run2_nab1_low.csv` (BF1 NAB1 CW, state `bf1_nab1_cp1_cp2_postA_spawn.p2s`, listener CIS =
team 1), `run4_bf2_uta1.csv` (BF2 UTA1 CW, state `bf2_uta1_cp3_con_cp1_postA_spawn.p2s`, listener CIS = team 1).
Times are game time (clock / 294.912 MHz) from the cause to the line's first frame.

| # | game | cause (RAM) | line that played | onset |
|---|---|---|---|---|
| 1 | BF1 | own bleed timer reaches 30.0 (team 1 rate 0.75, latch set) | `cis_off_com_report_us_overwhelmed` | 1.90 s |
| 2 | BF1 | the same timer, 30.0 s later | `cis_off_com_report_us_overwhelmed` | 0.93 s |
| 3 | BF1 | cp2 owner 0 -> 2 (the enemy captures) | `cis_off_com_report_lost_commandpost` (= `VO_CIS_RepCapture`) | 1.00 s |
| 4 | BF1 | cp1 owner 1 -> 0 (ours neutralised) | `cis_off_com_report_lost_commandpost3` (= `VO_CIS_CISLost`) | 0.97 s |
| 5 | BF1 | team 1 tickets 21 -> 20 (threshold 20; knock-in: tickets poked to 23) | latch 0 -> 1, **no line** (CW script sets none) | — |
| 6 | BF1 | game-over flag 0 -> 1 | music slot: `cis_nab_amb_defeat` | 0.03 s |
| 7 | BF2 | cp0 owner 0 -> 1 (we capture) | `cis_off_com_report_captured_commandpost` | 0.03 s |
| 8 | BF2 | knock-in: team 1 bleed rate 0 -> 1/3 (latch set the next frame) | `cis_off_com_report_us_overwhelmed` | 0.03 s after the latch |
| 9 | BF2 | the bleed timer, 30.06 s after #8 | `cis_off_com_report_us_overwhelmed` (queue 0 -> 1 -> 0) | 0.03 s |
| 10 | BF2 | knock-in: team 2 bleed rate 0 -> 1/3 | `cis_off_com_report_enemy_losing`; no repeat in the next 14 s (then the script cleared the rate) | 0.07 s |
| 11 | BF2 | cp1 owner 0 -> 1; the SCRIPT sets team 2's rate 1/3 in the same frame | `cis_off_com_report_captured_commandpost`, then `cis_off_com_report_enemy_losing` when it ended; no repeat at +30 s or +60 s (the timer wrapped twice, nothing queued) | 0.03 s, 2.84 s |
| 12 | BF2 | cp1 owner 2 -> 0, cp5 2 -> 0 (the enemy's posts neutralised) | nothing | — |
| 13 | BF2 | knock-in: team 1 tickets 16, then 15 (threshold 15) | latch 0 -> 1, `cis_off_defeat_im` | 0.03 s |
| 14 | BF2 | cp5 owner 0 -> 2 (the enemy captures) | `cis_off_com_report_enemyCaptured_commandpost` | 0.03 s |

Reading: **the onset is one frame when the voice slot is idle (BF2: 0.03 s), and the rest of the line then playing
when it is busy** — BF1's 25-a-side battle keeps 3-4 chatter lines pending and a line always playing, and every
commander line still started as the NEXT line (rows 1-4), which is the key order of §1. Row 11 is the queue between
two commander lines (FIFO, 2.84 s = the first line's length).

## 5. Correction: BF1's conquest bleed is live [O]

`analysis/re-tickets-spawn.md` §2.4 read `AddBleedThreshold` as an append (B1 27234-27235, correct) and the scan in
`FUN_002ad830` as first-match (238705-238716, correct), and concluded that the shipped order (31, 0.0), (21, 0.75),
(11, 2.25), (1, 3.0) always selects rate 0. It missed that **the team's vtable slot +0xc, `FUN_002ad2f0`, sorts the
table**: `FUN_0042a150(team[0x13], team[0x12], 8, 0x2ad2e0)` (238416) = qsort(array +0x4c, count +0x48, 8-byte
elements) with the comparator at 0x2ad2e0 = `lw v1,0(a0); lw v0,0(a1); jr ra; subu v0,v1,v0` — ascending by
threshold (`re_check.py word b1 0x2ad2e0`; the function is the data word at 0x53ef04, next to the post-init and
update slots). [L] The table of both teams in a running BF1 NAB1 battle is `[(1, 3.0), (11, 2.25), (21, 0.75),
(31, 0.0)]`; team 1, holding 2 of 5 posts (value 20), had threshold index 2, rate 0.75, and lost one ticket at every
wrap of its accumulator (+0x20) — 6 wraps in 32.2 s of game time (expected 6.04 at 0.75 / 4 per second), beside 11
death decrements; team 2 (3 posts, value 30) had index 3, rate 0.

So on the PS2, BF1 conquest drains, while `alive < tickets`: **0 posts' worth (value <= 1) 0.75 tickets/s, <= 11
(one post) 0.5625/s, <= 21 (two posts) 0.1875/s, <= 31 none; above 31 none** (value = 10 per owned post + 1 per
neutral post). Both teams can bleed at once (2 + 2 posts and one neutral). The port plays BF2's rule on every map
(PLAN §Conquest), so nothing in the sim moves; `core/src/battle.rs`'s module comment repeats the wrong claim.

## 6. What the port needs (the build half of ANNC)

The rules as testable statements (BF2's on every map, per PLAN; BF1 differences are data):

1. Listener = the local player's team. Nothing plays before his first spawn, nothing new after the battle ends.
2. One voice queue per battle: one line at a time; a commander line (key 1.0) goes ahead of pending chatter, behind
   pending commander lines; at most 4 pending — the 5th drops the lowest, and when the new commander line is itself
   the lowest it replaces the previous lowest. A line starts the frame after it is asked for when the voice is idle.
3. T1/T2 by the post's own table `[listener faction][subject faction][Capture | Lost]`, read from the world file
   (the bake must keep the posts' `VO_*` properties); a missing entry is silence (the enemy's loss says nothing).
4. T4: at a team's bleed start; again every 30 s for the listener's own team only; nothing when the listener's own
   team is bleeding and the enemy starts. Names from the mission script.
5. T5: once per battle, the first team at or below trunc(0.1 x its start tickets); team 1 is checked first. A script
   without the call (BF1 CW, three BF2 scripts) is silent.
6. T6: needs a map boundary the port does not have yet (card BNDY): BF2's 6.5 s (BF1 10 s) countdown; the line once
   per excursion, when the warning appears.
7. T8: `common_objComplete` to the winners on BF2 maps; BF1 has none.

Port today: `core::sim` already raises `Event::Captured` / `Event::Neutralized`; `game::sounds::SoundLib` resolves
and plays a stream take by name; the mission RONs (`scripts/gen_missions.py`) do not carry the three setters' names
and the map bake drops the posts' `VO_*` properties.

### 6.1 What the build did (card ANNC, 2026-09-30)

- **Correction: the bake drops nothing.** `bake::map` has always written every instance property as (hash, value);
  the posts' `VO_*` lines were in every baked manifest and only the layout did not read them. No rebake.
  `game::layout` now reads `VO_<L>_<S><Event>` over the four commander factions into `CommandPostDef::vo`; over the
  32 baked maps that is 2,932 properties on 246 capture posts, equal to the disc's as multisets of (instance,
  property, line) (`lead_tmp/annc/check_posts.py`: 0 maps differ), and no baked property names a side outside the
  four. VO properties also sit on instances with no CaptureRegion, which the port does not model: other modes' spawn
  posts in BF2, and BF1's destroyable posts (END1 the bunker panel, GEO1 three techno-union posts, HOT1 the shield
  generator and one unnamed zone — 45 lines).
- `scripts/gen_missions.py` emits each mission's `voice:` block — the three setters' calls with the script's own team
  numbers; 57 missions, 0 differences from the bytecode census (`lead_tmp/annc/check_rons.py`), 16 scripts without a
  low-reinforcement line (13 BF1 Clone Wars scripts incl. `geo1r` and `tat1r`; BF2 `fel1c_con`, `kas2c_con`,
  `yav1c_con`), `kam1c_con` / `mus1c_con` swapped as on the disc.
- `core::announcer` holds rules 1-5 as a pure state machine and the voice queue; `game::announce` feeds it per frame
  and `main/windowed.rs` plays what the queue starts. Of 775 lines the 57 missions can ask for, one has no take:
  BF1 YAV1's temple post names `jungle2`, which is no sound in the baked banks.
- **The voice is busy longer than its take** [L]: in `run4_bf2_uta1.log` the slot went from "started" to "stopped"
  (or to the next queued line) 0.29-0.68 s after the take's own length, over 11 lines (the stream's start-up and
  tail; row 11's 2.84 s is the 2.34 s take, segment 9CB20905, plus 0.48 s). The port holds the voice for the take +
  0.45 s (`VOICE_GAP_SECS`, PROVISIONAL: the mean, not a value read from code).
- Not built: T6 (card BNDY), T7, T8 (`common_objComplete`: the script's condition is unread, §8), the destroyable
  posts' lines, table B.

## 7. Checks run

- **fn** (check 1): every `B1:` / `B2:` line above was found by `lead_tmp/annc/cite.py` by searching INSIDE the named
  function (`cites_b1.txt` 116 rows, `cites_b2.txt` 95 rows, 0 misses).
- **absent / callers** (check 2, `checks_absent.txt`): `FUN_002b1848` — 2 callers, both `FUN_002b1320`;
  `FUN_002b1918` — `FUN_002b1320` 241300 and `FUN_002ad830` 238824 (T7, found by this check); `FUN_00374038` — 2,
  both `FUN_00373b80`; `FUN_00374108` — 1 (371196); `DAT_005fed28`, `DAT_005fd083`, `DAT_005fd082` — one reader, two
  writers each; `SetBleedingRepeatTime` in 0 `.luac`; BF1 bindings: no `BroadcastVoiceOver`, `EnableCommandPostVO`,
  `EnableHeroVO`. Vtable calls are invisible to these searches: the update functions and `FUN_002ad2f0` were found as
  data words (0x53f14c, 0x62d26c, 0x53ef04).
- **twin** (check 3): every B2 function was opened and matched statement for statement with its B1 twin; the
  differences are stated where they exist (CP VO gate, out-of-bounds caller and time, direct-play preemption, T7).
- **word** (check 4): 30.0 `lui 0x41f0` @0x2b10b8 / 0x37391c; threshold default 20 `addiu ..,20` @0x2b107c / 0x3738f8;
  10.0 `lui 0x4120` @0x297f44; 6.5 `lui 0x40d0` @0x3507d8, 2.1667 `lui 0x400a / ori 0xaaab` @0x3507e8; queue cap
  `sltiu ..,5` @0x45d8e4 / 0x5507d4; drain divisor 4.0 `lui 0x4080` @0x2adbf4; team test `slti v0,a1,2` @0x2adc9c;
  the comparator's four words @0x2ad2e0 (`lead_tmp/annc/insn.py`, `re_check.py word`).
- **writers** (check 5): game-over flag B1 `DAT_00519084` (set 328420/328426/328433/328726, cleared 328143/328880),
  B2 `DAT_00600b3c` (set 482434-482766, cleared 482054/482942); demo flag `DAT_0051905c` / `DAT_00600b15` (set only by
  `ScriptCB_LaunchDemo`); `DAT_005fed28` default 1. The dead-branch claim this card met (BF1 bleed) was settled live.
- **Verifier** (one opus agent, foreground, given `lead_tmp/annc/claims.md` — claims and citations only): **32
  SUPPORTED, 5 PARTLY, 0 NOT SUPPORTED of 37**. It re-decoded every constant from ELF words, diffed B2's queue
  function against B1's instruction for instruction, reproduced 36 of the post parser's `VO_*` offsets from the
  hashes, and traced every store to the flags. The 5 PARTLY, all folded into the text above: the Mode3D level
  REPLACES the gain (it is not a factor) — the chatter-key bound in §1.6 is now [H]; B2's direct start has a third
  case (no line recorded); the enable-at-spawn conditions differ between the games (§2.0); T7 also needs the post
  value 0 and an offline game; the battle-end mask is the LOSERS' mask. It also found the shell's `grep` is a
  wrapper that skips binary files (use `/usr/bin/grep -a` for disc searches — the absence searches here were re-run
  that way) and a dead, unreferenced copy of the repeat-time setter at B1 0x2b19e0.

## 8. Stayed [H] / not done

- The session flags behind the ctor's early enable (`DAT_00518be4` family) and the network path of the events.
- BF2's 2.17 s out-of-bounds time: which class flag (bit 1 of class+0x1208) — read as "flyer" from context only.
- Whether any engine path plays a voice line at battle end (one function read, no exhaustive search).
- Hero line triggers (§2.6); the `Objective` script's exact condition for `common_objComplete`.
- BF1's key order uses the bus gains; not read from RAM (the live order agrees in 4 of 4 cases).
- The map boundary's data and test (`FUN_002956d8` / `FUN_0034c550`, chunk 0x54eacca7) — card BNDY.

## 9. Reproduction

```
python3 engine/artifacts/lead_tmp/annc/cite.py b1|b2            # the citation table
python3 engine/artifacts/lead_tmp/annc/luacalls.py bf1|bf2 [script] [--all]   # mission bytecode calls
python3 engine/artifacts/lead_tmp/annc/cpvo.py bf1|bf2 [--detail LVL]         # every post's VO_* properties
python3 engine/artifacts/lead_tmp/annc/sndq.py bf2 UTA rep_off_defeat_im      # a line's raw definition + Inherit
python3 engine/artifacts/lead_tmp/annc/linetab.py bf2 UTA - <names...>        # stream, takes, durations
python3 engine/artifacts/lead_tmp/annc/insn.py b1 0x45d638 0x45d998 slti      # immediates from the ELF
engine/artifacts/pcsx2/pcsx2_session.py start <iso> --name annc --state engine/artifacts/pcsx2/ref/states/bf2_uta1_cp3_con_cp1_postA_spawn.p2s --boot-wait 14 --renderer sw
python3 engine/artifacts/lead_tmp/annc/annc_live.py watch --game bf2 --planet UTA --name annc --seconds 205 --out run.csv \
    --poke 30:bleedrate:1:0.3333 --poke 100:bleedrate:1:0 --poke 105:bleedrate:2:0.3333 --poke 150:tickets:1:16
```
