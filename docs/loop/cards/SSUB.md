---
id: SSUB
title: Sound properties by the battle's sub-level: a planet's SOUND LVL defines one property once per sub-level (nab1cw, nab1gcw, nab2cw, nab2gcw) and the port keeps the file's LAST audible one for every map and era
milestone: M1
value: 3
phase: build
status: done
deps:
files: engine/crates/bake/src/sounds.rs,engine/crates/manifest/src/lib.rs,engine/crates/game/src/sounds.rs
declares: rebake=sounds
needs:
effort:
branch:
resume: CLOSED 2026-10-06 by the planner: built by card SNDE (e45d75a) - see the note at the end of the card
created: 2026-10-03
---
## Why
VSND's replay referee (2026-10-03, analysis/re-vehicle-sounds.md 10b): NAB's fighter tank engine record has other
graphs in nab2gcw than in nab1cw, and the file's last definition gave 19,107 of 19,419 layer-ticks against 19,374
with the battle's own sub-level. The engine records are now picked by sub-level (`SoundLib::engine`,
`game::foley::sublevels`); the sound PROPERTIES are not: `game::sounds::merge_props` keeps the last audible
definition of a name (its own comment: 873 of TAT1's 2210 records share a hash, 10 differ; PROVISIONAL).

## Do
1. Measure first: per baked map, the names whose definitions differ between sub-levels (gain, pitch, distances,
   samples, layers) and which of them the port's events / foley / engines ever start. If none that plays differs
   on the six showcase maps, close the card with the table.
2. Else: bake each property's sub-level (`mungefs::sound_props::find_named_sublevels` already names them; a
   `level` field on `manifest::SoundProp`, written only where a name has differing definitions, keeps the files
   small), and `SoundLib` picks by `foley::sublevels(map dir, factions)` like `engine`.

## Acceptance
- The census table (names with differing definitions per map; how many the port can start).
- A test on BF1 NAB1: for every name with differing definitions, the property the library returns for a CW battle
  equals the nab1cw sub-level's own (read from the disc), and for a GCW battle nab1gcw's.
- `SOUND_LOG` on bf1_nab1 (the foley pad replay): the starts' gains before / after, the rows that changed listed.

## Notes
- 2026-10-06 planner: **CLOSED as built by card SNDE** (e45d75a, `reports/20261006_1350_card_SNDE.md`). SNDE did this card's step 2 for another reason (the commander's era): `bake sounds` keeps each record's sub-level (`SoundProp::level`), `SoundLib` resolves a name in the battle's own sub-levels first (`foley::sublevels`), foley tables and engine records by the same list, and `merge_props`'s PROVISIONAL note now covers only two definitions INSIDE one sub-level. Its checks stand in for this card's test: `a_battle_plays_its_missions_own_sub_level` reads every name of the mission's sub-level from the disc LVL, 57 missions, 53,907 names, 0 of other audio (the SNDE report's numbers; I read `game/tests/sound_sublevels.rs` and `game/src/sounds.rs`, I did not run the test). **Not run by anyone: this card's census of names whose GAIN / PITCH / DISTANCES differ between sub-levels and the `SOUND_LOG` before / after on bf1_nab1.** The whole record is the sub-level's own now, so the rule is in; if a wrong gain is ever heard, `cargo run -p game --example audit_takes -- <mission>` is the tool. Undo: `loopctl.py set SSUB status=todo`.
