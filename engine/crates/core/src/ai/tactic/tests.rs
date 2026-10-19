//! Card TPOS: the position table's and the Tactician's tests (out of
//! line: `tactic.rs` is under the 2,000-line cap).

use super::*;
use crate::ai::FightState;
use crate::sim::PadState;
use crate::unit::{Controller, UnitHandle, UnitSpec};

/// The L graph's sim (`ai::tests::l_sim`) with a crate 1 m high across
/// the corridor 3 m south of hub 0 - LOW cover for hub 0's centre and
/// its east and west points toward the south - the table built, and an
/// enemy (idle, kept alive) at `enemy`. The bot stands at `bot`.
fn crate_scene(bot: Vec3, enemy: Vec3) -> (Sim, UnitHandle, UnitHandle) {
    use crate::collision::{Body, Shape};
    let (mut sim, h) = super::super::tests::l_sim();
    sim.collision.push_body(Body::new(
        glam::Mat4::from_translation(Vec3::new(0.0, 0.5, -3.0)),
        Shape::Box {
            half: Vec3::new(1.5, 0.5, 0.3),
        },
    ));
    sim.build_positions();
    let e = sim.spawn(&UnitSpec::trooper(1, enemy)).expect("enemy");
    let u = sim.unit_mut(h).expect("bot");
    u.soldier.pos = bot;
    u.soldier.yaw = crate::ai::yaw_toward(enemy - bot);
    (sim, h, e)
}

fn run(sim: &mut Sim, e: UnitHandle, ticks: u32, mut each: impl FnMut(&Sim)) {
    for _ in 0..ticks {
        sim.step(&[PadState::default()]);
        sim.unit_mut(e).expect("enemy").health.cur = 300.0;
        each(sim);
    }
}

/// Two hubs 12 m apart on flat ground, A (0, 0) with the crate 3 m
/// south of it and B (0, 12) with the post (owned by the enemy, so the
/// bot's objective); the bot at B, an enemy 25 m south of A.
fn two_hub_scene() -> (Sim, UnitHandle, UnitHandle) {
    use crate::collision::{Body, Shape};
    use crate::nav::{Arc, Hub, NavGraph, RouteTable};
    let (mut sim, h) = super::super::tests::l_sim();
    let hub = |z: f32, arcs: Vec<u16>| Hub {
        pos: Vec3::new(0.0, 0.0, z),
        radius: 2.0,
        arcs,
    };
    let n = 2usize;
    let rows = vec![Vec::new(); crate::nav::TYPE_COUNT * n];
    sim.nav = Some(
        NavGraph::new(
            vec![hub(0.0, vec![0]), hub(12.0, vec![0])],
            vec![Arc {
                a: 0,
                b: 1,
                type_mask: 1,
                flags: 0,
            }],
            RouteTable::from_rows(n, crate::nav::TYPE_COUNT, rows),
            Vec::new(),
            Vec::new(),
        )
        .expect("graph"),
    );
    let post = Vec3::new(0.0, 0.0, 12.0);
    sim.cps[0].def.pos = post;
    sim.cps[0].def.capture = crate::region::Region::sphere(post, 3.0);
    sim.cps[0].nav_hub = Some(1);
    sim.resolve_nav_floors();
    sim.collision.push_body(Body::new(
        glam::Mat4::from_translation(Vec3::new(0.0, 0.5, -3.0)),
        Shape::Box {
            half: Vec3::new(1.5, 0.5, 0.3),
        },
    ));
    sim.build_positions();
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let e = sim.spawn(&UnitSpec::trooper(1, enemy)).expect("enemy");
    let u = sim.unit_mut(h).expect("bot");
    u.soldier.pos = post;
    u.soldier.yaw = crate::ai::yaw_toward(enemy - post);
    (sim, h, e)
}

