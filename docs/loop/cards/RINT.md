---
id: RINT
title: Render interpolation: the sim stays 30 Hz, the windowed game draws positions, the camera and animation blended between the last two ticks at the display's rate (the maintainer approved 10-20; after HITCH's recorder) - today a 60 / 144 Hz frame repeats the last tick, so motion steps
milestone: M1
value: 5
phase: build
status: todo
deps: HITCH
files: engine/crates/game/src/walk.rs, engine/crates/game/src/main/windowed.rs, engine/crates/game/src/soldier_vis.rs, engine/crates/game/src/viewer/vehicles.rs, engine/crates/game/src/bolt_vis.rs
declares:
needs:
effort:
branch:
resume:
created: 2026-10-20
alpha: yes
---
## Why
The maintainer, 2026-10-20 17:05 (INBOX, standing for PERF / HITCH / RINT): "(a) approved: RINT, the sim stays 30 Hz and the drawn
positions, camera and animation interpolate between the last two ticks at the display's rate (open the card after HITCH's
recorder)." Today the windowed game draws the latest 30 Hz tick as it is: at 60 / 144 Hz a frame repeats the last tick's
picture, so motion steps (no interpolation between ticks was found when HITCH was carded).

## Evidence
`game/src/walk.rs:804` `advance_with` (the fixed-step accumulator; `accum` left over after the ticks is the blend factor),
`game/src/main/windowed.rs:1443` (the frame loop), the draw-list builders that read unit / vehicle / bolt / camera state
(`soldier_vis.rs`, `viewer/vehicles.rs`, `bolt_vis.rs`, the camera). HITCH's recorder (Part 1) gives the frame split.

## What
Keep the previous tick's drawn state (unit and vehicle transforms, the camera, animation clip times, bolts' heads) and draw
`lerp(prev, cur, accum / TICK)` (slerp for rotations; a teleport, respawn or seat change snaps, no blend across it). The
sim never reads the blend: it is presentation only. The player's own camera turn keeps its input latency (the look input
applied at the display's rate if the PS2-measured feel allows; FEEL_NOTES line either way).

## Acceptance
- Sim hash unchanged (`game --sim-hash 100`, `scripts/sim_gates.sh`): nothing in core changes.
- A unit test: at accum = TICK/2 a unit's drawn position is the midpoint of its two ticks; a respawned unit is drawn at its
  new spot with no blend (mutation-checked).
- HITCH's recorder on a 144 Hz no-vsync windowed run (KAS2, 1 min, camera following a running soldier): the frames whose
  drawn camera position did not move while the player moved go from ~ (1 - 30/fps) of frames to 0; pasted before / after.
- Goldens unchanged (they render one tick, blend 1.0) or declared.

## Notes
Opened by the planner 2026-10-20 17:50 on the maintainer's approval; waits on HITCH (deps) so its recorder can measure it.