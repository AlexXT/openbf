---
id: SSEL
title: The start-of-level and respawn flow: pick a SIDE (or random), then a command post, then a class at the start of a level; post and class after a death - the PS2 BF2 screens, on the sim's real spawn choice
milestone: M5
value: 5
phase: re
status: todo
deps: SPSEL
files:
declares:
needs:
effort:
branch:
resume: RE FIRST, from the PS2 disc and PCSX2 (not the PC game, whose screens differ): capture the BF2 PS2 start-of-level sequence (side select incl. random, then the spawn map to pick a command post, then class select) and the death/respawn screen, and write `analysis/re-spawn-flow.md`; then build on the sim's spawn choice (SPSEL is the sim half).
created: 2026-10-19
first: 5
---

## Why
(the maintainer, 2026-10-19, and Reddit's top ask after the release) At the start of a level the player should choose a side (or random), then a command post, then a class; after a death, the post and the class again. Today there is no side pick at all, the post cursor on the spawn screen is UI-only (the sim picks the spawn node itself), and the class row is real (`Sim::spawn_choice`).

## Evidence
`ui/src/spawn.rs` (the spawn screen: class row real, post cursor UI-side); card SPSEL (the spawn POST in the sim, the PS2's spawn-map pick); card BMAP2 (the spawn screen's map view); card CFOVM (the PS2 camera FOV in the spawn-select and death cameras); card UISN (the screens' sounds); `analysis/re-ambience-music.md` §6 (the PS2 spawn-screen state machine, already read). The in-game unit select (the rotating trooper line-up) is likely close to the same on PC and PS2 - verify against a PCSX2 capture before reusing a layout. PC-derived layouts are NOT the PS2's: the front-end shell differs between the platforms.

## Acceptance
1. `analysis/re-spawn-flow.md` [O]: the PS2's screen order, what each screen shows and which inputs move it, the side-select options (side 1 / side 2 / random / spectate?), what the death screen is, with PCSX2 captures cited; absence claims carry receipts per RULES.
2. Build (own cards): a side-select screen at level start, post pick that reaches the sim (with SPSEL), class pick with the unit line-up, the respawn flow; each pinned by a test on the sim's choice and a golden of the screen.
3. Replays reproduce: the choices ride the input log like the class choice does.

## Notes