/// A bot fighting a target across a crate takes a position with cover
/// from it - the crate's LOW cover at hub A, 12 m off, not one of the
/// open points of hub B it stands in - reserved for it, with a line to
/// the target from its firing point; it walks there and stands in its
/// cycle.
#[test]
fn a_bot_takes_the_covered_position_across_a_crate() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = two_hub_scene();
    assert!(sim.positions.positions.len() >= 18, "two hubs' points");
    assert!(
        sim.positions.positions.iter().any(|q| q.low != 0),
        "the crate is cover"
    );
    run(&mut sim, e, 240, |_| {});
    let u = sim.unit(h).expect("bot");
    assert_eq!(u.ai.fight.mode, FightState::COVER, "at its position");
    let i = u.ai.tpos.held.expect("a position");
    let q = sim.positions.positions[i as usize];
    let d = Vec2::new(enemy.x - q.pos.x, enemy.z - q.pos.z);
    assert!(
        u.ai.tpos.covered && q.low & sector_pair(d) != 0,
        "LOW cover from the target"
    );
    assert!(u.ai.tpos.until > sim.tick, "reserved");
    assert!(
        nav::xz_dist(u.soldier.pos, q.pos) < 1.0,
        "it stands there: {} at {} for {}",
        u.soldier.pos,
        q.pos,
        sim.tick
    );
    let eye = firing_eye(q.pos, enemy, 0);
    let chest = sim.unit(e).expect("enemy").muzzle();
    assert!(line_open(&sim, eye, chest), "a line to the target");
    // The cycle: hidden (crouched) and firing (standing) in turn.
    let mut phases = std::collections::BTreeSet::new();
    run(&mut sim, e, 300, |s| {
        let u = s.unit(h).expect("bot");
        if u.ai.tpos.held == Some(i) {
            phases.insert(u.ai.tpos.firing);
        }
    });
    assert_eq!(phases.len(), 2, "it hides and fires in turn");
}

/// Two bots fighting from one hub never hold positions within
/// [`SPACING`] of each other - and both get one.
#[test]
fn two_bots_never_hold_positions_within_two_metres() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = crate_scene(Vec3::new(0.3, 0.0, 0.3), enemy);
    let h2 = sim
        .spawn(&UnitSpec {
            controller: Controller::Ai,
            ..UnitSpec::trooper(0, Vec3::new(-0.3, 0.0, 0.5))
        })
        .expect("second bot");
    let (mut both, mut close) = (0u32, Vec::new());
    run(&mut sim, e, 300, |s| {
        let at = |h: UnitHandle| {
            let u = s.unit(h)?;
            u.ai.tpos
                .held
                .map(|i| s.positions.positions[i as usize].pos)
        };
        if let (Some(a), Some(b)) = (at(h), at(h2)) {
            both += 1;
            if nav::xz_dist(a, b) < SPACING {
                close.push(s.tick);
            }
        }
    });
    assert!(both > 100, "both held positions: {both} ticks");
    assert!(close.is_empty(), "within {SPACING} m at ticks {close:?}");
}

/// Teammates hold positions a grenade's blast apart ([`SPREAD`]) where
/// the place allows: hub 0 grown to 8 m (its ring points 5.6 m out), two
/// bots starting side by side.
#[test]
fn teammates_hold_positions_a_blast_apart_where_the_place_allows() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = crate_scene(Vec3::new(0.3, 0.0, 0.3), enemy);
    if let Some(g) = sim.nav.as_mut() {
        g.hubs[0].radius = 8.0;
    }
    sim.build_positions();
    let h2 = sim
        .spawn(&UnitSpec {
            controller: Controller::Ai,
            ..UnitSpec::trooper(0, Vec3::new(-0.3, 0.0, 0.5))
        })
        .expect("second bot");
    let mut apart = Vec::new();
    run(&mut sim, e, 120, |s| {
        let at = |h: UnitHandle| {
            let u = s.unit(h)?;
            u.ai.tpos
                .held
                .map(|i| s.positions.positions[i as usize].pos)
        };
        if let (Some(a), Some(b)) = (at(h), at(h2)) {
            apart.push(nav::xz_dist(a, b));
        }
    });
    assert!(
        apart.len() > 60,
        "both held positions: {} ticks",
        apart.len()
    );
    let near = apart.iter().filter(|d| **d < SPREAD).count();
    assert_eq!(near, 0, "closer than {SPREAD} m on {near} ticks");
}

