//! Card TPOS — the POSITION TABLE (`analysis/bot-design.md` 4.1): every place
//! on a map a soldier might stand to fight, with what can be known about its
//! cover without looking again. Built once when the posts are in
//! ([`Sim::build_positions`]), a pure function of the map — its graph, its
//! hints, its barriers, its floors and bodies — so DERIVED and not hashed.
//!
//! Sources (OURS; the census that sized them is card BOTD's,
//! `lead_tmp/botd/tools/cover_census.patch`, `analysis/bot-design.md` M6):
//! the authored Cover and Snipe hints (the PS2's own, re-soldier-ai 3), the
//! points [`EDGE_OUT`] outside each soldier barrier's four edge middles and
//! four corners (the PS2's bots stand within a metre of a barrier a quarter of
//! their time, M3), and each hub's centre with 8 points at [`HUB_RING`] of its
//! radius. A point is kept when a soldier can stand there ([`Builder::stand`]):
//! a floor on its storey or the open terrain, dry, on the level, in no soldier
//! barrier, head room.
//!
//! Per position: the floor point, its source, the hint's facing and stances,
//! the nearest hub on its storey, and two masks over [`DIRS`] directions
//! (bit `k` = the direction `(cos a, 0, sin a)`, `a = k / 16` of a turn):
//! LOW cover (something shut at crouch-eye height within [`NEAR`], the lane
//! open at stand-eye height for [`LANE`]: crouch behind it, stand to fire) and
//! CORNER cover (shut at both heights, a [`LEAN`] step to a side has the
//! lane: lean to fire). The rays are the bolt's (terrain + the map's own
//! bodies, no vehicle hull), as perception's.

use crate::fmath;
use crate::nav;
use crate::sim::{Level, Sim};
use glam::{Vec2, Vec3};

/// The sector masks' directions.
pub const DIRS: usize = 16;
/// LOW / CORNER cover: something shut within this of the position (m).
pub const NEAR: f32 = 3.0;
/// ...and a lane this long open to fire along (m).
pub const LANE: f32 = 30.0;
/// The eye heights over the floor: crouched and standing (m).
pub const CROUCH_EYE: f32 = 0.9;
pub const STAND_EYE: f32 = 1.6;
/// A corner's lean step (m).
pub const LEAN: f32 = 1.0;
/// A barrier's edge points lie this far outside it (m).
pub const EDGE_OUT: f32 = 0.9;
/// A hub's ring points lie at this share of its radius.
pub const HUB_RING: f32 = 0.7;
/// A point is in a barrier when within this of it (m): the census's test.
const IN_BARRIER: f32 = 0.3;
/// A floor this far from the storey asked for is another storey (m).
const STOREY: f32 = 1.5;
/// Head room over the floor (m), asked from 0.3 m up.
const HEAD_ROOM: f32 = 1.5;
/// A barrier's edge point takes its storey from the nearest hub within this.
const EDGE_HUB: f32 = 40.0;
/// The grid cell of the table's index (m).
const CELL: f32 = 8.0;

/// Where a position came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Source {
    /// An authored Cover hint (kind 2).
    Cover = 0,
    /// An authored Snipe hint (kind 0).
    Snipe = 1,
    /// A hub's centre or one of its ring points.
    Hub = 2,
    /// A point outside a soldier barrier's edge or corner.
    Edge = 3,
}

impl Source {
    pub const ALL: [Source; 4] = [Source::Cover, Source::Snipe, Source::Hub, Source::Edge];
    pub fn name(self) -> &'static str {
        match self {
            Source::Cover => "cover hints",
            Source::Snipe => "snipe hints",
            Source::Hub => "hub points",
            Source::Edge => "barrier edges",
        }
    }
}

/// One standable place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// The floor point a soldier stands on.
    pub pos: Vec3,
    pub source: Source,
    /// A hint's authored facing (its `rows[2]`, XZ, unit); zero for the rest.
    pub facing: Vec2,
    /// A hint's primary and secondary stance masks (0 = none authored).
    pub stance: u8,
    pub stance2: u8,
    /// The nearest hub on this storey ([`u16::MAX`] = none).
    pub hub: u16,
    /// LOW cover per direction (module header).
    pub low: u16,
    /// CORNER cover per direction.
    pub corner: u16,
}

