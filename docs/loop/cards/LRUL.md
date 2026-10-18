---
id: LRUL
title: Loop rules in loopctl: audit by risk (item 3) and the M-ALPHA picker — `alpha: yes` first at equal value, `defer: post-alpha` and `acceptance: needed` never picked while other work exists, the counts in the scoreboard, the card rules printed in `orient`
milestone: M0
value: 5
phase: build
status: done
deps:
files: scripts/loopctl.py,engine/artifacts/loop/audits.md
declares:
needs:
branch: lrul
resume: landed e937a13b 2026-09-30
created: 2026-09-30
---

## Why
The maintainer's INBOX note 2026-09-29 22:40 (approved loop improvements), items 3 and 6. Both change `scripts/loopctl.py`, so they are one card.
- **Item 3, audit by risk.** All 5 audits of vehicle or behaviour research found problems (HOVR, VDTH, VTURB, VWLK,
  VENT). The 3 audits of rendering research found none (SUND, TALPH, DSHN), because pixel pairs already check those.
  The JWAL audit (PARTLY) fits the same pattern.
- **Item 6.** The planner tagged the cards on 2026-09-30 (`alpha: yes`, `defer: post-alpha`, `acceptance: needed`).
  Only the values enforce that order today (deferred value-4 cards were lowered to 3, with `value_pre_alpha` kept).

## Do
- **Audit picker.** Pick the most recent landing or `[O]` claim not yet in `audits.md`, preferring these domains:
  sim, AI, vehicle, physics, collision, and any RE a build depends on for behaviour.
  - Rendering RE is audited only when its PS2 pair check failed, when it touched shared code, or as a 1-in-3 sample.
  - The picker cannot read a report's verdict, so write a `domain:` line per audit target into `audits.md` and let
    the audit session choose from a list `orient` prints.
- **audits.md.** Every entry gets a `domain:`. Back-fill the existing entries (HOVR vehicle, VDTH vehicle, VTURB vehicle,
  VWLK vehicle, VENT vehicle, SUND render, TALPH render, DSHN render, JWAL collision). The scoreboard prints the error
  rate per domain.
- **Picker.**
  - At equal value, `alpha: yes` goes first.
  - A `defer: post-alpha` card is taken only when no alpha or untagged card is eligible.
  - An `acceptance: needed` card is never taken; the planner gets the refill instead.
  - `BACKLOG.md` shows the tags.
- **Scoreboard / orient.**
  - An M-ALPHA line: alpha cards open / in progress / done, and the blockers.
  - Counts of deferred and `acceptance: needed` cards.
  - `orient` prints the card rules the maintainer approved:
    - a research pass opens at most 2 new cards, unless a card is value ≥ 4 or blocks a landed card; extra findings go
      as notes on existing cards;
    - every card states a measurable acceptance (a parity delta, a leak-census count, a sim replay hash, a pixel
      test).
- **self-test** covers each rule (a deferred card loses to an untagged one; an `acceptance: needed` card is never
  picked; a render audit is skipped unless sampled).

## Acceptance
- `loopctl.py self-test` PASS with the new cases.
- `loopctl.py next --lane 1 --lanes 2` on a copy of today's cards (`OPENBF_LOOP_DIR`) picks an alpha card.
- `scoreboard` prints the M-ALPHA line and the tag counts.
- Every gate PASS, nothing declared (tooling only).
