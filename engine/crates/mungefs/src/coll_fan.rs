//! The SIGNED FAN repair of a `coll` mesh's leaves (card STARC).
//!
//! A `LEAF` is a convex planar polygon and the port triangulates it as a fan
//! — on all but 73 of the two discs' 274,308 leaves (measured,
//! `artifacts/lead_tmp/starc/tools/collcensus.py`). The exporter wrote a face
//! WITH A HOLE, or a concave one, as a fan round one apex whose triangles
//! carry a SIGN: a triangle wound the other way takes its area back out, and
//! adjacent fan triangles were then merged into one non-convex "polygon".
//! BF2 POL1's `pol1_bldg_radar_main01` is the case that found it: the wall
//! behind the radar room's door is five leaves — `(75,76,48)`, `(77,78,48)`,
//! `(76,77,48)`, `(48,78,75,47,58,56)`, `(75,48,47)` — whose directed edges,
//! with every opposite pair cancelled, leave the wall's outline (577.6 m²)
//! and the doorway's (36.8 m², wound the other way). A two-sided fan of the
//! same leaves covers the doorway twice: the port's bots stood in the pocket
//! behind the door (32 recycles over 9 seeds), where the PS2's soldiers walk
//! through (23 crossings of the wall's plane in 420 s, every one inside the
//! doorway — `artifacts/lead_tmp/starc/README.md`).
//!
//! So a group of coplanar leaves round a NON-CONVEX leaf whose fans overlap
//! (the area they cover is more than the area they sum to) is replaced by the
//! region they sum to: the loops their uncancelled edges form, the holes
//! bridged into their outlines, ear-clipped. A group this cannot read — edges
//! that do not close into simple loops, a result whose area is not the signed
//! sum — keeps its fans.
//!
//! OURS, PROVISIONAL: what the PS2's soldier narrowphase does with such a
//! leaf is not read. Its RAY test (BF2 `FUN_00165070`) is one-sided and asks
//! a hit to lie inside every edge of the stored polygon — an unverified
//! reading, not in `analysis/` (card COLN).

use std::collections::HashMap;

/// Vertices this far off a leaf's plane are not in it (metres).
const PLANE_TOL: f64 = 0.02;
/// A vertex this far outside an edge makes the polygon non-convex (metres).
const CONVEX_TOL: f64 = 0.02;

type P3 = [f64; 3];
type P2 = [f64; 2];

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: P3, b: P3) -> P3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn unit(a: P3) -> Option<P3> {
    let l = dot(a, a).sqrt();
    (l > 1e-9).then(|| [a[0] / l, a[1] / l, a[2] / l])
}
fn cross2(a: P2, b: P2, c: P2) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn fan(poly: &[u16]) -> impl Iterator<Item = [u16; 3]> + '_ {
    (1..poly.len().saturating_sub(1)).map(move |i| [poly[0], poly[i], poly[i + 1]])
}

/// A leaf's plane: the unit normal of its first fan triangle with an area,
/// and a point on it.
fn plane(v: &[P3], poly: &[u16]) -> Option<(P3, P3)> {
    fan(poly).find_map(|[a, b, c]| {
        let (a, b, c) = (v[a as usize], v[b as usize], v[c as usize]);
        unit(cross(sub(b, a), sub(c, a))).map(|n| (n, a))
    })
}

/// Is every vertex of `poly` on the inner side of every one of its edges,
/// for one winding of the whole polygon?
fn convex(v: &[P3], poly: &[u16], n: P3) -> bool {
    let k = poly.len();
    let one = |s: f64| {
        (0..k).all(|i| {
            let (a, b) = (v[poly[i] as usize], v[poly[(i + 1) % k] as usize]);
            unit(sub(b, a)).is_none_or(|e| {
                poly.iter()
                    .all(|&p| s * dot(cross(e, sub(v[p as usize], a)), n) >= -CONVEX_TOL)
            })
        })
    };
    one(1.0) || one(-1.0)
}

