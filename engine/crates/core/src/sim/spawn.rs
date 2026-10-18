//! Who is on the field: [`Sim::spawn`] through the battle's reinforcement
//! gate and the no-overspawn shedding; the class tables ([`Sim::ai_class`],
//! [`Sim::spawn_ai`]); the spawn screen's class choice (`spawn_select`, step
//! 0a); the respawn at a friendly post (`respawn_slot`, step 0), paced by the
//! [`SpawnWaves`]; where a team may spawn ([`Sim::spawn_points`]: its posts'
//! nodes and its live command walkers' points, card CWSP) and which of them a
//! bot takes — one post drawn by `AISpawnWeight`, then one of that post's
//! nodes ([`Sim::spawn_posts`], `keep_or_draw_spawn_post`, card BSPN); the floor a spawn
//! stands on ([`Sim::spawn_ground`] — a post's node inside a solid body on
//! top of it, [`Sim::resolve_spawn_seats`], card LLEG); and the headless
//! referees' [`Sim::vacate_player_slot`].
//! The spawn, class and spawn-screen tests live here.

use super::input::{buttons, PadState};
use super::state::{Event, Sim};
use crate::aim_assist::AssistState;
use crate::battle::SpawnOutcome;
use crate::unit::{Controller, SpawnPost, UnitHandle, UnitSpec};
use glam::{Vec2, Vec3};

/// PROVISIONAL (ours): the weight EVERY class keeps once its share of the team
/// is alive, as a fraction of an even share (`Sim::ai_class_weights`).
pub const CLASS_FLOOR: f32 = 0.05;
/// PROVISIONAL (ours; the maintainer, 2026-10-01): how the team's alive-AI cap is shared
/// out — the DEFAULT class (the mission's first: the rifleman, the battle
/// droid; it carries the highest authored minimum on all 114 team tables)
/// takes [`CLASS_SHARE_DEFAULT`] parts, a STANDARD class
/// [`CLASS_SHARE_STANDARD`] and a SPECIAL one [`CLASS_SHARE_SPECIAL`].
pub const CLASS_SHARE_DEFAULT: f32 = 4.0;
pub const CLASS_SHARE_STANDARD: f32 = 2.0;
pub const CLASS_SHARE_SPECIAL: f32 = 1.0;
/// The special classes are the table's entries from this index on — by the
/// discs' own class ORDER: BF2 `SetupTeams` lists soldier, assault, engineer,
/// sniper, then `officer` and `special` (its two unlock classes); BF1's fifth
/// `AddUnitClass` is the wookiee, the dark trooper, the jet trooper or the
/// droideka. Over the 57 missions that is exactly the officers, wookiees,
/// smugglers, droidekas, dark troopers and jet troopers.
pub const CLASS_SPECIAL_FROM: usize = 4;

/// A class's parts of the team by its place in the table (see
/// [`CLASS_SHARE_DEFAULT`]).
pub fn class_share(index: usize) -> f32 {
    match index {
        0 => CLASS_SHARE_DEFAULT,
        i if i >= CLASS_SPECIAL_FROM => CLASS_SHARE_SPECIAL,
        _ => CLASS_SHARE_STANDARD,
    }
}

/// Spawn waves (re-tickets-spawn.md §1.5/§2.5: BF1 member update
/// `FUN_00295c40`, BF2 `FUN_0034cea8` + manager `FUN_0036c750`, same scheme).
/// A wave opens every `delay` s and releases one spawn slot every `interval`
/// s (`slots = delay / interval`); a member killed during wave `w` is assigned
/// wave `w + 1` and the team's next slot number, and may spawn — once its own
/// respawn timer has expired — when
/// `(w_m − w) * slots + (s_m − s) < 0`, i.e. its absolute slot is behind the
/// manager's. Missions call `SetSpawnDelay(10, 0.25)` (engine defaults
/// 15.0 / 0.5). PROVISIONAL detail: slot numbers restart at 0 on each new
/// wave and are handed out in death order (the doc pins the comparison and
/// the wave+1 assignment, not the slot bookkeeping).
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnWaves {
    pub delay: f32,
    pub interval: f32,
    pub timer: f32,
    pub wave: u32,
    pub next_slot: u32,
}

impl SpawnWaves {
    /// `SetSpawnDelay(10, 0.25)` as every retail conquest mission sets it.
    pub fn mission_default() -> Self {
        Self {
            delay: 10.0,
            interval: 0.25,
            timer: 0.0,
            wave: 0,
            next_slot: 0,
        }
    }

    pub fn slots(&self) -> u32 {
        (self.delay / self.interval).round().max(1.0) as u32
    }

    pub fn cur_slot(&self) -> u32 {
        (self.timer / self.interval) as u32
    }

    pub fn tick(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= self.delay {
            self.timer -= self.delay;
            self.wave += 1;
            self.next_slot = 0;
        }
    }

    /// Slot for a member that just died: next wave, next slot.
    pub fn assign(&mut self) -> (u32, u32) {
        let slot = (self.wave + 1, self.next_slot);
        self.next_slot += 1;
        slot
    }

    /// Absolute slots the member is ahead of the manager (≤ 0 = may spawn).
    fn ahead(&self, (w, s): (u32, u32)) -> i64 {
        (i64::from(w) - i64::from(self.wave)) * i64::from(self.slots())
            + (i64::from(s) - i64::from(self.cur_slot()))
    }

    pub fn ready(&self, slot: (u32, u32)) -> bool {
        self.ahead(slot) < 0
    }

    /// Seconds until the member's slot passes (HUD "respawn in").
    pub fn countdown(&self, slot: (u32, u32)) -> f32 {
        let ahead = self.ahead(slot) + 1;
        let into_slot = self.timer - self.cur_slot() as f32 * self.interval;
        (ahead as f32 * self.interval - into_slot).max(0.0)
    }
}

/// A spawn node authored UNDER its floor is looked for from this far over the
/// node (m). PROVISIONAL (ours).
pub const SPAWN_LIFT: f32 = 3.0;

/// A floor a soldier walks to fewer than this many cells from is a PEN
/// ([`Sim::floor_reach`]; about 110 m2). PROVISIONAL (ours).
pub const PEN_CELLS: usize = 200;
/// The pen test's cell (m). PROVISIONAL (ours).
pub const PEN_CELL: f32 = 0.75;
/// The pen test walks off a ledge no higher than this (m): the AI's own
/// ledge rule (`ai::LEDGE_DROP`).
pub const PEN_DROP: f32 = 4.0;
/// A capsule the wall pass moves by more than this is in a body (m).
const PEN_CLEAR: f32 = 0.02;

/// A flood cell: its grid step and its storey (1.5 m bands), so a floor over
/// a floor is a cell of its own.
fn flood_key(ix: i32, iz: i32, y: f32) -> (i32, i32, i32) {
    (ix, iz, (y / 1.5).floor() as i32)
}
/// A penned spawn node's floor is looked for up to this far over it (whole
/// m). PROVISIONAL (ours): BF1 Naboo Plains' column is 4.26 m over its node.
pub const SEAT_REACH: u32 = 6;

/// One class spawn point of a command walker at `pos` with entity rotation
/// `rot`, as a spawn node (re-command-walker-post §4 [O], B2 FUN_00356998 /
/// B1 FUN_0029df70): the authored position through the walker's matrix
/// (`pos + x·right + y·up + z·forward`), facing the walker's heading turned
/// by the authored angle, yaw only. RAM vector: `the_ram_walker_gives_its_six_points`.
///
/// PROVISIONAL: the PS2 then pulls the point back along a ray from the
/// walker's bounding-sphere centre to the first terrain or building hit; the
/// port seats the unit with [`Sim::spawn_ground`] instead. The turn's sense
/// is the stored quaternion's: BF2 Hoth's RAM holds (0, 0.0436, 0, ±0.9990)
/// for 5° / 355° (`lead_tmp/cwsp/cp_ram2.py`), the form of
/// [`quat_y`](crate::vehicle::hover::quat_y) — the heading plus the angle,
/// so the AT-AT's two sides face 5° outward. That a spawned unit shows the
/// 5° is not measured (§11).
pub fn walker_point(pos: Vec3, rot: glam::Quat, (local, angle): (Vec3, f32)) -> (Vec3, f32) {
    let fwd = rot * Vec3::Z;
    (pos + rot * local, crate::fmath::atan2(fwd.x, fwd.z) + angle)
}

impl Sim {
    /// The floor a unit spawned at `node` stands on: the ground under the
    /// node's own height, and when there is none — a node authored under its
    /// floor (TAT1's `(-257.9, 10.6, -11.2)` is 2.06 m under the street, past
    /// the terrain's bury rule) — the ground under [`SPAWN_LIFT`] over it.
    /// Without this such a unit stood on the no-ground floor at -300 for the
    /// round (and now dies at the kill floor every time it spawns).
    ///
    /// The same when the ground under the node is another floor, OFF the
    /// playable level ([`Sim::level_at`], card LVLF): BF2 Utapau's
    /// `(-46.0, 0.0, 225.2)` is 1.01 m under its landing and 74.8 m over the
    /// sinkhole's floor, where every unit that drew the node spawned and was
    /// given up as lost 45 s later (61 of the map's 80 lost deaths in 8
    /// rounds). PROVISIONAL (ours): how the PS2 seats a spawn is not read.
    ///
    /// A post's node whose floor is a PEN stands on its seat instead
    /// ([`Sim::resolve_spawn_seats`], card LLEG).
    pub fn spawn_ground(&self, node: Vec3) -> f32 {
        match self
            .spawn_seats
            .iter()
            .find(|(n, _)| n.x == node.x && n.z == node.z)
        {
            Some(&(_, y)) => y,
            None => self.node_floor(node),
        }
    }

