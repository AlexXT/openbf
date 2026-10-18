---
id: FOFG
title: The sweep's off-graph share (fidelity offgraph_pct) counts every alive unit where it sits, a turret gunner at its seat and a pilot in its vehicle: count units on foot only and print the seated share beside it - card TURR's KAM1 +8.65 was 7.28 seated gunners
milestone: M0
value: 5
phase: build
status: done
deps:
files: engine/crates/game/examples/round_to_outcome.rs, scripts/sweep_seeds.py
declares: sweep
needs:
effort:
branch: fofg
resume: landed b04d6f5b 2026-10-16
created: 2026-10-16
alpha: yes
---

## Why
The fidelity gate's `offgraph_pct` (scripts/gates.py: the `N` locus share of the sweep's `shares` line, written by
`Shares::observe` in `game/examples/round_to_outcome.rs`) samples every alive unit once a second at its position, riders at
their seat. It was built (cards RCYC / OFFG) to catch soldiers who walk off the nav graph. A seated gunner on a building
turret, or a pilot driving across open ground, is not that - but it counts. Card TURR (bots man turrets, branch `turr`) was
refused on it: offgraph_pct@bf2_kam1 4.41 -> 13.06 and the all-map value 6.25 -> 8.21, of which 7.28 points on KAM1 are
gunners in their seats (`lead_tmp/turr/offg`, report `reports/20261016_0621_card_TURR.md`). Any vehicle or turret card
meets the same wall. Planner call 2026-10-16b (reversible): the share measures units ON FOOT; the seated share is printed
beside it so nothing is hidden.

## Evidence
- `lead_tmp/turr/offg/`: TURR's scratch foot-only counter (bin `rto_foot2` = branch + counter); its all-units value
  reproduces the gate's 13.06 exactly; seated 7.28, on foot 5.78 (boarding 0.88, within 12 m of a turret 1.50, elsewhere 3.40).
- gates.toml `[fidelity.metrics.offgraph_pct]` tol_map_abs 8.5, tol_agg_abs 0.6, dir down.

## What to build
1. `Shares::observe`: the locus counts (H / A / N / none) and their denominator take AI and player units ON FOOT only
   (no `mount`); a new `seated S %` field on the shares line = share of alive-unit samples in a vehicle or turret seat.
   Keep the line's existing field order so `sweep_seeds.py` / `bot_quality.py` parse it unchanged (check both parsers;
   add `seated` to sweep_seeds' parser only if a census wants it).
2. The module header (`//! shares ...`) and the `Shares` doc say what is counted.

## Acceptance
- Main's binary and the branch over the 32 maps x 8 seeds (seeds 1-8): every map with no vehicle or turret ever mounted in
  the round gives the SAME `locus` numbers to the printed digit (paste the diff of the 12+ such rows); every vehicle map's N
  share moves only by removing riders (paste the per-map table main / branch / seated).
- Scratch-apply the change onto branch `turr` and re-run its KAM1 8 seeds: offgraph@bf2_kam1 equals TURR's foot-only
  counter (5.78 +- 0.01, `lead_tmp/turr/offg`).
- One mutation: count riders again -> the KAM1-on-turr number goes back to 13.06.
- After landing, the PLANNER re-takes the fidelity baseline (every vehicle map's value moves: declared `sweep`; the
  fidelity rows that move go down = better, so the gate passes; say so in the report).

## Notes
Unblocks TURR (deps). Not oracle-guarded: round_to_outcome.rs is the sweep binary, not gates.py.
