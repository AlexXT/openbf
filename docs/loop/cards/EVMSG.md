---
id: EVMSG
title: The PS2's on-screen event messages, built: BF2's systemMessages lines (death / hero / post lines under the PS2's gate, per-viewer team colour, fade 0.5 - hold 3.0 - fade 0.5 s, four lines) and its points + kills pop-up, fed from the sim on every map — replaces the placeholder feed
milestone: M3
value: 5
phase: build
status: done
deps: BHUD2
files: engine/crates/ui/src/hud.rs, engine/crates/ui/src/hud_tree.rs, engine/crates/ui/src/hud_draw.rs, engine/crates/game/src/event_feed.rs, engine/crates/game/src/hud_feed.rs, engine/crates/game/src/walk.rs, engine/crates/game/src/main/windowed.rs, engine/crates/game/src/main/headless.rs, engine/crates/game/src/main/battle.rs, engine/crates/mungefs/src/locl.rs, engine/crates/bake/src/hud.rs, engine/crates/bake/src/main.rs, engine/crates/manifest/src/hud.rs, scripts/rebake.sh, engine/CRATES.md
declares: rebake=common; goldens=tat1_hud.png,tat1_spawn.png
needs:
effort: high
branch: evmsg
resume: landed 9d4dade5 2026-10-02
created: 2026-09-30
alpha: yes
---
## Why
The maintainer's INBOX 2026-09-30 11:40: a kill, a death, points, capturing a post or someone else capturing one shows a brief
message on the PS2; the port shows a placeholder box ("X killed Y" for every death, `game/src/main/windowed.rs` ~1163).

## Evidence — `analysis/re-event-messages.md` ([O], verifier 25/4/0 of 29; PS2 frames + states `pcsx2/ref/evmsg/`)
- Neither game has a kill feed: a death makes a line only when the victim is a human or a hero (§1.1). Killing an
  ordinary AI shows NO text — the feedback is the statistics pop-up (BF2: points + kills with "+n", 5 s + 1 s fade).
- BF2's lines live in the layout's `player1systemMessages` MultiLineText: upper-left, 4 lines, `gamefont_tiny` + shadow,
  fade 0.5 / hold 3.0 / fade 0.5 s, a full queue drops the message; blue own / red enemy / grey neutral per viewer; a
  kill line takes the victim's team colour.
- BF1 shows "<TEAM> CAPTURED THE <POST>" / "<TEAM> LOST THE <POST>"; BF2 shows no post line (its Locl still holds both).

## Do (port decisions are in the RE doc §7 — reversible)
- Bake: the English `Locl` table of each CORE.LVL → `common*/locl/english.ron` ([hash → text]; `mungefs::locl`).
- `ui::hud`: MultiLineText (queue, admit, expire by DisplayTime, fade, scroll; EventText + EventColor) and the
  `player1stats` group's events (`statistic.points/.kills`, the Delta texts, `.changed`).
- `game::hud_feed`: sim events → lines: types 0–3 under the PS2 gate; the hero lines; post captured / lost on EVERY map
  (BF2's own unused strings 0xdd279cfc / 0xae1ec790 — the maintainer's ask; the BF2 PS2 shows none, one switch reverses it);
  per-viewer colour from BF2's table. AI names from Locl `common.sides.<side>.firstnames/lastnames` (PROVISIONAL rule).
- Leave the slots: squad orders (SQMK / SQACK, cyan) and awards (AWDM, cyan / yellow) are lines of this element.
- Retire `windowed.rs`'s feed box.

## Acceptance (run and paste)
- Unit test, MultiLineText at 30 Hz: a line queued at t = 0 is fully shown from 0.5 s, starts fading at 3.5 s, is gone
  at 4.0 s, each within ±2 frames (PS2 RAM 0.534 / 2.969 / 0.534); a fifth line while four are up is dropped; when the
  top line goes the others move up one line.
- Unit test, the gate: AI kills AI → no line; AI kills the human → "<name> killed Player 1" in the viewer's own-team
  colour (1,86,213); the human kills an ordinary AI → no line and a `statistic.changed` with kills "+1"; a human
  suicide → "Player 1 killed himself"; a post capture → "<team> captured the <post>" in the capturing team's colour.
- A 640×480 4:3 port render holding the line "Ben Pulastra killed Player 1" against
  `engine/artifacts/pcsx2/ref/evmsg/bf2_hot1_kill_line.png`: the text box within ±4 px of x 49–359, y 90–98.
- `--sim-hash` unchanged; no sweep or arena move (the feed reads events, it makes none).

## Notes
- ANNC RE 2026-09-30 (`analysis/re-announcer.md`): BF1's event consumer `FUN_00307a00`, BF2's twin `FUN_003e0320`; events 4 and 5 also ask for the post's voice line. "Leaving the battlefield" is card BNDY.
- Not this card: the tooltip bar (card TIPS: "This CP is now controlled by your team." is a tooltip), split-screen layouts (`2playerhud` … have their own `playerN.message` elements — with the split-screen card).

## Original card text (planner 2026-09-30, before the RE)
What to find (per game, per event): the text (the localized string tables), colour, icon, position, duration, fade,
stacking, and whose screen shows it. The full event list: kill, death, team-kill / suicide, points, CP captured / lost /
neutralised by either team, pickups, vehicle events, hero events, round start, ticket warnings. PLAN: the BF2 HUD on
every map. Where BF1 has an event BF2 lacks, BF1's text in BF2's style. Leave a slot for award messages and points
pop-ups (AWDM). Acceptance as written then: a PS2 versus port pair per event type from a scripted PCSX2 session that
makes a kill, dies, and captures a CP: the text identical, position within ±4 px at 640×480-equivalent (HUD mapping of
re-hud §2.4), onset/duration within ±2 frames of the PS2's.

- SQMK 2026-10-02: the placeholder box is now `game/src/event_feed.rs` (`EventFeed::frame`, fed in `main/windowed.rs` and the headless `--hud` path) — replace it there. It reads a tick's events once and coalesces an identical line; keep both rules in the systemMessages feed (a render frame that steps no sim tick must not re-admit a line). The order line's count is the maintainer's ask (`order_line`), the PS2 line has none.

## Note from EVTK (2026-10-02, landed 0b99471)
The windowed frame now reads each stepped tick's events in `walk.advance_with(dt, pads, |walk| …)` (`main/windowed.rs`);
`self.feed.frame` is called there per tick. Feed the PS2 message element from that hook, not from the frame body: a
frame steps 0, 1 or several ticks (`game/src/event_sounds.rs` has the count test).
