---
id: BAKEG
title: Gate tooling: a scratch bake of the 'sounds' group without 'classes' recovers fewer sound names (114 vs 464 on TAT1) — scratch_bake.sh must link or bake classes/ whenever sounds is baked, or land.py's main bake check reports every */sounds/names.ron as not reproducible
milestone: M0
value: 5
phase: cleanup
status: done
deps:
files: scripts/scratch_bake.sh,scripts/gates.py
declares:
needs:
effort:
branch: bakeg
resume: landed ff13dbaa 2026-10-03
created: 2026-09-29
alpha: yes
---
## Why
VTUR's landing (2026-09-29) declared rebake=...,sounds,global-sound and land.py's main bake check reported DIFF=34 — every */sounds/names.ron — because the scratch bake's classes/ was empty (the sound-names step reads $OUT/classes). Main's own rebake was right (464 names on TAT1, matching the earlier full-group scratch); the landing was committed on that reading. The next card that declares sounds hits the same false alarm.

## Evidence
engine/artifacts/lead_tmp/vtur/land/main/cmp_bake.txt (34 DIFF, all names.ron); diff of tat1/sounds/names.ron: candidates 813983 / named 464 (main) vs 434090 / 114 (scratch). scripts/scratch_bake.sh:77 creates an empty classes/. Also: ffmpeg was upgraded to 8.1.3 on 2026-09-26, so every OGG re-encodes byte-differently from a pre-09-26 bake; VTUR's landing re-encoded all audio on main (sounds + global-sound), so main is consistent again.

## Acceptance
scratch_bake.sh bakes (or symlinks the shared) classes/ before sounds; a scratch 'sounds' bake cmp's DIFF=0 against main; behaviour-neutral.

- Planner 2026-10-02 14:45: value 3 -> 4, `alpha: yes` (as GSKIP was: a gate tool the alpha cards land through). SNDT's landing hit it again (34 `names.ron` DIFFs, 'bake on main not reproducible', committed through on the lander's reading) and FOLY, WSND, VSND, AMBM and USND will each declare `rebake=sounds`. Land this before FOLY's landing if a lane is free. Undo: `loopctl.py set BAKEG value=3 alpha=`.

- Planner 2026-10-02 23:55: value 4 -> 5, `files:` set. It was hit a third time (FOLY, d51ff74: "bake on main not
  reproducible DIFF=34", all `*/sounds/names.ron`; VTUR and SNDT before it) and each time the lander committed through a
  red line on his own reading. WSND's build, VSND's build, AMBM and FOL2 all declare `rebake=sounds` next: a real
  `names.ron` defect would be waved through the same way. At value 4 it was never picked (the 14:45 planner asked for it
  "before FOLY's landing"). Undo: `loopctl.py set BAKEG value=4`.
- **Item 2 (from TSTX's landing, `reports/20261002_2144_land_TSTX.md` "For the planner"): a partial rebake can leave
  another group stale.** STARC (c9d2776) changed `mungefs/coll.rs` and declared `rebake=maps`; the vehicle collision in
  the `side` group (`side_bf2/vehicles/rep/collision/rep_fly_vwing.glb`, 342 bytes) stayed stale for 12 hours until
  TSTX's full bake gate refused an unrelated branch on it. Do: when a diff touches bake code (`mungefs|bake|manifest`,
  gates.py's own test at its line ~513) and the card declares only some groups, the branch gate's scratch bake compares
  EVERY group and refuses a DIFF in an undeclared one, naming the file and the group to declare - or, if a full scratch
  bake per landing costs too much, say what it costs (time it) and do it for diffs under `mungefs/` only.
  Acceptance for item 2: a `gates_selftest.py` / `land.py --self-test` fixture - a diff touching `mungefs/src/coll.rs`
  with `rebake=maps` and a DIFF in `side` - is refused with the file named; behaviour-neutral for every other landing.

- **Done 2026-10-03 (ff13dba, report `reports/20261003_0100_card_BAKEG.md`).** Item 1: a scratch `sounds` bake links the
  shipped classes/ - cmp against main DIFF 34 -> 0 (TAT1 named 114 -> 465). Item 2: a bake-code diff with a declared
  partial rebake bakes every group (about 45 s) and an undeclared group's difference FAILS with the file and group named
  (`gates_selftest.py`: the STARC fixture; mutation fails 4 checks). The path -> group table matched all 22690 files of
  ten per-group scratch bakes.
