---
id: HCORN
title: A BF1 speeder bike that starts a leg within its 20 m node radius of the leg's first detour corner skips that corner at once and drives into the barrier it went round (card HJAM's corner reach; AUDIT HJAM 2026-10-16): BF1 Tatooine leg 54 -> 55, barrier267's NE corner (-148, 44) - 63 of 72 such starts die within 2 s against the rock at (-150, 37.8), 1 of 56 before HJAM; BF1 Endor legs 4 -> 14 and 34 -> 38 the same. RE what the PS2 does at such a corner first, then fix and test the driver's radius input
milestone: M3
value: 4
phase: re
status: todo
deps: HJAM
files: engine/crates/core/src/ai/route.rs, engine/crates/core/src/ai/hover_drive.rs
declares: sim; sweep
needs:
effort:
branch:
resume:
created: 2026-10-16
alpha: yes
---
## Why
AUDIT HJAM (2026-10-16, `engine/artifacts/lead_tmp/audit-hjam/EVIDENCE.md`). Card HJAM gave a hover's driver the PS2's
node radius at detour corners (ForwardSpeed x 0.5: 20 m for BF1's bike; both ELFs' `FUN_002a6318` / `FUN_00234db0` hover
branch, re-checked). It removed the corner circling (near-corner driving 14.8 % -> 1.4 % of hover driving time). But a
leg that STARTS with its first corner already inside 20 m (the hub before it is also reached at 20 m) skips the corner on
its first tick and the bike aims straight across the barrier the corner walks round. Traced, BF1 Tatooine (tat1) seed 2,
unit 0 on bike 12: hub 56 reached 18.9 m out at 37 m/s heading west, 11 m south of its leg; the next leg 54 -> 55 has one
corner (-148, 44), barrier267's NE corner (barrier at (-155.3, 39.1), half 4.2 x 4.8 m), 6.5 m away -> skipped at once;
the rider dies 5 ticks later at (-150.0, 37.8), the barrier's east face (`bb/tr_x0.err`, `bb/tat1/tat1_s2.json`).

tat1 is a showcase map (PLAN.md 130): about 2.5 bikes a round explode at that one rock in play.

## Evidence (all under engine/artifacts/lead_tmp/audit-hjam/)
- `ent/` (tat1 seeds 1-24, the landing's switch binary: HJAMX=3 = the parent byte for byte): leg 54 -> 55 starts carrying
  the corner, died within 2 s: parent 1/56, HJAM 63/72; the same leg without the corner 0/65 and 0/63. Starts and start
  speed equal (121 vs 135, median 35 m/s) - the rule makes the leg deadly, it does not send more bikes into it.
- `dth/`: 57 of tat1's 201 rider-with-vehicle deaths at (-148.0, 14.8, 37.5) over 24 seeds (2 there before); tat1 bike
  crashes a ride 0.028 -> 0.091 (rides 963 -> 981).
- `ent2/legs.py` (14 hover maps x 12 seeds): BF1 Endor leg 4 -> 14 corner starts 2/100 -> 26/88, leg 34 -> 38 5/98 -> 28/143;
  BF1 Yavin 17 -> 20 was already deadly before (49/135 -> 72/161). BF1 Endor's crash a ride 0.23 -> 0.30 overall
  (the PS2 0.28 = 15 of 54 rides, re-hover-ai 11), so the AVERAGE is the PS2's; the question is WHERE they crash.
- Mutations M5 / M6 (`mut/`): `hover_drive::drive` handing `advance_route` 0.0 or `arrive * 0.5` passes all 1,587 tests
  - the corner rule's (and VAI's hub reach's) input in its only caller is untested.

## RE first (the faithful answer is not known)
By the RE the PS2's follower also takes a non-last node within R (`FUN_002f1140` d^2 < R^2), so the PS2 may skip this
corner too. Open: (a) the corner's offset for a HOVER on the PS2 (the A* over barrier corners at r + 0.5 m - which r for a
hover?) against the port's (-148, 44) 2.4 m off the barrier's corner; (b) barrier267 `flags: 31` against the PS2 hover's
barrier mask (does a hover plan round it at all?); (c) whether the PS2's obstacle probe (1.5 x speed ray, re-hover-ai 2)
turns the bike off this rock - the port's probe did not; (d) the PS2's own BF1 Endor crash SPOTS (re-hover-ai 11's capture,
15 crashes in 5 min) against the port's legs 4 -> 14 and 34 -> 38.

## Acceptance
- tat1 leg 54 -> 55 corner starts dying within 2 s <= 5 % over 24 seeds (`ent2/legs.py` on the new binary) - or, if the
  RE shows the PS2 crashes there too, a PS2 capture of it and this card closed as faithful.
- BF1 Endor near-corner driving share stays <= 1 % (HJC census) and crash a ride inside the PS2's 0.28 +- 0.06.
- A test that drives `hover_drive::drive` past a corner inside the node radius and fails under M5 / M6 (the radius the
  driver hands in zeroed / halved).