---
id: KSEAL
title: BF2 Kamino's ocean is lit by the sun on the PS2 (FFT normals in the packet): (8,19,19) at 314-393 m where the port's unlit unity draw reads (26-49, 38-57, 34-55) with white foam
milestone: M1
value: 5
phase: build
status: done
deps: KSEA
files: engine/crates/game/src/viewer/look.rs,engine/crates/render/src/shader.wgsl,engine/crates/render/src/lighting.rs,engine/crates/render/src/frame.rs,engine/crates/render/src/lib.rs
declares: goldens=maps/bf2_kam1.png,walkabout_stats.ron; parity=bf2_kam1
needs:
effort:
branch: kseal
resume: landed 0fb84669 2026-10-01
created: 2026-09-30
alpha: yes
---

## Why
BF2 Kamino's sea is dark and blue-shifted on the PS2 and bright with white foam in the port: the port draws the ocean
unlit at unity (`viewer::look::water_look`, PROVISIONAL [H] for BF2), the PS2 lights it. `analysis/re-kamino.md` §2.3.

## Evidence
- BF2's ocean vertex packer `FUN_00574a70` writes three NORMAL bytes per vertex and submits a normal stream
  (708056–708058, 708105) [V]; the colour word is half-white with the foam byte as alpha (708067).
- PS2, lightning frozen (`lead_tmp/kamr/ps2exp/out/s2_ctl0..2.png`): (8–9, 19, 19) at 314–393 m, (6,15,15) at 428 m,
  (5,13,13) at 521 m, where the port's bare base texture reads (21,34,29) — ratios (0.38–0.48, 0.56–0.59, 0.66–0.69)
  = the sun (65,85,105)/128 × ~0.8. With the lightning live the sea follows the lights ((2,9,7) at a flash's start).
- Port at the same pose, ocean stepped: (26–49, 38–57, 34–55) (`views/sea_compare.png`).
- BF1's ocean stays unity (PS2 (17,30,33) vs port (20,33,29), KOC) — do not touch it.

## Do
1. Read the far/near ocean draw's light path if cheap (which program, ambient or not — `FUN_00575670`'s three programs);
   otherwise build PROVISIONAL: base and foam × the sun's diffuse term from the FFT normals the port already computes.
2. Fit to the control row; keep the rule in one place (`water_look`), labelled PROVISIONAL with the fit.

## Acceptance
- At the `s2_ctl0.json` pose, rows by distance (x 0..110): the port within ±5 levels per channel of the PS2 control row
  at 314, 363, 393, 428 and 470 m (median of 3 water phases).
- No pure-white foam: p99 luma of the sea box ≤ the PS2's + 15.
- BF1 KAM1's sea box unchanged (byte-identical render with the flag off for BF1).
- `parity.py` bf2_kam1 before / after (after KSEA, same instrument).

- Note (KSEA build, 2026-09-30): KSEA's horizon line moves here. After KSEA the sea ends on the PS2's tile row with
  black behind, but on BF2 the edge shows: at the `s2_ctl0` pose (x 0..110, 4-row bands) the port's last sea bands read
  (25–27, 29–36, 27–33) against black, edge contrast 32.5 levels; the PS2's read (3–4, 5–8, 5–8), contrast 3–6
  (`lead_tmp/ksea/rows.py --edge`). **Add to the acceptance: edge contrast ≤ 12 at that pose.** BF1 needs nothing: its
  PS2 frame has the same visible edge (cp0_cp2_beta1_v3_water, x 400..440: PS2 black to row 244 then 15/26/28/33 G; port
  black to row 240 then 21/53/26/33).
  Lead for the fit: the PS2's sea darkens with distance by more than its fog — G 19 at 383 m, 14 at 436–479, 11.6 at
  532, 8.4 at 598, 5.4 at 637, where the black fog alone (the amounts the green poke measures, now matched by the port:
  +25/41/35/29/17 at 682/638/598/563/532 m vs the PS2's +30/43/39/31/23) would leave ~16 at 598. The far branch of
  `FUN_00575670` draws its tiles ONCE, with "fogspread" bound and two colour words: the fog colour × `DAT_007c5754` and
  `DAT_0076c258..264` × `DAT_007c5754` (708447–708467); RAM in `kamr/ps2exp/out/s2_ctl0.p2s`: `DAT_007c5754` = 0.5,
  `DAT_0076c258..` = (1,1,1,1) — read, not traced further [H].

- Landed (2026-10-01, report `reports/20261001_2300_card_KSEAL.md`): three PROVISIONAL constants in `water_look` — sun
  weight 0.75, foam blend 0.5, colour kept at the far pass's end 0.5 (from FogRange.near). Acceptance run: the five
  rows within 3.5 / 1.9 / 1.2 / 2.1 / 2.1 levels (worst channel, median of 3 phases); p99 luma 27.7 (PS2 60.1, rain);
  last tile row (3.7,5.3,6.0) against black, PS2 (5.4,5.8,6.1); BF1 KAM1 frames byte-identical; parity bf2_kam1
  16.292 → 16.053. Leads left in `analysis/re-kamino.md` §2.6: the lighting VU program, the faint foam and the far
  loss are unread; the port's sea is 2–3 levels bluer; the port's wave range is ~20 % over the PS2's RAM (20 states).
