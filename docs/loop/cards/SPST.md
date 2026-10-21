---
id: SPST
title: A bot draws a POST, then a point of it: the PS2's AI picks its spawn post uniformly (BF1 FUN_00261e80) or by AISpawnWeight (BF2 FUN_002e7240, default 100) once per death and keeps it; the port draws one node from the pooled list, so a post with more nodes takes more spawns
milestone: M3
value: 4
phase: build
status: todo
deps: CWSP
files: engine/crates/core/src/sim/spawn.rs,engine/crates/core/src/cp.rs,engine/crates/game/src/walk.rs,engine/crates/game/examples/round_to_outcome.rs
declares: sweep; sim; arena
needs:
effort:
branch:
resume:
created: 2026-10-02
alpha: yes
---

## Why
Found by card CWSP's RE (2026-10-02, `analysis/re-command-walker-post.md` section 6, verifier-checked). On the PS2 an
AI member that dies waits 4.0 s, then draws ONE POST and keeps it until it spawns (redrawn only if that post dies or
stops being its team's): BF1 `FUN_00261e80` — uniform over the scheduled posts its team may spawn at; BF2
`FUN_002e7240` — weighted by the post's `AISpawnWeight` (cp+0x11c, default 100.0; the disc authors it once, KAS2.LVL
instance `baserush` = 1000; no conquest script sets it). The spawn then draws a node of THAT post's path (or a class
spawn point). The port's `Sim::respawn_slot` draws one node uniformly from the pooled nodes of every post the team
owns (`Sim::spawn_points`), and `setup_conquest` walks the same pool round-robin for the opening wave — so a post with
13 nodes takes three times the spawns of a post with 4. A team's spawns bunch at its many-node posts; the PS2 spreads
them evenly over posts. Player-visible on every map (where each side appears), so M3.

## Evidence
- PS2 members' own records (`lead_tmp/cwsp/members1.py`, `members2.py`): BF1 Hoth, 24 Imperial AI over 3 posts:
  10 / 9 / 5; BF2 Hoth, 18 AI over 4 posts: 5 / 5 / 4 / 4; BF2 Geonosis, 25 Republic members over 4 posts: 6 / 5 / 8 / 6.
- Live appearances over 240 s (`lead_tmp/cwsp/spawn_share.py bf2 hot1L`): 11 / 9 / 8 / 11 (chi-square 0.7 against
  quarters); BF1 Hoth 65 s: 11 / 9 / 6.
- Port node counts on Hoth: BF1 CP6 4 nodes; BF2 CP6 4, CP3 7 (the baked `manifest.ron` paths) — with CWSP's two
  walkers (6 points each) a node-uniform draw gives BF1 Hoth 75 % at the AT-ATs (PS2 two thirds) and BF2 52 % (PS2 a
  half); on a map with one 13-node and one 4-node post it is 76 % / 24 % against the PS2's 50 / 50.

## Acceptance
1. Referee first: a `core` test — two posts of 4 and 12 nodes, 2000 respawns: each post takes 50 % +- 3 % (today
   25 / 75; keep that as the mutation check); a post with weight 1000 beside two of 100 takes 1000 / 1200.
2. `round_to_outcome`'s per-post spawn census (CWSP adds it) on BF1 HOT1, BF2 HOT1 and one map with unequal node
   counts: the opening wave and the first 240 s within the PS2's shares above (chi-square p > 0.05 against uniform).
3. 8 seeds x 32 maps beside a null variant: finish counts and win splits reported per map; a map whose finish count
   drops is explained or the card is parked with the map named.

## Notes
- One draw per death, kept until the spawn (the member's own post); the opening wave is the same draw per member.
- BF2's `AISpawnWeight` instance key (hash 0xE07282C2) is not baked today; KAS2's `baserush` instance carries 1000
  (check its layer: conquest or campaign) — parse it if it is a conquest post, else default 100 everywhere.
- The human's post is card SPSEL (keep-or-nearest, the spawn map's pick).

- Planner 2026-10-02 09:30: SPSEL (re-scoped today, `deps: CWSP`) puts the PLAYER's chosen post into the sim by the PS2's
  spawn-map rule. This card's "a member draws ONE POST and keeps it until it spawns" is the same state for a bot: build
  on SPSEL's field if it landed first (or leave one a human can use), not a second mechanism. Not merged: this card
  moves the spawn split and the sweep on every map, SPSEL moves nothing for an idle player.
- (card CWSP, 2026-10-02) CWSP landed the walkers into the pooled list. Measured (`SPAWN_CENSUS=1`, 8 seeds): BF1 HOT1
  opening wave 12 of 17 at the AT-ATs (PS2 14 of 24); BF2 HOT1 5 of 17, all at ONE AT-AT (PS2 4 + 4 of 18); BF2 GEO1 0 of
  17 at the AT-TE (PS2 2 of 16) — the round-robin never reaches the walker's points behind the posts' nodes. The opening
  wave (`WalkSim::opening_wave`) needs the post-then-point draw too; add to this card's acceptance: BF2 GEO1 >= 1 opening
  spawn at the AT-TE on 8 of 8 seeds, BF2 HOT1 both AT-ATs take opening spawns.

- Planner 2026-10-02 14:45: value 3 -> 4, `alpha: yes`. CWSP (M2, landed 69ba267) left an acceptance line unmet that is this card's: BF2 GEO1 0 of 17 opening spawns at the AT-TE and BF2 HOT1 (a showcase map) all 5 at ONE AT-AT, because the pooled round-robin never reaches a walker's points. CONT's build makes every bot draw its target post at the spawn, so the spawn split now shapes the whole battle. Shares `sim/spawn.rs` with SPSEL: one at a time. Undo: `loopctl.py set SPST value=3 alpha=`.

## Note (planner 2026-10-03 07:15, from BPRL's landing) - the walker post's light
`lead_tmp/bprl/build/lights_vs_ram.py` set the port's still post lights against the PS2's live r = 8 omni lights in
all 95 stored BF2 walk states: 552 of 558 match. The 5 PS2-only ones are one light: red, r = 8, at (-423.9, -18.2,
186.0) on BF2 Hoth in every one of its five states, and it is no world post - by its place it is the AT-AT's own
command post (CWSP's walker post). The port's stills have no battle and so no walker; nobody has looked at whether
the port's walker post carries its post light in a battle (`game/src/point_light.rs`). Whoever is in the walker
post's code next: one headless HOT1 battle frame with `PXC_LIGHTS=1`-style printing of the lights, the walker post's
among them or not, and if not, add it (the post's light at the walker's post point, owner colour). Small; no card.
- SPSEL landed 2026-10-20 (6dc5a68): a human's post is `Sim::spawn_post_choice[slot]`, moved by `pick_spawn_post` (along + 0.25 perp^2), read through `keep_or_nearest_post`; `player_spawn_posts(team)` is the eligible list with live positions. Reuse it, do not add a second mechanism.
