---
id: HAIS
title: Hover bots steer by the PS2's law: the shared FUN_002a54d8 with a strafe speed, FUN_002a45a0's squared turn curve, and FUN_00220528's own goal handling (the port's steer_to is OURS)
milestone: M2
value: 4
phase: re
status: done
deps: WAI
files:
declares: sim; sweep
needs:
effort:
branch:
resume: closed 2026-10-03: analysis/re-hover-ai.md (docs b073094), numbers on VDROP
created: 2026-10-01
alpha: yes
---
## Why
The port's hover driver (ai::ride::steer_to) is OURS / PROVISIONAL: a proportional turn and a throttle cone. The PS2's
hover AI FUN_00220528 calls the same steering FUN_002a54d8 the walker uses, with K+0x670 (ForwardSpeed), K+0x678
(a strafe speed), K+0x680 / K+0x688 (B2:135769), and its turn stick comes from FUN_002a45a0's hover path:
u = clamp(-l.x x (4.5 - clamp(rate, 1, 3)), +-1), turn = u|u|. Card WAI transcribed and verified those two functions;
what is left is FUN_00220528's own part (it edits the goal and halves the throttle on some condition, B2:135614-135760).

## Evidence
analysis/re-walker-ai.md sections 3-4 ([O]). lead_tmp/wai/wai_sample.py + wai_check.py replay a transcription against
live PS2 RAM tick by tick (walkers: 7504 of 7504 ticks); the same sampler takes a hover with its vtable pair.

## Acceptance
- RE (max effort, the discipline): FUN_00220528 transcribed; a live capture of AI hovers on two maps (BF2 + BF1 twin
  opened) replayed with >= 99 % of the driving ticks equal on throttle, strafe and turn.
- The build is card VDROP's (planner 2026-10-02: it depends on this card): the hover driver writes those sticks; hover
  boards / metres driven per map in round_to_outcome do not fall, 8 seeds no finish count drops. Close this card by
  writing the doc, the replay numbers and the PS2's speeder-bike loss count onto VDROP.

## Planner note (2026-10-02 04:30)
Value 3 -> 4, `alpha: yes`: VDROP (4, alpha) now depends on this card. VDROP's step 2 ("build the PS2's hover
stick") and this card's "then build" were the same work written twice; this card is the RE alone (max effort, the
discipline, the live replay) and the build is VDROP's. While the AI hovers are being captured, also count on Endor
(BF2 END1; BF1 END1 if the session allows) how many speeder bikes are lost with a rider aboard per minute of battle:
that is the PS2 side of VDROP's acceptance 1. Reverse: `loopctl.py set HAIS value=3 alpha=` and `set VDROP deps=WAI`.

- Planner 2026-10-02 09:30: `files:` emptied while this card is `phase: re` (it listed `core/src/ai/ride.rs`, which
  would have made WAI2 and VBAIL ineligible while the RE runs; an RE writes `analysis/` only). VDROP is the build and
  lists the file.

## Closed 2026-10-03 (lane 2, report `reports/20261003_0245_card_HAIS.md`)
`analysis/re-hover-ai.md` (docs commit b073094; verifier 49 SUPPORTED / 0 / 0). Acceptance as run:
- FUN_00220528 transcribed (probe, MovingTurnOnly floor, unstick manoeuvre, x0.6, bail-out) and its BF1 twin
  FUN_001eedc0 opened AND replayed.
- Replay, driving ticks equal on throttle, strafe and turn: BF2 Endor 22617 / 22621 (99.98 %), BF2 Kashyyyk 15815 /
  15832 (99.89 %), BF2 Geonosis 6491 / 6492 (99.98 %), BF1 Endor 24089 / 24222 (99.45 %).
- PS2 speeder bikes lost with a rider, 5.00 min of battle: BF2 END1 4 (0 crashes), BF1 END1 16 (15 crashes).
- The build is VDROP's; its acceptance is re-based on these numbers.
