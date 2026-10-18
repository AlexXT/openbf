//! Phase C's tuning instrument and acceptance referee (D14): a seeded N-v-N
//! bot battle, run once per difficulty tier, printing one comparable row each.
//!
//! ```text
//! cargo run --release -p game --example arena -- [ticks] [seed=42] [flat|<map_key>] [tier=all]
//! ```
//!
//! `ticks` defaults to 3,000 on the flat field and 9,000 on a map. The target
//! defaults to `flat`; the tier to `all`, which runs the four in order and
//! adds a `monotonic:` verdict line.
//!
//! ## The two fields
//! - **flat** — a bare [`core::sim::Sim`] on a 400 m flat heightfield (the
//!   `core::ai` test rig's shape), NO command posts and NO nav graph, 8 v 8
//!   `UnitSpec::trooper` bots on two lines 30 m apart facing each other. 30 m
//!   is inside every tier's view distance (Easy is the shortest: 0.8 × 49 m).
//!   With no posts there are no spawn points, so it is a ONE-LIFE fight and
//!   the run ends at `ticks` or when a team is wiped out. `omni_radius` 10 m
//!   (the BF2 value) — no manifest to read it from.
//! - **a map key** — `round_to_outcome`'s conquest setup at 16 v 16 with the
//!   idle player, the same map for every tier so the rows compare. The setup
//!   below is COPIED from `round_to_outcome::run_map`, not shared: cargo
//!   examples cannot import each other. `SWEEP_SLOT0=bot` makes slot 0 a bot
//!   here too (the sweep's default; see `slot0_is_bot` for why this example's
//!   default is the other one).
//!
//! ## The row
//! `tier  kills/min  hit_rate  ttfk_s  shots_per_kill  ai_shots  unperceived  hash`
//! folded from `sim.events` tick by tick (`Sim::step` clears them each tick):
//! kills are `Event::Kill` with BOTH ends AI — the idle player's deaths and
//! the AI's hits on it are excluded and printed separately as `player_hits`,
//! because a map row would otherwise mix a stationary free target into the
//! bots' hit rate. `ai_shots` and `unperceived` are `NavStats`' own counters;
//! the event fold's shot count is printed beside them when the two disagree,
//! which would be a bug in one of them.
//!
//! Two runs with the same arguments print byte-identical STDOUT — wall time,
//! the one column that cannot be, goes to STDERR, as the sweep's progress
//! does — and a different seed prints a different fight. The `#[cfg(test)]`
//! module at the bottom pins that in process; `game/Cargo.toml` declares this
//! example with `test = true` so `cargo test` runs it.

use core::sim::{PadState, Sim};
use core::skill::Difficulty;
use core::unit::{Controller, UnitSpec};
use game::walk::WalkSim;
use glam::Vec3;
use std::path::Path;

/// Ticks per second — the fixed sim rate every rate column divides by.
const HZ: f64 = core::TICK_HZ as f64;

/// One tier's row.
struct Row {
    tier: Difficulty,
    kills: u64,
    ai_shots_events: u64,
    ai_hits: u64,
    ttfk: Option<u64>,
    player_hits: u64,
    player_deaths: u64,
    nav_ai_shots: u64,
    unperceived: u64,
    hash: u64,
    ticks: u64,
    wall_s: f32,
    alive: [usize; 2],
}

impl Row {
    fn kills_per_min(&self) -> f64 {
        self.kills as f64 / (self.ticks as f64 / HZ / 60.0)
    }

    fn hit_rate(&self) -> f64 {
        if self.ai_shots_events == 0 {
            0.0
        } else {
            self.ai_hits as f64 / self.ai_shots_events as f64
        }
    }

    fn shots_per_kill(&self) -> f64 {
        if self.kills == 0 {
            f64::INFINITY
        } else {
            self.ai_shots_events as f64 / self.kills as f64
        }
    }

