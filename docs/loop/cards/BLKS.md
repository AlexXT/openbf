---
id: BLKS
title: The black box's human row carries the sprint's inputs (energy, the depleted latch, the stance's time, MaxSpeed) so 'sprint does nothing' can be read from a snapshot (the maintainer 2026-10-04: no sprint as the BF1 stormtrooper on KAS2, fine as BF2 clones)
milestone: M0
value: 4
phase: build
status: todo
deps:
files: engine/crates/game/src/blackbox.rs
declares:
needs:
effort:
branch:
resume:
created: 2026-10-04
alpha: yes
---

## Why
The maintainer, 2026-10-04: "sprint worked when I was playing the clones, for stormtrooper sprint wasn't working or doing anything so that should be investigated."

## Evidence
Read from his snapshots that day (all build 1bde70c):
- `playtest/prev2.json` (BF1 KAS2, imp_inf_storm_trooper, 2026-10-03): sprint held, 6.45 -> 9.67 m/s. The class can sprint.
- `playtest/prev1.json` (BF1 KAS2, the same class, 2026-10-04 12:15): two samples with the sprint bit, both in stance 3, speed 6.0 and 6.3 m/s (one airborne). Either short taps or a sprint that did not build speed - the row carries no energy, so it cannot be told.
- `playtest/last.json` (BF2 KAM1, rep_inf_ep3_rifleman): 8.0 -> 10.3 m/s.
- Ruled out from the rows: the 0.9 x MaxSpeed entry floor (run speed 6.16-6.45 against a floor near 5.8) and ground flicker (0 flips in 149 dense ticks).
- Since commit 6300e19 one click of L3 latches the sprint (`input::SprintLatch`), which removes the short-tap case.

## Acceptance
- The human row has energy, the depleted latch, time in stance and the class's MaxSpeed; the digest prints them; `blackbox_read` shows a sprint that was asked for and did not build speed.
- A headless check: the BF1 stormtrooper on KAS2 with the stick forward and SPRINT held for 5 s reaches 1.5 x MaxSpeed on flat ground, on the beach's slope and in the shallows; if it does not, the cause is fixed here or carded.

## Notes
Filed 2026-10-04 by Claude in chat from the maintainer's playtest (build 1bde70c, main 4e51e84). The maintainer's own words are quoted; nothing here is measured unless it says so.
- 2026-10-05 (the maintainer, via Claude): the snapshot's header also names the ACTIVE CHEATS. The maintainer has played every test so far with invincibility and infinite ammo on, so no kill / death number in a snapshot to date describes a human (card HSTD must not use them).
- 2026-10-05 planner: value 3 -> 4: card HSTD (4, the first step of the maintainer's difficulty cards) waits on this one, and the maintainer asked for the stormtrooper's sprint to be investigated. Undo: `loopctl.py set BLKS value=3`.
- 2026-10-06 planner: **read SPUP's find before the headless check** (c1ae263, `reports/20261006_1310_card_SPUP.md`). The maintainer's left stick peaks at 32255 of 32767 (0.984) in every snapshot; a sprint start asked 0.9 x MaxSpeed and his run on a slope fell under it. That may be the whole of "the stormtrooper's sprint did nothing on KAS2" (a beach and slopes) while "fine as BF2 clones" was Kamino's flat decks. First step: in `playtest/prev1.json` read the stick's peak, the floor's slope under the two sprint-bit samples and the speed against 0.9 x MaxSpeed (BF1's uphill rule is the normal's y). If SPUP's two changes (`input::STICK_FULL` 0.95; the start test along the floor) already cover them, say so on the card and keep only the black box's fields + the headless check. The snapshots predate SPUP (build 1bde70c).