impl Position {
    /// Cover of either kind toward the direction `d` (XZ).
    pub fn covered_toward(&self, d: Vec2) -> bool {
        (self.low | self.corner) & sector_bit(d) != 0
    }
    /// Any cover at all.
    pub fn covered(&self) -> bool {
        self.low | self.corner != 0
    }
}

/// The sector bit of direction `d` (XZ); 0 for a zero vector.
pub fn sector_bit(d: Vec2) -> u16 {
    if d.length_squared() < 1e-8 {
        return 0;
    }
    let a = fmath::atan2(d.y, d.x).rem_euclid(std::f32::consts::TAU);
    let k = (a / std::f32::consts::TAU * DIRS as f32).round() as usize % DIRS;
    1 << k
}

/// What the build did, per source: points tried, standable, covered.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TableStats {
    pub tried: [u32; 4],
    pub standable: [u32; 4],
    pub covered: [u32; 4],
    pub rays: u64,
    /// Wall-clock seconds the build took (diagnostic; not deterministic).
    pub secs: f64,
}

/// The map's positions, indexed by an [`CELL`] m grid.
#[derive(Debug, Clone, Default)]
pub struct PositionTable {
    /// Sorted by grid cell (x, then z), then source order.
    pub positions: Vec<Position>,
    /// Per occupied cell, the run of `positions` in it.
    cells: std::collections::BTreeMap<(i32, i32), (u32, u32)>,
    pub stats: TableStats,
}

fn cell_of(x: f32, z: f32) -> (i32, i32) {
    ((x / CELL).floor() as i32, (z / CELL).floor() as i32)
}

impl PositionTable {
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// The indices of the positions within `r` (XZ) of `p`, in table order.
    pub fn within(&self, p: Vec3, r: f32, out: &mut Vec<u32>) {
        out.clear();
        let (x0, z0) = cell_of(p.x - r, p.z - r);
        let (x1, z1) = cell_of(p.x + r, p.z + r);
        for cx in x0..=x1 {
            for (_, &(a, b)) in self.cells.range((cx, z0)..=(cx, z1)) {
                for i in a..b {
                    if nav::xz_dist(self.positions[i as usize].pos, p) <= r {
                        out.push(i);
                    }
                }
            }
        }
    }

    fn index(mut positions: Vec<Position>, stats: TableStats) -> Self {
        // A stable sort: one cell's points keep their build order.
        positions.sort_by_key(|q| cell_of(q.pos.x, q.pos.z));
        let mut cells = std::collections::BTreeMap::new();
        let mut a = 0usize;
        while a < positions.len() {
            let c = cell_of(positions[a].pos.x, positions[a].pos.z);
            let mut b = a + 1;
            while b < positions.len() && cell_of(positions[b].pos.x, positions[b].pos.z) == c {
                b += 1;
            }
            cells.insert(c, (a as u32, b as u32));
            a = b;
        }
        Self {
            positions,
            cells,
            stats,
        }
    }
}

/// The build's state: the map, and the rays it has cast.
struct Builder<'a> {
    sim: &'a Sim,
    g: &'a nav::NavGraph,
    rays: u64,
}

