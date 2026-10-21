//! What a bot on foot is DOING: [`think`], one AI unit's tick as ONE priority
//! chain — perceive and retarget on the think period, then (every tick) the
//! chain that picks what walks the bot (the return to the level, the run from
//! a grenade, the held position, the vehicle it boards, the route leg) and
//! what it looks at — which hands exactly one [`AimRequest`] to the gunner
//! ([`super::gunner_foot`]) and exactly one [`MoveRequest`] to the mover
//! ([`super::mover`]). Card SPL11, `analysis/bot-design.md` 3: this file
//! writes no stick and no button.
//!
//! The gunner runs BETWEEN the two requests: the close hold, the fight and
//! the step out of a friend's line read its report
//! ([`super::gunner_foot::GunReport`]). Card TPOS deleted the strafe on a
//! new burst, and the step goes for a bot on a Tactician's position.
//!
//! Card TPOS: a fighter's position is the Tactician's when the map's table
//! has one for the fight (`super::tactic::fight`: walked to as a ring point,
//! `FightState::TAKE`, then its hide / fire cycle, `FightState::COVER`); a
//! hit there counts against it (`super::tactic::hit`) instead of card
//! REACT's row. FGHT's own style below is the fight when nothing passes.
//!
//! Card MOVR: the mover answers each request with an outcome
//! ([`super::Walk`]); a failed walk climbs [`recover`]'s ladder, and the
//! link of the chain that ran is named in `AiState::behaviour`. The think's
//! phase is the unit's index.
//!
//! Card FGHT: a seen target inside the weapon's range is FOUGHT from a
//! position of the bot's own fixed style — it stands to shoot on a spot
//! clear of the other fighters, or circles on a ring round its target
//! (`fight_position`, `ring_point`, `stand_spot`; the PS2's attack modes 2
//! and 3, `analysis/re-soldier-ai.md` 4.4, 4.5).
//!
//! Card REACT: a bot on a position that is HIT answers by its tier's row —
//! it stays, strafes, rolls aside or moves to a new position (`react`; the
//! PS2's attacker ignores the hit, a named deviation); no such step goes
//! through a death region (`react_ok`), and a roll or a move ends in a hub
//! of the graph (`in_hub`).
//!
//! Here too, until their own cards replace them: the held position (card
//! HOLD: `spot_free`, `stand_ok`, `roll_clear`, the expired hold settled at a
//! hub, `hub_reached`; its shuffle and its hold at a hub went with card
//! TPOS),
//! the run from a grenade (`hazard_flee`) and the lost bot's jump
//! (`lost_jump`). The position, fight, reaction, grenade, lost-bot and dodge
//! tests live here.

use super::draw::{
    stance_hold, unit_unit, SALT_CIRCLE, SALT_DODGE, SALT_GRENADE, SALT_HOLD, SALT_REACT,
    SALT_STYLE,
};
use super::gunner_foot::{self, AimRequest};
use super::mover::{self, MoveRequest};
use super::route::{advance_route, leg_point, retarget, sub_point};
use super::state::{
    AiState, Behaviour, FightState, HoldState, LostState, MoverState, NavTick, SteerPick, Walk,
    WalkAsk,
};
use super::{
    CIRCLE_PAUSE, CIRCLE_STOP, CORNER_ARRIVE, DODGE_COOLDOWN, FIGHT_BEYOND, FIGHT_SPACING,
    FIGHT_STEP, GIVE_UP_AFTER, HOLD_ARRIVE, HOLD_CLOSE, HOLD_COMMIT, HOLD_MOVE_PERIOD, HOLD_QUIET,
    HOLD_RANGE, HOLD_SPACING, HOLD_SPOT_RADIUS, HOLD_STRAFE, LOST_JUMP_AGAIN, LOST_JUMP_CLEAR,
    LOST_RECYCLE, PROBE_DIST, REACT_COOLDOWN, REACT_STEP, RETURN_ARRIVE, RETURN_ARRIVE_DY,
    RETURN_PLANS, RING_ARRIVE, RING_BACK, RING_SNAP, ROLL_LANE, ROLL_LEAD, ROLL_PAST, ROLL_REACH,
    SHUT_PROBE_UP, SHUT_RECYCLE, SIDESTEP_TICKS, STAND_CROUCH, STAND_SHARE, THINK_PERIOD,
};
use crate::fmath;
use crate::nav;
use crate::perception;
use crate::sim::{PadState, Sim};
use crate::unit::{Unit, UnitHandle};
use glam::{Vec2, Vec3};

/// What one tick of a bot on foot reads once and every layer shares: the
/// unit, its muzzle, its weapon class, the difficulty tier and the unit's own
/// skill vector under the tier's overrides.
#[derive(Clone, Copy)]
pub(super) struct Tick<'a> {
    pub me: UnitHandle,
    pub u: &'a Unit,
    pub eye: Vec3,
    pub wc: &'a crate::weapon::WeaponClass,
    pub tier: crate::skill::Tier,
    pub skill: crate::skill::Skill,
}

