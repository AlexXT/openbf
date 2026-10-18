---
id: BSHL
title: The BF2 shell and menus' real look (PLAN §Presentation): today's shell (goldens/shell.png) is a working placeholder — BF2's shell art, flow and fonts, content adapted to the port's features
milestone: M5
value: 2
phase: re
status: todo
deps:
files:
declares:
needs:
branch:
resume:
created: 2026-09-28
defer: post-alpha
---

## Why
The maintainer (INBOX 2026-09-28 (d)); PLAN §Presentation: BF2's shell and menus. Today's shell (`engine/goldens/shell.png`) is a
working placeholder. The content may change to fit the port's features; the look and flow stay BF2's.

## Acceptance
- RE: BF2's shell art, fonts and flow on the disc; build cards per screen. M6 polish; after BHUD.

## Notes
- Created 2026-09-28 by the planner.

## Planner note (2026-09-30, M-ALPHA)
`defer: post-alpha` — not needed for the alpha's six showcase maps or its checklist. Value 2 (kept).

## Planner note (2026-09-30 13:30) — the greyed-out Multiplayer entry (the maintainer, INBOX 2026-09-30 10:05)
Milestone M6 → **M5** (PLAN §Milestones now: M5 campaign/GC/polish → M6 PACK → M7 Multiplayer, held). Part of this
card's acceptance, whichever build makes the main menu (this card, or the placeholder shell if a main menu is added before):
- The main menu shows **Multiplayer** in the slot it has in BF2's main menu, drawn in the disabled style. It cannot be
  chosen: focus skips it, or confirm does nothing — whichever reads better with the pad (record the call in FEEL_NOTES.md).
- The entry carries an `enabled` flag in data (default false), so switching it on later is one line.
- A test pins the entry present and disabled. Nothing behind it is built.
Today's shell (`ui/src/shell.rs`) is the Instant Action setup only, with no main menu, so there is nothing to change yet.
- 2026-10-06 planner (from ERAM's report, 8d6c15a): two small things in today's working shell that this card's rework should not carry over, and that are cheap to fix alone if the maintainer asks (a shell golden moves): (1) the army rows read "Team 1 / Team 2" although Team 1 is always the PLAYER's army since ERAM (picking Empire first makes you the Empire, at its posts) - "Your army / Enemy" says what it does; (2) the Mode row still shows "con" when the mission loaded is `<map>_con_cw.ron`. STATUS question 27 asks the maintainer.
