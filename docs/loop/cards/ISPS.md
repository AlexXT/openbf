---
id: ISPS
title: The world fragment shader at a laptop's cost: specialise it per light path, convert the uniform colours once on the CPU, and keep the far-cull discard only on draws that can be culled - priced on the iGPU at -5.3 ms on KAM1 and -4.4 ms on GEO1 at 2880x1920, same picture
milestone: M0
value: 5
phase: build
status: todo
deps:
files: engine/crates/render/src/shader.wgsl, engine/crates/render/src/pipelines.rs, engine/crates/render/src/frame.rs, engine/crates/render/src/lib.rs
declares: goldens=none expected (CPU pow at the quantisation step if any: declare + view)
needs:
effort:
branch:
resume:
created: 2026-10-20
alpha: yes
---
## Why
Card ISPD's re-take on main 8729cae (engine/artifacts/ispd/README.md, "Update 2026-10-20"): after ISPF the biggest GPU cost left on the Intel iGPU is the world fragment shader itself. A flat-colour fs takes KAM1's models 11.85 -> 2.26 ms and GEO1's terrain 5.94 -> 0.82 ms at 2880x1920. Mesa INTEL_DEBUG=fs: the world fs is ~770-980 instructions (SIMD16, no spills, ~60 log/exp) - every LightPath's case, the sRGB pows of uniform colours and the far-cull discard compiled into every pixel.
Priced variants (GPU ms sum, 2880x1920, quiet machine, control repeats to 0.01 ms; rows in README, shaders engine/artifacts/ispd/x/*.wgsl, runs win/*_{y,z,w,v}_*):
- KAM1 14.01 -> nocull 12.05, hoist 13.23, nocull+hoist 10.89, lean (nocull + hoist + only the lit/terrain paths) 8.74.
- GEO1 13.73 -> 11.18 / 12.98 / 10.39 / 9.36 (terrain 5.94 -> 3.20; the later terrain light pass 3.04 -> 2.22 without the discard: early depth restored).
- Textures, the detail map and the fog table (var<private> FOGSPREAD_DIAGONAL) cost nothing measurable.

## The change (same picture, less work per pixel)
1. The light path specialised: one fragment entry (or a WGSL `override` constant per pipeline) per `LightPath` group, so a pixel runs only its own path's code; draws already sort by layer/mesh, the run picks the pipeline.
2. Uniform colours converted ONCE: `linear_to_srgb(globals.sun_color / light2_color / ambient_ground / ambient_sky / character_ambient / fog_color)` become precomputed uniforms (CPU writes the GS-unit values; the CPU twin `render::lighting::light_sum` already has the formula).
3. The far-cull `discard` only where it can fire: a pipeline without `far_cull` for a run whose instances carry no cull band, no near edge and no fade (cull = inf, cull_near = 0, fade = 1 - most draws while OPENBF_FAR_CULL is off); the discard pipeline for the rest (LOD bands, foliage fades).
Not in scope: per-vertex lighting (the PS2's VU1 way; it would move goldens - a later card if the table still asks for it).

## Acceptance
- The ISPD table rows (artifacts/ispd/win1.sh + table.py, quiet machine) for bf2_kam1, bf2_geo1, bf1_nab1, tat2 at 2880x1920 before/after on the same binary pair: GPU sum down by at least 3 ms on KAM1 and GEO1.
- No golden moves, or only ones at the quantisation step from the CPU-side pow (declared, viewed, max channel delta <= 1/255 stated); sim hash untouched.
- Mesa INTEL_DEBUG=fs (MESA_SHADER_CACHE_DISABLE=true) instruction count of the lit opaque fs pasted before/after.