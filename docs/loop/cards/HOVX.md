---
id: HOVX
title: Hover, BF2's boost + energy (re-hover §5.1 I5, §6.3, §6.7) and the speeder-bike jump (§6.9) — the hull bodies split to HHUL, the water float to HWAT
milestone: M2
value: 4
phase: build
status: done
deps: HOVR,HCON
files: core/src/vehicle/hover.rs,core/src/vehicle/hover/tests.rs,core/src/sim/seats.rs,game/src/vehicle_class.rs,game/tests/vehicle_class.rs,game/examples/hover_drive.rs,CRATES.md
declares: sweep
needs:
branch: hovx
resume: landed 24016cab 2026-09-28
created: 2026-09-28
---
## Scope (2026-09-28, the HOVX build session)
Split: this card builds BF2's boost + energy (§5.1 I5, §6.3, §6.7) and the speeder-bike jump (§6.9). The hull bodies need RE first (which bodies meet terrain — with no TerrainCollision line every body is in the subset, the coll mesh included) → card HHUL; the water float needs the WMAP wet grid baked into the manifest → card HWAT.

## Why
HOVR landed only the spring points meeting the world (exact for the TAT1 skiff, which lists no hull prim). Tanks, bikes and the command hovers need their hull spheres, the water float, boost and the bike jump before VSEAT's driving feels right on every map.
## Evidence
analysis/re-hover.md [O] §4 S3, §6.3, §6.9, §7 K6-K9; HOVR report engine/artifacts/loop/reports/20260928_0923_land_HOVR.md.
## Acceptance
Unit tests pin each new term against re-hover's numbers; hover_drive on a BF2 bike (jump height, boost speed) and a tank (hull sphere stops it at a wall); sweep outcomes unchanged (only hashes).

**Note (HOVR audit, 2026-09-28):** the BoostSpeed turn scale (§6.10, B2:134384-134397) is live whenever |v| > ForwardSpeed, boost or not; BoostSpeed is 17-40 on all 138 RAM hovers. It moved to HCON with §6.19. For the hull (K7-K9): FUN_00169668 (B2:7937-8054) pushes out fully only for other-class 0x10/0x80 (a mass split for 4/8, no push otherwise); check the terrain/building classes before pushing hulls out of them. The water float K+0xe76 is on for 115 of 138 RAM hovers (off only on the snail tank).
