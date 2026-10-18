---
id: AIMA
title: The PS2's default shooting angle: the shot leaves above the camera axis (RETC measured the reticle ~5 degrees above the view centre), so the camera can look down without the bullets going into the ground - RE the aim ray, then match it in the sim and put the HUD reticle at that height
milestone: M3
value: 5
phase: build
status: done
deps:
files: engine/crates/core/src/sim/input.rs,engine/crates/core/src/sim/step.rs,engine/crates/core/src/unit.rs,engine/crates/game/src/walk.rs,engine/crates/ui/src/hud.rs,engine/crates/game/src/main/battle.rs,engine/crates/game/src/bolt_vis.rs
declares: sim; arena; goldens=tat1_hud.png
needs:
effort:
branch: aima
resume: landed 24b8acd0 2026-10-02
created: 2026-10-02
alpha: yes
---

## Why
The maintainer, 2026-10-02: "the original ps2 games had the marker for where the bullet goes higher up, so the camera could be
angled up higher and looking down without the bullets going straight into the ground. Look into the ps2 default
shooting angle, and adjust the hud marker accordingly. It should be higher up."
The RE is done (`analysis/re-aim-assist.md` §10, [O] UNAUDITED, 2026-10-02). On the PS2 the chase camera looks
`TiltValue` BELOW the aim (BF1 5 deg, BF2 10 deg); the aim ray leaves the camera's own position along the aim; the
reticle sits where that ray goes on screen (line 178.5 of 480 on BF1, 153.35 on BF2; the frame's centre is 240); the shot
leaves a fire point 1.58 m up and flies at the first thing the ray hits 5.5-65.5 m out. The port's camera looks along the
aim, its reticle is on the window centre and its shot leaves 1.425 m up along the aim.

## Evidence
- `analysis/re-aim-assist.md` §10: the rule (10.2), the section data per game and stance (10.1), the pitch (10.3), the
  reticle (10.4), the measured shots (10.5), the drawn camera and its collision (10.6), the change list (10.8).
- `engine/artifacts/lead_tmp/aima/`: `model_bf1.py` / `model_bf2.py` (the rule re-run on a save state: 132 states to
  0.5 mm), `bolts.py`, `probe_live.py` + `live/*.json` (44 single shots at 7 pitches per game), `stored_reticle.py`,
  `frames/ps2_reticle_sheet.png` and `frames/ps2_live_pitch_sheet.png` (the PS2 side of the frame pair).

## Do (build) - section 10.8 is the list
1. `core`: a camera section per game and stance on the soldier class (eye / centre height, TrackOffset, tilt): BF1
   stand (1.8, (0,0,3.5), 5), crouch (1.3, (0,0.15,3.5), 5), prone (0.5, (0,0,3.0), 5); BF2 stand (1.8, (0,0,3.2), 10),
   crouch (1.3, (0,0.15,3.0), 10), prone (0.5, (0,0,3.0), 5), sprint (1.8, (0,0,4.0), 10). Two functions on a unit: the
   ideal camera (eye, frame) and the aim ray (A, d), by §10.2 step 4. More than one view (split screen): tilt x 0.4.
2. `core/src/sim/step.rs`, a human on foot only: H = the first hit of the ray from A + d*(TrackOffset.z + 2) over 60 m
   (other units' capsules, vehicle hulls, the ordnance layer, the terrain), else its far end; the bolt leaves the fire
   point F (§10.2 step 2's stance table) toward H; the spread as today. Bots are not touched here (card FPNT).
3. `core/src/sim/input.rs`: the acquisition's pos = A (drop the 2.104 m constant).
4. `game/src/walk.rs`: `WalkSim::camera` = that ideal camera, then `settle_camera` as today. BF1 maps take BF1's table
   (this is CAMS's 3.5 m; CAMS keeps the per-class exceptions).
5. `ui/src/hud.rs`: `reticule.position` = (0.5, the projection of A + 1024*d through the drawn camera).
6. A human's spawn pitch -0.07 rad (PS2: every soldier's; bots' is FPNT's).
7. FEEL_NOTES: replace BHUD2's "Reticle on the window centre" line; CRATES.md / module headers for the files touched.

## Acceptance (run each, paste the output into the report)
- **The PS2's numbers, a core test**: standing on a flat floor at pitch -0.0700 rad - BF1 table: A is 2.1043 +- 0.002 m
  above the feet, the ideal camera's pitch -9.011 +- 0.01 deg, F is 1.5828 +- 0.002 m up, 0.060 m right, 0.2424 m ahead.
  BF2 table: A 2.3543 m, camera -14.011 deg, the same F.
- **The wall test** (flat floor, a wall across the aim, BF1 table, the spread off): wall 20 m out - the bolt's line
  passes within 0.05 m of the point where the reticle's ray meets the wall. Wall 80 m out - H is the ray's end (65.5 m
  from A) and the bolt meets the wall less than 0.15 m above the ray's point. Wall 5 m from the feet (inside the ray's
  5.5 m start) - H is the ray's end and the bolt meets the wall 0.45-0.55 m BELOW the ray's point (the PS2's blind
  spot; its F is 0.507 m under the ray). At the default pitch no bolt meets the floor within 5 m of the feet. At the
  steepest aim (-0.8 rad) on a flat floor the bolt lands 1.50-1.57 m ahead of the feet (the rule gives 1.53 m with BF1's
  table and 1.54 m with BF2's; the PS2 live, on rising ground: 1.43 m BF1, 1.51 m BF2).
- **The reticle**: at the PS2's 4:3 frame and vertical FOV the reticle's anchor is at y = 178.5 +- 0.5 of 480 on a BF1
  map and 153.35 +- 0.5 on a BF2 map; at 16:9 the same fraction of the height (`game/tests/hud_layout.rs`).
- **The frame pair**: a port frame at the NAB1 spawn pose and one at the HOT1 spawn pose beside
  `lead_tmp/aima/frames/ps2_reticle_sheet.png`, the reticle on the same row (`lead_tmp/aima/pairs/`).
- **Gates**: `sim` is declared (replays with a human firing); `goldens=tat1_hud.png` (the reticle moves up); the sweep
  and both arenas PASS unchanged - no human fires in them: paste their rows.

## Notes
- The rule is the same for both games; only the class data differs. PitchLimits (-0.8, 1.1) and prone +-0.4 are
  already the port's.
- Not in this card: the fire point for bots (FPNT); the camera's smoothing (AimTension 30 on BF1, MoveTension 15), its
  collision rule (§10.6: five rays from feet + 1.5 m, pull in at once, recover 0.75 / s) and the zoom sections (tilt
  3.5, offset (0.4, 0.05, 2.8)) - notes on CAMS; the reticle's colours and size (RETC).
- A choice to record in FEEL_NOTES: the PS2's blind spot (a wall or the ground nearer than 5.5 / 5.2 m along the
  reticle's ray is not seen, and the shot then lands 47-64 lines under the reticle). Faithful is the default; a modern
  alternative (start the ray at the fire point's distance) is one constant - say which was taken.

---
## The card as filed (2026-10-02), kept

## Why
The maintainer, 2026-10-02: "the hud is coming along, but the original ps2 games had the marker for where the bullet goes higher up,
so the camera could be angled up higher and looking down without the bullets going straight into the ground. Look into
the ps2 default shooting angle, and adjust the hud marker accordingly. It should be higher up."
In the port the reticle sits at the window centre and the shot goes along the camera axis (check this first), so a
third-person camera that looks down at the player's back aims into the ground.

## Evidence already in the repo
- `RETC` (this loop): "ps2live measured the crosshair at screen (320, ~178) of 640x480, ~5 degrees above the camera axis"
  (the 480-line frame's centre is 240, so the reticle is 62 px above it). `analysis/re-aim-assist.md` section 3.7 has the
  pick loop and the reticle colours.
- AIMF landed the PS2's filtered cone acquisition (`Sim::reticle_pick`).

## Do
1. **RE (bounded, section 5 discipline): where does a soldier's shot leave, on each game?** The aim direction the weapon
   fires along (camera ray vs the muzzle/aim point; a pitch offset; the PlayerController's aim target), the camera's own
   pitch limits and its look-at offset from the soldier, and where on screen that ray lands (the reticle). Both BF1 and
   BF2 (they may differ). Confirm with PCSX2 frames or RAM: fire at a wall at a known distance from a known camera and
   read where the hit lands against the view centre, at two pitches and two ranges (parallax: the offset is an angle
   or a point - they differ with distance; say which).
2. **Sim**: make the port's shot leave along that ray (the sim side first - AI shots are unchanged unless the PS2's bots
   use the same rule), keep the magnet / cone acquisition centred on that ray, not on the view centre. Declare the
   sim and parity moves; goldens that move are viewed before commit.
3. **HUD**: draw the reticle where that ray meets the screen (this moves BHUD2's reticle up; RETC owns its colours,
   texture and size - coordinate: if RETC has landed, move its offset; if not, put the offset in one named constant it
   will reuse). Mind the Hor+ anchor (COMPARISON_RULES) so the offset is an angle, not a fixed pixel row.
4. Check the camera collision and the third-person look-down limits against the PS2's (can the player look down as far as
   the PS2's camera allows without the shot hitting the player's own feet?).

## Acceptance
- The RE note (`analysis/re-aim-assist.md` new section) with `[O]`/`[V]` marks, the angle or point and its source (decomp
  address + a PCSX2 measurement for each game).
- A test: a shot fired from the default camera at a wall at 5, 20, 80 m lands where the PS2's rule puts it, equal to the
  measurement within its tolerance; a shot at the default pitch does not go into the floor within 5 m.
- A port frame beside a PS2 frame with the reticle at the same screen height (`lead_tmp/aima/`).

## Notes
- Related: RETC (reticle look), AIMF (acquisition), BHUD2 (HUD tree). Decoration vs sim: the sim change moves hashes
  where a human soldier's shots are in a sweep - the AI arena is unaffected by a human-only rule; say so with numbers.

- Planner 2026-10-02 14:45: value 4 -> 5 (the maintainer's INBOX note of 10-02: OBST and AIMA, value 4-5, after the `first:` list, which is empty since STEND landed). `files:` emptied while the phase is `re` (a claimed card's files count as busy: with `sim/step.rs`, `weapon.rs` and `ui/src/hud.rs` listed this RE would have held GREN, JARC, PWUP, SQACK and BHUD3 for its whole run). Restore at the build: `engine/crates/core/src/sim/input.rs,engine/crates/core/src/sim/step.rs,engine/crates/core/src/weapon.rs,engine/crates/game/src/walk.rs,engine/crates/ui/src/hud_draw.rs,engine/crates/ui/src/hud.rs`. RETC (value 3) keeps the reticle's colours and size; its place is this card's. Undo: `loopctl.py set AIMA value=4`.
