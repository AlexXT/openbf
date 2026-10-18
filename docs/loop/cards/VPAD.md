---
id: VPAD
title: Vehicle spawn pads: the class per team side (Atk = team 1, Def = team 2), following the CP's owner, SpawnTime respawn, one vehicle per pad (BF2 SpawnCount), the idle decay
milestone: M2
value: 4
phase: build
status: done
deps: VENT,VCLS,VTEAM
files: engine/crates/core/src/vehicle_pad.rs,engine/crates/core/src/sim/vehicles.rs,engine/crates/game/src/vehicle_pads.rs
declares: sweep
needs:
branch: vpad
resume: landed 896b924 2026-09-26; report reports/20260926_1414_build_VPAD.md; next HOVR moves the pad vehicles
created: 2026-09-25
---

## Why
Vehicles come only from pads: 336 `com_item_vehicle_spawn` instances on 25 maps, 53 classes.

## Evidence
`analysis/re-vehicles.md` §7 [O]: defaults (SpawnTime 10, DecayTime 10, idle thresholds 10 / 20 s, Team −1, first spawn
immediate); the side table (7 rows × Class<SideName> / Class<Side>Atk / Class<Side>Def; Atk = team 1, Def = team 2 of
that side); follow the ControlZone CP's team, a change resets the countdown to SpawnTime; one vehicle per pad (BF1),
`SpawnCount` (BF2); decay: idle and no player within r + r + 10 m for 20 s (CP lost / outside its area) or 10 s (the CP's
side now maps to another class), then MaxHealth / DecayTime damage per second; ExpireTimeEnemy/Field are never read. §1:
the per-map census; the port's GCW / CW missions.

## Acceptance
`core` pads built from the manifest instances + ClassDb; a census test over every baked map: each pad resolves a class
per team for the map's missions (report `gun_walk_kaadu` and `re_hover_barcspeeder` as absent, as the PS2 would spawn
nothing); spawn / respawn / CP-follow / decay unit tests on the §7 numbers; vehicles spawn in a TAT1 battle (hover
classes drive through HOVR, others idle until their integrator lands — declare `sweep` / `sim` moves).

**Note (VBAKE, 2026-09-26):** side vehicles are baked to `side_<game>/vehicles/<side>/` (`vehicles.ron`). 13 chunk / LOD
models name a texture only ANOTHER side LVL ships (e.g. BF1 REP speeder-bike chunks → `imp_hover_speederbike`); the
texture lookup for a drawn vehicle must search every loaded side (and common), as the PS2's global namespace does
(analysis/re-vehicles.md §10 item 3).
