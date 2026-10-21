---
id: HATK
title: Attacker markers on the HUD: when you are hit, the PS2 BF2 HUD points to where the shot came from (on screen, behind you, off screen) - the Target element's attacker icons
milestone: M3
value: 4
phase: re
status: todo
deps: BHUD2
files:
declares:
needs:
effort:
branch:
resume: RE FIRST: the PS2 BF2 HUD's attacker markers (`player1target`: 'objective / flag / attacker / locked-on icons on screen, behind, off screen'): what draws when the player is hit, its look (icon or arrow, colour), its fade, its direction maths on screen / behind / off screen; BF1's equivalent if any.
created: 2026-10-20
first: 12
---

## Why
(the maintainer, 2026-10-20) When the player is shot, the original shows where the shot came from - he thinks a little arrow on screen pointing the way (maybe from another game). The BF2 HUD config does list attacker icons in its world-marker element; the port draws none.

## Evidence
`analysis/re-hud.md` §2 'World markers', `hud_target_*` textures (`re-hud-icons.md`); the sim's `last_hit` (the attacker) in `Unit`.

## Acceptance
1. `analysis/re-hud-attacker.md` [O] with PCSX2 captures: exactly what appears and where, for a hit from the front, side, behind and a far shot; BF1 vs BF2; or a receipted finding that it is not in a PS2 game (then the port may add one as a named modern option, the maintainer's call).
2. Built on the HUD tree; a test pins the direction; a golden.

## Notes
