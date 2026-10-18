---
id: CLSE2B
title: BF1 bots aim with NO error vector (the PS2's rule, re-soldier-ai 18.5) - built, measured and held back by CLSE2: with it every BF1 map's bots kill from farther off (an enemy within 20 m 10.9 -> 8.1 % of unit time, the fidelity gate's all-map value 11.15 -> 9.66) because the port's far fire is already over the PS2's; land it together with the far fire's fix
milestone: M3
value: 5
phase: build
status: blocked
deps: CLSE2,CLSE4
files: engine/crates/core/src/ai/gunner_foot.rs,engine/crates/core/src/ai/state.rs,engine/crates/core/src/ai/behave.rs,engine/crates/core/src/perception.rs
declares: sweep; arena; sim
needs: maintainer — BF1's no-error aim (the PS2's rule) lowers BF1's contact share 11.95 -> 9.24 % (all maps 12.73 -> 11.46, over the gate's 1.0) while damage per aimer-second moves toward the PS2 in 8 of 8 bins and the PS2's own Kashyyyk Docks contact is 5.2 % (port 6.7 -> 5.1): land it with an approval for contact20_pct on the BF1 maps [default: yes, the rule is the PS2's and BF1's contact bar has no BF1 capture behind it], or hold it for 2-3 more BF1 PS2 captures first?
effort:
branch: clse2b
resume: The behaviour gate's refusal is CAUSED: contact20_pct on the BF1 maps (card text, 'Built and measured'). On the maintainer's yes: planner adds [approvals.CLSE2B] for contact20_pct@(all maps) and the BF1 maps that fail after the baseline re-take, clears needs, sets status=parked; land session: rebase clse2b, land.py. On 'more captures': PCSX2 BF1 captures of 3 more maps (SAIR's sampler), tools/ps2_bf1_contact.py, then decide. The stale gate baseline (5fbbbd4) let this branch PASS once - do not land on that. (planner 2026-10-13 07:10: reworded only - the picker calls a planner every 3 sessions while a blocked card's resume or needs holds the gate group's name or the claim keyword; a card that waits on the maintainer must not. The old text: lead_tmp/planner_1013b/CLSE2B.before.md)
created: 2026-10-12
alpha: yes
---
## Why
The PS2's BF1 bot aims at its target's lead point with no error at all (B1 FUN_00233200 -> FUN_00232a70; a still PS2
bot's view ray passes its target at a flat 0.06 m at every range - analysis/re-soldier-ai.md 18.5). The port's BF1
round still puts D10's whole radius (0.6-1.4 m, the shooter's own tracked error, a direction drawn per think) on it:
gunner_foot::seen_aim's BF1 branch, whose header says it is known wrong. Card CLSE2 built the PS2's rule, measured it,
and took it out of its landing (branch clse2: commit 8ba75a7 has it, d57ab7d holds it back).

## Evidence (2026-10-12, lead_tmp/clse2: sw/, gates_fid/, README)
- 8 seeds x 32 maps, the rule alone (CLSE2=1 of the switch binary, proto_switches_d096ae7.patch): rounds 256 / 256,
  kills a minute on the 15 BF1-rule maps 33.4 -> 34.9.
- hp per aimer-second at 0-10 / 10-20 / 20-30 / over 30 m (clse/tools/split_port.py, BINS3=1, sw/b1_v0 and b1_v1):
  Kashyyyk Docks 20.6 / 17.5 / 18.4 / 15.9 -> 32.7 / 22.6 / 23.2 / 19.9 against the PS2's 27 / 28 / 19 / 13;
  Naboo Plains 14.1 / 17.4 / 17.8 / 13.8 -> 16.2 / 22.6 / 22.5 / 17.8 against 28.6 / 24.7 / 17.5 / 17.6.
  Nearer the PS2 inside 20 m, PAST it beyond 20 m (shots a second and hits per shot: 2.0-2.1 and 0.14 -> 0.17 beyond 30 m).
- The fidelity gate on the branch with the rule in (gates_fid/fidelity/fidelity.md): contact20_pct all maps 11.15 ->
  9.66 (tolerance 1.0); BF1's maps 10.87 -> 8.12, every one down (Kamino 6.60 -> 3.06, Naboo Theed 11.28 -> 6.65,
  Rhen Var Harbor 14.31 -> 9.81, Bespin Cloud City 22.90 -> 17.56), BF2's 11.43 -> 11.20. CAUSED by the rule: its code
  is the path, and no BF2 map moves.
- So the rule is right and the port's BF1 far fire is wrong: a bot that aims exactly kills at 30 m+ what the PS2's
  bot does not. What the PS2's BF1 bot does there is not measured (shots a second, hits per shot, the weapon's spread
  x its two skill multipliers, re-ai-difficulty 2.2; 18.9's [H]: BF1's team mates fire at skill 4 on Normal).

## What to do
1. Measure the PS2's BF1 far fire (SAIR's captures lead_tmp/sair/ps2/bf1_*, clse2/tools/bf1_split.py; a BF1 weapon
   sampler like clse/tools/run_w.sh does not exist yet): shots per aimer-second and health fallen per shot by range,
   beside the port's. Card CLSE4 does this for BF2 - share the method.
