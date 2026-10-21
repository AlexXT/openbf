//! Card FLNK (`analysis/bot-design.md` 6.2, deviation D5) — the FLANK TABLE:
//! per ordered pair of post hubs, the best OTHER way over the soldier arcs,
//! stored at load. OURS: the PS2's bots walk the authored route table only,
//! and every hub they never visit lies off every authored post-to-post route
//! (`analysis/re-soldier-ai.md` 10b); toward a post that table offers a
//! second next hop at 0-2 % of its entries on ten maps (bot design M5), so a
//! tilt of its weights has nothing to tilt.
//!
//! For each ordered pair (S, T) of post hubs:
//! - the AUTHORED set: every arc of every alternative of weight > 0 the
//!   route table offers from S toward T, followed hub by hub;
//! - the MAIN route: from S, the heaviest alternative at each hub (not back
//!   the way it came; the higher hub on equal weights), until T - no main
//!   route (a dead end, a loop) = no flank route for the pair;
//! - the flank route: the shortest way from S to T over the soldier arcs
//!   (XZ between hub centres) with the authored set's arcs costing
//!   [`AUTHORED_COST`] times their length, kept when at most
//!   [`AUTHORED_SHARE`] of its arcs are authored ones and it is at most
//!   [`LONGER`] times the main route's length.
//!
//! The reference is `lead_tmp/botd/tools/flank.py` (bot design M5): the same
//! search over the disc's plan, whose per-map counts this table's equal.
//! A share of each pair's attackers takes the route ([`FlankRoute::share`],
//! drawn per match; the team board gives the role, `ai::team::board`). An
//! attack starts from the post nearest the hub it was planned from
//! ([`FlankTable::origin`]), and a flanker joins its route at the route's
//! nearest hub ([`FlankTable::join`]).
//! Derived from the graph and the posts, never hashed.

use super::{xz_dist, Mover, NavGraph, SOLDIER};

/// A flank route is at most this many times the main route's length.
pub const LONGER: f32 = 1.35;
/// ...and at most this share of its arcs are on the pair's authored set.
pub const AUTHORED_SHARE: f32 = 0.5;
/// The search's cost factor on the pair's authored arcs.
pub const AUTHORED_COST: f32 = 3.0;
/// The share of a pair's attackers that flanks, drawn per match between
/// these (the maintainer's decision 5: 20 % on average, between 10 and 30).
pub const SHARE_MIN: f32 = 0.10;
pub const SHARE_MAX: f32 = 0.30;
/// An attack planned from a hub within this many arcs of a post's hub
/// starts from that post ([`FlankTable::origin`]): the bots spawn on the
/// posts' spawn nodes, which locate on the hubs round the post's own
/// (Mygeeto: attacks planned from 30 hubs, five of them posts).
pub const ORIGIN_HOPS: u16 = 2;
/// A flanker off its route's hubs walks to the nearest of them within this
/// many arcs ([`FlankTable::join`]); farther, the authored route.
pub const JOIN_HOPS: u16 = 3;

/// One stored flank route.
#[derive(Debug, Clone, PartialEq)]
pub struct FlankRoute {
    /// The post hubs it joins.
    pub from: u16,
    pub to: u16,
    /// Its hubs, `from` first and `to` last, and the arc between each two.
    pub hubs: Vec<u16>,
    pub arcs: Vec<u16>,
    /// Its length and the main route's (m, XZ between hub centres).
    pub len: f32,
    pub main_len: f32,
    /// This match's share of the pair's attackers that takes it.
    pub share: f32,
}

/// The flank routes of one world's posts ([`NavGraph::flank_table`]).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlankTable {
    /// Ordered post pairs with an authored main route.
    pub pairs: u32,
    pub routes: Vec<FlankRoute>,
    /// Per post hub (ascending), the arcs from it to every hub over the
    /// soldier arcs ([`NavGraph::hop_distances_into`]; `u16::MAX` none).
    pub post_hops: Vec<(u16, Vec<u16>)>,
}

impl FlankTable {
    /// The flank route from post hub `from` to post hub `to`.
    pub fn find(&self, from: u16, to: u16) -> Option<usize> {
        self.routes
            .iter()
            .position(|r| r.from == from && r.to == to)
    }

