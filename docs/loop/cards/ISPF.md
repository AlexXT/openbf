---
id: ISPF
title: The DomeInfo frame filter at a laptop's cost: 33.6 of 46 ms a frame at 2880x1920 on the Intel iGPU (every BF1 map, BF2 TAT2) - make its n x n box separable and read display bytes without pow, same picture
milestone: M0
value: 5
phase: build
status: done
deps:
files: engine/crates/render/src/filter.rs, engine/crates/render/src/filter.wgsl
declares: goldens=maps/bf1_bes1.png,maps/bf1_bes2.png,maps/bf1_end1.png,maps/bf1_geo1.png,maps/bf1_kam1.png,maps/bf1_kas1.png,maps/bf1_kas2.png,maps/bf1_nab1.png,maps/bf1_nab2.png,maps/bf1_rhn1.png,maps/bf1_rhn2.png,maps/bf1_yav1.png,maps/bf1_yav2.png,maps/tat1.png,maps/tat2.png,tat1_hud.png,tat1_overview.png,tat1_spawn.png
needs:
effort:
branch: ispf
resume: landed e0bb520b 2026-10-20
created: 2026-10-20
alpha: yes
---
## Why
Card ISPD's table (engine/artifacts/ispd/README.md, 2026-10-20): on this machine's Intel iGPU (OPENBF_ADAPTER=igpu) the frame filter costs 5.4 ms at 1920x1080 and 33.6 ms at 2880x1920 - BF1 NAB1 / END1 and TAT2 run 21-22 fps at the Surface Pro 8's resolution, 69 fps with OPENBF_NO_FILTER=1. Every BF2 map without it holds 72-98 fps there. The cost grows with the square of the resolution: fs_filter averages an n x n box per pixel, n = box_side(height) = height/224 (5 at 1080, 9 at 1920; MAX_BOX 12), and encodes every texel to display bytes with f_linear_to_srgb (3 pow each): 81 loads + 243 pow per pixel at 1920 lines.

## Evidence
engine/artifacts/ispd/win/bf1_nab1_2880x1920_{idle,nofilter}/frames.csv (OPENBF_GPU_TIMING=1 sections: filter 33.72 ms; knocked out the GPU sum drops 47.5 -> 13.8 ms, every other section within 0.05). filter.rs header: the port's box is centred on each pixel at native resolution (analysis/re-glow.md §3 [O] is the PS2 rule: box2 to 256x224, threshold, blur, add I/128).

## Plan
1. Read the frame copy through a NON-sRGB view (create the copy texture with view_formats [format.remove_srgb_suffix()]): textureLoad then returns the display bytes the shader now rebuilds with pow - the same values to float rounding.
2. Make the box separable: pass 1 averages n texels along x into an Rgba16Float (display units) target, pass 2 averages n along y and applies threshold + gain over the frame, as fs_filter does now. 2n loads instead of n^2.
3. Keep box_side and the threshold rule untouched - the picture is the PS2's rule as today.

## Acceptance
- The ISPD table's filter section on bf1_nab1 at 2880x1920 (artifacts/ispd/win1.sh bf1_nab1 2880x1920 90 <tag> --cheat god, then table.py): from 33.7 ms to <= 6 ms, and at 1920x1080 from 5.4 to <= 1.5 ms; paste both rows.
- The BF1 / TAT goldens and parity: unchanged or within rounding (declare and VIEW the ones that move; a move above 1 LSB at a threshold edge is investigated, not accepted).
- The unit test the_box_is_one_ps2_half_pixel still holds; add one that the two-pass box equals the n x n box on a fixture (a GPU referee in render/tests).
- planner 2026-10-20 05:20: `alpha: yes` added so the slice that unblocks ISPD (the maintainer's `first: 2`) goes first at value 5;
  BSHLA and the M3 fives otherwise sorted ahead of it. Undo: `loopctl.py set ISPF alpha=`.

## Result (build 2026-10-20, branch ispf 39272db + 3e9c732)
- Built: a row pass (n texels along x, through a non-sRGB view of the copy = display bytes, no pow) into an
  Rgb10a2Unorm target, then the column pass with threshold + gain; both passes take two texels per linear sample
  (half-way between texel centres, clamp-to-edge = the per-texel clamp). box_side / threshold rule untouched.
- iGPU, bf1_nab1, `win1.sh ... --cheat god` (dirs `artifacts/ispd/win/bf1_nab1_*_{main3fe5b,ispfpair,ispfpair2}`):
  2880x1920 filter **33.56 -> 4.46 ms** (GPU 47.56 -> 18.44 ms, 20.6 -> 52.1 fps); 1920x1080 **5.35 -> 1.25 ms**
  (85.3 -> 130.8 fps). Other sections equal within 0.02 ms in the paired clean runs.
  Steps: separable + no pow alone 6.32 / 1.62 ms; f16 rows 8.76 (slowed GPU run); knock-outs at 2880: copy 0.43 ms,
  two passes at n=1 2.73 ms.
- Referee `the_two_pass_box_is_the_square_box` (render/tests): worst 0.55-0.67 levels off the n x n model at
  720/1080/1920 lines on the dGPU and the iGPU; mutations (column pass without its box; pair sampled at a texel centre)
  fail it by 19-61 levels.
- Goldens: 18 filter goldens move by max 1 level (QUANT.md), tat1_spawn by 2 at ONE pixel: before the overlay the
  branch frame is within 1 of main everywhere and nearer the PS2 model (max err 0.64 vs main's 0.82); at (756,262)
  the model is 120.48 - main 121, branch 120 - and the spawn overlay turns that tie into 31/29
  (`lead_tmp/ispf/spawnchk/`).
