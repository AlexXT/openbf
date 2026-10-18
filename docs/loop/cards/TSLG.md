---
id: TSLG
title: A turret stands at the base of a leg instead of on the platform it belongs to (the maintainer's playtest 2026-10-07, BF2 Kamino): find the platform and every turret slot like it, and put each turret where the PS2 puts it (the pad / hardpoint of the stand, not the stand's origin)
milestone: M2
value: 4
phase: capture
status: todo
deps:
files:
declares:
needs:
effort:
branch:
resume:
created: 2026-10-07
alpha: yes
---

## Why
The maintainer, 2026-10-07: "At the end of the round I noticed a platform that normally hosts a turret but there was no turret there. A turret was at the base of this leg thing - I think the turret should be on this leg thing as a turret station."

## Evidence
- The snapshot cannot name the platform: the maintainer stood at (-125.6, 70.2, 151.0) on BF2 KAM1 and looked 6 m ahead at (-128.1, 70.2, 151.5); no turret or platform class lies within 25 m of that spot (nearest: `tur_weap_built_gunturret1013` at (-102.0, 69.3, 142.3), `kam_prop_crate`s, and `kam_bldg_tower2` / `pillar2` / `landplatform1` at about (-84, 69, 146), 42 m). So he saw it from there or elsewhere in that round - ASK HIM or find it (acceptance 1).
- KAM1's manifest has 33 `tur_*` instances (y 51.4 - 79.4; six at 51-53 m on the low decks, the rest at 69-79). A turret stands where its instance's translation says; nothing in the port moves it to a stand's hardpoint. `tur_bldg_chaingun_roof` instances (`help`, `ambush`, `...roof10`) are roof turrets.
- Related: TBLD (build pads become turrets), VTURD (a turret's gun turns, damage shows), TURR (bots man turrets).

## Acceptance
1. The platform is found: for each of KAM1's 33 turret instances a PCSX2 frame beside the port's (same spot, same bearing) and a note per instance: turret present / on its stand / at its stand's base / missing. The list is the finding; the maintainer's spot is one row.
2. For each defect the cause is read (the instance's own parent or hardpoint in the world data, the class's skeleton node, the pad rule in `vehicle_pads::add_turrets`) and fixed so the turret stands where the PS2's does; the same census run on the other BF2 maps with turrets.
3. A test: every turret instance's gun is within 0.5 m of the stand model's turret node where the stand has one.

## Notes
Filed 2026-10-07 by Claude in chat from the maintainer's playtest (BF2 Kamino, F9 at tick 10701, build 6f6160b; snapshot `playtest/prev1.json`). His words are quoted; nothing here is measured unless it says so. A question for the maintainer in STATUS: which post / which side of the map, or press F9 standing next to the platform (the snapshot's `focus.point` is the look point).
- 2026-10-07 11:30 (the maintainer, second F9, via Claude): "it's platforms like these. There should be turrets on all these platforms, and grenades if thrown correctly fall through the floor." Screenshot `playtest/pete_20261007/5_kam1_platform_no_turret.png`, snapshot `playtest/pete_20261007/f9_kam1_t3481.json` (BF2 KAM1, tick 3481, build 6f6160b): he stands at (103.1, 52.4, 75.0) on a large low deck (y 52.4), rail ring with lit strips, a floor grid, two raised blocks on the near rail, rain. This is a DIFFERENT place from the first F9 (-125.6, 70.2, 151.0, a high deck at y 70), so it is a kind of platform, not one spot.
- From the port's world data (read by Claude): within 47 m of this spot there is NO turret or pad instance (nearest `com_weap_gunturret104` at (57.0, 51.4, 85.1), 47 m) and no building instance within 28 m (pillars, 'gutter' sound emitters). The deck's own mesh is not one of the manifest's instances. KAM1 has 33 `tur_*` instances (21 `tur_weap_built_gunturret` build pads, 12 `tur_bldg_chaingun_roof`) - the RE note counts 16 pads on KAM1 (`re-build-zones.md`, RespawnTime 120 s).
- So two readings, to be told apart from the PS2: (a) the PS2 has a pad or turret on these decks and the port's world lacks the instance (a bake that drops it, or an object the manifest does not list); (b) the PS2's turrets here are pads the PS2's bots BUILD within seconds (card TBLD) and the port neither builds nor draws them. The census (acceptance 1) decides: PCSX2 frame of this deck, from this spot, beside the port's. There is no PS2 reference frame within 80 m of it (`pcsx2/ref/bf2_kam1`: the nearest eye is 83 m).

- 2026-10-09 12:45 planner. GRFL (296fa0c) landed the grenade fix and measured it at the maintainer's SECOND deck only (103.1, 52.4, 75.0: 0 of 4 through). His first deck (the F9 at tick 10701, `playtest/prev1.json`, y about 70 - this card's platform) was not measured, and 935 of Kamino's 937 remaining pass-throughs are throws that cross `kam_bldg_mushplatform1`'s soldier-only skirt. When this card stands at the first deck: run `ordnance_census`'s four grenades at his position from the snapshot and write the count here and on GRFG.
