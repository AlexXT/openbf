---
id: TIER
title: Do the four AI tiers order by kills? arena_tat2's one seed flips with every AI landing (kills / min easy, normal, hard, elite: AIRC 18.0 23.4 26.2 28.6 yes; OFFG 16.0 25.6 26.6 24.4 no; STLVL 20.0 22.4 23.6 28.8 yes; STTAP 17.2 26.0 24.4 25.0 no) - measure 8 seeds per tier before anyone tunes a tier
milestone: M3
value: 5
phase: measure
status: todo
deps: CONT,BOTD,TPOS,GRAI
files:
declares:
needs:
effort:
branch:
resume:
created: 2026-10-02
---
## Why
The difficulty setting is the one AI knob the maintainer will turn with a controller, and the only thing that checks it is the
arena gate's `monotonic:` line: one 9,000-tick run per tier on one seed. Two reports in a row put "kills are no longer
monotonic by tier" under For the maintainer (OFFG, STTAP) and nobody owns it. Planner 2026-10-02 09:30.

## Evidence (`arena 9000 42 tat2 all`, line 3-7 of `arena_tat2_1.txt` in each baseline dir, read by the planner)
| baseline (`lead_tmp/loop_baselines/`) | easy | normal | hard | elite | `monotonic: kills` |
|---|---|---|---|---|---|
| 20261002_0354_AIRC, 0422_LVLF | 18.0 | 23.4 | 26.2 | 28.6 | yes |
| 20261002_0513_OFFG | 16.0 | 25.6 | 26.6 | 24.4 | no |
| 20261002_0705_STLVL (main's baseline now) | 20.0 | 22.4 | 23.6 | 28.8 | yes |
| branch sttap (`lead_tmp/sttap/gates/`) | 17.2 | 26.0 | 24.4 | 25.0 | no |
`hits` reads `no` in every one of them (hit rate easy 0.12-0.15, the other three 0.18-0.24 in no fixed order).
The order flips with each landing, so one seed cannot say whether the tiers differ at all above `easy`.

## What to do
1. On main, `arena 9000 <seed> tat2 all` for seeds 1-8 (the gate's own example; no code change; a known binary, so no
   memory cap needed). Per tier: mean and spread of kills / min, hit rate, time to first kill.
2. Say which neighbouring tiers are apart by more than the seed spread and which are not.

## Acceptance
- A table in the report and on this card: per tier, the 8-seed mean and min-max of kills / min and hit rate.
- A verdict per neighbouring pair (easy < normal, normal < hard, hard < elite): apart / inside the noise.
- If a pair is inside the noise or reversed in the means: one build card (value 4) naming the `Skill` fields that
  should separate them, with the PS2's values where `analysis/` has them; else close this card and add one line to
  `FEEL_NOTES.md`. Either way propose whether the gate's `monotonic:` line should be an 8-seed mean (a note on the
  report; gate-script edits are their own card).

## Notes
- No `files:`: a measurement with the gate's binary. Do not tune a tier in this card.

- Planner 2026-10-02 14:45: `deps: CONT`. CONT's build replaces the commander's scoring; eight seeds per tier measured before it lands would be stale the same day.

## Planner 2026-10-02 17:45 - waits for BOTD
This card changes the soldier brain (`core/src/ai`) that the bot programme is about to redesign (the maintainer, 2026-10-02: "are we just implementing so many bandaid fixes to a fundamentally flawed bot algorithm"; BOTD: no rewrite before he has read the design). Held so no lane builds on the old brain meanwhile; BOTD adopts, rewrites or closes it when it cuts its build cards. Undo: remove BOTD from `deps:`.

## BOTD (2026-10-03) - stays the measurement; its build is card TCMP (bot-design.md section 7)
Run it on main BEFORE TCMP changes a tier. In BOTV's deps.
- 2026-10-05 (the maintainer, via Claude): the difficulty design now lets the humans' AI teammates sit on a LOWER tier than the enemy's bots (cards DIFH / DIFS / DIFB). Measure the four tiers so that a team can carry a tier of its own, and report the shooting columns (aim error, first shot, bursts) separately from the movement and position columns.
- 2026-10-05 planner - **The maintainer has read the bot design: "BOTD read, go" (INBOX 2026-10-03 14:08). All eight decisions take the note's defaults; the 16 deviations D1-D16 are all on. This card is released (BOTD is done).** Decisions 2 and 8 are card TCMP's (no damage bonus at Hard / Elite, one think rate): measure today's tiers WITH the bonus still in, so TCMP's change is a before / after.
- 2026-10-05 planner: value 3 -> 5: every card BOTV waits on is ranked with the chain, so the bot programme runs in the maintainer's order ahead of the fours (his notes of 2026-10-02 16:30 / 17:40 and 2026-10-03 14:08). Undo: `loopctl.py set TIER value=3`. `deps` gains TPOS, GRAI: the design's build order (bot-design.md 12, row 13) takes this measurement right before TCMP, on the bots TCMP will change, not on today's. Undo: `set TIER deps=CONT,BOTD`.
- 2026-10-07 (the maintainer, via Claude): the reaction to a hit is a tier ladder (card REACT): write the stay / strafe / roll / new-position weights per tier next to the evasive-step and roll-when-hit columns, so Easy mostly stays and Elite mostly moves.
