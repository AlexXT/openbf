//! The flank table on small graphs: a diamond (one authored side, the other
//! not), a far other side, and an other way that reuses too much of the
//! authored one.

use super::*;
use crate::nav::{Arc, Hub, RouteTable, TYPE_COUNT};
use glam::Vec3;

/// A soldier graph: hubs at `(x, z)`, arcs between hub pairs, and the
/// authored table's one alternative (weight 31) per `(hub, dest, arc)`.
pub(crate) fn graph(
    hubs: &[(f32, f32)],
    arcs: &[(u16, u16)],
    table: &[(u16, u16, u16)],
) -> NavGraph {
    let n = hubs.len();
    let mut hs: Vec<Hub> = hubs
        .iter()
        .map(|&(x, z)| Hub {
            pos: Vec3::new(x, 0.0, z),
            radius: 2.0,
            arcs: Vec::new(),
        })
        .collect();
    for (i, &(a, b)) in arcs.iter().enumerate() {
        hs[usize::from(a)].arcs.push(i as u16);
        hs[usize::from(b)].arcs.push(i as u16);
    }
    let mut rows = vec![Vec::new(); TYPE_COUNT * n];
    for row in rows.iter_mut().take(n) {
        *row = vec![0u8; n];
    }
    for &(h, d, arc) in table {
        let slot = hs[usize::from(h)]
            .arcs
            .iter()
            .position(|&a| a == arc)
            .expect("slot");
        rows[usize::from(h)][usize::from(d)] = 31 << 3 | slot as u8;
    }
    let arcs = arcs
        .iter()
        .map(|&(a, b)| Arc {
            a,
            b,
            type_mask: 1,
            flags: 0,
        })
        .collect();
    NavGraph::new(
        hs,
        arcs,
        RouteTable::from_rows(n, TYPE_COUNT, rows),
        Vec::new(),
        Vec::new(),
    )
    .expect("graph")
}

/// Posts on hubs 0 and 3; the authored way both ways over hub 1 at
/// (20, 5), the other side over hub 2 at (20, `alt_z`).
pub(crate) fn diamond(alt_z: f32) -> NavGraph {
    graph(
        &[(0.0, 0.0), (20.0, 5.0), (20.0, alt_z), (40.0, 0.0)],
        &[(0, 1), (1, 3), (0, 2), (2, 3)],
        &[(0, 3, 0), (1, 3, 1), (3, 0, 1), (1, 0, 0)],
    )
}

#[test]
fn the_diamond_stores_the_other_side_both_ways() {
    let g = diamond(-8.0);
    let t = g.flank_table(&[3, 0, 3], 5);
    assert_eq!(t.pairs, 2, "0 -> 3 and 3 -> 0 have a main route");
    assert_eq!(t.routes.len(), 2);
    let k = t.find(0, 3).expect("0 -> 3");
    let r = &t.routes[k];
    assert_eq!(r.hubs, vec![0, 2, 3]);
    assert_eq!(r.arcs, vec![2, 3]);
    let side = |z: f32| 2.0 * (400.0f32 + z * z).sqrt();
    assert!((r.main_len - side(5.0)).abs() < 1e-3, "{}", r.main_len);
    assert!((r.len - side(8.0)).abs() < 1e-3, "{}", r.len);
    assert!((SHARE_MIN..=SHARE_MAX).contains(&r.share), "{}", r.share);
    let back = &t.routes[t.find(3, 0).expect("3 -> 0")];
    assert_eq!(back.hubs, vec![3, 2, 0]);
    // The walk: the next hop after each hub on it, none off it or at its end.
    assert_eq!(t.next(k, 0), Some((2, 2)));
    assert_eq!(t.next(k, 2), Some((3, 3)));
    assert_eq!(t.next(k, 3), None);
    assert_eq!(t.next(k, 1), None);
}

#[test]
fn an_other_side_more_than_35_percent_longer_is_not_stored() {
    // 2 x sqrt(20^2 + 30^2) = 72.1 m against 41.2: x 1.75.
    let t = diamond(-30.0).flank_table(&[0, 3], 5);
    assert_eq!((t.pairs, t.routes.len()), (2, 0));
    // 2 x sqrt(20^2 + 17.5^2) = 53.2 m: x 1.29 is.
    let t = diamond(-17.5).flank_table(&[0, 3], 5);
    assert_eq!(t.routes.len(), 2);
}

#[test]
fn a_way_mostly_over_the_authored_arcs_is_not_stored() {
    // Main 0-1-2-3-4 (a bend at hub 3); the cheapest other way, 0-1-2-4,
    // keeps two of its three arcs on the authored set.
    let g = graph(
        &[
            (0.0, 0.0),
            (20.0, 0.0),
            (40.0, 0.0),
            (60.0, 3.0),
            (60.0, 0.0),
        ],
        &[(0, 1), (1, 2), (2, 3), (3, 4), (2, 4)],
        &[(0, 4, 0), (1, 4, 1), (2, 4, 2), (3, 4, 3)],
    );
    let t = g.flank_table(&[0, 4], 5);
    assert_eq!(t.pairs, 1, "4 -> 0 has no authored row");
    assert!(t.routes.is_empty(), "{:?}", t.routes);
}

#[test]
fn the_share_is_drawn_per_pair_and_per_match_inside_its_bounds() {
    let mut seen = Vec::new();
    for seed in 0..200u32 {
        for (a, b) in [(0u16, 3u16), (3, 0), (7, 72)] {
            let s = share_draw(seed, a, b);
            assert!((SHARE_MIN..SHARE_MAX).contains(&s), "{s}");
            seen.push(s);
        }
    }
    let mean = seen.iter().sum::<f32>() / seen.len() as f32;
    assert!((mean - 0.20).abs() < 0.01, "mean {mean}");
    assert_ne!(share_draw(1, 0, 3), share_draw(1, 3, 0));
    assert_ne!(share_draw(1, 0, 3), share_draw(2, 0, 3));
    assert_eq!(share_draw(9, 7, 72), share_draw(9, 7, 72));
}
