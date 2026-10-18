---
id: PWUP
title: Pickups, built: a dead soldier's drop (its class's table, one roll, the pool cap), the item on the ground (falls, lands, 31 s), the pickup by touch (health to the cap, +clips, either team, only when it gives something), drawn and heard; BF2's energy and offense / defense items
milestone: M3
value: 5
phase: build
status: done
deps:
files: engine/crates/core/src/sim/pickups.rs,engine/crates/core/src/sim/state.rs,engine/crates/core/src/sim/step.rs,engine/crates/core/src/sim/combat.rs,engine/crates/core/src/sim/blast.rs,engine/crates/core/src/unit.rs,engine/crates/game/src/classdb.rs,engine/crates/game/src/roster.rs,engine/crates/game/src/viewer/pickups.rs,engine/crates/game/src/viewer/vehicles.rs,engine/crates/game/src/main/battle.rs,engine/crates/game/src/event_sounds.rs,engine/crates/game/src/fx.rs,engine/CRATES.md
declares: sim; sweep; arena; leak
needs:
effort:
branch: pwup
resume: landed 6bc42904 2026-10-03
created: 2026-09-28
alpha: yes
---

## Why
`analysis/re-powerups.md` (`[O]`, commit 772d174; verifier 60 / 3 / 0 of 63; four live PS2 battles). On the PS2 a dead
soldier drops a health or ammo pickup on 80 % of deaths (BF1 Alliance / Republic / CIS), 25 % (BF1 Empire) or 70 %
(BF2); it lies 31 s and anyone of either team who walks over it and needs it takes it. The port has none of it: no
drop, no item, nothing drawn (M-ALPHA lists pickups).

## What to build (re-powerups §9; each step its own commit)
1. **Data** — per soldier class its drop table resolved the PS2's way (§3.1: parent first; `DropItemClass` /
   `DropItemProbability` write the current index, `NextDropItem` moves it, each class starts at 0 on a copy of its
   parent's table); per item class its numbers (§2.1) and box (§2.3); per map the pool size (§2.4: BF2 25; BF1 from
   the script, 25 by default). A droideka gets no table. The keys are already in `assets_baked/classes/*.ron` and the
   item models in `common_bf1` / `common_bf2` — no rebake.
2. **Sim** (`core/src/sim/pickups.rs`) — the drop at a soldier's death: one draw of the sim's RNG, the cumulative
   walk, nothing when the pool is full (§3.2); spawn at feet + 1.0 m with §3.4's velocity (BF1 the unit's; BF2 the
   unit's above 3 m/s, else 3 m/s to its left). The item: gravity -9.8, lands, life 30 -> -1 s. The pickup (§5.2): any
   living soldier-type unit of either team; landed, or older than 0.5 s; health to the cap, +clips on every weapon
   to its cap; taken only when something was given; reach = the unit's body against the item's box (0.7-1.0 m
   head-on).
3. **Draw and sound** (`game/src/pickup_vis.rs`) — the item's model upright at the item (BF1 hidden for its last 2
   s, BF2 fading over the last second), the pickup sound (`PowerupSound`), the taker's gain effect (`Powerup_blue`
   / `Powerup_red`; BF2 `com_sfx_powerup_*`).
4. **BF2 extras** — energy +150; offense x1.5 for 12 s and defense x0.5 for 20 s (§5.5); a hero takes no health and
   wastes a buff item.
`core/src/sim.rs` needs one `mod` line: it is NOT in `files:` because the maintainer's numbered card BLKB lists it — add the
line, it rebases cleanly. Observers and censuses stay scratch examples (`round_to_outcome.rs` is not listed either).
Ammo goes to EVERY weapon the unit carries: today a `Unit` holds one (`Unit.weapon`, `spare` rounds); if GREN's slot
model has landed give each slot its clips, otherwise the one weapon and a line in `resume:` for the rest.
The picker (dry run 2026-10-02): while this card builds, GREN, JARC and SQACK wait (shared `sim/step.rs` /
`sim/combat.rs` / `weapon.rs`), BLKB does not.

## Acceptance (run each, paste the output into the report)
- **Table census** (a test): 10,000 deaths per table — BF1 A (ammo 0.40 / health100 0.30 / dual 0.10), BF1 B (0.05 /
  0.10 / 0.10), BF2 regular (0.25 / 0.15 / 0.02 / 0.02 / 0.08 / 0.08 / 0.10) — every item's share within 0.015 of
  its table entry, and the class census of §3.1 (BF1 21 / 13 / 9 / 5 classes per table, 8 with none; BF2 121 / 33)
  from the baked classes.
- **Sim tests**: an untouched item is gone 31.0 s after the drop (930 ticks); with the pool full a death drops
  nothing; a full-health unit walks over a health item and leaves it; a hurt ENEMY takes it (BF1 health100 +75,
  health25 +50, dual +100; BF2 +200 / +100 / +200, capped); ammo +1 clip to the cap; an item in flight is not taken
  before 0.5 s; a BF2 victim standing still sends its item 3 m/s to its left. Mutation-check the pickup test (drop
  the "gives something" rule and see it fail).
- **Battle census** (scratch example or `round_to_outcome` observer, sweep seeds): drops / soldier deaths per table
  within 0.03 of the table on BF1 NAB1 (0.80; Gungans 0) and BF2 UTA1 / MYG1 (0.70); live items never above the pool
  size; pickups per minute beside the PS2's (NAB1 1.2, HOT1 0.4, MYG1 1.1, UTA1 4.9 with nobody near the player).