/// Produce this tick's pad for AI unit `me`, and say what it had to fall back
/// on (diagnostics only — see [`NavTick`]).
pub fn think(sim: &Sim, me: UnitHandle, st: &mut AiState) -> (PadState, NavTick) {
    let mut nt = NavTick::default();
    st.walk = WalkAsk::default();
    let Some(u) = sim.unit(me) else {
        return (PadState::default(), nt);
    };
    super::ride::settle(sim, st);
    let eye = u.muzzle();
    let wc = &sim.weapon_classes[u.weapon_class];

    let tier = sim.difficulty_of(u.team).tier();
    // This unit's own skill vector, under the tier's overrides (D1/D9). It is
    // DERIVED per think, never stored: `Unit.skill` is the state.
    let skill = sim.difficulty_of(u.team).skill_at(u.skill);

    let thinking = sim.tick >= st.next_think;
    if thinking {
        // Card MOVR: the think's PHASE is the unit's index (the design's
        // "one think rate everywhere, the phase by unit index"), not the tick
        // it spawned on: a wave that spawns together does not think together.
        // The first think is at once.
        st.next_think = sim.tick + THINK_PERIOD - (sim.tick + u64::from(me.index)) % THINK_PERIOD;
        // Phase C: LOOK and LISTEN (`crate::perception`), then pick a target
        // out of what this unit actually perceives — never out of the sim's
        // omniscient unit list, which is what `ENGAGE_RANGE` + `visible` did.
        let (held, heard) = perception::scan(sim, me, st, tier);
        // Card TEAM: then what the friends near by see.
        nt.told = perception::tell(sim, me, st, tier);
        nt.acquired = u8::from(match super::man::arc_of(sim, u) {
            Some(holds) => perception::choose_target_with(sim, me, st, tier, holds),
            None => perception::choose_target(sim, me, st, tier),
        });
        nt.contacts = held;
        nt.heard = heard;
        // Card UCALL: hearing that made a record while no enemy is in sight
        // turns the unit to the sound (B1's hearing check, the fourth of an
        // agent's interrupt checks, `FUN_00252dc8` 171455: re-unit-chatter.md
        // 2.4). PROVISIONAL [H]: the check's two early returns are not
        // traced; "no enemy seen now" stands in for them.
        nt.heard_turn = u8::from(heard > 0 && !st.contacts.iter().flatten().any(|c| c.seen_now));
        // BF1 (D10): the shooter's tracked-position error, once per think.
        if sim.ground_rules != crate::soldier::GroundRules::Bf2 {
            gunner_foot::track_error(sim, st, &skill);
        }
        // The HUNT destination is deliberately NOT bounded by perception
        // (brief D6): it is the "nothing capturable to walk at" fallback, and
        // the commander is omniscient by design too.
        let mut hunt: Option<(f32, Vec3)> = None;
        for (h, o) in sim.alive_units() {
            if h == me || o.team == u.team {
                continue;
            }
            let d2 = (o.capsule().center() - eye).length_squared();
            if hunt.is_none_or(|(bd, _)| d2 < bd) {
                hunt = Some((d2, o.soldier.pos));
            }
        }
        // An order the player gave outranks the commander's plan. A Follow
        // or Hold unit is not in the plan at all, so `retarget` would put it
        // on the Phase A fallback; the command branch runs instead (a
        // `Disperse` unit stays in the pool and never gets here).
        // Card SNPR: a sniper's spot - kept, left when its lane is quiet,
        // picked (`snipe::upkeep`).
        let seen_any = st.contacts.iter().flatten().any(|c| c.seen_now);
        super::snipe::upkeep(sim, me, u, st, seen_any);
        match (u.command, ring_walk(sim, st), super::snipe::walk(st)) {
            (Some(c), _, _) if !c.in_pool() => crate::squad::steer(sim, me, st, &c, &mut nt),
            // Card FGHT: a circling bot walks over the graph to its ring
            // point's hub and straight at the point from there (as a boarder
            // walks to its vehicle).
            (_, Some(p), _) => super::route::retarget_board(sim, me, st, p, &mut nt),
            // Card SNPR: a sniper, the same way to its spot.
            (_, None, Some(p)) => super::route::retarget_board(sim, me, st, p, &mut nt),
            _ => retarget(sim, me, st, hunt.map(|(_, p)| p), &mut nt),
        }
        // Card VAI: a bot on foot in the commander's pool may go for a
        // vehicle (`ride::claim`); an ordered one stays with its order.
        if u.mount.is_none() && u.command.is_none_or(|c| c.in_pool()) {
            let had = super::ride::board_point(sim, st).is_some();
            super::ride::claim(sim, me, u, st);
            // Card WAI2: a new claim routes to its vehicle at once - card
            // TURR: one the offer pass made this tick too.
            let fresh = st.ride.is_some_and(|r| r.since == sim.tick);
            if let Some(at) = super::ride::board_point(sim, st).filter(|_| !had || fresh) {
                nt.claim = NavTick::CLAIM_NEW;
                super::route::retarget_board(sim, me, st, at, &mut nt);
            }
        }
        // Card BNDS: is this bot still on the playable level? Asked on the
        // ground only (a faller keeps what it had); two thinks off running
        // start the return walk, one think on ends it.
        // Card VMESH: one shut inside a WATERTIGHT map body (a closed prop,
        // never a room, cave or tunnel: those have open edges and bots walk
        // in them) is lost too, at once and whatever it is doing; it is
        // given up after `SHUT_RECYCLE`.
        let shut = u.mount.is_none()
            && sim
                .shut_in_solid(u.soldier.pos + Vec3::Y * SHUT_PROBE_UP)
                .is_some();
        if shut {
            st.lost = Some(match st.lost.filter(|l| l.shut) {
                Some(l) => l,
                None => LostState {
                    since: sim.tick,
                    confirmed: true,
                    shut: true,
                    ..LostState::default()
                },
            });
        } else if st.lost.is_some_and(|l| l.pinned) {
            // Card RLOOP: a pinned bot's walk is not the level test's; it
            // ends with the walk (below).
        } else if u.mount.is_none() && u.soldier.grounded {
            st.lost = match (sim.level_at(u.soldier.pos), st.lost.filter(|l| !l.shut)) {
                (crate::sim::Level::Off(_), None) => Some(LostState {
                    since: sim.tick,
                    ..LostState::default()
                }),
                (crate::sim::Level::Off(_), Some(l)) => Some(LostState {
                    confirmed: true,
                    ..l
                }),
                _ => None,
            };
        }
        // Card RETW: a confirmed spell looks for the FLOOR walk back once
        // (`Sim::return_path`), and again from where it stands when the walk
        // is spent with the bot still off — its kept points walked, or a
        // progress window with no progress (`mover::run`) —
        // [`RETURN_PLANS`] times in all; then the hubs.
        if let Some(mut l) = st.lost.filter(|l| l.confirmed && !l.shut && !l.pinned) {
            let spent = l.path.len > 0 && l.path_at >= l.path.len;
            if l.plans == 0 || (spent && l.plans < RETURN_PLANS) {
                l.plans += 1;
                l.path = sim.return_path(u.soldier.pos).unwrap_or_default();
                l.path_at = 0;
                st.lost = Some(l);
            }
        }
        // Card HOLD: the position's upkeep. With nobody perceived any more
        // the fight is over and the bot advances; else it keeps the hint's
        // stance and, every few seconds, may step to another spot near by.
        if let Some(mut h) = st.hold.filter(|h| sim.tick < h.until) {
            // (Card TEAM: what a friend tells keeps nobody on a position.)
            if held > 0 && st.contacts.iter().flatten().any(|c| !c.told) {
                h.last_contact = sim.tick;
            }
            if let Some((_, p)) = hunt {
                h.look = p + Vec3::Y * 1.4;
            }
            if sim.tick.saturating_sub(h.last_contact) >= HOLD_QUIET {
                h.until = sim.tick;
            } else {
                // (Card TPOS: no shuffle - a fighter's cycle and its next
                // position are the Tactician's, `tactic::fight`.)
                st.want_stance = stance_hold(st.want_stance, h.stance_mask, me, sim.tick);
            }
            st.hold = Some(h);
        }
        // Card TEAM: a bot with the COVER role (`team::board`) guards its
        // position, facing the approach, while no fight holds it.
        if let Some(g) = guard_post(sim, st) {
            let q = &sim.positions.positions[st.role.at as usize];
            let mut h = st
                .hold
                .filter(|h| sim.tick < h.until && h.spot == g)
                .unwrap_or(HoldState {
                    spot: g,
                    goal: g,
                    commit_until: sim.tick + HOLD_COMMIT,
                    ..HoldState::default()
                });
            h.until = sim.tick + THINK_PERIOD + HOLD_CLOSE;
            h.last_contact = sim.tick;
            h.look = st.role.look;
            let d = st.role.look - g;
            h.stance_mask = if q.shields_toward(Vec2::new(d.x, d.z)) {
                crate::commander::MASK_CROUCH
            } else {
                0
            };
            st.hold = Some(h);
        }
    }

    // Card VAI: a seated bot's think is its seat's (`ride::drive`).
    if let Some(m) = u.mount {
        st.stall = None; // a rider is not on the walker's watch (card RCYC)
        st.mv = MoverState::default();
        st.behaviour = Behaviour::Ride;
        return super::ride::drive(sim, me, m, st, nt, thinking);
    }

    let mut pad = PadState::default();
    // Phase C decision 2 (brief D7): the target and what is KNOWN about it.
    // SEEN → aim at the live unit's lead point and fire once the tier's
    // reaction has passed; MEMORY → aim at the last known position and never
    // fire; nothing perceived → the travelling look.
    let held = st.target.and_then(|h| perception::contact_of(st, h));
    let seen = held.filter(|c| c.seen_now);
    let enemy = seen.and_then(|c| sim.unit(c.handle));
    let memory = held.filter(|c| !c.seen_now).map(|c| c.last_pos);
    let pos = u.soldier.pos;
    let tk = Tick {
        me,
        u,
        eye,
        wc,
        tier,
        skill,
    };
    // D9: the AI sniper distance table's spread, every tick (the gunner's).
    gunner_foot::range_table(&tk, st, enemy);
    // Card GRAI: the weapon for this target, once a think (the gunner's).
    if thinking {
        super::arsenal::think(sim, &tk, st, enemy);
    }

    let leg_hub = st.route.filter(|r| !r.is_last()).map(|r| r.next_hub);
    advance_route(sim, me, st, pos, 0.0, &mut nt);
    // Card HOLD: a hub just reached settles a hold that has run out.
    if leg_hub.is_some_and(|h| st.route.map(|r| r.next_hub) != Some(h)) {
        hub_reached(sim, st);
    }
    // ...and a grenade on the ground near by is a thing to run from.
    let flee = hazard_flee(sim, me, u, &tier);
    // The on-hit reaction as a ROLL (below): the side, this tick only.
    let mut roll: i8 = 0;

    // D11's DODGE-ON-HIT. The PS2's on-hit dodge is a re-plan of the
    // movement, not a roll (RE §9.4), so ours is the side-step that already
    // exists: the move target is offset `SIDESTEP_OFFSET` along the path
    // perpendicular for `SIDESTEP_TICKS`. The coin is the skill's
    // `lerp(0, 0.2)` and the cooldown is the RE's 2 s. The side ALTERNATES
    // from the steering's remembered side, so two dodges in a row do not
    // walk a unit into the same wall twice.
    // Card REACT: a bot on a position (a hub's, a fight's) answers the hit
    // by its tier's row instead (`react`).
    if u.last_hit
        .is_some_and(|(_, t)| sim.tick > 0 && t == sim.tick - 1)
        && sim.tick >= st.dodge_until
    {
        if st.fight.tactician() {
            // Card TPOS: a hit counts against the position (`tactic::hit`).
            super::tactic::hit(sim, &tk, st);
        } else if let Some(h) = st.hold.filter(|h| sim.tick < h.until) {
            roll = react(sim, &tk, st, h, enemy, &mut nt);
        } else if unit_unit(me.index, sim.tick, me.gen, SALT_DODGE) < skill.dodge_chance {
            let side: i8 = if st.slide > 0 { -1 } else { 1 };
            st.dodge_until = sim.tick + DODGE_COOLDOWN;
            match st.route {
                Some(mut r) => {
                    r.sidestep = side * (SIDESTEP_TICKS + 1);
                    st.route = Some(r);
                }
                // Nothing to offset: strafe the pad directly for the same window.
                None => {
                    st.strafe_until = sim.tick + SIDESTEP_TICKS as u64;
                    st.strafe_side = side;
                }
            }
        }
    }

    // Card REACT: the roll a reaction asked for, now that the bot is under
    // way to where it ends.
    if let Some(mut h) = st.hold.filter(|h| h.roll_at != 0 && sim.tick >= h.roll_at) {
        roll = h.strafe_side;
        h.roll_at = 0;
        st.hold = Some(h);
    }

    let route_leg = st.route.filter(|r| !r.is_last());
    let post = st.target_cp.and_then(|i| sim.cps.get(i as usize));

    // The point this tick walks at: the current detour corner, else the leg's
    // waypoint, else (no graph at all) the target post. A corner is XZ only —
    // walking is XZ and the gun is levelled while travelling — so it takes the
    // unit's own y and costs no ground query.
    let corner = st.route.and_then(|r| r.corner());
    let goal: Option<Vec3> = match (st.route, st.cmd) {
        // On an arc leg, along the arc's line (`leg_point`) — but not while
        // the steering is going round something (`slide`): a wall on the line
        // is passed by leaving the line, and pulling back to it flipped the
        // slide's side every few ticks.
        (Some(r), _) => Some(if st.slide == 0 {
            let v = u.soldier.vel;
            leg_point(&r, pos, Vec2::new(v.x, v.z).length())
        } else {
            sub_point(&r, pos)
        }),
        // A COMMANDED unit with no route walks at the point its order gives
        // it — the leader, its standing point by him, or the spot it holds —
        // and stands once there. It has no post (the command branch clears
        // `target_cp`), so this arm is the whole of its walking.
        (None, Some(c)) => {
            Some(c.goal).filter(|g| nav::xz_dist(pos, *g) > crate::squad::STAND_ARRIVE)
        }
        // Brief D7: with no route leg and no post to walk at, a unit that has
        // LOST sight of its target walks at the last place it knew of, until
        // the memory expires. Nothing else about walking changes — an engaged
        // unit still walks its route (A.1 decision 4).
        (None, None) => post.map(|cp| cp.def.pos).or(memory),
    };
    // Card VAI: a boarding claim walks the bot at its vehicle (its own
    // give-up clock, not the route's stuck window, ends a claim that cannot
    // get there) — card WAI2: over the graph to the vehicle's hub
    // (`route::retarget_board`), straight at it on the last leg or with no
    // route.
    let boarding = super::ride::board_goal(sim, me, u, st, &mut nt);
    let goal = match boarding {
        Some(_) if st.route.is_some_and(|r| !r.is_last()) => goal,
        _ => boarding.or(goal),
    };
    // Card BNDS: a bot off the playable level walks BACK to it before
    // anything else — at the nearest hub it can stand level with
    // (`Sim::return_point`), the next one after a window with no progress.
    // Off it for `LOST_RECYCLE` with no way back, it is given up.
    // Card RETW: ...along the floor walk found for it when there is one
    // (each of its points reached in turn), and only then at a hub.
    if let Some(mut l) = st.lost.filter(|l| l.confirmed) {
        while l.path_at < l.path.len && {
            let p = l.path.pts[usize::from(l.path_at)];
            nav::xz_dist(pos, p) <= RETURN_ARRIVE && (pos.y - p.y).abs() <= RETURN_ARRIVE_DY
        } {
            // (A fresh progress window for the next point.)
            l.path_at += 1;
            l.window = 0;
            l.ref_dist = 0.0;
        }
        st.lost = Some(l);
    }
    // Card RLOOP: a pinned bot at the end of its walk is at its waypoint;
    // its route goes on from there.
    if st.lost.is_some_and(|l| l.pinned && l.path_at >= l.path.len) {
        st.lost = None;
    }
    let lost = st.lost.filter(|l| l.confirmed);
    let ret = lost.and_then(|l| {
        if l.path_at < l.path.len {
            Some(l.path.pts[usize::from(l.path_at)])
        } else {
            sim.return_point(pos, l.tries)
        }
    });
    if let Some(l) = lost {
        let after = if l.shut { SHUT_RECYCLE } else { LOST_RECYCLE };
        let over = sim.tick.saturating_sub(l.since) >= after;
        if l.pinned {
            // (On the level: given up as the last resort's, not as lost.)
            nt.stalled |= u32::from(over);
        } else {
            nt.lost = 1;
            nt.recycle |= u32::from(over);
        }
    }
    // Card FGHT: the ring point a circling bot walks to — over the graph,
    // and straight at it on the last leg and with no route.
    let ring = ring_walk(sim, st)
        .filter(|_| ret.is_none() && flee.is_none() && boarding.is_none() && st.cmd.is_none());
    let goal = match ring {
        Some(p) if !st.route.is_some_and(|r| !r.is_last()) => Some(p),
        _ => goal,
    };
    let at_ring = ring.filter(|p| goal == Some(*p));
    // Card SNPR: the sniper's spot - over the graph, and straight at it on
    // the last leg and with no route.
    let spot = super::snipe::walk(st).filter(|_| {
        ret.is_none() && flee.is_none() && boarding.is_none() && st.cmd.is_none() && ring.is_none()
    });
    let goal = match spot {
        Some(p) if !st.route.is_some_and(|r| !r.is_last()) => Some(p),
        _ => goal,
    };
    let at_spot = spot.filter(|p| goal == Some(*p));
    // Card HOLD: the position held (when nothing more urgent walks the bot).
    let holding = st.hold.filter(|h| {
        sim.tick < h.until
            && ret.is_none()
            && flee.is_none()
            && boarding.is_none()
            && ring.is_none()
    });
    // Diagnostic (card BLKF): which link of the chain below walks the bot.
    let walk_kind = match () {
        _ if ret.is_some() => WalkAsk::RETURN,
        _ if flee.is_some() => WalkAsk::FLEE,
        _ if holding.is_some() => WalkAsk::HOLD,
        _ if boarding.is_some() && goal == boarding => WalkAsk::BOARD,
        _ => WalkAsk::LEG,
    };
    // Card MOVR: where the walk ENDS, when it ends somewhere — a held
    // position, the last leg's point (the last-node radius), a post or a
    // remembered enemy walked straight at. A commanded bot's stand is its
    // order's (`squad::STAND_ARRIVE`, above).
    let arrive = match walk_kind {
        WalkAsk::HOLD => holding.map(|h| (h.goal, HOLD_ARRIVE)),
        // (Card TPOS: a position is walked onto, as a held one is.)
        WalkAsk::LEG if ring.is_some() && st.fight.mode == FightState::TAKE => {
            at_ring.map(|p| (p, HOLD_ARRIVE))
        }
        WalkAsk::LEG if ring.is_some() => at_ring.map(|p| (p, RING_ARRIVE)),
        WalkAsk::LEG if spot.is_some() => at_spot.map(|p| (p, RING_ARRIVE)),
        WalkAsk::LEG => match (st.route, st.cmd) {
            (_, Some(_)) => None,
            (Some(r), None) => r.is_last().then_some((r.waypoint, CORNER_ARRIVE)),
            (None, None) => goal.map(|g| (g, CORNER_ARRIVE)),
        },
        _ => None,
    };
    let goal = ret.or(flee).or(holding.map(|h| h.goal)).or(goal);

    // Where to look: the engaged enemy's lead point, else where we are
    // walking, else the target post (the last leg, and the no-graph
    // placeholder).
    let aim = match (enemy.zip(seen), memory, corner, route_leg) {
        // A seen enemy: the gunner's own aim point (its lead point under the
        // D10 error), and the only target it fires on.
        (Some((e, c)), _, _, _) => AimRequest::Enemy(e, c),
        // A remembered contact is aimed at where it was last known to be —
        // a point on the target, no lead (the PS2 aims at a point, and there
        // is no velocity to lead with once the eyes are off it).
        (None, Some(p), _, _) => AimRequest::Memory(p + Vec3::Y * 1.4),
        // Card HOLD: a holder standing on its point with nobody in sight
        // watches the way the nearest enemy is.
        (None, None, _, _)
            if holding.is_some_and(|h| {
                h.look != Vec3::ZERO && nav::xz_dist(pos, h.goal) <= HOLD_ARRIVE
            }) =>
        {
            AimRequest::Travel(holding.map_or(pos, |h| h.look))
        }
        (None, None, Some(_), _) => AimRequest::Travel(goal.unwrap_or(pos)),
        // Running from a grenade, or walking back to the level: where it goes.
        (None, None, None, _) if flee.is_some() || ret.is_some() => {
            AimRequest::Travel(goal.unwrap_or(pos) + Vec3::Y * 1.4)
        }
        (None, None, None, Some(r)) => AimRequest::Travel(r.waypoint),
        (None, None, None, None) => AimRequest::Travel(match post {
            Some(cp) => cp.def.pos + Vec3::Y * 1.4,
            // Hunting on the last leg: the destination hub itself.
            None => match st.route {
                Some(r) => r.waypoint,
                // D6: a commanded unit LOOKS where it walks — the idle arm
                // below is for a unit with no route, no post AND no order.
                None => match st.cmd {
                    Some(c) => c.goal + Vec3::Y * 1.4,
                    None => {
                        st.steer = SteerPick::Idle;
                        st.behaviour = Behaviour::None;
                        return (pad, nt);
                    }
                },
            },
        }),
    };
    // The gunner: the look sticks and the trigger. With nothing to turn to
    // (the aim point is the muzzle itself) the tick ends here.
    let Some(gun) = gunner_foot::run(sim, &tk, st, &aim, &mut pad, &mut nt) else {
        st.steer = SteerPick::Idle;
        st.behaviour = Behaviour::None;
        return (pad, nt);
    };
    // Card FFLN: a friend in the line of fire. The shot waits (the gunner), and a
    // holder whose last step's commit time is over steps aside, away from the
    // friend, through its goal's one writer. A bot walking its leg walks on:
    // the line clears by itself. Card TPOS: not a bot on a Tactician's
    // position - those are chosen out of the friends' lines - but still the
    // fallback's stander (without it, BF2 Hoth's fighters with a teammate
    // within 1.5 m went 3.6 -> 5.8 % over 8 seeds; with it 3.6 %). No strafe
    // on a new burst any more.
    if let (Some(side), Some(mut h), false) = (gun.line, holding, st.fight.tactician()) {
        let (sy, cy) = fmath::sin_cos(u.soldier.yaw);
        let p = pos + Vec3::new(cy, 0.0, -sy) * (f32::from(side) * HOLD_STRAFE);
        if sim.tick >= h.commit_until
            && react_ok(sim, u, p)
            && spot_free(sim, me, p)
            && h.step_to(p, sim.tick, false)
        {
            h.strafe_side = side;
            st.hold = Some(h);
        }
    }
    // The holder as it is now (the dodge and the step out of a friend's line
    // above may have moved its goal this tick).
    let holding = holding.and(st.hold);
    // A seen enemy inside [`HOLD_RANGE`]: stand and fight (a remembered one
    // never holds the bot: the design's "the unit may move toward it").
    let hold = gun.range.is_some_and(|r| r <= HOLD_RANGE);
    // Card FGHT: FIGHT — a seen target inside the weapon's range + 10 m
    // (`analysis/re-soldier-ai.md` 4.2: the combat helper's attack). A
    // RIDER only as the vehicle it rides, and only by a bot whose weapon
    // choice found one that hurts it with the hull in its band (card HVYW,
    // `ArmState::vehicle`: the launcher against a tank; a grenade is thrown
    // in passing): a bot that took a position against one it
    // cannot hurt kept it for good (BF1 Rhen Var, seed 4: twelve bots round
    // one gunner seat, no outcome in 60,000 ticks) — it walks on.
    let fight = enemy.zip(gun.range).filter(|(e, r)| {
        (e.mount.is_none() || st.arm.vehicle) && *r <= wc.ai_range[2] + FIGHT_BEYOND
    });
    let objective = sim.commander[usize::from(u.team).min(1)].of(me.index);
    let defending = matches!(objective, Some(crate::commander::Objective::Defend(_)));
    // Card FGHT (it was card HOLD's close hold, "stand where it is while he
    // is inside 25 m"): a free bot with a target in reach fights from a
    // position of its style (`fight_position`), which lasts [`HOLD_CLOSE`]
    // past the last tick the target was in reach — the 2 s a fight lasts at
    // least, and the 2 s after which it is given up.
    let free = ret.is_none() && flee.is_none() && boarding.is_none() && st.cmd.is_none();
    // Card TPOS: hidden at its position a bot does not see its target - the
    // hide phase's point - and it keeps the position while it remembers it
    // (without this the fight lapsed 2 s into the hide and the bot walked
    // off, then back: the maintainer's Kamino scene).
    // Card TEAM: not on a told record - a friend's sight is aimed at, it
    // does not keep the bot at a position it cannot fire from.
    // Card TPOSR: and the Tactician runs on the remembered target as on a
    // seen one - the walk there, the hide / fire cycle, the leave tests (hit
    // twice, round its cover, out of the band, the 5-12 s timer): the PS2's
    // cover state machine runs every update, its walk and hidden states
    // keeping the target (re-soldier-ai 4.6; AUDIT TPOS: a bot hit twice kept
    // its position 22 s, hidden, while its target was out of sight).
    if let (true, Some(c), true, Some(mut h)) = (
        st.fight.tactician(),
        held.filter(|c| !c.seen_now && !c.told),
        free,
        st.hold,
    ) {
        let foe = super::tactic::Foe::remembered(c.last_pos);
        if super::tactic::fight(sim, &tk, st, &foe, &mut h, thinking, &mut nt) {
            h.until = h.until.max(sim.tick + HOLD_CLOSE);
        }
        st.hold = Some(h);
    }
    let manning = boarding.is_some()
        && st
            .ride
            .and_then(|r| r.board)
            .is_some_and(|vh| super::man::is_turret(sim, vh));
    match fight.filter(|_| free) {
        Some((e, r)) => fight_position(sim, &tk, st, e, r, thinking, &mut nt),
        None if !free || st.hold.is_none_or(|h| sim.tick >= h.until) => {
            if st.fight.mode == FightState::TAKE {
                nt.tpos_ev |= super::tactic::EV_WALK_DROPPED;
            }
            super::tactic::release(st, sim.tick);
            st.fight.mode = 0;
        }
        None => {}
    }
    // Card TPOSR: the position held stays reserved while it is held.
    super::tactic::keep(sim, st, thinking, &mut nt);
    // Card RCYC: who the last resort (the mover's progress watch) never
    // gives up — a bot on a held position, fighting an enemy it sees,
    // running from a grenade or under a squad order.
    let excused = holding.is_some()
        || ring.is_some()
        || flee.is_some()
        || enemy.is_some()
        || st.cmd.is_some();
    // The holder's goal may have moved this tick: its destination with it.
    let arrive = match (walk_kind, holding) {
        (WalkAsk::HOLD, Some(h)) => Some((h.goal, HOLD_ARRIVE)),
        _ => arrive,
    };
    let goal = match (walk_kind, holding) {
        (WalkAsk::HOLD, Some(h)) => Some(h.goal),
        _ => goal,
    };
    let mv = MoveRequest {
        kind: walk_kind,
        goal,
        arrive,
        // A walk to a ring point ends there, not in the post's region.
        post: post.filter(|_| ring.is_none()),
        defending,
        // The stand of a boarder or a commanded bot with an enemy inside
        // 25 m — and of a free bot on the tick it takes its fight position
        // (from the next it is `holding`, or walks to its ring point). Card
        // TURR: not of a bot walking to a turret, whose gun is the fight (it
        // stood 0.4 m out of reach shooting the enemy it was sent to man
        // the turret against).
        in_range: (hold && !free && !manning)
            || (fight.is_some() && free && ring_walk(sim, st).is_none()),
        ret,
        hold: holding,
        flee: flee.is_some(),
        boarding: boarding.is_some(),
        roll,
        excused,
        // Card HVYW (OURS): ... but not for a throw at a VEHICLE that cannot
        // be made from here - the grenade is the one weapon most bots have
        // for a tank and no 3 s window ends that choice, so a bot under a
        // turret its arc does not reach stood until it was shot (BF1 Rhen
        // Var, seed 3: 40 s). It walks on, and throws where it can.
        throwing: enemy.is_some_and(|e| e.mount.is_none() || !gun.no_throw)
            && matches!(
                super::arsenal::armed(sim, u, st),
                super::arsenal::Armed::Thrown(..)
            ),
    };
    // The mover: the move sticks, the stance, the roll and the jump. It
    // answers whether the bot stands at its post.
    let moved = mover::run(sim, &tk, st, &mv, &mut pad, &mut nt);
    let arrived_at_post = moved.at_post;
    // Card MOVR: every walk has an outcome, and a failed one is handled.
    match moved.walk {
        // Card FGHT: at the ring point, or where the way to it is shut: the
        // circle's pause, where the bot stands.
        // Card TPOS: at the Tactician's position its cycle starts; a way
        // there that is shut leaves it (the next think asks again).
        Walk::Arrived if ring.is_some() && st.fight.mode == FightState::TAKE => {
            if let Some(mut h) = st.hold {
                if super::tactic::walk_arrived(sim, &tk, st, &mut h) {
                    nt.tpos_ev |= super::tactic::EV_WALK_ARRIVED;
                    st.hold = Some(h);
                }
            }
        }
        // (Given up near enough, it is there where it stopped.)
        Walk::Failed if ring.is_some() && st.fight.mode == FightState::TAKE => {
            let mut h = st.hold.unwrap_or_default();
            if super::tactic::cut_short(sim, &tk, st, &mut h) {
                nt.tpos_ev |= super::tactic::EV_WALK_NEAR;
                st.hold = st.hold.map(|_| h);
            } else {
                nt.tpos_ev |= super::tactic::EV_WALK_GIVEN_UP;
                if let Some(h) = st.hold.as_mut() {
                    h.settle(pos);
                }
                st.fight.mode = FightState::STAND;
            }
        }
        // (Not a bot already at its Tactician's position: `tactic::fight`
        // ended the walk there this tick.)
        Walk::Arrived | Walk::Failed if ring.is_some() && !st.fight.tactician() => {
            if let Some(h) = st.hold.as_mut() {
                h.settle(pos);
            }
            st.fight.mode = FightState::PAUSE;
            st.fight.pause_until = sim.tick + CIRCLE_PAUSE;
        }
        // Card SNPR: at the sniper's spot, or the way to it shut.
        Walk::Arrived if at_spot.is_some() && holding.is_none() => {
            super::snipe::arrive(sim, me, st);
        }
        Walk::Failed if spot.is_some() && holding.is_none() => {
            super::snipe::walk_failed(sim, st);
            recover(sim, st, &mut nt);
        }
        Walk::Failed => recover(sim, st, &mut nt),
        _ => {}
    }
    // A bot that has arrived is not recovering any more.
    if moved.walk == Walk::Arrived {
        st.mv.recover = 0;
    }
    let was = st.behaviour;
    st.behaviour = match moved.walk {
        _ if ret.is_some() => Behaviour::Recover,
        _ if flee.is_some() => Behaviour::Evade,
        _ if super::snipe::holds(st, holding) => Behaviour::Overwatch,
        // Card TEAM: on its guard position (the Cover role).
        _ if holding.is_some_and(|h| guard_post(sim, st) == Some(h.spot)) => Behaviour::Guard,
        _ if holding.is_some() || ring.is_some() => Behaviour::Fight,
        _ if walk_kind == WalkAsk::BOARD || boarding.is_some() => Behaviour::Board,
        _ if st.cmd.is_some() => Behaviour::Follow,
        Walk::Arrived if post.is_some() && defending => Behaviour::Guard,
        Walk::Arrived if post.is_some() => Behaviour::Capture,
        _ if hold || (fight.is_some() && free) => Behaviour::Fight,
        Walk::Idle => Behaviour::None,
        _ if st.mv.recover > 0 => Behaviour::Recover,
        _ => Behaviour::Travel,
    };
    // Card UTALK: the trooper agent's two talking points (B1 `FUN_0024b7a0`
    // 165905: it starts away from its goal; `FUN_0024baf0` 166190: it finds
    // itself at the goal - re-squad-orders.md 5). The port's trooper is the
    // walk to the post: set out from any other behaviour, or arrive.
    nt.trooper = match (was, st.behaviour) {
        (w, Behaviour::Travel) if w != Behaviour::Travel => NavTick::SET_OUT,
        (Behaviour::Travel, Behaviour::Guard | Behaviour::Capture) => NavTick::ARRIVED,
        _ => 0,
    };
    // The DWELL counter (Phase B diagnostics): consecutive ticks spent within
    // DWELL_RADIUS of the post of an objective this unit has not completed. It
    // is small for a unit walking past its post and large only for one that
    // stands beside an objective it cannot finish — a capture volume above it,
    // a door it cannot open — which is what the acceptance asks about.
    let near_objective = objective
        .and_then(|o| o.cp())
        .and_then(|c| sim.cps.get(c as usize))
        .is_some_and(|cp| nav::xz_dist(pos, cp.def.pos) <= crate::commander::DWELL_RADIUS);
    st.unreachable_dwell = if near_objective && !arrived_at_post {
        st.unreachable_dwell.saturating_add(1)
    } else {
        0
    };
    nt.dwell = st.unreachable_dwell;
    (pad, nt)
}

