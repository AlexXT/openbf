---
id: ARVS
title: Arena head-to-head: tier X on one team against tier X-1 on the other (kill share, ticket margin, 24 seeds) - the arena today sets ONE tier for both teams, so its kills / min measures how deadly the fight is, not which tier wins (card TIER: Elite's extra movement when hit lowers BOTH sides' kills)
milestone: M3
value: 4
phase: build
status: todo
deps:
files: engine/crates/game/examples/arena.rs, engine/crates/core/src/sim/state.rs, engine/crates/core/src/skill.rs
declares: arena
needs:
effort:
branch:
resume:
created: 2026-10-19
---
## Why
Card TIER (2026-10-19, report `loop/reports/20261019_1535_measure_TIER.md`): `arena` puts one `Sim::difficulty` on both teams. Above Normal the tiers do not separate on Tatooine 2 (24 seeds) and on the flat field Elite is BELOW Hard (kills / min 60.8 vs 77.5, hit rate 0.394 vs 0.465, 22 of 24 seeds), because Elite is harder to hit as well as a better shot: its REACT weights (moves more when hit), its view 1.5 and reaction 5 lower the OTHER side's hits too. A same-tier fight cannot show that Elite beats Hard; card TCMP's acceptance ("Easy < Normal < Hard <= Elite on kills a minute") is judged on this instrument.
## What to build
- A per-team tier in the sim (`Sim::difficulty` stays the default; an optional override per team that every `sim.difficulty.tier()/skill_at()` reader takes by the unit's team). DIFB's "teammate shooting tier" needs the same thing: build it once.
- `arena <ticks> <seed> <map> vs <tierA>:<tierB>` (and `vs all`: easy:normal, normal:hard, hard:elite, both sides swapped so the spawn side cancels): per side kills, deaths, hit rate, ttfk; a `beats:` line per pair.
## Acceptance
- A null: `vs normal:normal` prints the same hash as today's `arena ... normal` (the override unset when both are equal), and the gate's `arena 9000 42 tat2 all` stays byte-identical.
- The table: flat and tat2, 24 seeds, each neighbouring pair both ways, the kill share of the higher tier with its paired t (TIER measured the same-tier rows to beat: lead_tmp/tier/runs).
- A mutation: give tier B tier A's skill and the `beats:` line goes to a coin flip.