    fn print(&self) {
        let ttfk = match self.ttfk {
            Some(t) => format!("{:8.2}", t as f64 / HZ),
            None => format!("{:>8}", "-"),
        };
        let spk = if self.kills == 0 {
            format!("{:>8}", "-")
        } else {
            format!("{:8.1}", self.shots_per_kill())
        };
        println!(
            "{:<7} {:9.2} {:9.3} {ttfk} {spk} {:8} {:6} {:016x}  \
             ticks {}  alive {}/{}  player_hits {}  player_deaths {}",
            self.tier.label(),
            self.kills_per_min(),
            self.hit_rate(),
            self.nav_ai_shots,
            self.unperceived,
            self.hash,
            self.ticks,
            self.alive[0],
            self.alive[1],
            self.player_hits,
            self.player_deaths,
        );
        if self.ai_shots_events != self.nav_ai_shots {
            println!(
                "        NOTE event-folded AI shots {} != NavStats::ai_shots {}",
                self.ai_shots_events, self.nav_ai_shots
            );
        }
        // The acceptance is a `cmp` of two runs' STDOUT, so the one column
        // that cannot be identical — wall time — goes to stderr, exactly as
        // the sweep puts its progress there.
        eprintln!("arena {} {:.1} s", self.tier.label(), self.wall_s);
    }
}

/// The per-tick event fold both fields share: AI-on-AI kills, AI shots and
/// AI hits, the first kill's tick, and the idle player's side of it kept
/// apart.
struct Fold {
    kills: u64,
    shots: u64,
    hits: u64,
    ttfk: Option<u64>,
    player_hits: u64,
    player_deaths: u64,
}

impl Fold {
    fn new() -> Self {
        Fold {
            kills: 0,
            shots: 0,
            hits: 0,
            ttfk: None,
            player_hits: 0,
            player_deaths: 0,
        }
    }

