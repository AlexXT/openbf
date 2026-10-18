---
id: FXMF
title: Muzzle flash: draw the PS2's flash MODEL (muzzleflash / com_sfx_muzzleflash, additive, tinted FlashColor, scaled FlashRadius, fading over FlashLength) at every soldier-held gun's fire point (hp_weapons × the weapon's hp_fire) along the aim — analysis/re-muzzle-flash.md §8; replaces the PROVISIONAL quad
milestone: M1
value: 5
phase: build
status: done
deps:
files: engine/crates/core/src/flash.rs (new), engine/crates/core/src/lib.rs, engine/crates/game/src/roster.rs, engine/crates/game/src/fx.rs, engine/crates/game/src/soldier_vis.rs, engine/crates/game/src/bolt_vis.rs, engine/crates/bake/src/soldier.rs, engine/crates/bake/src/alpha.rs, engine/crates/manifest/src/lib.rs, engine/crates/render/src/particles.rs, engine/crates/render/src/particles.wgsl, engine/CRATES.md, engine/artifacts/loop/FEEL_NOTES.md
declares: rebake=common,side
needs:
branch: fxmf
resume: LANDED bd05f80 (report reports/20260930_1032_card_FXMF.md). Nothing left; ADDM/FXP4 carry the reuse notes.
created: 2026-09-30
alpha: yes
---

## Build (RE done 2026-09-30, lane 2: `analysis/re-muzzle-flash.md`, b5353b8; report `reports/20260930_1000_card_FXMF.md`)
- **What the PS2 does** (§0): each round restarts end = now + FlashLength; while t = (end − now)/FlashLength > 0 the
  weapon's render draws the MODEL `muzzleflash` (BF1) / `com_sfx_muzzleflash` (BF2) at P = weapon world × fire point
  (the `FirePointName` node, default `hp_fire`, of the `GeometryName` skeleton; (0,0,0) = the `hp_weapons` origin when
  there is none), rows X = (d×ŷ)R, Y = (d×(d×ŷ))R, Z = dR (d = the aim, cross products unnormalised), colour FlashColor
  with alpha (int)min(A, 2A·t), ADDITIVE, depth tested, no depth write, unlit, unfogged. FlashWidth / FlashLight* are
  never read: no light. Soldier-held guns only — no vehicle/turret/building gun, no sabre.
- **Port gaps** (§8): no fire points in the baked weapons; `meshmuzzleflash` / `com_sfx_meshmuzzleflash` baked with alpha
  255 (ALPHA RULE v2 rule 2) though the disc's alpha IS the flame; no additive model draw (FXP4 / ADDM want the same one).
- **Timing on the sim tick**, not the interpolated render time: FlashLength 0.025 s (every blaster) = the one tick of the
  shot at full alpha, as on the PS2. Add the FEEL_NOTES line.

## Acceptance (build)
- Data tests: the replayed looks reproduce the census (`lead_tmp/fxmf/census_bf1.txt` / `_bf2.txt`: 68 / 182 classes with
  R > 0; e.g. BF1 `rep_weap_inf_rifle` R 0.6, L 0.025, (80, 80, 255, 255); BF2 `com_weap_inf_rifle` (255, 100, 100, 255));
  the baked fire points equal the PS2 RAM's (`rep_weap_inf_rifle` (0, 0.0533, 1.097), `cis_weap_inf_pistol` (0.0103,
  0.0394, 0.2601), `all_weap_inf_rifle` (0, 0.0597, 0.6926)); the baked flash texture's mean alpha is the disc's 38/255.
- Unit tests of the §4.2 matrix (d = (0,0,1) → rows (−R,0,0), (0,−R,0), (0,0,R)) and the §4.3 fade (t 1 → A, 0.5 → A,
  0.25 → A/2).
- A side-view sheet of the port's flash (R 2.5 and natural R 0.6) viewed next to `lead_tmp/fxmf/ps2cap/sheet_bf1_L.png`;
  every gate PASS but the declared rebake.

## Why (original)
The muzzle flash is seen on every shot. It is NOT an `fx__` particle effect: `analysis/re-fx.md` §7 [V] — B1
`FUN_002864f0` (210374–210405) hashes the weapon's `MuzzleFlash` value and, for `small_muzzle_flash` /
`med_muzzle_flash` / `large_muzzle_flash` only, writes hardcoded fields (+0x1a4 = 0.6 / 1.0 / 1.8, +0x1a8 = 0.15 /
0.25 / 0.45, +0x194 = 0.1 / 0.15 / 0.3, +0x1ac = 0xff32ffff, +0x1b0 = 0xff0d3f3f / 0xff197f7f / 0xff25bfbf); any other
value keeps the weapon ctor's defaults (BF2 twin `FUN_00328fb0` 320842–320880). The port draws a PROVISIONAL quad.

## Acceptance (original, RE — done)
- RE (§5 discipline, both games): which reader draws the flash from those fields (sprite size/length, colour, the
  texture, the light radius/colour/duration, where on the weapon — the fire point / barrel hardpoint), the weapon
  ctor defaults, and the ODF `FlashColor` / `FlashLength` / `FlashLight*` keys if they feed the same fields.
- `analysis/re-muzzle-flash.md` (or a §13 of re-fx.md), then phase=build with files/declares.
- A PCSX2 frame of a flash (first-person-ish or side view; the §9 capture hid it behind the player's own body).
- Planner 2026-09-30 08:30: value 4 -> 5. With FXB and FXP2 landed and FXP3 next, the muzzle flash is the last effect on the maintainer's M-ALPHA list (muzzle flash, bolts, impacts, explosions) with no build in sight.