/// The endgame (a side down to its last reinforcements): no positions,
/// the bot's own fight - and a position held is left.
#[test]
fn nobody_takes_a_position_in_the_endgame() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = crate_scene(Vec3::ZERO, enemy);
    run(&mut sim, e, 60, |_| {});
    assert!(
        sim.unit(h).and_then(|u| u.ai.tpos.held).is_some(),
        "a position first"
    );
    sim.battle.teams[1].reinforcements = crate::ai::HOLD_ENDGAME as f32;
    run(&mut sim, e, 60, |s| {
        if s.tick > 75 {
            let u = s.unit(h).expect("bot");
            assert!(
                u.ai.tpos.held.is_none() && !u.ai.fight.tactician(),
                "tick {}",
                s.tick
            );
        }
    });
}

/// A position where the bot is hit twice is left for another.
#[test]
fn a_position_hit_twice_is_left() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = crate_scene(Vec3::ZERO, enemy);
    run(&mut sim, e, 60, |_| {});
    let first = sim
        .unit(h)
        .and_then(|u| u.ai.tpos.held)
        .expect("a position");
    assert_eq!(sim.unit(h).expect("bot").ai.fight.mode, FightState::COVER);
    // (Not the 5-12 s wait for another: only the hits.)
    sim.unit_mut(h).expect("bot").ai.tpos.next_at = u64::MAX;
    // Two hits, a burst apart (the second inside the cooldown would be
    // the same burst).
    for _ in 0..2 {
        let t = sim.tick;
        sim.unit_mut(h).expect("bot").last_hit = Some((e, t - 1));
        run(&mut sim, e, (REACT_GAP) as u32, |_| {});
    }
    run(&mut sim, e, 30, |_| {});
    let u = sim.unit(h).expect("bot");
    assert_ne!(u.ai.tpos.held, Some(first), "it left the position");
    assert_eq!(
        u.ai.tpos.prev,
        Some(first),
        "...which stays reserved a while"
    );
}
const REACT_GAP: u64 = crate::ai::REACT_COOLDOWN + 1;

/// No position passes (every one is beyond the weapon's band from a
/// target off to the east): the bot fights by its own style - the PS2's
/// band move - and asks again after [`RETRY`].
#[test]
fn no_winner_is_the_bots_own_fight() {
    let enemy = Vec3::new(47.0, 0.0, 0.0);
    let (mut sim, h, e) = crate_scene(Vec3::ZERO, enemy);
    let mut asked = false;
    run(&mut sim, e, 90, |s| {
        let u = s.unit(h).expect("bot");
        asked |= u.ai.tpos.retry_at > 0;
        assert!(u.ai.tpos.held.is_none(), "no position");
    });
    let u = sim.unit(h).expect("bot");
    assert!(asked, "the Tactician was asked");
    assert_eq!(u.ai.behaviour, crate::ai::Behaviour::Fight);
    assert!(
        matches!(
            u.ai.fight.mode,
            FightState::STAND | FightState::WALK | FightState::PAUSE
        ),
        "its own style: mode {}",
        u.ai.fight.mode
    );
}

