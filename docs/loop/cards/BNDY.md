---
id: BNDY
title: The map boundary: leaving the battlefield warns the player (HUD text + his team's 'leaving' line) and kills him after BF2's 6.5 s (BF1 10 s) - the port has no boundary at all; read the boundary data and the point test, then build it
milestone: M3
value: 4
phase: re
status: todo
deps:
files:
declares:
needs:
effort:
branch:
resume: RE half done, stopped by STOP 18:19 (2026-10-16): read engine/artifacts/lead_tmp/bndy/DRAFT_re-boundary.md first (claims C1-C8, D1-D2, T1-T5 with citations; RAM check of all 32 maps already AGREE, cmpshapes.txt). Next: (1) the five re_check receipts (absence: no other writer of the boundary list head, no AI reader of the point test; B2 FUN_0034c4f0 a1 pass-through by mdis.py), (2) the RE workflow verifier on the claim list, (3) the live point test: tools/bnd_live.py <points/MAP.json> <ref/states/MAP_walk_*_b000.p2s> --death on NAB1, TAT1, YAV2, BF2 HOT1, UTA1, MYG1 (+ BF1 KAM1 for the non-convex points), (4) draw the shapes over the port's top-down sheets, (5) lift into analysis/re-boundary.md, then set phase=build with the acceptance rewritten (BF2 pair on HOT1: UTA1/MYG1/YAV2 have no boundary).
created: 2026-09-30
alpha: yes
---

## Why
Filed by ANNC's RE (2026-09-30). On the PS2 a unit that leaves the map's boundary gets a HUD warning ("leaving the
battlefield"), hears his team's `<side>leaving` line and dies when a countdown runs out. The port has no boundary:
`core` has death regions only (`sim/state.rs` `death_regions`), so a player can walk off every map. This is the missing
half of ANNC's trigger T6.

## Evidence
`analysis/re-announcer.md` §2.4 ([V], verifier SUPPORTED on D1-D4):
- BF1 `FUN_00297e60(dt, member)` (B1 222656-222701): outside every boundary shape -> timer 10.0 s; the line once, for a
  local human; at 0 the unit is killed; back inside -> timer -1 (the next excursion starts again).
- BF2 `FUN_00350660` (B2 346687-346735): 6.5 s, or 2.1667 s when the unit's vehicle class has bit 1 of class+0x1208
  [H: a flyer]; the line is played by the HUD element `FUN_00391fe0` (389580-389610) when it first shows, with an
  alpha pulse; text key `game.tooltips.leaving_battlefield`. A soldier flag (bit 7 of soldier+0x660) forces the state
  [H: untraced].
- The boundary object: B1 `DAT_00518314`, B2 `DAT_005feca8`; a linked list of shapes built from named paths (B1
  `FUN_00295678` / `FUN_002957b8`, B2 `FUN_0034c4f0` / `FUN_0034c638`); "outside" = inside none of them
  (B1 `FUN_002956d8`, B2 `FUN_0034c550`); no shapes at all = never outside.
- Lines per mission: `lead_tmp/annc/bf1_calls.txt` / `bf2_calls.txt` (`SetOutOfBoundsVoiceOver`); five BF1 scripts set none.

## What to find
- Where the boundary comes from on the disc (the world's boundary / path chunk, the Lua call that names it), the shape
  test (segments of a path, which side is inside, the height rule if any), and which maps have one.
- Who is tested: every unit or humans only; AI behaviour at the boundary (do bots turn back?).
- The BF2 2.17 s case (which vehicles) and the soldier flag that forces the warning.

## Acceptance
- RE doc with [V]/[O] claims and the verifier run; per map, the boundary polygon drawn over the port's top-down sheet.
- A PS2 check of the point test: for the six showcase maps, >= 20 sample points per map classified inside / outside by
  the PS2 (the live member timer after a RAM hold at each point, `walk_bf1.py` / `walk_bf2.py`'s hold) and by the
  transcription — 0 disagreements.
- Built: a unit test of the timer rule (outside at t0 -> warning + line at t0, death at t0 + 6.5 s, back inside
  resets), and a PS2 pair on BF2 UTA1: warning onset and death time within ±0.2 s of the PS2's.

## Notes
- PLAN plays BF2's rules on every map: default 6.5 s everywhere, BF1's 10 s recorded as data.
- ANNC's build leaves T6 to this card; the line names are already in ANNC's mission RON fields.

- Planner 2026-09-30 21:45: `alpha: yes` — every showcase map has a boundary, and PLAN's alpha checklist says "no soldier or AI out of bounds".

- Fable 2026-10-01: NOT this card's boundary (the 2-D leave-the-battlefield timer is still unbuilt), but related work landed: KFLR 5b12384 (a kill floor 15 m under the lowest playable floor; death regions and the floor kill past the invincibility cheat, as the PS2's do - `analysis/re-kill-floor.md` also confirms this card's boundary test is 2-D, x and z only) and BNDS 9440823 (bots stay inside the disc's BARR barriers at the level's edges and walk back when off it).
- Audit ANNC 2026-10-02 — the leaving lines carry locks the commander's other lines do not: BF2 `allleaving` has `PlayInterval 10` and every `repleaving` take is `(id, 1, 1, 0)` (a 1 s re-draw interval); `impleaving` / `cisleaving` and all four BF1 lines have none (`python3 engine/artifacts/lead_tmp/audit-annc/sndprop.py <SOUND LVL> leaving`). A request inside the interval plays nothing on the PS2, so the build needs the take / property lock for these two (memory `sound-take-locks-and-segs`).

## Note (card FXC, 2026-10-16): where the boundary comes from - the `bnd_` chunk
`analysis/re-chunk-census.md` 5 (verifier C8 recounted): every map LVL that has one carries a `bnd_` config chunk
`Boundary { Path(<hash>) ... }` (DATA 0x54eacca7 = FNV("boundary"), one 0x84874d36 = FNV("path") statement per path whose
argument is the path NAME's FNV); B2's level loader FUN_00177058 hands it to FUN_0034c3b0(DAT_005feca8, chunk) (B2 18380).
The paths are ordinary `path` chunks the port already reads (`mungefs::path`). Counts: BF1 one path on 18 maps
(`boundary` on BES1, NAB2(_H), RHN1/2, TAT1(H); `boundary1` on the other 11), **no `bnd_` on YAV1 / YAV2**; BF2 FEL1 4,
HOT1 3 (`boundary1..3`), SPA2 3, KAS2 / SPA7 / SPA9 2, eleven maps 1, and **an empty scope on COR1, DAG1, DEA1, MUS1, MYG1,
NAB2, TAN1, UTA1** - with no shapes the PS2 is never outside (2.4), so two BF2 showcase maps (UTA1, MYG1) and BF1 YAV2 have
no boundary unless the boundary object takes paths from elsewhere too: check FUN_0034c3b0 / B1 FUN_002954a8 for a second
source before building. Decoder: `engine/artifacts/lead_tmp/fxc/tools/bnd.py`.

## Note (lane 2, 2026-10-16 18:30, RE half done - DRAFT, not verified)
- Three of the six showcase maps have NO boundary on the PS2: BF1 YAV2 (no `bnd_`), BF2 UTA1 and MYG1 (empty scope) -
  the boundary object takes shapes only from `bnd_` (one caller of the push), and the list head is 0 in every
  YAV2 / UTA1 / MYG1 save state. The acceptance's "PS2 pair on BF2 UTA1" cannot be run: use BF2 HOT1 (3 shapes).
- The test is a CONVEX half-plane test per path (B1 FUN_002958f8 / B2 FUN_0034c780), union over paths, x and z only;
  BF1 BES1 / HOT1 / KAM1 paths are not convex, so the PS2 admits less than the drawn polygon there.
- The BF2 2.17 s is a flyer class with key 0xc6e9c80b = 1: only three guided-rocket ordnance classes on the disc,
  not starfighters. The "soldier flag" bit 7 of +0x660 is the FLYER's, with no writer found.
- RAM check done: all 32 baked maps' boundary edges in a save state = the transcription (271 edges, max 6.2e-5).
- Draft and tools: `engine/artifacts/lead_tmp/bndy/` (DRAFT_re-boundary.md, tools/, cmpshapes.txt, points/).