    /// Card LLEG: derive [`Sim::spawn_seats`] — for every post's spawn node
    /// whose floor is a PEN ([`Sim::floor_reach`] under [`PEN_CELLS`]: the
    /// node is inside a solid body), the lowest floor over it, within
    /// [`SEAT_REACH`], that is none. Asked when a post is added, with no
    /// vehicle on the field yet.
    ///
    /// BF1 Naboo Plains' cp3 node `(-107.2, -108.5, 63.2)` lies inside a
    /// fallen column 3 m thick: the PS2's soldier spawns ON it, at y -104.24,
    /// stands 1.5 s, walks off its end and drops to the ground (live RAM, one
    /// spawn in 464 s, `lead_tmp/lleg/ps2/nab1_soldiers.csv`); the port's
    /// stood on the ground inside the mesh and walked its walls until it was
    /// given up (384 of the map's 415 stalled windows in 8 rounds, 12 % of
    /// all 32 maps'). BF2 Mustafar's `(-66.2, 66.7, 23.1)` is inside a crate
    /// (not measured on the PS2); no other node of the 32 maps moves.
    /// PROVISIONAL (ours): the pen test and the reach — how the PS2 seats a
    /// spawn is not read.
    pub fn resolve_spawn_seats(&mut self) {
        let seats = self
            .cps
            .iter()
            .flat_map(|c| c.def.spawn_nodes.iter())
            .filter_map(|&(n, _)| self.pen_seat(n).map(|y| (n, y)))
            .collect();
        self.spawn_seats = seats;
    }

    fn pen_seat(&self, node: Vec3) -> Option<f32> {
        let y = self.node_floor(node);
        let at = |y: f32| Vec3::new(node.x, y, node.z);
        let mut over = (1..=SEAT_REACH)
            .map(|k| self.ground_under(node + Vec3::Y * k as f32, false))
            .filter(|&f| f > y + crate::collision::STEP_HEIGHT)
            .peekable();
        if over.peek().is_none() || self.floor_reach(at(y), PEN_CELLS) >= PEN_CELLS {
            return None;
        }
        let mut last = y;
        over.find(|&f| {
            let new = f > last;
            last = last.max(f);
            new && self.floor_reach(at(f), PEN_CELLS) >= PEN_CELLS
        })
    }

    fn node_floor(&self, node: Vec3) -> f32 {
        let y = self.ground_under(node, false);
        let lifted = || self.ground_under(node + Vec3::Y * SPAWN_LIFT, false);
        if y <= crate::collision::NO_GROUND {
            return lifted();
        }
        if self.level_at(Vec3::new(node.x, y, node.z)).is_off() {
            let up = lifted();
            if up > y && !self.level_at(Vec3::new(node.x, up, node.z)).is_off() {
                return up;
            }
        }
        y
    }

    /// How many [`PEN_CELL`] cells of floor a soldier standing at `from`
    /// walks to, counted up to `cap`: a flood over the grid, a step between
    /// two cells being one with a floor he stands on at its end (no steeper
    /// than 45 degrees up, no more than [`PEN_DROP`] down) and no wall
    /// between — the sweep of the stepped capsule, from the higher floor of
    /// the two, and the wall pass leaving him where he then stands. Card
    /// LLEG: under [`PEN_CELLS`] the place is a pen. PROVISIONAL (ours).
    pub fn floor_reach(&self, from: Vec3, cap: usize) -> usize {
        self.floor_flood(from, cap, &|_| false).0
    }

    /// [`Sim::floor_reach`]'s flood, stopped at the first cell whose floor
    /// point `goal` accepts: the cells counted, and that point with the
    /// length of the flood's shortest walk to it (m, in [`PEN_CELL`] steps
    /// along the grid). Card SPNC's
    /// census asks it whether a soldier on a spawn node walks to his first
    /// leg's hub at all, round walls and down no more than [`PEN_DROP`].
    pub fn floor_flood(
        &self,
        from: Vec3,
        cap: usize,
        goal: &dyn Fn(Vec3) -> bool,
    ) -> (usize, Option<(Vec3, f32)>) {
        let mut seen = std::collections::HashSet::from([flood_key(0, 0, from.y)]);
        let mut queue = std::collections::VecDeque::from([(0i32, 0i32, from.y, 0u32)]);
        let mut cells = 0;
        while let Some((ix, iz, y, d)) = queue.pop_front() {
            cells += 1;
            if cells >= cap {
                break;
            }
            let p = Vec3::new(
                from.x + ix as f32 * PEN_CELL,
                y,
                from.z + iz as f32 * PEN_CELL,
            );
            if goal(p) {
                return (cells, Some((p, d as f32 * PEN_CELL)));
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let Some(f) = self.flood_step(p, dx as f32, dz as f32) {
                    if seen.insert(flood_key(ix + dx, iz + dz, f)) {
                        queue.push_back((ix + dx, iz + dz, f, d + 1));
                    }
                }
            }
        }
        (cells, None)
    }

    /// One step of the floor flood from the floor point `a` by one
    /// [`PEN_CELL`] along (`dx`, `dz`): the floor a soldier stands on at its
    /// end ([`Sim::floor_flood`]'s rule), `None` when there is none or a wall
    /// is between.
    fn flood_step(&self, a: Vec3, dx: f32, dz: f32) -> Option<f32> {
        use crate::collision::Capsule;
        let height = crate::unit::capsule_height(crate::soldier::Stance::Stand);
        let radius = crate::unit::SOLDIER_RADIUS;
        let b = a + Vec3::new(dx, 0.0, dz) * PEN_CELL;
        let (f, n) = self.ground_under_n(b, false, a.y + PEN_CELL);
        if f <= crate::collision::NO_GROUND
            || n.y <= crate::soldier::FLOOR_MIN_NY
            || f - a.y > PEN_CELL
            || a.y - f > PEN_DROP
        {
            return None;
        }
        let base = Vec3::new(a.x, a.y.max(f), a.z);
        let cap = Capsule {
            base,
            height,
            radius,
        }
        .stepped();
        if self
            .collision
            .wall_sweep(&cap, Vec3::new(dx, 0.0, dz), PEN_CELL)
            .is_some()
        {
            return None;
        }
        let end = Capsule {
            base: Vec3::new(b.x, f, b.z),
            height,
            radius,
        };
        (self.collision.push_out_soldier(&end).length() <= PEN_CLEAR).then_some(f)
    }

    /// Is a soldier standing at `from` (feet on a floor) clear of every body
    /// - the wall pass leaves him where he is?
    pub fn soldier_clear(&self, from: Vec3) -> bool {
        let cap = crate::collision::Capsule {
            base: from,
            height: crate::unit::capsule_height(crate::soldier::Stance::Stand),
            radius: crate::unit::SOLDIER_RADIUS,
        };
        self.collision.push_out_soldier(&cap).length() <= PEN_CLEAR
    }

    /// Card TGAP: can a soldier standing at `from` WALK to within `reach` of
    /// `to` (XZ) on a floor within `tol` of `to_y` - round walls, by
    /// [`Sim::floor_flood`]'s steps, never more than `band` from the straight
    /// line between the two? A best-first search over the flood's grid (the
    /// cell nearest `to` first), given up after `cap` cells: `None` when it
    /// ran out of cells before it could answer, else whether it got there.
    /// PROVISIONAL (ours): the PS2 has no such test; it is the port's load-time
    /// check of an authored arc against its collision (BF1 Tatooine's arc
    /// 78 -> 77 runs into a leaning slab, shut on the PS2 too).
    #[allow(clippy::too_many_arguments)]
    pub fn floor_path(
        &self,
        from: Vec3,
        to: Vec2,
        to_y: f32,
        reach: f32,
        tol: f32,
        band: f32,
        cap: usize,
    ) -> Option<bool> {
        self.floor_path_near(from, to, to_y, reach, tol, band, cap)
            .0
    }

    /// [`Sim::floor_path`], and the cell nearest `to` (XZ) it walked to with
    /// its floor: what a census prints for an arc it calls walled.
    #[allow(clippy::too_many_arguments)]
    pub fn floor_path_near(
        &self,
        from: Vec3,
        to: Vec2,
        to_y: f32,
        reach: f32,
        tol: f32,
        band: f32,
        cap: usize,
    ) -> (Option<bool>, Vec3) {
        use std::cmp::Reverse;
        let start = Vec2::new(from.x, from.z);
        let line = to - start;
        let len2 = line.length_squared().max(1e-6);
        let off_line = |p: Vec2| {
            let t = ((p - start).dot(line) / len2).clamp(0.0, 1.0);
            (p - (start + line * t)).length()
        };
        // Integer priority (mm to the goal) keeps the order exact.
        let prio = |p: Vec2| ((p - to).length() * 1000.0) as u32;
        let mut seen = std::collections::HashSet::from([flood_key(0, 0, from.y)]);
        let mut heap = std::collections::BinaryHeap::from([(
            Reverse(prio(start)),
            0i32,
            0i32,
            from.y.to_bits(),
        )]);
        let mut cells = 0;
        let mut near = (f32::INFINITY, from);
        while let Some((_, ix, iz, yb)) = heap.pop() {
            let y = f32::from_bits(yb);
            let p = Vec2::new(from.x + ix as f32 * PEN_CELL, from.z + iz as f32 * PEN_CELL);
            let d = (p - to).length();
            if d < near.0 {
                near = (d, Vec3::new(p.x, y, p.y));
            }
            if d <= reach && (y - to_y).abs() <= tol {
                return (Some(true), near.1);
            }
            cells += 1;
            if cells >= cap {
                return (None, near.1);
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let q = p + Vec2::new(dx as f32, dz as f32) * PEN_CELL;
                if off_line(q) > band {
                    continue;
                }
                let Some(f) = self.flood_step(Vec3::new(p.x, y, p.y), dx as f32, dz as f32) else {
                    continue;
                };
                if seen.insert(flood_key(ix + dx, iz + dz, f)) {
                    heap.push((Reverse(prio(q)), ix + dx, iz + dz, f.to_bits()));
                }
            }
        }
        (Some(false), near.1)
    }

    /// Where `team` may spawn: the spawn nodes of the posts it may spawn at,
    /// in post order, then the points of its live command walkers
    /// ([`Sim::walker_spawn_points`]).
    pub fn spawn_points(&self, team: u8) -> Vec<(Vec3, f32)> {
        let mut points: Vec<(Vec3, f32)> = self
            .cps
            .iter()
            .filter(|c| c.spawnable_by(team))
            .flat_map(|c| c.def.spawn_nodes.iter().copied())
            .collect();
        points.extend(self.walker_spawn_points(team));
        points
    }

    /// The live command walkers `team` may spawn at, in vehicle order: a
    /// walker whose class authors spawn points and whose team is `team` — its
    /// pad's, from the spawn to the kill (re-command-walker-post §3 [O]: no
    /// ally, no capture). Destroyed, it is gone from here at once; the pad's
    /// next walker is here again.
    pub fn command_walkers(
        &self,
        team: u8,
    ) -> impl Iterator<Item = (crate::vehicle::VehicleHandle, &crate::vehicle::Vehicle)> {
        self.alive_vehicles().filter(move |(_, v)| {
            v.team == Some(team) && !self.class_spawn_points(v.class).is_empty()
        })
    }

