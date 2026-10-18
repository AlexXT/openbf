//! The far scene's LOWD extension (card UTAS, `analysis/re-far-lowd.md` §6 `[O]`).
//!
//! The PS2 draws every geometry that ships a `LOWD` model in its FAR scene
//! with that LOWD (re-far-lowd §0, §2.2-§2.5): a LOWD gives the gmod's LOD
//! record flags 3, flag 1 queues the object for the far scene whenever its
//! sphere reaches past `NearSceneRange[1]`, and flag 2 takes the far walker's
//! fade branch, which draws LOD index 3 — the LOWD slot. The port draws LOD0
//! everywhere, which is the modern choice, and loses only what a LOWD has
//! that its LOD0 does not. On the 32 shipped maps that is one model: BF2
//! UTA1's `uta1_bldg_cliffs`, whose LOD0 is the sinkhole's near half-cylinder
//! and whose LOWD is the whole ring — the far wall.
//!
//! [`extension`] cuts that difference out of the LOWD: the triangles whose
//! centroid lies more than [`OFF_LOD0`] from the LOD0 surface (model space;
//! both LODs share it), kept only when their area passes
//! [`MIN_EXTENSION_AREA`]. The runtime draws it per instance past
//! `NearSceneRange[1]` (`game::viewer::scene`).

use glam::Vec3;
use mungefs::model::Model;

/// A LOWD triangle is LOD0's own when its centroid lies within this many
/// metres of the LOD0 surface. PROVISIONAL and OURS: the RE's split
/// (`lead_tmp/re/uta1-sinkhole/extension_split.py`) at 2 / 5 / 10 / 20 m
/// keeps 371 / 357 / 351 / 333 of the cliffs' LOWD triangles.
pub const OFF_LOD0: f32 = 2.0;

/// The least extension area (m²) worth a model. PROVISIONAL and OURS: the
/// census of every LOWD on the 32 bakes under this rule
/// (`lead_tmp/utas/census2.txt`) gives UTA1's cliffs 1,036,060 m² and the next
/// geometry 32,423 m² (BF2 KAM1 `kam_bldg_mushplatform1`, a coarse LOWD
/// cutting LOD0's corners, not missing geometry). Everything below this is
/// the LOWD simplifying LOD0, which the port's LOD0 already draws better.
pub const MIN_EXTENSION_AREA: f32 = 100_000.0;

/// Below this area a triangle is a strip restart, not a surface.
const DEGENERATE: f32 = 1e-4;

/// The LOWD model's triangles that LOD0 lacks, as a model named
/// `<lowd>_far` with the LOWD's segments, materials and vertices (only the
/// triangle lists shrink) — or `None` when there is not enough of it.
pub fn extension(lod0: &Model, lowd: &Model) -> Option<Model> {
    if lod0.bone_space || lowd.bone_space {
        return None;
    }
    // Cheap reject first: the extension is never larger than the LOWD.
    let total: f32 = lowd
        .segments
        .iter()
        .flat_map(|s| s.triangles.iter().map(|t| tri(&s.positions, t)))
        .map(area)
        .sum();
    if total < MIN_EXTENSION_AREA {
        return None;
    }
    let near: Vec<[Vec3; 3]> = lod0
        .segments
        .iter()
        .flat_map(|s| s.triangles.iter().map(|t| tri(&s.positions, t)))
        .filter(|t| area(*t) >= DEGENERATE)
        .collect();
    if near.is_empty() {
        return None;
    }
    let mut out = lowd.clone();
    out.name = format!("{}_far", lowd.name);
    let mut kept = 0.0;
    for seg in &mut out.segments {
        let pos = &seg.positions;
        seg.triangles.retain(|t| {
            let v = tri(pos, t);
            let a = area(v);
            if a < DEGENERATE {
                return false;
            }
            let c = (v[0] + v[1] + v[2]) / 3.0;
            let off = near.iter().all(|n| dist2(c, n) > OFF_LOD0 * OFF_LOD0);
            if off {
                kept += a;
            }
            off
        });
    }
    out.face_count = out.segments.iter().map(|s| s.triangles.len() as u32).sum();
    (kept >= MIN_EXTENSION_AREA).then_some(out)
}

fn tri(p: &[[f32; 3]], t: &[u32; 3]) -> [Vec3; 3] {
    t.map(|i| Vec3::from(p[i as usize]))
}

fn area(t: [Vec3; 3]) -> f32 {
    0.5 * (t[1] - t[0]).cross(t[2] - t[0]).length()
}

/// Squared distance from `p` to the triangle `t` (Ericson, *Real-Time
/// Collision Detection* §5.1.5, closest point by Voronoi region).
fn dist2(p: Vec3, t: &[Vec3; 3]) -> f32 {
    let [a, b, c] = *t;
    let (ab, ac, ap) = (b - a, c - a, p - a);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return ap.length_squared();
    }
    let bp = p - b;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));
    if d3 >= 0.0 && d4 <= d3 {
        return bp.length_squared();
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return (p - (a + ab * (d1 / (d1 - d3)))).length_squared();
    }
    let cp = p - c;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));
    if d6 >= 0.0 && d5 <= d6 {
        return cp.length_squared();
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return (p - (a + ac * (d2 / (d2 - d6)))).length_squared();
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (p - (b + (c - b) * w)).length_squared();
    }
    let denom = 1.0 / (va + vb + vc);
    let (v, w) = (vb * denom, vc * denom);
    (p - (a + ab * v + ac * w)).length_squared()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_to_a_triangle_by_region() {
        let t = [Vec3::ZERO, Vec3::X * 10.0, Vec3::Z * 10.0];
        // Over the face, past a vertex, past an edge.
        assert!((dist2(Vec3::new(2.0, 3.0, 2.0), &t) - 9.0).abs() < 1e-4);
        assert!((dist2(Vec3::new(-3.0, 0.0, -4.0), &t) - 25.0).abs() < 1e-4);
        assert!((dist2(Vec3::new(5.0, 0.0, -2.0), &t) - 4.0).abs() < 1e-4);
        assert!((dist2(Vec3::new(6.0, 0.0, 6.0), &t) - 2.0).abs() < 1e-4);
    }
}
