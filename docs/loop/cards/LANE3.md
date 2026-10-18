---
id: LANE3
title: A third research lane: first time PCSX2's software renderer vs HW on one savestate (≤ 4 levels, CPU use), then LANE_PREF lane 3 (re, audit, capture), a gpu.lock / ci.lock / pcsx2.lock split, and a per-session XDG_CONFIG_HOME PCSX2 ini — the maintainer starts the lane himself
milestone: M0
value: 5
phase: build
status: done
deps: LDOC
files: scripts/loopctl.py,scripts/gates.py,engine/artifacts/pcsx2/pcsx2_session.py,engine/artifacts/pcsx2/pcsx2_capture.sh,engine/artifacts/pcsx2/README.md
declares:
needs:
branch: lane3
resume: LANDED d5df318 2026-09-30; artifacts side (pcsx2_session.py, pcsx2_capture.sh, README, ctl.sh) edited in place, backups in lead_tmp/lane3/backup
created: 2026-09-30
---

## Why
The maintainer's INBOX note 2026-09-29 22:40 (approved loop improvements), item 4. Research and capture are about 62% of spend and the queue's bottleneck. Three things stop a third lane
today:
1. `LANE_PREF` has only lanes 1 and 2.
2. Every PCSX2 run holds the one global `ci.lock`, so a RAM-only research session blocks the GPU gates.
3. `lead_tmp/re/pxc2/ctl.sh` edits the GLOBAL `~/.config/PCSX2/inis/PCSX2.ini` in place, so two sessions would fight
   over it.
Expected cost: about +(cost omitted)/day at current rates. The maintainer starts it with `LANE=3 scripts/autoloop.sh`. The loop never
launches autoloop.sh. deps LDOC only because both edit loopctl.py.

## Do (in this order; stop after the experiment if it fails)
1. **Experiment first.** On one savestate, boot the software renderer (`Renderer = 13`) and HW, from a private
   `XDG_CONFIG_HOME` ini copy.
   - Time both sessions.
   - Compare the frames (target ≤ 4 levels; PXC batch 2's control found water and haze within 4).
   - Record CPU use (32 threads, 62 GB).
   - Write the result into the PCSX2 README.
2. **Per-session config.** The launchers copy the ini into a per-session `XDG_CONFIG_HOME`, as they already do for
   `XDG_CACHE_HOME`. A RAM-only mode picks the software renderer.
3. **Locks.**
   - `gpu.lock`: GPU renders and HW PCSX2.
   - `ci.lock`: cargo and gates.
   - `pcsx2.lock`: a small cap on concurrent emulators (a counting lock: N slot dirs).
   - The scripts take the right one. The prompt wording goes to STATUS as a proposed line.
4. `LANE_PREF[3] = ["re", "audit", "capture", "measure"]`; `pick()` lets lane 3 take the periodic audit.

## Acceptance
- The experiment's numbers are in the README (time HW vs SW, max level difference, CPU %).
- Two PCSX2 sessions run at once with different renderers and neither touches `~/.config/PCSX2`: diff before/after.
- `loopctl.py self-test` covers the lane 3 preference.
- Every gate PASS, nothing declared.
- STATUS tells the maintainer the lane is ready.

## Planner note (2026-09-30 05:00)
Value 4 → 5. Untagged at 4 it sat behind ~33 alpha value-4 cards (the picker takes alpha first at equal value), yet
30 of the 50 open alpha cards are RE, and a third lane that prefers re/audit/capture is the direct lever on that queue.
It becomes eligible when LDOC lands (both edit loopctl.py). Reverse: `loopctl.py set LANE3 value=4`.
