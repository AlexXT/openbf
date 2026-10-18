---
id: AMBM
title: Music: the ambient music's start / middle / end by reinforcements (BF1: team 2's state; BF2: the less advanced team), Music records (fade, min playback, min inactive time, takes), victory / defeat at battle end, a streaming player for the stereo music streams
milestone: M1
value: 4
phase: build
status: done
deps:
files: engine/crates/mungefs/src/sound_props.rs,engine/crates/bake/src/sounds.rs,engine/crates/manifest/src/lib.rs,engine/crates/core/src/music.rs,engine/crates/core/src/lib.rs,engine/crates/game/src/music.rs,engine/crates/game/src/lib.rs,engine/crates/game/src/sounds.rs,engine/crates/game/src/mission.rs,engine/crates/audio/src/lib.rs,engine/crates/audio/src/stream.rs,engine/crates/game/src/main/windowed.rs,scripts/gen_missions.py,engine/CRATES.md
declares: rebake=sounds
needs:
effort:
branch: ambm
resume: landed 1bde70c8 2026-10-03
created: 2026-10-02
alpha: yes
---
## Build (after the RE, 2026-10-03: `analysis/re-ambience-music.md`, Docs 8a35a92) - MUSIC only
The RE split this card: ambience emitters -> card AMBS, command post sounds -> CPSN, interface sounds -> UISN.
What is left here is the music.

### Why
The port's battle has no music. On the PS2 the listener team's ambient music plays from the first spawn, changes
with the reinforcements and ends in the victory or defeat music.

