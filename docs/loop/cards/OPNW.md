---
id: OPNW
title: The opening wave draws a post, then a node, as the PS2's members do - today the bots are dealt round-robin over the flat list of every post's nodes (and the pads' command walkers' points), so a post with more nodes starts with more bots
milestone: M3
value: 3
phase: build
status: todo
deps: BSPN
files: engine/crates/game/src/walk.rs
declares: sim; sweep; arena; leak
needs:
effort:
branch:
resume:
created: 2026-10-09
---
## Why
Card BSPN made a bot's RESPAWN draw one eligible post and then one of its nodes (the maintainer, 2026-10-07: 'make the bot spawn the same as the ps2'). The opening wave was left as it is: `WalkSim::opening_wave` (`game/src/walk.rs`) deals `ai_per_team` bots round-robin over `Sim::spawn_points` plus `pad_walker_spawn_points`.

## Evidence
- The PS2's opening members draw a post by the same rule (re-command-walker-post 6, RAM: BF1 Hoth's 24 Empire AI chose the fixed post 10, the AT-ATs 9 and 5; BF2 Hoth 18 AI: 5 / 5 / 4 / 4).
- The port's opening column: `SPAWN_CENSUS=1` `spawns <key>:` line, `opening` (runs in `lead_tmp/bspn/sweep_fin`).
- `Sim::draw_spawn_post` / `spawn_post_nodes` (BSPN) do the draw for live posts; a pad's not-yet-built command walker needs a post of its own in the opening (`pad_walker_spawn_points` returns a flat list).
- Risks to check first: tests that pin the round-robin places, map goldens that draw the opening soldiers, several bots on one node at tick 0 (the PS2 has no 'occupied' test, and spreads the opening over about a second).

## Acceptance
- A test: posts of 1, 2 and 6 nodes each start with a third of the opening bots (within the draw's noise over many seeds).
- The `spawns` line's opening column over 8 seeds x 32 maps, before and after, beside an even share per post; the sweep's finish, stuck and recycle counts before and after.

- 2026-10-09 12:45 planner. `deps: BSPN`: the same rule and the same file (`game/src/walk.rs` is in BSPN's `files:`), and BSPN's branch is built and waiting on the maintainer; built first, this card would collide with it. Undo: `loopctl.py set OPNW deps=`.

- 2026-10-12 15:21 AUDIT BSPN (`loop/audits.md`). Size of what this card owns: the opening wave is 13.5 % of a round's spawns (8,448 of 62,753 over 8 seeds x 32 maps on the landing 2d1fb60; 13.4 % on main d096ae7; `spawns` rows, opening against round), all dealt round-robin over the flat node list. The PS2 side of the rule is now on firmer ground than Hoth's three counts: six independent PS2 captures (737 AI life starts on BF2 Dagobah, BF2 Kamino, BF1 Bespin) reject the flat list (chi2 54.5, 8 dof) and fit the even post draw (9.8) - `lead_tmp/audit-bspn/wf/K4/out3.txt`; those captures skip each slot's first appearance, so the OPENING members' own draw is still the RE's reading (re-command-walker-post 6) plus Hoth's 10 / 9 / 5. `deps: BSPN` is met (landed 2d1fb60). Card SPTH touches the same file (`walk.rs`) for a test of the weight's plumbing.
