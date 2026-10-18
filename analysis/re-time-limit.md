# OPUS RE (loop lead, 2026-09-24, lane 1) - [O] UNAUDITED: no Fable pass

**Queue TL: "can a PS2 conquest round that stalls at tickets = alive ever end?"** The one exit the
port did not implement was the conquest TIME LIMIT (`GameOptionsTimeLimitUp`, `re-tickets-spawn.md`
§1.3). **The answer: stock PS2 BF2 single-player conquest sets NO time limit.** No BF2 conquest
mission sets one, and none runs a round timer of its own. Every conquest mission passes
`multiplayerRules = true`, so it asks the game options for the limit, and the options value is
**0 = "Off"** in the ELF image for all four rule sets. With a limit of 0 the Lua never arms its
`loseTimer`. The limit is a **player option**: Instant Action → Options → Conquest → the timer, Off
or 5..60 minutes in steps of 5. It is Off at every power-on. When a player has set one, the round
ends at the limit: **the team with more reinforcements wins, and equal counts make both teams win**
(`MissionVictory({1,2})`). So the port owes no default time limit — the tickets = alive stall has no
timer exit on the stock disc either. The disc's opt-in timer is a shell option the port can add, Off
by default (queue **TLO**). Two things turned up on the way:
- the conquest reinforcement count is a player percentage too (§5);
- **Death Star conquest drains team 1 a second ticket per team-1 death** (§6, queue **DSD**).

Tags: [V] read it (lines, bytes or bytecode cited); [D] derived; [H] hypothesis. `[O]` means one Opus
pass plus one Opus verifier agent, with no Fable audit (§9).

Scratch, every script and every output: `engine/artifacts/lead_tmp/re/timelimit/`. The claims the
verifier was given are `claims.md`, `claims_followup.md` and `claims_c23.md`; its verdicts are
`verify2/VERDICTS.md`.

Conventions:
- **BF2 offsets:** one LOAD segment, file off = va − 0x160000 + 0x1000. BF2 `.text` has no
  `$gp`-relative access at all (0 instructions with base register 28), so every data address is
  formed by `lui` plus an immediate.
- **Lua:** the PS2 scripts are Lua 5.0 bytecode (SIZE_OP 6, A 8, B 9, C 9, 4-byte float numbers,
  RK base 128, as `re-tickets-spawn.md` §1.3 found). They were disassembled by `lua50dis.py`, with
  branch semantics per lua-5.0.2 `lvm.c`. The verifier confirmed those semantics in the ELF's own VM,
  `FUN_004ab5d0`: EQ/LT/LE at DECOMP2 570450–570455, TEST at 570463–570469, and RK `< 0x80` at 570134.
- **Naming:** a Lua function is named by its proto path (`main.f7`) and the global or method it is
  stored as.
- **PC source:** the modtools source (`community_mods/gametoast_lua/.../*.lua`) was a reading aid
  only. Every claim below is read off the PS2 bytecode, the ELF or the decomp.

## 0. The answer in one table

