---
id: PWAI
title: BF2 bots near the player detour to a pickup: every 2 s a bot at AI level of detail 3+ looks for an item within 6 m that gives it health or ammo and walks to it (BF1 bots never do)
milestone: M3
value: 3
phase: build
status: todo
deps: PWUP,BOTD,MOVR
files: engine/crates/core/src/ai/pickup_seek.rs
declares: sim; sweep
needs:
effort:
branch:
resume:
created: 2026-10-02
---
## Why
`analysis/re-powerups.md` §6 (`[O]`, 772d174). BF2 only: `FUN_002bd628` -> `FUN_002be050(6.0)`. A bot whose AI level of
detail is 3 or more (4 within 7 m of a human player, 3 to 20 m, less 1 per failed visibility test), on foot, looks
every 2.0 s (4.0 s more after a hit) at up to 6 items within 6 m; for one whose class gives its health type health it
lacks or ammo it lacks, and that passes a path test, it queues: here, 1 m past the item, here again. Energy and buff
items are not looked for. BF1 has no such code (the only reader of the item type is the supply behaviour).
With the player away no PS2 bot detours (the four battles of re-powerups §8 had the player on the spawn map).

## What to build
A new `core/src/ai/pickup_seek.rs` called from the bot's think (the caller's one line is the builder's to place; keep
`ai/brain.rs` edits to that line - the maintainer's numbered cards list it): the 2 s / 4 s cadence, the 6 m gather over the
sim's live items (card PWUP), the need test (PWUP's give-health / give-ammo dry run), a walkable straight line as the
path test (PROVISIONAL: `FUN_002ef6b8` is unread), the three-point detour. BF2 only. The port's stand-in for the level
of detail: a human player within 20 m.

## Acceptance (run each, paste the output)
- A PS2 sample FIRST (the rate is not measured): `lead_tmp/pwup/live/pw_live.py bf2 <state>` with the player spawned
  and standing in the battle (hold him, god mode: walk_bf2.py's hold), counting items taken within 20 m of the player
  per minute against the same battle's far items (re-powerups §8: 1.1 / 4.9 per minute with nobody near).
- Sim tests: a hurt bot 5 m from a health item with a human 10 m away reaches it within 4 s; the same bot with no
  human within 20 m does not turn; a full-health, full-ammo bot does not; an energy or buff item is ignored; a BF1
  bot never detours. Mutation-check the need test.
- Battle census: detours per minute and items taken near the player, beside the PS2 sample's number.
- Gates: declared `sim; sweep` only.

## Notes
- Created 2026-10-02 by PWUP's RE pass (lane 2). Deps PWUP (the items must exist).
## Planner 2026-10-02 17:45 - waits for BOTD
This card changes the soldier brain (`core/src/ai`) that the bot programme is about to redesign (the maintainer, 2026-10-02: "are we just implementing so many bandaid fixes to a fundamentally flawed bot algorithm"; BOTD: no rewrite before he has read the design). Held so no lane builds on the old brain meanwhile; BOTD adopts, rewrites or closes it when it cuts its build cards. Undo: remove BOTD from `deps:`.

## BOTD (2026-10-03) - adopted; after MOVR
Under D1 (one think rate for every bot, bot-design.md 9) the PS2's "near the player only" detour applies to every bot: say so in FEEL_NOTES. Not in BOTV's deps.
- 2026-10-05 planner - **The maintainer has read the bot design: "BOTD read, go" (INBOX 2026-10-03 14:08). All eight decisions take the note's defaults; the 16 deviations D1-D16 are all on. This card is released (BOTD is done).** Decision 8 (one think rate, D1): the detour applies to every bot, not only those near the player.
