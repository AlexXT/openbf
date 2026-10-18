---
id: BMAP2
title: The minimap's other views and missing icons: Select toggles Small / Large (the whole map at 0.948, numbered posts, alpha 0.25) and the spawn screen shows the Spawn view with the PostSelect ring; the shield generator's hud_dest_icon post on HOT1; the yellow direction mesh in a vehicle / turret; Lua markers
milestone: M3
value: 4
phase: build
status: todo
deps: BMAP
files: engine/crates/ui/src/hud_map.rs,engine/crates/game/src/minimap.rs,engine/crates/ui/src/hud.rs,engine/crates/game/src/main/battle.rs,engine/crates/game/src/main/windowed.rs
declares:
needs:
effort:
branch:
resume:
created: 2026-10-03
alpha: yes
---
## Why
BMAP (4c8bc60) drew the Map element's Small view only. analysis/re-hud-map.md sections 2.2, 3, 4 and 7 hold the rest, already RE'd and checked on PS2 frames.
- Large view on Select (FUN_003a8d18): no heading rotation, the mission's north angle, PositionLarge, the mapping x 0.948 about the screen centre (PROVISIONAL, measured), backdrop alpha 0.25, hud_large_map_fov x 0.95, north pointer scale 5.
- Spawn view on the spawn screen (dead): PositionSpawn, backdrop 0.65 x 448, the selected post's hud_icon_flash ring at MapScale x 2.
- HOT1's PS2 post 5 is the shield generator building (hud_dest_icon 3.0 at (-423.5, 204.9)); the port has no map icon for it (game/tests/hud_layout.rs says so). Find how a destructible objective registers a post (refreshPost) and feed it.
- PlayerDirection mesh (yellow, alpha 0.5) when a vehicle / turret aim is 15 deg or more off the camera.
- The post number's font and drop are PROVISIONAL ([H] gamefont_tiny, +4 px); the wedge lines look brighter than the PS2's.
The Target element's world markers stay with card TMRK.

## Acceptance (measurable)
- Large view: a 640x480 port layout at lead_tmp/re/bmap/cap/hot1_large.p2s and nab2_large.p2s puts the posts within 3 px of the frames' (re-hud-map 2.2: 10 posts, max residual 1.66 px with the model).
- Spawn view: NAB2 spawn map posts p1 (194.0, 326.1), p2 (91.1, 273.8), p4 (194.3, 229.2) within 3 px.
- HOT1 postA spawn: the shield generator's icon within 3 px of (608.0, 84.9).

## Note (planner 2026-10-03 07:15, from BMAP's report) - first step: one line in the RE note
BMAP found that `analysis/re-hud-map.md` 2.3's conversion line is wrong (`reports/20261003_0530_card_BMAP.md`,
"Finding"): the note says the bake stores diag(-1,1,-1) x RAM; BMAP measured that a `mapbounds` box stored with the
plain diag(-1,1,-1) (within 0.01) is held in RAM as an axis-aligned box with the identity, and any other (TAN1's
2.3 deg tilt) exactly as stored - RAM's centre and R on 16 of 16 BF2 maps, where the note's rule gives +137.3 on
TAN1. It is in code as `minimap::bounds_of` (PROVISIONAL: the threshold is measured, not read) and the note was left
unedited. Before building the Large / Spawn views on it: re-run that 16-map check, then correct 2.3 with a dated
line and commit it (`loopctl.py commit-docs`). The planner did not edit the note: it had not re-measured the claim.
