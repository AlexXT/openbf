---
id: VCLS
title: Vehicle class data: bake each map's own entc PROPS (TAT1's skiff, BES1's cloud car) and read hover and seat props into typed structs with the PS2's constructor defaults
milestone: M2
value: 5
phase: build
status: done
deps:
files: engine/crates/bake/src/map.rs,engine/crates/game/src/classdb.rs,engine/crates/game/src/vehicle_class.rs,engine/crates/game/tests/vehicle_class.rs
declares: rebake=tat1,tat2,maps
needs:
branch: vcls
resume:
created: 2026-09-25
---

## Why
A vehicle's drive and seat numbers live in its class props. Side classes are baked (hashes → strings) but nothing turns
them into typed numbers, and MAP-LOCAL classes (TAT1's `tat_hover_skiff`, BES1's `bes_fly_cloudcar`) are not baked at
all — a map bake keeps only a class's geometry and collision (`ManifestClass`).

## Evidence
`analysis/re-vehicles.md` §3.1 (the hover props, their offsets and the ctor defaults, identical in both games), §5 (the
FLYERSECTION seat ring: BODY = pilot, TURRET1..8 = gunner sections, PassengerSlots ≤ 4; PilotType none/self/vehicle/
remote(/vehicleself) = 0..3(4), default 2 in BF2), §10 item 2. `bake classes` already parses any LVL
(`bake/src/classes.rs`).

## Acceptance
The map bake writes `<map>/classes.ron` (entc/wpnc/ordc/expc of the map LVL, the side-classes format) — REBAKE maps;
`game::classdb` loads it beside the side DBs; `HoverClass::from(&ClassDb, name)` returning every §3.1 field with the PS2
ctor default when absent (incl. up to 6 spring points with their per-point factors); a `SeatSpec` list from the section
props in ring order. Unit tests: the skiff (SetAltitude 1.0, GravityScale 4.0, three spring points, ForwardSpeed 16,
Traction 30, a BODY pilot seat + TURRET1..3 gunner seats with `com_weap_bldg_gunturret`) and `imp_hover_speederbike`
(StrafeSpeed 0, three spring points, one pilot seat with a weapon). Every gate PASS except the declared rebake (manifest
unchanged elsewhere).