/// Another position, when the 5-12 s wait sends the bot on, is at least
/// [`MOVE_MIN`] from the one it leaves - or it stays (the maintainer's Kamino
/// scene: hops of 0.6-1.5 m between neighbouring points).
#[test]
fn a_new_position_is_a_move_not_a_twitch() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = crate_scene(Vec3::ZERO, enemy);
    let (mut held, mut moves, mut asked) = (None::<u32>, Vec::new(), 0u32);
    for _ in 0..12 {
        // The wait is over at once, every time.
        if let Some(u) = sim.unit_mut(h) {
            if u.ai.fight.mode == FightState::COVER {
                u.ai.tpos.next_at = 0;
                asked += 1;
            }
        }
        run(&mut sim, e, 30, |s| {
            let u = s.unit(h).expect("bot");
            if let (Some(a), Some(b)) = (held, u.ai.tpos.held) {
                if a != b {
                    let (p, q) = (
                        s.positions.positions[a as usize],
                        s.positions.positions[b as usize],
                    );
                    moves.push(nav::xz_dist(p.pos, q.pos));
                }
            }
            held = u.ai.tpos.held.or(held);
        });
    }
    assert!(asked >= 6, "the wait ran out at a position {asked} times");
    assert!(moves.iter().all(|d| *d >= MOVE_MIN), "moves of {moves:?} m");
}

/// A walk to its position that ends short of it - here its deadline, the
/// bot 1.3 m off - is arrived where the bot stands when within
/// [`NEAR_ENOUGH`]: the position kept, its cycle begun there.
#[test]
fn a_walk_cut_short_near_its_position_is_there() {
    let (mut sim, h, e) = two_hub_scene();
    let mut i = None;
    run(&mut sim, e, 120, |s| {
        let u = s.unit(h).expect("bot");
        if i.is_none() && u.ai.fight.mode == FightState::TAKE {
            i = u.ai.tpos.held;
        }
    });
    let i = i.expect("it walked to a position");
    let q = sim.positions.positions[i as usize].pos;
    let t = sim.tick;
    let u = sim.unit_mut(h).expect("bot");
    assert_eq!(u.ai.tpos.held, Some(i), "still on its way");
    u.ai.fight.mode = FightState::TAKE;
    u.soldier.pos = q + Vec3::new(1.3, 0.0, 0.0);
    u.soldier.vel = Vec3::ZERO;
    u.ai.tpos.phase_until = t - 1;
    u.ai.next_think = t;
    let mut seen = None;
    run(&mut sim, e, 30, |s| {
        let u = s.unit(h).expect("bot");
        if seen.is_none() && u.ai.fight.mode != FightState::TAKE {
            seen = Some((u.ai.fight.mode, u.ai.tpos.held, u.ai.tpos.stand));
        }
    });
    let (mode, held, stand) = seen.expect("the walk ended");
    assert_eq!((mode, held), (FightState::COVER, Some(i)), "there");
    let d = nav::xz_dist(stand, q);
    assert!(
        d > ARRIVE_NEAR && d <= NEAR_ENOUGH,
        "where it stood: {d} m off"
    );
}

/// A walk past its deadline far from its position fails: the position
/// is left, and this bot never asks for it again.
#[test]
fn a_walk_past_its_deadline_far_off_is_left_for_good() {
    let (mut sim, h, e) = two_hub_scene();
    let mut i = None;
    run(&mut sim, e, 120, |s| {
        let u = s.unit(h).expect("bot");
        if i.is_none() && u.ai.fight.mode == FightState::TAKE {
            i = u.ai.tpos.held;
        }
    });
    let i = i.expect("it walked to a position");
    let q = sim.positions.positions[i as usize].pos;
    let t = sim.tick;
    let u = sim.unit_mut(h).expect("bot");
    u.ai.fight.mode = FightState::TAKE;
    u.soldier.pos = q + Vec3::new(0.0, 0.0, 5.0);
    u.ai.tpos.phase_until = t - 1;
    u.ai.next_think = t;
    let mut again = Vec::new();
    let mut left = false;
    run(&mut sim, e, 300, |s| {
        let u = s.unit(h).expect("bot");
        left |= u.ai.tpos.held != Some(i);
        if left && u.ai.tpos.held == Some(i) {
            again.push(s.tick);
        }
    });
    assert!(left, "the position was left");
    assert!(again.is_empty(), "taken again at ticks {again:?}");
    assert!(sim.unit(h).expect("bot").ai.tpos.bad.contains(&Some(i)));
}