/// Card TEAM: the guard position of a bot with the Cover role, while no
/// fight holds it.
fn guard_post(sim: &Sim, st: &AiState) -> Option<Vec3> {
    st.role
        .post
        .filter(|_| st.fight.mode == 0)
        .and_then(|_| sim.positions.positions.get(st.role.at as usize))
        .map(|q| q.pos)
}

/// Card MOVR — RECOVER (`analysis/bot-design.md` 6.3): the walk has FAILED
/// again (the mover's progress window; `MoverState::recover` counts the
/// failures since the walk last made progress). The ladder:
///
/// 1. plan again from the hub the bot stands in and side-step — the window's
///    own cure (`mover::stuck_window`, `route::replan`), every failure;
/// 2. (card RETW: back along the floor to a hub it can stand level with);
/// 3. on every [`GIVE_UP_AFTER`]-th failure give the goal up — a boarder
///    drops its claim; any other reports its post and the commander draws
///    again without it (`commander::plan`, `MoverState::give_up`). Not
///    "stand where it is, in the post's region": a bot kept under its post's
///    floor would hold the post contested for good (card STEND);
/// 4. [`PIN_FAILS`](super::PIN_FAILS) failures at one place, whatever the
///    plan: the bot walks to its waypoint along the floor, or is recycled
///    when there is no such walk (card RLOOP, `stall::send_off`) — this count
///    starts again with every plan that is nearer its waypoint, so two
///    plans taken in turn never reached rung 3;
/// 5. the 120 s recycle (card RCYC, `stall::watch`), counted by the sweep.
fn recover(sim: &Sim, st: &mut AiState, nt: &mut NavTick) {
    st.mv.recover = st.mv.recover.saturating_add(1);
    if !st.mv.recover.is_multiple_of(GIVE_UP_AFTER) {
        return;
    }
    if st.ride.is_some_and(|r| r.board.is_some()) {
        super::ride::give_up(sim, st, nt);
    } else if let (Some(cp), None) = (st.target_cp, st.cmd) {
        st.mv.give_up = cp + 1;
    }
}

/// Card HOLD: hub `hub` has just been reached: a hold that has run out is
/// dropped, or pays one of the hubs it owes. Card TPOS: no bot takes a hold
/// at a hub any more because an enemy is about (card HOLD's 10-24 s dwell at
/// a hint or a drawn hub point) - a fight's position is the Tactician's.
/// Those holders were 11-24 % of Yavin 2's, Utapau's and Polis Massa's fight
/// time with no position, standing on a target they did not see; without
/// them B12 rose on 7 of 8 maps and kills a minute x1.06 (8 seeds).
fn hub_reached(sim: &Sim, st: &mut AiState) {
    if let Some(mut h) = st.hold {
        if sim.tick < h.until {
            return;
        }
        h.skip = h.skip.saturating_sub(1);
        st.hold = (h.skip > 0).then_some(h);
    }
}

/// Card JITR: is `p` free to hold — not within [`HOLD_SPACING`] of the point
/// another bot holds, nor of a player? OURS; the full reservation is card
/// BOTD's.
fn spot_free(sim: &Sim, me: UnitHandle, p: Vec3) -> bool {
    spot_clear_of(sim, me, p, HOLD_SPACING, None)
}

/// [`spot_free`] at a spacing of `r` (card FGHT: [`FIGHT_SPACING`]). With
/// `team` (a stander's own) a teammate on foot with no position counts where
/// it is, when its index is the lower: of two bots that take their positions
/// on one tick the first keeps its place.
fn spot_clear_of(sim: &Sim, me: UnitHandle, p: Vec3, r: f32, team: Option<u8>) -> bool {
    sim.alive_units().all(|(h, o)| {
        let at = match o.ai.hold.filter(|x| sim.tick < x.until) {
            _ if h == me || o.mount.is_some() => return true,
            Some(x) => x.goal,
            None if o.is_player => o.soldier.pos,
            None if team == Some(o.team) && h.index < me.index => o.soldier.pos,
            None => return true,
        };
        nav::xz_dist(at, p) >= r
    })
}