    /// One tick's events. A handle's `is_player` is read off `sim.units`
    /// directly — the victim of a `Kill` is already dead, so `Sim::unit`
    /// would answer `None`.
    fn eat(&mut self, sim: &Sim) {
        let human = |h: core::unit::UnitHandle| {
            sim.units.get(h.index as usize).is_some_and(|u| u.is_player)
        };
        for e in &sim.events {
            match *e {
                core::sim::Event::Shot { unit, .. } if !human(unit) => self.shots += 1,
                core::sim::Event::Hit {
                    shooter, victim, ..
                } if !human(shooter) => {
                    if human(victim) {
                        self.player_hits += 1;
                    } else {
                        self.hits += 1;
                    }
                }
                core::sim::Event::Kill { shooter, victim } if !human(shooter) => {
                    if human(victim) {
                        self.player_deaths += 1;
                    } else {
                        self.kills += 1;
                        self.ttfk.get_or_insert(sim.tick);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Is the map arm's slot 0 a BOT this run? `SWEEP_SLOT0` (env) is the SAME
/// switch `round_to_outcome` reads, and this example defaults the OTHER way:
/// `bot`/`1` opts in, anything else — and UNSET — keeps the idle player.
/// The published arena tables (PLAN §WHERE THINGS STAND) were baselined with
/// the human in slot 0 and a tier row is only comparable to the rows beside
/// it, so the default cannot move without the lead re-baselining all eight.
/// The flat arena has no player at all and is unaffected either way.
fn slot0_is_bot() -> bool {
    matches!(std::env::var("SWEEP_SLOT0").as_deref(), Ok("bot" | "1"))
}

/// The flat field: `Sim::empty` on 400 m of level ground, no posts, no graph,
/// 8 v 8 troopers on two lines 30 m apart.
fn flat_sim(seed: u32, tier: Difficulty) -> Sim {
    let mut sim = Sim::empty(seed);
    sim.difficulty = tier;
    // No manifest here, so the BF2 value stands (RE §9.1: BF1 maps are the
    // ones with no omni).
    sim.omni_radius = 10.0;
    sim.heightfield = Some(core::heightfield::Heightfield {
        size: 2,
        unit: 400.0,
        offset_x: 200.0,
        offset_z: 200.0,
        heights: vec![0.0; 4],
    });
    // Team 0 at z = +15 looking -Z (yaw 0), team 1 at z = -15 looking +Z.
    for i in 0..8u32 {
        let x = (i as f32 - 3.5) * 2.0;
        for (team, z, yaw) in [(0u8, 15.0, 0.0), (1u8, -15.0, std::f32::consts::PI)] {
            sim.spawn(&UnitSpec {
                controller: Controller::Ai,
                yaw,
                ..UnitSpec::trooper(team, Vec3::new(x, 0.0, z))
            })
            .expect("the arena's 8 v 8 fits in the default ticket pool");
        }
    }
    sim
}

fn run_flat(ticks: u64, seed: u32, tier: Difficulty) -> Row {
    let started = std::time::Instant::now();
    let mut sim = flat_sim(seed, tier);
    let mut f = Fold::new();
    let idle = [PadState::default()];
    let mut t = 0u64;
    while t < ticks {
        sim.step(&idle);
        f.eat(&sim);
        t += 1;
        // One life each: the round is over when a side is wiped out.
        let (a, b) = alive_counts(&sim);
        if a == 0 || b == 0 {
            break;
        }
    }
    row(tier, &sim, f, t, started)
}

fn alive_counts(sim: &Sim) -> (usize, usize) {
    let mut c = (0, 0);
    for (_, u) in sim.alive_units() {
        if u.team == 0 {
            c.0 += 1;
        } else {
            c.1 += 1;
        }
    }
    c
}

fn row(tier: Difficulty, sim: &Sim, f: Fold, ticks: u64, started: std::time::Instant) -> Row {
    let (a, b) = alive_counts(sim);
    Row {
        tier,
        kills: f.kills,
        ai_shots_events: f.shots,
        ai_hits: f.hits,
        ttfk: f.ttfk,
        player_hits: f.player_hits,
        player_deaths: f.player_deaths,
        nav_ai_shots: sim.nav_stats.ai_shots,
        unperceived: sim.nav_stats.unperceived_shots,
        hash: sim.hash(),
        ticks: ticks.max(1),
        wall_s: started.elapsed().as_secs_f32(),
        alive: [a, b],
    }
}

/// `missions/<key>_con.ron`, when the generator wrote one for this map.
fn mission_for(key: &str) -> Option<game::mission::MissionDef> {
    let dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../missions"));
    let path = dir.join(format!("{key}_con.ron"));
    path.exists()
        .then(|| game::mission::MissionDef::load(&path).ok())
        .flatten()
}

/// The map field. This is `round_to_outcome::run_map`'s setup, copied (an
/// example cannot import another example): mode sublevel, shared props, the
/// mission's tickets/factions/class mix, the nav graph, model collision, then
/// `setup_conquest` at 16 v 16 with the idle player.
fn run_map(key: &str, path: &Path, ticks: u64, seed: u32, tier: Difficulty) -> Option<Row> {
    let started = std::time::Instant::now();
    let mut scene = match game::assets::load_scene(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{key}: LOAD FAILED {e:#}");
            return None;
        }
    };
    scene.apply_mode_layer("con");
    let common_dir = path
        .parent()
        .map(|p| p.join(format!("common_{}", scene.manifest.source_game)));
    if let Some(cd) = common_dir.filter(|d| d.join("common.ron").exists()) {
        match game::assets::load_common(&cd) {
            Ok(c) => {
                scene.merge_common(c);
            }
            Err(e) => eprintln!("{key}: common assets unreadable: {e:#}"),
        }
    }

    let mission = mission_for(key);
    let mut layout = game::layout::MapLayout::from_manifest(&scene.manifest);
    let applied = mission.as_ref().map(|m| layout.apply_mission(m));

    let mut sim = Sim::empty(seed);
    sim.difficulty = tier;
    sim.omni_radius = core::sim::omni_radius_for(&scene.manifest.source_game);
    sim.water_level = scene.manifest.water.as_ref().map(|w| w.height);
    sim.ground_rules = core::soldier::GroundRules::for_game(&scene.manifest.source_game);
    sim.item_pool = game::roster::item_pool(&scene.manifest.source_game, &scene.manifest.map);
    if let Some(m) = &mission {
        sim.battle = core::battle::Battle::new(m.tickets());
    }
    let factions = mission
        .as_ref()
        .map(game::mission::MissionDef::factions)
        .unwrap_or_else(|| {
            game::roster::default_factions(&scene.manifest.source_game).map(str::to_owned)
        });
    let classes_dir = path
        .parent()
        .map(|p| p.join("classes"))
        .unwrap_or_else(|| std::path::PathBuf::from("classes"));
    let rosters = game::roster::FactionRoster::pair(
        &classes_dir,
        &scene.manifest.source_game,
        [factions[0].as_str(), factions[1].as_str()],
        mission.as_ref(),
    );
    if let Ok((_, warnings)) = &rosters {
        for w in warnings {
            eprintln!("{w}");
        }
    }
    let mut tables: [Vec<core::unit::UnitClass>; 2] = [Vec::new(), Vec::new()];
    let teams = match &rosters {
        Ok((rs, _)) => {
            // One weapon index per WEAPON NAME, as the sweep does.
            let mut names: Vec<(String, usize)> = Vec::new();
            let mut t = 0usize;
            rs.each_ref().map(|r| {
                for c in &r.classes {
                    let uc = c.register(&mut sim, &mut names);
                    tables[t].push(uc);
                }
                let f = r.fill_class();
                t += 1;
                game::walk::TeamTemplate {
                    faction: r.faction.clone(),
                    name: f.class.clone(),
                    consts: f.consts,
                    max_health: f.max_health,
                    health_type: f.health_type,
                    weapon_class: tables[t - 1][0].weapon_class,
                    weapon_slot: tables[t - 1][0].weapon_slot,
                    slots: tables[t - 1][0].slots.clone(),
                    drops: tables[t - 1][0].drops,
                    spare_clips: f.spare_clips,
                    geometry: f.geometry.clone(),
                    weapon_geometry: f.weapon_geometry.clone(),
                    classes: r
                        .classes
                        .iter()
                        .map(|c| game::walk::ClassVisual {
                            name: c.class.clone(),
                            label: c.label.clone(),
                            geometry: c.geometry.clone(),
                            weapon_geometry: c.weapon_geometry.clone(),
                            weapon_hud: c.weapon_hud.clone(),
                            held_hud: Vec::new(),
                            foley_class: c.foley_class,
                            cries: c.cries.clone(),
                            anim: c.anim.clone(),
                        })
                        .collect(),
                }
            })
        }
        Err(e) => {
            eprintln!("{key}: rosters unavailable: {e:#} (built-in troopers)");
            let t = game::walk::TeamTemplate::default_trooper();
            factions.map(|faction| game::walk::TeamTemplate {
                faction,
                ..t.clone()
            })
        }
    };
    let nav = game::layout::nav_graph(&scene.manifest, &scene.dir);
    let mut w = WalkSim::with_teams(sim, scene.heightfield(), Vec3::ZERO, teams, nav);
    w.sim.classes = tables;
    scene.install_collision(&mut w.sim);
    if let Some(a) = &applied {
        w.sim.death_regions = a.death_regions.clone();
    }
    w.setup_conquest(&layout, [16, 16], true);
    // Slot 0 as a bot, opt-in (see `slot0_is_bot`): the same un-spawn-and-refill
    // `round_to_outcome` does by default, so the two referees can be run on the
    // same field. OFF, the idle player stands there and its hits and deaths are
    // the row's `player_hits`/`player_deaths` columns as they always were.
    if slot0_is_bot() {
        if let Some((pos, yaw)) = w.sim.vacate_player_slot(0) {
            if w.sim.classes[0].is_empty() {
                let spec = w.teams[0].spec(0, pos, yaw, Controller::Ai, 0);
                w.sim.spawn(&spec);
            } else {
                w.sim.spawn_ai(0, pos, yaw);
            }
        }
    }

    let mut f = Fold::new();
    let idle = [PadState::default()];
    let mut t = 0u64;
    while t < ticks && w.sim.outcome.is_none() {
        w.step_tick(&idle);
        f.eat(&w.sim);
        t += 1;
    }
    Some(row(tier, &w.sim, f, t, started))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ticks_arg: Option<u64> = args.first().and_then(|s| s.parse().ok());
    let seed: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(42);
    let target = args.get(2).cloned().unwrap_or_else(|| "flat".to_owned());
    let tier_arg = args.get(3).cloned().unwrap_or_else(|| "all".to_owned());
    let tiers: Vec<Difficulty> = if tier_arg == "all" {
        Difficulty::ALL.to_vec()
    } else {
        match Difficulty::parse(&tier_arg) {
            Some(d) => vec![d],
            None => {
                eprintln!("arena: unknown tier {tier_arg:?} (easy|normal|hard|elite|all)");
                return;
            }
        }
    };
    let flat = target == "flat";
    let ticks = ticks_arg.unwrap_or(if flat { 3_000 } else { 9_000 });

    let map: Option<(String, std::path::PathBuf)> = if flat {
        None
    } else {
        let dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets_baked"));
        match game::assets::discover_maps(dir)
            .into_iter()
            .find(|(key, _, _)| *key == target)
        {
            Some((key, _, path)) => Some((key, path)),
            None => {
                eprintln!("arena: no baked map {target:?} under {}", dir.display());
                return;
            }
        }
    };

    println!(
        "arena: {} ticks {ticks} seed {seed}",
        map.as_ref().map_or("flat 8v8", |(k, _)| k.as_str())
    );
    println!(
        "{:<7} {:>9} {:>9} {:>8} {:>8} {:>8} {:>6} {:>16}",
        "tier", "kills/min", "hit_rate", "ttfk_s", "sh/kill", "ai_shots", "unperc", "hash"
    );
    let mut rows: Vec<Row> = Vec::new();
    for tier in tiers {
        let r = match &map {
            None => run_flat(ticks, seed, tier),
            Some((key, path)) => match run_map(key, path, ticks, seed, tier) {
                Some(r) => r,
                None => return,
            },
        };
        r.print();
        rows.push(r);
    }
    if rows.len() == 4 {
        let mono = |v: Vec<f64>| v.windows(2).all(|w| w[0] < w[1]);
        let kills = mono(rows.iter().map(Row::kills_per_min).collect());
        let hits = mono(rows.iter().map(Row::hit_rate).collect());
        println!(
            "monotonic: kills {}  hits {}",
            if kills { "yes" } else { "no" },
            if hits { "yes" } else { "no" }
        );
    }
    let bad: u64 = rows.iter().map(|r| r.unperceived).sum();
    if bad > 0 {
        println!("PERCEPTION ASSERTION FAILED: {bad} unperceived AI shots");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D14's acceptance: two in-process runs of the same arena are the same
    /// run, and the seed is what makes them differ. `game/Cargo.toml` sets
    /// `test = true` on this example so `cargo test` runs this against the
    /// example's OWN rig rather than a copy of it.
    #[test]
    fn the_flat_arena_is_deterministic_and_seeded() {
        let a = run_flat(600, 42, Difficulty::Normal);
        let b = run_flat(600, 42, Difficulty::Normal);
        assert_eq!(a.hash, b.hash, "same seed, same final state");
        assert_eq!(
            (a.kills, a.ai_hits, a.ai_shots_events),
            (b.kills, b.ai_hits, b.ai_shots_events)
        );
        assert_eq!((a.ticks, a.ttfk, a.alive), (b.ticks, b.ttfk, b.alive));
        assert_eq!(
            a.unperceived, 0,
            "the perception assertion holds in the arena"
        );

        let c = run_flat(600, 43, Difficulty::Normal);
        assert_ne!(a.hash, c.hash, "a different seed is a different fight");

        // The tier reaches the bots: Elite's fight is not Normal's.
        let e = run_flat(600, 42, Difficulty::Elite);
        assert_ne!(a.hash, e.hash);
        assert_eq!(e.unperceived, 0);
    }
}