/// A position whose straight way from the bot crosses a soldier barrier
/// is not chosen: the crate's covered points at hub A lie behind a
/// barrier across the corridor, so the bot at B holds none of them
/// (one behind a barrier was walked at and failed again and again,
/// The maintainer's Kamino scene).
#[test]
fn a_position_behind_a_barrier_is_not_chosen() {
    let (mut sim, h, e) = two_hub_scene();
    if let Some(g) = sim.nav.as_mut() {
        g.barriers.push(nav::Barrier {
            centre: Vec2::new(0.0, 6.0),
            axes: [Vec2::X, Vec2::Y],
            half: Vec2::new(6.0, 0.3),
            mask: 1 << nav::SOLDIER,
            enabled: true,
        });
    }
    let mut behind = Vec::new();
    run(&mut sim, e, 240, |s| {
        let u = s.unit(h).expect("bot");
        if let Some(i) =
            u.ai.tpos
                .held
                .filter(|i| s.positions.positions[*i as usize].pos.z < 6.0)
        {
            behind.push((s.tick, i));
        }
    });
    // (Without the barrier it takes the crate's cover:
    // `a_bot_takes_the_covered_position_across_a_crate`.)
    assert!(behind.is_empty(), "held behind the barrier: {behind:?}");
}

/// A Cover hint hides by its arrow alone, the PS2's rule
/// (re-soldier-ai 4.6): toward a target within 60 degrees of it, not
/// beyond; any other point only by what is shut round it.
#[test]
fn a_cover_hint_hides_by_its_arrow() {
    let mut q = Position {
        pos: Vec3::ZERO,
        source: Source::Cover,
        facing: Vec2::X,
        stance: 0,
        stance2: 0,
        hub: 0,
        low: 0,
        corner: 0,
        shut: 0,
        tall: 0,
    };
    assert!(q.hides_toward(Vec2::new(10.0, 5.0)), "27 degrees off");
    assert!(!q.hides_toward(Vec2::new(10.0, 20.0)), "63 degrees off");
    assert!(!q.hides_toward(Vec2::new(-10.0, 0.0)), "behind it");
    q.source = Source::Edge;
    assert!(!q.hides_toward(Vec2::X), "an edge point has no arrow");
    q.shut = sector_bit(Vec2::X);
    assert!(q.hides_toward(Vec2::X), "...but its shut sector");
}

/// The positions of hub A (the crate's) with cover toward the south.
fn covered_at_a(sim: &Sim) -> Vec<u32> {
    let d = Vec2::new(0.0, -1.0);
    (0..sim.positions.positions.len() as u32)
        .filter(|&i| {
            let q = &sim.positions.positions[i as usize];
            q.pos.z < 2.0 && q.shut & sector_pair(d) != 0
        })
        .collect()
}

/// A covered position is a candidate however many open ones lie nearer:
/// with 40 more open points round hub B, the bot there still takes the
/// crate's cover at A, 12 m off (the 32 nearest of every kind held 5.0
/// of Mygeeto's 42.6 covered candidates a query).
#[test]
fn a_covered_position_is_a_candidate_past_the_nearest_open_ones() {
    let (mut sim, h, e) = two_hub_scene();
    let mut all = sim.positions.positions.clone();
    let stats = sim.positions.stats;
    for k in 0..40 {
        let a = k as f32 / 40.0 * std::f32::consts::TAU;
        let r = 2.0 + (k % 5) as f32;
        let mut q = all[all.len() - 1];
        q.pos = Vec3::new(r * a.cos(), 0.0, 12.0 + r * a.sin());
        q.source = Source::Hub;
        (q.low, q.corner, q.shut, q.tall) = (0, 0, 0, 0);
        all.push(q);
    }
    sim.positions = PositionTable::index(all, stats);
    let mut covered = false;
    run(&mut sim, e, 240, |s| {
        let u = s.unit(h).expect("bot");
        covered |= u.ai.tpos.held.is_some() && u.ai.tpos.covered;
    });
    assert!(covered, "it took a covered position");
}