/// Card FGHT: the position a bot FIGHTS from, a seen target `e` at `range`
/// inside its weapon's reach (`analysis/re-soldier-ai.md` 4.4, 4.5). Each
/// bot has a fixed style, as on the PS2 (its controller's own number): it
/// STANDS to shoot — a third of the standers crouched — while the target is
/// inside the middle of the weapon's band, or it CIRCLES: it walks to a
/// point on a ring about the target (a random bearing, the radius drawn
/// round the weapon's optimal range), pauses 1.9 s there, or by
/// [`CIRCLE_STOP`] a think on the way, and takes the next. A scoped weapon stands wherever its
/// target is in range (card FGHTA; [`stands_at`]). The PS2's
/// zig-zag and flank styles need 12.5 and 17.5 m of open ground and are 2-5 %
/// of its attack time, and its cover draw waits for card TPOS: not built.
/// The position is a [`HoldState`], kept [`HOLD_CLOSE`] past the last tick
/// the target was in reach.
///
/// OURS (the maintainer, 2026-10-05): a stander does not stop beside another fighter
/// ([`stand_spot`]), and a ring is no wider than the bot's own distance
/// ([`ring_point`]).
fn fight_position(
    sim: &Sim,
    tk: &Tick,
    st: &mut AiState,
    e: &Unit,
    range: f32,
    thinking: bool,
    nt: &mut NavTick,
) {
    let Tick { me, u, wc, .. } = *tk;
    let pos = u.soldier.pos;
    let (stander, crouches, anywhere) = style(sim, tk);
    // Card HVYW: against a vehicle every bot stands where it is and shoots
    // (the PS2's stand mode and its band's stop, 4.4 / 4.5; its flank and
    // behind modes are not built) - no ring 75 m round a tank.
    let stands = e.mount.is_some() || (stander && (anywhere || stands_at(wc, range)));
    let live = st.hold.filter(|h| sim.tick < h.until);
    let mut h = live.unwrap_or(HoldState {
        spot: pos,
        goal: pos,
        strafe_side: if unit_unit(me.index, sim.tick, me.gen, SALT_HOLD ^ 4) < 0.5 {
            -1
        } else {
            1
        },
        shuffle_at: sim.tick + HOLD_MOVE_PERIOD,
        last_contact: sim.tick,
        ..HoldState::default()
    });
    if live.is_none() {
        if st.fight.mode == FightState::TAKE {
            nt.tpos_ev |= super::tactic::EV_WALK_DROPPED;
        }
        super::tactic::release(st, sim.tick);
        st.fight.mode = 0;
    }
    h.until = h.until.max(sim.tick + HOLD_CLOSE);
    // Card TPOS: the Tactician's position, when the table has one for this
    // fight; else the bot's own style below.
    if super::tactic::fight(
        sim,
        tk,
        st,
        &super::tactic::Foe::seen(e),
        &mut h,
        thinking,
        nt,
    ) {
        // (In the open, a croucher by style crouches.)
        if !st.tpos.covered && crouches && h.stance_mask == 0 {
            h.stance_mask = crate::commander::MASK_CROUCH;
        }
        st.hold = Some(h);
        return;
    }
    let ring = |h: &mut HoldState, st: &mut AiState, nt: &mut NavTick| match ring_point(
        sim,
        tk,
        e.soldier.pos,
    ) {
        Some(p) => {
            h.spot = p;
            h.goal = p;
            h.commit_until = sim.tick + HOLD_COMMIT;
            h.blocked = 0;
            st.fight.mode = FightState::WALK;
            // Over the graph to the point's hub, straight at it from there.
            super::route::retarget_board(sim, me, st, p, nt);
        }
        // Card TEAM: no ring point, and the target out of the trigger's
        // reach - walk at the target itself. Pausing for a ring point here
        // stood the last four soldiers of BF1 Kashyyyk (Hard, seed 2) 57 m
        // apart in the shallows, none in reach, the water refusing every
        // point: the round never ended.
        None if nav::xz_dist(pos, e.soldier.pos) >= wc.ai_range[2] => {
            let p = e.soldier.pos;
            h.spot = p;
            h.goal = p;
            h.commit_until = sim.tick + HOLD_COMMIT;
            h.blocked = 0;
            st.fight.mode = FightState::WALK;
            super::route::retarget_board(sim, me, st, p, nt);
        }
        None => {
            h.settle(pos);
            st.fight.mode = FightState::PAUSE;
            st.fight.pause_until = sim.tick + CIRCLE_PAUSE;
        }
    };
    match (stands, st.fight.mode) {
        // Card REACT: on its way to the position a hit sent it to (its
        // arrival is the mover's answer, in `think`, as a ring point's).
        (_, FightState::MOVE) => {}
        (true, FightState::STAND) => {}
        (true, mode) => {
            // A position it holds already (a hub's, card HOLD) is the one
            // it stands on; else, and after a circle, a spot of its own.
            if live.is_none() || mode != 0 {
                let p = stand_spot(sim, me, u);
                h.spot = p;
                h.goal = p;
                h.commit_until = sim.tick + HOLD_COMMIT;
                h.blocked = 0;
            }
            if h.stance_mask == 0 && crouches {
                h.stance_mask = crate::commander::MASK_CROUCH;
            }
            st.fight.mode = FightState::STAND;
        }
        // On its way (its arrival is the mover's answer, in `think`).
        (false, FightState::WALK) => {
            if thinking && unit_unit(me.index, sim.tick, me.gen, SALT_CIRCLE) < CIRCLE_STOP {
                h.settle(pos);
                st.fight.mode = FightState::PAUSE;
                st.fight.pause_until = sim.tick + CIRCLE_PAUSE;
            }
        }
        (false, FightState::PAUSE) => {
            if thinking && sim.tick >= st.fight.pause_until {
                ring(&mut h, st, nt);
            }
        }
        (false, _) => {
            h.stance_mask = 0;
            ring(&mut h, st, nt);
        }
    }
    st.hold = Some(h);
}

/// Does a bot whose style is to stand STAND with its target at `range`, or
/// take a ring point as a circler does? Inside the middle of the weapon's
/// band (the PS2's "2 becomes 3 outside the band", `analysis/re-soldier-ai.md`
/// 4.4) — and, card FGHTA, a scoped weapon anywhere between its `MinRange`
/// and `MaxRange`: the PS2 gives a class of UnitType scout with a rifle in
/// range mode 2 before that draw is made (`FUN_002abc38`, B2:231230-231234).
/// OURS, two named deviations: the PS2 keys on the soldier's CLASS, the port
/// on the weapon in hand ([`crate::weapon::WeaponClass::ai_distance_table`]);
/// and out of range the PS2's scout does not attack, the port's circles.
fn stands_at(wc: &crate::weapon::WeaponClass, range: f32) -> bool {
    let [min, opt, max] = wc.ai_range;
    (wc.ai_distance_table() && range > min && range < max)
        || (range > (min + opt) * 0.5 && range < (max + opt) * 0.5)
}

/// Card FGHT: a bot's fixed fighting style — does it STAND to shoot (else it
/// circles), and does it crouch when it stands? One draw per unit and life,
/// as the PS2's is its controller's own number (`analysis/re-soldier-ai.md`
/// 4.4): [`STAND_SHARE`] of the bots stand, a scoped weapon always, and
/// [`STAND_CROUCH`] of the standers crouch. The third answer is a test
/// scene's only: its standers stand at any range, not just inside the band.
fn style(sim: &Sim, tk: &Tick) -> (bool, bool, bool) {
    #[cfg(test)]
    if let Some((stands, anywhere)) = super::tests::STYLE.with(std::cell::Cell::get) {
        return (stands, false, anywhere);
    }
    let Tick { me, u, wc, .. } = *tk;
    let (stander, crouches) = style_of(me, sim.difficulty_of(u.team), wc.ai_distance_table());
    (stander, crouches, false)
}

/// [`style`]'s draw: (stands, crouches) of unit `me` at `difficulty`.
fn style_of(me: UnitHandle, difficulty: crate::skill::Difficulty, scoped: bool) -> (bool, bool) {
    let share = STAND_SHARE[usize::from(difficulty != crate::skill::Difficulty::Easy)];
    (
        scoped || unit_unit(me.index, 0, me.gen, SALT_STYLE) < share,
        unit_unit(me.index, 0, me.gen, SALT_STYLE ^ 1) < STAND_CROUCH,
    )
}

/// Card FGHT: the ring point a circling bot is walking to, while its fight's
/// position is live.
fn ring_walk(sim: &Sim, st: &AiState) -> Option<Vec3> {
    st.hold
        .filter(|h| st.fight.walks() && sim.tick < h.until)
        .map(|h| h.goal)
}

/// Card FGHT: a point of the circle's ring about `target` this bot can walk
/// to — a random bearing, the radius drawn in the PS2's band round the
/// weapon's optimal range (`FUN_002d9228`: opt - 0.2 (opt - min) to opt +
/// 0.2 (range - opt)): on the level, a floor a slope from here leads to,
/// dry, in no barrier, nobody else's. Four draws, each also tried half way;
/// `None` = it stays where it is this time. The PS2 hands the point to its
/// path finder untested; walls on the way are the mover's (a way that is
/// shut ends the walk where the bot stands).
///
/// OURS, PROVISIONAL, two rules. With a graph the point is taken in the hub
/// nearest the drawn one ([`RING_SNAP`], a drawn point of the hub's disc on
/// the hub's own floor): the walk stays on the authored graph. And the ring
/// is no wider than the bot's own distance from its target, a hub's point
/// no more than [`RING_BACK`] farther — a circling bot does not back away
/// from an enemy it has closed with (the PS2's ring does: 19-32 m for BF2's
/// rifle whatever the distance). The bearing is any, as the PS2's: a point
/// beyond the target is walked to (kept off the far side, 8 seeds x 32 maps
/// lost the gain in contact: 11.2 -> 10.2 % against the old 10.0 %).
fn ring_point(sim: &Sim, tk: &Tick, target: Vec3) -> Option<Vec3> {
    let Tick { me, u, wc, .. } = *tk;
    let pos = u.soldier.pos;
    let [min, opt, max] = wc.ai_range;
    let d = nav::xz_dist(pos, target);
    let (lo, hi) = (
        (opt - 0.2 * (opt - min)).min(d),
        (opt + 0.2 * (max - opt)).min(d),
    );
    let draw = |k: u32| unit_unit(me.index, sim.tick, me.gen, SALT_CIRCLE ^ k);
    (0..4u32).find_map(|k| {
        let (sa, ca) = fmath::sin_cos(draw(2 * k + 2) * std::f32::consts::TAU);
        let far = target + Vec3::new(ca, 0.0, sa) * (lo + (hi - lo) * draw(2 * k + 3));
        [1.0f32, 0.5].into_iter().find_map(|f| {
            let q = pos + (far - pos) * f;
            let rise = 1.0 + 0.35 * nav::xz_dist(pos, q);
            // With a graph: in the nearest hub a slope from here leads to.
            let q = match sim.nav.as_ref() {
                Some(g) => {
                    let (_, i) = (0..g.hubs.len() as u16)
                        .filter_map(|i| {
                            let hub = g.hub(i)?;
                            let floor = sim.nav_floor(i)?;
                            let d = nav::xz_dist(hub.pos, q);
                            ((floor - pos.y).abs() <= rise && d <= RING_SNAP).then_some((d, i))
                        })
                        .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))?;
                    let hub = g.hub(i)?;
                    let (sb, cb) = fmath::sin_cos(draw(2 * k + 10) * std::f32::consts::TAU);
                    let r = hub.radius.min(HOLD_SPOT_RADIUS) * draw(2 * k + 11).sqrt();
                    Vec3::new(hub.pos.x + cb * r, pos.y, hub.pos.z + sb * r)
                }
                None => q,
            };
            let len = nav::xz_dist(pos, q);
            // (A hub's point may lie beyond the ring: still no backing away.)
            if len < 2.0 || nav::xz_dist(q, target) > d + RING_BACK {
                return None;
            }
            let y = sim.ground_point_under(Vec3::new(q.x, pos.y + rise, q.z), false);
            let q = Vec3::new(q.x, y, q.z);
            let ok = (y - pos.y).abs() <= rise
                && !sim.level_at(q).is_off()
                && sim.water_level.is_none_or(|w| y >= w - 0.2)
                && sim.nav.as_ref().is_none_or(|g| {
                    !g.barriers
                        .iter()
                        .any(|b| b.blocks(nav::SOLDIER) && b.contains(Vec2::new(q.x, q.z)))
                })
                && spot_free(sim, me, q);
            ok.then_some(q)
        })
    })
}

/// Card FGHT, OURS (the maintainer, 2026-10-05): where a bot STANDS to shoot — where
/// it is, unless that is within [`FIGHT_SPACING`] of another fighter's
/// position or of a player: then the first of eight drawn points within
/// [`FIGHT_STEP`] that is clear of them and can be stepped to.
fn stand_spot(sim: &Sim, me: UnitHandle, u: &Unit) -> Vec3 {
    let pos = u.soldier.pos;
    let clear = |p: Vec3| spot_clear_of(sim, me, p, FIGHT_SPACING, Some(u.team));
    if clear(pos) {
        return pos;
    }
    (0..8u32)
        .find_map(|k| {
            let a = unit_unit(me.index, sim.tick, me.gen, SALT_STYLE ^ (2 * k + 2));
            let r = unit_unit(me.index, sim.tick, me.gen, SALT_STYLE ^ (2 * k + 3));
            let (sa, ca) = fmath::sin_cos(a * std::f32::consts::TAU);
            let p = pos + Vec3::new(ca, 0.0, sa) * (FIGHT_STEP * (0.4 + 0.6 * r));
            (stand_ok(sim, u, pos, p) && clear(p)).then_some(p)
        })
        .unwrap_or(pos)
}

/// Card HOLD: can a soldier step from `from` to `p` and stand there — a floor
/// within a metre of `from`'s height, no wall on the way, no `BARR` barrier
/// crossed?
fn stand_ok(sim: &Sim, u: &Unit, from: Vec3, p: Vec3) -> bool {
    let f = sim.ground_point_under(Vec3::new(p.x, from.y + 1.0, p.z), false);
    if (f - from.y).abs() > 1.0 {
        return false;
    }
    let d = Vec3::new(p.x - from.x, 0.0, p.z - from.z);
    let len = d.length();
    if len < 1e-3 {
        return true;
    }
    let mut cap = u.capsule().stepped();
    cap.base = Vec3::new(from.x, cap.base.y - u.soldier.pos.y + from.y, from.z);
    if sim.collision.wall_sweep(&cap, d / len, len).is_some() {
        return false;
    }
    sim.nav.as_ref().is_none_or(|g| {
        g.barrier_stop(Vec2::new(from.x, from.z), Vec2::new(p.x, p.z), nav::SOLDIER) >= 1.0
    })
}

/// Card REACT (the maintainer, 2026-10-07: "if it doesn't die after being hit, it
/// just stays standing there"): what a bot on a position does about a HIT —
/// one draw per hit outside the cooldown ([`REACT_COOLDOWN`]: a burst is one
/// draw) over its tier's row ([`crate::skill::Tier::react`]): it STAYS, it
/// STRAFES ([`HOLD_STRAFE`] aside, the other way from its last step), it
/// ROLLS aside and stands where the roll ends, or it MOVES to a new
/// position — a point of the ring round the target it sees
/// ([`ring_point`], walked as a circler's; [`FightState::MOVE`]), and with
/// no target in sight, a scoped weapon or no ring point, a spot within
/// [`REACT_STEP`], in a hub of the graph as the roll's end is ([`in_hub`]).
/// A roll with no floor for it and a move with nowhere to go are a strafe; a bot already walking to a ring point rolls or walks on.
/// It answers the HIT and never looks for the shooter: the target is the
/// one it already sees. Returns the roll's side for the mover (0 = none).
///
/// OURS, a named deviation (`analysis/bot-design.md` 9): the PS2's soldier
/// ignores a damage message while it attacks (`analysis/re-soldier-ai.md`,
/// the fight row) and re-plans by its skill's `lerp(0, 0.2)` coin otherwise —
/// that coin is still the dodge of a bot walking its leg (`think`).
fn react(
    sim: &Sim,
    tk: &Tick,
    st: &mut AiState,
    mut h: HoldState,
    enemy: Option<&Unit>,
    nt: &mut NavTick,
) -> i8 {
    let Tick {
        me, u, wc, tier, ..
    } = *tk;
    st.dodge_until = sim.tick + REACT_COOLDOWN;
    let [strafes, rolls, moves] = tier.react;
    let x = unit_unit(me.index, sim.tick, me.gen, SALT_REACT);
    if x >= strafes + rolls + moves {
        return 0;
    }
    let pos = u.soldier.pos;
    let walking = st.fight.walks();
    let (sy, cy) = fmath::sin_cos(u.soldier.yaw);
    let right = Vec3::new(cy, 0.0, -sy);
    let side: i8 = if h.strafe_side > 0 { -1 } else { 1 };
    if x < moves && !walking {
        let ring = enemy
            .filter(|_| st.fight.mode != 0 && st.cmd.is_none() && !wc.ai_distance_table())
            .and_then(|e| ring_point(sim, tk, e.soldier.pos))
            .filter(|p| !sim.death_regions.iter().any(|r| r.contains(*p + Vec3::Y)));
        if let Some(p) = ring {
            h.spot = p;
            h.goal = p;
            h.commit_until = sim.tick + HOLD_COMMIT;
            h.blocked = 0;
            st.fight.mode = FightState::MOVE;
            super::route::retarget_board(sim, me, st, p, nt);
            st.hold = Some(h);
            return 0;
        }
        let near = (0..8u32).find_map(|k| {
            let a = unit_unit(me.index, sim.tick, me.gen, SALT_REACT ^ (2 * k + 2));
            let r = unit_unit(me.index, sim.tick, me.gen, SALT_REACT ^ (2 * k + 3));
            let (sa, ca) = fmath::sin_cos(a * std::f32::consts::TAU);
            let p = pos + Vec3::new(ca, 0.0, sa) * (REACT_STEP * (0.5 + 0.5 * r));
            (react_ok(sim, u, p) && in_hub(sim, p) && spot_free(sim, me, p)).then_some(p)
        });
        if let Some(p) = near {
            h.spot = p;
            h.step_to(p, sim.tick, true);
            st.hold = Some(h);
            return 0;
        }
    } else if x >= moves && x < moves + rolls {
        let lands = |s: i8| pos + right * (f32::from(s) * ROLL_REACH);
        if let Some(s) = [side, -side].into_iter().find(|s| {
            roll_clear(sim, u, *s) && react_ok(sim, u, lands(*s)) && in_hub(sim, lands(*s))
        }) {
            if walking {
                return s;
            }
            // It sets off for where the roll ends, stands there, and rolls
            // once it is under way (`think`).
            let p = lands(s);
            h.spot = p;
            h.step_to(p, sim.tick, true);
            h.strafe_side = s;
            h.roll_at = sim.tick + ROLL_LEAD;
            st.hold = Some(h);
            return 0;
        }
    }
    if !walking {
        for s in [side, -side] {
            let p = pos + right * (f32::from(s) * HOLD_STRAFE);
            if react_ok(sim, u, p) && spot_free(sim, me, p) {
                h.step_to(p, sim.tick, true);
                h.strafe_side = s;
                st.hold = Some(h);
                break;
            }
        }
    }
    0
}

