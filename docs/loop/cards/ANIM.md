---
id: ANIM
title: Soldier animation: upper-body blend layers, the droid anim banks, BF2 `_lower` clip naming, the first-person rig — `game::animator`'s table is PROVISIONAL
milestone: M1
value: 5
phase: re
status: done
deps:
files:
declares:
needs:
effort:
branch:
resume: RE done 2026-10-05: analysis/re-soldier-anim-layers.md (35c93e6). Build split: ANIMS (bake the sets) -> ANIML (two players + action layer) -> ANIMA (alert, aim), ANIMM (movement, rate, twist); leftovers ANIMX.
created: 2026-09-25
alpha: yes
---

## Evidence
analysis/PLAN_LOG_2026.md §ARCHIVED 2026-09-25 HANDOFF-OPUS.md HOLD 2 and 3: upper-body 16/17-track clips vs full-body; the `FUN_0022d828` / `FUN_00282f78` anim-set giants;
`cis_inf_sbdroid` rig; `humanfp`. Visual tuning (blend times, walk/run threshold) is fine if labelled PROVISIONAL.

## Acceptance
One RE answer per session, in `analysis/` ([O]); build cards for what moves the soldier's look (compare a PCSX2 pair).

## Notes
- UAUD1 (2026-09-29, analysis/unit-audit-bf1.md §3.7): each weapon names an `AnimationBank` (read by B1 `FUN_002864f0` 210216); BF1
  COMMON.LVL holds at least 23 `human_bazooka_*`, 20 `human_tool_*`, 20 `human_sbdroid_*`, 6 `human_pistol_*`,
  5 `human_bdroid_*`, 2 `human_grenade_*` clips beside the rifle set (`lead_tmp/unit-audit/anims.py`). With the primary
  alone, 10 of the 34 fielded BF1 classes play the wrong set: 7 launcher classes (bazooka), the SBD (sbdroid - on the
  PS2 both forearms out, `lead_tmp/unit-audit/pcsx2/10_sbd_idle_crop.png`), the pilot and assassin droids (bdroid).
  Proposed value 2 -> 4 (the SBD is every BF1 CW battle's most common CIS unit).
- UAUD2 (2026-09-29, analysis/unit-audit-bf2.md §3.7): BF2 CIS.LVL ships the droids' own sets: `sbdroid` (27 clips, 17 named
  `sbdroid_rifle_*`) for the SBD (SkeletonName sbdroid, every CW battle's most common CIS unit) and `bdroid` (11, 7
  named `bdroid_rifle_*`) for the assault / engineer / assassin droids; the officers carry 2-clip overlays
  (`clonecommander`, `bothanspy`, `impofficer`, `magnaguard`). Bazooka-bank primaries: the 9 rocketeers and the clone
  commander's chaingun; pistol-bank: the Imperial officer and the MagnaGuard. The port bakes 24 of its 34 names on
  every BF2 soldier: stand_walkbackward, the six prone clips, the two getdown_prone and die are missing from human_0/1
  (`assets_baked/side_bf2/soldiers/*.anims.ron`).

## Planner note (2026-09-30, M-ALPHA)
`alpha: yes` — soldier animation reads on camera. Value 2 → 3. Showcase maps: BF1 NAB1, BF1 TAT1 (the port's `tat1`), BF1 YAV2, BF2 UTA1, BF2 MYG1, BF2 HOT1 (PLAN §Milestones M-ALPHA).

- FOLY RE (2026-10-02, `analysis/re-foley.md` 4.4, `[O]`): the PS2's MOVEMENT clip chooser and playback rate are
  traced. BF1 `FUN_0022e990`: ids 5 / 7 walk / run forward, 6 / 8 backward, 9 / 0xb left, 10 / 0xc right, 0 idle
  under 0.3 m/s; forward runs above 5.5 m/s (5.0 to stay), strafe right above 5.0 / 5.5, and - sign slips -
  backward and strafe LEFT always take the run clip (live: left at 4.4 m/s plays run steps, right at 4.5 m/s walk
  steps); speeds are divided by the class's skeleton scale first; the clip's rate is x (0.95 v / v_clip + 0.05),
  so the feet do not slide. BF2 `FUN_0027ed18` + `FUN_0024f350`: walk / run by the soldier's speed mode, run above
  0.78 x MaxSpeed (0.71 to stay). The port's `animator::RUN_SPEED` 4.5 m/s and constant rate are the provisional
  stand-ins; a build piece for this card (the FOLY build keeps its own clip id and phase for the sounds).
- 2026-10-04 (the maintainer's playtest, via Claude; value 3 -> 5 at his word): "shooting, walking, and standing animation is not the exact same for every unit. It varies depending on weapon they're holding, so like where it sits on their shoulder ... the sbd doesn't hold a gun so it sticks its arm straight out and fires from its hand, not holding some invisible gun. When walking and shooting, the units need to do the shooting motion while still walking forwards. Right now when walking it doesn't change animation to shooting." Three things to read and build: (1) the per-weapon pose sets (rifle / pistol / bazooka / tool ... stand, walk, fire), (2) the super battle droid's own arm-fire set, (3) the upper-body fire layer over the walk.

## Result (2026-10-05, lane 2, RE done - docs 35c93e6)
`analysis/re-soldier-anim-layers.md` [O], verifier 51 claims: 41 SUPPORTED, 10 PARTLY folded in, 0 NOT.
- Per-weapon poses: two clip players; the lower body plays the rifle's movement clip, the 13 upper bones (BF2: all but
  root, pelvis, legs) the carried weapon's set - BF1 nine `AnimationBank` sets with a fallback table, BF2 one set per
  (skeleton family, weapon animation class). Rebuilt from the discs and equal to PS2 memory: BF1 979 of 979 slots,
  BF2 665,176 of 665,176 over 176 battle states.
- The super battle droid: BF1 set `sbdroid` (25 `human_sbdroid_*` clips), BF2 family `sbdroid` (25 `sbdroid_rifle_*`).
- Firing on the move: the shoot / charge / reload clip plays on the upper body, the legs keep the movement clip; PS2
  memory shows it on five soldiers (BF1 155 of 155 and BF2 182 of 182 posed soldiers match the rule).
Build cards: ANIMS (5) -> ANIML (5) -> ANIMA (4), ANIMM (4); unread leftovers ANIMX (2, post-alpha).
