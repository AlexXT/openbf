---
id: HASH
title: `Sim::hash` omits `assist` and most of `Bolt` — a hash-contract change (a declared sim re-baseline)
milestone: M0
value: 2
phase: build
status: todo
deps:
files:
declares:
branch:
resume:
created: 2026-09-25
---

## Evidence
analysis/PLAN_LOG_2026.md §ARCHIVED 2026-09-25 HANDOFF-OPUS.md §Standing backlog 7 (B1). `--sim-hash 100` is pinned by a test in `core/src/sim/state.rs`.

## Acceptance
The missing fields hashed; the pinned hash and `gates.toml` sim rows re-baselined, declared `sim`; replays still identical run to run.