/// The repaired groups of `polygons`: each is the leaves it replaces
/// (ascending) and the triangles that replace their fans.
pub(crate) fn repairs(
    vertices: &[[f32; 3]],
    polygons: &[Vec<u16>],
) -> Vec<(Vec<usize>, Vec<[u32; 3]>)> {
    let v: Vec<P3> = vertices
        .iter()
        .map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
        .collect();
    let mut taken = vec![false; polygons.len()];
    let mut out = Vec::new();
    for (i, poly) in polygons.iter().enumerate() {
        if taken[i] || poly.len() < 4 {
            continue;
        }
        let Some((n, o)) = plane(&v, poly) else {
            continue;
        };
        if convex(&v, poly, n) {
            continue;
        }
        let in_plane = |p: &[u16]| {
            p.len() >= 3
                && p.iter()
                    .all(|&k| dot(sub(v[k as usize], o), n).abs() < PLANE_TOL)
        };
        if !in_plane(poly) {
            continue;
        }
        // The coplanar leaves joined to this one through shared vertices.
        // One walk over a vertex -> leaves table: a chain of leaves costs
        // its length, not its length squared per pass.
        let mut at_vertex: HashMap<u16, Vec<usize>> = HashMap::new();
        for (j, q) in polygons.iter().enumerate() {
            if !taken[j] && in_plane(q) {
                for &k in q {
                    at_vertex.entry(k).or_default().push(j);
                }
            }
        }
        let mut member = vec![false; polygons.len()];
        member[i] = true;
        let mut group = vec![i];
        let mut next = 0;
        while next < group.len() {
            let g = group[next];
            next += 1;
            for k in &polygons[g] {
                for j in at_vertex.remove(k).unwrap_or_default() {
                    if !member[j] {
                        member[j] = true;
                        group.push(j);
                    }
                }
            }
        }
        group.sort_unstable();
        if let Some(tris) = region(&v, polygons, &group, n) {
            for &g in &group {
                taken[g] = true;
            }
            out.push((group, tris));
        }
    }
    out
}