impl Builder<'_> {
    /// Is the segment open — no terrain, no map body (no vehicle hull)?
    fn clear(&mut self, from: Vec3, dir: Vec3, len: f32) -> bool {
        self.rays += 1;
        self.sim.terrain_ray(from, dir, len, 1.0).is_none()
            && self
                .sim
                .collision
                .ray_with(
                    from,
                    dir,
                    len,
                    crate::collision::Layers::ORDNANCE,
                    crate::collision::HullFilter::Static,
                )
                .is_none()
    }

    /// The floor a soldier stands on at (x, z) on the storey of `y_ref`, when
    /// one can: else the open terrain's when nothing is built over it; dry, on
    /// the level, in no soldier barrier, with head room.
    fn stand(&mut self, x: f32, z: f32, y_ref: f32) -> Option<f32> {
        let sim = self.sim;
        let mut f = sim.ground_point_under(Vec3::new(x, y_ref + 1.0, z), false);
        if f <= crate::collision::NO_GROUND + 1.0 || (f - y_ref).abs() > STOREY {
            let t = sim.heightfield.as_ref().map(|h| h.height_at(x, z))?;
            f = sim.ground_point_under(Vec3::new(x, t + 0.2, z), false);
            if (f - t).abs() > 0.3 {
                return None;
            }
        }
        if sim.water_level.is_some_and(|w| f < w - 0.2) {
            return None;
        }
        if matches!(sim.level_at(Vec3::new(x, f, z)), Level::Off(_)) {
            return None;
        }
        let p = Vec2::new(x, z);
        if self
            .g
            .barriers
            .iter()
            .any(|b| b.enabled && b.stops(nav::SOLDIER) && b.contains_r(p, IN_BARRIER))
        {
            return None;
        }
        if !self.clear(Vec3::new(x, f + 0.3, z), Vec3::Y, HEAD_ROOM) {
            return None;
        }
        Some(f)
    }

    /// The (LOW, CORNER) masks at floor point `p`.
    fn sectors(&mut self, p: Vec3) -> (u16, u16) {
        let (mut low, mut corner) = (0u16, 0u16);
        let c = p + Vec3::Y * CROUCH_EYE;
        let s = p + Vec3::Y * STAND_EYE;
        for k in 0..DIRS {
            let (sa, ca) = fmath::sin_cos(k as f32 / DIRS as f32 * std::f32::consts::TAU);
            let d = Vec3::new(ca, 0.0, sa);
            if self.clear(c, d, NEAR) {
                continue;
            }
            if self.clear(s, d, NEAR) {
                if self.clear(s, d, LANE) {
                    low |= 1 << k;
                }
            } else {
                let side = Vec3::new(-d.z, 0.0, d.x);
                for sg in [-1.0f32, 1.0] {
                    if self.clear(s, side * sg, LEAN) && self.clear(s + side * (LEAN * sg), d, LANE)
                    {
                        corner |= 1 << k;
                        break;
                    }
                }
            }
        }
        (low, corner)
    }

    /// The floor of the hub nearest (x, z) in XZ within [`EDGE_HUB`].
    fn hub_floor_near(&self, x: f32, z: f32) -> Option<f32> {
        let mut best: Option<(f32, f32)> = None;
        for (i, h) in self.g.hubs.iter().enumerate() {
            let Some(f) = self.sim.nav_floor(i as u16) else {
                continue;
            };
            let d = Vec2::new(h.pos.x - x, h.pos.z - z).length();
            if d <= EDGE_HUB && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, f));
            }
        }
        best.map(|b| b.1)
    }

    /// The hub nearest `p` in XZ whose floor is on `p`'s storey.
    fn hub_of(&self, p: Vec3) -> u16 {
        let mut best: Option<(f32, u16)> = None;
        for (i, h) in self.g.hubs.iter().enumerate() {
            let Some(f) = self.sim.nav_floor(i as u16) else {
                continue;
            };
            if (f - p.y).abs() > super::HUB_LEVEL_TOL {
                continue;
            }
            let d = nav::xz_dist(h.pos, p);
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, i as u16));
            }
        }
        best.map_or(u16::MAX, |b| b.1)
    }
}

