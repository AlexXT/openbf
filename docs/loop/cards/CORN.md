---
id: CORN
title: Aim points in walls: the corner search walks a leg THROUGH a barrier it ignores (BF2 Tantive IV's leg 20 > 79: 36 of the worst door's 45 stalls, a corner inside the wall beside the pod room door), and a hub point or corner can lie behind a door's jamb - find the rule that cures them without the all-maps cost the first prototype had
milestone: M3
value: 4
phase: measure
status: todo
deps: DOORS
files: engine/crates/core/src/nav.rs
declares: sim; sweep; arena
needs:
effort: high
branch:
resume:
created: 2026-10-12
alpha: yes
---

## Why
Filed by card DOORS's measure pass (2026-10-12). After MOUTH2 and DOORS a door's LEAF no longer turns bots away; what
is left near doors is the point a bot walks at lying IN A WALL. On BF2 Tantive IV - the worst door map, 52 door
stalls per 20 door bot-minutes against Kamino's 1 - bots at hub 20 turning west walk at a path corner inside the wall
beside the pod room door, meet the corridor, commit a side and go 22 m round through the door. It is what the maintainer's
"they turned and walked into the wall behind" looks like on the maps he has not played yet.

## Evidence (main 4dbb460; kit `engine/artifacts/lead_tmp/doors/`, README there)
- **Tantive IV leg 20 > 79, traced** (`tools/urows.py` on `snap/bf2_tan1_main/bf2_tan1_s64_t1800.json` unit 25, t
  11.5-22; `tools/meet.py`: 22 of the map's 70 "going round" door stalls in 8 seeds walk at this one corner). The
  straight leg crosses `barrier219` (a 4 x 1.6 m barrier askew in the corridor at (-282.7, 149.8)). The bot's point of
  hub 79, (-298.89, 148.96), is 0.90 m outside `barrier33` (the wall north of the corridor) - inside the search's
  radius (`DETOUR_RADIUS` = the capsule's 0.5 + 0.5 = 1.0 m), so `NavGraph::leg_ignores` drops barrier33 for the
  whole leg, and the chain round barrier219 comes back as (-278.90, 151.68), (-282.84, 154.05): both INSIDE
  barrier33. Printed by the scratch switch `DOORS_DET=-277,148,3` (`scratch_knobs.patch`): 2 of 4 searches from hub
  20 answer so (goal points 0.90 and 0.40 m outside the barrier); a goal 1.27 m outside gets "no chain" and the bot
  walks straight. The PS2 tests containment with the unit radius alone (`ai.rs`, `DETOUR_RADIUS`'s note): by that
  test the 0.90 m goal is not ignored and the 0.40 m one still is.
- **Leg 24 > 25** at `rpodroom2`: corners (-294.4, 175.9) and (-298.7, 175.9) against `tan4_bldg_pod_b` (13 round
  stalls) - not traced further.
- **Aim points behind a jamb** (a hub point or a corner outside a door's opening; the maintainer's "centre line" ask, DOORS
  rule (a), not built): Tantive IV's tech-room doors (legs 101 > 102, 96 > 21, 12 > 13, 14 > 15: about 12 door stalls
  in 8 seeds), Kamino `kam_bldg_podroom_door24` (leg 88 > 87, the bot's point of hub 87 4.0 m off the centre of a
  7.5 m door; 2 stalls in 26 near visits), the Death Star's leg 4 > 1 corner (-56.1, -77.8), 6.9 m off the centre of
  a 10 m blast door (4 of the 5 leaf meetings that still end badly after DOORS, one bot).
- **A first prototype is measured and NOT good enough** (`DOORS=2` in `bin/rto_k2`: a corner inside the plain
  rectangle of a barrier the leg ignores is no node): Tantive IV door stalls 192 -> 140, its round class 70 -> 32,
  the map's P-stalls 1000 -> 913 (8 seeds x 3 min) - but all 32 maps move, and over 8 seeds x 2 min of snapshots
  S-stalls 2566 -> 2581 (paired t +0.15), P 8977 -> 8854 (t -1.19), Utapau's S 391 -> 471, Yavin 4's 70 -> 91,
  Endor's 139 -> 106, Kashyyyk's 82 -> 61; recycles 39 -> 43 over 8 sweep seeds (Tatooine 8 -> 14, Kashyyyk 9 -> 4):
  `out/ab_k2.txt`, `sw/a_0` against `sw/a_2`.

## What to do (measure first)
1. Census the corner search's own output on all 32 maps (RULES: census the PLAN at its point of use): of the
   crossing legs, how many get a corner inside ANY barrier's plain rectangle, how many of those inside one the leg
   ignores, per map; and how many legs ignore a barrier only because of the 0.5 m the port adds to the PS2's radius.
2. Variants behind switches in one binary (null = main byte for byte): (i) the prototype; (ii) containment for
   `leg_ignores` with the capsule radius alone (the PS2's test), crossings as now; (iii) an ignored barrier is
   ignored only for the segment that ends at the point it holds; (iv) for a leg that crosses a door's opening, the
   aim point clamped into the opening (DOORS rule (a)) - judged by Tantive IV's tech-room doors.
3. Trace Utapau and Yavin 4 under the winner before believing a gain (the prototype's rise there is unexplained).

## Acceptance
- Tantive IV: door stalls per 20 door bot-minutes (`lead_tmp/doors/tools/census.py`, 8 seeds x 3 min; main 4dbb460
  60.7, bc10b82 51.3) at most 35, and the leg 20 > 79 corner cluster (`tools/meet.py`) gone.
- All 32 maps, `mouth/tools/ab.py` over 8 seeds: S-stalls not up (paired t under +1), no map's S up by over 20 %
  without a trace; 24 seeds x 32 maps paired: rounds 768 / 768, recycles, lost and floor deaths not worse; the maintainer's
  Kamino scene not worse (run it on the FIRST variant that shows a gain).
- A test per rule from the Tantive IV barriers' own numbers (barrier33, barrier219, the two end points), each
  mutation-checked.

## Notes
- 2026-10-12 DOORS: the kit's `tools/classify.py` splits a door's stalls by cause; 48 % are bots standing in a
  firefight (`hold` 2 / `fight` within a second) - count them out before reading a door's stall number.
- `core/src/nav.rs` is also OBSV2's / TGAP's / FLNK's neighbourhood (`route.rs` calls the search): check what is in
  flight before building.
