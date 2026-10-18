---
id: DAGS
title: BF2 Dagobah: team 0's bots stand in 'recover' for 20-140 s at (2.4..4.1, 4.5, -47.9) within seconds of spawning (already on main, 2 of 8 seeds; the PS2's spawn draw of card BSPN sends more bots there and its landing is refused on stand20_pct@bf2_dag1) - trace one case, name the spawn node / post and the cause, fix it
milestone: M3
value: 5
phase: build
status: done
deps:
files: engine/crates/core/src/ai/route.rs engine/crates/core/src/ai/route/tests.rs
declares: sweep; arena; sim
needs:
effort:
branch: dags
resume: landed 4ca302f2 2026-10-12
created: 2026-10-11
alpha: yes
first: 2
---
## Why
Card BSPN (the maintainer's ask, approved by him 2026-10-11 for Bespin's falls) cannot land: the fidelity gate refuses stand20_pct@bf2_dag1 0.36 -> 1.08 on seeds 1-8 and 1.32 on the confirm seeds 9-16 (tolerance 0.7). The standing is not new - main has it at this same spot - the PS2's post-then-node draw only sends more bots to it.

## Evidence (land session 2026-10-11, report reports/20261011_1643_land_BSPN.md)
- Gate rows, Dagobah, seeds with a standing spell: baseline (main 4c238b1, engine identical to 8ad30f4) 1 of 8 (202 foot-s); branch bspn be6d9ac seeds 1-8 4 of 8 (143, 106, 242, 103 foot-s), seeds 9-16 5 of 8 (112, 103, 210, 192, 115). Nearly all of it is behaviour 'recover'.
- From the 4 Hz samples (botq/bf2_dag1_s<seed>.csv; still within 1 m for 20 s+ in 'recover'): team 0 bots at (2.2..4.1, 4.4..4.6, -47.8..-48.0), 6-40 s after their spawn, for 23-140 s - baseline s6 (units 9, 8) and s7 (units 6, 12); branch s1 (units 6, 5), s2 (unit 11), s9 (unit 7), s15 (unit 10). Second spot, below it at y -3.9: (10.6..19.8, -3.9, -43.8..-52.3), 20-31 s spells, both builds. One team 1 bot at (35.7, 3.7, 149.1) for 86 s on the branch's seed 3.
- Files: engine/artifacts/lead_tmp/bspn/land/branch/fidelity{,_confirm}/ (branch), engine/artifacts/lead_tmp/fidelity_rebase_20261009/gates/fidelity/ (baseline).

