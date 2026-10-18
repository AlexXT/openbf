---
id: VLOCK
title: In-vehicle aim assist: BF2's lock-on for every seat weapon (the reversible default VTUR took from PLAN §Vehicles & space) — RE BF2's lock button and acquisition loop, then build it for riders
milestone: M2
value: 3
phase: re
status: todo
deps: VTUR
files:
declares:
needs:
branch:
resume:
created: 2026-09-29
defer: post-alpha
---
## Why
VTUR recorded the call (FEEL_NOTES 2026-09-29): vehicles use BF2's lock-on, not BF1's magnet. Nothing is built: a seat weapon has no assist.

## Evidence
re-vehicles §9: BF2's weapon class parses LockOnRange/LockOnAngle/LockOffAngle and TargetLockMaxDistance / ...Lose (B2 FUN_00328fb0, B2:319976-320750); six Controllable ticks copy them into one per-player lock object (B2:109086, 116820, 132164, 159266, 179562, 188918). The lock button and BF2's acquisition loop were not located [H]. BF1 alternative: the magnet runs in any seat whose weapon has LockOnRange > 0.

## Acceptance
RE (discipline, verifier): BF2's lock acquisition, hold and loss, its button; then a build card or this one re-phased with the numbers; FEEL_NOTES updated; STATUS 'Calls you may want to reverse' keeps the line.

## Planner note (2026-09-30, M-ALPHA)
`defer: post-alpha` — not needed for the alpha's six showcase maps or its checklist. Value 3 (kept).