/// A covered position farther than [`COVER_REACH`] is not walked to: a
/// bot 17-18 m from the crate's points takes none of them.
#[test]
fn a_covered_position_beyond_reach_is_not_taken() {
    let (mut sim, h, e) = two_hub_scene();
    sim.unit_mut(e).expect("enemy").soldier.pos = Vec3::new(0.0, 0.0, -15.0);
    let cov = covered_at_a(&sim);
    assert!(!cov.is_empty());
    sim.unit_mut(h).expect("bot").soldier.pos = Vec3::new(0.0, 0.0, 17.0);
    let (mut far_picks, mut bad, mut was) = (0, Vec::new(), None);
    run(&mut sim, e, 240, |s| {
        let u = s.unit(h).expect("bot");
        let fresh = u.ai.tpos.held.is_some() && u.ai.tpos.held != was;
        was = u.ai.tpos.held;
        if !fresh {
            return;
        }
        let near = cov
            .iter()
            .map(|&i| nav::xz_dist(s.positions.positions[i as usize].pos, u.soldier.pos))
            .fold(f32::MAX, f32::min);
        if near > COVER_REACH {
            far_picks += 1;
            if u.ai.tpos.covered {
                bad.push(s.tick);
            }
        }
    });
    assert!(far_picks > 0, "it chose from beyond the reach");
    assert!(bad.is_empty(), "covered picks out of reach at {bad:?}");
}

/// The target gone round the cover leaves the position at its re-score,
/// every [`RESCORE`] (the PS2's 5 s), not at the next think.
#[test]
fn a_position_is_rescored_every_five_seconds() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = crate_scene(Vec3::new(0.3, 0.0, 0.3), enemy);
    run(&mut sim, e, 400, |_| {});
    let u = sim.unit_mut(h).expect("bot");
    assert_eq!(u.ai.fight.mode, FightState::COVER, "at its position");
    assert!(u.ai.tpos.covered);
    let i = u.ai.tpos.held.expect("a position");
    u.ai.tpos.next_at = u64::MAX / 2;
    let r0 = u.ai.tpos.rescore_at;
    let t0 = sim.tick;
    assert!(r0 > t0 + 30, "the re-score is ahead: {r0} at {t0}");
    sim.unit_mut(e).expect("enemy").soldier.pos = Vec3::new(0.0, 0.0, 25.0);
    let mut left = None;
    run(&mut sim, e, (r0 - t0) as u32 + 60, |s| {
        let u = s.unit(h).expect("bot");
        if left.is_none() && u.ai.tpos.held != Some(i) {
            left = Some(s.tick);
        }
    });
    let left = left.expect("left once round");
    assert!(left >= r0, "left at {left}, before its re-score at {r0}");
}

/// A bot at an open position takes a covered one when it comes free,
/// within the 2 s of its next ask, not at the 5-12 s timer: the crate's
/// points at A are reserved by an idle teammate, the bot holds an open
/// point at B; the reservation goes.
#[test]
fn an_open_position_gives_way_to_cover_when_it_comes_free() {
    let (mut sim, h, e) = two_hub_scene();
    let cov = covered_at_a(&sim);
    let centre = *cov
        .iter()
        .min_by(|a, b| {
            let p = |i: u32| sim.positions.positions[i as usize].pos.length();
            p(**a).total_cmp(&p(**b))
        })
        .expect("a covered point");
    let m = sim
        .spawn(&UnitSpec::trooper(0, Vec3::new(30.0, 0.0, 30.0)))
        .expect("mate");
    let hold = |s: &mut Sim, on: bool| {
        let t = s.tick;
        let mu = s.unit_mut(m).expect("mate");
        mu.ai.tpos.held = on.then_some(centre);
        mu.ai.tpos.until = if on { t + 1000 } else { 0 };
    };
    for _ in 0..240 {
        hold(&mut sim, true);
        run(&mut sim, e, 1, |_| {});
    }
    let u = sim.unit_mut(h).expect("bot");
    assert_eq!(u.ai.fight.mode, FightState::COVER, "at a position");
    assert!(!u.ai.tpos.covered, "an open one");
    u.ai.tpos.next_at = u64::MAX / 2;
    hold(&mut sim, false);
    let mut covered = false;
    run(&mut sim, e, 90, |s| {
        let u = s.unit(h).expect("bot");
        covered |= u.ai.tpos.held.is_some() && u.ai.tpos.covered;
    });
    assert!(covered, "it moved to the cover within 3 s");
}

