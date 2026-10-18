---
id: PADM
title: Controllers work out of the box: gilrs has no mapping for the maintainer's GameSir G7 Pro (USB 3537:1022), so the right stick (camera) is dead and X/Y are swapped — bundle SDL_GameControllerDB + a project mapping file via GilrsBuilder::add_mappings, log each pad's resolution, warn on an unresolved camera/fire
milestone: M3
value: 5
phase: build
status: done
deps:
files: engine/crates/input/src/lib.rs,engine/crates/input/pads.txt,engine/crates/input/examples/pads.rs,engine/crates/game/src/main/windowed.rs,engine/crates/ui/src/hud.rs,engine/CRATES.md
declares:
needs:
branch: padm
resume:
created: 2026-09-29
---

## Why
The maintainer's INBOX 2026-09-28: on his GameSir G7 Pro the camera cannot turn, so nothing is testable on a pad. Value 5 by his
word ("M3 or earlier"). Engine plumbing, not PS2 behaviour: write Rust directly.

## Evidence (Claude's probe for the maintainer, gilrs 0.11.2 / gilrs-core 0.6.8 = engine/Cargo.lock, on the real pad)
- USB `3537:1022`, generic HID (usbhid, js0), SDL GUID `03000000373500002210000010010000`; 15 buttons (evdev 304–318),
  axes X/Y/Z/Rz + GAS/BRAKE + HAT0. The right stick is ABS_Z / ABS_RZ, not RX/RY.
- gilrs's bundled DB has no entry: `map_name=None`, `mapping_source=Driver` → `RightStickX/Y -> None` (camera dead),
  North=307 / West=308 (the reverse of SDL's: RELOAD and ACTION swapped), triggers only digital TL2/TR2.
- Upstream SDL_GameControllerDB (mdqinc, 2026-09-28) lacks it too; the same-layout GameSir pads' mapping, applied to this
  GUID, resolves everything. **Verified by the maintainer on the real pad (2026-09-28 evening) — ship it as is:**
  `03000000373500002210000010010000,GameSir G7 Pro,a:b0,b:b1,back:b10,dpdown:h0.4,dpleft:h0.8,dpright:h0.2,dpup:h0.1,guide:b12,leftshoulder:b6,leftstick:b13,lefttrigger:a5,leftx:a0,lefty:a1,rightshoulder:b7,rightstick:b14,righttrigger:a4,rightx:a2,righty:a3,start:b11,x:b3,y:b4,platform:Linux,`
- The two `gilrs::Gilrs::new()` sites: `crates/game/src/main/windowed.rs:331` and `:790`; the pad → PadState map is
  `crates/input/src/lib.rs` (FIRE = `Button::RightTrigger2` at :44, aim `Axis::RightStickX` at :109).

## Build
1. One constructor in `input` (e.g. `input::open_gilrs()`) used by both sites: `GilrsBuilder::new().add_mappings(..)`
   with (a) a bundled current SDL_GameControllerDB text (gilrs's own copy is the fallback; a vendored upstream copy is
   fine — it is zlib-licensed community data, not a game asset) and (b) a small project-owned
   `crates/input/pads.txt` for pads the DB lacks, starting with the line above. `SDL_GAMECONTROLLERCONFIG` must still
   apply on top (check gilrs's precedence; env wins).
2. Startup log per pad: name, GUID, mapping matched (source), and which of the port's axes/buttons resolved to a code.
   A clear console + on-screen warning when `RightStickX/Y` or FIRE does not resolve — never a silently dead camera.
3. Triggers: under the mapping, `RightTrigger2` is an analog axis. Make FIRE (and L2's action) work from either the
   button or the axis with one threshold (0.5, PROVISIONAL, a FEEL_NOTES line). The PS2's L2/R2 are read as digital
   buttons by both games — confirm with one `re_check.py` look at the pad read (pressure bytes unused) or record [H].
4. Unit test: a fake mapping string for the G7 GUID resolves RightStickX/Y and X=307/Y=308 (gilrs's mapping parser, no
   device needed); the existing input tests stay green.

## Acceptance
- The maintainer's pad, no env var: camera turns, face buttons match SDL's layout, R2 fires. The build session cannot hold the
  pad: prove it with the unit test on the mapping + a startup log line from a run with no pad ("0 pads"), and write
  the exact check for the maintainer in the report's For the maintainer (plug in, run, read the log line).
- declares: none expected (no sim/golden change); `ci=+N`.

## Notes
- The maintainer's workaround until this lands, from `engine/`: `SDL_GAMECONTROLLERCONFIG='<the line above>' ./target/release/game`.
