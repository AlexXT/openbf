---
id: HTGT
title: The target's name and health on the HUD: what you shoot at (soldier, droid, vehicle, turret) shows its health at the top, the bar dropping as you hit it with bullets, grenades or rockets, coloured by team (friend blue, enemy red, neutral white) - the PS2 BF2 HUD's Target element, every map of both games
milestone: M3
value: 5
phase: re
status: todo
deps: BHUD2
files:
declares:
needs:
effort:
branch:
resume: RE FIRST (PS2 disc + PCSX2, both games): what the HUD shows for the thing under the reticle - the BF2 HUD's Target element (`player1target`, FUN_003acae8; `.target.className`, `.target.healthFraction`, `.weapon1.target.teamColor`), BF1's equivalent if any - WHEN it appears (on aim, after a hit, a hold time), what it names, how the bar behaves as damage lands, vehicles / turrets / buildings, the team colours. Then build it on the HUD tree.
created: 2026-10-20
first: 11
---

## Why
(the maintainer, 2026-10-20) The originals show the health of what you are shooting at at the top of the screen and it drops as you hit it - soldiers, and vehicles you hit with bullets, grenades or rockets. The maintainer does not remember whether the PS2 needs a hit first, a reticle pass or a hold of about a second, nor whether BF1 does it; he wants it on every map of both games. Colours as he remembers: teammate blue, enemy red, neutral (a droid) white.
BHUD3 lists 'the target's name and health' among a dozen HUD elements (value 4, not started); this card is that item alone, first.

## Evidence
`analysis/re-hud.md` (§2 element list, 'World markers: Target `player1target`', the reticle colour from `.weapon1.target.teamColor`); `Sim::reticle_pick` (the reticle's pick, already read by the HUD feed); card BHUD3 (the other HUD elements), BHUD2; `analysis/re-aim-assist.md` (the target the PS2 tracks).

## Acceptance
1. `analysis/re-target-hud.md` [O]: when the readout shows and hides, its content (name / class, bar, numbers?), the colours per relation, what counts as a target (soldier, droid, vehicle, turret, building, command post?), the bar's behaviour under damage (instant / eased), BF1 vs BF2; each claim pixel-measured against PCSX2 where visual.
2. Built on the BF2 HUD tree for every map of both games (BF1 maps use BF2's HUD per PLAN): the readout follows the sim's real health; a test pins the fraction, the hide rule and the colour; a golden with a target under the reticle.
3. Hits from bullets, grenades and rockets move the bar (RKEX / KNOCK landing makes blasts count).

## Notes
