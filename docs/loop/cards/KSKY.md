---
id: KSKY
title: Kamino's sky is 0.6 as bright on the PS2: Effect("Lightning") darkens dome shell 0 to 255*(1-SkyDomeDarkenFactor) = 153 between flashes (BF1 KAM1, BF1 YAV2, BF2 KAM1) - the port draws every shell at white
milestone: M1
value: 5
phase: build
status: done
deps:
files: engine/crates/mungefs/src/fx.rs,engine/crates/manifest/src/lib.rs,engine/crates/bake/src/map.rs,engine/crates/game/src/viewer/scene.rs,engine/crates/render/src/sky.rs
declares: goldens=any; rebake=maps; parity=bf1_kam1,bf2_kam1,bf1_yav2
needs:
effort:
branch: ksky
resume: landed 97df43ad 2026-10-01
created: 2026-09-30
alpha: yes
---

## Why
The largest single term of Kamino's parity error (bf1_kam1 26.6, bf2_kam1 22.5, the two worst maps): the port's sky is
1.6–2.1 × as bright as the PS2's on both Kaminos. From KAMR's RE, `analysis/re-kamino.md` §1 ([O], verifier 21/1/0).

## Evidence
- Disc: `Effect("Lightning")` in BF1 KAM1, BF1 YAV2 and BF2 KAM1 authors `SkyDomeDarkenFactor(0.4)` (0x034aebfd),
  `SunlightFadeFactor(0.1)`, `FadeTime(0.2)`, `TimeBetweenFlashesMinMax(3,5)` (`lead_tmp/kamr/re/fxkeys.py <LVL>`).
- ELF: the tick (B2 `FUN_0045ae80`, B1 `FUN_00378120`) stores the sky's dome colour = white·(1 − f) + black·f; between
  flashes f = the factor → **(153,153,153,255)**. Only dome shell 0 takes it; the other shells draw at white
  (B2 `FUN_003cb918` 425999–426009, B1 `FUN_002f2308` 280278–280284).
- PS2 RAM: 153 in 7/7 BF2 KAM1, 9/10 BF1 KAM1 (one mid-flash, 255) and 4/4 BF1 YAV2 states (`re/ramsky.py`).
- Prototype (`lead_tmp/kamr/tools/dome_prototype.patch`, shell 0 × 0.6): parity bf1_kam1 26.62 → 21.80, bf2_kam1
  22.49 → 19.20; sky band PS2/port bf2 0.76 → 0.94, bf1 0.47 → 0.64.

## Do
1. `mungefs::fx`: parse `Effect("Lightning")` (all keys — WTHR needs the rest) and `LightningBolt`; manifest field
   (`rebake=maps`; skipped when the map authors none, so the other 29 manifests do not move).
2. `viewer::scene::upload_sky`: shell 0's colour × (1 − SkyDomeDarkenFactor) when the effect is enabled; the other shells
   unchanged. The flashes (dome and fog colour to white, lights) are WTHR's — leave a hook, not an animation.
3. `sky.ambient` has no PS2 reader (hash 0x1c9640eb is in neither ELF; white on all 32 maps): keep or drop, say which.

## Acceptance
- Pixel test at the stored walk poses: the top-left sky band (x 0..140, y 0..28, the views `lead_tmp/kamr/skyratio.py`
  accepts) PS2/port median within 0.85..1.15 on bf2_kam1 (main: 0.76); bf1_kam1 reported (main 0.47; prototype 0.64 —
  the rest is the open item in re-kamino §8, card it if it stays under 0.85).
- `parity.py` against the landing's parent: bf2_kam1 down by ≥ 2.5 and bf1_kam1 by ≥ 4.0 (prototype −3.29 / −4.82 on
  0994d9a; main 5c73167 reads 20.80 / 26.62 after BPNT); bf1_yav2 reported; no other map moves.
- A unit test pins 255·(1 − 0.4) = 153 and that shell 1 (`kam1_city`) stays white.
- Goldens: only the two Kamino map sheets and BF1 YAV2's move (viewed).

## Notes
- Four reference states were saved mid-flash (dome 224 / 245 / 193 / 255): `bf2_kam1_cw_sideselect`,
  `bf1_kam1_cw_cis_spawnmap`, `bf1_kam1_cw_sideselect`, `bf1_kam1_walk_cp1_cp10_b000` — frames from them are not the
  resting look.
