---
id: WHELD
title: The held weapon model follows the selected weapon: a soldier (the player and the bots) draws the weapon in its hands, not always its class's primary
milestone: M3
value: 5
phase: build
status: todo
deps:
files: engine/crates/game/src/soldier_vis.rs
declares:
needs:
effort:
branch:
resume: BUILD: (1) inventory per class slot which weapon geometries the bake already has (`scripts/rebake.sh` side group bakes each class's primary and a few pistols/launchers) and which it does not; (2) read the sim's selected weapon for a unit (core's weapon state, read-only) and draw THAT weapon's model in `soldier_vis::weapon_draws`; (3) bots too.
created: 2026-10-19
first: 3
---

## Why
(the maintainer, 2026-10-19) Switching weapons in the game changes how the soldier fires and behaves, but the soldier still holds the model of its class's primary: `game/src/soldier_vis.rs` `weapon_draws` (line ~1084) draws `weapon_for(team, class)`, the class's primary, for every soldier; only the animator follows the weapon in hand. The held model must follow the selected weapon, for the player and for the bots.

## Evidence
`soldier_vis.rs` header comments (primary-weapon GeometryNames in first-use order, `weapon_for`); the fire point is read off the drawn weapon's FirePointName node (`soldier_vis` fire-point function), so it must follow the same model; card HUDW baked BF2's pistols for the HUD row; `analysis/re-grenades.md` §2.4 (the weapon channels and SWITCH_WEAPON); the PS2's own behaviour for what a grenade or a secondary looks like in the hand needs a PCSX2 look (re-soldier-anim-layers.md for the hold poses).

## Acceptance
1. A table in the card or `analysis/`: for every soldier class of both games, each weapon slot, its GeometryName, and whether that model is baked today.
2. Where a model is missing, `scripts/rebake.sh`'s side group bakes it (declare `rebake=side`); no new weapon is drawn that the PS2 does not draw (a slot with no hand model on the PS2 shows none).
3. In a battle the player and a bot holding the second weapon show its model and fire from ITS fire point; a test pins the model choice by slot; the unit-gallery or a golden shows two weapons of one class.
4. Goldens that move are predicted from data (units with a non-primary selected in a golden's view: probably none), declared, viewed. The sim hash is untouched.

## Notes
Reads sim state, writes none. If the animator's hold pose per weapon is wrong for a model, that is ANIMX's, not this card's.
