---
id: FLNK
title: Flank routes: per post pair the best other way within +35 % is stored at load and a share of the attackers (10-30 %, drawn per match) takes it - the route table has no second way to tilt (the maintainer's annoyance 2; bot design 6.2)
milestone: M3
value: 5
phase: build
status: todo
deps: BOTD,TEAM,FGHT
files: engine/crates/core/src/nav.rs,engine/crates/core/src/ai/route.rs,engine/crates/core/src/ai/team.rs
declares: sim; sweep; arena
needs:
effort: high
branch:
resume:
created: 2026-10-03
alpha: yes
---

## Why
The maintainer's annoyance 2 (2026-10-02): "Some maps have totally unused parts ... On some maps you just know where everyone is going to go" - and his cautions: "It is still fun to have fighting hotspots", bots must not become "dumb" or spread thin. Measured (`analysis/bot-design.md` M5, 6.2): toward a post the authored route table has a second next hop at 0-2 % of its entries on ten maps, so a tilt of the weights cannot do it; another route within +35 % exists for 20 of 20 post pairs on Mygeeto, 22 of 30 on Tatooine Dune Sea, 13 of 18 on Yavin Arena, 18 of 30 on Polis Massa, 13 of 29 on Utapau, 7 of 12 on BF2 Hoth, 3 of 17 on Kamino. The PS2: every never-visited hub lies off every authored post-to-post route (`re-soldier-ai.md` 10b).

## What to build
1. **The flank table at load** (`nav.rs`): per ordered pair of post hubs, the best other route - a search over the soldier arcs with that pair's authored arcs costing x3 - kept when no more than 35 % longer than the authored main route and with at most half its arcs on the authored set. Reference: `lead_tmp/botd/tools/flank.py`, output `lead_tmp/botd/out/flank.txt`.
2. **The share:** per match, a seeded draw gives each post pair a flank share between 10 and 30 % (default 20 %; the maintainer's decision 5). The Team board (card TEAM) gives the role Flank to that share of a post's attackers standing at a hub with a flank route, never in a group under 3, kept at least 20 s; the Navigator walks the stored hub list, then the authored route.
3. Printed per map: pairs with a flank route, the match's shares, hubs visited.
4. LATER, only if the bars hold (a note on this card, not this landing): the share leans to the flank while friends die on the main route (a decaying count per hub).

## Acceptance (run and paste)
- The table's per-map counts equal `flank.py`'s (pairs within +35 %) on the ten maps of M5.
- **B11:** hubs never visited by a soldier (SAIR's `hubuse.py` rule: feet within max(radius, 3 m) of the centre) on BF2 KAM1 and POL1 <= the PS2's 32 % and 28 %, reported on the six showcase maps - WITH B5 (an enemy within 20 m), B6 (deaths a minute) and B7 (the three busiest cells' share of deaths) holding on every map with a PS2 column, and no map's contact share or kills a minute dropping by more than the seed noise against main (8 seeds; a drop is re-run over 24 more).
- A test: in a group of 10 attackers with a flank route and a share of 0.2, exactly 2 take it; a group of 2 takes none.
- Sweep 8 seeds x 32 maps: 256 / 256. Declared: `sim; sweep; arena`.

## Notes
- Cut from `analysis/bot-design.md` (card BOTD, 2026-10-03). Waits for the maintainer's read (deps BOTD). In BOTV's deps.
- "Another route" may only reuse ways authored for other post pairs (Mygeeto: 8 of its 20); it still splits an attack. The dead ground flank routes reach is 3-37 % of the off-route hubs; the rest is reached by positions (card TPOS).
- Unit counts per map are card UCNT.
- 2026-10-05 planner - **The maintainer has read the bot design: "BOTD read, go" (INBOX 2026-10-03 14:08). All eight decisions take the note's defaults; the 16 deviations D1-D16 are all on. This card is released (BOTD is done).** Decision 5 is settled: **flankers 20 % per match, drawn between 10 and 30**.
- 2026-10-05 planner: value 4 -> 5: every card BOTV waits on is ranked with the chain, so the bot programme runs in the maintainer's order ahead of the fours (his notes of 2026-10-02 16:30 / 17:40 and 2026-10-03 14:08). Undo: `loopctl.py set FLNK value=4`. `effort: high` (the builds the maintainer's top concern rests on; a build runs at medium by default). Undo: `loopctl.py set FLNK effort=`.
