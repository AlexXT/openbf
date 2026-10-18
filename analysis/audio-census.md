# Gameplay audio census (card SNDC, 2026-10-02) - [O] UNAUDITED

What the PS2 plays in a battle, by name, against what the port plays. One showcase map per game: BF1 Naboo Plains
(NAB1, Clone Wars, player = CIS super battle droid) and BF2 Utapau (UTA1, Clone Wars, player = CIS). Tags: [L] measured
live on PCSX2, [V] read in the decomp (function checked with `re_check.py fn`), [D] derived, [H] hypothesis. This is a
measurement, not an RE pass: no verifier agent ran; the engine facts in section 1 are the instrument's layout and are
checked by what the instrument reads (section 1.3).

> **Corrections (2026-10-03, card AMBM, `analysis/re-ambience-music.md` 7).** The command post hum is picked by the
> post OWNER's faction (`all` / `rep` -> `_goodie`, `cis` / `imp` -> `_baddie`), not by friend / foe. The Utapau run
> does not show `start` -> `middle` -> `end`: the takes seen are shared between the three properties; and the battle
> had not ended at `cis_off_victory_im` (the low-reinforcement line - post lines follow it), so no victory music is
> in the log. The spawn screen's "shell_menu_accept" rows are `shell_menu_enter` (one shared sample). The ambience
> streams are the world's `soundstream` regions: `nab_amb_meadow` (EB51145B), `uta_amb_wind` (7C8CA2CF),
> `amb_hangar` (4FEEB00F).

> **Correction (2026-10-13, card USND, `analysis/re-unit-chatter.md` 2.6, 5).** The voice rows of sections 2 and 3
> name a line by its audio, and a take is shared by several properties: `..._hide`, `..._approach` and
> `..._predamage` are never requested on the PS2 (no caller [receipt:69e3eb1c]; their gates never arm) - `gungan_chatter_approach` and
> `gungan_chatter_heartbeat` here are the Gungans' `LowHealthSound`, asked for as long as they live. Per-event rates
> with the property named exactly, and BF2's lines with the player inside the fight (8 to 12 a minute against the
> 0 to 1 of this census), are in that document.

## 0. Headline

- **The port plays 5 of the 24 event rows the PS2 plays on BF1 and 4 of the 21 heard on BF2**: infantry fire, reload,
  the bolt's strike, the commander, and (BF1) the squad's answer to the player's own order. Everything else is
  silent: footsteps and all other foley, hurt and death voices, AI chatter, vehicle engines, vehicle and turret
  weapons, walker steps, explosions, the bolt's flight hum, weapon changes, music, ambience, command post sounds,
  pickups, menu sounds.
- **What the port does play is cut off at 50 m and has the wrong loudness** [L port, V disc]. `bake sounds` resolves a
  property's `Inherit` template only inside its own SOUND LVL, and the templates live in CORE.LVL. Baked
  `rep_weap_inf_rifle_fire` (both maps): gain 1.0, 5 - 50 m, rolloff 1. Disc template 0xB1AD507C (in
  `core_bf1/sounds/sounds.ron` itself): gain 0.7, 5 - 400 m, rolloff 6 (BF2 3.5). Explosions inherit 30 - 450 m
  (0xBFA71AD3), engines 25 / 45 - 450 m, foley 10 - 50 m gain 0.7, chatter 16 - 150 m. On the PS2 85 % (BF1) and
  94 % (BF2) of the infantry fire the player hears starts beyond 50 m (median 153 m / 77 m). In the port's UTA1
  battle the player hears 0 of 760 CIS shotgun shots and 11 of 540 Republic rifle shots.
- The foley table's sound names are not strings anywhere on the disc; 303 were recovered on NAB by hashing
  `<side>_inf_<unit>_<action>_<surface>[_left|_right]` (e.g. `cis_inf_droid_run_grass_left`): 572 of the LVL's 1080
  properties were unnamed before, 263 after (about 3 chance collisions expected among the 303 [D]; the complete
  run / walk x left / right sets are safe).