2. Build the far fire's fix and this rule in ONE branch: seen_aim's BF1 branch returns the lead point; AiState::err /
   err_pos, track_error, SALT_ERR_DIR and ERR_NO_TARGET go (commit 8ba75a7 is that state, tests included).

## Acceptance
- BF1: hp per aimer-second on Kashyyyk Docks and Naboo Plains within 20 % of the PS2's in all four range bins.
- The fidelity gate passes: contact20_pct on the BF1 maps at or over main's (10.87 on the 2026-10-12 baseline).
- 256 of 256 rounds; a test that fails when a BF1 bot's aim point is not the lead point.
- 2026-10-12 18:49: CLSE4 is on main (a490579): the far fire's two rules are in for BF1 too (bursts 3..6 with a flat 2.0 s, the
  trigger inside MaxRange). BF1's numbers on that main, 14 maps x 8 seeds: contact20 11.48 %, deaths with an enemy
  inside 10 m 25.7 %, kills a minute 30.9 (`lead_tmp/clse4/sw/br_b1`, `out2/contact_spells_bf1.txt`) - re-measure
  this card's build against those, not against the numbers in its text.

## Built and measured 2026-10-12 19:08 (branch clse2b 01572b9 = main bc10b82 + the rule; NOT landed; lead_tmp/clse2b/README.md)
Main a490579 + observer against the same + the rule, 14 `bf1_` maps x 8 seeds, the first 540 s; 112 of 112 rounds.
- hp per aimer-second, 0-10 / 10-20 / 20-30 / over 30 m (PS2 = clse2/out/bf1_split_ps2.txt, Kashyyyk's two captures pooled):
  Kashyyyk Docks main 19.3 / 15.7 / 16.0 / 10.2 -> 28.8 / 21.4 / 20.8 / 12.7, the PS2 25.1 / 31.9 / 19.9 / 12.4;
  Naboo Plains main 15.0 / 15.4 / 14.6 / 8.7 -> 15.2 / 18.9 / 18.8 / 11.1, the PS2 28.6 / 24.7 / 17.5 / 17.6.
  Nearer the PS2 in 8 of 8 bins, over it by 15 % at most (after CLSE4 the far fire is no longer over the PS2's);
  within 20 % in 4 of 8 bins (main 3 of 8) - bar 1 is NOT met by either build.
- An enemy within 20 m: 11.48 -> 9.09 % of unit time (14 maps); on the gate's rows 15 BF1-rule maps 11.95 -> 9.24, down
  on 13, all maps 12.73 -> 11.46 (tolerance 1.0), BF2's 17 maps 13.41 -> 13.41. Bar 2 is NOT met. CAUSED (the rule's
  code is the path). `gates.py --groups fidelity` says PASS only because its baseline is main 5fbbbd4, before DEFS and
  CLSE4 raised contact - a land session would land this; do not until the question below is answered.
- Deaths with an enemy inside 10 m 25.7 -> 19.6 %, with none within 20 m 45.1 -> 51.6 %; kills a minute 30.8 -> 33.4;
  team kills 0.9 -> 0.5 %.
- The PS2's own BF1 contact (teams 1 and 2 on foot, tools/ps2_bf1_contact.py): Kashyyyk Docks 5.1 / 5.3 % in two
  captures (re-soldier-ai 15.5 has the same), 80 % of deaths with no enemy within 20 m, median 33-35 m - the port
  6.7 -> 5.1 %, 59.5 -> 67.2 %, 23 -> 25 m: there the rule moves the port TOWARD the PS2 and main is over it.
  Naboo Plains 13.6 % (one 143 s capture), 61 % and 25 m - the port 7.0 -> 7.3 %, 66.7 -> 68.8 %, 24 m: not moved.
  So "contact at or over main's" is BF2's bar; no BF1 capture supports it (two maps only: thin).
- Lead, not followed: the port's BF1 bots hold a living enemy as target 25-37 % of unit time, the PS2's hold an aim
  order at one 11-14 % (aimer-seconds over AI unit-seconds, same files) - the definitions differ (the port's target
  includes remembered contacts), so check that before reading it.

## Planner 2026-10-12 19:30 - the baseline is re-taken; the gate refuses this branch now, as the session said it should
`gates.toml [fidelity]` is main bc10b82 (after MOUTH2, DEFS and CLSE4; `lead_tmp/fidelity_rebase_20261012e/`): an
enemy within 20 m, all maps, 12.73 / 12.92 on seeds 1-8 / 9-16. This branch's own run (`lead_tmp/clse2b/gates_fid`,
seeds 1-8) reads 11.46: 1.27 under, tolerance 1.0. The stale PASS is gone; a land job would be refused.
CAUSED, so only the maintainer's yes writes the approval: STATUS question 40 (the session's recommendation and mine: yes).
The card stays `blocked` with `needs:`. Nothing else changed on it.
