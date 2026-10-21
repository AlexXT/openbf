---
id: BSHL
title: The BF2 shell and menus' real look (PLAN §Presentation): today's shell (goldens/shell.png) is a working placeholder — BF2's shell art, flow and fonts, content adapted to the port's features
milestone: M5
value: 5
phase: re
status: done
deps:
files:
declares:
needs:
effort:
branch:
resume: done 2026-10-20: analysis/re-bf2-shell.md (767318c); split into build cards BSHLA (title + stacks + bake, value 5), BSHLB (IA screen, value 4), LDBF2 (BF2 loading screen, value 4)
created: 2026-09-28
defer: 
first: 6
---

## Why
(2026-10-19, the maintainer, first: 6, no longer post-alpha: the community's top ask) The PS2 BF2 shell, captured from the PS2 DISC and PCSX2 - the PS2 front-end menus differ from the PC game's, so PC-derived layouts are leads only. The in-game unit-select screen (rotating trooper line-up) is probably close to the same on both and is SSEL's. Menu art, fonts, screen order, layout constants: read the disc's own compiled shell scripts (BF2's SHELL.LVL / SHELLPS2.LVL) first, then capture each screen in PCSX2 and match it pixel-measured (RULES: visual claims are pixel-measured).

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
- planner 2026-10-19 16:25 (the maintainer, INBOX 10-19 (a)): the PS2 BF2 shell differs from the PC one - capture it from the PS2 disc and PCSX2 (SHELL.LVL's own compiled shell scripts hold the screens' order and layout), never from PC-derived layouts. Loading screens already exist (`bake/src/loadscreen.rs`, `ui/src/loading.rs`); check their tips / order against the PS2 in the same pass.

## RE done (2026-10-20, lane 1, second session)
`analysis/re-bf2-shell.md` (767318c): flow, the one stack style (palette, pads, measured geometry), the disabled
button (grey 110, focus skips it - the port's Multiplayer takes it, so the 2026-09-30 note's choice is "focus skips
it"), the SHELL.MVS movie, the 512x448 screen, the IA screen, the BF2 loading screen (tips 1..54). Build cards:
BSHLA (value 5), BSHLB (value 4, deps BSHLA), LDBF2 (value 4). ERAM's two notes moved to BSHLB. The loading screen's
tips / order check (planner 10-19) is LDBF2's.