## 1. The instrument

### 1.1 PS2: `engine/artifacts/lead_tmp/sndc/snd_live.py` (new)

`snd_live.py watch --game bf1|bf2 --planet <SOUND LVL> --name <session> --seconds N --out run.csv --pad "T:KEY:MS,..."`
reads a running PCSX2 every ~4 ms and writes one row per sound START, with the game clock. Pad presses and the run
length are timed by the GAME clock (another lane's sweep took the emulator to 1 % speed twice during this card).
`snd_live.py state <file.p2s>` reads the same from a save state.

- **78 one-shot voices** [V]: B1 pool `*(0x5228cc)` = 0x74cb60 (`FUN_00459500` 518720), B2 `*(0x60be70)` = 0x832dc0
  (`FUN_0054c6c8` 681714), stride 0x110 (B1 `FUN_004599e0` 519025; B2 `FUN_0054cba8` 682020), loop bound 0x4e
  (B1 `FUN_00459e08` 519128). State flags u16 at +0x7c (B1) / +0x84 (B2): bit 0x08 = playing, 0x40 = idle
  (B1 `FUN_0044e218` 511258-511265). Sample record pointer at +0xa0 (B1 `FUN_0044b068` 509009) / +0xa8 (B2
  `FUN_0053ee88` 672593); the record's id (the SampleBank hash) is at record+0x10 [L: RAM]. Position copy per voice:
  B1 0x674b30 + 16 i (519032), B2 0x743520 + 16 i (682027). A start with zero gain is refused (B1 518974), so a
  "started" row is a sound that was in range.
- **39 handle voices** [V]: B1 `*(0x5220c4)` = 0x73fe10, B2 `*(0x60b654)` = 0x826030, stride 0x150, count 0x27 (B1
  `FUN_00448c70` 507393-507411; B2 `FUN_0053ca78` 670883-670885), same state / sample offsets [L]. Engine layers
  appear ONLY here; a one-shot shows in both sets. The "audible" column counts starts in this set. What the set IS
  (the voices actually mixed, or the voices a caller holds a handle to) is not established [H]; either way a row
  here is a playing sample. Found because no engine sample was in the 78 voices of any of 330 reference states,
  including one with the player piloting an AAT (`lead_tmp/sqveh/nab1_cis_aat_pilot.p2s`): a RAM scan for the
  engine's sample records found their references at 0x7420d0 / 0x741f80 / 0x740690 = this pool + 0xa0.
- **6 stream slots** (music, voice, ambience): as `lead_tmp/annc/annc_live.py` (analysis/re-announcer.md section 1).
- **Names**: a sample id -> every sound property of the SOUND LVL's `snd_` scripts that lists it
  (`lead_tmp/s12/re/sound_scratch/<game>_<PLANET>.snd.json`); `a|b` = one sample shared by several properties (the
  voice does not keep the property); `parent>HASH` = an unnamed layer of a layered / parameterized sound. The `Group`
  field (hash 0x5FB91E8C; `weapons`, `explosion`, `Ordnance`, `vehicles`, `collision`, `props` named) sorts the rest.
- Tools beside it: `tab.py` (a run by name, or the near-player time line beside the pad presses), `census.py`
  (per-sound counts), `tables.py` (the tables below), `states.py` (the sounds playing in every reference state:
  336 distinct names on BF1, 474 on BF2, `states_census.txt`), `brute.py` (the foley names), `odfkeys.py` (section 4).

### 1.2 Port: `SOUND_LOG` (scratch patch `lead_tmp/sndc/port/sound_log.patch`)

The headless walk replay (`game --headless --walk --walk-replay`) with 37 added lines: after each stepped tick it
calls `game::event_sounds::each` and the commander's `frame` as `main/windowed.rs` does and writes
`tick,src,name,has,take,dist`. Input `lead_tmp/sndc/port/pad120.ron` (120 s: walk, jump, fire, reload, secondary,
switch, roll, the four squad orders, sprint), `--mode con --mission missions/<map>_con_cw.ron`. Sim hashes after 7200
ticks: NAB1 `53c29970f8fcc3c8`, UTA1 `64c40dde6eaa7deb`. Logs `lead_tmp/sndc/port/<map>_sounds.csv`, `_events.csv`.
The patch is the referee the gap cards re-use; it is not landed.

### 1.3 The runs [L]

| run | game s | starts (78 / 39 / streams) | file |
|---|---|---|---|
| BF1 NAB1, scripted pad | 105.0 | 3647 / 3604 / 70 | `bf1_nab1_run2.csv` |
| BF1 NAB1, pause menu + respawn | 22.0 | (1644 rows) | `bf1_nab1_ui.csv` |
| BF2 UTA1, scripted pad (the battle ended 49 s into the run: `cis_uta_amb_end`, then `cis_off_victory_im`; 1190 / 1479 starts before, 708 / 869 after) | 105.0 | 1898 / 2348 / 9 | `bf2_uta1_run2.csv` |
| BF2 UTA1, pause menu + respawn + spawn screen | 28.7 | (400 rows) | `bf2_uta1_ui.csv` |

States: `ref/states/bf1_nab1_cp1_cp2_postA_spawn.p2s`, `bf2_uta1_cp3_con_cp1_postA_spawn.p2s`, SW renderer. The pad
script is the instrument's own check: each press is followed within 0.1 game s by its sound at the player (4 m from
the eye) - `W` -> `cis_inf_droid_walk_grass_left`, then run steps alternating right / left every 0.30 s (BF1; 0.27 s
BF2 on stone), `Cross` -> `cis_inf_droid_jump_grass`, and `..._land_grass` 0.90 - 1.03 s later, `R1` ->
`cis_weap_inf_blaster_fire` + `com_weap_inf_ord_hum_sm` every 0.20 s, `L1` -> `cis_weap_inf_rocket_fire` +
`com_weap_inf_ord_hum_rocket`, `Square` / `R2` -> `cis_weap_inf_equip_med`, BF2 `Circle` -> `..._squat_stone` /
`..._roll_stone`; a hit taken -> `cis_inf_com_chatter_wound`, death -> `droid_death` (`tab.py <run> <game> <P> near`;
the first runs `bf1_nab1_run1.csv` / `bf2_uta1_run1.csv`, 78 voices only, show the same).

