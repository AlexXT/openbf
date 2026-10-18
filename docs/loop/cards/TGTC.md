---
id: TGTC
title: Scratch cargo targets are deleted when their card is done: land.py removes <card>_target (and the card's worktree leftovers) after a landing, parking a card removes its target, stale *-scratch_target / *_pxc_target go at the next session start, and the scoreboard prints lead_tmp's apparent + btrfs-exclusive size
milestone: M0
value: 5
phase: build
status: done
deps:
files: scripts/land.py,scripts/loopctl.py,scripts/cleanse_delete.sh
declares:
needs:
branch: tgtc
resume: landed 06651bd6 2026-09-29
created: 2026-09-29
---

## Why
The maintainer's INBOX 2026-09-28: 15 `lead_tmp/<card>_target` dirs (btrfs reflinks of `engine/target`, ~45 GB apparent /
~20 GB exclusive each) piled up in one day — ~300 GB real, nearly all of /home. He wants it automatic, in the tools
(land.py), not only in cleanse. On 2026-09-29 05:30 two more were waiting (`vturb_target`, `uaud-scratch_target`,
89 GB apparent) and the planner's `scripts/cleanse_delete.sh --yes` was denied by the classifier ("Shared Scratch
Sweep") — so the tooling must do it as part of a reviewed job step, not a session's ad-hoc sweep.

## Build
1. `land.py <ID> --commit` (after main is fast-forwarded and the card is `done`): remove that card's
   `lead_tmp/<branch>_target`, and the worktree if still present (unlink the worktree symlinks first, then
   `git worktree remove` without --force — memory: a forced remove is denied). `--abort` keeps it (the card may resume).
2. `loopctl.py set <ID> status=parked|done|blocked`: remove `<branch>_target` (a parked card rebuilds from a reflink
   seed in ~10 s, per the reflink recipe).
3. `loopctl.py next` (each session start, before the claim): remove `lead_tmp/*-scratch_target` and `*_pxc_target`,
   and any `<name>_target` whose card is done/parked/blocked, or has no card, no branch and no worktree — **never** one whose card or
   branch holds a live claim (lock pid alive) or is in-progress. Reuse `cleanse_delete.sh`'s Tier A test (factor it into
   one function both call). Keep `engine/target`.
4. `loopctl.py scoreboard`: one line `lead_tmp: <apparent> apparent, <exclusive> exclusive (btrfs filesystem du -s),
   N *_target dirs` so growth shows in STATUS. `btrfs filesystem du` may need no root on a subvolume the user owns;
   if it fails, print apparent only and say so.
5. Self-test like `cleanse_delete.sh --self-test`: a fixture with a live-claim target, a done-card target, a scratch
   target and a parked-card target → exactly the right three go.

## Acceptance
- The self-test passes; a dry-run on the real tree lists exactly the dead targets; a real landing of this card removes
  its own `tgtc_target` (report the df before/after).
- Behaviour-neutral for the game: every gate PASS, nothing declared.

## Notes
- The maintainer also asked for the rule in the prompt. Planners may not edit the prompt; the tooling above enforces it anyway,
  and the exact prompt line is proposed in STATUS (2026-09-29) for the maintainer to paste.
- Past sessions learned to unlink a worktree's symlinks before `git worktree remove` — do it in code.
