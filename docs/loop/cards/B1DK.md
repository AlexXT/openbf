---
id: B1DK
title: Kamino's decks after LSUM: BF2 makes an EntityLight whose hardpoint the carrier lacks at the carrier's origin (Kamino's bunkers: 10 lights of r 20 in the middle of the decks; POL1 2, DEA1 1) and the bake skips them; BF1's sun-coloured specular pass is fed the linear sun colour (3-5 x too weak under Kamino's sun)
milestone: M1
value: 5
phase: build
status: done
deps: LSUM
files: engine/crates/bake/src/entity_light.rs,engine/crates/bake/src/map.rs,engine/crates/game/src/assets.rs,engine/crates/game/src/viewer/scene.rs,engine/crates/game/src/soldier_vis.rs,engine/crates/game/src/viewer/vehicles.rs,engine/CRATES.md
declares: goldens=maps/bf2_kam1.png,maps/bf2_pol1.png,maps/bf2_dea1.png,maps/bf2_uta1.png,maps/bf1_kam1.png,maps/bf1_kas2.png,maps/bf1_yav1.png,units/bf1_all_inf_pilot.png,units/bf1_cis_inf_bdroid.png,units/bf1_cis_inf_sbdroid.png,units/bf1_imp_inf_darktrooper.png,units/bf1_rep_inf_pilot.png,units/bf1_rep_inf_trooper.png,walkabout_stats.ron; rebake=maps
needs:
effort:
branch: b1dk
resume: landed 71c38556 2026-10-05
created: 2026-09-30
alpha: yes
defer: 
---

## Build (written by this card's RE pass, 2026-10-05; `analysis/re-kamino-decks.md`) - this section is the card now
**The RE's answer.** No single term lights these floors: five things separate the port's deck from the PS2's (the
doc's §0). The largest, the light sum's colour space with the ambient floor, is card **LSUM** (first). BF1's local
lights are card **B1LL** (after this one). This card builds the two that are Kamino's own:

1. **BF2: an EntityLight on a hardpoint the carrier's skeleton lacks is created at the carrier's ORIGIN** (§5; B2
   `FUN_001d4db8` / `FUN_00451e58`; RAM 13 of 13 lights). Each `kam_bldg_bunker1` carries two `kamlight`
   (181,172,163) and one `kamlight_blue` (2,50,100), r 20, not Static, half a metre from the centre vertex of its
   deck's fan; `kam_bldg_pod2` one. `bake::entity_light::attach` skips them (`continue` on a missing bone; a class
   with no skeleton too). On a BF2 map place them at (0,0,0) of the carrier instead. Also POL1 (`pol_prop_cavern`: two
   Static (188,234,255) r 8) and DEA1 (`dea1_bldg_beam_generator`: one r 24, StrobeRandom - steady here like every
   EntityLight, card LFLR). BF1's rule is the opposite (a missing bone keeps nothing): do not apply the origin rule
   to BF1.
2. **BF1's sun-coloured specular pass** (§7): `SpecPass::rgb_under` multiplies the sun colour by 255/128 but its three
   callers (`viewer/scene.rs`, `soldier_vis.rs`, `viewer/vehicles.rs`) hand it the LINEAR colour; the GS gets the
   SunInfo bytes raw. Encode before the multiply. Kamino: (0.105,0.181,0.281) -> (0.508,0.664,0.820).

**Evidence** (`engine/artifacts/lead_tmp/b1dk/`): `origin_ram.py` (6 of 6 carriers, 13 lights at their origins), `attach_census.sh` (which maps
change), `skel.py`; PS2 knock-outs `ps2/frames/k2__*.png`, `k2w__*.png` (no model point lights: the deck 4.5 m from
the post falls from 124/119/127 to 55/60/65; the three origin lights are worth 21-25 of that); prototype renders
`park/origin_gsum1/`, `var/k2w/`, `var1/`; sheet `sheet_prototype.png` (PS2 | main | prototype).

**Acceptance (run and paste; all on top of LSUM).**
- `PXC_LIGHTS=1 pose_render` on bf2_kam1 lists ten r-20 lights at the places `lead_tmp/bpnt/lights.py` prints for
  `ref/states/bf2_kam1_walk_cp2_cp3_b000.p2s` (three at (-60.95, 60.31, 303.49), ...); bf2_pol1 two at (-17, 23, -20);
  bf2_dea1 one at (-105.0, 49.98, -107.93). A bake test: a class whose model lacks the hardpoint still gets its light,
  at the origin, on BF2 and not on BF1.
- BF2 deck: `walk/cp2_cp3_b090` box (430,278,627,346) within 12 levels per channel of the PS2's 75/81/87 (main
  39/43/46; prototype 64/71/77 - 67/73/78), and `walk/cp2_cp3_b000` box (430,360,627,470) within 8 of 72/78/86. The
  turned pose `lead_tmp/b1dk/ps2/frames/k2w__ctl.png` (pose in `k2w__ctl.pose.json`), box (430,278,627,336), within
  15 of 97/102/110 (prototype 86/91/99).
- BF1 deck: the box (450,330,640,470) at `walk/cp0_cp2` and `walk/cp4_cp4` x b000/b090/b180/b270 within 14 levels per
  channel of the PS2's (the original card's 12 is met with the sum's form 2 and missed by 2 at bearing 270 with form
  1; prototype table in the doc's §4.2). The deck must change with the bearing: b270 at least 10 over b090.
- `scripts/parity.py`: bf1_kam1 <= 13.7 and bf2_kam1 <= 11.9 (LSUM alone: 14.48 / 12.28; main 19.29 / 16.71).
  The original card's "bf1_kam1 down by >= 2, bf2_kam1 down by >= 0.66" is met by LSUM already.