- **Picture**: one port frame with a dropped item of each class on the ground (BF1 NAB1, BF2 UTA1) beside a PCSX2
  frame of an item (`ref/states/bf1_nab1_sideselect.p2s` holds six at known positions: `lead_tmp/pwup/tools/
  ram_items.py`); the pickup sound in the SOUND_LOG referee (the SNDC note below).
- Gates: declared `sim; sweep; arena` only (a battle golden that shows an item is a declared golden — list it).

## Original card (the RE phase, done 2026-10-02)
Why: the maintainer (INBOX 2026-09-28 (d)): health/ammo pickups dropped by dead units (`com_item_powerup_*`-style classes) have no card.
Acceptance: RE ([O]): which classes drop what, the drop chance, lifetime, what each restores, pickup radius, both
games — `analysis/re-powerups.md`. A build card (sim + a draw), declare sim/sweep — this card.

## Notes
- Created 2026-09-28 by the planner. M3 (plays right).
- UAUD1 (2026-09-29, analysis/unit-audit-bf1.md §3.4): every BF1 class authors `DropItemClass` / `DropItemProbability` / `NextDropItem`
  (read by B1 `FUN_0020f2e0`, 0x406d2b75, line 123060). Three profiles: ammo 0.40 / health100 0.30 /
  `com_item_powerup_dual` 0.10 (ALL, REP, CIS rank and file); ammo 0.05 / health25 0.10 / health100 0.10 (IMP, the ALL
  basic marksman, the ALL snow vanguard); ammo 0.20 / health25 0.05 / health100 0.0 (heroes, Wookiee locals); Jawas
  0.5 / 0.4 / 0.1. The pilots' `*_powerup_dispenser` (ch1) places `com_item_powerup_dual`.
- UAUD2 (2026-09-29, analysis/unit-audit-bf2.md §3.4): all 49 fielded BF2 classes inherit `com_inf_default`'s (INGAME.LVL) drop list:
  `com_item_powerup_ammo` 0.25, `_health25` 0.15, `_offense` 0.02, `_defense` 0.02, `_health100` 0.08, `_dual` 0.08,
  `_energy` 0.10 (7 entries, each with NextDropItem "-"). Heroes (`com_jedi_default`) author their own 6. The engineers'
  ch1 `*_powerup_dispenser` places health & ammo.

## Planner note (2026-09-30, M-ALPHA)
`alpha: yes` — pickups are on the alpha list. Value 3 → 4. Showcase maps: BF1 NAB1, BF1 TAT1 (the port's `tat1`), BF1 YAV2, BF2 UTA1, BF2 MYG1, BF2 HOT1 (PLAN §Milestones M-ALPHA).

## Note from GREN (2026-09-30, analysis/re-grenades.md §5.1, §5.2)
The item entity's physics is read there (B1 class `FUN_001f4408`, update `FUN_001f3aa0`, contact `FUN_001f40a0`; B2
`FUN_002263c0` is its setter): `LifeSpan` default FLT_MAX, `Rebound` 0.3, `Friction` 0.5; gravity -9.8 x `Gravity`;
soldiers and vehicles are passed through; a contact slower than 1 m/s along the normal lands it (aligned to the
surface), a faster one bounces. The powerup DISPENSER is the same `dispenser` weapon as the mines' (per-unit `MaxItems`
4, drop velocity (0.3..MaxStrength) x 10 m/s + the owner's) — card MINE builds the weapon; the powerup item's own
pickup rule is this card's.

## Planner note (2026-10-02 09:30)
Value 4 -> 5: pickups are a line of M-ALPHA's checklist and nothing of them is read or built. The loop runs two lanes
since 09:18 (no research lane), so an RE card at value 4 is not reached until every value-5 card is taken; at 5 lane 2
takes it beside CONT. Bound it: the drop (who, chance, when), the item's pickup rule (radius, what it restores, who
may take it), lifetime - the item's physics is already read (the GREN note above). The dispenser weapon stays MINE's.
Reverse: `loopctl.py set PWUP value=4`.

- SNDC census 2026-10-02 (`analysis/audio-census.md`): a pickup plays its class's `PowerupSound` - `com_weap_ammo_pickup` / `com_weap_powerup_pickup` (one shared sample; 3 starts in 105 s on NAB1, 10 on UTA1, 4 m from the eye in the first UTA1 run), `health_recharge` (layered) and `com_item_gonkrecharge` at the droids. BF2 also says a line on a hand-out (`cis1_inf_chatter_GivePowerup`). Add the pickup sound to the build's acceptance (SOUND_LOG referee, `lead_tmp/sndc/port/sound_log.patch`).

## Built and landed (2026-10-03, afb7926 + 6bc4290, docs 951e3e4; report `reports/20261003_0300_card_PWUP.md`)
All four steps: the class tables and item classes (`classdb.rs` `drop_table`, `roster.rs`), the sim
(`core/src/sim/pickups.rs`: drop, item, pickup, BF2 energy and buffs), the draw (`game/src/viewer/pickups.rs`, not
`pickup_vis.rs`: the upload helpers live in the viewer), the sounds and the gain effects. Acceptance run: table census,
11 sim tests (mutation-checked), 8-seed battle census (every table within 0.03), port frames and the SOUND_LOG line on
NAB1 and UTA1. NOT run: the PCSX2 frame beside the port's, and the pickup rate is 2-4x the PS2's on three maps — card
PWPX. Not ported: `analysis/re-powerups.md` 9b's last bullet.
