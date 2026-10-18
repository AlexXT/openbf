---
id: VSND
title: Vehicle sounds 1: a seat gun's shot plays the GUN's FireSound (today the rider's rifle), and the engine loop - the soundparameterized record baked, 1-4 looping layers per piloted vehicle with gain and pitch from the PS2's graphs (speed, acceleration, health, turn-on / turn-off time)
milestone: M2
value: 5
phase: build
status: done
deps:
files: engine/crates/mungefs/src/sound_props.rs,engine/crates/bake/src/sounds.rs,engine/crates/manifest/src/lib.rs,engine/crates/manifest/src/tests.rs,engine/crates/audio/src/lib.rs,engine/crates/game/src/sounds.rs,engine/crates/game/src/engine_sound.rs,engine/crates/game/src/engine_sound/tests.rs,engine/crates/game/src/lib.rs,engine/crates/game/src/foley.rs,engine/crates/game/src/vehicle_pads.rs,engine/crates/game/src/walk.rs,engine/crates/game/src/bolt_vis.rs,engine/crates/game/src/event_sounds.rs,engine/crates/game/src/main/windowed.rs,engine/crates/game/src/main/headless.rs,engine/crates/game/src/main/battle.rs,engine/crates/core/src/sim/state.rs,engine/crates/core/src/sim/step.rs,engine/crates/core/src/sim/guns.rs,engine/CRATES.md
declares: rebake=sounds
needs:
effort:
branch: vsnd
resume: landed 79eac425 2026-10-03
created: 2026-10-02
alpha: yes
---
## Build (after the RE, 2026-10-03: `analysis/re-vehicle-sounds.md`, Docs 3d2fa0f)
The RE is done. This card builds the two things a player hears most: the seat guns and the engine loop. Walker
steps, turret whir, boarding, collision and the creatures' cries are card VSN2 (they need this card's loop handle).