    /// The hop after hub `cur` on route `k`: (arc, next hub); `None` when
    /// `cur` is not on it or is its end.
    pub fn next(&self, k: usize, cur: u16) -> Option<(u16, u16)> {
        let r = self.routes.get(k)?;
        let i = r.hubs.iter().position(|&h| h == cur)?;
        Some((*r.arcs.get(i)?, *r.hubs.get(i + 1)?))
    }

    /// The post an attack planned from hub `h` starts from: the post hub
    /// fewest arcs from `h`, within [`ORIGIN_HOPS`] (the lower hub on a
    /// tie).
    pub fn origin(&self, h: u16) -> Option<u16> {
        self.post_hops
            .iter()
            .filter_map(|(p, d)| Some((*d.get(usize::from(h))?, *p)))
            .filter(|&(d, _)| d <= ORIGIN_HOPS)
            .min()
            .map(|(_, p)| p)
    }

    /// Where a flanker standing at hub `cur`, off route `k`'s hubs, joins
    /// it: the hub of the route but its end fewest arcs from `cur` by
    /// `hops` (the field from `cur`), within [`JOIN_HOPS`] (the later hub
    /// on a tie).
    pub fn join(&self, k: usize, hops: &[u16]) -> Option<u16> {
        let r = self.routes.get(k)?;
        let n = r.hubs.len().saturating_sub(1);
        r.hubs[..n]
            .iter()
            .enumerate()
            .filter_map(|(i, &h)| Some((*hops.get(usize::from(h))?, std::cmp::Reverse(i), h)))
            .filter(|&(d, _, _)| d <= JOIN_HOPS)
            .min()
            .map(|(_, _, h)| h)
    }
}

/// The share drawn for the pair `from` -> `to` from the match's `seed`: a
/// hash of the three (splitmix64), so no draw of the sim's own streams
/// moves and each pair of each map draws its own.
pub fn share_draw(seed: u32, from: u16, to: u16) -> f32 {
    let pair = u64::from(from) << 16 | u64::from(to);
    let mut x = u64::from(seed) << 32 ^ pair.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    let u = (x >> 40) as f32 / (1u64 << 24) as f32;
    SHARE_MIN + (SHARE_MAX - SHARE_MIN) * u
}

impl NavGraph {
    fn hub_gap(&self, a: u16, b: u16) -> f32 {
        match (self.hub(a), self.hub(b)) {
            (Some(x), Some(y)) => xz_dist(x.pos, y.pos),
            _ => 0.0,
        }
    }

    /// The soldier route table's AUTHORED set from `s` toward `dst` (one
    /// flag per arc) and its MAIN route (module header); `None` = no main
    /// route.
    pub fn authored_route(&self, s: u16, dst: u16) -> (Vec<bool>, Option<Vec<u16>>) {
        let n = self.hubs.len();
        let mut used = vec![false; self.arcs.len()];
        // The alternatives of weight > 0 at `h` toward `dst`: (weight, arc).
        let alts = |h: u16| -> Vec<(u8, u16)> {
            let Some(hub) = self.hub(h) else {
                return Vec::new();
            };
            self.routes
                .entries(SOLDIER, h, dst)
                .filter(|e| e >> 3 > 0)
                .filter_map(|e| {
                    let a = *hub.arcs.get(usize::from(e & 7))?;
                    self.arcs
                        .get(usize::from(a))
                        .filter(|arc| arc.touches(h))
                        .map(|_| (e >> 3, a))
                })
                .collect()
        };
        let mut seen = vec![false; n];
        let mut stack = vec![s];
        while let Some(h) = stack.pop() {
            if seen[usize::from(h)] {
                continue;
            }
            seen[usize::from(h)] = true;
            if h == dst {
                continue;
            }
            for (_, a) in alts(h) {
                used[usize::from(a)] = true;
                stack.push(self.arcs[usize::from(a)].other(h));
            }
        }
        let mut main = vec![s];
        let (mut h, mut prev) = (s, None);
        while h != dst && main.len() <= 400 {
            let best = alts(h)
                .into_iter()
                .map(|(w, a)| (w, self.arcs[usize::from(a)].other(h)))
                .filter(|&(_, o)| Some(o) != prev)
                .max();
            let Some((_, o)) = best else {
                return (used, None);
            };
            if main.contains(&o) {
                return (used, None);
            }
            (prev, h) = (Some(h), o);
            main.push(o);
        }
        (used, (h == dst).then_some(main))
    }

