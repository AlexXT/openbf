---
id: TIPS
title: BF2's tooltip bar (the bottom hint line: 'This CP is now controlled by your team.', 'Stand near an enemy CP to capture it.', enter/exit vehicle, build a turret ...): the table at 0x5ff4ac, each tip's trigger, timing and repeat rule
milestone: M3
value: 3
phase: re
status: todo
deps:
files:
declares:
needs:
effort:
branch:
resume:
created: 2026-09-30
alpha: 
---
## Why
EVMSG's RE (`analysis/re-event-messages.md` §1, 2026-09-30): the only text BF2 shows when the player's team takes the
post he stands on is a TOOLTIP — "This CP is now controlled by your team." (Locl 0x64bcc6d2) in the bottom bar
(`player1tooltipsGroup` / `player1tooltipsTEXT`: MultiLineText, NumLines 3, DisplayTime default 2, frame mesh
`hud_tooltips_frame`, fade 0.25 / 0.5; events `player%d.tooltips` / `.disable`). The port shows no tooltip.

## Leads
- The table of 0x20-byte records at 0x5ff4ac ff. in SWBFRNT2.ELF: a Locl hash + six floats / flags per tip ("Activate
  boost", "Turrets can be built here with a fusion cutter.", "Stand near an enemy CP to capture it.", "This CP is now
  controlled by your team.", "Enter/Exit vehicle: {btnb}", "Enter/Exit turret: {btnb}", "Take off / land: {btna}" …) —
  dump it whole (`lead_tmp/re/evmsg/keys.py` resolves hashes), find its reader and each tip's trigger.
- BF1's twin: `game.tooltips.*` key strings (approach_cp "THIS IS A COMMAND POST", approach_vehicle, after_enter_vehicle,
  approach_turret, approach_unbuilt, approach_health_ammo, reinforcements, flyertakeoff, disablemsg "DISABLE TOOL TIPS IN
  OPTIONS"), `FUN_002db4c8`, `ScriptCB_NextToolTipState`.
- PS2 frames: `engine/artifacts/pcsx2/ref/evmsg/bf2_hot1_cp_tooltip_xshot.png`, `bf2_hot1_kill_line.png` (bottom bar).
- An options switch turns tips off on the PS2 — the port needs the same switch (FEEL_NOTES).

## Acceptance
- RE: every tip of the table with its trigger, show time, repeat rule and text ([V]/[O], verifier run).
- Built (after BHUD2 + EVMSG's MultiLineText): a scripted scene per tip kind in which the port shows the PS2's text in the
  bottom bar, onset within ±0.5 s of the trigger and the PS2's duration ±2 frames — a unit test per rule; a PS2 versus
  port pair of the bar within ±4 px (640×480).
