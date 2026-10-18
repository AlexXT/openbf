---
id: GRAIA
title: The weapon choice, from its audit: 8 of 12 mutations pass every test (all four rules of the second commit), three PS2 rules the port has differently ("plenty" is the load a weapon was issued, BF1 scores one shot, BF2 does not choose a reloading weapon), and the bots' grenades put Tantive IV's team kills at 5.1 % of kills (2.0 % without them; the bar is 3 %)
milestone: M3
value: 5
phase: build
status: done
deps: GRAI
files: engine/crates/core/src/ai/arsenal.rs,engine/crates/core/src/ai/arsenal/tests.rs,engine/crates/core/src/weapon.rs
declares: sim; sweep; arena
needs:
effort:
branch: graia
resume: landed e9315326 2026-10-12
created: 2026-10-07
alpha: yes
---
## Why
AUDIT GRAI 2026-10-07: PARTLY (`engine/artifacts/loop/audits.md`; evidence `engine/artifacts/lead_tmp/audit-grai/README.md`,
`analysis/re-grenades.md` 7.5, report `reports/20261007_1735_audit_GRAI.md`). The landing's numbers reproduce byte for
byte and its rules mostly hold against both ELFs and four PS2 captures. What does not:
1. **Tests.** 8 of 12 source mutations pass all 567 core tests: all four rules of the second commit (the stand to
   throw, the clear arc, MinRange at the landing, the head start from the own grenade) and four of the first's (BF1's
   plenty threshold, the explosive claim in `think`, `GROUP_RADIUS` 9 -> 13, the lead). The thrown score is caught only
   by a fixture's premise.
2. **Three PS2 rules `arsenal::choose` has differently** (re-grenades 7.5; read by the audit, no verifier yet):
   "plenty" is the load a weapon was ISSUED (the port asks the rounds left - a rifle down to 20 / 2 rounds gives way to
   the pistol - and `the_choice_prefers_range_then_plenty_then_score` pins that reading); BF1's score is one shot's
   damage (the port uses BF2's rate on BF1 maps: a rebel sniper 20-24 m from his target takes the pistol, rate 200
   against 160, where 160 a shot against 40 keeps the rifle); BF2 does not choose a weapon that is reloading (state 4)
   - its bot fights on with the pistol. Measured, each is small: see Acceptance.
3. **The grenades' cost to friends.** Main, 8 seeds x 32 maps: 936 team kills + 87 self kills with 8,392 throws; with
   the grenade never forced (the weapon choice alone) 608 + 0. Team + self kills of kills per map: bf2_tan1 5.1 % (2.0 %
   without the grenades; 4.3 % with the card off), bf2_dea1 3.3 %, bf2_cor1 and bf2_pol1 3.0 %. BOTD's bar B4 is 3 % on
   every map; without the grenades every map is at or under 2.9 %. The landing's "1,098 -> 913" is the net of the mask
   (no rockets at infantry) and the grenades.
Also measured, for whoever weighs them: the stand to throw costs 0.54 points of contact20 on main (paired t -2.8) and
saves 16 self kills in 256 rounds; the clear-arc / MinRange tests halve the self kills (172 -> 87) and save 142 team
kills; the head start halves them too (160 -> 87). On today's main the card raises no recycles (67 against 75 off).

## What to build (each step its own commit)
0. One verifier pass over re-grenades 7.5's corrections 1-4 (the audit's 24 agents went elsewhere).
1. Tests until each of the 12 mutations fails one: `lead_tmp/audit-grai/tools/mut_run.sh <id> <path> <file>` with the
   replacement files `lead_tmp/audit-grai/wf/M01..M12/` (the mutation worktree is at 3439714: re-make the files on
   main if `arsenal.rs`, `mover.rs` or `behave.rs` have moved). A wall between bot and target for the clear arc; a
   target walking in for MinRange and the lead; a second bot of the team for the claim; a soldier at 10-12 m for the
   radius; BF1 rules with 3-20 rounds for the threshold; the sim test without `all_stand()` for the stand.
2. "Plenty" from the issued load in both games (`WeaponState` has no such field: the audit's `cap`,
   `knob_main2.patch`); the pinned "two rounds are not plenty" goes.
3. BF1's one-shot score under BF1 rules (`per_shot` in the same patch; PROVISIONAL: the explosion's falloff at 2 m,
   B1 `FUN_00221530`, is taken as 1).
4. BF2 rules: a reloading weapon is not chosen (bit 512 in `knob_main5.patch`; state 5 is not identified -
   PROVISIONAL, reload only).
5. The friendly cost: first the PS2's number (`lead_tmp/ffln/run_tk.sh` on BF2 Tantive IV or the Death Star: team
   kills of kills; the port's bar is the bot design's, not the PS2's 12.1 %), then a rule - no throw with a friend
   within the blast's outer radius of where the grenade is to land (BF1's own group gate asks for no friend of the
   thrower within 9 m of the target) - as a named deviation if the PS2 has none.

## Acceptance (run and paste into the report)
- The 12 mutations: 12 of 12 fail a test (today 4).
- Steps 2-4, each against its parent, 8 seeds x 32 maps: no map's finish count drops; trigger pulls by family and
  recycles in the report. The audit's "before" (README table): 128 moves 42 of 256 rows and nothing by over 2 %;
  256 moves every BF1-rule row, pistol pulls on BF1 maps -4 %, recycles 67 -> 81 (take 24 more seeds on the maps
  that move before believing either way); 512 on Kamino / Polis Massa: slot 1 in the hands 10.1 -> 10.9 %, 7.4 -> 8.3 %.
