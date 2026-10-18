---
id: CFOV
title: The PS2 soldier camera's field of view: BF1 48.0° × 37.69°, BF2 65.0° × 52.06° (h × v), measured in RAM; the port's chase camera uses 70° vertical
milestone: M1
value: 5
phase: build
status: done
deps:
files: engine/crates/game/src/walk.rs,engine/crates/render/src/camera.rs
declares:
needs:
branch: cfov
resume: landed cded403b 2026-09-28
created: 2026-09-25 (from QUEUE.md)
---

## The maintainer's decision (2026-09-25)
Use each game's PS2 VERTICAL FOV (BF1 37.69 deg, BF2 52.06 deg) as the default, widened horizontally for the window (Hor+, never stretched), and add an FOV option to the settings so a modern screen can open it up.

## Outcome (2026-09-28, lane 1)
LANDED cded403: `WalkSim::camera` takes `ps2_fov_y(source_game)` × the player's FOV option (pause menu, profile
`fov_scale`, 0.75–2.0, capped at 100°); Hor+ unchanged. No committed golden frames through `WalkSim::camera` (the goldens
use GOLDEN_CAM / overview / gallery cameras), so nothing moved; sim/simhash/sweep/arenas PASS. Pairs: BES1 and KAS2 spawn
pairs line up; 213/214 PCSX2 states carry the exact value (outlier → CFOVM). Report `reports/20260928_1435_card_CFOV.md`.

## Why
Every port frame on every map is framed at 70° vertical; the PS2's soldier camera is 37.69° (BF1) / 52.06° (BF2)
vertical. It is the widest-reaching look gap left (every PCSX2 pair is misframed by it) and it sets feel.

## Evidence
PS2 RAM on every state in `pcsx2/ref/states/` (third person, standing, zoom 1): BF1 cam+0x88/+0x8c = 0.44523/0.34131,
BF2 cam+0x138/+0x13c = 0.63707/0.48838 (FINDINGS F5; offsets named by `analysis/re-sky-pass.md` §2/§4). The port lines
up with the PS2 frames only at those values (`lead_tmp/re/pxc/pose/p_s1_fov.png`).

## Acceptance
- The chase camera's vertical FOV = 2·atan(0.34131) (BF1) / 2·atan(0.48838) (BF2), labelled PROVISIONAL (measured in
  RAM, the writers unread → CFOVM); horizontal widens with the window (Hor+), never stretched.
- An FOV option in the settings (a vertical-FOV override or a scale; the default = the PS2 value); one FEEL_NOTES line.
- Every golden that frames through `WalkSim::camera` is a DECLARED move (list them from the code first) and viewed;
  sim, simhash, sweep, arenas unmoved. One PCSX2 pair per game at a spawn pose lines up (`pcsx2/pairs.py`).

**Planner (2026-09-28):** re → build, value 4 → 5. The third-person standing value is measured and is what the maintainer's
decision needs; the per-mode RE (crouch/prone/zoom/first person/spawn/death, BF2's 48° camera) moved to CFOVM so it
does not hold the default back. Reverse: `set CFOV phase=re` and fold CFOVM back in.

## Original QUEUE.md entry (verbatim, 2026-09-25)
CFOV. [TODO — **[2026-09-25 the maintainer] SCOPE: this is a modern PC port. The measured PS2 FOV (BF1 48.0° × 37.69°, BF2 65.0° × 52.06°, 4:3) gives the intended VERTICAL FOV/framing; the port must keep that vertical FOV and widen the horizontal FOV with the window's aspect ratio (Hor+), fitting any resolution/aspect without stretching or 4:3 letterboxing — never hard-code the 4:3 horizontal FOV. See `engine/artifacts/pcsx2/ref/COMPARISON_RULES.md`.**] — ADDED 2026-09-25 (lane 1) by PXC batch 1 (`engine/artifacts/pcsx2/ref/FINDINGS.md` F5); **[RE] FIRST** (job D, at max), then a build [GOLDENS]; the RE is FREE (disc + `analysis/`); the build touches `game/src/walk.rs` (and a comment in `render/src/camera.rs`) — shared with no queued entry today] **The PS2 soldier camera's field of view: BF1 48.0° × 37.69°, BF2 65.0° × 52.06° (h × v), measured in RAM; the port's chase camera uses 70° vertical** [RE] — measured: `engine/artifacts/pcsx2/pose.py` on every state in `engine/artifacts/pcsx2/ref/states/` (third person, standing, zoom S = 1): BF1 cam+0x88 / +0x8c = 0.44523 / 0.34131 (near 0.63), BF2 cam+0x138 / +0x13c = 0.63707 / 0.48838 (near 0.5) — the offsets `analysis/re-sky-pass.md` §2 (BF2 `FUN_0057b8b0`: cam+0x138 = tan(fov·0.5), +0x13c = that · aspect) and §4 (BF1 `FUN_00491008` / `FUN_00490268`) name; the port renders line up with the PS2 frames only at those values (`engine/artifacts/lead_tmp/re/pxc/pose/p_s1_fov.png`: PS2 | 37.7° | 70°). The port: `WalkSim::camera` (`game/src/walk.rs`) takes `Camera::default()`'s `fov_y` = 70° (`render/src/camera.rs:36`), the placeholder PLAN §Presentation says the traced value "drops into when camera RE lands". Leads from PXC's verifier (2026-09-25, all eight citations SUPPORTED): BF2 `FUN_0052a968` is fdlibm `tanf` (SWBFRNT2 657906; `FUN_0052a308` is `atanf`); the aspect argument is `FUN_005805c8()` = `DAT_007c56e8` = 0x3f443fe6 = 0.7666 (set in `FUN_00580b08`, 715827; split screen passes ×0.5, 63179/63183; the constructor `FUN_0057a5b8` passes 0.75, 710783); `FUN_001b93b0` builds a camera at 62937–62940 with `FUN_0057b8b0`(near 0.7, far 120, **fov 48°**) and makes it active (`FUN_00582c50`, 63010) — yet the soldier camera reads 65.0°, so the value is per mode; BF1's `FUN_00490fc0` (554166–554167) stores +0x88 = argument 3·(0.5/near) — start there. RE (§5): the writers of BF1 cam+0x88/+0x8c and BF2's `FUN_0057b8b0` callers — which constant, ODF or config value each camera mode passes (third person standing / crouched / prone, first person, scope zoom, vehicles and turrets, the spawn-select and death cameras); read each constant's ELF bytes (check 4); an absence claim ("no per-class FOV") needs the search (check 2); a PCSX2 state per mode confirms the value (`pose.py` prints it). Build (after the RE): the chase camera takes the traced VERTICAL FOV per game (the port stays hor+, PLAN §Presentation), PROVISIONAL [O] where the RE is [D]. GATE (RE): `analysis/re-camera-fov.md` (§5 header, [O] tags, the verifier's verdicts), an RE_LOG pointer. GATE (build): ci + `scripts/gpu_tests.sh`; every golden that frames through `WalkSim::camera` (list them from the code before building) is a DECLARED move with sheets; the sweep and the replay hashes UNMOVED (the camera is not in the sim); one PCSX2 pair per game at a spawn pose lines up (`pcsx2/pairs.py`). Files: RE — `analysis/re-camera-fov.md`, `analysis/RE_LOG.md`, scratch `engine/artifacts/lead_tmp/re/camera-fov/`; build — `game/src/walk.rs`, `render/src/camera.rs`.