## 2. BF1 Naboo Plains, 105 game seconds + the menu sample

Counts are sound starts: one-shot voices / audible (handle) voices / stream starts. Port column: names from the port's
120 s log with their start counts (all distances).

| category | PS2 event | sound properties heard (started / audible / stream starts) | ODF key or source | port plays it |
|---|---|---|---|---|
| Vehicles | vehicle and turret weapon fire | `cis_weap_hover_aat_cannon_fire`, `com_weap_turret_fire`, `cis_weap_hover_aat_launcher_fire` (+6): 970 / 923 / 0 | FireSound, FireLoopSound | **N** |
| Weapons | infantry weapon fire (incl. charge / spin-up) | `rep_weap_inf_rifle_fire`, `cis_weap_inf_blaster_fire`, `cis_weap_inf_pistol_fire` (+9): 1516 / 1305 / 0 | FireSound, ChargeSound | Y: `cis_weap_inf_blaster_fire` 1427, `rep_weap_inf_rifle_fire` 1252, `cis_weap_inf_rocket_fire` 258 (+4); in range: note 2 |
| Weapons | weapon change / mode change (equip) | `cis_weap_inf_equip_med`, `rep_weap_inf_equip_sm`, `cis_weap_inf_equip_lg` (+1): 31 / 24 / 0 | WeaponChangeSound, ChangeModeSound | **N** (the port's 10 `cis_weap_inf_equip_med` starts are the wrist blaster's ReloadSound, not a weapon change) |
| Weapons | reload | `cis_weap_inf_reload_med>E6A296B7`, `rep_weap_inf_reload_med>FAB93B6D`: 12 / 15 / 0 | ReloadSound | Y: `cis_weap_inf_reload_med` 95, `rep_weap_inf_reload_lg` 75, `rep_weap_inf_reload_med` 14 (27 in range; note 3) |
| Weapons | bolt / rocket in flight (loop on the ordnance) | `com_weap_inf_ord_hum_sm`, `com_weap_inf_ord_hum_lg`, `com_weap_inf_ord_hum_rocket`: 174 / 116 / 0 | OrdnanceSound | **N** |
| Impacts | bolt strikes (unit or world) | `imp_weap_ord_exp`: 88 / 54 / 0 | CollisionSound | Y: `imp_weap_ord_exp` 266 (34 in range) |
| Impacts | grenade / mine bounce and landing | `com_weap_inf_grenade_bounce`, `E7EEBDA6`: 36 / 38 / 0 | CollisionOtherSound | **N** |
| Explosions | ordnance and object explosions (layered: low / high / near / distant / frequent) | `imp_weap_ord_exp_lg_frequent`, `com_weap_inf_conc_grenade_exp_low`, `com_weap_inf_haywire_exp` (+8): 990 / 842 / 0 | explosion class SoundProperty | **N** |
| Vehicles | engine (parameterized, 2-4 layered loops) | `cis_hover_aat_engine_parameterized>5ACF7D70`, `interceptorTank_engine_parameterized>24C23A6F`, `cis_hover_stap_engine_parameterized>5709327A` (+11): 0 / 338 / 0 | EngineSound | **N** |
| Vehicles | walker / creature footsteps | `kaadustepright`, `kaadustepleft`: 88 / 88 / 0 | FootstepSound0-5 | **N** |
| Vehicles | turret turning, boarding, collision, creature calls, droideka fold | `Kaadu`, `vehicle_equip`, `kaaducry` (+2): 34 / 27 / 0 | TurretYawSound, TurretPitchSound, TurretActivateSound, VehicleCollisionSound | **N** |
| Foley | soldier footsteps: run / walk x left / right x surface | `cis_inf_droid_run_grass_right`, `cis_inf_droid_run_grass_left`, `cis_inf_droid_walk_grass_left` (+9): 217 / 203 / 0 | foley table (hash-only names) | **N** |
| Foley | jump, land, roll, squat, get up | `cis_inf_droid_getup_grass`, `cis_inf_droid_jump_grass`, `cis_inf_droid_land_grass` (+5): 20 / 14 / 0 | foley table | **N** |
| Voice | hurt and death voice (sample bank, 3-D) | `rep_inf_com_chatter_wound`, `cis_inf_com_chatter_wound`, `rep_inf_com_chatter_death` (+3): 154 / 117 / 0 | HurtSound, DeathSound | **N** |
| Voice | AI combat chatter (voice stream, queued) | `gungan_chatter_acquired`, `gungan_chatter_heartbeat`, `gungan_chatter_approach` (+12): 0 / 0 / 62 | AcquiredTargetSound ... / VOSound | **N** |
| Voice | squad-order talk (voice stream) | `ai_cis_inf_com_chatter_tac_yes`, `ai_cis_inf_com_chatter_tac_follow`, `ai_cis_inf_com_chatter_tac_allclear` (+3): 0 / 0 / 16 | SC* / AISC* keys | partly: the answer to the PLAYER's order only (1 `cis_inf_com_chatter_tac_allclear`); the AI's own talk **N** |
| Voice | commander lines | `cis_off_com_report_us_overwhelmed`, `all_off_com_report_lost_commandpost`: 0 / 0 / 5 | mission Lua | Y: `cis_off_com_report_us_overwhelmed` 3, `cis_off_com_report_lost_commandpost` 3 (+3) |
| Music | ambient music: start / middle / end by tickets | `cis_nab_amb_middle`: 0 / 0 / 4 | SetAmbientMusic | **N** |
| Ambience | map ambience streams | `EB51145B` (stream `nab1`): 0 / 0 / 2 | sound regions / streams | **N** |
| Ambience | command post hum loop (friend / foe), capture charge / discharge | `com_blg_commandpost_goodie`, `com_blg_commandpost_baddie`: 19 / 99 / 0 | command post class | **N** |
| Ambience | prop emitters and droids (fire, fx7, gonk, health droid) | `com_amb_fire`, `com_item_gonkrecharge`, `health_recharge>C8D7E417` (+2): 8 / 10 / 0 | AmbientSound, map sound emitters | **N** |
| UI | pickup (ammo / health) | `com_weap_ammo_pickup`: 3 / 4 / 0 | PowerupSound | **N** |
| UI | menu move / accept / exit, class select | `shell_select_change`, `shell_menu_accept`, `shell_menu_exit` (CORE bank): 16 / 16 / 0 | CORE.LVL bank | **N** |
| ? | an unidentified loop that shares its sample with `com_weap_energy_depleted` / `com_weap_remote_charge_fire` (note 1) | `0FB06B34`: 74 / 100 / 0 | 0xF4A517D4 / 0xA0B09F74 name the two | **N** |