/// A walk to a near position that comes no [`GAIN`] nearer in
/// [`GAIN_TICKS`] is cut short there and then, long before its deadline
/// (bots slid along a pod's flank for the whole deadline).
#[test]
fn a_walk_that_gains_nothing_for_a_second_is_cut() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = crate_scene(Vec3::new(0.0, 0.0, 6.0), enemy);
    let mut i = None;
    run(&mut sim, e, 120, |s| {
        let u = s.unit(h).expect("bot");
        if i.is_none() && u.ai.fight.mode == FightState::TAKE {
            i = u.ai.tpos.held;
        }
    });
    let i = i.expect("it walked to a position");
    let q = sim.positions.positions[i as usize].pos;
    // Pinned 4 m off it, its deadline far ahead.
    let at = q + Vec3::new(0.0, 0.0, 4.0);
    let t0 = sim.tick;
    let u = sim.unit_mut(h).expect("bot");
    u.ai.fight.mode = FightState::TAKE;
    u.ai.tpos.held = Some(i);
    u.ai.tpos.phase_until = t0 + 3000;
    u.ai.tpos.best = 4.0;
    u.ai.tpos.best_at = t0;
    let stalled0 = sim.nav_stats.tpos_ev[EV_WALK_STALLED.trailing_zeros() as usize];
    let mut ended = None;
    for _ in 0..90 {
        if let Some(u) = sim.unit_mut(h) {
            u.soldier.pos = at;
            u.soldier.vel = Vec3::ZERO;
        }
        run(&mut sim, e, 1, |s| {
            let u = s.unit(h).expect("bot");
            if ended.is_none() && (u.ai.fight.mode != FightState::TAKE || u.ai.tpos.held != Some(i))
            {
                ended = Some(s.tick);
            }
        });
    }
    let ended = ended.expect("the walk ended");
    assert!(ended <= t0 + GAIN_TICKS + 20, "ended at {ended}, from {t0}");
    assert_eq!(
        sim.nav_stats.tpos_ev[EV_WALK_STALLED.trailing_zeros() as usize] - stalled0,
        1,
        "by the no-progress cut"
    );
}

/// The drop test: the way across a 1 m deep trench between two platforms
/// drops; along one platform it does not.
#[test]
fn a_way_over_a_trench_drops() {
    use crate::collision::{Body, Shape};
    let (mut sim, _) = super::super::tests::l_sim();
    for x in [-4.0, 4.0] {
        sim.collision.push_body(Body::new(
            glam::Mat4::from_translation(Vec3::new(x, 0.5, 0.0)),
            Shape::Box {
                half: Vec3::new(3.0, 0.5, 3.0),
            },
        ));
    }
    let (a, a2, b) = (
        Vec3::new(-5.0, 1.0, 0.0),
        Vec3::new(-2.0, 1.0, 2.0),
        Vec3::new(5.0, 1.0, 0.0),
    );
    assert!(no_drop(&sim, a, a2), "along a platform");
    assert!(!no_drop(&sim, a, b), "over the trench");
}