    /// Vehicle class `class`'s spawn points (a command walker's
    /// `WalkerParams::spawn_points`; empty for anything else).
    fn class_spawn_points(&self, class: u32) -> &[(Vec3, f32)] {
        self.walker_params
            .get(class as usize)
            .and_then(Option::as_ref)
            .map_or(&[], |p| &p.spawn_points)
    }

    /// The spawn points of `team`'s live command walkers (card CWSP,
    /// re-command-walker-post §4 [O]): each class point through the walker's
    /// LIVE frame ([`walker_point`]). Not a `CommandPost`: nothing captures
    /// it, and BF2's bleed and all-posts rule — the port's on every map —
    /// leave it out (§7).
    pub fn walker_spawn_points(&self, team: u8) -> Vec<(Vec3, f32)> {
        self.command_walkers(team)
            .flat_map(|(_, v)| {
                self.class_spawn_points(v.class)
                    .iter()
                    .map(|&p| walker_point(v.pos, v.rot, p))
            })
            .collect()
    }

    /// The points of the command walkers `team`'s pads are about to build —
    /// a pad of the team with no vehicle yet and a command-walker class —
    /// in the pad's own pose. For the OPENING wave only: the PS2's walker
    /// stands on its pad before the first unit spawns (BF2: the walker at
    /// 0.06 s, units from 0.9 s; §10.3), where the port's pads build on the
    /// first tick, after the wave is placed.
    pub fn pad_walker_spawn_points(&self, team: u8) -> Vec<(Vec3, f32)> {
        self.vehicle_pads
            .iter()
            .filter(|p| p.team == Some(team) && p.vehicles.is_empty() && p.wants_spawn())
            .flat_map(|p| {
                let class = p.def.class_for(p.team).map_or(u32::MAX, |c| c.spec.class);
                let rot = crate::vehicle::hover::quat_y(-p.def.yaw);
                self.class_spawn_points(class)
                    .iter()
                    .map(move |&pt| walker_point(p.def.pos, rot, pt))
            })
            .collect()
    }

    /// The posts `team` may spawn at, each with its weight in a bot's draw
    /// (card BSPN): its command posts in post order, then its live command
    /// walkers in vehicle order — to the PS2 a walker's post is a post like
    /// any other, constructed with the default weight
    /// (re-command-walker-post §2, §6 [O]).
    pub fn spawn_posts(&self, team: u8) -> Vec<(SpawnPost, f32)> {
        let mut posts: Vec<(SpawnPost, f32)> = self
            .cps
            .iter()
            .enumerate()
            .filter(|(_, c)| c.spawnable_by(team))
            .map(|(i, c)| (SpawnPost::Post(i as u16), c.ai_spawn_weight))
            .collect();
        posts.extend(
            self.command_walkers(team)
                .map(|(h, _)| (SpawnPost::Walker(h), crate::cp::AI_SPAWN_WEIGHT_DEFAULT)),
        );
        posts
    }

    /// Post `post`'s spawn nodes for `team`: a command post's own, or a live
    /// command walker's class points through its live frame. Empty when the
    /// team may not (or no longer) spawn there.
    pub fn spawn_post_nodes(&self, team: u8, post: SpawnPost) -> Vec<(Vec3, f32)> {
        match post {
            SpawnPost::Post(i) => self
                .cps
                .get(usize::from(i))
                .filter(|c| c.spawnable_by(team))
                .map_or_else(Vec::new, |c| c.def.spawn_nodes.clone()),
            SpawnPost::Walker(h) => self
                .command_walkers(team)
                .find(|(w, _)| *w == h)
                .map_or_else(Vec::new, |(_, v)| {
                    self.class_spawn_points(v.class)
                        .iter()
                        .map(|&p| walker_point(v.pos, v.rot, p))
                        .collect()
                }),
        }
    }

    /// One draw of a post for a bot of `team`, as BF2's `FUN_002e7240`
    /// (re-command-walker-post §6 [O]): with S the sum of the eligible
    /// posts' weights, `r = uniform(0, S − 1)` and the first post in list
    /// order at which the running subtraction of the weights reaches ≤ 0.
    /// `None` when the team has no post or every weight is 0. BF1's draw
    /// (`FUN_00261e80`) is uniform over the same list; with BF1's equal
    /// weights this rule is that to within 1 part in S − 1, and like BF1's
    /// it takes no draw when there is one post.
    pub fn draw_spawn_post(&mut self, team: u8) -> Option<SpawnPost> {
        let posts = self.spawn_posts(team);
        let total: f32 = posts.iter().map(|p| p.1).sum();
        if total <= 0.0 {
            return None;
        }
        if let [(only, _)] = posts[..] {
            return Some(only);
        }
        let mut r = self.lcg.uniform(0.0, total - 1.0);
        for &(post, w) in &posts {
            r -= w;
            if r <= 0.0 {
                return Some(post);
            }
        }
        posts.iter().rev().find(|p| p.1 > 0.0).map(|p| p.0)
    }

    /// Dead bot `i`'s post for its next life (card BSPN;
    /// re-command-walker-post §6 [O]): the one it already drew while its
    /// team may still spawn there (the post not captured, the walker alive),
    /// else a fresh draw. Called from the respawn loop once the unit's timer
    /// has run out, so the draw falls where the PS2's does — at the end of
    /// the respawn delay — and is kept through the wave and alive-cap waits.
    pub(super) fn keep_or_draw_spawn_post(&mut self, i: usize) -> Option<SpawnPost> {
        let team = self.units[i].team;
        let kept = self.units[i]
            .spawn_post
            .filter(|&p| self.spawn_posts(team).iter().any(|q| q.0 == p));
        let post = kept.or_else(|| self.draw_spawn_post(team));
        self.units[i].spawn_post = post;
        post
    }

    /// Remove `n` alive AI of `team` from the field (no-overspawn shedding).
    pub(super) fn shed_ai(&mut self, team: u8, n: u32) {
        let mut left = n;
        for i in 0..self.units.len() {
            if left == 0 {
                break;
            }
            let u = &mut self.units[i];
            if u.alive && !u.is_player && u.team == team {
                u.alive = false;
                u.respawn_at = None;
                let h = UnitHandle {
                    index: i as u32,
                    gen: u.gen,
                };
                self.events.push(Event::Despawn { unit: h });
                self.unseat_unit(h);
                // Phase D: a bot the no-overspawn rule sheds must not come back
                // carrying the order it had (it comes back from its `UnitSpec`).
                crate::squad::on_unit_died(self, h);
                left -= 1;
            }
        }
    }

    /// Battle admission for a (re)spawn: ticket gate + shedding.
    fn admit(&mut self, team: u8, is_player: bool) -> bool {
        match self.battle.try_spawn(team as usize, is_player) {
            SpawnOutcome::Refused => false,
            SpawnOutcome::Spawned { despawned_ai } => {
                if despawned_ai > 0 {
                    self.shed_ai(team, despawned_ai);
                }
                true
            }
        }
    }

    /// Spawn through the battle's reinforcement gate (death consumes the
    /// ticket, spawn only bumps the alive count; AI refused while tickets <=
    /// alive; a PLAYER taking the last slots sheds AI — core::battle). Feet
    /// snapped to the terrain. Returns None if refused.
    pub fn spawn(&mut self, spec: &UnitSpec) -> Option<UnitHandle> {
        if !self.admit(spec.team, spec.is_player()) {
            return None;
        }
        let slot = self.units.iter().position(|u| !u.alive);
        let gen = match slot {
            Some(i) => self.units[i].gen + 1,
            None => 1,
        };
        let mut unit = spec.build(gen, &self.weapon_classes);
        // D2: "every bot receives the tier's skill at spawn" — enforced HERE
        // so it holds for every spawn path (the class table's `spawn_ai`, the
        // team template's, a test's hand-built spec), and mirrored into the
        // spec so a respawn from it starts from the same place. Players and
        // dummies keep 0: `Unit.skill` is a BOT's number.
        unit.skill = crate::skill_for(spec.controller, self.difficulty.base_skill());
        unit.spec.skill = unit.skill;
        // The probe starts from the AUTHORED node y: spawn nodes are authored
        // ON the platforms (BF1 KAM1 `CP4SpawnPath` nodes sit at y ~ 61.4-62.3
        // for a post at y = 61), so the model floor under them is what grounds
        // the unit, not the terrain far below.
        unit.soldier.pos.y = self.spawn_ground(spec.pos);
        let index = match slot {
            Some(i) => {
                self.units[i] = unit;
                i
            }
            None => {
                self.units.push(unit);
                self.units.len() - 1
            }
        };
        let h = UnitHandle {
            index: index as u32,
            gen,
        };
        if let Some(slot) = spec.player_slot {
            let slot = usize::from(slot);
            self.reserve_slot(slot);
            if self.players[slot].is_none() {
                self.players[slot] = Some(h);
            }
        }
        Some(h)
    }

    /// Take player slot `slot`'s live unit OFF the field and free the slot,
    /// without charging the team a reinforcement. Returns where it stood
    /// (`pos`, `yaw`) so the caller can put an ordinary bot in its place;
    /// `None` when the slot holds no live unit.
    ///
    /// This is a HEADLESS-REFEREE hook (queue S0), not a game mechanic. The
    /// referees (`game/examples/round_to_outcome.rs`, `game/examples/arena.rs`)
    /// build their sim through `WalkSim`, which always seats a player in slot
    /// 0, and then never press a button. An idle human is not a unit either
    /// disc's round contains, and it is not neutral: `Battle::try_spawn` lets
    /// a player displace AI to take the last reinforcement slots, `alive()`
    /// counts it against the `alive < tickets` ticket rule for the whole
    /// round because it never dies, and [`Sim::apply_team_handicap`] reads
    /// `is_player` over `units` to decide which side is the human's. The game
    /// binary never calls this.
    ///
    /// Not a death: `Battle::on_death` spends a reinforcement and this does
    /// not — the unit is being un-spawned, so only the live count moves. The
    /// battle invariant can only relax (`alive()` drops with
    /// `alive_players`), so nothing is shed.
    pub fn vacate_player_slot(&mut self, slot: usize) -> Option<(Vec3, f32)> {
        let h = (*self.players.get(slot)?)?;
        let u = self.unit(h)?;
        let (pos, yaw, team) = (u.soldier.pos, u.soldier.yaw, u.team);
        let i = h.index as usize;
        self.units[i].alive = false;
        // A vacated unit must not come back: the respawn loop takes any dead
        // unit whose `respawn_at` is due, exactly as `shed_ai` guards.
        self.units[i].respawn_at = None;
        let t = &mut self.battle.teams[usize::from(team.min(1))];
        t.alive_players = t.alive_players.saturating_sub(1);
        debug_assert!(t.invariant_holds());
        self.events.push(Event::Despawn { unit: h });
        self.unseat_unit(h);
        // Phase D: the orders this slot gave go with it (`on_unit_died` reads
        // the still-set `spec.player_slot`), so clear before freeing the slot.
        crate::squad::on_unit_died(self, h);
        self.players[slot] = None;
        Some((pos, yaw))
    }