/// Build the table for `sim`'s map (empty with no graph). Pure: the same
/// map gives the same table, point for point.
pub fn build(sim: &Sim) -> PositionTable {
    let t0 = std::time::Instant::now();
    let Some(g) = sim.nav.as_ref() else {
        return PositionTable::default();
    };
    let mut b = Builder { sim, g, rays: 0 };
    let mut stats = TableStats::default();
    let mut out: Vec<Position> = Vec::new();
    let mut push = |b: &mut Builder, stats: &mut TableStats, mut q: Position| {
        let (low, corner) = b.sectors(q.pos);
        q.low = low;
        q.corner = corner;
        q.hub = b.hub_of(q.pos);
        let s = q.source as usize;
        stats.standable[s] += 1;
        stats.covered[s] += u32::from(q.covered());
        out.push(q);
    };
    let blank = |pos: Vec3, source: Source| Position {
        pos,
        source,
        facing: Vec2::ZERO,
        stance: 0,
        stance2: 0,
        hub: u16::MAX,
        low: 0,
        corner: 0,
    };
    for h in &g.hints {
        let source = match h.kind {
            2 => Source::Cover,
            0 => Source::Snipe,
            _ => continue,
        };
        stats.tried[source as usize] += 1;
        let f = b.stand(h.pos.x, h.pos.z, h.pos.y).or_else(|| {
            b.hub_floor_near(h.pos.x, h.pos.z)
                .and_then(|y| b.stand(h.pos.x, h.pos.z, y))
        });
        let Some(f) = f else {
            continue;
        };
        let ahead = Vec2::new(h.rows[2][0], h.rows[2][2]);
        let mut q = blank(Vec3::new(h.pos.x, f, h.pos.z), source);
        q.facing = ahead.try_normalize().unwrap_or(Vec2::ZERO);
        q.stance = h.stance_mask;
        q.stance2 = h.secondary_stance_mask;
        push(&mut b, &mut stats, q);
    }
    for (i, h) in g.hubs.iter().enumerate() {
        let Some(f0) = sim.nav_floor(i as u16) else {
            continue;
        };
        for k in 0..9 {
            let (dx, dz) = if k == 8 {
                (0.0, 0.0)
            } else {
                let (sa, ca) = fmath::sin_cos(k as f32 / 8.0 * std::f32::consts::TAU);
                (ca * h.radius * HUB_RING, sa * h.radius * HUB_RING)
            };
            stats.tried[Source::Hub as usize] += 1;
            let (x, z) = (h.pos.x + dx, h.pos.z + dz);
            if let Some(f) = b.stand(x, z, f0) {
                push(&mut b, &mut stats, blank(Vec3::new(x, f, z), Source::Hub));
            }
        }
    }
    for br in g
        .barriers
        .iter()
        .filter(|br| br.enabled && br.stops(nav::SOLDIER))
    {
        for (sx, sz) in [
            (1.0f32, 0.0f32),
            (-1.0, 0.0),
            (0.0, 1.0),
            (0.0, -1.0),
            (1.0, 1.0),
            (1.0, -1.0),
            (-1.0, 1.0),
            (-1.0, -1.0),
        ] {
            let grow = |s: f32, half: f32| s * (half + if s != 0.0 { EDGE_OUT } else { 0.0 });
            let p = br.centre + br.axes[0] * grow(sx, br.half.x) + br.axes[1] * grow(sz, br.half.y);
            stats.tried[Source::Edge as usize] += 1;
            let Some(y) = b.hub_floor_near(p.x, p.y) else {
                continue;
            };
            if let Some(f) = b.stand(p.x, p.y, y) {
                push(
                    &mut b,
                    &mut stats,
                    blank(Vec3::new(p.x, f, p.y), Source::Edge),
                );
            }
        }
    }
    stats.rays = b.rays;
    stats.secs = t0.elapsed().as_secs_f64();
    PositionTable::index(out, stats)
}

impl Sim {
    /// Card TPOS: (re)build the position table ([`build`]) — once the
    /// posts are in, since the floors it stands on are walked from them.
    pub fn build_positions(&mut self) {
        self.positions = build(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_direction_falls_in_its_own_sector() {
        assert_eq!(sector_bit(Vec2::new(1.0, 0.0)), 1);
        assert_eq!(sector_bit(Vec2::new(0.0, 1.0)), 1 << 4);
        assert_eq!(sector_bit(Vec2::new(-1.0, 0.0)), 1 << 8);
        assert_eq!(sector_bit(Vec2::new(0.0, -1.0)), 1 << 12);
        // Just under half a sector off still rounds to the sector.
        let a = (0.49f32 / 16.0) * std::f32::consts::TAU;
        assert_eq!(sector_bit(Vec2::new(a.cos(), a.sin())), 1);
        assert_eq!(sector_bit(Vec2::ZERO), 0);
    }
}
