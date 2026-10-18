//! Tier-1 (data-gated): the water lattice yields exactly ONE cell per `CPAT`
//! patch — the cell whose `WMAP` nibble sets bit 3 — and every other wet cell
//! keeps its quad. SKIPs cleanly when the extracted disc data is absent.
//!
//! `analysis/re-water.md` §4 [V]: the per-frame cull hangs the patch packet on
//! the tile record's `+0x24` only when `z * grid + x` matches the patch's own
//! index (BF1 `FUN_004770a8` 539306-539324, BF2 `FUN_00561b58` 696884-696892),
//! and the flat-water draw takes the patch iff that pointer is set. The bake used to
//! drop every wet cell whose CENTRE fell inside a patch's AABB instead, and
//! each `LPTC INFO` box is its cell padded by half a cell on every side (two
//! cells wide over 32 m cells — `mungefs/tests/water_lvl.rs` pins DEA1's
//! `[-176,-240]..[-112,-176]`), so its edges ran through the eight
//! neighbours' centres and the rule deleted the whole 3x3 block.

use bake::water_glb::triangle_count;
use mungefs::{chunks, open, water::Water, Chunk};

fn lvl(rel: &str) -> Option<Vec<u8>> {
    std::fs::read(format!(
        "{}/../../../extracted/{rel}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .ok()
}

fn tern_of(buf: &[u8]) -> Chunk<'_> {
    let inner = open(buf).unwrap();
    chunks(inner)
        .flatten()
        .find(|c| &c.tag == b"tern")
        .expect("a tern")
}

/// The rule this entry replaced, kept here as the measurement of the defect:
/// a wet cell whose centre lies in any patch's AABB, inclusive.
fn dropped_by_the_old_aabb_rule(w: &Water, x: usize, z: usize) -> bool {
    let o = w.cell_origin(x, z);
    let half = w.info.cell_size / 2.0;
    let (cx, cz) = (o[0] + half, o[1] + half);
    w.patches
        .iter()
        .any(|p| cx >= p.aabb.0[0] && cx <= p.aabb.1[0] && cz >= p.aabb.0[2] && cz <= p.aabb.1[2])
}

/// `(kept by the bit-3 rule, dropped by the bit-3 rule, dropped by the old
/// AABB rule)` over every wet cell.
fn census(w: &Water) -> (usize, usize, usize) {
    let g = w.info.grid as usize;
    let mut kept = 0;
    let mut dropped = 0;
    let mut old = 0;
    for z in 0..g {
        for x in 0..g {
            if !w.is_wet(x, z) {
                continue;
            }
            if w.has_patch(x, z) {
                dropped += 1;
            } else {
                kept += 1;
            }
            if dropped_by_the_old_aabb_rule(w, x, z) {
                old += 1;
            }
        }
    }
    (kept, dropped, old)
}

/// One `CPAT` map and what this entry measured on it.
struct Expect {
    name: &'static str,
    lvl: &'static str,
    wet: usize,
    patches: usize,
    /// Lattice cells the disc's bit-3 rule keeps.
    kept: usize,
    /// Wet cells the old AABB-centre rule removed — the defect, measured.
    old_rule_removed: usize,
}

/// The three maps that ship a `CPAT`. The disc marks 4 / 8 / 1 cells; the
/// AABB rule took 17 / 32 / 2 and left holes beside every patch.
#[test]
fn the_three_patch_maps_yield_one_cell_per_patch() {
    const MAPS: [Expect; 3] = [
        Expect {
            name: "BF1 NAB2",
            lvl: "bf1/lvl/DATA/_LVL_PS2/NAB/NAB2.LVL",
            wet: 38,
            patches: 4,
            kept: 34,
            old_rule_removed: 17,
        },
        Expect {
            name: "BF2 NAB2",
            lvl: "bf2/lvl/DATA/_LVL_PS2/NAB/NAB2.LVL",
            wet: 136,
            patches: 8,
            kept: 128,
            old_rule_removed: 32,
        },
        Expect {
            name: "BF2 DEA1",
            lvl: "bf2/lvl/DATA/_LVL_PS2/DEA/DEA1.LVL",
            wet: 2,
            patches: 1,
            kept: 1,
            old_rule_removed: 2,
        },
    ];
    let mut seen = 0;
    for Expect {
        name,
        lvl: rel,
        wet,
        patches,
        kept,
        old_rule_removed: old,
    } in MAPS
    {
        let Some(buf) = lvl(rel) else {
            eprintln!("SKIP: extracted {name} not present");
            continue;
        };
        seen += 1;
        let w = Water::parse(&tern_of(&buf))
            .unwrap()
            .unwrap_or_else(|| panic!("{name} has WATR"));

        // Bit 3 is only ever a count of the patches that follow it.
        assert_eq!(w.patch_cells(), w.patches.len(), "{name}: bit-3 cells");
        assert_eq!(w.patches.len(), patches, "{name}: LPTC count");
        assert_eq!(w.wet_cells(), wet, "{name}: wet cells");

        let (got_kept, got_dropped, got_old) = census(&w);
        eprintln!(
            "{name}: wet {wet}  patches {patches}  kept {got_kept}  \
             yielded {got_dropped}  (the old AABB rule took {got_old})"
        );
        assert_eq!(got_kept, kept, "{name}: lattice cells kept");
        assert_eq!(got_dropped, patches, "{name}: one cell per patch");
        assert_eq!(got_dropped + got_kept, wet, "{name}: every wet cell once");
        // The defect, measured: the box-centre rule took the 3x3 block.
        assert_eq!(got_old, old, "{name}: the old rule's removals");
        assert!(got_old > got_dropped, "{name}: the old rule over-removed");

        // The pairing itself, on the disc: the i-th `LPTC` belongs to the
        // i-th bit-3 cell in row-major order, and its `INFO` box is exactly
        // that cell padded by HALF A CELL on every side — which is why the
        // box reaches the neighbours' centres and must not pick the cells.
        let g = w.info.grid as usize;
        let bit3: Vec<(usize, usize)> = (0..g)
            .flat_map(|z| (0..g).map(move |x| (x, z)))
            .filter(|&(x, z)| w.has_patch(x, z))
            .collect();
        assert_eq!(bit3.len(), w.patches.len(), "{name}");
        let half = w.info.cell_size / 2.0;
        for (i, (p, &(x, z))) in w.patches.iter().zip(&bit3).enumerate() {
            let o = w.cell_origin(x, z);
            assert_eq!(
                (p.aabb.0[0], p.aabb.0[2], p.aabb.1[0], p.aabb.1[2]),
                (
                    o[0] - half,
                    o[1] - half,
                    o[0] + w.info.cell_size + half,
                    o[1] + w.info.cell_size + half
                ),
                "{name} patch {i}: the LPTC box is its cell padded by half a cell"
            );
            // How much of the cell it replaces the patch actually paints.
            let area: f32 = p
                .triangles
                .iter()
                .map(|t| {
                    let (a, b, c) = (
                        p.positions[t[0] as usize],
                        p.positions[t[1] as usize],
                        p.positions[t[2] as usize],
                    );
                    ((b[0] - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (b[2] - a[2])).abs() / 2.0
                })
                .sum();
            let of_cell = 100.0 * area / (w.info.cell_size * w.info.cell_size);
            eprintln!(
                "  {name} patch {i} at cell ({x},{z}): {} tris, {area:.1} m2 = {of_cell:.1} % of its cell",
                p.triangles.len()
            );
            assert!(
                (13.0..97.0).contains(&of_cell),
                "{name} patch {i}: {of_cell:.1} % of its cell — the header says 13-96 %"
            );
        }

        let own: usize = w.patches.iter().map(|p| p.triangles.len()).sum();
        assert_eq!(
            triangle_count(&w),
            kept * 2 + own,
            "{name}: two triangles per kept cell plus the patches' own"
        );
    }
    if seen == 0 {
        eprintln!("SKIP: no extracted LVL present at all");
    }
}

/// The other nine water maps ship a zero-length `CPAT`: no nibble has bit 3,
/// so every wet cell keeps its quad and the rule change cannot move a byte.
#[test]
fn the_patchless_water_maps_keep_every_wet_cell() {
    const MAPS: [(&str, &str); 9] = [
        ("BF1 KAM1", "bf1/lvl/DATA/_LVL_PS2/KAM/KAM1.LVL"),
        ("BF1 KAS1", "bf1/lvl/DATA/_LVL_PS2/KAS/KAS1.LVL"),
        ("BF1 KAS2", "bf1/lvl/DATA/_LVL_PS2/KAS/KAS2.LVL"),
        ("BF1 YAV1", "bf1/lvl/DATA/_LVL_PS2/YAV/YAV1.LVL"),
        ("BF2 DAG1", "bf2/lvl/DATA/_LVL_PS2/DAG/DAG1.LVL"),
        ("BF2 FEL1", "bf2/lvl/DATA/_LVL_PS2/FEL/FEL1.LVL"),
        ("BF2 KAM1", "bf2/lvl/DATA/_LVL_PS2/KAM/KAM1.LVL"),
        ("BF2 KAS2", "bf2/lvl/DATA/_LVL_PS2/KAS/KAS2.LVL"),
        ("BF2 YAV1", "bf2/lvl/DATA/_LVL_PS2/YAV/YAV1.LVL"),
    ];
    let mut seen = 0;
    for (name, rel) in MAPS {
        let Some(buf) = lvl(rel) else {
            continue;
        };
        seen += 1;
        let w = Water::parse(&tern_of(&buf))
            .unwrap()
            .unwrap_or_else(|| panic!("{name} has WATR"));
        assert_eq!(w.patches.len(), 0, "{name} ships a zero-length CPAT");
        assert_eq!(w.patch_cells(), 0, "{name}: no nibble sets bit 3");
        let (kept, dropped, _) = census(&w);
        assert_eq!(dropped, 0, "{name}: nothing yields");
        assert_eq!(kept, w.wet_cells(), "{name}: every wet cell draws");
        assert_eq!(triangle_count(&w), kept * 2, "{name}");
    }
    if seen == 0 {
        eprintln!("SKIP: no extracted water LVL present");
    }
}