    /// Grow the per-slot tables (`players`, `assist`, `magnet`,
    /// `reticle_pick`, `spawn_choice`, `prev_spawn_bits`) so `slot` is addressable. They are
    /// kept the same length so a slot index is valid in all of them.
    fn reserve_slot(&mut self, slot: usize) {
        if self.players.len() <= slot {
            self.players.resize(slot + 1, None);
        }
        self.sync_slot_tables();
    }

    /// Re-align `assist`/`magnet`/`reticle_pick`/`spawn_choice`/`prev_spawn_bits` with
    /// `players` (callers may assign into `players` directly — it is a public
    /// field).
    pub fn sync_slot_tables(&mut self) {
        let n = self.players.len();
        if self.assist.len() != n {
            self.assist.resize(n, AssistState::default());
        }
        if self.magnet.len() != n {
            self.magnet.resize(n, None);
        }
        if self.reticle_pick.len() != n {
            self.reticle_pick.resize(n, None);
        }
        if self.spawn_choice.len() != n {
            self.spawn_choice.resize(n, 0);
        }
        if self.prev_spawn_bits.len() != n {
            self.prev_spawn_bits.resize(n, 0);
        }
    }

    /// How likely each class of `team` is to be the next AI spawn (the maintainer,
    /// 2026-10-01 — ours, replacing the walk of the `AddUnitClass` minimums;
    /// the PS2's own rule is untraced, card AICLS).
    ///
    /// The team's alive-AI cap is shared out by [`class_share`]: the default
    /// class 4 parts, a standard class 2, a special class 1 — on a BF2 team
    /// of 16 that is 5.3 riflemen, 2.7 of each standard class and 1.3 of each
    /// special one; on a BF1 team of 32, 11.6 / 5.8 / 2.9. A class's weight
    /// is the part of its share not yet alive, plus a small floor
    /// ([`CLASS_FLOOR`] of an EVEN share, the same for every class): each one
    /// alive takes one unit off its class's weight; a class at or past its
    /// share keeps only the floor, so sixteen riflemen or sixteen droidekas
    /// can happen and almost never do. A class at its authored `max` (BF2
    /// `SetupTeams`) has weight 0 — the disc's own cap.
    pub fn ai_class_weights(&self, team: u8) -> Vec<f32> {
        let table = &self.classes[usize::from(team.min(1))];
        let cap = self.ai_alive_cap.max(1) as f32;
        let parts: f32 = (0..table.len()).map(class_share).sum();
        let floor = CLASS_FLOOR * cap / table.len().max(1) as f32;
        (0..table.len())
            .map(|c| {
                let alive = self
                    .units
                    .iter()
                    .filter(|u| {
                        u.alive && !u.is_player && u.team == team && usize::from(u.spec.class) == c
                    })
                    .count() as u32;
                if table[c].max.is_some_and(|m| alive >= m) {
                    0.0
                } else {
                    let share = cap * class_share(c) / parts;
                    (share - alive as f32).max(0.0) + floor
                }
            })
            .collect()
    }

    /// Which class the team's NEXT AI spawn takes: one draw of the sim's LCG
    /// over [`Sim::ai_class_weights`]. Class 0 when every class is full.
    pub fn ai_class(&mut self, team: u8) -> u8 {
        let w = self.ai_class_weights(team);
        let total: f32 = w.iter().sum();
        if total <= 0.0 {
            return 0;
        }
        let mut r = self.lcg.unit() * total;
        let mut last = 0;
        for (c, &wc) in w.iter().enumerate() {
            if wc <= 0.0 {
                continue;
            }
            last = c;
            if r < wc {
                return c as u8;
            }
            r -= wc;
        }
        last as u8
    }

    /// Spawn one AI of the team's next class at `pos`/`yaw`. `None` (and no
    /// cursor advance) when the battle refuses the spawn.
    pub fn spawn_ai(&mut self, team: u8, pos: Vec3, yaw: f32) -> Option<UnitHandle> {
        let t = usize::from(team.min(1));
        if self.classes[t].is_empty() {
            return None;
        }
        let c = self.ai_class(team);
        let spec = self.classes[t][usize::from(c)].spec(
            team,
            pos,
            yaw,
            Controller::Ai,
            c,
            self.difficulty.base_skill(),
        );
        let h = self.spawn(&spec)?;
        self.ai_spawned[t] += 1;
        Some(h)
    }

    /// Rebuild dead slot `i` at a friendly spawn node (new generation).
    ///
    /// A BOT spawns at one node of the post it drew
    /// ([`Sim::keep_or_draw_spawn_post`], card BSPN): post first, then node,
    /// so a post's share of the spawns does not grow with its node count. A
    /// HUMAN still draws one node from the flat list of every post's
    /// ([`Sim::spawn_points`]) until the spawn map gives him the pick (card
    /// SPSEL).
    pub(super) fn respawn_slot(&mut self, i: usize) {
        let team = self.units[i].team;
        let is_player = self.units[i].is_player;
        let points = if is_player {
            self.spawn_points(team)
        } else {
            match self.keep_or_draw_spawn_post(i) {
                Some(post) => self.spawn_post_nodes(team, post),
                None => Vec::new(),
            }
        };
        if points.is_empty() {
            return;
        }
        if !self.admit(team, is_player) {
            return;
        }
        // Uniform node pick via the inline LCG (BF1 FUN_0029df70 / BF2
        // FUN_00356998 draw), then ground snap.
        let (pos, yaw) = points[self.lcg.index(points.len() as u32) as usize];
        let gen = self.units[i].gen + 1;
        let old = self.units[i].spec.clone();
        // A PLAYER comes back as the class their spawn screen chose (D5). A
        // choice past the end of the table clamps to the last class, so a
        // table that shrank between missions cannot index out of bounds.
        // An AI re-rolls its class by the same weights (the maintainer, 2026-10-01); it
        // is dead here, so it does not count against its old class.
        let ai_class = (old.player_slot.is_none()
            && old.controller == Controller::Ai
            && !self.classes[usize::from(old.team.min(1))].is_empty())
        .then(|| self.ai_class(old.team));
        let table = &self.classes[usize::from(old.team.min(1))];
        let spec = match old.player_slot.filter(|_| !table.is_empty()) {
            Some(slot) => {
                let c = self
                    .spawn_choice
                    .get(usize::from(slot))
                    .copied()
                    .unwrap_or(0);
                let c = usize::from(c).min(table.len() - 1);
                table[c].spec(
                    old.team,
                    pos,
                    yaw,
                    old.controller,
                    c as u8,
                    self.difficulty.base_skill(),
                )
            }
            None => match ai_class {
                Some(c) => table[usize::from(c)].spec(
                    old.team,
                    pos,
                    yaw,
                    old.controller,
                    c,
                    self.difficulty.base_skill(),
                ),
                None => UnitSpec { pos, yaw, ..old },
            },
        };
        let mut unit = spec.build(gen, &self.weapon_classes);
        // An AI respawn re-derives the tier's skill (D2): a difficulty change
        // between deaths reaches the bot at its next life, and a bot never
        // comes back with a stale level.
        unit.skill = crate::skill_for(spec.controller, self.difficulty.base_skill());
        unit.spec.skill = unit.skill;
        unit.soldier.pos.y = self.spawn_ground(pos);
        unit.kills = self.units[i].kills;
        unit.deaths = self.units[i].deaths;
        self.units[i] = unit;
        let h = UnitHandle {
            index: i as u32,
            gen,
        };
        // The respawned unit keeps its player slot.
        if let Some(slot) = self.units[i].spec.player_slot {
            let slot = usize::from(slot);
            if self.players.get(slot).copied().flatten().map(|p| p.index) == Some(i as u32) {
                self.players[slot] = Some(h);
            }
        }
        self.events.push(Event::Respawn { unit: h });
    }