/// The region a group's signed fans sum to, triangulated — `None` when the
/// fans already are that region, or when this cannot read it.
fn region(v: &[P3], polygons: &[Vec<u16>], group: &[usize], n: P3) -> Option<Vec<[u32; 3]>> {
    let area = |[a, b, c]: [u16; 3]| {
        dot(
            cross(
                sub(v[b as usize], v[a as usize]),
                sub(v[c as usize], v[a as usize]),
            ),
            n,
        ) / 2.0
    };
    let (mut cover, mut signed) = (0.0, 0.0);
    let mut edges: HashMap<(u16, u16), i32> = HashMap::new();
    for &g in group {
        for t in fan(&polygons[g]) {
            let a = area(t);
            cover += a.abs();
            signed += a;
            for e in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                *edges.entry(e).or_default() += 1;
            }
        }
    }
    if cover - signed.abs() <= 1e-3 * cover.max(1.0) {
        return None; // the fans do not overlap: they are the region
    }
    // Every directed edge less its opposite; what is left bounds the region.
    let mut next: HashMap<u16, u16> = HashMap::new();
    let mut keys: Vec<_> = edges.keys().copied().collect();
    keys.sort_unstable();
    for (a, b) in keys {
        let k = edges[&(a, b)] - edges.get(&(b, a)).copied().unwrap_or(0);
        if k > 1 || (k == 1 && next.insert(a, b).is_some()) {
            return None;
        }
    }
    // The plane's 2-D frame, turned so the region's outlines run anticlockwise.
    let s = if signed < 0.0 { -1.0 } else { 1.0 };
    let n = [n[0] * s, n[1] * s, n[2] * s];
    let seed = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = unit(cross(seed, n))?;
    let w = cross(n, u);
    let at = |i: u16| -> P2 { [dot(v[i as usize], u), dot(v[i as usize], w)] };
    let mut starts: Vec<u16> = next.keys().copied().collect();
    starts.sort_unstable();
    let (mut outer, mut holes): (Vec<Vec<u16>>, Vec<Vec<u16>>) = (Vec::new(), Vec::new());
    for s0 in starts {
        let Some(mut c) = next.remove(&s0) else {
            continue;
        };
        let mut lp = vec![s0];
        while c != s0 {
            lp.push(c);
            c = next.remove(&c)?;
        }
        if lp.len() < 3 {
            return None;
        }
        let a: f64 = (1..lp.len() - 1)
            .map(|i| cross2(at(lp[0]), at(lp[i]), at(lp[i + 1])) / 2.0)
            .sum();
        if a > 0.0 {
            outer.push(lp);
        } else {
            holes.push(lp);
        }
    }
    // Each hole into the outline that holds it, the rightmost hole first.
    let max_u = |h: &Vec<u16>| h.iter().map(|&i| at(i)[0]).fold(f64::MIN, f64::max);
    holes.sort_by(|a, b| max_u(b).total_cmp(&max_u(a)));
    for h in holes {
        let o = outer.iter_mut().find(|o| inside(&at, o, at(h[0])))?;
        *o = bridge(&at, o, &h)?;
    }
    let mut tris = Vec::new();
    for o in outer {
        ear_clip(&at, o, &mut tris)?;
    }
    let got: f64 = tris
        .iter()
        .map(|t| cross2(at(t[0] as u16), at(t[1] as u16), at(t[2] as u16)) / 2.0)
        .sum();
    if (got - signed.abs()).abs() > 1e-3 * cover.max(1.0) {
        return None;
    }
    // The fans' own facing: the port's tests are two-sided, the glb keeps it.
    if s < 0.0 {
        for t in &mut tris {
            t.swap(1, 2);
        }
    }
    Some(tris)
}

/// Is `p` inside the loop (even-odd)?
fn inside(at: &impl Fn(u16) -> P2, lp: &[u16], p: P2) -> bool {
    let mut c = false;
    for i in 0..lp.len() {
        let (a, b) = (at(lp[i]), at(lp[(i + 1) % lp.len()]));
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < a[0] + (p[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0])
        {
            c = !c;
        }
    }
    c
}

