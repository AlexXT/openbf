---
id: CLN11
title: Cleanse 16 (game): hygiene, comments-are-claims, ledger and tests over the game diff 306c4f0..988b229, CRATES.md counts, artifact READMEs
milestone: M0
value: 3
phase: cleanup
status: done
deps:
files: engine/CRATES.md,engine/crates/game/src/walk.rs,engine/crates/game/src/music.rs,engine/crates/game/src/announce.rs,engine/crates/game/src/holo.rs,engine/crates/game/src/point_light.rs,engine/crates/game/src/blackbox.rs
declares:
needs:
effort:
branch: cln11
resume: landed d896ca4 2026-10-15
created: 2026-10-15
owner: CLEANSE 6067
---

## Why
Cleanse 16, area game (diff 306c4f0..988b229: 87 commits touching the crate). Two read-only Opus reviewers (an adversarial read of the
changed source; hygiene, ledger, tests, CLI flags, artifact READMEs) and the lead. Findings and their end states are in the report.
Found so far: CRATES.md 24 stale line counts (VERIFIED, size_report --check-map).

## Evidence
`engine/artifacts/lead_tmp/cln11/` (ci0.log: ci green on the untouched branch, the cleanse's precondition).

## Acceptance
- `size_report.sh --check-map` 0 stale counts, `--strict` exit 0, `analysis_index.py --check` exit 0.
- Every gate PASS, nothing declared (sim hashes and the seed-42 sweep equal main's).
- Each finding fixed, declined with a reason, or carried to a card (the report lists them).

## Notes
