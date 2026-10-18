---
id: RETRYCAP
title: Cap attempts per card and route a stuck one elsewhere: a `tries` counter on the card (a session that ends without landing or closing it counts one), after 3 builds / 2 RE sessions the picker stops handing the card out, parks it with a `resume:` naming what each try found, and files a one-line Question for the maintainer or a planner re-cut
milestone: M0
value: 4
phase: build
status: todo
deps:
files: scripts/loopctl.py,engine/artifacts/briefs/LOOP_PROMPT.md
declares:
needs:
effort:
branch:
resume:
created: 2026-10-08
---

## Why
Research rec. 9 ("agents grind past payoff": Gambi's finding that the chance of success falls to about 25 % after the third attempt; freeqaz). The loop has time caps per SESSION (300 turns / 3.5 h; LOOP_PROMPT 'Budget and wrap-up'; land-it-now at ~90 min) but NOTHING bounds how many sessions a card gets. No rule exists: `grep -in 'attempt\|retry\|tries\|give up'` over LOOP_PROMPT.md, loopctl.py and the card headers finds none (checked 2026-10-08), and `postmortem` writes a DIED journal line without counting it. RE cards cost about (cost omitted) per session; a card that keeps being re-taken is the cost concentration the audit measured (52 % of spend in RE).

## Evidence
- `scripts/loopctl.py`: `pick()` takes the top eligible card; `postmortem()` (autoloop.sh after each session) detects "a claim left or a card in-progress with no report" and writes the DIED line; `resume:` and `status=parked` already exist, and a parked card with no `branch:` becomes an ordinary card again once its deps are done (card LRES), so `parked` alone does NOT stop re-picking - the rule needs its own field the picker honours.
- Audit counts (research README): 12 of 92 sessions were resumes after a death; recurring failure class = false absence claims.

## Do
1. Header field `tries: <n>` (default 0) and `try_cap: <n>` (optional per-card override). `loopctl.py` bumps `tries` when a session ends on the card without `status=done` and without a `land` (`postmortem`, plus the session's own wrap-up for a clean "did not finish"). The count appears in `orient`'s briefing ("try 2 of 3: previous tries found: <resume text>") and in BACKLOG's tags.
2. Caps: build / measure / capture 3, re 2 (default; `try_cap` overrides). At the cap `pick()` skips the card; the card gets `status=blocked` (no `needs:`, the maintainer is not the first stop) and a generated planner item: PLANNER's next run sees "STUCK: <id> after <n> tries" with the resume lines and must do ONE of: re-cut the card smaller, cut a prerequisite RE card, file a Question for the maintainer in STATUS.md, or close it as not worth it (value 1, defer post-alpha). It may reset `tries` only by doing one of those (note in the card).
3. Never a cap on landing: a card with a live `branch:` that needs only `land.py` is a land job and is not counted as a failed try.
4. LOOP_PROMPT: one short paragraph under Rules/Budget: "Two sessions on the same card without a result is the signal to write what you found into `resume:` and stop, not to try the same path a third time." and tell RE sessions to record in `resume:` what each tried approach found before ending.
5. `loopctl.py self-test` fixtures: a card at its cap is skipped by `pick`, one under the cap is not, a re card caps at 2, a `try_cap` override works, a landing job is never skipped.

## Acceptance
- `loopctl.py self-test` and `loopctl.py check` pass with the new cases pasted; `check` rejects a non-integer `tries`.
- `loopctl.py next --lane 1 --lanes 1` against a fixture lead shows a capped card is not picked (existing self-test harness, no real claim taken).
- The existing backlog ranking is unchanged for cards with `tries: 0` (diff of the generated table before and after, pasted).
- Today's open cards need no edit (the field is optional).

## Notes
Smallest version first: if the planner routing (item 2) is too much, ship the counter, the cap in `pick()` and the orient line, and file the routing as a follow-up. Touches `scripts/loopctl.py` (the picker); the self-test is the safety net, so run it before and after. The loop is stopped (STOP file) at filing time.