/// A Cover hint inside a soldier barrier's 0.3 m margin is moved out of
/// it (by at most [`HINT_PUSH`]); one deep inside is not (Polis Massa
/// dropped 172 of its 210 hints at build).
#[test]
fn a_hint_in_a_barriers_margin_is_pushed_out() {
    let (mut sim, _) = super::super::tests::l_sim();
    if let Some(g) = sim.nav.as_mut() {
        g.barriers.push(nav::Barrier {
            centre: Vec2::ZERO,
            axes: [Vec2::X, Vec2::Y],
            half: Vec2::new(2.0, 1.0),
            mask: 1 << nav::SOLDIER,
            enabled: true,
        });
    }
    let g = sim.nav.as_ref().expect("graph");
    let b = Builder {
        sim: &sim,
        g,
        rays: 0,
    };
    let q = b.out_of_barriers(Vec2::new(0.0, 1.15)).expect("pushed out");
    assert!(q.y >= 1.0 + IN_BARRIER && (q.y - 1.15) <= HINT_PUSH, "{q}");
    assert!((q.x).abs() < 1e-4, "along the nearer face: {q}");
    assert!(
        b.out_of_barriers(Vec2::new(0.0, 0.2)).is_none(),
        "deep inside"
    );
}

/// The mover's "arrived" counts only near the position held: a bot whose
/// walk ended 4 m short of it (the outcome of its walk to the one before)
/// is still on its way; at it, it is there.
#[test]
fn a_walk_ended_short_of_the_position_held_is_not_its_arrival() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = crate_scene(Vec3::new(0.3, 0.0, 0.3), enemy);
    run(&mut sim, e, 400, |_| {});
    let i = sim.unit(h).expect("bot").ai.tpos.held.expect("a position");
    let q = sim.positions.positions[i as usize].pos;
    for (off, there) in [(4.0, false), (0.3, true)] {
        let u = sim.unit_mut(h).expect("bot");
        u.soldier.pos = q + Vec3::new(0.0, 0.0, off);
        u.ai.fight.mode = FightState::TAKE;
        let u = sim.unit(h).expect("bot");
        let mut st = u.ai;
        let mut hs = st.hold.unwrap_or_default();
        let tk = crate::ai::behave::Tick {
            me: h,
            u,
            eye: u.muzzle(),
            wc: &sim.weapon_classes[u.weapon_class],
            tier: sim.difficulty.tier(),
            skill: sim.difficulty.skill_at(u.skill),
        };
        assert_eq!(
            walk_arrived(&sim, &tk, &mut st, &mut hs),
            there,
            "{off} m off"
        );
        let mode = if there {
            FightState::COVER
        } else {
            FightState::TAKE
        };
        assert_eq!(st.fight.mode, mode, "{off} m off");
    }
}

/// The 5-12 s timer at a covered position takes another only with cover
/// (the PS2's "another qualifying hint"): with an open point 5 m off and the
/// crate's other points within [`MOVE_MIN`], the bot keeps its cover.
#[test]
fn the_timer_from_cover_takes_only_cover() {
    let enemy = Vec3::new(0.0, 0.0, -25.0);
    let (mut sim, h, e) = crate_scene(Vec3::new(0.3, 0.0, 0.3), enemy);
    let mut all = sim.positions.positions.clone();
    let stats = sim.positions.stats;
    let mut q = all[all.len() - 1];
    q.pos = Vec3::new(5.0, 0.0, 0.0);
    q.source = Source::Hub;
    (q.low, q.corner, q.shut, q.tall) = (0, 0, 0, 0);
    all.push(q);
    sim.positions = PositionTable::index(all, stats);
    run(&mut sim, e, 400, |_| {});
    let now = sim.tick;
    let u = sim.unit_mut(h).expect("bot");
    assert_eq!(u.ai.fight.mode, FightState::COVER, "at its position");
    assert!(u.ai.tpos.covered, "with cover");
    let i = u.ai.tpos.held.expect("a position");
    u.ai.tpos.next_at = now;
    u.ai.tpos.hits = 0;
    let mut kept = true;
    run(&mut sim, e, 30, |s| {
        let u = s.unit(h).expect("bot");
        kept &= u.ai.tpos.held == Some(i) && u.ai.tpos.covered;
    });
    assert!(kept, "it kept its cover");
    let u = sim.unit(h).expect("bot");
    assert!(
        u.ai.tpos.next_at > sim.tick,
        "the timer was asked and drawn again"
    );
}

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
