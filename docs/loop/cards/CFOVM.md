---
id: CFOVM
title: The PS2 camera FOV in the spawn-select and death cameras (the soldier camera's FOV per mode is answered by re-zoom.md: the class's ThirdPersonFOV / FirstPersonFOV, default 48 deg, BF2 soldiers 65 / 70, tan(FOV / 2) x 0.7666 / zoom) - which constant, ODF or config value each of the two passes, and the one FEL1 outlier state
milestone: M1
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
created: 2026-09-28
defer: post-alpha
---

## Why
CFOV ships the RAM-measured third-person standing FOV. The PS2 changes the FOV per camera mode (BF2's FUN_001b93b0 builds a 48° camera; the soldier camera reads 65°), so crouch/prone, first person, the scope zoom and the spawn/death cameras may differ.

## Evidence
CFOV's original QUEUE entry (verbatim in `cards/CFOV.md`): BF2 FUN_0057b8b0 callers, FUN_0052a968 = tanf, the aspect DAT_007c56e8 = 0.7666 (split screen ×0.5), BF1 FUN_00490fc0 (+0x88 = argument 3·(0.5/near)).

## Acceptance
- `analysis/re-camera-fov.md` (§5 header, [O] tags, verifier verdicts) + RE_LOG pointer: each mode's value, its source (constant / ODF / config) with ELF bytes; a PCSX2 state per mode confirms (`pose.py`).
- A build card (or a recorded no-change) for the modes the port has.

## Notes
Split from CFOV by the planner 2026-09-28. Vehicle cameras are VSEAT's (ThirdPersonFOV per seat).
- CFOV (2026-09-28, landed cded403): the default ships from `game::walk::ps2_fov_y` (tan-half 0.34131 / 0.48838). Census of `pcsx2/ref/*/*.json`: 213/214 states read exactly 37.69° (BF1) / 52.06° (BF2); the one outlier, BF2 FEL1 `cp3_cp3_1_postA_v3.json`, reads tan_half_v 0.40237 = 43.84° with far 52 and zoom_S 1.0 — a second camera state worth identifying here (not a zoom). A per-mode value slots in as a stance/mode lookup beside `camera_section` in `game/src/walk.rs`.
- UAUD1 (2026-09-29, analysis/unit-audit-bf1.md §3.6): classes carry their own CAMERASECTION values - the Wookiee smuggler's STAND track
  centre 2.25 / offset 4.0 (vs 1.8 / 3.5), the droideka's 0.5 up / 4.5 back, the locals' 2.0 eye; the port's chase
  camera uses one setup for every class. The DISTANCE and height per class are card CAMS: the PS2 sits 3.50 m back
  in 19 of 22 BF1 states, and the port's BF2 table gives 3.2. This card keeps the FOV per mode.

## Planner note (2026-09-30, M-ALPHA)
`defer: post-alpha` — not needed for the alpha's six showcase maps or its checklist. Value 3 (kept).
- 2026-10-06 planner (INBOX 2026-10-06 10:45, the ZOOM RE's find 3): **most of this card is answered** by `analysis/re-zoom.md` sections 1 and 6 ([O], verifier 48 of 52 SUPPORTED): the soldier camera's FOV is the Controllable class's `ThirdPersonFOV` / `FirstPersonFOV` (default 48 deg; BF2's soldiers 65 / 70), vertical tan = tan(FOV / 2) x 0.7666 / zoom, eased by exp(-25 dt); the scope zoom is card ZOOM's build, the scope view card ZSCP's. **What this card still holds: the spawn-select and the death cameras** (and the one FEL1 outlier state in the note above, tan_half_v 0.40237). The title is narrowed to that; value 3 and `defer: post-alpha` kept. Do not re-read the soldier camera.
