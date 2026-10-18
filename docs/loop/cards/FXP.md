---
id: FXP
title: The real particle effects: muzzle flash, bolts and tracers, impacts on ground/walls/water, explosions — game::fx draws PROVISIONAL coloured quads (its 'HOLD FOR FABLE 4' is stale); RE the PS2's fx__ format and emitters, then build
milestone: M1
value: 5
phase: re
status: done
deps:
files:
declares:
needs:
branch:
resume: RE done (analysis/re-fx.md, 515b024); split into FXP1 (parse+bake) -> FXP2 (runtime+draw) -> FXP3 (spawn hooks), plus FXMF (muzzle flash RE) and FXB (laser bolt RE)
created: 2026-09-28
alpha: yes
---

## Why
The maintainer (INBOX 2026-09-28 (d)): the real particle effects — muzzle flash, bolts and tracers, impacts on ground, walls and
water, explosions — are seen in every second of every battle. `engine/crates/game/src/fx.rs` draws PROVISIONAL coloured
quads and its header says "HOLD FOR FABLE 4" (planner read line 6): that hold is stale; there is no Fable on openbf now,
so it is the loop's own §5 RE.

## Acceptance
- RE ([O]): the `fx__` chunk format (emitters, particles, textures, blend) and which effect each weapon/ordnance/impact
  names (ODF keys), for both games; `analysis/re-fx.md`.
- mungefs parses it, the bake carries it, a build card draws it; muzzle/impact/explosion PCSX2 pairs viewed.
- Remove the stale HOLD note from fx.rs's header.

## Notes
- Created 2026-09-28 by the planner. FXC (census of the other unparsed chunks: `shdw`, `CLTH`, `bnd_`, `font`) keeps the
  rest; its `fx__` part moves here. VDTH's explosions should consume this.

## Planner note (2026-09-30, M-ALPHA)
`alpha: yes` — real effects are the alpha's first visible item; the 'hold for Fable' is void (no Fable on openbf). Value 4 → 5. Showcase maps: BF1 NAB1, BF1 TAT1 (the port's `tat1`), BF1 YAV2, BF2 UTA1, BF2 MYG1, BF2 HOT1 (PLAN §Milestones M-ALPHA).

## Outcome (2026-09-30, RE session, lane 2)
RE done: `analysis/re-fx.md` ([O], commit 515b024; one Opus verifier 34 SUPPORTED / 6 PARTLY folded in / 0 NOT
SUPPORTED). Split: **FXP1** (mungefs parse + bake, value 5) → **FXP2** (runtime + draw; removes fx.rs's stale HOLD
note) → **FXP3** (spawn hooks + the PCSX2 pairs); **FXMF** (muzzle flash: three hardcoded presets, not an `fx__`
effect — RE) and **FXB** (the laser bolt's own draw — RE). PS2 reference frames of real impacts:
`engine/artifacts/lead_tmp/fxp/ps2cap/`. Report `engine/artifacts/loop/reports/20260930_0222_card_FXP.md`.