    /// Step each DEAD player's class choice from its pad (D5). Runs at the
    /// top of `step`, before the respawn loop, so a press this tick reaches
    /// the respawn this tick.
    ///
    /// Edges come from `prev_spawn_bits` rather than `Soldier::prev_buttons`
    /// because a dead soldier does not step, so its own button history goes
    /// stale. The bits are recorded for EVERY slot with a pad, alive or dead,
    /// so a press held across a death does not fire the moment the player
    /// dies. At most one step per tick; `NEXT` wins if both rise. An alive
    /// player's press is ignored outright.
    ///
    /// A slot with no unit at all takes team 0's table — the only team a
    /// fresh sim's first slot can belong to.
    pub(super) fn spawn_select(&mut self, pads: &[PadState]) {
        if self.spawn_choice.len() != self.players.len() {
            self.sync_slot_tables();
        }
        const MASK: u32 = buttons::SPAWN_CLASS_NEXT | buttons::SPAWN_CLASS_PREV;
        for s in 0..self.players.len() {
            let bits = pads.get(s).map_or(0, |p| p.buttons) & MASK;
            let rose = bits & !self.prev_spawn_bits[s];
            self.prev_spawn_bits[s] = bits;
            if rose == 0 {
                continue;
            }
            let unit = self.players[s].and_then(|h| self.units.get(h.index as usize));
            match unit {
                // Alive: the spawn screen is not up, so the bits do nothing.
                Some(u) if u.alive => continue,
                _ => {}
            }
            let team = unit.map_or(0, |u| u.team);
            let n = self.classes[usize::from(team.min(1))].len() as u32;
            if n == 0 {
                continue;
            }
            let c = u32::from(self.spawn_choice[s]).min(n - 1);
            self.spawn_choice[s] = if rose & buttons::SPAWN_CLASS_NEXT != 0 {
                ((c + 1) % n) as u8
            } else {
                ((c + n - 1) % n) as u8
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cp::CpDef;
    use crate::region::Region;
    use crate::sim::tests::{duel, fire, flat_ground, sphere_cp};
    use crate::sim::InputLog;
    use crate::weapon::WeaponClass;
    use crate::TICK_DT;

    #[test]
    fn spawn_waves_release_slots_in_order() {
        let mut w = SpawnWaves::mission_default();
        assert_eq!(w.slots(), 40);
        // Two deaths at t = 0.5 s (wave 0): slots (1,0) and (1,1).
        for _ in 0..15 {
            w.tick(TICK_DT);
        }
        let a = w.assign();
        let b = w.assign();
        assert_eq!((a, b), ((1, 0), (1, 1)));
        assert!(!w.ready(a));
        assert!((w.countdown(a) - 9.75).abs() < 0.05, "{}", w.countdown(a));
        // Advance to just past 10.25 s: slot 0 released, slot 1 not yet.
        while w.wave < 1 || w.cur_slot() < 1 {
            w.tick(TICK_DT);
        }
        assert!(w.ready(a) && !w.ready(b));
        while w.cur_slot() < 2 {
            w.tick(TICK_DT);
        }
        assert!(w.ready(b));
        // A death in wave 1 waits for wave 2; slot numbering restarted.
        let c = w.assign();
        assert_eq!(c, (2, 0));
        assert!(!w.ready(c));
    }

    #[test]
    fn a_respawned_bot_re_derives_the_tiers_skill() {
        // D2: an AI respawn takes the CURRENT tier's skill, so a difficulty
        // change between lives reaches the bot; a player's stays 0.
        let mut s = Sim::empty(9);
        s.difficulty = crate::skill::Difficulty::Easy;
        s.heightfield = Some(flat_ground());
        s.add_cp(
            CpDef {
                name: "cp1".into(),
                pos: Vec3::ZERO,
                capture: Region::sphere(Vec3::ZERO, 8.0),
                control: None,
                value_bleed: 1,
                value_atk: [1, 1],
                value_def: [1, 1],
                spawn_nodes: vec![(Vec3::new(4.0, 0.0, 0.0), 0.0)],
                capture_time: 10.0,
                neutralize_time: 10.0,
            },
            Some(1),
        );
        let mut ai = UnitSpec::trooper(1, Vec3::new(0.0, 0.0, -20.0));
        ai.controller = Controller::Ai;
        let h = s.spawn(&ai).unwrap();
        assert_eq!(s.unit(h).unwrap().skill, 8);
        // Kill it, raise the tier, and let the respawn loop rebuild the slot.
        let i = h.index as usize;
        s.units[i].alive = false;
        s.units[i].respawn_at = Some(0);
        s.units[i].spawn_slot = None;
        s.difficulty = crate::skill::Difficulty::Hard;
        s.respawn_slot(i);
        assert!(s.units[i].alive, "the slot came back");
        assert_eq!(s.units[i].skill, 20, "the NEW tier's skill");
        assert_eq!(s.units[i].spec.skill, 20);
    }

    /// A class table entry with the given mission counts (everything else is
    /// the built-in trooper's, so a class differs only in what is tested).
    fn class(min: u32, max: Option<u32>) -> crate::unit::UnitClass {
        class_w(min, max, 0)
    }

    /// `class`, but firing weapon class `weapon_class` — what a respawn has
    /// to carry over for a class change to be observable.
    fn class_w(min: u32, max: Option<u32>, weapon_class: usize) -> crate::unit::UnitClass {
        crate::unit::UnitClass {
            consts: crate::soldier::SoldierConstants::default(),
            max_health: 300.0,
            health_type: crate::weapon::HealthType::Person,
            weapon_class,
            spare_clips: 6,
            weapon_slot: 0,
            slots: std::sync::Arc::new([]),
            jet: false,
            capture_posts: true,
            drops: crate::sim::DropTable::default(),
            min,
            max,
        }
    }

    /// A sim on flat ground with one post to spawn at and `table` as team 0's
    /// class list.
    fn class_sim(table: Vec<crate::unit::UnitClass>) -> Sim {
        let mut s = Sim::empty(7);
        s.heightfield = Some(flat_ground());
        s.classes[0] = table;
        s.add_cp(
            CpDef {
                name: "cp1".into(),
                pos: Vec3::ZERO,
                capture: Region::sphere(Vec3::ZERO, 8.0),
                control: None,
                value_bleed: 1,
                value_atk: [1, 1],
                value_def: [1, 1],
                spawn_nodes: vec![(Vec3::new(0.0, 0.0, 5.0), 0.0)],
                capture_time: 10.0,
                neutralize_time: 10.0,
            },
            Some(0),
        );
        s
    }

    /// The maintainer's class weights (2026-10-01): with nobody alive the default
    /// class is twice as likely as a standard one and four times a special
    /// one; one alive takes a unit off its class's weight; a class at its
    /// share keeps only the floor, the same for every class; a class at its
    /// authored max is out. Sixteen spawns of a BF2 table come out near
    /// 5 / 3 / 3 / 3 / 1 / 1.
    #[test]
    fn ai_classes_are_drawn_by_share_and_by_what_is_not_yet_alive() {
        let mut s = class_sim(vec![class(0, None); 6]);
        s.ai_alive_cap = 16;
        // 4 + 2 + 2 + 2 + 1 + 1 parts of 16, plus 5 % of an even share.
        let floor = 0.05 * 16.0 / 6.0;
        let w = s.ai_class_weights(0);
        for (c, share) in [
            16.0 / 3.0,
            8.0 / 3.0,
            8.0 / 3.0,
            8.0 / 3.0,
            4.0 / 3.0,
            4.0 / 3.0,
        ]
        .into_iter()
        .enumerate()
        {
            assert!((w[c] - (share + floor)).abs() < 1e-4, "class {c}: {w:?}");
        }
        let h = s.spawn_ai(0, Vec3::new(0.0, 0.0, 5.0), 0.0).expect("spawn");
        let c = usize::from(s.units[h.index as usize].spec.class);
        let w1 = s.ai_class_weights(0);
        assert!(
            (w[c] - w1[c] - 1.0).abs() < 1e-4,
            "one alive: one unit less ({w1:?})"
        );
        let mut n = [0u32; 6];
        n[c] += 1;
        for _ in 1..16 {
            let h = s.spawn_ai(0, Vec3::new(0.0, 0.0, 5.0), 0.0).expect("spawn");
            n[usize::from(s.units[h.index as usize].spec.class)] += 1;
        }
        assert!(
            n[0] >= 4 && n[0] > n[4] && n[0] > n[5] && n[1] + n[2] + n[3] >= 6,
            "the default class leads, the standard ones follow: {n:?}"
        );
        assert!(n[4] <= 3 && n[5] <= 3, "the special ones are few: {n:?}");
        // A class at its share keeps the floor only.
        let w = s.ai_class_weights(0);
        if n[0] >= 6 {
            assert!((w[0] - floor).abs() < 1e-4, "{n:?} {w:?}");
        }
        // Over many teams the shares hold: 33 / 17 / 17 / 17 / 8 / 8 %.
        let mut total = [0u32; 6];
        for seed in 0..200 {
            let mut s = class_sim(vec![class(0, None); 6]);
            s.ai_alive_cap = 16;
            s.lcg = crate::lcg::GameLcg::new(seed * 7919 + 1);
            for _ in 0..16 {
                let h = s.spawn_ai(0, Vec3::new(0.0, 0.0, 5.0), 0.0).expect("spawn");
                total[usize::from(s.units[h.index as usize].spec.class)] += 1;
            }
        }
        let pct: Vec<f32> = total.iter().map(|&k| k as f32 / 32.0).collect();
        assert!((pct[0] - 33.3).abs() < 3.0, "default: {pct:?}");
        assert!(
            pct[1..4].iter().all(|p| (p - 16.7).abs() < 3.0),
            "standard: {pct:?}"
        );
        assert!(
            pct[4..].iter().all(|p| (p - 8.3).abs() < 3.0),
            "special: {pct:?}"
        );
        let mut s = class_sim(vec![class(0, Some(1)), class(0, None)]);
        s.ai_alive_cap = 16;
        let mut got = [0u32; 2];
        for _ in 0..8 {
            let h = s.spawn_ai(0, Vec3::new(0.0, 0.0, 5.0), 0.0).expect("spawn");
            got[usize::from(s.units[h.index as usize].spec.class)] += 1;
        }
        assert!(got[0] <= 1, "the authored max holds: {got:?}");
        assert_eq!(s.ai_spawned, [8, 0]);
        // The cursor only advances on a spawn that happened.
        assert_eq!(s.spawn_ai(1, Vec3::ZERO, 0.0), None, "team 1 has no table");
        assert_eq!(s.ai_spawned, [8, 0]);
    }

    /// S0: `vacate_player_slot` un-spawns slot 0's idle human without
    /// spending a ticket, and the bot that fills the hole is an ordinary AI
    /// out of the class table — the headless referees' slot-0 rule.
    #[test]
    fn vacating_a_player_slot_frees_it_without_a_ticket() {
        let mut s = class_sim(vec![class(1, None), class(1, None)]);
        let node = Vec3::new(0.0, 0.0, 5.0);
        let p = s
            .spawn(&UnitSpec::player(0, node))
            .expect("the player spawns");
        s.spawn_ai(0, node, 0.0).expect("one bot beside it");
        let before = s.battle.teams[0].tickets();
        assert_eq!(
            (s.battle.teams[0].alive_players, s.battle.teams[0].alive_ai),
            (1, 1)
        );

        let (pos, yaw) = s.vacate_player_slot(0).expect("slot 0 held the player");
        assert_eq!((pos, yaw), (s.units[p.index as usize].soldier.pos, 0.0));
        // The pool is untouched — this is not a death.
        assert_eq!(s.battle.teams[0].tickets(), before);
        assert_eq!(
            (s.battle.teams[0].alive_players, s.battle.teams[0].alive_ai),
            (0, 1)
        );
        assert_eq!(s.players[0], None);
        assert!(s.unit(p).is_none(), "off the field");
        assert_eq!(s.units[p.index as usize].respawn_at, None, "and stays off");
        // Idempotent: an empty slot vacates to `None`.
        assert_eq!(s.vacate_player_slot(0), None);

        // The replacement is an ordinary bot: it reuses the freed unit slot,
        // takes the class table's next class, and is not a player.
        let h = s.spawn_ai(0, pos, yaw).expect("the filler spawns");
        assert_eq!(h.index, p.index);
        let u = &s.units[h.index as usize];
        assert!(!u.is_player && u.spec.player_slot.is_none());
        assert_eq!(u.spec.class, 1, "the table's next class, not the player's");
        assert_eq!(
            (s.battle.teams[0].alive_players, s.battle.teams[0].alive_ai),
            (0, 2)
        );
        // Stepping the sim does not resurrect the vacated player: with no
        // pad-driven unit left, `players` is empty of it and nothing respawns.
        for _ in 0..90 {
            s.step(&[PadState::default()]);
        }
        assert!(s.units.iter().all(|u| !u.is_player || !u.alive));
    }

    /// A class at its `max` is never drawn, and is drawn again once one dies.
    #[test]
    fn a_class_at_its_max_is_skipped() {
        let mut s = class_sim(vec![class(1, None), class(4, Some(1)), class(1, Some(1))]);
        let node = Vec3::new(0.0, 0.0, 5.0);
        let mut n = [0u32; 3];
        for _ in 0..12 {
            let h = s.spawn_ai(0, node, 0.0).expect("spawn");
            n[usize::from(s.units[h.index as usize].spec.class)] += 1;
        }
        assert_eq!((n[1], n[2]), (1, 1), "one of each capped class: {n:?}");
        assert_eq!(s.ai_class_weights(0)[1], 0.0);
        // Kill the class-1 unit and class 1 has a weight again.
        let i = s
            .units
            .iter()
            .position(|u| u.spec.class == 1)
            .expect("a class-1 unit");
        s.units[i].alive = false;
        assert!(s.ai_class_weights(0)[1] > 0.0);
    }

    /// Two sims built the same way from the same table hash alike, and the
    /// three new pieces of STATE all move the hash (the class tables
    /// themselves are setup data and do not).
    #[test]
    fn the_spawn_select_state_is_part_of_the_hash() {
        let table = || vec![class(1, None), class(1, None)];
        let mut a = class_sim(table());
        let mut b = class_sim(table());
        for s in [&mut a, &mut b] {
            s.spawn(&UnitSpec::player_in_slot(0, 0, Vec3::ZERO))
                .unwrap();
            s.spawn_ai(0, Vec3::new(0.0, 0.0, 5.0), 0.0).unwrap();
        }
        assert_eq!(a.hash(), b.hash(), "same table, same spawns, same hash");

        let base = a.hash();
        a.spawn_choice[0] = 1;
        assert_ne!(a.hash(), base, "spawn_choice is hashed");
        a.spawn_choice[0] = 0;
        assert_eq!(a.hash(), base);

        a.prev_spawn_bits[0] = buttons::SPAWN_CLASS_NEXT;
        assert_ne!(a.hash(), base, "prev_spawn_bits is hashed");
        a.prev_spawn_bits[0] = 0;
        assert_eq!(a.hash(), base);

        a.ai_spawned[1] += 1;
        assert_ne!(a.hash(), base, "ai_spawned is hashed");
        a.ai_spawned[1] -= 1;
        assert_eq!(a.hash(), base);

        // A unit's CLASS is state too, even between two classes that are
        // otherwise identical.
        let i = a.units.iter().position(|u| !u.is_player).unwrap();
        a.units[i].spec.class = 1;
        assert_ne!(a.hash(), base, "UnitSpec::class is hashed");
    }

    /// A two-class sim with a live player in slot 0, dead and waiting.
    fn spawn_select_sim() -> Sim {
        let mut s = class_sim(vec![class_w(1, None, 0), class_w(1, None, 1)]);
        s.add_weapon_class(WeaponClass::bf2_rifle());
        s.spawn(&UnitSpec::player_in_slot(0, 0, Vec3::ZERO))
            .unwrap();
        s
    }

    fn kill_player(s: &mut Sim) {
        let i = s.players[0].unwrap().index as usize;
        s.units[i].alive = false;
        s.units[i].respawn_at = Some(u64::MAX);
    }

    fn press(bit: u32) -> PadState {
        PadState {
            buttons: bit,
            ..PadState::default()
        }
    }

    /// D5: a dead player's NEXT press steps the class ONCE however long it is
    /// held; PREV wraps the other way; an ALIVE player's press does nothing.
    #[test]
    fn the_spawn_screen_steps_the_class_on_the_press_edge_only() {
        let mut s = spawn_select_sim();
        // Alive: the bits are ignored (and do not arm an edge for later).
        for _ in 0..5 {
            s.step(&[press(buttons::SPAWN_CLASS_NEXT)]);
        }
        assert_eq!(s.spawn_choice[0], 0, "an alive player cannot re-class");

        kill_player(&mut s);
        for _ in 0..5 {
            s.step(&[press(buttons::SPAWN_CLASS_NEXT)]);
        }
        assert_eq!(
            s.spawn_choice[0], 0,
            "a button held across the death is not a new press"
        );
        s.step(&[PadState::default()]);
        for _ in 0..5 {
            s.step(&[press(buttons::SPAWN_CLASS_NEXT)]);
        }
        assert_eq!(s.spawn_choice[0], 1, "held for five ticks = one step");
        s.step(&[PadState::default()]);
        for _ in 0..5 {
            s.step(&[press(buttons::SPAWN_CLASS_PREV)]);
        }
        assert_eq!(s.spawn_choice[0], 0);
        // PREV off 0 wraps to the last class.
        s.step(&[PadState::default()]);
        s.step(&[press(buttons::SPAWN_CLASS_PREV)]);
        assert_eq!(s.spawn_choice[0], 1);
        // Both at once: NEXT wins.
        s.step(&[PadState::default()]);
        s.step(&[press(buttons::SPAWN_CLASS_NEXT | buttons::SPAWN_CLASS_PREV)]);
        assert_eq!(s.spawn_choice[0], 0);
    }

    /// The choice reaches the RESPAWN: the player comes back on the chosen
    /// class's weapon.
    #[test]
    fn the_respawn_takes_the_chosen_class() {
        let mut s = spawn_select_sim();
        let i = s.players[0].unwrap().index as usize;
        assert_eq!(s.units[i].weapon_class, 0);
        kill_player(&mut s);
        s.step(&[press(buttons::SPAWN_CLASS_NEXT)]);
        assert_eq!(s.spawn_choice[0], 1);
        // Let the respawn loop take it.
        s.units[i].respawn_at = Some(s.tick);
        s.step(&[PadState::default()]);
        let h = s.players[0].expect("still slot 0");
        let u = s.unit(h).expect("respawned");
        assert!(u.alive);
        assert_eq!(u.spec.class, 1, "rebuilt from the chosen class");
        assert_eq!(u.weapon_class, 1, "and with that class's weapon");
    }

    /// Slot 1's pad does not move slot 0's choice.
    #[test]
    fn the_choice_is_per_slot() {
        let mut s = class_sim(vec![class(1, None), class(1, None), class(1, None)]);
        for slot in 0..2u8 {
            s.spawn(&UnitSpec::player_in_slot(
                slot,
                0,
                Vec3::new(f32::from(slot) * 20.0, 0.0, 0.0),
            ))
            .unwrap();
        }
        for slot in 0..2usize {
            let i = s.players[slot].unwrap().index as usize;
            s.units[i].alive = false;
            s.units[i].respawn_at = Some(u64::MAX);
        }
        s.step(&[PadState::default(), press(buttons::SPAWN_CLASS_NEXT)]);
        assert_eq!(s.spawn_choice, [0, 1]);
        // A slot past the end of the pad slice idles (P9's rule).
        s.step(&[press(buttons::SPAWN_CLASS_NEXT)]);
        assert_eq!(s.spawn_choice, [1, 1]);
    }

    /// A log that presses the spawn-select bits replays byte-identically.
    #[test]
    fn a_spawn_select_log_replays_identically() {
        let log = InputLog {
            seed: 0x5EED_0C1A,
            slots: 1,
            ticks: (0..400)
                .map(|i| PadState {
                    lx: ((i * 53) % 65536 - 32768) as i16,
                    buttons: match i % 37 {
                        0..=4 => buttons::SPAWN_CLASS_NEXT,
                        9..=11 => buttons::SPAWN_CLASS_PREV,
                        20..=22 => buttons::FIRE,
                        _ => 0,
                    },
                    ..Default::default()
                })
                .collect(),
        };
        let run = || {
            let mut s = spawn_select_sim();
            kill_player(&mut s);
            for pads in log.frames() {
                s.step(pads);
            }
            (s.hash(), s.spawn_choice[0])
        };
        let (h1, c1) = run();
        let (h2, c2) = run();
        assert_eq!(h1, h2);
        assert_eq!(c1, c2);
        assert!(c1 == 0 || c1 == 1);
    }

    /// BF2's AT-AT class points (x, z, angle°), `imp_walk_atat`.
    const ATAT_POINTS: [(f32, f32, f32); 6] = [
        (8.0, 0.0, 5.0),
        (8.0, 4.0, 5.0),
        (8.0, -4.0, 5.0),
        (-8.0, 0.0, 355.0),
        (-8.0, 4.0, 355.0),
        (-8.0, -4.0, 355.0),
    ];

    /// A sim on flat ground whose vehicle class 0 is a command walker with
    /// the AT-AT's six points, and a team-0 pad at `pad_pos` that builds it.
    fn command_walker_sim(pad_pos: Vec3, pad_yaw: f32) -> Sim {
        use crate::vehicle::{SeatKind, VehicleKind, VehicleSpec};
        use crate::vehicle_pad::{PadClass, PadDef};
        use crate::weapon::HealthType;
        let mut s = Sim::empty(11);
        s.heightfield = Some(flat_ground());
        s.walker_params = vec![Some(crate::vehicle::walker::WalkerParams {
            spawn_points: ATAT_POINTS
                .iter()
                .map(|&(x, z, a)| (Vec3::new(x, 0.0, z), a.to_radians()))
                .collect(),
            ..crate::vehicle::walker::tests::atst()
        })];
        s.add_vehicle_pad(PadDef {
            name: "vehiclespawn_21".into(),
            pos: pad_pos,
            yaw: pad_yaw,
            cp: None,
            team: Some(0),
            spawn_time: 2.0,
            decay_time: 20.0,
            spawn_count: 1,
            classes: [
                Some(PadClass {
                    landed_height: None,
                    spec: VehicleSpec {
                        kind: VehicleKind::Walker,
                        class: 0,
                        home_team: None,
                        neutral_when_empty: false,
                        spawn_pad: None,
                        pos: Vec3::ZERO,
                        yaw: 0.0,
                        max_health: 60000.0,
                        health_type: HealthType::Vehicle,
                        seats: vec![SeatKind::Pilot],
                    },
                    radius: 3.0,
                }),
                None,
            ],
        });
        s
    }

    /// Card CWSP's test vector (re-command-walker-post §10, BF2 Hoth RAM,
    /// `cp_ram2.py`): the walker at (228.79, −6.00, 51.06) with right
    /// (0.308, 0, 0.952) and forward (−0.952, 0, 0.308) puts point 0
    /// (8, 0, 0) at (231.26, −6.00, 58.68) and point 5 (−8, 0, −4) at
    /// (230.14, −6.00, 42.22); all six are its team's and nobody else's.
    #[test]
    fn the_ram_walker_gives_its_six_points() {
        let mut s = command_walker_sim(Vec3::ZERO, 0.0);
        s.step(&[]);
        let (h, _) = s.command_walkers(0).next().expect("the pad built it");
        let origin = Vec3::new(228.79, -6.00, 51.06);
        let (right, fwd) = (Vec3::new(0.308, 0.0, 0.952), Vec3::new(-0.952, 0.0, 0.308));
        let heading = crate::fmath::atan2(fwd.x, fwd.z);
        {
            let v = s.vehicle_mut(h).unwrap();
            v.pos = origin;
            v.rot = crate::vehicle::hover::quat_y(heading);
            assert!((v.rot * Vec3::X - right).length() < 1e-3, "the RAM frame");
        }
        let got = s.spawn_points(0);
        assert_eq!(got.len(), 6);
        for (i, &(x, z, a)) in ATAT_POINTS.iter().enumerate() {
            // The transform by the RAM rows themselves.
            let want = origin + right * x + fwd * z;
            assert!(
                (got[i].0 - want).length() < 0.01,
                "point {i}: {:?} / {want:?}",
                got[i].0
            );
            assert!((got[i].1 - (heading + a.to_radians())).abs() < 1e-5);
        }
        // The tool's own world points. The rows above are RAM's to three
        // decimals (1.0006 long), so 8 m along them is good to 0.015 m.
        let ram = [
            (0, Vec3::new(231.26, -6.00, 58.68)),
            (5, Vec3::new(230.14, -6.00, 42.22)),
        ];
        for (i, want) in ram {
            assert!(
                (got[i].0 - want).length() < 0.015,
                "RAM point {i}: {:?} / {want:?}",
                got[i].0
            );
        }
        assert_eq!(s.walker_spawn_points(0), got, "no fixed post here");
        assert!(s.spawn_points(1).is_empty(), "never the enemy's");
        // A walker of the other team gives team 0 nothing.
        s.vehicle_mut(h).unwrap().team = Some(1);
        assert!(s.spawn_points(0).is_empty());
        assert_eq!(s.spawn_points(1).len(), 6);
    }

    /// The post lives and dies with its walker (§0.6): destroyed, its points
    /// leave the pool at once; the pad's next walker brings them back. And
    /// before the pad's first walker the opening wave reads the same points
    /// off the pad's pose.
    #[test]
    fn a_dead_walker_gives_none_and_the_rebuilt_one_gives_again() {
        let pad = Vec3::new(40.0, 0.0, -10.0);
        let mut s = command_walker_sim(pad, 0.7);
        assert!(s.spawn_points(0).is_empty(), "no walker yet");
        let opening = s.pad_walker_spawn_points(0);
        assert_eq!(opening.len(), 6);
        assert!(s.pad_walker_spawn_points(1).is_empty());
        s.step(&[]);
        let live = s.spawn_points(0);
        assert_eq!(live.len(), 6);
        for (o, l) in opening.iter().zip(&live) {
            let d = o.0 - l.0;
            assert!(d.x.abs() < 0.2 && d.z.abs() < 0.2, "{o:?} / {l:?}");
            assert!((o.1 - l.1).abs() < 0.02);
        }
        assert!(
            s.pad_walker_spawn_points(0).is_empty(),
            "the pad has built its walker"
        );
        // A respawn lands on one of the six.
        let mut ai = UnitSpec::trooper(0, Vec3::new(0.0, 0.0, -200.0));
        ai.controller = Controller::Ai;
        let u = s.spawn(&ai).unwrap();
        let i = u.index as usize;
        s.units[i].alive = false;
        s.respawn_slot(i);
        let at = s.units[i].soldier.pos;
        assert!(s.units[i].alive);
        assert!(
            live.iter()
                .any(|p| (p.0.x - at.x).abs() < 1e-3 && (p.0.z - at.z).abs() < 1e-3),
            "{at:?}"
        );
        // Kill the walker: the points go with it, and nobody respawns.
        let (h, _) = s.command_walkers(0).next().unwrap();
        s.destroy_vehicle(h, None);
        assert!(s.spawn_points(0).is_empty(), "a dead walker has no post");
        s.units[i].alive = false;
        s.battle.teams[0].alive_ai -= 1;
        s.respawn_slot(i);
        assert!(!s.units[i].alive, "nowhere to spawn");
        // The pad rebuilds it after SpawnTime (2 s here).
        for _ in 0..90 {
            s.step(&[]);
        }
        assert_eq!(s.spawn_points(0).len(), 6, "the new walker's post");
    }

    /// Death → 4 s respawn at a friendly post (new generation), ticket paid;
    /// no friendly post → no respawn.
    #[test]
    fn respawn_at_friendly_cp_on_the_next_wave() {
        let (mut s, _p, t) = duel();
        let cp = s.add_cp(
            sphere_cp("cp2", Vec3::new(60.0, 0.0, -60.0), 8.0, false),
            Some(1),
        );
        let mut death_tick = None;
        for _ in 0..90 {
            s.step(&[fire()]);
            if s.events.iter().any(|e| matches!(e, Event::Kill { .. })) {
                death_tick = Some(s.tick);
                break;
            }
        }
        let d = death_tick.unwrap();
        assert_eq!(s.battle.teams[1].alive(), 0);
        let mut respawn_tick = None;
        for _ in 0..400 {
            s.step(&[PadState::default()]);
            if s.events.iter().any(|e| matches!(e, Event::Respawn { .. })) {
                respawn_tick = Some(s.tick);
                break;
            }
        }
        let r = respawn_tick.expect("respawned");
        // 4 s member timer, then the next wave: wave 1 opens at 10.0 s and
        // slot 0 passes at 10.25 s → tick 308 whatever the death tick.
        assert!(r - d >= 120, "member timer: {} ticks", r - d);
        assert!(
            (307..=309).contains(&r),
            "next-wave slot 0 at ~10.25 s: tick {r}"
        );
        let h2 = s.handle(t.index as usize);
        assert_ne!(h2.gen, t.gen);
        let u = s.unit(h2).unwrap();
        assert!(
            (u.soldier.pos - s.cps[cp].def.pos).length() < 5.0,
            "at the post's node"
        );
        assert_eq!(u.health.cur, 300.0);
        assert_eq!(u.deaths, 1);
        assert_eq!(s.battle.teams[1].alive(), 1);
        assert_eq!(s.battle.teams[1].tickets(), 249);

        // Without a friendly post the dead stay dead.
        let (mut s2, _, t2) = duel();
        for _ in 0..300 {
            s2.step(&[fire()]);
        }
        assert!(s2.unit(t2).is_none());
        assert!(s2.units[t2.index as usize].respawn_at.is_some());
        assert!(s2
            .events
            .iter()
            .all(|e| !matches!(e, Event::Respawn { .. })));
    }

    /// Slot reuse + generation guard: a handle to a dead unit never resolves
    /// to its replacement.
    #[test]
    fn dead_handles_never_alias_new_units() {
        let (mut s, _p, t) = duel();
        for _ in 0..90 {
            s.step(&[fire()]);
            if s.unit(t).is_none() {
                break;
            }
        }
        assert!(s.unit(t).is_none());
        let t2 = s
            .spawn(&UnitSpec::trooper(1, Vec3::new(5.0, 0.0, -30.0)))
            .unwrap();
        assert_eq!(t2.index, t.index, "slot reused");
        assert_ne!(t2.gen, t.gen);
        assert!(s.unit(t).is_none());
        assert!(s.unit(t2).is_some());
    }

    /// A closed box of triangles on the ground: a solid prop as the bake
    /// gives it — walls and a top, nothing inside.
    fn shell(c: Vec3, half: Vec3) -> crate::collision::Body {
        let v = |i: u32| {
            let s = |b: u32| if i & b == 0 { -1.0 } else { 1.0 };
            [
                s(1) * half.x,
                if i & 2 == 0 { 0.0 } else { 2.0 * half.y },
                s(4) * half.z,
            ]
        };
        let quads = [
            [0, 1, 3, 2],
            [4, 6, 7, 5],
            [0, 4, 5, 1],
            [2, 3, 7, 6],
            [0, 2, 6, 4],
            [1, 5, 7, 3],
        ];
        let idx: Vec<u32> = quads
            .iter()
            .flat_map(|q| [q[0], q[1], q[2], q[0], q[2], q[3]])
            .collect();
        let pos: Vec<[f32; 3]> = (0..8).map(v).collect();
        crate::collision::Body::new(
            glam::Mat4::from_translation(c),
            crate::collision::Shape::Mesh(crate::collision::trimesh(&pos, &idx).expect("a mesh")),
        )
    }

    fn wall(c: Vec3, half: Vec3) -> crate::collision::Body {
        crate::collision::Body::new(
            glam::Mat4::from_translation(c),
            crate::collision::Shape::Box { half },
        )
    }

    /// A post at the origin whose one spawn node is `node`.
    fn node_post(s: &mut Sim, node: Vec3) {
        let mut def = sphere_cp("cp1", Vec3::ZERO, 6.0, false);
        def.spawn_nodes = vec![(node, 0.0)];
        s.add_cp(def, Some(0));
    }

    /// Card LLEG: a post's spawn node inside a solid body (BF1 Naboo Plains'
    /// cp3 node in its fallen column, 3 m thick) spawns its unit ON the body,
    /// as the PS2 does (y -104.24 over a node at -108.5), not on the ground
    /// inside the mesh, where it walked the walls until it was given up.
    #[test]
    fn a_spawn_node_inside_a_solid_body_spawns_on_top_of_it() {
        let mut s = Sim::empty(7);
        s.heightfield = Some(flat_ground());
        s.collision
            .push_body(shell(Vec3::new(20.0, 0.0, 0.0), Vec3::new(1.5, 1.5, 4.5)));
        let node = Vec3::new(20.0, 0.0, 3.5);
        // The pen: the ground under the node is inside the walls.
        assert!(s.ground_under(node, false).abs() < 0.05);
        assert!(s.floor_reach(node, PEN_CELLS) < 40, "a pen");
        node_post(&mut s, node);
        assert_eq!(s.spawn_seats.len(), 1);
        assert!((s.spawn_ground(node) - 3.0).abs() < 0.05, "on the body");
        let h = s
            .spawn(&crate::unit::UnitSpec::trooper(0, node))
            .expect("spawned");
        let y = s.unit(h).expect("alive").soldier.pos.y;
        assert!((y - 3.0).abs() < 0.05, "the unit stands on it: {y}");
        // Only a post's own node: a point beside it is seated as before.
        assert!(s.spawn_ground(node + Vec3::X * 0.5).abs() < 0.05);
    }

    /// Card LLEG: a node in a room with a doorway, a node under a bare roof
    /// and a node on a low crate are not penned: each keeps its floor.
    #[test]
    fn a_spawn_node_with_a_way_out_keeps_its_floor() {
        let mut s = Sim::empty(7);
        s.heightfield = Some(flat_ground());
        // A room 6 m x 6 m, walls 3 m, a roof, a 2 m doorway in its +x wall.
        let c = Vec3::new(40.0, 0.0, 0.0);
        let side = |dx: f32, dz: f32, hx: f32, hz: f32| {
            wall(c + Vec3::new(dx, 1.5, dz), Vec3::new(hx, 1.5, hz))
        };
        for b in [
            side(-3.0, 0.0, 0.2, 3.2),
            side(0.0, 3.0, 3.2, 0.2),
            side(0.0, -3.0, 3.2, 0.2),
            side(3.0, 2.0, 0.2, 1.0),
            side(3.0, -2.0, 0.2, 1.0),
            wall(c + Vec3::Y * 3.2, Vec3::new(3.2, 0.2, 3.2)),
            // A bare roof over open ground.
            wall(Vec3::new(60.0, 3.2, 0.0), Vec3::new(3.0, 0.2, 3.0)),
        ] {
            s.collision.push_body(b);
        }
        // A crate 1.2 m high, the node on its top.
        s.collision
            .push_body(shell(Vec3::new(80.0, 0.0, 0.0), Vec3::new(1.0, 0.6, 1.0)));
        assert!(s.floor_reach(c, PEN_CELLS) >= PEN_CELLS, "out by the door");
        for (node, floor) in [
            (c, 0.0),
            (Vec3::new(60.0, 0.0, 0.0), 0.0),
            (Vec3::new(80.0, 1.2, 0.0), 1.2),
        ] {
            node_post(&mut s, node);
            assert!((s.spawn_ground(node) - floor).abs() < 0.05, "{node}");
        }
        assert!(s.spawn_seats.is_empty(), "{:?}", s.spawn_seats);
    }

    /// A sim on flat ground with three team-0 posts of 1, 2 and 6 spawn
    /// nodes (post k's nodes at x = 100 k, z = the node's number).
    fn three_post_sim() -> Sim {
        let mut s = Sim::empty(23);
        s.heightfield = Some(flat_ground());
        for (k, n) in [1usize, 2, 6].into_iter().enumerate() {
            let pos = Vec3::new(100.0 * k as f32, 0.0, 0.0);
            s.add_cp(
                CpDef {
                    name: format!("cp{k}"),
                    pos,
                    capture: Region::sphere(pos, 8.0),
                    control: None,
                    value_bleed: 1,
                    value_atk: [1, 1],
                    value_def: [1, 1],
                    spawn_nodes: (0..n)
                        .map(|j| (pos + Vec3::new(0.0, 0.0, j as f32), 0.0))
                        .collect(),
                    capture_time: 10.0,
                    neutralize_time: 10.0,
                },
                Some(0),
            );
        }
        s
    }

    /// Unit `i` dies and comes back `n` times; the spawns per post (by the
    /// node it stands on) and, for the six-node post, per node.
    fn respawn_shares(s: &mut Sim, i: usize, n: u32) -> ([u32; 3], [u32; 6]) {
        let (mut posts, mut nodes) = ([0u32; 3], [0u32; 6]);
        let is_player = s.units[i].is_player;
        for _ in 0..n {
            s.units[i].alive = false;
            let t = &mut s.battle.teams[0];
            if is_player {
                t.alive_players -= 1;
            } else {
                t.alive_ai -= 1;
            }
            s.respawn_slot(i);
            assert!(s.units[i].alive);
            let at = s.units[i].soldier.pos;
            let k = (at.x / 100.0).round() as usize;
            posts[k] += 1;
            if k == 2 {
                nodes[at.z.round() as usize] += 1;
            }
        }
        (posts, nodes)
    }

    /// Card BSPN (re-command-walker-post §6 [O]): a bot draws a post, then
    /// one of THAT post's nodes — posts of 1, 2 and 6 nodes take a third of
    /// the spawns each (a flat draw over the nine nodes gives 11 / 22 /
    /// 67 %), and the six nodes of the big post a sixth of its spawns each.
    #[test]
    fn a_bot_draws_a_post_then_one_of_its_nodes() {
        let mut s = three_post_sim();
        let mut ai = UnitSpec::trooper(0, Vec3::new(0.0, 0.0, -200.0));
        ai.controller = Controller::Ai;
        let i = s.spawn(&ai).unwrap().index as usize;
        let (posts, nodes) = respawn_shares(&mut s, i, 3000);
        for (k, &n) in posts.iter().enumerate() {
            assert!((900..=1100).contains(&n), "post {k}: {posts:?}");
        }
        for (j, &n) in nodes.iter().enumerate() {
            let want = posts[2] as f32 / 6.0;
            assert!((n as f32 - want).abs() < 0.3 * want, "node {j}: {nodes:?}");
        }
        assert_eq!(s.units[i].spawn_post, None, "a live bot holds no choice");
    }

    /// BF2's `AISpawnWeight`: a post of weight 1000 among two of 100 takes
    /// 1000 / 1200 of the spawns (the disc's one case, KAS2's `baserush`);
    /// a post of weight 0 takes none.
    #[test]
    fn the_post_draw_goes_by_ai_spawn_weight() {
        let mut s = three_post_sim();
        s.cps[0].ai_spawn_weight = 1000.0;
        let mut ai = UnitSpec::trooper(0, Vec3::new(0.0, 0.0, -200.0));
        ai.controller = Controller::Ai;
        let i = s.spawn(&ai).unwrap().index as usize;
        let (posts, _) = respawn_shares(&mut s, i, 3000);
        assert!((2400..=2600).contains(&posts[0]), "{posts:?}");
        assert!(posts[1] > 180 && posts[2] > 180, "{posts:?}");
        s.cps[0].ai_spawn_weight = 0.0;
        let (posts, _) = respawn_shares(&mut s, i, 600);
        assert_eq!(posts[0], 0, "{posts:?}");
        assert!(posts[1] > 240 && posts[2] > 240, "{posts:?}");
    }

    /// The draw is kept from the end of the respawn delay until the spawn,
    /// and made again only when the team can no longer spawn there: the post
    /// captured, or the command walker destroyed.
    #[test]
    fn a_bots_post_is_kept_until_it_stops_being_eligible() {
        let mut s = three_post_sim();
        let mut ai = UnitSpec::trooper(0, Vec3::new(0.0, 0.0, -200.0));
        ai.controller = Controller::Ai;
        let i = s.spawn(&ai).unwrap().index as usize;
        s.units[i].alive = false;
        let first = s.keep_or_draw_spawn_post(i).expect("three posts");
        let lcg = s.lcg.state();
        for _ in 0..50 {
            assert_eq!(s.keep_or_draw_spawn_post(i), Some(first));
        }
        assert_eq!(s.lcg.state(), lcg, "a kept post costs no draw");
        // The choice decides where the next life starts: hashed state.
        let held = s.hash();
        s.units[i].spawn_post = None;
        assert_ne!(s.hash(), held, "the kept post is part of the hash");
        s.units[i].spawn_post = Some(first);
        assert_eq!(s.hash(), held);
        // The enemy takes it: the next ask draws one of the other two.
        let SpawnPost::Post(k) = first else {
            panic!("no walker here")
        };
        s.cps[usize::from(k)].owner = Some(1);
        let second = s.keep_or_draw_spawn_post(i).expect("two posts left");
        assert_ne!(second, first);
        assert!(s.spawn_posts(0).iter().any(|p| p.0 == second));
        // And the spawn is at the kept post's nodes.
        s.battle.teams[0].alive_ai -= 1;
        s.respawn_slot(i);
        let SpawnPost::Post(k2) = second else {
            panic!("no walker here")
        };
        let at = s.units[i].soldier.pos;
        assert_eq!((at.x / 100.0).round() as u16, k2, "{at:?}");

        // A command walker is a post like any other; dead, it is drawn over.
        let mut s = command_walker_sim(Vec3::new(40.0, 0.0, -10.0), 0.7);
        s.step(&[]);
        let (h, _) = s.command_walkers(0).next().expect("the pad built it");
        let i = s.spawn(&ai).unwrap().index as usize;
        s.units[i].alive = false;
        assert_eq!(s.keep_or_draw_spawn_post(i), Some(SpawnPost::Walker(h)));
        assert_eq!(s.spawn_post_nodes(0, SpawnPost::Walker(h)).len(), 6);
        assert!(s.spawn_post_nodes(1, SpawnPost::Walker(h)).is_empty());
        s.destroy_vehicle(h, None);
        assert_eq!(s.keep_or_draw_spawn_post(i), None, "nowhere left");
        assert_eq!(s.units[i].spawn_post, None);
    }

    /// The human is not changed by card BSPN: until the spawn map gives him
    /// the pick (card SPSEL) he draws one node from the flat list of every
    /// post's, so the 1 / 2 / 6-node posts take 1/9, 2/9 and 6/9.
    #[test]
    fn a_human_still_draws_from_the_flat_node_list() {
        let mut s = three_post_sim();
        let me = UnitSpec::player(0, Vec3::new(0.0, 0.0, -200.0));
        let i = s.spawn(&me).unwrap().index as usize;
        let (posts, _) = respawn_shares(&mut s, i, 2700);
        for (k, want) in [300u32, 600, 1800].into_iter().enumerate() {
            let d = posts[k].abs_diff(want);
            assert!(d < want / 5, "post {k}: {posts:?}");
        }
        assert_eq!(s.units[i].spawn_post, None);
    }
}
