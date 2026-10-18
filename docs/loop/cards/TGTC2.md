---
id: TGTC2
title: The target sweep misses capture and audit scratch: cover `<card>-render_target`, `<card>-scratch_target`, `audit-<card>-*_target` and the matching `openbf-wt/<name>` worktrees once the card is done/parked or the audit has ended — live-claim and running-process checks first
milestone: M0
value: 5
phase: build
status: done
deps: LANE3
files: scripts/loopctl.py
declares:
needs:
effort:
branch: tgtc2
resume: landed a7196742 2026-10-16
created: 2026-09-30
---

## Why
The maintainer's INBOX note 2026-09-29 22:05. PXWA and PXWB are `done`, yet `lead_tmp/pxwa-render_target` (23.3 GB real) and
`pxwb-render_target` (23.8 GB) and their detached worktrees `openbf-wt/pxwa-render` / `pxwb-render` remain. Their names
match no card id, so TGTC's sweep never sees them. The audits leave `audit-<card>-*-scratch_target` dirs and
`openbf-wt/audit-*-scratch` / `dshn-scratch` worktrees too (the orientation lists five). lead_tmp held 119 GB real on
09-29, about half of it this dead scratch. deps LANE3 only because both edit loopctl.py.

- **State at 2026-09-30 00:12 (planner):** the pxwa/pxwb render targets and worktrees are gone already (removed by hand); lead_tmp has no `*_target` but `kaad_target`. Left: the detached worktrees `openbf-wt/audit-jwal-816a9e1-scratch`, `audit-jwal-a8da834-scratch`, `audit-jwal-hyb-scratch`, `audit-jwal-scratch`, `dshn-scratch`. The rule is still needed for the next capture or audit.

## Do
- `target_verdicts` / `dead_targets` map these names to their card: `<card>-render`, `<card>-scratch`,
  `audit-<card>-*`. A target is dead when its card is done or parked (render/scratch), or when no live AUDIT claim
  exists and the audit's report is written.
- The same verdicts cover `openbf-wt/<name>` detached worktrees: unlink the shared-dir symlinks with the tool's own
  path, then `git worktree remove` without --force (memory: the guard denies `--force` and a session's `rm` of the
  symlinks; the sweeper's own code path is the route). A worktree with uncommitted changes is saved as a patch under
  lead_tmp first, or kept and listed.
- Before deleting: a live claim or a running process whose cwd or cmdline is under the path keeps it (as TGTC does).
- Age is never the criterion (the maintainer: the month-old set is the ISOs, the discs, Ghidra and the first RE docs).

## Acceptance
- `loopctl.py self-test` target cases for each new name form: dead/alive, a live claim, a running process.
- One real `loopctl.py targets --yes` run frees the PXWA/PXWB/audit scratch. Report the GB freed, from
  `btrfs filesystem du -s` before and after.
- Every gate PASS.
- **State at 2026-09-30 05:00 (planner):** one more detached scratch worktree, `openbf-wt/audit-leaf-scratch` (515b024, the LEAF audit's untracked examples); `lead_tmp/audit-leaf_target` is already gone. The only `*_target` is `ldoc_target` (live claim).

- Planner 2026-10-12 19:30: one more for this card's pass over `loopctl.py`. `eligible()` compares `files:` as exact
  strings, and sessions write short paths (`core/src/ai/steer.rs` on DOORS, `core/src/nav.rs` on CORN; four older
  cards too): DOORS in progress did not hold SKIRT or OBSV back, both of which edit `steer.rs`. I normalised six
  cards by hand and the DOORS session wrote the short form again. Wanted: `set` / `new` rewrite a `files:` entry
  that does not exist but does under `engine/crates/` to the full path, and `check` reports one that exists nowhere
  unless the card says it is a new file. Also: a planner's `set` on another session's in-progress card writes
  `owner: PLANNER <pid>` on it (`note_owner`) - it should not when the card's own claim is live.

## Outcome (2026-10-16, lane 1) - LANDED a719674
render targets/trees, cmdline holders, files: normalisation + check warnings, note_owner live guard; self-test 17 new cases,
6/6 mutations killed; real targets --yes freed 0 GB (nothing dead today). Report reports/*_card_TGTC2.md.
