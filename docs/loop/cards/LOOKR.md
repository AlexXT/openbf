---
id: LOOKR
title: The right stick's turn ramps up as on the PS2: a held stick starts at 48 (BF1) / 65 (BF2) deg/s and climbs linearly to 309 / 286 deg/s over one second, pitch is flat - the port turns at one flat rate from the first tick (183 deg/s under the maintainer's look scale); RE what drives and resets the ramp, then build it
milestone: M3
value: 4
phase: re
status: todo
deps:
files:
declares:
needs:
effort:
branch:
resume:
created: 2026-10-06
alpha: yes
---
## Why
The ZOOM RE session (INBOX 2026-10-06 10:45, find 1) measured, live on both PS2s, that a human's yaw rate is NOT flat:
with the right stick held the turn starts slow and climbs for about a second. The port turns at
`max_turn_speed x stick` from the first tick (`core/src/soldier.rs`, `self.yaw -= turn_in * c.max_turn_speed * ctl[2] * dt`),
under the maintainer's look scale (`input::LOOK_SCALE` 0.80, his playtest tune of 2026-10-04: "a slower camera").
A slow start is what makes small aim corrections easy on a pad, and the PS2's top rate is higher than the port's,
so both ends of the port's turn differ from the original. The maintainer's top concern list has "controls"; no card named this.

## Evidence (measured rows, not re-run by the planner)
- `analysis/re-zoom.md` section 6 [M] and section 1; rows `engine/artifacts/lead_tmp/zoom/ps2/out/bf1_turn_plain_{0,1}.csv`,
  `bf2_*turn_plain*`, probe `lead_tmp/zoom/ps2/zoom_probe.py`.
- BF1 rifleman, right stick held 1 s: 47.7 deg/s on the first update, +9.03 deg/s per update, 309.4 deg/s after 1 s
  (= MaxTurnSpeed 4.0 rad/s x 1.35, the sensitivity factor). BF2: 64.9 -> 286.2 deg/s. Pitch is flat: 38.7 (BF1) /
  57.2 (BF2) deg/s. Zoomed, every one of these is divided by the weapon's divisor (card ZOOM builds that).
- Code named, NOT read through: B1 `FUN_002098b8` 119344-119345 (class +0x1844 + (class +0x1840 - class +0x1844) x a
  ramp in the Controllable), B2 `FUN_00247fa8` 162907-162920; BF1 scales by the sensitivity setting
  (0.19 x n + 0.4, `FUN_0034caf0`).
- The port today: full stick = 4.0 rad/s x 0.80 = 183 deg/s from tick 1 (planner's arithmetic from the two constants,
  not a measurement).

## Do
1. **RE, bounded** (the RE discipline; one verifier): what drives the ramp and what resets it - a released stick, a
   reversal, a part-deflected stick (does the ramp scale with deflection or only with time held), a zoom, a vehicle
   seat; the two class fields' ODF keys and defaults; BF2's sensitivity option. A few live rows per reset case with
   `zoom_probe.py` (it already logs yaw per update). Output: a section in `analysis/re-zoom.md` or a short
   `analysis/re-look-ramp.md`, RE_LOG pointer.
2. Re-phase to build with `files:` (expected: `core/src/soldier.rs`, `core/src/soldier/tests.rs`,
   `core/src/aim_assist.rs` if the assist's rate shares it) and `declares:` (the player's turn only: bots write the
   look sticks through `gunner_foot` - say whether their turn goes through the same ramp on the PS2; if it does,
   `sim; sweep; arena` move and are declared).

## Acceptance
- RE: each rule above tagged [V] / [D] / [H] with the checks pasted and the verifier's verdicts.
- Build: a core test that replays a held full right stick for 30 updates per game and compares the per-update yaw
  rate with the PS2's rows (`bf1_turn_plain_0.csv`, the BF2 twin): every update within 2 %, pitch flat within 2 %.
  One test per reset rule the RE names. Mutation check: a flat rate fails the test.
- `FEEL_NOTES.md`: one line (the ramp, its two ends, how to turn it off).

## Notes
- **The maintainer's tune stays his.** `LOOK_SCALE` 0.80 was set by the maintainer with a pad in hand against the port's FLAT turn. The
  default this card takes: build the PS2's ramp SHAPE and both PS2 ends (the feel rule: the PS2's measured values
  first), keep `LOOK_SCALE` as the one multiplier on top, and give the ramp one constant that switches it off.
  STATUS asks the maintainer (question 25); if he says "leave my turn alone", close this card with the RE note only.
- Find 2 of the same INBOX note (BF2's PS2 reload is D-pad Down; both games' default pad tables are in the ELFs at
  BF1 0x519170 / BF2 0x600e40, five modes) is recorded here because no remap card exists yet: PLAN "Platform & meta"
  promises "PS2 default mapping, full remap", the port's layout is the maintainer's own (INBOX 2026-10-04), and the tables are
  what a "PS2 default" preset of a remap screen would be built from. Not part of this card's build.
- Filed by the planner 2026-10-06 14:00 from the INBOX note. Undo: `loopctl.py set LOOKR status=done` with a line why.
- 2026-10-07 planner, from GRNP's report: **the PS2 lets the aim go 0.6 deg higher than the port** (a PS2 BF2 throw at 63.6 deg; the port clamps at 63.0) - the one throw of 15 that still differs (0.67 m). The pitch limits belong with this card's look-stick RE: read both limits (up and down) with the ramp.