## Acceptance
- The cause named from ONE traced case (a headless round with BLACKBOX_ON=stuck on bf2_dag1, or BLACKBOX_AT at the spell's tick): which post's spawn node drops the bot there, what its recover is waiting on, the body under it.
- After the fix: stand20_pct@bf2_dag1 on main's baseline rule AND on branch bspn's rule at or under the baseline 0.36 + 0.7 on seeds 1-8 and 9-16; no spell of 20 s+ at the spot in 16 seeds.
- **The fix is the general rule, not a Dagobah patch (the maintainer, 2026-10-11, via Claude):** a spawned bot's FIRST leg starts at a hub the
  floor walked from its feet reaches (`walk_floor`), not at the nearest hub in XZ - here the hub at its own height (hub 12, point
  y 2.7), not hub 16 eight metres below. Named as ours or as the PS2's, whichever a PS2 capture from this spawn node shows.
  A unit test: a bot spawned on a deck over a lower route takes the deck's own hub.
- The endless recover / travel cycle is card RLOOP and the all-maps spawn-node check is card SPNC: do not fold them in here.
- BSPN no longer waits for this card (the maintainer, 2026-10-11: an exposed bug does not fail the fix that exposed it): it lands with `[approvals.BSPN] ... exposes = "DAGS, BESR"`. When THIS card is done, the planner takes `stand20_pct@bf2_dag1` out of that approval and re-takes the fidelity baseline.

## Trace (Claude in chat, 2026-10-11, main 8ad30f4; snapshot `lead_tmp/dags/main_8ad30f4_bf2_dag1_s7_t3000.json`, `blackbox_read --unit 6` / `--unit 3`)
Seed 7, BF2 Dagobah, units 6 and 3 (team 0). On main's own fidelity run 10 such spells of 76-172 s show on seeds 3, 6, 7, 10, 11
(`lead_tmp/fidelity_rebase_20261011/gates2/fidelity*/botq/bf2_dag1_s*.csv`).
- Both SPAWN at one node, **(-20.6, 4.0, -45.1)**, feet y 4.0 (ticks 1230 and 1399).
- Both are put straight on leg **16 -> 15** (`locus N`, off the graph). Hub 16 (Hub22) is at (-6.6, -54.4) and hub 15 (Hub21) at
  (1.0, -47.0), radius 2.6 - and the leg's aim point is at **y -3.5**: `wp [1.0, -3.5, -47.0]`. Other bots cross hub 15 with their
  feet at y -3.3 .. -3.7 and pass in 1-8 samples. So the route runs about 8 m BELOW the level these two spawn on.
- They walk the upper level to hub 15's XZ (wp_d 0.39 at tick 1350, feet y 6.3), never count as at the hub, and from tick 1365 are
  against `dag1_prop_barrier_1 dag1_prop_barrier_23 @ (1.9, 4.0, -42.4)` (`stop wall 0.00`) at (2.4..4.0, 4.5, -47.9).
- From tick 1470 it alternates `recover` (10 s) and `travel` (2 s): recover re-plans `-1 -> 12` (hub 12's point (-10.0, 2.7, -66.2),
  the only one near its own height) then `12 -> 16` (y -3.4), each time with the barrier still shut in front; `stuck` events every
  6-12 s; `level 0.0`, `lost 0` throughout, so neither the lost rule nor a recycle ends it. 150 s for both.
- Why BSPN multiplies it: the flat node draw gave this node one share among all of team 0's nodes; the PS2's post-first draw gives
  its post a whole post's share (main 1 round in 8, BSPN 4-5 in 8).
NOT traced: which post owns the node; whether a floor walk from the node reaches hub 12 (the ramp down); what the PS2's bot does
from this node (a capture there is the bar); why the hub's "top floor" reads 7.0 in `floor_dy`.

Candidate fixes, to measure: (1) a spawned bot's first leg starts at a hub the floor walked from its feet reaches (`walk_floor`),
not the nearest hub in XZ - here hub 12, not 16; (2) a bot that has cycled recover / travel N times without 2 m of net movement
is lost (RETW's floor path back) or recycled, not left 150 s; (3) census every spawn node on the 32 maps: the node's floor against
the level of the hub its first leg takes. Same family as card MOUTH (a leg started from where its floor does not lead).

## Measure (lane 2, 2026-10-12 12:40; main 03422f5 = the binary; all files under `engine/artifacts/lead_tmp/dags/`)
**The cause, traced** (seed 13, unit 8; `trace/t2500/bf2_dag1_s13.json`, `trace/bf2_dag1_s13.json`, `trace/dbg2_s13.err`):
- The node is the SECOND of post **cp4**'s three (cp4 = the cave post at (15.0, -4.0, -50.0), hub 14): authored (-21.0, 3.8, -45.0),
  floor 4.0 - on the ground OVER the cave; the other two are in the cave (y -3.6, -1.8). `FLOOR_GRID` in `trace/s13.err`.
- A bot's first leg after a spawn is planned by `route::retarget_phase_a` (no commander objective yet), and that ONE entry
  asks `NavGraph::unit_hub_on` directly: the nearest arc in XZ is 16-12 at its hub-16 end, so the leg is 16 -> 15, hub floors
  -3.4 / -3.5, feet 4.0. Every other entry (`retarget_to_post`, `retarget_hunt`, `retarget_board`, `replan`) goes through
  `start_hub` = the same locate + STLVL's `level_hub`; at the node `start_hub` answers hub 10 (the floor walk reaches hubs
  10, 12, 41, 18, 42 and 11 - so yes, it reaches hub 12). The debug print: `plan_leg ... cur 16 dest 4 start_hub Some(10)`.
- The bot runs east over the cave to hub 15's XZ, climbs `dag1_prop_barrier_23` (feet 5.8) and drops into a pocket of it at
  (2.5, 4.6, -47.8): `STOP Wall at 0.00 m body 327`, in the air every other sample; recover count 29 after 80 s.
- Today's size: the spot holds a 20 s+ recover spell in 13 of 16 seeds of the planner's baseline run (main 19708fa, BSPN on) and
  11 of 16 on 03422f5 (27 spells); 34 of 129 lives from the node cross onto the cave's roof. `tools/spells.py`, `tools/nodelives.py`.

**The PS2 from this node** (`ps2/bf2_dag1_soldiers.csv` 5 min at 2 Hz, `ps2/bf2_dag1_d1_units.csv` 8 min at 10 Hz with the
follower; the state `pcsx2/ref/states/bf2_dag1_ai16_gcw_sideselect.p2s` is kept): 7 lives begin at (-20.6, 3.9, -45.1); **7 of 7
walk south** along x -19 to (-18.2, 3.7, -69.1), turn to hub 12 (-10.0, -66.2) and go down the ramp into the cave (follower nodes
(-19.7, -45.1) -> (-18.4, -59.8) -> (-18.2, -69.1): a corner walk round the body at (-15, -60)). Of 118,901 soldier samples none
is on the ground over the cave path (x -8..8, z -52..-40, y > 3). So the PS2 starts this bot at hub 12 - the FAR end of its nearest arc.

**Variants, one scratch binary** (`DAGS=<bits>`; off = main hash for hash; worktree `openbf-wt/dags-proto-scratch`, target `lead_tmp/dags-proto_target`):
| | v0 main | v1 `start_hub` in `retarget_phase_a` | v5 = v1 + the nearest arc's far end when the near end is off the storey |
|---|---|---|---|
| Dagobah, 16 seeds: 20 s+ recover spells at the spot | 27 (11 seeds) | 0 | 0 |
| lives from the node: ever over the cave / median life / gone in 12 s | 34 of 129 / 26.7 s / 22 % | 0 of 133 / 14.5 s / 40 % | 0 of 141 / 26.5 s / 10 % |
| ... at 12 s: south or in the cave (the PS2's way) | 46 | 3 | 108 |
| 16 seeds x 32 maps: rounds / recycles / kills / world deaths | 512 / 110 / 117,940 / 1,259 | 512 / 109 / 117,775 / 1,261 | 512 / 107 / 118,449 / 1,289 |
| B9 unexplained standing, Dagobah / mean of maps | 0.91 / 0.202 % | 0.00 / 0.163 % | 0.00 / 0.155 % |
| Dagobah recycles, 16 + 24 more seeds | 8 + 15 | 0 + 1 | 0 + 0 |
| BF2 Utapau recycles, 16 + 24 more seeds | 1 + 1 | 2 + 0 | **8 + 10** (a pen at (-186, 9.4, 135), `trace/uta_v5/`) |
| BF1 Naboo Plains recycles, 16 + 24 | 0 + 4 | 6 + 3 | 5 + 2 (the 0 -> 6 was one seed; the spot is in older sweeps of other branches) |
`ab/v{0,1,5}/seeds.md`, `tools/abcmp.py`, `more/` (seeds 17-40 on six maps). v1 is the build: clean on every map. v5 walks the
PS2's way here but is CAUSED-worse on Utapau - not built; the PS2's arc-end start is card LOC3B's question (note there).
**Named:** the rule is OURS (the port's level rule, deviation D17, applied at the entry that missed it). The PS2 starts this bot at
hub 12; the port after the fix at hub 10 (level_hub's nearest by score: 21.6 against 24.3), so its bots from this node walk north
over open ground and live 14.5 s at the median against 26.7.
The card's two other places are not this fault: (10.6..19.8, -3.9, -43.8..-52.3) is cp4's own capture region (the spells there are
`guard` / `capture`), and (35.7, 3.7, 149.1) is cp1's. Not done here, by the card's own line: the endless recover / travel cycle
(RLOOP) and the all-maps node check (SPNC); the hunt destination in `retarget_phase_a` still takes the bare locate (its twin in
`retarget_hunt` has `level_hub`) - switch `DAGS=2` in the scratch binary, no effect on Dagobah, not measured on all maps.