    /// The shortest way from `s` to `dst` over the arcs a soldier may walk,
    /// the arcs flagged in `pen` costing [`AUTHORED_COST`] times their
    /// length: (hubs, arcs). Ties go to the lower hub.
    fn flank_search(&self, s: u16, dst: u16, pen: &[bool]) -> Option<(Vec<u16>, Vec<u16>)> {
        use std::cmp::Ordering;
        use std::collections::BinaryHeap;
        #[derive(PartialEq)]
        struct Open(f32, u16);
        impl Eq for Open {}
        impl Ord for Open {
            fn cmp(&self, o: &Self) -> Ordering {
                o.0.total_cmp(&self.0).then(o.1.cmp(&self.1))
            }
        }
        impl PartialOrd for Open {
            fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
                Some(self.cmp(o))
            }
        }
        let n = self.hubs.len();
        let mut best = vec![f32::INFINITY; n];
        let mut back: Vec<Option<(u16, u16)>> = vec![None; n];
        let mut open = BinaryHeap::new();
        best[usize::from(s)] = 0.0;
        open.push(Open(0.0, s));
        while let Some(Open(c, h)) = open.pop() {
            if h == dst {
                break;
            }
            if c > best[usize::from(h)] {
                continue;
            }
            for &ai in &self.adj[usize::from(h)] {
                let arc = self.arcs[usize::from(ai)];
                if !arc.allows(Mover::SOLDIER) {
                    continue;
                }
                let o = arc.other(h);
                let k = if pen[usize::from(ai)] {
                    AUTHORED_COST
                } else {
                    1.0
                };
                let c2 = c + self.hub_gap(h, o) * k;
                if c2 < best[usize::from(o)] {
                    best[usize::from(o)] = c2;
                    back[usize::from(o)] = Some((h, ai));
                    open.push(Open(c2, o));
                }
            }
        }
        back[usize::from(dst)]?;
        let (mut hubs, mut arcs, mut h) = (vec![dst], Vec::new(), dst);
        while h != s {
            let (p, a) = back[usize::from(h)]?;
            hubs.push(p);
            arcs.push(a);
            h = p;
        }
        hubs.reverse();
        arcs.reverse();
        Some((hubs, arcs))
    }

    /// Card FLNK: the flank table for the posts on hubs `posts` (module
    /// header), each route's share drawn from `seed`.
    pub fn flank_table(&self, posts: &[u16], seed: u32) -> FlankTable {
        let mut hubs: Vec<u16> = posts
            .iter()
            .copied()
            .filter(|&h| usize::from(h) < self.hubs.len())
            .collect();
        hubs.sort_unstable();
        hubs.dedup();
        let mut t = FlankTable::default();
        let mut queue = Vec::new();
        for &p in &hubs {
            let mut d = Vec::new();
            self.hop_distances_into(p, Mover::SOLDIER, &mut d, &mut queue);
            t.post_hops.push((p, d));
        }
        for &s in &hubs {
            for &d in &hubs {
                if s == d {
                    continue;
                }
                let (used, Some(main)) = self.authored_route(s, d) else {
                    continue;
                };
                t.pairs += 1;
                let main_len: f32 = main.windows(2).map(|w| self.hub_gap(w[0], w[1])).sum();
                let Some((path, arcs)) = self.flank_search(s, d, &used) else {
                    continue;
                };
                let len: f32 = path.windows(2).map(|w| self.hub_gap(w[0], w[1])).sum();
                let authored = arcs.iter().filter(|&&a| used[usize::from(a)]).count();
                if main_len <= 0.0
                    || authored as f32 > AUTHORED_SHARE * arcs.len().max(1) as f32
                    || len > LONGER * main_len
                {
                    continue;
                }
                let share = share_draw(seed, s, d);
                t.routes.push(FlankRoute {
                    from: s,
                    to: d,
                    hubs: path,
                    arcs,
                    len,
                    main_len,
                    share,
                });
            }
        }
        t
    }
}

#[cfg(test)]
pub(crate) mod tests;