### Do
1. **Seat gun fire** (doc 0.9, 10): `Event::Shot` carries the weapon class that fired (`core/src/sim/guns.rs` 347
   has `w.class`; the infantry path its unit's class); `game/src/event_sounds.rs` plays THAT class's `FireSound`.
   Events are not hashed - prove the sim hash does not move.
2. **Bake** (doc 1): `mungefs::sound_props` reads `soundparameterized` (0x12EBD555: `Name`, `SoundLayered`, `Layer {
   Name, ParameterGraph(parameter, graph, target) }`) and `ParameterGraph` records (0xE657A914: `ControlPoint(x,
   y)`), per sub-level like the layered records; `bake sounds` writes them resolved - per engine: its layers in the
   LAYERED record's order, each {sound, delay, deviation, graphs: [(parameter, target, points)]}, the graph joined to
   its layer by the layer NAME (doc 2.1). Parameters and targets as names (`speed`, `acceleration`, `health`,
   `turnontime`, `turnofftime`, BF2 `proximity`; `Gain`, `Pitch`); anything else is kept and ignored.
3. **Audio**: a loop handle whose gain, rate and pan can be set every frame (`audio::Audio`; card WHUM needs the
   same handle - whichever of the two lands first builds it, the other reuses it).
4. **`game::engine_sound`** (presentation only, no sim state, nothing hashed): per vehicle class `EngineSound`,
   `TurnOffTime` (default 3.0); per vehicle the PS2's instance - on / turning off, timer, previous speed, filtered
   acceleration (doc 3, 4):
   - on while the pilot seat has an occupant; boarding -> timer 0, layers start after their delay; no pilot ->
     turning off, timer 0, stop when timer >= `TurnOffTime`; a pilot during the run-down -> turn on again from 0; a
     destroyed vehicle stops at once;
   - hover: speed = |forward . velocity| / S, S = `ForwardSpeed` when the throttle is > 0 else `ReverseSpeed`;
     acceleration A += ((speed - previous) / dt - A) x 0.01 per 30 Hz update; health = current / max;
   - walker: speed = |velocity| in m/s; acceleration = previous speed - speed / dt (the PS2's own slip, doc 4 -
     keep it: it is what picks the gain graph's left end, RAM in doc 6.7);
   - per layer: gain factor = product of its Gain graphs, pitch factor = product of its Pitch graphs; `turnontime`
     graphs get the timer while on, `turnofftime` graphs while turning off, the other of the two reads 1.0; gain =
     clamp(property gain x factor, 0, 1) x the level by distance, rate = max(0.1, property pitch x factor); graphs
     linear between points, flat outside.
   The 0.01 filter is per PS2 update (30 Hz): PROVISIONAL-labelled conversion if the caller's rate differs. The
   PS2's 39-voice limit is not copied; a layer beyond its `MuteDistance` is not played.
5. `SOUND_LOG` rows for the referee: `engine,<vehicle>,<record>,<layer>,gain factor,pitch factor` once a second per
   live engine, and the seat-gun fire rows under the gun's sound name.

### Acceptance (run and paste)
- **Seat guns**: `SOUND_LOG` on bf1_nab1 (the foley pad replay, 105 s): `cis_weap_hover_aat_cannon_fire` /
  `com_weap_turret_fire` (or whichever seat guns fired) each appear, their total = the number of seat-gun `Shot`
  events of the run, and no mounted unit's shot logs an infantry fire sound. Before: 0 vehicle weapon names.
- **Engine count**: each logged second, live engines = vehicles with an occupied pilot seat + those inside their
  run-down.
- **The PS2's own numbers through the port's code**: a referee that reads the BAKED nab1 engines and replays
  `lead_tmp/vsnd/live1.csv` + `live2.csv` with the fixed rule of `lead_tmp/vsnd/verify/strict_replay.py` (stored
  speed, acceleration, timer, flags -> Gain and Pitch factor per layer): >= 99 % of the 19,419 layer-ticks within
  0.002 (the disc graphs give 19,374). Plus unit tests with the AAT's graphs inline: at rest pitch 0.80 / 0.85 / 0.60
  and gain factor 0.7 / 0.6 / 0.8; full speed pitch 1.0; the filter 0.99^15 over 15 updates; turn-off stops at
  timer >= 3.0; a walker at 5.58 m/s -> gain factors 0.7 / 1.0 / 1.0 on Utapau's `atst_engine_parameterized`.
- **The port's own hover**: an AAT standing and at full forward speed in a headless run: logged pitch factor of
  layer 1 0.80 and 1.00.
- Mutation-check one test (break the product into a sum, see it fail).

### Not in this card
Card VSN2: walker steps and `HydraulicSound`, turret whir, `TurretActivateSound` / `Deactivate`,
`VehicleCollisionSound`, `TurnOnSound` / `TurnOffSound` (kaadu, tauntaun). Note on USND: the jet pack's and the
droideka's engine ride the same component (doc 4). Flyers (take-off, `BoostSound`, BF2's proximity and cockpit
chatter): post-alpha.

## Why
SNDC (`analysis/audio-census.md`): the port plays no vehicle sound at all. PS2, 105 s: NAB1 970 vehicle / turret
weapon starts, 338 audible engine-layer starts, 88 kaadu steps, boarding and collision; UTA1 172 weapon starts, 232
engine layers (AT-RT), 269 walker steps (`com_weap_layered_at_step`, `atst_leg_up`), turret whir.

## Evidence
- Engines: `EngineSound = <x>_engine_parameterized` (20 classes BF1, 25 BF2) is a record of kind 0x12EBD555 that
  names a `soundlayered` of 2-4 looping properties (AAT: 0x842C3A54, 0xE0539C6C, 0x5ACF7D70; templates 25 / 45 -
  450 m). On the PS2 they play in the 39 handle voices only (B1 `*(0x5220c4)`, B2 `*(0x60b654)`, stride 0x150) -
  `snd_live.py state lead_tmp/sqveh/nab1_cis_aat_pilot.p2s --game bf1 --planet NAB` shows the player's AAT's three
  layers. How the parameter (speed? throttle?) drives pitch and the layer mix is not read [H].
- Weapons: `cis_weap_hover_aat_cannon_fire` 434 starts, `com_weap_turret_fire` 364, `imp_weap_hover_fightertank_*`,
  the beam's `FireLoopSound`. The port's log has no vehicle weapon name in 120 s (376 `vehicle_hit` events).
- Other keys: `FootstepSound0-5` (walkers, kaadu), `TurretYawSound` / `TurretPitchSound` (`turret_whir_*_lp`),
  `TurretActivateSound` / 0x1ADA38C1 (`vehicle_equip`, heard on boarding), `VehicleCollisionSound`, 0x069134E5
  (`atst_leg_up`), 0xEA603E52 / 0x042DABAE (droideka fold / unfold, jet pack on / off).

## Do (re, then build)
RE the parameterized record (B1 loader `FUN_003579a0` / create `FUN_00356bd8` for kind 0x12EBD555) and what feeds it;
which update starts walker footsteps. Build: the vehicle weapons' `FireSound` first (known behaviour, no RE), then
engines as looping voices that follow the vehicle, steps, turret whir, boarding, collision.

## Acceptance
- SOUND_LOG referee on bf1_nab1 (`pad120.ron`): `cis_weap_hover_aat_cannon_fire` and `com_weap_turret_fire` starts
  within 2x of the PS2's per-shot count for the port's own vehicle shots (one sound per vehicle `Shot`), and one
  engine loop alive per occupied vehicle (count = occupied vehicles each second of the log).
- A PS2 pair for the engine parameter: `snd_live.py` on the AAT state, standing vs full speed, the three layers'
  pitch / gain fields read from the handle voices, against the port's values for the same speeds.

- Planner 2026-10-02 14:45: value 4 -> 5. M2's milestone is vehicles and every vehicle is silent (PS2: 1,092 starts + 338 engine layers in 105 s on NAB1); WAI2, M2's other value-5 card, now waits for PARR. Bound the RE to the engine loop (the parameterized record and its 39-voice set) and seat / turret weapon fire first. Undo: `loopctl.py set VSND value=4`.

- Planner 2026-10-02 14:45: `deps: AIMA` is an ORDERING dep, not a technical one. Lane 2 prefers an RE card at equal value; without it lane 2 would leave the builds (GREN, PARR) for this RE the moment UWPN lands, while lane 3 (the research lane) is on AIMA. With it the card opens when AIMA's RE is done and the research lane is free. Undo: `loopctl.py set VSND deps=`.
- AIMA's RE session 2026-10-02 16:25: the ordering dep is cleared - AIMA's RE is done (Docs 2aac586) and the card went on to build, so `deps: AIMA` would now have held this RE behind a build. Undo: `loopctl.py set VSND deps=AIMA`.

- Note from WSND's RE (2026-10-02, `analysis/re-weapon-sounds.md` 1-3): a `soundparameterized` names a `soundlayered`
  and attaches its graphs to the layers BY LAYER NAME (the first value of a `Layer` entry: B1 `FUN_0045b958` <-
  `FUN_00460978` 523889). Its player holds one voice per layer and starts each when the layer's delay has passed (B1
  `FUN_0045c700` 520974-521000, the deviation redrawn every update; B2 `FUN_0054f5f8` draws start and stop once,
  683836-683848). A looping sound on a handle that got no voice (out of range, no voice free) is asked for again on
  every update (`FUN_00355a28` 343230-343239) - an engine comes in when the vehicle comes into range. WSND bakes the
  layers as (sound, delay, dev) and drops the name: keep the name in the bake if this card needs it.
