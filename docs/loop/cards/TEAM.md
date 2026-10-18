---
id: TEAM
title: Team sense: a unit grid for every 'who is near' question, roles with capacities inside a post's group (cover for a capturer, overwatch), fire lanes in the position choice, contacts passed on within 20 m (bot design 4.3)
milestone: M3
value: 5
phase: build
status: todo
deps: BOTD,TPOS
files: engine/crates/core/src/ai/team.rs,engine/crates/core/src/ai/behave.rs,engine/crates/core/src/perception.rs,engine/crates/core/src/sim/step.rs
declares: sim; sweep; arena
needs:
effort: high
branch:
resume:
created: 2026-10-03
alpha: yes
---

## Why
The maintainer, 2026-10-02: bots "all standing next to each other", shooting each other; "awareness of friends". `analysis/bot-design.md` 4.3 (the Team board), 4.1 (the unit grid). The PS2 has no team sense at all (SAIR 6). A teammate within 1.5 m: 8.7-17.9 % of a port bot's time against the PS2's 2.0-9.3 % (`lead_tmp/botm/post_jitr/seeds.md`).

## What to build (`ai/team.rs`)
1. **The unit grid:** 8 m cells, units sorted by cell each tick by a counting sort into buffers sized at load; "who is within r of p". `friend_in_line`, `spot_free`'s successor (reservations), the hunt-target loop and `perception::scan`'s candidate loop read it instead of walking every unit.
2. **Roles inside a post's group**, per team once a second in unit-index order, kept at least 20 s (bot-design.md 4.3): Assault (default), Overwatch (scout classes with a free Snipe hint), Cover (while a friend captures: up to 2 others take guard positions facing the approach). Flank is card FLNK's and Gunner card TURR's: this card builds the board they plug into.
3. **Fire lanes:** each friend's line of fire feeds the Tactician's "outside a friend's line" filter.
4. **Contacts passed on:** a contact a friend within 20 m holds becomes "heard" for the bot - walked toward and aimed at, never fired at unseen (by tier: bot-design.md section 7).

## Acceptance (run and paste)
- A test: the grid's "within r" equals the brute-force loop on 1,000 random scenes (positions, radii); the sim hash of a 6-a-side replay is unchanged by the grid alone (land the grid first if needed).
- **B3:** a teammate within 1.5 m <= the PS2's on KAM1, POL1, DEA1, KAS2 (8 seeds). **B4:** team kills of kills <= 3 % on every map.
- Role counts printed per map (cover, overwatch); a test that no capacity is exceeded.
- ms a tick at 33 units not up (bot-design.md M2's method: `lead_tmp/botd/tools/sample.sh`, `prof_keys.py`).
- Sweep 8 seeds x 32 maps: 256 / 256. Declared: `sim; sweep; arena`.

## Notes
- Cut from `analysis/bot-design.md` (card BOTD, 2026-10-03). Waits for the maintainer's read (deps BOTD). In BOTV's deps.
- Not in it: a squad leader, formations, plans. Five roles at most.
- 2026-10-05 planner - **The maintainer has read the bot design: "BOTD read, go" (INBOX 2026-10-03 14:08). All eight decisions take the note's defaults; the 16 deviations D1-D16 are all on. This card is released (BOTD is done).** Decision 4: **no garrison** (no Guard role that keeps 1-2 bots at a front-line post beyond the design's cover-for-a-capturer). Decision 5 (the flank share, 20 %, drawn 10-30 per match) is card FLNK's; the role's capacity here follows it.
- 2026-10-05 planner: value 4 -> 5: every card BOTV waits on is ranked with the chain, so the bot programme runs in the maintainer's order ahead of the fours (his notes of 2026-10-02 16:30 / 17:40 and 2026-10-03 14:08). Undo: `loopctl.py set TEAM value=4`. `effort: high` (the builds the maintainer's top concern rests on; a build runs at medium by default). Undo: `loopctl.py set TEAM effort=`.
- 2026-10-06 HUBP landed (408bc9c): what is left of "a teammate within 1.5 m" after the hub points, 8 seeds - KAS2 2.9, DEA1 11.6, KAM1 5.8, POL1 13.9 % (PS2 2.0 / 9.3 / 6.6 / 5.7). The remainder is bots standing at posts and holding (POL1: holders 6.6 % of unit-time, KAM1 cp2's idle bots) and POL1's in-file corridors; `lead_tmp/hubp/ab/near.py <dir> <map>` splits the close pairs by hold / still / move and by 6 m cell. Bar B3 is this card's.

- 2026-10-06 MOVR landed (6061eec): arrived bots stand (a latch), and "a teammate within 1.5 m" rose a little with
  it - 24 seeds: KAM1 6.5 -> 7.2 %, POL1 14.0 -> 14.8 %, DEA1 12.6 -> 12.3, KAS2 3.3 -> 3.2 (PS2 6.6 / 5.7 / 9.3 / 2.0).
  Bots that stand near each other no longer jitter apart; the reservation is this card's.
- 2026-10-07 planner, from SPRD's report (`reports/20261006_2233_card_SPRD.md`): **a gap in FFLN's line-of-fire check** - a salvo already queued when a friend walks into the cone still fires (the PS2's fire cycle fires a started salvo whatever the trigger does). One such shot in 900 ticks of `a_bot_does_not_fire_through_a_friend`, which now does not count it. Not carded (it did not hit); this card's fire lanes are where it belongs: count team hits from queued salvos in the `teamfire` line before deciding whether a started salvo should be cut.