### What the PS2 does (doc section 3)
- `SetAmbientMusic(team, x, name, state, 1)`: threshold = (int)(x x the team's reinforcements at the call) and a
  `Music` record per team and state (0 start, 1 middle, 2 end). Per map: Naboo 1.0 / 0.99 / 0.1, Utapau 1.0 / 0.8 /
  0.2 (the first text of this card had Naboo's for both).
- Per-team state: 2 if T <= thr[2], 1 if T <= thr[1], 0 if T <= thr[0], else none. **BF1: the state of team 2**
  (team 1's only when team 2 has none); **BF2: the minimum over the teams that have one**, 0 when none.
- Every frame once a human has spawned and until game over: ask Music[listener team][state]; stop both teams'
  Music of state - 1; stop the old listener's when the listener team changes.
- A `Music` (chunk `mus_` of the SOUND LVL): `SoundStream`, `FadeInTime`, `FadeOutTime`, `MinPlaybackTime`,
  `MinInactiveTime`, BF2 `MaxPlaybackTime`, `Priority`. One Music plays at a time (the asked one with the highest
  Priority whose inactive time has passed). Start: fade in. A stop request waits for `MinPlaybackTime`, then fades
  out over `FadeOutTime`. After a stop it cannot start for `MinInactiveTime` (the first start is not delayed). A
  BF1 take (Looping 0) ends by itself and the next one in list order follows after the inactive time; a BF2 take
  loops until `MaxPlaybackTime`.
- Battle end: bus `soundfx` faded to 0, all sounds stopped, the listener team's `SetVictoryMusic` or
  `SetDefeatMusic` Music.

### Do
`mus_` records and the music stream properties into the bake; the script's music calls into the mission RON
(`scripts/gen_missions.py`); a pure music module (`core`: the two state rules, the Music state machine, the
manager) fed per frame by the game; a streaming voice for the stereo music streams (the maintainer: music stays streamed -
the bake indexes them, `sounds/streams/streams.ron` and `audio/global_bf{1,2}/global.ron`; the runtime decodes the
PS2 ADPCM blocks from the disc's LVL). PROVISIONAL in code: BF1's rule (no PS2 state separates it from BF2's;
`re-ambience-music.md` 11 names the live check).

### Acceptance
- Rule test against PS2 RAM: a fixture dumped by `engine/artifacts/lead_tmp/ambm/tools/ram_music.py --verbose`
  (reinforcements, thresholds, asked state of 105 BF1 + 122 BF2 states): the port's rule gives the PS2's state in
  every row, including `bf2_yav1_cp3_landingzone_post1_spawn` (134 / 119 against 150 / 120 / 30 -> start).
- Timing test, BF1 Naboo `cis_nab_amb_middle` (takes 26.12 / 31.29 / 28.30 / 26.66 s, MinInactiveTime 30): the
  model's take starts land on the PS2's (176.61 -> 233.02 -> 294.59 s; tolerance 0.5 s) and follow the list order.
- A headless log line per music change on bf1_nab1 and bf2_uta1 (`start` at the first spawn, the next state when
  the game's rule says so, victory / defeat for the listener's team at the end), with a mutation check: swap the
  two rules and see the bf2_yav1 row fail.
- The music is audible in the windowed game (a manual listen is NOT the acceptance; the log and the tests are).

---
(the card's original text, kept)
## Why
SNDC (`analysis/audio-census.md`): the port's battle has no music, no ambience and no interface sound. PS2: the
ambient music plays from the first second (`cis_nab_amb_middle`; UTA1 `cis_uta_amb_start` -> `_middle` -> `_end`,
then `cis_off_victory_im`), one or two ambience streams run all battle, every command post hums, and every menu move
clicks.

## Evidence (live, `lead_tmp/sndc/`)
- Music: stream slot 0 (BF1) / 2 (BF2); `SetAmbientMusic(team, fraction, name, state)` start / middle / end at ticket
  fraction 1.0 / 0.99 / 0.1 and the Victory / Defeat music are specified in `analysis/re-announcer.md` 2.5 and section
  1 (the VoiceOverManager owns them). The stereo music streams are indexed and not decoded by the bake (the maintainer's
  decision: music stays streamed) - the runtime needs a streaming decoder for them.
- Ambience streams: BF1 NAB1 slot 3, property 0xEB51145B on stream `nab1`; BF2 UTA1 slots 3 and 4, 0x4FEEB00F and
  0x7C8CA2CF (unnamed `<map>_emt` properties; the bake has their OGGs under `sounds/streams/`). What starts them (a
  sound region, the mission script) is not read [H].
- World loops (sample voices): `com_blg_commandpost_goodie` / `_baddie` (a post's hum by friend / foe, 99 audible
  starts NAB1), `com_blg_commandpost_charge` / `_discharge` on a capture (keys 0xF17A02E6, 0x16E4A90E, 0x8E39006A,
  0x12CAACAA also name `_capture`, `_lost`, `_dispute`), prop emitters (`windmill`, `neon_buzz`, `com_amb_fire`),
  `AmbientSound` of the droids (`droid_fx7_idle_chatter`, `com_item_gonkrecharge`).
- Interface (CORE.LVL bank, 2-D): `shell_select_change` on every pause-menu and spawn-map move, `shell_menu_accept`
  on accept, `shell_menu_exit`, `shell_select_unit` on the class pick.

## Do (re, then build; split if it does not fit one session)
RE: what starts the ambience streams and the prop emitters (the map's sound regions / `snd_` emitters), the command
post's sound states. Build: the music stream player + `SetAmbientMusic` states + victory / defeat; ambience streams;
post hum and capture sounds; the four interface sounds in the pause menu and the spawn screen.

## Acceptance
- A headless log line per music state change on bf2_uta1 (`start` at spawn, `middle` when a team's tickets pass 0.99,
  `end` at 0.1, victory / defeat for the listener's team), checked against the PS2 order above.
- SOUND_LOG referee: one hum loop per command post in range, friend / foe by owner; `shell_select_change` /
  `shell_menu_accept` counts equal to the menu moves / accepts of a scripted pause-menu input.
- The ambience streams' start condition from RE, and the port starting the same properties on NAB1 and UTA1.
## Note from SNDT (2026-10-02)
- Sound spaces scale 3-D sounds: when a sound's space is not the listener's, the PS2 multiplies its level by the mean
  of that space's `obstructionhfgain` / `occlusionhfgain` (B1 `FUN_004599e0` 518970-518972, `FUN_0046a2f8`,
  `FUN_0044d698`; the default space holds 0.5 / 0.5 or 0.1 / 0.1 in 38 BF1 states, 0-9 spaces per map).
  Not ported; `analysis/re-sound-distance.md` claim 7. Distances and gains are now the disc's
  (`game::sounds::attenuation`); referee `lead_tmp/sndt/referee.py <SOUND_LOG csv> <map dir>`.

- Note from WSND's RE (2026-10-02, `analysis/re-weapon-sounds.md` 4.3): the prop class's `SoundProperty` (B1
  `FUN_001fb6e8`, B2 `FUN_0022f6e8` -> class+0x304) is a sound on the object's own handle at the object's position,
  started and updated by B2 `FUN_0022ef70` (145627-145628) unless bit 2 of class+0x330; BF2 adds `StartMovementSound`
  (+0x30c) / `StopMovementSound` (+0x314) one-shots. A looping sound on a handle is retried every update until it is
  in range (3.4). Not the explosion class's key of the same name.