- The PS2 rows re-taken on the last commit with `lead_tmp/audit-grai/ps2/port_slots.sh`: slot 1 in the hands no
  further from the PS2's 14.0 % (Kamino) / 11.7 % (Polis Massa) than today's 10.1 / 7.4; a throw per bot-seconds on
  Polis Massa within 25 % of the PS2's 347-363 (today 406).
- Team + self kills of kills at most 3 % on every map over 8 seeds (today bf2_tan1 5.1, bf2_dea1 3.3) - or step 5's
  PS2 number in the card with the reason the port keeps more.
- Gates: `sim; sweep; arena` declared and viewed.

- 2026-10-09 12:45 planner. **value 4 -> 5 and into BOTV's deps** (the audit left it as the planner's call). BOTV's bar B4 (team + self kills at most 3 % of kills on every map) is missed on BF2 Tantive IV (5.1 %) and the Death Star (3.3 %) by the bots' own grenades, and GRFL (296fa0c) has since put team kills up 9 % in the sweep (1,031 -> 1,120 over 8 seeds: grenades now go off on decks). Take step 5 (the PS2's rate on Tantive IV, then "no friend where it lands") on today's main: re-measure the 5.1 % first, it is from before GRFL. Undo: `loopctl.py set GRAIA value=4` and drop it from BOTV's `deps`.
- AUDIT FGHTA 2026-10-09 13:40 (for step 5 and bar B4; `engine/artifacts/lead_tmp/audit-fghta/re/main_32seeds.txt`): card FGHTA's lower stop chance for circlers (1 in 6 -> 1 in 14.4 a think) raises team kills on main 296fa0c, 32 seeds x 32 maps, per-seed paired: 120.1 -> 139.8 a seed with the stop chance alone (t +4.8), 133.0 with both of FGHTA's rules (t +4.0); 1.65 -> 1.82 % of kills. Per map, both rules off | main: bf2_cor1 2.63 | 3.24 %, bf2_myg1 2.66 | 3.01, bf2_mus1 3.38 | 4.20, bf2_dea1 3.13 | 3.36, bf2_tan1 5.35 | 5.90, bf2_pol1 3.74 | 3.19. A bot that stops less on its ring walk crosses more lines of fire: take the numbers on main AFTER FGHTA as this card's baseline (the switch `AUDIT_FGHTA=2` of `lead_tmp/audit-fghta/knob_main.patch` gives the old chance back).
- 2026-10-09 15:32 build (lane 1; report `reports/20261009_1532_card_GRAIA.md`). **Built on branch `graia`, not landed.** Step 0: verifier 8 claims, 7 supported, 1 partly (BF2's "supplies" bonus doubles ammo[2]). Step 1: 12 of 12 mutations fail a test. Steps 2-4, each against its parent over 8 seeds x 32 maps: no finish count drops; team + self kills 1,204 -> 1,199 -> 1,161 -> 1,128. Step 5 (no grenade with a friend within 12 m of the target, or where one walks to): 1,128 -> 659 over 8 seeds, 24 seeds 1.96 -> 1.19 % of kills and **no map over 3 %** (tan1 6.7 -> 2.9; with no forced grenade at all tan1 is 3.1). Acceptance MISSED: a throw per bot-seconds on Polis Massa 384 with rules 2-4, 710 with the friend rule (PS2 347-363; 9 m without the lead gives 462 and tan1 4.1 %); slot 1 in the hands 9.8 / 8.6 % (PS2 14.0 / 11.7; main 10.1 / 7.4). The PS2's Tantive IV team-kill share was not taken (load 230-300). Refused at the branch gate by fidelity noprog@bf2_geo1 3 -> 12: noise on a baseline from main 3439714 (see `resume:`).
- CLSE 2026-10-09 16:58 (a note for the weapon choice; `analysis/re-soldier-ai.md` 17.1, `engine/artifacts/lead_tmp/clse/out/table_{ps2,port_main}.txt`): **inside 10 m of its aimed enemy a PS2 BF2 bot holds a rifle for 52-55 % of the time, the port's for 30-31 %** (pistol 15-22 % against 30-34 %, shotgun 8-9 % against 11-17 %; two new PS2 captures with the weapon in hand read every sample, the port's main 177140d over 8 seeds, Kamino and Polis Massa). A pistol in the port deals 32-41 hp per aimer-second by its own hits, a rifle 40-67. Whether that is the choice rule or who gets close (class mix) is not split: re-take it by class when this card's rules are in (`lead_tmp/clse/tools/split_port.py`, `split_ps2.py`).
- 2026-10-12 09:52 land BSPN: BSPN landed (main 2d1fb60) on an approved `stand20_pct@bf2_dag1` 0.36 -> 1.08, and `[fidelity]` still holds the 8ad30f4 baseline. Rebased on main this branch will read that move as its own until the planner re-takes the baseline - a refusal on that one number is not this card's.
- 2026-10-12 10:26 land GRAIA: LANDED, main e931532 (report `reports/20261012_1026_land_GRAIA.md`). Team kills of kills at most 2.11 % (seeds 1-8) / 2.39 % (9-16) on every map, 32 / 32 maps finish 8 / 8. Left open for the planner: a throw per bot-seconds on Polis Massa 710 against the PS2's 347-363, slot 1 in the hands 9.8 / 8.6 % against 14.0 / 11.7, the PS2's Tantive IV team-kill share not taken.
