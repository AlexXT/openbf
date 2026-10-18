---
id: BHUD
title: The BF2 HUD, part 1 — bake it: the five INGAME.LVL hud_ layouts as a named element tree, the four CORE.LVL bitmap fonts, and the sheets/meshes the port lacks (analysis/re-hud.md); BHUD2 draws it, BMAP the minimap
milestone: M3
value: 5
phase: build
status: done
deps:
files: engine/crates/mungefs/src/hud.rs (new), engine/crates/mungefs/src/font.rs (new), engine/crates/mungefs/src/lib.rs, engine/crates/manifest/src/hud.rs (new), engine/crates/manifest/src/lib.rs, engine/crates/bake/src/hud.rs (new), engine/crates/bake/src/map.rs (bake_common calls it), engine/CRATES.md
declares: rebake=common
needs:
branch: bhud
resume:
created: 2026-09-28
alpha: yes
---

## Why (RE done 2026-09-30, `analysis/re-hud.md`)
The whole BF2 HUD is DATA on the disc: five `hud_` config chunks in BF2 INGAME.LVL (`hudtransforms`, `1playerhud`,
`2playerhud`, `3playerhud`, `4playerhud`; the `mungefs::config` DATA/SCOP format; 145 of 156 property hashes named in
`engine/artifacts/lead_tmp/re/bhud/names.txt`, dumper `huddump.py`). Fonts: four `font` chunks in BF2 CORE.LVL. The port
bakes the INGAME sheets and meshes already, but not the layout, the fonts, the five BF2 COMMON.LVL sheets
(`hud_icon_flash`, `border_3_pieces`, `opaque_black`, `btn_directional_pad*`, `stats_kills`), CORE's `white`, or the
side-LVL `hud_*` meshes (`hud_cis_trishot` CIS, `hud_wampa` SNW, `hud_stealth` ALL, `hud_imp_flamethrower` IMP). This card
bakes them; BHUD2 draws them; BMAP does the minimap and world markers.

## Do
- `mungefs::hud`: parse a `hud_` chunk with `config::parse_tree` into a tree whose nodes carry the resolved name (the
  names.txt table as a const map, hex for the 11 unknown), the raw args (f32 / string) and children.
- `mungefs::font`: `font` = NAME, HEAD (u16 count, u8 pages, u8 line height), FTEX (per page NAME + `tex_`), FBOD (count × 24 B:
  u16 code, u8 page, u8 advance, u8 x0, u8 x1, u8 y0, u8 y1, f32 u0 u1 v0 v1) — re-hud §1.2.
- `bake` (common group, BF2): `common_bf2/hud/<name>.ron` × 5, `common_bf2/fonts/<font>.ron` + page KTX2s (alpha kept), the
  COMMON / CORE sheets above into `common_bf2/textures/`, the side `hud_*` meshes into the side bakes (or common). Schema in
  `manifest::hud` (round-trip test in `manifest/src/tests.rs`).

## Acceptance (measurable)
- Bake: `common_bf2/hud/` holds 5 trees with exactly 125 / 2,343 / 4,722 / 9 / 5,271 records (hudtransforms / 1p / 2p /
  3p / 4p) and 1 / 42 / 4 / 1 / 6 top-level elements (re-hud §7); every name resolves but the 11 listed unknowns.
- Fonts: 4 fonts × 226 glyphs, pages 1 / 3 / 3 / 2 (large / medium / small / tiny); a data-gated test decodes medium '!'
  = page 0, advance 5, box 0..4 × 0..16.
- The 5 COMMON sheets + `white` exist in the bake; `cmp_bake` DIFFs only the new files (declared `rebake=common`); no
  golden, sweep or sim move.

## Original acceptance (2026-09-28)
- RE: where BF2's HUD art and layout live on the disc (textures, fonts, the layout constants), per element (health,
  ammo, reticle — RETC, minimap, tickets, CP bar). → done, `analysis/re-hud.md`.
- Build cards: bake the art (BF2 first, BF1 icon fallback), draw it anchored per COMPARISON_RULES (no stretching).
  → this card (bake), BHUD2 (draw), BMAP (minimap + markers).

## Notes
- Created 2026-09-28 by the planner. RETC (the reticle) is one element; do it first or fold it in.

## Planner note (2026-09-30, M-ALPHA)
`alpha: yes` — the real BF2 HUD is on the alpha list. Value 3 → 4. Showcase maps: BF1 NAB1, BF1 TAT1 (the port's `tat1`), BF1 YAV2, BF2 UTA1, BF2 MYG1, BF2 HOT1 (PLAN §Milestones M-ALPHA).

## Planner note (2026-09-30 13:30)
Value 4 → 5: the real BF2 HUD is an M-ALPHA checklist item, the port still draws the egui placeholder, and this bake is
now build-ready (BHUD2 and BMAP follow it).
