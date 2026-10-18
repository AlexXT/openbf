---
id: LRES
title: loopctl pick hands out cards it should not: (1) a cleanse's CLN* card while the cleanse session that opened it is live (it holds CLEANSE, not the card id); (2) any parked card as a land job, even with no branch and unmet deps (SQVEH, 2026-09-30 21:36 - the prompt now parks vehicle cards that lack a runnable check)
milestone: M0
value: 5
phase: build
status: done
deps:
files: scripts/loopctl.py
declares:
needs:
effort:
branch: lres
resume: landed 0c96beed 2026-10-02
created: 2026-09-30
alpha: yes
---
## Why
pick() step 1 resumes any in-progress card whose id is not a live claim. A cleanse/planner/audit session claims CLEANSE/PLANNER/AUDIT and opens a CLN* (or other) card without claiming its id, so the next free lane is told to resume that card while the owner is still working it — 2026-09-30 06:50 lane 1 was handed CLNT while lane 2 (pid 9841, CLEANSE + MAIN) ran `land.py CLNT --pid 9841`. Lane 1 stood down (report reports/20260930_0652_resume_CLNT.md); a less careful session would have edited the worktree mid-gates or run a second land.py.
## Fix (suggested)
Either a session that opens a card claims its id too (`loopctl new` under a live job claim, or cleanse.md tells the cleanse to `claim <ID>`), or step 1 skips a card whose `branch` has a live land.py/gates.py, or whose id appears in lead_tmp/<branch>/land/state.json with a live pid.
## Acceptance
loopctl self-test: a temp loop dir with CLEANSE held by a live pid and an in-progress CLN* card opened by it -> pick() does not return that card; with the CLEANSE holder dead -> it does.
- 2026-09-30 07:10 (lane 1): third lane-1 session picked CLNT while lane 2 (pid 9841, CLEANSE+MAIN) ran `land.py CLNT` — report `reports/20260930_0710_resume_CLNT.md`. Each costs a session; worth raising to 4 if it recurs.
- Planner 2026-09-30 08:30: value 3 -> 4. It recurred three times in 20 minutes (06:52-07:10) and the third auto-blocked CLNT while CLEANSE was landing it.

- Planner 2026-09-30 21:45, value 4 -> 5, a SECOND case for the same function (`pick`, scripts/loopctl.py:316-320): step 2 hands out ANY `status: parked` card as a `land` job, ignoring `deps` and whether a `branch:` exists. The prompt (the maintainer, 2026-09-30) now tells a session to park a vehicle card that has no runnable check, with `resume:` naming the check — SQACK did that for SQVEH (no branch, deps SQACK not done), and lane 3 got `{"job": "land", "card": "SQVEH", "reason": "parked, MAIN free"}` at 21:36. Fix: a parked card is a land job only with a `branch:` that exists; a parked card without one is an ordinary card job once its deps are done (its `resume:` is the first step). Acceptance, added: self-test — a parked fixture card with no branch and an unmet dep is not returned; with the dep done it is returned as `card`, not `land`; a parked card with a branch still returns `land`.
- 2026-10-02 05:16 (lane 2): fourth occurrence, first on CLNG — lane 2 was handed `resume CLNG` while lane 1 (pid 6080, CLEANSE + MAIN) ran `land.py CLNG --pid 6080` (776804, 90 s old). Stood down, report `reports/20261002_0516_resume_CLNG.md`.
- 2026-10-02 05:18 (lane 2): fifth occurrence, second on CLNG two minutes after the fourth — same owner (pid 6080, `land.py CLNG` pid 776804 in gates_branch). Stood down, report `reports/20261002_0518_resume_CLNG.md`; attempts n = 2, so the auto-block fires two picks from now while the landing is still in phase 1.
- 2026-10-02 05:19 (lane 2): sixth occurrence, third on CLNG — same owner (pid 6080, `land.py CLNG` pid 776804 still in gates_branch). Stood down, report `reports/20261002_0519_resume_CLNG.md`; attempts n = 3, so the next pick auto-blocks CLNG while lane 1 is landing it.
- Planner 2026-10-02 17:45: tagged alpha (the 14:45 planner deferred the tag until PARR was in progress; PARR landed). A THIRD case for the same step 1 (`pick`, loopctl.py ~324): it resumes an in-progress card whose `needs:` is set. TSTX (needs: the maintainer's chat session owns it) was handed out four times in 2 minutes (17:39-17:42, reports `20261002_174*_resume_TSTX.md`) and auto-blocked. Fix: step 1 skips a card with `needs:` (as `eligible()` already does). Acceptance, added: self-test - an in-progress fixture card with `needs:` set and no live claim is not returned as `resume`.
- 2026-10-02 23:35 (lane 1): LANDED 0c96bee. Acceptance run on main: `loopctl.py self-test` PASS, 10 new `pick guards` lines (cleanse owner live / dead, live land.py / dead, parked no-branch + unmet dep / dep done / missing branch / with branch, needs: set / cleared); each guard removed in a scratch copy fails its line. Report `reports/20261002_2335_card_LRES.md`.