| question | answer |
|---|---|
| Does PS2 retail SP conquest SET a time limit? | **No.** None of the 33 BF2 `*_con` missions sets `timeLimit`; all 33 pass `multiplayerRules = true`. So `Objective:Start` takes the game-options value, `ScriptCB_GetCONMaxTimeLimit()` = `iCONTimer[rules] × 60`, and `iCONTimer` is **0 for every rule set** in the ELF image. With `timeLimit` 0, no `loseTimer` exists. [V][O] (§1 hops 1–7) |
| Does any conquest mission run its own round-ending timer? | **No.** Only 4 of the 33 create a timer at all: TAN1's `timeConsole` (0.3 s, one-shot, turbine-console health +1) and DEA1's `trash_timer` (the compactor's 7 s / 5 s / 15 s cycle). None of the 33 calls `MissionVictory`. [V][O] (§1 hop 6) |
| Which Lua constant, what value? | `iCONTimer`: the engine table at 0x5fa8c8, indexed by the rules index `DAT_005fa7d0` (0 for `'instantaction'`). Its image value is **0**. The Instant Action options screen offers **0 (Off) or 5..60 min, step 5** (`Clamp(iCONTimer + 5·dir, 0, 60)`). [V][O] (§1 hop 8, §2) |
| Is it kept between sessions? | **No** — it starts Off at every power-on. Nothing but `ScriptCB_SetNetGameDefaults` writes the table, and in SP only the options screen calls that. The screen reads the engine's own current values, never a saved profile. [D][O] (§3) |
| What ends the round when a limit IS set? | `ObjectiveConquest:GameOptionsTimeLimitUp`: more reinforcements wins; equal → `MissionVictory({1,2})`, both teams win. [V][O] (§1 hop 9) |
| Does the port owe one? | **Not by default.** The disc's stock exits are the port's: 0 tickets → `MissionDefeat`, and 20 s of domination (`core/src/sim/round.rs`). BF2 has no copy of BF1's exhaustion cull (§4). The opt-in timer is queue **TLO**, Off by default, with no gate re-baseline. [D][O] |
| So can a stalled round end on the disc? | With stock options, only by a death or by domination. A death takes a ticket and an alive unit together, so tickets = alive holds until 0 → defeat. A round in which nobody dies and nobody holds every post (TAN1's zero-shot round) has no exit on the stock BF2 disc either. [D][O] |
| BF1 (bf1_kas2 is a BF1 map)? | No BF1 conquest time limit was found. BF1's only `timelimit` string is a GameSpy key, and none of the 148 files under `extracted/bf1/lua` names one ([V] as searches). "BF1 has none" stays [H], because a timer need not be named. BF1's own stall exit is the engine's exhaustion cull: 900 s, then one AI every 15 s (`re-tickets-spawn.md` §2.6). The port does not run it, because it plays every map under BF2's conquest rules (PLAN §LOCKED FEATURE SET, Modes & rules). [H][O] |

## 1. The hops (BF2, SLUS-212.40)

1. **Instant Action selects rule set 0.** In `shell/ifs_instant_top`, Enter (`main.f2`, pc 16–23;
   when going forward, pc 4) sets `ifs_missionselect.bForMP = nil`. It then calls
   `ScriptCB_SetGameRules('instantaction')` before `ScriptCB_SetIFScreen('ifs_missionselect')`.
   [V][O]
2. **The rule-set index.** `ScriptCB_SetGameRules` (0x0019c2e8, decomp 43365–43404) is the Lua
   binding named at ELF pair 0x5faab0 → 0x19c2e8.
   - It hashes its argument and matches it against the four names behind the pointers at 0x5fa7c0
     (line 43383). It then calls `FUN_0019bfc8(index)` (line 43400).
   - `FUN_0019bfc8` stores `DAT_005fa7d0 = index` (line 43335). Raw: `sw $t9, -0x5830($s6)` at
     0x19c248, with `$s6` = `lui 0x60` at 0x19c0b8 and `$t9` = `$a0` at 0x19bfcc.
   - The pointers (file off 0x49b7c0) name `'instantaction'`, `'mp'`, `'campaign'` and
     `'metagame'` = 0..3. The image value of `DAT_005fa7d0` is 4 (no rules chosen yet).

   [V][O]
3. **Conquest starts the base objective.** In `mission/objectiveconquest`, `ObjectiveConquest:Start`
   (`main.f7`, pc 14–21) calls `ScriptCB_SetNumBots(ScriptCB_GetCONNumBots())`, then
   `Objective.Start(self)`. ObjectiveConquest stores no `timeLimit` of its own. [V][O]
4. **The base objective takes the options' limit under `multiplayerRules`.** In `mission/objective`,
   `Objective:Start` is `main.f7`, stored as `Objective.Start` at main pc 46–47; the class default is
   `timeLimit = 0` at main pc 14.
   - pc 14–18: `timeLimit = timeLimit or 0`.
   - pc 29–41: **if `multiplayerRules`**, `usingGameOptionsTimeLimit = self:GetGameTimeLimit()`, and if
     that `~= 0`, `timeLimit = it`. Otherwise `usingGameOptionsTimeLimit = 0`.

   [V][O]
5. **The timer exists only for `timeLimit > 0`.**
   - pc 108–131: if `0 < timeLimit`, `loseTimer = FindTimer('loseTimer')` or
     `CreateTimer('loseTimer')`, then `SetTimerValue(loseTimer, timeLimit)`, `StartTimer` and
     `ShowTimer` (the HUD countdown).
   - pc 141–148: if `loseTimer` exists, `OnTimerElapse(main.f7.f2, loseTimer)`.

   [V][O]
6. **Every conquest mission passes `multiplayerRules = true`, no `timeLimit`, and no round timer of
   its own.**
   - All 33 `mission/*_con.luac` have exactly one `SETTABLE multiplayerRules := <LOADBOOL true>`
     each, in `main.f0` (`ScriptPostLoad`), into the `ObjectiveConquest:New` table. None stores
     `timeLimit` or `timeLimitWinningTeam`, and the byte string `timelimit` (any case) occurs in 0 of
     the 33 (`mprules_scan.txt`).
   - The verifier's own undumper also found 0 dynamic-key stores. Each mission `DoFile`s only
     `ObjectiveConquest` and `setup_teams`.
   - Reading every global and method name in all 33 (`con_globals_scan.py`): `MissionVictory` is read
     by 0 scripts and `CreateTimer` by 4, with `SetTimerValue`, `StartTimer` and `OnTimerElapse` in
     the same 4 only.
     - `tan1{c,g}_con`: `ScriptPostLoad` pc 84–98 creates `timeConsole`, 0.3 s. Its handler
       `main.f0.f0` sets the turbine console's `CurHealth` to its health + 1, then destroys the
       timer.
     - `dea1{c,g}_con`: `TrashStuff` (`main.f6`, called first thing by `ScriptPostLoad`) runs
       `trash_timer`, 7 s. Its handler `main.f6.f0` alternates `AddDeathRegion('deathregion')` + 5 s
       with `RemoveRegion` + 15 s: the trash compactor.
   - The one `MissionDefeat` outside `Objective` is DEA1's (§6).
   - The two stalled BF2 maps' default missions are `kam1g_con` and `tan1g_con`
     (`engine/missions/bf2_{kam1,tan1}_con.ron`, line 6).
   - Across all 269 BF2 Lua files, the `timeLimit` field is stored only by `objective.luac` (main
     pc 14, `main.f7` pc 18 and 39) and by nine story-campaign scripts (13 stores of 30..480 s in
     `cor1c_c`, `dea1g_c`, `fel1c_c`, `hot1g_c`, `kam1c_c`, `kas2c_c`, `pol1g_c`, `spa1g_c` and
     `yav1g_c`). Those scripts select the `'campaign'` rules.

   [V][O]
7. **The options value is 0.** `ObjectiveConquest:GetGameTimeLimit` (`main.f5`) is `return
   ScriptCB_GetCONMaxTimeLimit()`, the only reader of that binding in all 269 BF2 Lua files.
   `ScriptCB_GetCONMaxTimeLimit` (0x0019b8b0, decomp 43015–43022) is the Lua binding named at ELF
   pair 0x5fa9c0 → 0x19b8b0. It pushes `(float)(int[0x5fa8c8 + 4·DAT_005fa7d0] × 60)`.
   - Raw code from 0x19b8b0 (file off 0x3c8b0): `lw -0x5830(lui 0x60)`; `addiu -0x5738(lui 0x60)` =
     0x5fa8c8; `sll 2`; `addu`; `lw`; `addiu $a1, $zero, 0x3c`; `mult`; `cvt.s.w`; `jal 0x49faf8`
     (`lua_pushnumber`: tag 3, decomp 561316–561330).
   - The int table at 0x5fa8c8..0x5fa8df (file off 0x49b8c8) is **all 0**. The values are minutes,
     ×60 to the seconds `SetTimerValue` takes.

   [V][O]
8. **The player option.**
   - `shell/ifs_instant_options_overview` lists the option groups global, host, hero, **conquest**,
     ctf, hunt and assault. It hides only `host`, and only when not in MP (`main.f0` pc 4–9).
   - `shell/ifs_instant_options_tags` gives the conquest group `{con_numbots, con_mult, con_timer}`.
   - `shell/ifs_instant_options` adjusts `con_timer` as `iCONTimer = Clamp(iCONTimer + 5·dir, 0, 60)`
     (`main.f6` pc 78–90). It labels 0 `'ifs.mp.create.opt.off'` and anything else `'%d'` (`main.f2`
     pc 106–136).
   - On Back it pushes its table through `ScriptCB_SetNetGameDefaults` (`main.f10` → `push_prefs`
     `main.f20`), which stores the table's `iCONTimer` into `int[0x5fa8c8 + 4·rules]` (decomp line
     44422).

   [V][O]
9. **Time up.** `Objective`'s `main.f7.f2` is the `loseTimer` handler. It returns if the objective is
   complete. If `usingGameOptionsTimeLimit ~= 0` it calls `self:GameOptionsTimeLimitUp()`; otherwise
   `self:Complete(timeLimitWinningTeam)`. `ObjectiveConquest:GameOptionsTimeLimitUp` (`main.f6`):
   - `t1 = GetReinforcementCount(1)`, `t2 = GetReinforcementCount(2)`;
   - `t2 < t1` → `MissionVictory(1)`;
   - `t1 < t2` → `MissionVictory(2)`;
   - otherwise `MissionVictory({1,2})`.

   [V][O]

## 2. The per-rule-set option tables (ELF image, file off = va − 0x15f000)

`FUN_0019bfc8` switches the per-rule-set "current" values together, and the getters index each table
by `DAT_005fa7d0`. Each table below is 0x18 bytes: five ints (the command-line parser's loops fill
five; the rules index runs 0..4, 4 being "none chosen yet") and a zero word. Image values
(`tables_5fa7d0.txt`):

| table | Lua key (`ScriptCB_Get/SetNetGameDefaults`) | image value | what reads it |
|---|---|---|---|
| 0x5fa7d8 | `iDMMult` | 100 % | `SetReinforcementCount` for a `_dm` / `_tdm` mission |
| 0x5fa7f0 | `iCONMult` | 100 % | `SetReinforcementCount` for a `_con` mission (§5) |
| 0x5fa850 | `iCTFScore` | 5 | `ScriptCB_GetCTFCaptureLimit` |
| 0x5fa868 | `iHUNTimer` | 5 min | `ScriptCB_GetHuntMaxTimeLimit` (×60) |
| 0x5fa880 | `iHUNTScoreLimit` | 50 | `ScriptCB_GetHuntScoreLimit` |
| 0x5fa898 | `iASSScoreLimit` | 180 | `ScriptCB_GetAssaultScoreLimit` |
| 0x5fa8b0 | `iCTFTimer` | **0 (Off)** | `ScriptCB_GetCTFMaxTimeLimit` (×60) |
| 0x5fa8c8 | `iCONTimer` | **0 (Off)** | `ScriptCB_GetCONMaxTimeLimit` (×60) |

The Hunt timer is the only non-zero timer in the block (5 minutes). Whether Hunt's Lua takes it in SP
was not read. [V][O] for the bytes and the readers. The Lua key names are the strings at 0x6163a0..
(`netgamedefaults_keys.txt`), and the setters' store for each key is in `ScriptCB_SetNetGameDefaults`
(decomp 44320–44425).

## 3. Why it is Off at every power-on — the writers (self-check 2)

- **Code.** Every `.text` instruction whose 16-bit immediate is the low half of an address in
  0x5fa8c8..0x5fa8df is one of six, whatever the base register and however far the `lui` is
  (`immscan.py bf2 0x5fa8c8 0x5fa8e0`).
  - Three form 0x5fa8c8, in `ScriptCB_GetCONMaxTimeLimit` (0x19b8c4), `ScriptCB_GetNetGameDefaults`
    (0x19dadc) and `ScriptCB_SetNetGameDefaults` (0x19dfdc).
  - The other three are not this table:
    - two `addiu -0x5738` under `lui 0x63` form 0x62a8c8 (0x3510f4, with `lui $v0,0x63` at
      0x3510c4; 0x351174, with `lui $v0,0x63` at 0x35114c);
    - one `ori 0xa8d4` under `lui 0xcf49` is a hash compared with `beql` (0x3be280).
  - The verifier's own constant-propagation scan (`verify2/constprop_scan.py`) finds the same three
    and no other.
- **Pointers.** No aligned or unaligned word of the file points into the option block
  0x5fa7c0..0x5fa960 (`ptrscan.py` and the verifier's `ptr_whole.py`: 0).
- **The other writers into the block** store elsewhere. The command-line parser `FUN_00170770` fills
  0x5fa850 and 0x5fa868 with 5-entry loops (decomp 14301 and 14655) and stores single words
  elsewhere, never this table. The rules switch `FUN_0019bfc8` writes 0x5fa808/818/828/838,
  0x5fa8e0.. and 0x5fa8f0.., never 0x5fa8c8.. (43297–43363).
- **No C caller.** The only references to the three functions are their Lua binding pairs
  (0x5fa9c4, 0x5faaf4, 0x5faafc); there is no `jal`/`j` to any of them. [V][O]
- **What this cannot see:** an address built by non-constant arithmetic, or by a base outside the
  block plus an offset.
  - The one dynamic writer near the block is `ScriptCB_SetSpectatorMode`'s byte store
    `(&DAT_005fa7b8)[(int)arg]` (decomp 43259). It would need an index of 272 to reach the table,
    and its stock callers pass 0 (the verifier's reading).
  - Also unseen: writes from outside the EE ELF (IOP/DMA, VU). [H]
- **Lua.** Over all 186 BF2 LVL files (every one an uncompressed `ucfb`; 4 of them hold the Lua
  chunks), `ScriptCB_SetNetGameDefaults` occurs only in SHELL.LVL, twice: once in
  `ifs_instant_options` and once in `ifs_mp_autonet` (MP, which passes `ScriptCB_GetNetGameDefaults()`
  straight back). `iCONTimer` occurs only in SHELL.LVL, three times, all in `ifs_instant_options`.
  The generic `SetMissionTimer` binding is referenced by no Lua file and no LVL. [V][O]
- **The options screen never restores a saved value.** Its `set_defaults` (`main.f19`) is run by
  Enter and by `ifs_instant_options_fnResetDefaults`, which `ifs_missionselect`'s Enter calls. It
  re-reads `GamePrefs = ScriptCB_GetNetGameDefaults()` (pc 10–12).
  - That returns a fresh table whose `iCONTimer` is always set, since `FUN_001a5bf0` pushes the key
    and the number with no condition (decomp 49253–49262).
  - So its `GamePrefs.iCONTimer = GamePrefs.iCONTimer or 60` (pc 83–89) never fires: a Lua number 0
    is true.

  [V][O] Hence the value lives only in the ELF's `.data` from power-on, and a player's choice lasts
  until power-off. [D][O]

## 4. BF2 has no copy of BF1's exhaustion cull (self-check 3, the twin opened the other way)

BF1's only stall exit is the team update's cull (`re-tickets-spawn.md` §2.6). BF1 `FUN_002ad830` seeds
its accumulator with −900.0 once the reserve is exhausted (SWBFRONT decomp 238722; raw
`lui $at, 0xc461; mtc1 $at, $f0` at VA 0x2ada88). It then kills the alive AI member with the lowest
`A − 2·B + 5·C` (238747).

**BF2 materialises −900.0 nowhere:**
- Its only two `lui 0xc461` (0x22c860 in `FUN_0022c7a8`, 0x258868 in `FUN_002586e0`) are each
  followed by `ori 0xd811` and a compare: the property hash 0xc461d811.
- No word outside `.text` is 0xc4610000.
- The scoring expression has 0 hits in BF2's decomp.
- BF2's team-vtable slot +0x14, where BF1's cull lives, is an "is networked" getter
  (`re-tickets-spawn.md` §1.1).

[V][O] for the searches. "BF2 has no cull of any shape" stays [H]: a cull with other constants
elsewhere is not excluded.

## 5. Found on the way (1) — the reinforcement % is a player option too

`re-tickets-spawn.md` §1.2 calls `SetReinforcementCount`'s percentage rescale a no-op, because both
tables are all 100 in the shipping ELF. That holds for the stock options only.
- **The option.** The same screen's `con_mult` row sets `iCONMult = Clamp(iCONMult + 10·dir, 10,
  500)` (`main.f6` pc 65–77), which is pushed into 0x5fa7f0 + 4·rules (decomp 44400).
- **The scaling.** `SetReinforcementCount` (0x0018c3e8, decomp 32419–32463) scales a `_con`
  mission's count by `table[rules] / 100`, once per team.
  - It picks the table by the suffix after the last `_` of the mission name, compared with
    `'_con'` / `'_dm'` / `'_tdm'` at 0x613fd8...
  - A per-team latch bit in `DAT_005fa848` stops a second scaling; `SetNetGameDefaults` clears it.
  - A count < 0 becomes 0x7fffffff, unscaled.

So a player can run SP conquest at 10–500 % tickets. [V][O]

**The port differs here:** `ui::shell`'s Reinforcements row is an ABSOLUTE count (10..1000, step 50).
It is seeded from the mission's team-1 count and applied to BOTH teams (`game/src/main/windowed.rs`
`tickets_set: true`). The disc's row is a percentage of each team's own authored count (default
100 %). Only `bf1_kam1_con` authors unequal counts (200 / 220). Queue TLO carries the fix.

## 6. Found on the way (2) — Death Star conquest drains team 1 twice per death

`dea1c_con` and `dea1g_con` both register two extra handlers in `ScriptPostLoad` (`main.f0` pc 25–31):
- `OnCharacterDeathTeam(main.f0.f0, 1)`, where `main.f0.f0` = `AddReinforcements(1, -1)` with **no
  condition**;
- `OnTicketCountChange(main.f0.f1)`, where `main.f0.f1` = `if count == 0 then MissionDefeat(team)`.

[V][O]

`Objective:Start`'s own death handler still runs: `main.f7.f0`, registered at pc 135–137 because the
conquest objective has no container. It takes one ticket per death from every team, floored at 0
(`re-tickets-spawn.md` §1.3). **So on DEA1 a team-1 character's death costs team 1 two tickets, and
a team-2 death costs one.** [D][O]

Two readings behind that are not closed:
- **The filter's subject.** That the `Team` filter selects the DYING character's team is [D]. The PC
  modtools' own `gametype_conquest.lua` uses the same call as `-- teammate death`, and the PC
  `dea1g_con.lua` comments it `-- handle reinforcment loss and defeat condition`. The PS2 filter is
  built by the generic `On%s%s` + `Team` registration (`FUN_00478658`, decomp 536623; strings
  0x63ad88 / 0x63ad90), and the compare it installs was not traced. [H] on the binding itself.
- **The floor.** The extra drain has no floor check. Whether `AddReinforcements` clamps at 0 is in
  `FUN_0036e0d8`, not read here. [H]

**The port has neither handler:** `engine/missions/bf2_dea1_con{,_cw}.ron` carry no such field, and
`scripts/gen_missions.py` extracts none. Lua team 1 is the port's team 0 (`all` in GCW, `rep` in
CW). Queue **DSD** carries the fix; it moves bf2_dea1's sweep row, so it re-baselines the sweep.

## 7. What the port does today, and what it owes

- **The round's exits.** `core/src/sim/round.rs` `update_cps` ends a round at 0 tickets (`Objective.lua`
  `OnTicketCountChange`) or by domination (`cp::tick_domination`), and has no timer. Those are
  exactly the disc's stock exits. `grep -rniE 'time_limit|timelimit' engine/crates/{core,game}/src`
  finds nothing (queue TL, 2026-09-17).
- **The NO-OUTCOME maps are not a missing-timer defect.** The sweep's NO-OUTCOME maps (bf2_kam1,
  bf2_tan1, bf1_kas2) are not caused by a missing timer. Whether the disc's AI would break those
  stalls is a separate question about the AI, not about the rules.
- **Queue TLO (the enabled build):** the Instant Action conquest option group.
  - The timer: Off by default (the faithful stock value), 5..60 min in steps of 5. At the limit the
    round ends with the reinforcement comparison, and a tie is both-win. A HUD countdown shows when
    a limit is set (`ShowTimer`).
  - The reinforcement % (§5).
  - Off and 100 % by default, so no gate value moves.
- **Queue DSD:** DEA1's second ticket per team-1 death (§6). This is a [SWEEP] re-baseline.

## 8. Not closed ([H], for a later pass)

- BF1 has no conquest time limit. This rests on two searches (the ELF's strings, and all 289 files
  under `extracted/bf1`, 148 of them under `lua/`), not a proof. [H]
- BF2 has no exhaustion cull of any shape. §4 excludes BF1's own cull (its constants), not another.
  [H]
- A write into the CON-timer table through an address built by arithmetic, or from outside the EE
  ELF (§3). [H]
- DEA1: the `Team` filter's subject and `AddReinforcements`' floor (§6). [H]
- What `ifs_mp_autonet` does with the table under the `'mp'` rules (MP only). [H]
- The localized text behind `'ifs.mp.create.opt.timer'` (the label's units) was not read. [H]

## 9. How this was verified (`OPUS_LOOP_PROMPT.md` §5)

- **Hops re-read by the lead.** Every hop in §1 was re-read in this session from the disassembly
  (`dis_*.txt`), the decomp lines and the raw bytes. So were the Lua binding pairs of all five
  bindings (`ScriptCB_GetCONMaxTimeLimit`, `ScriptCB_SetGameRules`, `ScriptCB_Get/SetNetGameDefaults`,
  `SetReinforcementCount`: name → function at 0x5fa9c0 / 0x5faab0 / 0x5faaf0 / 0x5faaf8 / 0x5f9bd8).
- **Self-check 1, the function each line lives in.** Every decomp citation's enclosing
  `/* ==== … ==== */` header was printed:
  - 43015 `ScriptCB_GetCONMaxTimeLimit`;
  - 43297 `FUN_0019bfc8`;
  - 43365 `ScriptCB_SetGameRules`;
  - 44123 `ScriptCB_GetNetGameDefaults`;
  - 44320 `ScriptCB_SetNetGameDefaults`;
  - 49241/49253 `FUN_001a5ba0`/`FUN_001a5bf0`;
  - 14061 `FUN_00170770`;
  - 536623 `FUN_00478658`.

  Every Lua citation names its proto and the global or method that holds it.
- **Self-check 2, every absence claim searched.**
  - The CON-timer writers (§3): an immediate superset scan and a lui-resolving scan over the whole
    block; `$gp` use, 0; the pointer scan.
  - `ScriptCB_SetNetGameDefaults` / `iCONTimer` / `SetMissionTimer` over all 186 LVLs.
  - `timeLimit` stores and every global/method name over all 33 `_con` missions and all 269 BF2
    `.luac`.
  - `timelimit` over every BF1 file.
  - −900.0 over both ELFs.

  Counts are in the claims files and were reproduced by the lead (`lead_repro.py`).
- **Self-check 3, the twin opened.** §4 opens BF1's cull and searches BF2 for its constants and its
  expression.
- **Self-check 4, constants from bytes.**
  - The tables at 0x5fa7d0..0x5fa90c were read from the file (`tables_5fa7d0.txt`), not from decomp
    literals.
  - The ×60 is the instruction `addiu $a1,$zero,0x3c` at 0x19b8b0 ff.
  - The −900.0 is `lui 0xc461`.
- **The verifier** was one `opus` agent given the claims and the citations only (C1–C26), briefed to
  falsify every absence claim with a search of its own. It returned **25 SUPPORTED, 1 PARTLY, 0 NOT
  SUPPORTED** (`verify2/VERDICTS.md`).
  - It checked the RK base and the branch semantics in the ELF's own VM.
  - It wrote its own undumper, which parses all 269 files and agrees with `lua50dis.py` on every
    cited proto.
  - It ran a constant-propagation scan and a whole-file pointer scan.
  - **The PARTLY, C18:** the BF1 Lua tree holds 148 files (147 `.luac` + `shell/mcfg_12d97623.bin`),
    not 153, and the GameSpy key table starts at 0x51bb54. This document uses the corrected count.
    The result ("no `timelimit`") holds for all 148, and for all 289 files under `extracted/bf1`.
  - The lead reproduced the verifier's load-bearing counts (`lead_repro.py` and one-off scans): 186
    LVLs all `ucfb`; SetNetGameDefaults ×2 and iCONTimer ×3, all in SHELL.LVL; SetMissionTimer 0;
    the 16 `timeLimit` stores; 1 BF1 hit; no `j`/`jal` to any of the five bindings, and
    `FUN_0019bfc8` called only from 0x19c3e8 and 0x19e2e0.
  - An earlier verifier run (iteration 3) was lost when that session ended while waiting on it. Its
    scratch (`verify/`) was not given to the second verifier.
- **What stays [H]:** §8.