**Not this card.** The BF2 mid-floor box (256,278,627,346) of the old acceptance holds the PS2's own soldier (x
270-370, lit near-white): use the soldier-free box above. BF2's bearing-270 highlight (PS2 157/163/171, prototype
107/114/122) is the specular pass plus the HDR bloom it triggers: card GLOWB.

The text below is the card as the RE pass found it.

## Why
BF1 Kamino is the worst map of the parity board (26.62) and its floors are the largest term after the sky (KSKY) and
the sea (KSEA): the landing-pad decks read 1.5–2 × darker than the PS2's, uniformly over RGB, at every bearing.
From KAMR's pass, `analysis/re-kamino.md` §5.

## Evidence
- Deck box (450,330)-(640,470) at the stored walk poses, main 5c73167 (`lead_tmp/kamr/box.py`, renders
  `lead_tmp/kamr/parity_main_5c73167/render`): cp0_cp2 b000 PS2 (65,79,83) / port (41,46,52); b270 (85,102,116) /
  (43,49,58); cp4_cp4 b000 (73,82,92) / (42,46,52); b270 (85,102,116) / (43,49,58). The port's deck does not change
  with the bearing at all (41–43); the PS2's does (61–85, 148 where a highlight sits).
- BF2's same decks were 1.7 × dark until BPNT landed BF2's local lights (3c024fb): now (72,76,81) vs the PS2's
  (72,78,86) at cp2_cp3 b000. BF1 has no `lght` chunk; `analysis/re-point-lights.md` is BF2 only.
- BF1 KAM1 defines `whitelight` / `kamlight` / `kamlight_blue` (base `Light`) but the baked world places none; the deck
  material's `spec_pass` bakes `rgb: null` on BF1 (`rtyp: Specmap`, power 4).

## What to find
Which term lights BF1's decks: the Specmap second pass (does the port draw it on BF1 at all — `rgb: null`), a BF1
local-light path (the `Light` classes, `SetAttached…`/world lights in BF1's world file), or the DomeInfo filter
(BRGT). One PS2 knock-out per candidate on a `bf1_kam1_walk_*` state settles it.

## Acceptance
- RE: the term named, [V]/[O], with a patched-state frame that removes it on the PS2.
- Built (same card or a build card): the deck box at the four bearings of cp0_cp2 and cp4_cp4 within ±12 levels per
  channel of the PS2's; `parity.py` bf1_kam1 down by ≥ 2.

## Planner note (2026-10-02 04:30)
`alpha: yes` -> `defer: post-alpha`, value 4 kept. PLAN §Milestones: "pure lighting-parity work off the showcase maps"
waits for the alpha, and BF1 Kamino is not one of the six. The maintainer's seven Kamino cards of 09-30 (KRMP, KGRY, KPOS, KSTK,
GRND, KSKY, KSEAL) are all closed; this was never one of them. After KSKY the map scores 19.3 on the parity board (27.0
before, KSKY's report); its worst post cp4_cp4 is 19.5, fifth on the board. Reverse: `loopctl.py set B1DK defer= alpha=yes`.

## Note from card BPRL's RE (2026-10-03, lane 3)
The title's "BF2's matched once BPNT landed its local lights" does not hold. BF2 Kamino's deck is dark like BF1's: at
`bf2_kam1 walk/cp2_cp3_b090` the PS2's deck is 82/84/89 at mid-floor (box (256,278,627,346)) and the port's 45/48/51.
Near the camera (352,360,627,470) the port had 82/86/91 against the PS2's 80/84/93 only because the post's light drew
a perspective-blended pool there, which the PS2 does not show (`analysis/re-vertex-colour-interp.md` sections 4-5);
with the PS2's blend (BPRL's build) that box is 53/57/62 and bf2_kam1's parity rises 16.05 -> 16.78. So the deck
term this card is after is missing in BOTH games, and BF2's PS2 states (`bf2_kam1_walk_*`) can serve the knock-out.
Planner: widen the title to both games; BPRL's build declares `parity=bf2_kam1` against this card.

## Planner 2026-10-03 07:15 - both games, value 4 -> 5, alpha, no longer post-alpha
- The title is widened as BPRL's RE asked: the missing deck term is missing in BOTH games. BPRL's landing (b9c7939)
  took away the perspective-blended pool that had hidden it near the camera on BF2, so BF2 Kamino's deck is now dark
  where the player stands (bf2_kam1's parity rose 0.655 at the landing, declared against this card, and stands at
  16.71 in `loop/parity/summary.md`; `reports/20261003_0555_card_BPRL.md`).
- Why 5 and alpha, against the 04:30 planner's deferral: that note deferred it as BF1-only lighting parity off the six
  showcase maps. It is now a floor 1.8 x too dark on the map the maintainer plays (INBOX 2026-09-30 22:50: the Kamino cards go
  before value-5 cards on other maps), and it got visibly darker tonight.
- Use BF2's states for the knock-out (`bf2_kam1_walk_*`); one PS2 knock-out per candidate, as "What to find" says.
- Acceptance, added for BF2: the mid-floor box (256,278,627,346) of `bf2_kam1 walk/cp2_cp3_b090` within +-12 levels per
  channel of the PS2's 82/84/89 (port 45/48/51 in BPRL's RE; re-measure first); `parity.py` bf2_kam1 down by 0.66
  or more from 16.71 (BPRL's declared rise taken back); BF1's lines stand.
- Reverse: `loopctl.py set B1DK value=4 alpha= defer=post-alpha`.
- 2026-10-05 (card LSUM landed, bf263f8): the light sum is now made in display units and the ambient floor is gone (report `reports/20261005_1505_card_LSUM.md`). LSUM is on main: this card is unblocked. Form 1 was kept (form 2 scored 10.052 against 9.742).
