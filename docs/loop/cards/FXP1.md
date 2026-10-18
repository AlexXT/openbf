---
id: FXP1
title: Particle effects, part 1: mungefs parses fx__ ParticleEmitter trees (every key of analysis/re-fx.md §2, both games' layouts, §3's rules) and the bake carries each LVL layer's effects (COMMON/INGAME, sides, maps)
milestone: M1
value: 5
phase: build
status: done
deps:
files: engine/crates/mungefs/src/particle.rs,engine/crates/mungefs/src/fx.rs,engine/crates/mungefs/src/lib.rs,engine/crates/mungefs/Cargo.toml,engine/crates/mungefs/tests/fx_lvl.rs,engine/crates/mungefs/tests/data/fx_ram_bf1_nab1.json,engine/crates/bake/src/effects.rs,engine/crates/bake/src/effect_names.txt,engine/crates/bake/src/lib.rs,engine/crates/bake/src/main.rs,engine/crates/bake/src/map.rs,engine/crates/manifest/src/lib.rs,engine/crates/manifest/src/tests.rs,scripts/rebake.sh,engine/CRATES.md
declares: rebake=common,side,maps,tat1,tat2
needs:
branch: fxp1
resume: landed 657d8bd8 2026-09-30
created: 2026-09-30
alpha: yes
---

## Why
The real particle effects (the maintainer, INBOX 2026-09-28 (d)) — impacts, explosions, trails, vehicle damage — are `fx__`
`ParticleEmitter` trees the port does not read. `analysis/re-fx.md` (card FXP's RE, [O]) cracked the whole format and
checked the PS2's in-memory layout field by field against RAM in both games. This card is the data half: parse + bake.

## Evidence
`analysis/re-fx.md` §1–§4 (key table §2 with every hash, B1/B2 offsets and defaults; §3 parse rules; §4 RAM checks
`lead_tmp/fxp/ramcheck2.py` / `ramcheck_bf2.py`). Census `lead_tmp/fxp/census2.py`: BF1 128 effects / 701 emitters,
BF2 292 / 1702. Where they live: BF1 COMMON.LVL (109) + maps; BF2 INGAME.LVL (162) + sides + maps (§1).

## Acceptance
- `mungefs` parses a `fx__` chunk into `Effect { name_hash, emitters: Vec<Emitter> }` with every §2 key: ranges as
  `{base, span}` (random = span ≠ 0), colour keys ×1/255, the RGB/HSV mode (the Spawner's is the one used), nested
  `ParticleEmitter` children (sibling instances) and Geometry `EMITTER` children, Transformer chains (Position
  Accelerate/Reach/Scale, Size·Rotation Move/Reach/Scale, Color Move/Reach) with Next, Geometry Type/Blend/Texture/
  Model/SparkLength/TimePerFrame/FrameSize/TotalFrames/Looping/Blur*; defaults from §2's ctor columns; the game tag
  (BF1: FrameSize unused, fixed 4×4; BF2: FrameSize in texels); §3's continuous-emitter rule. The BF2 split-screen
  halving is NOT applied (a PS2 budget; modern port) — record it in FEEL_NOTES-style comment.
- The NAME hash is fnv20 (FNV-1a over `byte | 0x20`, `_` → 0x7f); lookups by name use the same hash.
- Data test (skips without the disc): §1's counts (128/701, 292/1702, types and blends), `ord_explosion`'s two emitters
  as §1 lists them, and a sample of §4's RAM values (dump 20 templates from `ramcheck2.py`'s match into a JSON vector).
- The bake writes the effects per LVL layer (`common_bf1`, `common_bf2` from INGAME, each `side_*` side, each map incl.
  tat1/tat2) into the manifest (or an `effects.ron` beside it), keyed by hash with the cracked name where known
  (`lead_tmp/fxp/AGENT_C_fxnames.txt`), and makes sure every texture/model an effect names is baked.
- Gates: only the declared rebake groups DIFF (the manifest schema + the effects); no golden, sweep or sim move.

## Notes
- The environment `Effect("…")` records in the same chunks stay unparsed here (FXC / WTHR / water).
- FXP2 (runtime + draw) and FXP3 (spawn hooks) follow; FXMF / FXB are the muzzle flash and the bolt.