/// Card REACT: may a bot on a position step, roll or move from where it
/// stands to `p` (a reaction's, and the shuffle, the strafe on a burst and
/// the step out of a friend's line) — [`stand_ok`], `p` on the level, and
/// no metre of the way inside a mission death region?
fn react_ok(sim: &Sim, u: &Unit, p: Vec3) -> bool {
    let from = u.soldier.pos;
    let n = nav::xz_dist(from, p).ceil().max(1.0) as u32;
    stand_ok(sim, u, from, p)
        && !sim.level_at(p).is_off()
        && (1..=n).all(|k| {
            let q = from + (p - from) * (k as f32 / n as f32);
            let q = Vec3::new(q.x, from.y, q.z);
            !sim.death_regions
                .iter()
                .any(|r| r.contains(q) || r.contains(q + Vec3::Y))
        })
}

/// Card REACT: does `p` lie in a hub of the graph, on the hub's floor (any
/// point, with no graph)? A roll or a move ends on the authored graph: on
/// Bespin Platforms 4 of 55 rolls that ended beside it were followed by a
/// fall from the walk that came next (4 seeds, traced).
fn in_hub(sim: &Sim, p: Vec3) -> bool {
    sim.nav.as_ref().is_none_or(|g| {
        (0..g.hubs.len() as u16).any(|i| {
            g.hub(i)
                .is_some_and(|hub| nav::xz_dist(hub.pos, p) <= hub.radius)
                && sim.nav_floor(i).is_some_and(|f| (f - p.y).abs() <= 1.0)
        })
    })
}

/// Card HOLD: is there [`ROLL_REACH`] of floor to roll over toward `side`
/// (+1 right, −1 left of the facing) — level floor every metre and to each
/// side of the line, no wall, no barrier?
fn roll_clear(sim: &Sim, u: &Unit, side: i8) -> bool {
    let (sy, cy) = fmath::sin_cos(u.soldier.yaw);
    let dir = Vec3::new(cy, 0.0, -sy) * f32::from(side);
    let fwd = Vec3::new(-sy, 0.0, -cy);
    let pos = u.soldier.pos;
    // Card REACT: a roll that carries (it did not before: `ROLL_LEAD`) ends
    // a metre or two past its reach and off its line — so floor ON THE
    // LEVEL, in a lane [`ROLL_LANE`] to each side, to [`ROLL_PAST`] beyond
    // (the first sweep with rolls that travel: Bespin Platforms' falls
    // 196 -> 261 in 32 rounds).
    let n = (ROLL_REACH + ROLL_PAST).ceil() as u32;
    (1..=n).all(|k| {
        [-ROLL_LANE, 0.0, ROLL_LANE].into_iter().all(|lane| {
            let p = pos + dir * k as f32 + fwd * lane;
            (sim.ground_point_under(p + Vec3::Y, false) - pos.y).abs() <= 1.0
                && !sim.level_at(Vec3::new(p.x, pos.y, p.z)).is_off()
        })
    }) && stand_ok(sim, u, pos, pos + dir * ROLL_REACH)
}

/// Card HOLD: the point to run to from a grenade on the ground
/// ([`crate::sim::Hazard`]) — the nearest one this bot stands inside the
/// warning radius of, has noticed (the tier's chance, one coin per bot per
/// grenade) and has had the tier's reaction delay to act on: straight away
/// from it, to 3 m past its radius. The PS2 warns every AI unit within 7.5 m
/// after a difficulty-ranked delay (re-grenades.md §7.4); what its
/// controller then does is not traced, and running clear is ours.
fn hazard_flee(sim: &Sim, me: UnitHandle, u: &Unit, tier: &crate::skill::Tier) -> Option<Vec3> {
    let pos = u.soldier.pos;
    let mut best: Option<(f32, Vec3)> = None;
    for z in &sim.hazards {
        let d = pos - z.pos;
        let dist = Vec2::new(d.x, d.z).length();
        if dist >= z.radius || d.y.abs() > z.radius {
            continue;
        }
        // Card GRAI (OURS): the thrower knows its own grenade — no coin, no
        // delay (the PS2 leaves the thrower out of the warning altogether).
        let own = z.owner == Some(me);
        if !own && unit_unit(me.index, z.since, me.gen, SALT_GRENADE) >= tier.grenade {
            continue;
        }
        let (lo, hi) = tier.grenade_delay;
        let wait = unit_unit(me.index, z.since, me.gen, SALT_GRENADE ^ 1);
        if !own && sim.tick < z.since + lo + ((hi - lo) as f32 * wait) as u64 {
            continue;
        }
        let away = if dist > 1e-3 {
            Vec3::new(d.x, 0.0, d.z) / dist
        } else {
            // On top of it: back the way the bot faces away from.
            let (sy, cy) = fmath::sin_cos(u.soldier.yaw);
            Vec3::new(sy, 0.0, cy)
        };
        if best.is_none_or(|(bd, _)| dist < bd) {
            best = Some((dist, pos + away * (z.radius - dist + 3.0)));
        }
    }
    best.map(|b| b.1)
}

