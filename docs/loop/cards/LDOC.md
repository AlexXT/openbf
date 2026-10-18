---
id: LDOC
title: `loopctl commit-docs <paths>` commits a finished RE doc atomically under the MAIN claim (docs never sit dirty in main), and autoloop.sh journals a dead session (its phase and the claim it left)
milestone: M0
value: 5
phase: build
status: done
deps: LRUL
files: scripts/loopctl.py,scripts/autoloop.sh
declares:
needs:
branch: ldoc
resume: landed 5b33161f 2026-09-30
created: 2026-09-30
---

## Why
The maintainer's INBOX note 2026-09-29 22:40 (approved loop improvements), item 5. Uncommitted RE docs in main blocked another lane's landing: TALPH waited about 30 min (memory "RE drafts
off the main checkout"). A session that dies leaves no journal line saying where it stopped. deps LRUL only because
both edit loopctl.py.

## Do
- **`loopctl.py commit-docs <path…> -m <msg>`.**
  - Claim MAIN (one chain: mkdir && pid).
  - Refuse any path outside `analysis/` or `engine/artifacts/`-excluded doc paths.
  - `git add` exactly those paths, commit (with the attribution line) and release MAIN, in one call.
  - Refuse if MAIN is held, if a path is not under `analysis/` / `PLAN.md` / `engine/CRATES.md`, or if the index
    already has staged changes.
- **autoloop.sh.** After a session ends, if its claim is still held by a dead pid, or its card is `in-progress` with no
  report file written since the session started, append a line:
  `- DIED <date> lane N: job <job> card <ID> phase <phase>, claim <name> left (freed)`.
  The launcher is live and unversioned for running lanes (memory "autoloop live script is stale"), so the change takes
  effect when a lane restarts. Say so in the report.

## Acceptance
- `loopctl.py self-test` covers commit-docs: a refusal case and a commit case, in a temp git repo.
- A scripted fake dead session produces the DIED line in a temp journal (`OPENBF_LOOP_DIR`).
- Every gate PASS, nothing declared.