## 3. BF2 Utapau, 105 game seconds (49 of battle, 56 after the result) + the menu sample

| category | PS2 event | sound properties heard (started / audible / stream starts) | ODF key or source | port plays it |
|---|---|---|---|---|
| Vehicles | vehicle and turret weapon fire | `com_weap_turret_fire_small`, `cis_weap_hover_aat_cannon_fire`, `com_weap_turret_fire_mini` (+2): 172 / 172 / 0 | FireSound, FireLoopSound | **N** |
| Weapons | infantry weapon fire (incl. charge / spin-up) | `rep_weap_inf_rifle_fire`, `cis_weap_inf_rifle_fire`, `all_weap_inf_commando_pistol_fire` (+15): 521 / 396 / 0 | FireSound, ChargeSound | Y: `cis_weap_inf_wrist_blaster_fire` 876, `cis_weap_inf_shotgun_fire` 760, `rep_weap_inf_rifle_fire` 540 (+7); in range: note 2 |
| Weapons | weapon change / mode change (equip) | `com_weap_inf_equip_med`, `com_weap_inf_equip_sm`, `com_weap_inf_equip_lg`: 38 / 19 / 0 | WeaponChangeSound, ChangeModeSound | **N** (the port's 8 `com_weap_inf_equip_med` starts are a ReloadSound, not a weapon change) |
| Weapons | reload | `com_weap_inf_reload_lg`, `com_weap_inf_reload_med`: 2 / 1 / 0 | ReloadSound | Y: `com_weap_inf_reload_lg` 97, `com_weap_inf_reload_med` 31 (1 in range; note 3) |
| Weapons | bolt / rocket in flight (loop on the ordnance) | `com_weap_inf_ord_hum_lg`, `com_weap_inf_ord_hum_sm` (+4): 107 / 29 / 0 | OrdnanceSound | **N** |
| Impacts | bolt strikes (unit or world) | `imp_weap_ord_exp`: 231 / 101 / 0 | CollisionSound | Y: `imp_weap_ord_exp` 2930 (350 in range) |
| Impacts | grenade / mine bounce and landing | `com_weap_inf_grenade_bounce`, `509BDFEF`, `com_weap_mine_land>4E176B01`: 89 / 75 / 0 | CollisionOtherSound | **N** |
| Explosions | ordnance and object explosions (layered: low / high / near / distant / frequent) | `com_weap_inf_conc_grenade_exp_low`, `imp_weap_ord_exp_lg_frequent`, `com_weap_thermaldetonator_exp` (+11): 392 / 376 / 0 | explosion class SoundProperty | **N** as an explosion (9 `imp_weap_ord_exp_lg` are a rocket's CollisionSound on a hit) |
| Vehicles | engine (parameterized, 2-4 layered loops) | `atst_engine_parameterized>7BB58EF1`, `atst_engine_parameterized>E9B18ACC`, `atst_engine_parameterized>A0883F04` (+4): 0 / 232 / 0 | EngineSound | **N** |
| Vehicles | walker / creature footsteps | `com_weap_layered_at_step`, `atst_leg_up`, `cis_inf_droideka_step`: 269 / 247 / 0 | FootstepSound0-5 | **N** |
| Vehicles | turret turning, boarding, collision, creature calls, droideka fold | `com_veh_collision_lg`, `turret_whir_yaw_lp`, `turret_whir_pitch_lp` (+3): 52 / 57 / 0 | TurretYawSound, TurretPitchSound, TurretActivateSound, VehicleCollisionSound | **N** |
| Foley | soldier footsteps: run / walk x left / right x surface | `cis_inf_droid_run_stone_left`, `cis_inf_droid_run_stone_right`, `cis_inf_droid_walk_stone_right` (+1): 66 / 60 / 0 | foley table (hash-only names) | **N** |
| Foley | jump, land, roll, squat, get up | `cis_inf_droid_jump_stone`, `cis_inf_droid_land_stone`, `cis_inf_droid_squat_metal` (+2): 9 / 5 / 0 | foley table | **N** |
| Voice | hurt and death voice (sample bank, 3-D) | `cis_inf_com_chatter_wound`, `rep_inf_com_chatter_wound`, `droid_death` (+1): 270 / 241 / 0 | HurtSound, DeathSound | **N** |
| Voice | AI combat chatter (voice stream, queued) | (not heard in this sample; note 5) | VOSound | - |
| Voice | squad-order talk (voice stream) | (not heard in this sample) | VOSound SC_* events | - |
| Voice | commander lines | `cis_off_com_report_captured_commandpost`, `cis_off_com_report_enemy_losing`, `cis_off_com_report_lost_commandpost` (+1): 0 / 0 / 7 | mission Lua | Y: `cis_off_com_report_enemyCaptured_commandpost` 2, `cis_off_com_report_captured_commandpost` 1 |
| Music | ambient music: start / middle / end by tickets, then victory | `cis_uta_amb_middle`, `cis_uta_amb_end`: 0 / 0 / 3 (`cis_off_victory_im` 1 more, counted in the commander row; `cis_uta_amb_start` was playing at the state's start) | SetAmbientMusic | **N** |
| Ambience | map ambience streams | `4FEEB00F`, `7C8CA2CF`: 0 / 0 / 6 | sound regions / streams | **N** |
| Ambience | command post hum loop (friend / foe), capture charge / discharge | `com_blg_commandpost_baddie`, `com_blg_commandpost_goodie`, `com_blg_commandpost_discharge` (+1): 14 / 54 / 0 | command post class | **N** |
| Ambience | prop emitters and droids (windmill, neon, fx7, gonk) | `326DCB7A`, `windmill`, `droid_fx7_idle_chatter` (+1): 31 / 99 / 0 | AmbientSound, map sound emitters | **N** |
| UI | pickup (ammo / health) | `com_weap_ammo_pickup`: 10 / 9 / 0 | PowerupSound | **N** |
| UI | menu move / accept, class select | `shell_select_change`, `shell_menu_accept`, `shell_select_unit` (CORE bank): 18 / 3 / 0 | CORE.LVL bank | **N** |

Notes for both tables:
1. A row's names are examples; the full per-sound list is `lead_tmp/sndc/<game>_<map>_census.csv` (130 BF1 rows, 102
   BF2). `0FB06B34|com_weap_energy_depleted|com_weap_remote_charge_fire` (BF1, 74 starts, a looping sample shared by
   three properties) is not identified [H].
2. Port fire in range (distance < the BAKED max distance, 50 m): BF1 581 of 3412 starts, BF2 273 of 2761. With the
   disc's 400 m every one of them is in range.
3. Reload: the PS2 started 12 (BF1) and 2 (BF2) reload sounds in 105 s; the port fired 184 and 128 `Reload` events in
   120 s (27 and 1 within 50 m). Whether the PS2's AI reloads silently or rarely is not established [H] (card WSND).
   **Settled (WSND, `re-weapon-sounds.md` 8):** neither - the sound is cut off at 50 m, every PS2 start was within
   48.6 m, and per in-range shot the two agree (0.055 against 0.046); a bot's reload plays the sound.
4. Layered sounds: a `soundlayered` record's list is not a list of layer names. It interleaves layer property hashes
   with other values - `com_weap_throw`: 0x77075624, a layer, 0x3E99999A (0.3 as f32); `com_weap_inf_conc_grenade_exp`
   starts with a float, a layer, 0, 0 [L: the `snd_` dump]; the record's layout is not decoded here [H]. The bake
   stores every value as a layer hash (`cis_weap_inf_reload_med`: `layers: [1996966820, 3869415095, 1050253722]`), so
   the port starts the one real layer at once. The PS2 keeps delayed layers in a 78-entry pool (B1 0x751e40, stride
   0x80, `FUN_0045c130` 520611-520627 [V]) - a layer can start late.
   **Decoded (WSND, `re-weapon-sounds.md` 1-2):** an entry is `Layer(layer name, sound, delay [, deviation])`; the
   first word above is the name `null`, 0x77074BA4 (0x77075624 was a misprint).
5. Not sampled here: hero sounds, jet packs under the player (`rep_inf_Jetpack_engine_parameterized` was audible 25
   times from AI jet troopers on NAB1), space, cutscene / shell music, BF2's AI `VOSound` chatter (one line,
   `cis1_inf_chatter_GivePowerup`, in the first BF2 run `bf2_uta1_run1.csv`; ANNC measured it as sparse on UTA1).
6. **Vehicle rows settled (VSND, `re-vehicle-sounds.md`):** the engine's parameters are `speed`, `acceleration`,
   `health`, `turnontime`, `turnofftime` (sections 1-4); the raw hashes of section 4 are 0xEA603E52 `TurnOnSound`,
   0x042DABAE `TurnOffSound`, 0x069134E5 `HydraulicSound`; 0x1ADA38C1 (`WeaponChange`) is read by neither ELF - the
   boarding sound is `TurretActivateSound`. The "audible" engine starts are layers re-taking a voice [H].

## 4. Which ODF keys name sounds (static, `odfkeys.py`)

Every property of the baked classes whose value is a sound property of the map's SOUND LVL
(`lead_tmp/sndc/bf1_odfkeys.csv`, `bf2_odfkeys.csv`; 90 keys on BF1, 66 on BF2; key names from the property-name
dictionary, 47 and 29 of them still hashes). The keys the port plays: `FireSound`, `ReloadSound`, `CollisionSound`
(3 keys). Named in the port's source but not played: `OrdnanceSound`, `FireEmptySound`. The largest unplayed ones:

| key | classes BF1 / BF2 | heard on the PS2 (section 2 / 3 row) |
|---|---|---|
| `DeathSound`, `HurtSound` | 73, 44 / 62, 38 | hurt and death voice |
| `SoundProperty` (explosion classes) | 68 / 118 | explosions |
| `WeaponChangeSound`, `ChangeModeSound` | 60, 54 / 97, 93 | weapon change |
| `OrdnanceSound` | 75 / 46 | flight hum |
| the AI chatter keys (`AcquiredTargetSound`, `HidingSound`, `FleeSound`, ... 7 keys), BF2 `VOSound` (1037 values) | 36-38 / 46 | AI chatter |
| the squad-order keys (30 hashes: 15 `SC*` + 15 `AISC*`) | 36 each / - | squad talk (cards SQACK, USND) |
| `JumpSound`, `LandSound`, `RollSound`, `SquatSound`, `ProneSound`, `StandSound` (per WEAPON: `*_mvt_*`) | 31 each / 93-100 | none heard: the foley table's sounds played instead [L]; when these play is open [H] |
| `EngineSound` | 20 / 25 | engines |
| `VehicleCollisionSound`, `TurretYaw/Pitch/ActivateSound`, `FootstepSound0-5` | 23, 10, 10, 10, 6 / 45, 66, 66, 67, 6 | vehicles |
| `PowerupSound`, `ChargeSound`, `FireLoopSound`, `CollisionOtherSound`, `AmbientSound`, `LowHealthSound` | 7, 13, 7, 20, 6, 8 / 12, 18, 7, 4, 4, 1 | pickup, charge, beam loop, bounce, droid idle, heartbeat |
| command post keys 0xF17A02E6 / 0x16E4A90E / 0x8E39006A / 0x12CAACAA (`com_blg_commandpost_capture` / `_discharge` / `_lost` / `_dispute`) | 2 / 3 | command post (charge + discharge heard) |

## 5. Gaps -> cards

| gap | PS2 starts per 105 s (BF1 / BF2) | card |
|---|---|---|
| distances, gain and rolloff from the CORE templates | every 3-D sound | **SNDT** (new, M1 5, build) |
| soldier foley | 237 / 75 one-shots, the player's own every 0.3 s while moving | **FOLY** (new, M1 5, re) |
| vehicle engines, vehicle / turret weapons, walker steps, turret whir, boarding, collision | 1092 + 338 audible loops / 493 + 232 | **VSND** (new, M2 4, re) |
| explosions (layered), flight hum, bounce, weapon change, empty, charge / loop; the reload count | 1231 / 626 | **WSND** (new, M1 4, re) |
| music, ambience streams, command post and prop loops, pickups, menu sounds | 6 stream starts + 46 / 9 + 73 | **AMBM** (new, M1 4, re) |
| hurt / death voice, AI chatter | 154 + 62 stream lines / 270 + 0 | USND (note added) |
| grenade bounce + explosion sound | 36 / 89 bounces | GREN (note added) |
| pickup sound | 3 / 10 | PWUP (note added) |

Five new cards against the card rule's two: each is value >= 4 (M-ALPHA's "sounds like a game", the maintainer's INBOX
2026-09-29) and they do not share files. SNDT first: it changes what every other card's sounds are heard as.

## 6. Reproduction

```
cd engine/artifacts
python3 pcsx2/pcsx2_session.py start <ROOT>/iso/SWBF1.iso --name s --state pcsx2/ref/states/bf1_nab1_cp1_cp2_postA_spawn.p2s --boot-wait 12 --renderer sw
python3 lead_tmp/sndc/snd_live.py watch --game bf1 --planet NAB --name s --seconds 105 --out run.csv --pad "2:w:4000,8:k:200,12:e:3000,..."
cd lead_tmp; python3 sndc/census.py bf1 NAB run.csv > census.csv; python3 sndc/tables.py bf1 census.csv sndc/port/bf1_nab1_sounds.csv
python3 sndc/tab.py run.csv bf1 NAB near        # the pad check of section 1.3
python3 sndc/snd_live.py state <any .p2s> --game bf2 --planet UTA
```
Port: apply `lead_tmp/sndc/port/sound_log.patch` in a scratch worktree, build `game`, then
`SOUND_LOG=out.csv game --map assets_baked/bf1_nab1 --mode con --mission missions/bf1_nab1_con_cw.ron --headless --walk --walk-replay lead_tmp/sndc/port/pad120.ron`.
