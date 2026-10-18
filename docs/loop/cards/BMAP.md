---
id: BMAP
title: The BF2 minimap and world markers: the Map element (the map's <world>_map masked by map_mask, FOV wedge, posts, targets, north pointer; small / large / spawn views) and the Target element (objective, flag, attacker, lock-on markers on / behind / off screen) — RE their placement and scale, then draw them in BHUD2's HUD
milestone: M3
value: 5
phase: build
status: done
deps: BHUD2
files: engine/crates/ui/src/hud_map.rs,engine/crates/ui/src/hud_draw.rs,engine/crates/ui/src/hud_tree.rs,engine/crates/ui/src/hud.rs,engine/crates/game/src/minimap.rs,engine/crates/game/src/hud_feed.rs,engine/crates/game/src/main/battle.rs,engine/crates/game/src/walk.rs,engine/CRATES.md
declares: goldens=tat1_hud.png
needs:
effort:
branch: bmap
resume: landed 4c8bc606 2026-10-03
created: 2026-09-30
alpha: yes
---
## Why (RE done 2026-09-30, `analysis/re-hud-map.md`, commit 027dad9)
The PS2's minimap and world markers are now known and checked: heading-up, eye-centred, radius from the controlled
class's MapViewMin/Max × MapSpeedMin/Max (soldiers ±50 m), the `<world>_map` image over the `mapbounds` square (else the
posts' box + 100 m — BF1's rule), SetMapNorthAngle on the big maps, posts / units / Lua markers / FOV wedge / north
pointer, and the Target element's marker styles, off-screen clamp and distance scale (§2–§5). Predicted from RAM, 263 of
264 post icons land within ±3 px on 16 maps. The port has none of it (egui placeholder, no minimap).

## Do (build, after BHUD2's interpreter lands)
- Feed (re-hud-map §8): camera eye + forward; the controlled entity's class MapViewMin/Max, MapSpeedMin/Max (defaults
  125 / 250 / 10 / 100) and horizontal speed; every CP (position, owner, capturing team, number, entity-class MapTexture
  / MapScale — class RON props 2646838656 / 244072421); units (position, heading, team, MapTexture / MapScale, the
  detected rule of §3 as PROVISIONAL: friendly always, enemies within 15 m, or 60 m inside ±24° of the view, or firing).
- Map extent: the `mapbounds` region from the manifest (convert its rotation: the bake stores diag(−1,1,−1)·RAM — HOT1
  identity in RAM; signed halves, max picks the larger signed value, so TAN1 gets −89.43), else the CP box + 100 m (BF1).
  North angle per mission (bake SetMapNorthAngle from the mission script, else §2.2's table: COR1-GCW / NAB2 180° + flag,
  TAN1 180°).
- Draw (Small / Large / Spawn): backdrop = the map image's sub-rect (§2.3 formula) masked by `map_mask` at α 0.5 × 0.85,
  turned by −heading in Small; posts 6 px × MapScale in team colours + the 1 Hz capture blink, clamped to the rim, the
  number fading with 1 − 4d²; units hidden outside the circle; FOV mesh `hud_map_fov` ×1.15; north pointer on the image's
  −Z edge; Large = the mapping × 0.948 about the screen centre (PROVISIONAL, measured). Target markers per re-hud-map §5
  (styles by on-screen / LOS / kind, ±0.9 NDC clamp, MinScale→MaxScale) fed by the port's own events until TMRK says
  which entities the PS2 marks.
- BF1 maps: `<terrain>_map`, the CP-box rule, north 0.
- Hor+: the minimap anchors top-right (COMPARISON_RULES); keep BHUD2's stretch decision for the ellipse.

## Acceptance (measurable, build)
- A 640×480 port render at `bf2_hot1_cp1_cp6_postA_spawn.p2s`'s pose and battle state (eye (1.484, −10.097), θ 150.1°):
  post icons within ±3 px of re-hud-map §7 (p1 (534.4, 79.8), p2 (606.7, 70.7), p5 (608.0, 84.9)); the same at HOT1 walk
  cp1 b000 (p2 (510.9, 116.2), p5 (502.2, 104.4)).
- TAN1 walk cp0: the backdrop image mirrored like the PS2's (the §2.3 NCC check ≥ 0.8 against the PS2 frame's lower disc).
- `tat1_hud.png` moves (declared, viewed); a 16:9 render keeps the minimap round and top-right.

## Original text (2026-09-30)
analysis/re-hud.md §3: the 1-player HUD's minimap (Map player1map at (0.856, 0.186), Scale 0.9) and its world markers
(Target player1target) are data in 1playerhud, but their DRAW is code the BHUD RE did not trace: how the <world>_map
image maps to world X/Z (bounds, zoom, whether it turns with the player), which icon and colour a command post gets per
team and while contested, how PositionLarge / PositionSpawn switch, and how Target markers scale with distance
(MinScale / MaxScale). The 1-player HUD shows CP capture only here (no commandPost bar outside 4playerhud).

## Do
- RE the Map element (FUN_003a2d20 and its methods, the refreshPost / refreshTarget / refreshMarker events, the %s_map
  bind in FUN_003a7af0) and the Target element (FUN_003acae8); PCSX2: the minimap at two poses of one map (does it rotate?)
  and a contested post.
- Then a build (after BHUD2): draw both in the HUD interpreter; BF1 maps use their own *_map (naboo1_map …).

## Acceptance (measurable)
- RE: analysis/re-hud-map.md with the world→minimap transform checked at 2 PCSX2 poses (icon positions within ±3 px of
  the PS2 frame's).
- Build: the port's minimap at HOT1 postA v1 places the CP icons within ±3 px of the PS2 frame's.

## Notes

- CWSP RE (2026-10-02, `analysis/re-command-walker-post.md` section 5): a command walker's post has a map icon of its
  own — BF2 `FUN_003a6a48` draws each registered post at its OWNER's live position in the owner's team colour with
  the owner class's `MapTexture` / `MapScale` (AT-AT `atat_icon` 3.5, AT-TE `atte_icon` 2.2), rotated by the owner's
  heading (a fixed post's is not rotated), and prints the post's number (`HUDIndex`, "%d") only when `HUDIndexDisplay`
  is set — the AT-AT / AT-TE author 0, so no number. Activating the post removes the walker's ordinary vehicle marker.
  Frames: `lead_tmp/cwsp/shots/bf2_hot1_spawnmap.png` (two blue AT-AT icons, no number), `bf1_hot1_spawnmap.png`.

## Planner 2026-10-03 03:35 - value 4 -> 5; `files:` cleaned
The alpha's definition names "the real BF2 HUD"; the minimap is its largest missing element and is on screen in every
frame of a battle. The RE is done (027dad9) and BHUD2 has landed, so nothing holds the build.
`files:` had notes in brackets inside the list ("battle.rs (feed)"), which the picker compares as whole strings: it
would not have seen the overlap with SQHUD (`ui/src/hud.rs`, `game/src/main/battle.rs`). Plain paths now.
Undo: `loopctl.py set BMAP value=4`.