/// Card BNDS: should a lost bot JUMP this tick? Its way `d` is shut by a wall
/// low enough to clear (the sweep of its capsule lifted [`LOST_JUMP_CLEAR`]
/// is open) — a rim rail from outside, a kerb over the step. It jumps from
/// where it stands: the stick steers a jump in the air (card AIRC), and on
/// BF2 Kamino's 50 rim stops a standing jump with the stick held clears
/// every one a running jump does. One try per [`LOST_JUMP_AGAIN`]
/// (`LostState::jump_after`); between tries the steering slides it along.
pub(super) fn lost_jump(sim: &Sim, u: &Unit, d: Vec3, pick: SteerPick, l: &mut LostState) -> bool {
    if pick == SteerPick::Fwd || !u.soldier.grounded || sim.tick < l.jump_after {
        return false;
    }
    let cap = u.capsule();
    if sim
        .collision
        .wall_sweep(&cap.stepped(), d, PROBE_DIST)
        .is_none()
    {
        return false; // a drop or a barrier, not a wall
    }
    let mut over = cap;
    over.base += Vec3::Y * LOST_JUMP_CLEAR;
    if sim
        .collision
        .wall_sweep(&over, d, PROBE_DIST + 0.5)
        .is_some()
    {
        return false; // too high to jump
    }
    l.jump_after = sim.tick + LOST_JUMP_AGAIN;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::tests::{all_circle, all_stand, l_sim, Style};

    /// Card FGHT (it was card HOLD's "at a hub"): a bot that sees an enemy
    /// inside its weapon's reach takes a position and fights from it — it
    /// stops walking its route — and once the enemy is gone and the position
    /// has been quiet for `HOLD_QUIET` it walks on.
    #[test]
    fn a_bot_fights_from_a_position_while_it_sees_an_enemy_and_then_advances() {
        // A stander: it keeps its place.
        let _style = all_stand();
        use crate::sim::PadState;
        use crate::unit::UnitSpec;
        let (mut sim, h) = l_sim();
        // A dummy 75 m up the first leg: out of sight at the start (49 m),
        // in sight after some 26 m of it.
        let e = sim
            .spawn(&UnitSpec::trooper(1, Vec3::new(0.0, 0.0, -75.0)))
            .expect("enemy");
        let mut held_at = None;
        for _ in 0..600 {
            sim.step(&[PadState::default()]);
            sim.unit_mut(e).expect("enemy").health.cur = 300.0;
            let u = sim.unit(h).expect("alive");
            if u.ai.hold.is_some_and(|x| sim.tick < x.until) {
                held_at = Some(u.soldier.pos);
                break;
            }
        }
        let p0 = held_at.expect("it took a position");
        let seen_from = nav::xz_dist(p0, Vec3::new(0.0, 0.0, -75.0));
        assert!(
            (40.0..50.0).contains(&seen_from),
            "on sight, inside the weapon's reach: {seen_from} m from the enemy"
        );
        // Eight seconds on it is still there.
        for _ in 0..240 {
            sim.step(&[PadState::default()]);
            sim.unit_mut(e).expect("enemy").health.cur = 300.0;
        }
        let u = sim.unit(h).expect("alive");
        assert!(
            u.ai.hold.is_some_and(|x| sim.tick < x.until),
            "still on its position while the enemy stands there"
        );
        assert_eq!(u.ai.behaviour, Behaviour::Fight);
        let held = u.soldier.pos;
        assert!(
            nav::xz_dist(held, p0) < crate::ai::HOLD_SHUFFLE + HOLD_STRAFE + 1.0,
            "it has not walked on: {held} from {p0}"
        );
        // The enemy goes; the position goes quiet; the bot advances to the post.
        sim.units[e.index as usize].alive = false;
        for _ in 0..900 {
            sim.step(&[PadState::default()]);
        }
        let u = sim.unit(h).expect("alive");
        assert!(
            nav::xz_dist(u.soldier.pos, Vec3::new(40.0, 0.0, -40.0)) < 8.0,
            "it went on to the post: {}",
            u.soldier.pos
        );
    }

    /// Flat ground, no graph: `n` AI troopers of team 0 a metre and a half
    /// apart at the origin, facing an enemy dummy `d` metres up -Z.
    fn fight_scene(n: usize, d: f32) -> (Sim, Vec<UnitHandle>, UnitHandle) {
        use crate::heightfield::Heightfield;
        use crate::unit::{Controller, UnitSpec};
        let mut sim = Sim::empty(11);
        sim.heightfield = Some(Heightfield {
            size: 2,
            unit: 400.0,
            offset_x: 200.0,
            offset_z: 200.0,
            heights: vec![0.0; 4],
        });
        let bots = (0..n)
            .map(|i| {
                sim.spawn(&UnitSpec {
                    controller: Controller::Ai,
                    ..UnitSpec::trooper(0, Vec3::new(1.5 * i as f32, 0.0, 0.0))
                })
                .expect("bot")
            })
            .collect();
        let e = sim
            .spawn(&UnitSpec::trooper(1, Vec3::new(0.0, 0.0, -d)))
            .expect("enemy");
        (sim, bots, e)
    }

    /// Card REACT: a stander in a fight, HIT once at tick `at` (Normal's
    /// sight; the tier is `d`). What it did about it: 0 stayed, 1 strafed
    /// (its goal 2.5 m aside, its spot kept), 2 rolled (its spot the roll's
    /// 10 m aside), 3 moved (on its way to a ring point, or a spot 3.5-7 m
    /// off) — and the tick its next reaction may come.
    fn hit_once(d: crate::skill::Difficulty, at: u64) -> (usize, u64) {
        use crate::sim::PadState;
        // The same scene twice: without the hit, and with it.
        let run = |hit: bool| {
            let (mut sim, bots, e) = fight_scene(1, 30.0);
            sim.difficulty = d;
            while sim.tick < at {
                sim.step(&[PadState::default()]);
                sim.unit_mut(e).expect("enemy").health.cur = 300.0;
            }
            let pos = sim.unit(bots[0]).expect("alive").soldier.pos;
            if hit {
                sim.unit_mut(bots[0]).expect("alive").last_hit = Some((e, at - 1));
            }
            sim.step(&[PadState::default()]);
            (sim.unit(bots[0]).expect("alive").ai, pos)
        };
        let ((calm, _), (st, pos)) = (run(false), run(true));
        assert_eq!(calm.fight.mode, FightState::STAND, "it stands to shoot");
        let (h0, h) = (calm.hold.expect("a position"), st.hold.expect("still"));
        let (spot, goal) = (nav::xz_dist(h.spot, pos), nav::xz_dist(h.goal, pos));
        let what = match () {
            _ if st.fight.mode == FightState::MOVE => 3,
            _ if h.roll_at != 0 => {
                assert_eq!(h.roll_at, at + 6, "the roll's lead");
                assert!((spot - 10.0).abs() < 0.01 && spot == goal, "{spot} {goal}");
                2
            }
            _ if h.spot != h0.spot => {
                assert!(
                    (3.49..=7.01).contains(&spot) && spot == goal,
                    "{spot} {goal}"
                );
                3
            }
            _ if h.goal != h0.goal => {
                assert!((goal - 2.5).abs() < 0.01, "a strafe of {goal} m");
                1
            }
            _ => {
                assert_eq!((h, st.fight), (h0, calm.fight), "an unnamed reaction");
                0
            }
        };
        (what, st.dodge_until)
    }

    /// Card REACT (the maintainer, 2026-10-07): a bot that stands to shoot answers a
    /// hit by its tier's row — it stays, strafes, rolls or moves in the
    /// row's shares (400 hits a tier, each within 7 points of the row, as
    /// numbers: the row's own constants would follow a broken table), most
    /// at Easy stay and fewest at Elite, and every hit arms the 1.5 s
    /// cooldown, a stay too.
    #[test]
    fn a_hit_stander_stays_strafes_rolls_or_moves_by_its_tiers_row() {
        use crate::skill::Difficulty as D;
        let _style = all_stand();
        let rows = [
            (D::Easy, [0.65, 0.20, 0.05, 0.10]),
            (D::Normal, [0.35, 0.30, 0.15, 0.20]),
            (D::Hard, [0.20, 0.30, 0.20, 0.30]),
            (D::Elite, [0.10, 0.30, 0.25, 0.35]),
        ];
        let mut stays = Vec::new();
        for (d, row) in rows {
            let mut n = [0u32; 4];
            for k in 0..400u64 {
                let at = 90 + k;
                let (what, until) = hit_once(d, at);
                n[what] += 1;
                assert_eq!(until, at + 45, "{d:?}: the cooldown after a {what}");
            }
            for (i, want) in row.iter().enumerate() {
                let got = n[i] as f32 / 400.0;
                assert!(
                    (got - want).abs() < 0.07,
                    "{d:?}: reaction {i} in {got} of the hits, the row says {want} ({n:?})"
                );
            }
            stays.push(n[0]);
        }
        assert!(stays.windows(2).all(|w| w[0] > w[1]), "stays {stays:?}");
    }

    /// Card REACT: a burst is ONE draw — a second hit inside the cooldown
    /// changes nothing; and a stander under steady fire for 20 s (a hit
    /// every third of a second, Normal) does not stand still for long
    /// (the maintainer: "it just stays standing there"), keeps its fight, and when it
    /// was sent to a new position stands to shoot again once it is there.
    #[test]
    fn steady_fire_moves_a_stander_about_and_it_fights_on() {
        use crate::sim::PadState;
        let _style = all_stand();
        let (mut sim, bots, e) = fight_scene(1, 30.0);
        let b = bots[0];
        for _ in 0..90 {
            sim.step(&[PadState::default()]);
            sim.unit_mut(e).expect("enemy").health.cur = 300.0;
        }
        let (mut last, mut path) = (sim.unit(b).expect("alive").soldier.pos, 0.0f32);
        let (mut still, mut longest, mut walked, mut again) = (0u32, 0u32, 0u32, 0u32);
        for k in 0..600u64 {
            let before = sim.unit(b).expect("alive").ai;
            // (Inside the cooldown every tick: the answer must not change.)
            let cool = sim.tick < before.dodge_until;
            if k % 10 == 0 || cool {
                sim.unit_mut(b).expect("alive").last_hit = Some((e, sim.tick - 1));
            }
            sim.step(&[PadState::default()]);
            sim.unit_mut(e).expect("enemy").health.cur = 300.0;
            sim.unit_mut(b).expect("alive").health.cur = 300.0;
            let u = sim.unit(b).expect("alive");
            if cool {
                assert_eq!(u.ai.dodge_until, before.dodge_until, "re-armed");
                again += u32::from(
                    u.ai.hold.map(|h| h.spot) != before.hold.map(|h| h.spot)
                        && u.ai.fight.mode == before.fight.mode
                        && u.ai.fight.mode == FightState::STAND,
                );
            }
            walked += u32::from(u.ai.fight.mode == FightState::MOVE);
            assert_eq!(u.ai.behaviour, Behaviour::Fight, "tick {}", sim.tick);
            let step = nav::xz_dist(u.soldier.pos, last);
            (last, path) = (u.soldier.pos, path + step);
            // (Standing: under 0.5 m/s.)
            still = if step < 0.5 / 30.0 { still + 1 } else { 0 };
            longest = longest.max(still);
        }
        assert_eq!(again, 0, "a stander's spot moved inside the cooldown");
        // (Card TPOS: 40 m with the shuffle; without it 39.3.)
        assert!(path > 30.0, "{path} m in 20 s of fire");
        assert!(longest < 200, "{longest} ticks standing still under fire");
        assert!(walked > 0, "it never walked to a new position");
        // The fire stops: it ends its walk and stands to shoot.
        for _ in 0..450 {
            sim.step(&[PadState::default()]);
            sim.unit_mut(e).expect("enemy").health.cur = 300.0;
        }
        let u = sim.unit(b).expect("alive");
        assert_eq!(u.ai.fight.mode, FightState::STAND);
        assert!(nav::xz_dist(u.soldier.pos, u.ai.hold.expect("a position").goal) < 1.0);
    }

    /// Card REACT: the roll CARRIES the bot — pressed once it is under way
    /// (a soldier standing still rolls on the spot), it is in the roll a
    /// third of a second after the hit and some 10 m aside when it stands
    /// again, on its new spot.
    #[test]
    fn a_hit_bots_roll_carries_it_aside() {
        use crate::sim::PadState;
        use crate::soldier::Stance;
        let _style = all_stand();
        let d = crate::skill::Difficulty::Normal;
        let at = (90..200)
            .find(|at| hit_once(d, *at).0 == 2)
            .expect("a roll");
        let (mut sim, bots, e) = fight_scene(1, 30.0);
        sim.difficulty = d;
        while sim.tick < at {
            sim.step(&[PadState::default()]);
            sim.unit_mut(e).expect("enemy").health.cur = 300.0;
        }
        let from = sim.unit(bots[0]).expect("alive").soldier.pos;
        sim.unit_mut(bots[0]).expect("alive").last_hit = Some((e, at - 1));
        let mut rolling = 0;
        for k in 0..60 {
            sim.step(&[PadState::default()]);
            let u = sim.unit(bots[0]).expect("alive");
            rolling += u32::from(u.soldier.stance == Stance::Roll);
            if k == 10 {
                assert_eq!(u.soldier.stance, Stance::Roll, "not rolling yet");
            }
        }
        let u = sim.unit(bots[0]).expect("alive");
        let gone = nav::xz_dist(u.soldier.pos, from);
        assert!(rolling > 20, "{rolling} ticks in the roll");
        assert!(gone > 8.0 && gone < 12.0, "{gone} m aside after the roll");
        assert!(nav::xz_dist(u.soldier.pos, u.ai.hold.expect("a position").spot) < 2.5);
    }

    /// Card REACT: no step, roll or move of a bot on a position goes into
    /// or through a mission death region.
    #[test]
    fn a_reaction_never_goes_through_a_death_region() {
        use crate::region::Region;
        let (mut sim, bots, _) = fight_scene(1, 30.0);
        let p = Vec3::new(2.5, 0.0, 0.0);
        let ok = |sim: &Sim, p: Vec3| react_ok(sim, sim.unit(bots[0]).expect("alive"), p);
        assert!(ok(&sim, p) && ok(&sim, p * 4.0));
        sim.death_regions = vec![Region::sphere(Vec3::new(5.0, 0.0, 0.0), 1.0)];
        assert!(ok(&sim, p), "short of it");
        assert!(!ok(&sim, p * 2.0), "into it");
        assert!(!ok(&sim, p * 4.0), "through it");
    }

    /// Card FGHT (`analysis/re-soldier-ai.md` 4.5): a bot whose style is to
    /// STAND keeps its place while it fights; one whose style is to CIRCLE
    /// walks to points of a ring round its target and pauses on them — and
    /// never ends up farther from the target than it began (OURS: the ring is
    /// no wider than its own distance).
    #[test]
    fn a_stander_keeps_its_place_and_a_circler_walks_a_ring_round_its_target() {
        use crate::sim::PadState;
        let run = |style: Style| {
            let (mut sim, bots, e) = fight_scene(1, 30.0);
            let (mut path, mut last, mut far, mut modes) = (0.0f32, Vec3::ZERO, 0.0f32, [0u32; 4]);
            let (mut fights, mut strayed) = (0u32, 0.0f32);
            for _ in 0..600 {
                sim.step(&[PadState::default()]);
                sim.unit_mut(e).expect("enemy").health.cur = 300.0;
                let u = sim.unit(bots[0]).expect("alive");
                path += nav::xz_dist(u.soldier.pos, last);
                last = u.soldier.pos;
                far = far.max(nav::xz_dist(last, Vec3::new(0.0, 0.0, -30.0)));
                strayed = strayed.max(nav::xz_dist(last, Vec3::ZERO));
                modes[usize::from(u.ai.fight.mode)] += 1;
                fights += u32::from(u.ai.behaviour == Behaviour::Fight);
            }
            drop(style);
            assert!(fights > 590, "it fights throughout: {fights} of 600 ticks");
            (path, far, modes, strayed)
        };
        // (Its steps on the position are card HOLD's shuffle and strafe.)
        let (_, _, modes, strayed) = run(all_stand());
        assert!(
            strayed <= crate::ai::HOLD_SHUFFLE + HOLD_STRAFE + 0.5,
            "a stander keeps its place: {strayed} m from it"
        );
        assert!(modes[usize::from(FightState::STAND)] > 590, "{modes:?}");
        let (path, far, modes, _) = run(all_circle());
        assert!(path > 20.0, "a circler walks: {path} m in 20 s");
        assert!(
            far < 31.5,
            "never farther from its target than it began: {far} m"
        );
        assert!(
            modes[usize::from(FightState::WALK)] > 100
                && modes[usize::from(FightState::PAUSE)] > 50,
            "it walks and it pauses: {modes:?}"
        );
    }

    /// Card FGHT, OURS (the maintainer, 2026-10-05: "everyone stops in their tracks
    /// which puts them all right next to each other"): bots that stand to
    /// shoot do not stand side by side — the first keeps its place, every
    /// next one takes a spot [`FIGHT_SPACING`] clear of the others'.
    #[test]
    fn standers_do_not_stop_beside_each_other() {
        use crate::sim::PadState;
        let _style = all_stand();
        // The enemy steps into sight a second in, so each bot sees it on its
        // own think (a fight starts on a think; the thinks are a tick apart).
        let (mut sim, bots, e) = fight_scene(4, 300.0);
        // A second after that every position is taken, and no shuffle has run.
        for t in 0..60 {
            sim.step(&[PadState::default()]);
            if t == 30 {
                sim.unit_mut(e).expect("enemy").soldier.pos = Vec3::new(0.0, 0.0, -30.0);
            }
            sim.unit_mut(e).expect("enemy").health.cur = 300.0;
            for b in &bots {
                sim.unit_mut(*b).expect("bot").health.cur = 300.0;
            }
        }
        let spots: Vec<Vec3> = bots
            .iter()
            .map(|b| {
                let u = sim.unit(*b).expect("alive");
                assert_eq!(u.ai.fight.mode, FightState::STAND);
                u.ai.hold.expect("a position").spot
            })
            .collect();
        assert_eq!(spots[0], Vec3::ZERO, "the first keeps its place");
        for (i, a) in spots.iter().enumerate() {
            for b in &spots[i + 1..] {
                // (4 m is `FIGHT_SPACING`, as a number: the rule's own
                // constant would follow a broken rule.)
                assert!(
                    nav::xz_dist(*a, *b) >= 3.99,
                    "two standers' spots {:.1} m apart: {spots:?}",
                    nav::xz_dist(*a, *b)
                );
            }
        }
    }

    /// Card FGHT: a ring point lies in the PS2's band round the weapon's
    /// optimal range (`FUN_002d9228`) from a far target, and no farther from
    /// a near target than the bot itself is.
    #[test]
    fn a_ring_point_is_on_the_weapons_ring_and_no_wider_than_the_bots_distance() {
        for (d, lo, hi) in [(45.0f32, 24.0f32, 36.0f32), (10.0, 0.0, 10.0)] {
            let (mut sim, bots, _) = fight_scene(1, d);
            for tick in [1u64, 40, 400, 4000, 7, 77, 777, 7777] {
                sim.tick = tick;
                let class = sim.unit(bots[0]).expect("alive").weapon_class;
                // The class defaults: a ring of 24 to 36 m.
                sim.weapon_classes[class].ai_range = [0.0, 30.0, 60.0];
                let u = sim.unit(bots[0]).expect("alive");
                let wc = &sim.weapon_classes[u.weapon_class];
                let tk = Tick {
                    me: bots[0],
                    u,
                    eye: u.muzzle(),
                    wc,
                    tier: sim.difficulty.tier(),
                    skill: sim.difficulty.skill_at(u.skill),
                };
                let target = Vec3::new(0.0, 0.0, -d);
                let p = ring_point(&sim, &tk, target).expect("open ground: a point");
                let r = nav::xz_dist(p, target);
                // (The half-way try is only for a point that cannot be had.)
                assert!(
                    (lo - 0.01..=hi + 0.01).contains(&r),
                    "target at {d} m: a ring point {r} m from it, band {lo}..{hi}"
                );
            }
        }
    }

    /// The fight modes of bot 0 over 20 s ([none, stand, walk, pause] ticks)
    /// with its target `d` metres off and its weapon's AI ranges `ai`
    /// (`scoped`: a single-shot zoom weapon, the AI's sniper table).
    fn modes_at(d: f32, ai: [f32; 3], scoped: bool) -> [u32; 4] {
        use crate::sim::PadState;
        let (mut sim, bots, e) = fight_scene(1, d);
        let class = sim.unit(bots[0]).expect("alive").weapon_class;
        let c = &mut sim.weapon_classes[class];
        c.ai_range = ai;
        if scoped {
            (c.zoom_max, c.trigger_single, c.yaw_spread) = (8.0, true, 0.0);
        }
        assert_eq!(c.ai_distance_table(), scoped);
        let mut modes = [0u32; 4];
        for _ in 0..600 {
            sim.step(&[PadState::default()]);
            sim.unit_mut(e).expect("enemy").health.cur = 300.0;
            sim.unit_mut(bots[0]).expect("bot").health.cur = 300.0;
            modes[usize::from(sim.unit(bots[0]).expect("alive").ai.fight.mode)] += 1;
        }
        modes
    }

    /// Card FGHTA (`analysis/re-soldier-ai.md` 4.4): a bot whose style is to
    /// stand stands only while its target is inside the middle of its
    /// weapon's band — for [0, 24, 64] between 12 and 44 m; nearer or farther
    /// it takes ring points as a circler does. The style is forced, the band
    /// is the rule's own.
    #[test]
    fn a_stander_circles_outside_the_middle_of_its_weapons_band() {
        let _style = crate::ai::tests::all_standers();
        let stand = usize::from(FightState::STAND);
        // (A standing soldier is seen from 49 m: the far case is at 47.)
        for (d, stands) in [(10.0f32, false), (30.0, true), (47.0, false)] {
            let modes = modes_at(d, [0.0, 24.0, 64.0], false);
            if stands {
                assert!(modes[stand] >= 590, "at {d} m it stands: {modes:?}");
            } else if d < 12.0 {
                // (Its ring is no wider than its own distance: never in.)
                assert_eq!(modes[stand], 0, "at {d} m it circles: {modes:?}");
            } else {
                // It walks in to a ring point, and stands once in the band.
                let walk = modes[usize::from(FightState::WALK)];
                assert!(walk >= 10, "at {d} m it walks to a ring point: {modes:?}");
            }
        }
    }

    /// Card FGHTA: a scoped rifle stands wherever its target is in range —
    /// the PS2's scout rule (`FUN_002abc38`), by the bot's own draw. It was a
    /// circler inside (MinRange + OptimalRange) / 2 = 36 m: on its spot for 0
    /// of 600 ticks at 12, 20 and 30 m. An unscoped weapon of the same ranges
    /// still follows the band; out of range the scoped one circles (OURS).
    #[test]
    fn a_scoped_rifle_stands_wherever_its_target_is_in_range() {
        let stand = usize::from(FightState::STAND);
        for d in [12.0f32, 20.0, 30.0, 45.0] {
            let modes = modes_at(d, [8.0, 64.0, 128.0], true);
            assert!(modes[stand] >= 590, "scoped, at {d} m: {modes:?}");
        }
        let modes = modes_at(5.0, [8.0, 64.0, 128.0], true);
        assert_eq!(modes[stand], 0, "scoped, inside MinRange: {modes:?}");
        let _style = crate::ai::tests::all_standers();
        let modes = modes_at(20.0, [8.0, 64.0, 128.0], false);
        assert_eq!(modes[stand], 0, "unscoped, under the band: {modes:?}");
    }

    /// Card FGHTA: the style's draw — half the bots stand (Easy: three
    /// quarters), a third of them would crouch, and a scoped weapon always
    /// stands. Over 200 unit indices; the numbers are the PS2's weights
    /// (`analysis/re-soldier-ai.md` 4.4), not the rule's constants.
    #[test]
    fn half_the_bots_stand_three_quarters_on_easy_and_a_third_crouch() {
        use crate::skill::Difficulty;
        let share = |d: Difficulty, k: usize| {
            let n = (0..200u32)
                .filter(|&index| {
                    let s = style_of(UnitHandle { index, gen: 1 }, d, false);
                    [s.0, s.1][k]
                })
                .count();
            n as f32 / 200.0
        };
        for d in [Difficulty::Normal, Difficulty::Hard, Difficulty::Elite] {
            let s = share(d, 0);
            assert!((0.4..=0.6).contains(&s), "{d:?}: {s} stand");
        }
        let s = share(Difficulty::Easy, 0);
        assert!((0.65..=0.85).contains(&s), "Easy: {s} stand");
        let c = share(Difficulty::Normal, 1);
        assert!((0.233..=0.433).contains(&c), "{c} crouch");
        assert!((0..200u32).all(|index| style_of(
            UnitHandle { index, gen: 3 },
            Difficulty::Normal,
            true
        )
        .0));
    }

    /// Card FGHTA: a stander whose draw says so crouches on its spot, and
    /// one whose draw does not stands upright.
    #[test]
    fn a_stander_that_draws_the_crouch_crouches() {
        use crate::sim::PadState;
        use crate::skill::Difficulty;
        for crouches in [false, true] {
            // The first bot index whose draw is a stander of this kind.
            let n = (0..200u32)
                .find(|&index| {
                    style_of(UnitHandle { index, gen: 1 }, Difficulty::Normal, false)
                        == (true, crouches)
                })
                .expect("one in 200") as usize
                + 1;
            let (mut sim, bots, e) = fight_scene(n, 30.0);
            let me = *bots.last().expect("a bot");
            assert_eq!((me.index as usize, me.gen), (n - 1, 1));
            for _ in 0..90 {
                sim.step(&[PadState::default()]);
                sim.unit_mut(e).expect("enemy").health.cur = 300.0;
                for b in &bots {
                    sim.unit_mut(*b).expect("bot").health.cur = 300.0;
                }
            }
            let u = sim.unit(me).expect("alive");
            assert_eq!(u.ai.fight.mode, FightState::STAND);
            let mask = u.ai.hold.expect("a position").stance_mask;
            assert_eq!(
                mask == crate::commander::MASK_CROUCH,
                crouches,
                "bot {n}: mask {mask}"
            );
        }
    }

    /// Card FGHTA (`analysis/re-soldier-ai.md` 4.5): a circler's pause lasts
    /// 1.9 s (57 ticks; the next ring point is drawn on the first think after
    /// it), and on its way it stops for one at the PS2's rate — 1 in 12 per
    /// 0.4 s think, 0.21 a second: about 1 in 14 of the port's thinks, not 1
    /// in 6 and not never.
    #[test]
    fn a_circler_pauses_57_ticks_and_stops_on_its_way_at_the_ps2s_rate() {
        use crate::sim::PadState;
        let _style = all_circle();
        let rate = CIRCLE_STOP * crate::TICK_HZ as f32 / THINK_PERIOD as f32;
        assert!((0.203..=0.214).contains(&rate), "{rate} stops a second");
        let (mut walk_ticks, mut stops, mut pauses) = (0u32, 0u32, Vec::new());
        for n in 1..=6usize {
            // A wide ring (32 to 45 m round a target 45 m off): long walks.
            let (mut sim, bots, e) = fight_scene(n, 45.0);
            let me = *bots.last().expect("a bot");
            let class = sim.unit(me).expect("alive").weapon_class;
            sim.weapon_classes[class].ai_range = [0.0, 40.0, 80.0];
            let (mut run, mut was, mut goal) = (0u64, 0u8, Vec3::ZERO);
            for _ in 0..1800 {
                sim.step(&[PadState::default()]);
                sim.unit_mut(e).expect("enemy").health.cur = 300.0;
                for b in &bots {
                    sim.unit_mut(*b).expect("bot").health.cur = 300.0;
                }
                let u = sim.unit(me).expect("alive");
                let mode = u.ai.fight.mode;
                walk_ticks += u32::from(mode == FightState::WALK);
                if mode == FightState::PAUSE {
                    run += 1;
                    // A pause taken short of the ring point: the chance's.
                    stops += u32::from(
                        was == FightState::WALK && nav::xz_dist(u.soldier.pos, goal) > 4.0,
                    );
                } else if run > 0 {
                    pauses.push(run);
                    run = 0;
                }
                was = mode;
                goal = u.ai.hold.map_or(goal, |h| h.goal);
            }
        }
        assert!(pauses.len() > 20, "pauses: {pauses:?}");
        // (A pause after which no ring point can be had is taken again.)
        let single = pauses.iter().filter(|p| **p < 57 + THINK_PERIOD).count();
        assert!(
            pauses.iter().all(|p| *p >= 57) && 2 * single > pauses.len(),
            "a pause is 57 ticks and ends on the next think: {pauses:?}"
        );
        // 1 in 14.4 thinks; 1 in 6 is 0.167.
        let per_think = stops as f32 * THINK_PERIOD as f32 / walk_ticks as f32;
        assert!(
            (0.04..=0.105).contains(&per_think),
            "{stops} stops in {walk_ticks} walking ticks: {per_think} a think"
        );
    }

    /// Card FGHTA: a seen target is fought to the weapon's MaxRange + 10 m
    /// (`analysis/re-soldier-ai.md` 4.2) — at + 5 m it is, at + 15 m not.
    #[test]
    fn a_target_is_fought_to_ten_metres_past_the_weapons_range() {
        let _style = all_stand();
        let modes = modes_at(35.0, [0.0, 15.0, 30.0], false);
        assert!(
            modes[usize::from(FightState::STAND)] >= 590,
            "at MaxRange + 5: {modes:?}"
        );
        let modes = modes_at(45.0, [0.0, 15.0, 30.0], false);
        assert_eq!(modes[0], 600, "at MaxRange + 15: {modes:?}");
    }

    /// Card TEAM: a circler whose target is past the trigger's reach and
    /// that finds no ring point (water over the ground refuses every one)
    /// walks at its target until it is in reach and fires; it no longer
    /// pauses where it stands for good (BF1 Kashyyyk at Hard, seed 2: four
    /// last soldiers 57 m apart, the round never ended).
    #[test]
    fn a_circler_with_no_ring_point_walks_at_a_target_out_of_reach() {
        use crate::sim::{Event, PadState};
        let _style = all_circle();
        let (mut sim, bots, e) = fight_scene(1, 0.0);
        let reach = {
            let u = sim.unit(bots[0]).expect("bot");
            sim.weapon_classes[u.weapon_class].ai_range[2]
        };
        let start = reach + 5.0;
        sim.unit_mut(e).expect("enemy").soldier.pos = Vec3::new(0.0, 0.0, -start);
        sim.water_level = Some(1.0);
        // (Hard's sight, 61 m standing, sees it - Normal's 49 m does not.)
        sim.difficulty = crate::skill::Difficulty::Hard;
        let (mut near, mut shots) = (start, 0u32);
        for _ in 0..450 {
            sim.step(&[PadState::default()]);
            sim.unit_mut(e).expect("enemy").health.cur = 300.0;
            let u = sim.unit(bots[0]).expect("alive");
            near = near.min(nav::xz_dist(u.soldier.pos, Vec3::new(0.0, 0.0, -start)));
            shots += sim
                .events
                .iter()
                .filter(|v| matches!(v, Event::Shot { unit, .. } if *unit == bots[0]))
                .count() as u32;
        }
        assert!(
            near < reach,
            "it walked into reach: {near} m (reach {reach})"
        );
        assert!(shots > 0, "and fired");
    }

    /// Card FGHTA: with a nav graph a ring point is taken INSIDE a hub — the
    /// one nearest the drawn point, within 10 m of it (`RING_SNAP`) — and
    /// never in a hub that lies more than 2 m farther from the target than
    /// the bot stands (`RING_BACK`). The numbers are written out: the rule's
    /// constants would follow a broken rule.
    #[test]
    fn a_ring_point_on_a_graph_lies_in_a_hub_and_never_farther_back() {
        use crate::sim::PadState;
        // The L graph: hubs (0, 0), (0, -40) and (40, -40), 2 m wide.
        let hubs = [
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, -40.0),
            Vec3::new(40.0, 0.0, -40.0),
        ];
        let points = |at: Vec3, target: Vec3| {
            let (mut sim, h) = l_sim();
            sim.step(&[PadState::default()]);
            sim.unit_mut(h).expect("bot").soldier.pos = at;
            let class = sim.unit(h).expect("alive").weapon_class;
            sim.weapon_classes[class].ai_range = [0.0, 30.0, 60.0];
            (1..=64u64)
                .filter_map(|tick| {
                    sim.tick = 17 * tick;
                    let u = sim.unit(h).expect("alive");
                    let tk = Tick {
                        me: h,
                        u,
                        eye: u.muzzle(),
                        wc: &sim.weapon_classes[u.weapon_class],
                        tier: sim.difficulty.tier(),
                        skill: sim.difficulty.skill_at(u.skill),
                    };
                    ring_point(&sim, &tk, target)
                })
                .collect::<Vec<Vec3>>()
        };
        // From hub 0, a target 44.7 m off between hubs 1 and 2 (20 m from
        // each): its 24-36 m ring passes within 10 m of both.
        let got = points(Vec3::ZERO, Vec3::new(20.0, 0.0, -40.0));
        assert!(got.len() >= 8, "ring points on the graph: {}", got.len());
        for p in &got {
            let near = hubs
                .iter()
                .map(|h| nav::xz_dist(*p, *h))
                .fold(f32::MAX, f32::min);
            assert!(
                near <= 2.01,
                "a ring point {near} m from the nearest hub: {p}"
            );
        }
        // On the first leg 20 m from hub 0, a target 8 m off toward it: the
        // ring is 8 m wide, the only hub within 10 m of it is hub 0, 12 m
        // from the target — every point of it is over 2 m farther back.
        let got = points(Vec3::new(0.0, 0.0, -20.0), Vec3::new(0.0, 0.0, -12.0));
        assert!(got.is_empty(), "a circle does not back away: {got:?}");
    }

    /// Card HOLD: a grenade on the ground sends an Elite bot inside its
    /// warning radius clear of it; a bot outside the radius does not stir.
    #[test]
    fn a_bot_runs_clear_of_a_grenade_on_the_ground() {
        use crate::sim::{PadState, HAZARD_RADIUS};
        use crate::skill::Difficulty;
        let run = |at: Vec3| {
            let (mut sim, h) = l_sim();
            sim.difficulty = Difficulty::Elite;
            // No objective to walk to: the only thing that moves it is the grenade.
            sim.cps.clear();
            sim.nav = None;
            sim.unit_mut(h).expect("alive").skill = Difficulty::Elite.base_skill();
            sim.add_hazard(at, 150);
            for _ in 0..120 {
                sim.step(&[PadState::default()]);
            }
            nav::xz_dist(sim.unit(h).expect("alive").soldier.pos, at)
        };
        let near = run(Vec3::new(2.0, 0.0, 1.0));
        assert!(near > HAZARD_RADIUS, "ran clear: {near:.1} m from it");
        let far = run(Vec3::new(12.0, 0.0, 0.0));
        assert!(
            (far - 12.0).abs() < 0.5,
            "outside the radius it stays: {far:.1}"
        );
    }

    /// Card HOLD: the dodge only rolls where there is floor to roll over.
    #[test]
    fn a_roll_needs_clear_floor() {
        use crate::collision::{Body, CollisionWorld, Shape};
        let (mut sim, h) = l_sim();
        let u = sim.unit(h).expect("alive");
        assert!(
            roll_clear(&sim, u, 1) && roll_clear(&sim, u, -1),
            "open ground"
        );
        // A wall 4 m to the right (+X of a unit facing -Z).
        sim.collision = CollisionWorld::new(
            vec![Body::new(
                glam::Mat4::from_translation(Vec3::new(4.0, 0.0, 0.0)),
                Shape::Box {
                    half: Vec3::new(0.5, 6.0, 30.0),
                },
            )],
            Vec::new(),
        );
        let u = sim.unit(h).expect("alive");
        assert!(!roll_clear(&sim, u, 1), "a wall in the way");
        assert!(roll_clear(&sim, u, -1), "the other side is still open");
    }

    /// Card BNDS: a bot on a hull, 4 m under its deck past the rim barrier,
    /// is OFF the level; two thinks confirm it, it walks back at the nearest
    /// hub and climbs to the deck, and the state is gone once it is on it.
    #[test]
    fn a_bot_off_the_level_walks_back_onto_it() {
        use crate::ai::tests::rim_sim;
        use crate::sim::{Level, PadState};
        let (mut sim, h) = rim_sim(true);
        let y = sim.ground_under(Vec3::new(-18.0, 12.0, 0.0), false);
        sim.unit_mut(h).expect("alive").soldier.pos = Vec3::new(-18.0, y, 0.0);
        assert!(sim.level_at(Vec3::new(-18.0, y, 0.0)).is_off());
        let mut was_lost = false;
        for _ in 0..600 {
            sim.step(&[PadState::default()]);
            let u = sim
                .unit(h)
                .expect("alive: no recycle on a hull it can leave");
            was_lost |= u.ai.lost.is_some_and(|l| l.confirmed);
        }
        let u = sim.unit(h).expect("alive");
        assert!(was_lost, "the return walk was on");
        assert_eq!(sim.level_at(u.soldier.pos), Level::On, "{}", u.soldier.pos);
        assert!(u.soldier.pos.y > 9.5, "back on the deck: {}", u.soldier.pos);
        assert_eq!(u.ai.lost, None);
    }

    /// Card BOWL: the same on the FIRST metres of the hull — 3 m past the
    /// rail and 1.5 m under the deck, where the level test used to say ON
    /// and a bot was never lost. Two thinks confirm it and it walks back.
    #[test]
    fn a_bot_just_past_the_rail_is_lost_and_walks_back() {
        use crate::ai::tests::rim_sim;
        use crate::sim::{Level, PadState};
        let (mut sim, h) = rim_sim(true);
        let y = sim.ground_under(Vec3::new(-13.0, 12.0, 0.0), false);
        assert!(y < 9.0 && y > 8.0, "on the hull: {y}");
        sim.unit_mut(h).expect("alive").soldier.pos = Vec3::new(-13.0, y, 0.0);
        assert!(sim.level_at(Vec3::new(-13.0, y, 0.0)).is_off());
        // Until it is back on the level (what it does at the rim afterwards
        // is not this test's — card JARC; the walk along the rim after the
        // return steps off it again under the old rules too, and where the
        // 300th tick fell in that cycle was luck).
        let mut lost_at = None;
        for t in 0..300 {
            sim.step(&[PadState::default()]);
            let u = sim.unit(h).expect("alive");
            if lost_at.is_none() && u.ai.lost.is_some_and(|l| l.confirmed) {
                lost_at = Some(t);
            }
            if lost_at.is_some() && u.ai.lost.is_none() {
                break;
            }
        }
        assert!(
            lost_at.is_some_and(|t| t <= 2 * THINK_PERIOD),
            "lost within two thinks: {lost_at:?}"
        );
        let u = sim.unit(h).expect("alive");
        assert_eq!(sim.level_at(u.soldier.pos), Level::On, "{}", u.soldier.pos);
        assert!(u.soldier.pos.y > 9.2, "back at the rim: {}", u.soldier.pos);
        assert_eq!(u.ai.lost, None);
    }

    /// Card AIRC: a rail across the hull, its top 0.9 m over the hull under
    /// it, shuts the lost bot's way back with a low wall. It jumps from where
    /// it stands with the stick held at the wall — the air steers it over —
    /// and never backs off for a run-up.
    #[test]
    fn a_lost_bot_jumps_a_low_wall_from_where_it_stands() {
        use crate::ai::tests::rim_sim_with;
        use crate::collision::{Body, Shape};
        use crate::sim::{Level, PadState};
        let rail = Body::new(
            glam::Mat4::from_translation(Vec3::new(-12.4, 9.0, 0.0)),
            Shape::Box {
                half: Vec3::new(0.1, 0.5, 10.0),
            },
        );
        let (mut sim, h) = rim_sim_with(true, vec![rail]);
        // At rest against the rail's outer face (x -12.5, his radius 0.5).
        let y = sim.ground_under(Vec3::new(-13.05, 9.5, 0.0), false);
        assert!(y < 9.0 && y > 8.0, "on the hull: {y}");
        sim.unit_mut(h).expect("alive").soldier.pos = Vec3::new(-13.05, y, 0.0);
        // Until it is back on the level (what it does on the deck afterwards
        // is not this test's: card JARC).
        let (mut reached, mut backed, mut jumped) = (f32::MIN, 0.0f32, false);
        for _ in 0..300 {
            sim.step(&[PadState::default()]);
            let u = sim.unit(h).expect("alive");
            let x = u.soldier.pos.x;
            jumped |= u.soldier.stance == crate::soldier::Stance::Jump;
            if x < -12.5 {
                reached = reached.max(x);
                backed = backed.max(reached - x);
            }
            if jumped && u.ai.lost.is_none() {
                break;
            }
        }
        let u = sim.unit(h).expect("alive");
        assert!(jumped, "it jumped");
        assert!(backed < 0.6, "no run-up: backed off {backed} m");
        assert_eq!(u.ai.lost, None);
        assert_eq!(sim.level_at(u.soldier.pos), Level::On, "{}", u.soldier.pos);
        // (Card RETW: inside the rail it may land on the hull's first
        // metre, under the rim, which is on the level — the return walk
        // aims at its floor walk's point, no longer at the hub's centre.)
        assert!(
            u.soldier.pos.x > -12.3 && u.soldier.pos.y > 9.2,
            "over the rail, back on the level: {}",
            u.soldier.pos
        );
    }

    /// Card RETW: a ground slab under a deck (top y 5, x and z within ±8)
    /// whose hub 0 is pinned by a post on it, and hub 1 on the ground 20 m
    /// east, pinned by a second post; a wall under the deck (x 3, z -20..4)
    /// and a barrier at x 12.5 that keeps hub 1's walk from vouching for
    /// the ground west of x 2.5. A bot under the deck there is OFF the level
    /// (the deck is 5 m over it), and its way to hub 1 is round the wall's
    /// end.
    fn under_deck_sim(wall: bool, pad: bool) -> (Sim, UnitHandle) {
        use crate::collision::{Body, CollisionWorld, Shape};
        use crate::cp::CpDef;
        use crate::nav::{Arc, Hub, NavGraph, RouteTable};
        use crate::region::Region;
        use crate::unit::{Controller, UnitSpec};
        use glam::Mat4;
        let slab = |c: Vec3, half: Vec3| Body::new(Mat4::from_translation(c), Shape::Box { half });
        let mut bodies = vec![
            slab(Vec3::new(0.0, -0.25, 0.0), Vec3::new(40.0, 0.25, 40.0)),
            slab(Vec3::new(0.0, 4.75, 0.0), Vec3::new(8.0, 0.25, 8.0)),
        ];
        if wall {
            bodies.push(slab(Vec3::new(3.0, 1.5, -8.0), Vec3::new(0.25, 1.5, 12.0)));
        }
        if pad {
            bodies.push(slab(Vec3::new(0.0, 0.75, 0.0), Vec3::new(1.5, 0.75, 1.5)));
        }
        let mut sim = Sim::empty(7);
        sim.collision = CollisionWorld::new(bodies, Vec::new());
        let hub = |x: f32, arcs: Vec<u16>| Hub {
            pos: Vec3::new(x, 2.0, 0.0),
            radius: 4.0,
            arcs,
        };
        let mut g = NavGraph::new(
            vec![hub(0.0, vec![0]), hub(20.0, vec![0])],
            vec![Arc {
                a: 0,
                b: 1,
                type_mask: 1,
                flags: 0,
            }],
            RouteTable::empty(2),
            Vec::new(),
            Vec::new(),
        )
        .expect("graph");
        g.barriers.push(nav::Barrier {
            centre: glam::Vec2::new(12.5, 0.0),
            axes: [glam::Vec2::X, glam::Vec2::Y],
            half: glam::Vec2::new(0.5, 30.0),
            mask: 1,
            enabled: true,
        });
        sim.nav = Some(g);
        for (name, at) in [
            ("cp1", Vec3::new(0.0, 5.0, 0.0)),
            ("cp2", Vec3::new(20.0, 0.0, 0.0)),
        ] {
            sim.add_cp(
                CpDef {
                    name: name.into(),
                    pos: at,
                    capture: Region::sphere(at, 3.0),
                    control: None,
                    value_bleed: 100,
                    value_atk: [100, 100],
                    value_def: [100, 100],
                    spawn_nodes: vec![],
                    capture_time: 10.0,
                    neutralize_time: 12.0,
                },
                Some(1),
            );
        }
        let h = sim
            .spawn(&UnitSpec {
                controller: Controller::Ai,
                ..UnitSpec::trooper(0, Vec3::ZERO)
            })
            .expect("spawned");
        sim.unit_mut(h).expect("alive").soldier.pos =
            Vec3::new(0.0, if pad { 1.5 } else { 0.0 }, 0.0);
        (sim, h)
    }

    /// [`under_deck_sim`]'s bot walked until it is back on the level (at
    /// most 300 ticks): the tick and where.
    fn walk_back(sim: &mut Sim, h: UnitHandle) -> Option<(u32, Vec3)> {
        use crate::sim::PadState;
        for t in 0..300u32 {
            sim.step(&[PadState::default()]);
            let u = sim.unit(h).expect("alive");
            if u.ai.lost.is_none() && t > 2 * THINK_PERIOD as u32 {
                return Some((t, u.soldier.pos));
            }
        }
        None
    }

    /// Card RETW: a bot under a deck walks back to the level along the
    /// FLOOR — round the wall's end to the hub on its own floor, inside the
    /// first return window — where the straight line at the nearest hub
    /// stood it under the deck it cannot reach.
    #[test]
    fn a_bot_under_a_deck_walks_round_a_corner_to_a_level_hub() {
        use crate::sim::Level;
        let (mut sim, h) = under_deck_sim(true, false);
        let feet = Vec3::new(0.0, 0.0, 0.0);
        assert_eq!(sim.level_at(feet), Level::Off(5.0), "under the deck");
        assert_eq!(
            sim.return_point(feet, 0).map(|p| p.x),
            Some(0.0),
            "the nearest hub is the deck's, overhead"
        );
        let path = sim.return_path(feet).expect("a floor walk");
        let pts = &path.pts[..usize::from(path.len)];
        assert!(
            pts.iter().any(|p| p.z > 4.0) && pts.iter().all(|p| p.y.abs() < 0.1),
            "round the wall's end, on the ground: {pts:?}"
        );
        let (t, at) = walk_back(&mut sim, h).expect("back on the level within 10 s");
        assert_eq!(sim.level_at(at), Level::On);
        assert!(
            at.z > 4.0 && at.x > 2.5 && at.y.abs() < 0.1,
            "round the wall's end, on the ground: {at} at tick {t}"
        );
        // From a pad 1.5 m up (BF2 Kamino's pod pad): the walk steps down
        // its kerb, which a lost bot's steering refuses anywhere else.
        let (mut sim, h) = under_deck_sim(true, true);
        assert!(
            sim.level_at(Vec3::new(0.0, 1.5, 0.0)).is_off(),
            "on the pad"
        );
        let (t, at) = walk_back(&mut sim, h).expect("down the kerb and back within 10 s");
        assert!(
            at.z > 4.0 && at.x > 2.5 && at.y.abs() < 0.1,
            "down the kerb, round the wall's end: {at} at tick {t}"
        );
        // The control: with no wall the same walk is a straight line.
        let (open, _) = under_deck_sim(false, false);
        let p = open.return_path(feet).expect("a floor walk");
        assert_eq!(
            p.len,
            1,
            "one straight walk: {:?}",
            &p.pts[..usize::from(p.len)]
        );
    }

    /// Card BNDS: a bot off the level for `LOST_RECYCLE` is given up — it
    /// dies as lost, with its ticket, and respawns like any other death.
    #[test]
    fn a_bot_with_no_way_back_is_recycled() {
        use crate::ai::tests::rim_sim;
        use crate::sim::{DeathCause, Event, PadState};
        let (mut sim, h) = rim_sim(true);
        let y = sim.ground_under(Vec3::new(-18.0, 12.0, 0.0), false);
        let u = sim.unit_mut(h).expect("alive");
        u.soldier.pos = Vec3::new(-18.0, y, 0.0);
        // Off the level since long ago, as if every return had failed.
        u.ai.lost = Some(LostState {
            confirmed: true,
            ..LostState::default()
        });
        u.ai.next_think = u64::MAX;
        sim.tick = LOST_RECYCLE + 1;
        sim.step(&[PadState::default()]);
        assert!(sim.unit(h).is_none(), "given up");
        assert!(sim.events.iter().any(|e| matches!(
            e,
            Event::EnvDeath {
                cause: DeathCause::Lost,
                ..
            }
        )));
    }

    /// D11's dodge-on-hit (RE §9.4: a movement re-plan, not a roll). A unit
    /// with no route strafes; the coin is the skill's `lerp(0, 0.2)`; the
    /// cooldown is 2 s.
    #[test]
    fn a_hit_unit_dodges_on_the_skill_coin_and_holds_the_cooldown() {
        use crate::sim::Sim;
        use crate::unit::{Controller, UnitSpec};
        let build = |skill: u8| {
            let mut s = Sim::empty(5);
            let h = s
                .spawn(&UnitSpec {
                    controller: Controller::Ai,
                    ..UnitSpec::trooper(0, Vec3::ZERO)
                })
                .unwrap();
            let other = s
                .spawn(&UnitSpec::trooper(1, Vec3::new(0.0, 0.0, -8.0)))
                .unwrap();
            s.unit_mut(h).unwrap().skill = skill;
            (s, h, other)
        };
        // A tick whose coin passes at skill 20 (chance 0.2) and one whose
        // coin does not — the same unit, so only the draw differs.
        let (s0, h, _) = build(20);
        let hit = |t: u64| unit_unit(h.index, t, h.gen, SALT_DODGE);
        let yes = (1..500).find(|&t| hit(t) < 0.2).expect("a passing tick");
        let no = (1..500).find(|&t| hit(t) >= 0.2).expect("a failing tick");
        drop(s0);

        let fire = |skill: u8, tick: u64, st: &mut AiState| {
            let (mut s, h, other) = build(skill);
            s.tick = tick;
            s.unit_mut(h).unwrap().last_hit = Some((other, tick - 1));
            let (pad, _) = think(&s, h, st);
            (pad, s.tick)
        };

        // Skill 20 on a passing tick: it strafes, and the pad carries it.
        let mut st = AiState::default();
        let (pad, tick) = fire(20, yes, &mut st);
        assert_ne!(st.strafe_side, 0, "no dodge at tick {yes}");
        assert_eq!(st.strafe_until, tick + SIDESTEP_TICKS as u64);
        assert_eq!(st.dodge_until, tick + DODGE_COOLDOWN);
        assert_ne!(pad.lx, 0, "the strafe reaches the pad: {pad:?}");

        // The same unit on a failing tick: nothing.
        let mut st = AiState::default();
        let (pad, _) = fire(20, no, &mut st);
        assert_eq!(st.strafe_side, 0, "dodged on a failing coin at {no}");
        assert_eq!(pad.lx, 0);

        // Skill 1: `dodge_chance` is 0, so no tick can pass.
        let mut st = AiState::default();
        let (_, _) = fire(1, yes, &mut st);
        assert_eq!(st.strafe_side, 0, "skill 1 must never dodge");
        assert_eq!(st.dodge_until, 0);

        // The cooldown: a hit inside the 2 s window is ignored even on a
        // passing coin.
        let mut st = AiState::default();
        let (_, t0) = fire(20, yes, &mut st);
        let armed = st.dodge_until;
        let later = (yes + 1..armed).find(|&t| hit(t) < 0.2);
        if let Some(t) = later {
            let (mut s, h, other) = build(20);
            s.tick = t;
            s.unit_mut(h).unwrap().last_hit = Some((other, t - 1));
            st.strafe_side = 0;
            st.strafe_until = 0;
            think(&s, h, &mut st);
            assert_eq!(st.strafe_side, 0, "dodged at {t}, inside the cooldown");
            assert_eq!(st.dodge_until, armed, "the cooldown was re-armed");
        }
        assert_eq!(armed, t0 + DODGE_COOLDOWN);
    }
}
