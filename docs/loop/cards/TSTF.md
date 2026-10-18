---
id: TSTF
title: Four test defects found by TSTX's inventory - a temp-file race between bake classes tests, a loadscreen test that panics instead of skipping, a `return` that ends a terrain test's map loop early, and roster / leaf checks that pass vacuously in a worktree (their data sits in gitignored lead_tmp)
milestone: M0
value: 3
phase: build
status: todo
deps: TSTX
files: engine/crates/bake/src/classes.rs,engine/crates/bake/src/loadscreen.rs,engine/crates/mungefs/src/terrain.rs,engine/crates/game/tests/suite/roster.rs,engine/crates/game/src/viewer/leaf.rs
declares:
needs:
effort:
branch:
resume:
created: 2026-10-02
alpha: no
---

## Evidence (`lead_tmp/tstx/test_inventory.md`, "Other problems found"; read-only inventory, NOT yet checked by hand)
1. `bake/src/classes.rs`: three tests write the same per-process temp file; under `cargo test` (threads, one pid) they
   can delete each other's file. Under nextest (one process per test) the race cannot happen, but the fallback path
   still runs them as threads. Give each its own name.
2. `bake/src/loadscreen.rs:160`: no check that the BF2 COMMON file exists - panics where every other data test skips.
3. `mungefs/src/terrain.rs:920` and `:1164`: a `return` that probably should be `continue` - one missing map ends the
   loop and the rest are never tested.
4. Some `game/tests/roster.rs` and `game/src/viewer/leaf.rs` checks read `engine/artifacts/lead_tmp/` data (gitignored):
   in a worktree whose `engine/artifacts` is not linked they pass WITHOUT testing anything. Either move the data they
   need under the tracked tree (small files) or make the absence a printed SKIP and count them.

## Do / Acceptance
Check each claim first (the inventory is a model's read). Fix the real ones; for 3, say how many more maps are now
tested. ci counts may move; nothing fails.