/// Do the open segments `a-b` and `c-d` cross?
fn crosses(a: P2, b: P2, c: P2, d: P2) -> bool {
    let (d1, d2) = (cross2(a, b, c), cross2(a, b, d));
    let (d3, d4) = (cross2(c, d, a), cross2(c, d, b));
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

/// `outer` with `hole` cut in: out along a bridge from the outline's vertex
/// nearest the hole's rightmost that sees it, round the hole, and back.
fn bridge(at: &impl Fn(u16) -> P2, outer: &[u16], hole: &[u16]) -> Option<Vec<u16>> {
    let hi = (0..hole.len()).max_by(|&a, &b| at(hole[a])[0].total_cmp(&at(hole[b])[0]))?;
    let h = at(hole[hi]);
    let mut order: Vec<usize> = (0..outer.len()).collect();
    let d2 = |i: usize| {
        let p = at(outer[i]);
        (p[0] - h[0]).powi(2) + (p[1] - h[1]).powi(2)
    };
    order.sort_by(|&a, &b| d2(a).total_cmp(&d2(b)));
    let clear = |p: P2| {
        [outer, hole]
            .iter()
            .all(|lp| (0..lp.len()).all(|i| !crosses(h, p, at(lp[i]), at(lp[(i + 1) % lp.len()]))))
    };
    let oi = order.into_iter().find(|&i| clear(at(outer[i])))?;
    let mut out = Vec::with_capacity(outer.len() + hole.len() + 2);
    out.extend_from_slice(&outer[..=oi]);
    out.extend((0..=hole.len()).map(|k| hole[(hi + k) % hole.len()]));
    out.extend_from_slice(&outer[oi..]);
    Some(out)
}

/// Ear-clip the anticlockwise polygon `lp` (a bridged hole repeats two of its
/// vertices) into `tris`.
fn ear_clip(at: &impl Fn(u16) -> P2, mut lp: Vec<u16>, tris: &mut Vec<[u32; 3]>) -> Option<()> {
    while lp.len() > 3 {
        let k = lp.len();
        let ear = (0..k).find(|&i| {
            let (a, b, c) = (lp[(i + k - 1) % k], lp[i], lp[(i + 1) % k]);
            let (pa, pb, pc) = (at(a), at(b), at(c));
            if cross2(pa, pb, pc) <= 1e-9 {
                return false;
            }
            lp.iter().all(|&q| {
                let p = at(q);
                q == a
                    || q == b
                    || q == c
                    || !(cross2(pa, pb, p) >= 0.0
                        && cross2(pb, pc, p) >= 0.0
                        && cross2(pc, pa, p) >= 0.0)
            })
        });
        // No ear: a vertex with no area (a bridge's spur) goes first.
        let i = ear.or_else(|| {
            (0..k).find(|&i| {
                cross2(at(lp[(i + k - 1) % k]), at(lp[i]), at(lp[(i + 1) % k])).abs() <= 1e-9
            })
        })?;
        if ear.is_some() {
            tris.push([
                u32::from(lp[(i + k - 1) % k]),
                u32::from(lp[i]),
                u32::from(lp[(i + 1) % k]),
            ]);
        }
        lp.remove(i);
    }
    if cross2(at(lp[0]), at(lp[1]), at(lp[2])) > 1e-9 {
        tris.push([u32::from(lp[0]), u32::from(lp[1]), u32::from(lp[2])]);
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BF2 POL1 `pol1_bldg_radar_main01`'s doorway wall, from the disc: the
    /// eight vertices and the five leaves on its plane (indices renumbered
    /// 47, 48, 56, 58, 75..78 -> 0..7).
    fn pol1_wall() -> (Vec<[f32; 3]>, Vec<Vec<u16>>) {
        let v = vec![
            [-29.28, -2.99, -82.35],  // 47
            [-39.78, -2.99, -114.68], // 48
            [-39.78, 14.0, -114.68],  // 56
            [-29.28, 14.0, -82.35],   // 58
            [-36.51, 1.86, -104.6],   // 75
            [-36.51, 8.72, -104.6],   // 76
            [-34.85, 8.72, -99.49],   // 77
            [-34.85, 1.86, -99.49],   // 78
        ];
        let leaves = vec![
            vec![4, 5, 1],
            vec![6, 7, 1],
            vec![5, 6, 1],
            vec![1, 7, 4, 0, 3, 2],
            vec![4, 1, 0],
        ];
        (v, leaves)
    }

    /// How many of `tris` hold the point `s` metres along the wall from
    /// vertex 47 at height `y`.
    fn holds(v: &[[f32; 3]], tris: &[[u32; 3]], s: f64, y: f64) -> usize {
        let (a, b) = (v[0], v[1]);
        let len = f64::from((b[0] - a[0]).hypot(b[2] - a[2]));
        let p2 = |p: [f32; 3]| -> P2 {
            let (dx, dz) = (f64::from(p[0] - a[0]), f64::from(p[2] - a[2]));
            [
                (dx * f64::from(b[0] - a[0]) + dz * f64::from(b[2] - a[2])) / len,
                f64::from(p[1]),
            ]
        };
        tris.iter()
            .filter(|t| {
                let (a, b, c) = (
                    p2(v[t[0] as usize]),
                    p2(v[t[1] as usize]),
                    p2(v[t[2] as usize]),
                );
                let d = [
                    cross2(a, b, [s, y]),
                    cross2(b, c, [s, y]),
                    cross2(c, a, [s, y]),
                ];
                d.iter().all(|&x| x >= 0.0) || d.iter().all(|&x| x <= 0.0)
            })
            .count()
    }

    #[test]
    fn the_radar_rooms_wall_keeps_its_doorway() {
        let (v, leaves) = pol1_wall();
        // The plain fans cover the doorway (twice) — the defect.
        let fans: Vec<[u32; 3]> = leaves
            .iter()
            .flat_map(|l| fan(l).map(|t| t.map(u32::from)))
            .collect();
        assert_eq!(holds(&v, &fans, 20.7, 3.0), 2, "the fans fill the doorway");
        let r = repairs(&v, &leaves);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, vec![0, 1, 2, 3, 4]);
        let tris = &r[0].1;
        // A rectangle with a rectangular hole: 8 triangles.
        assert_eq!(tris.len(), 8);
        // The doorway (18.0..23.4 m along, y 1.86..8.72) is open ...
        for (s, y) in [(20.7, 3.0), (18.3, 2.0), (23.1, 2.0), (20.7, 8.5)] {
            assert_eq!(holds(&v, tris, s, y), 0, "doorway at ({s}, {y})");
        }
        // ... and the wall round it is there once.
        for (s, y) in [
            (10.0, 3.0),
            (28.0, 3.0),
            (20.7, 11.0),
            (20.7, 0.0),
            (1.0, -2.0),
            (33.0, 13.0),
        ] {
            assert_eq!(holds(&v, tris, s, y), 1, "wall at ({s}, {y})");
        }
        // Its area is the wall less the doorway: 34.0 x 17.0 - 5.37 x 6.86.
        let area: f64 = tris
            .iter()
            .map(|t| {
                let p = |i: u32| v[i as usize].map(f64::from);
                let c = cross(sub(p(t[1]), p(t[0])), sub(p(t[2]), p(t[0])));
                dot(c, c).sqrt() / 2.0
            })
            .sum();
        assert!((area - 540.8).abs() < 0.5, "{area}");
    }

    #[test]
    fn convex_leaves_and_a_fan_that_is_its_region_are_left_alone() {
        // A quad and a triangle; an L-shaped hexagon fanned from its inner
        // corner (every fan triangle wound one way: the fan is the region).
        let v = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [5.0, 0.0, 5.0], // the L's inner corner
            [5.0, 0.0, 3.0],
            [8.0, 0.0, 3.0],
            [8.0, 0.0, 8.0],
            [3.0, 0.0, 8.0],
            [3.0, 0.0, 5.0],
        ];
        let leaves = vec![vec![0u16, 1, 2, 3], vec![0, 1, 2], vec![4, 5, 6, 7, 8, 9]];
        assert!(repairs(&v, &leaves).is_empty());
    }

    #[test]
    fn a_concave_leaf_whose_fan_overdraws_is_clipped_to_its_outline() {
        // The same L, fanned from an OUTER corner: the fan covers the notch.
        let v = vec![
            [5.0, 0.0, 3.0],
            [8.0, 0.0, 3.0],
            [8.0, 0.0, 8.0],
            [3.0, 0.0, 8.0],
            [3.0, 0.0, 5.0],
            [5.0, 0.0, 5.0],
        ];
        let leaves = vec![vec![3u16, 4, 5, 0, 1, 2]];
        let r = repairs(&v, &leaves);
        assert_eq!(r.len(), 1);
        let area: f64 = r[0]
            .1
            .iter()
            .map(|t| {
                let p = |i: u32| v[i as usize].map(f64::from);
                let c = cross(sub(p(t[1]), p(t[0])), sub(p(t[2]), p(t[0])));
                dot(c, c).sqrt() / 2.0
            })
            .sum();
        // 5 x 5 less the 2 x 2 notch.
        assert!((area - 21.0).abs() < 1e-6, "{area}");
        assert_eq!(r[0].1.len(), 4);
    }
}
