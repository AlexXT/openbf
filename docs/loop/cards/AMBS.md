---
id: AMBS
title: Ambience: the world's soundstream regions, sound objects and prop loops - each map's ambience streams (looping 3-D at the region's centre, distances from the region's size), com_snd_amb_static objects (Mygeeto's neon, Tatooine's sandcrawler), props with SoundProperty (fires, vaporators) and effect sounds
milestone: M1
value: 4
phase: build
status: done
deps:
files: engine/crates/game/src/ambience.rs, engine/crates/game/src/sounds.rs, engine/crates/game/src/event_sounds.rs, engine/crates/game/src/lib.rs, engine/crates/game/src/main/windowed.rs, engine/crates/game/src/main/headless.rs, engine/crates/game/tests/ambience_ps2.rs
declares:
needs:
effort:
branch: ambs
resume: landed 408a8d35 2026-10-16
created: 2026-10-03
alpha: yes
---
## Why
The port's battle has no ambience. On the PS2 every map has one or more ambience streams all battle long and fixed
loops on props and world objects (fires, vaporators, neon signs, control panels). RE: `analysis/re-ambience-music.md`
sections 1-2 (8a35a92).

## Evidence
- A world region named `soundstream <stream property> <divisor>` is a looping 3-D stream sound at the region's
  centre, MinDistance = |SIZE| / divisor, MaxDistance = MuteDistance = |SIZE|, started once at battle start (BF2
  restarts a stopped one every update). The level by distance is `analysis/re-sound-distance.md`'s law with those
  three distances replacing the property's own.
- World objects of class `com_snd_amb_static` carry `Sound`, `MinDistance`, `MaxDistance` (MuteDistance = Max);
  one `com_snd_amb_streaming` object exists (YAV1 `waterfall`).
- A prop class with `SoundProperty` loops that sound at the prop with the sound's OWN distances: BF1 from battle
  start, BF2 from the first time the prop is drawn, at prop matrix x `SoundOffset`. Never stopped. A non-looping
  sound is re-asked whenever its handle is idle (its `PlayInterval` / `PlayProbability` pace it).
- An effect's `SoundName` plays once on the emitter (a looping one follows it): Naboo's `com_amb_fire`.
- Tools: `engine/artifacts/lead_tmp/ambm/tools/` (`regions.py`, `region_census.py`, `snd_objects.py`,
  `ram_regions.py`).

## Do
Bake a per-map emitter list (regions, sound objects, props with `SoundProperty`: position, sound, kind, the
distance overrides); play each as a looping voice whose gain is the PS2 level at the listener; the stream ones
from the baked OGGs under `sounds/streams/`. Effects' `SoundName` on their emitter.
Not in this card: `soundspace` occlusion, `soundtrigger` (BF1 Hoth, BF2 space), `BuildingSound` (silent on the
PS2), BF2 `StartMovementSound` / `StopMovementSound` / `SoundWhenMoving` (Death Star bridge only).

## Acceptance
- The baked emitter list of each showcase map equals the PS2's list in RAM (`ram_regions.py <state>`), name and
  distances to 0.01 m: BF1 NAB1 1 stream (`nab_amb_meadow` 420.97 / 420.97); BF1 TAT1 2 streams (`tat_amb_desertwind`
  601.54, `tat_amb_canyon` 72.48 / 144.97) + `sandcrawler` 50 / 400 + 11 `vaporatorhum` + 5 `com_amb_fire` + 1
  `sarlacc_amb`; BF1 YAV2 1 stream; BF2 UTA1 3 streams (235.70 / 471.40, 145.95 / 189.74, 126.11 / 163.94); BF2 MYG1
  1 stream + 37 `neon_buzz_low` (23 x 2.0 / 30, 14 x 1.5 / 30); BF2 HOT1 5 streams. A test prints both lists and
  fails on a difference.
- SOUND_LOG on a headless walk of NAB1 and UTA1: the ambience loops are listed from tick 0 with the level the law
  gives at the listener (UTA1's wind: 1 inside 235.7 m, linear to 0 at 471.4 m - a unit test with three distances).
- Mutation check: drop the divisor (min = max) and see the UTA1 level test fail.
- 2026-10-06 planner: `engine/CRATES.md` taken out of `files:`. The picker refuses a card whose files overlap a card in progress, and 22 open cards listed the map, so any one of them held the other 21 (a chain card in progress held WADE and WTHR). The map's row for a new or moved source file is still owed in the same branch (`scripts/size_report.sh --check-map`); a rebase conflict in it is one line. Undo: add the path back.
