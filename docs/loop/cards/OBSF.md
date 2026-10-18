---
id: OBSF
title: A holder whose point lies beside a crate circles the crate's rim (OBST's early turn, the maintainer's Kamino scene: 17 reversals in 10 s): a holder's give-up counts the early turn as a pick that is not straight
milestone: M3
value: 5
phase: build
status: done
deps: OBST
files: engine/crates/core/src/ai/avoid.rs
declares: sim; sweep; arena
needs:
effort:
branch: obsf
resume: landed b870afc9 2026-10-07
created: 2026-10-07
owner: OBST 1099535
---
## Why
Card OBST (landed e9ce6db, 2026-10-07) walks a bot by a small body it sees up to 3 m ahead (`avoid::look_ahead`, `SteerPick::Round`). The landing's own check of the maintainer's F9 scene (BF2 Kamino, an idle human at -4.0, 60.3, 108.6; 8 seeds x 5 snapshots, `lead_tmp/obst/tools/scene_seeds.sh`) found holders on a 1 m step to a hold point that lies BESIDE a crate steering `round` for 10 s and more, along the crate's grown disc and back: unit 31 at (-12.9, 61.1, 125.3), hold goal (-14.08, 60.75, 125.66), 17 reversals, 4 m of path, 0.4 m net (`lead_tmp/obst/scene_final/t641800.json`). `Round` is not a blocked pick, so the holder's give-up (`HOLD_BLOCKED`) never ran. Trails over 6 reversals within 20 m: main before OBST 3 of 500 (worst 9), OBST 4 of 451 (worst 17).

## What to do (as built, 2026-10-07)
A holder's give-up (`HOLD_BLOCKED`, card JITR: a goal the steering cannot go straight at for 1 s is given up for where the bot stands) counts EVERY pick that is not straight, the early turn included (`mover.rs`, one line + a test). Leg walkers are untouched.

**Measured and NOT taken** (`lead_tmp/obst/`; base main 9cd9b52 + OBST e9ce6db; 24 seeds = `sw_<name>` + `sw_<name>_s9_24`):
| variant | scene: trails over 6 reversals (worst) | census stalled windows | 24 seeds: rounds, recycles, no-progress at end |
|---|---|---|---|
| main before OBST | 3 of 500 (9) | 2.41 % | (8 seeds: 256 / 256, 116, 22) |
| OBST as landed | 4 of 451 (17) | 1.25 % | 768 / 768, 169, 41 |
| the early turn only for a body between the bot and its aim point (`fix1.patch`) | 1 of 478 (16) | 1.28 % | 768 / 768, **224**, 58 |
| fix1 + "beside" = any shut step, not only a body's sweep (`fix2_cumulative.patch`) | 3 of 461 (9) | 1.32 % | **767** / 768, 180, 69 |
| **this card** (`fix3.patch`) | **1 of 469 (7)** | 1.22 % | 768 / 768, 183, 45 |

## Acceptance (ran)
- `a_hold_point_behind_a_crate_is_given_up_not_circled` (mover.rs): the bot walks round the crate, gives the point up and stands within 2 s; mutation (the early turn not counted) kills it.
- The maintainer's scene, 8 seeds x 5 snapshots: 1 trail over 6 reversals of 469, worst 7 (OBST 4 of 451, worst 17; main before OBST 3 of 500, worst 9). The bar "no bot over 6 reversals" is still missed by one trail (#11, 7 reversals, 14 m of path).
- Census 1.22 % of windows stalled (OBST 1.25 %); 24 seeds x 32 maps 768 / 768 rounds, recycles 183 against OBST's 169 (like variants ran 169 - 224).
