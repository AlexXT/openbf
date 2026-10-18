//! A scripted ride (card VSEAT): the pad a player would give to walk to the
//! nearest vehicle it may board, enter it, drive, and get out — recorded as
//! an [`InputLog`] so `--walk-replay` replays (and captures) it. The script
//! reads the sim only to choose its pad, so the log alone reproduces the
//! run. Headless `--ride-script <out.ron>` (`main/headless.rs`) drives it.

use crate::walk::WalkSim;
use core::sim::{buttons, InputLog, PadState};
use core::vehicle::VehicleHandle;
use glam::Vec3;

/// How far the ride drives, path length over the ground.
pub const DRIVE_M: f32 = 200.0;
const WALK_TICKS: u32 = 60 * 30;
const DRIVE_TICKS: u32 = 90 * 30;
const STAND_TICKS: u32 = 60;

/// What happened, for the caller's report.
#[derive(Debug, Default)]
pub struct Ride {
    pub log: InputLog,
    /// The vehicle, and the tick ACTION seated the player.
    pub entered: Option<(VehicleHandle, usize)>,
    pub driven_m: f32,
    /// The tick ACTION put the player back on foot, and where.
    pub exited: Option<(usize, Vec3)>,
}

fn wrap(a: f32) -> f32 {
    let t = std::f32::consts::TAU;
    (a + std::f32::consts::PI).rem_euclid(t) - std::f32::consts::PI
}

/// Run the ride on `walk` (slot 0's player), stepping it; the log starts at
/// `walk`'s current tick with `seed`.
pub fn script(walk: &mut WalkSim, seed: u32) -> Ride {
    let mut ride = Ride {
        log: InputLog {
            seed,
            ..InputLog::default()
        },
        ..Ride::default()
    };
    let q = PadState::quantize;
    let press = PadState {
        buttons: buttons::ACTION,
        ..PadState::default()
    };
    let step = |walk: &mut WalkSim, ride: &mut Ride, pad: PadState| {
        ride.log.ticks.push(pad);
        walk.step_tick(&[pad]);
    };
    let p = walk.player;

    // 1. Walk to the nearest vehicle the player may board.
    for _ in 0..WALK_TICKS {
        if walk.sim.enter_target(p).is_some() {
            break;
        }
        let Some(me) = walk.sim.unit(p).map(|u| u.soldier.clone()) else {
            step(walk, &mut ride, PadState::default());
            continue;
        };
        let goal = walk
            .sim
            .alive_vehicles()
            .filter(|(h, _)| walk.sim.enter_seat(p, *h).is_some())
            .map(|(_, v)| v.pos)
            .min_by(|a, b| {
                a.distance_squared(me.pos)
                    .total_cmp(&b.distance_squared(me.pos))
            });
        let pad = match goal {
            Some(g) => {
                let d = g - me.pos;
                let err = wrap(core::fmath::atan2(-d.x, -d.z) - me.yaw);
                PadState {
                    rx: q(-3.0 * err),
                    ly: if err.abs() < 0.5 { i16::MAX } else { 0 },
                    ..PadState::default()
                }
            }
            None => PadState::default(),
        };
        step(walk, &mut ride, pad);
    }
    if walk.sim.enter_target(p).is_none() {
        return ride;
    }
    step(walk, &mut ride, press);
    let Some((vh, _)) = walk.sim.seat_of(p) else {
        return ride;
    };
    ride.entered = Some((vh, ride.log.ticks.len() - 1));

    // 2. Drive: full throttle, full right stick while stuck.
    let Some(mut last) = walk.sim.vehicle(vh).map(|v| v.pos) else {
        return ride;
    };
    let mut stuck = 0u32;
    for t in 0..DRIVE_TICKS {
        if ride.driven_m >= DRIVE_M {
            break;
        }
        let Some(v) = walk.sim.vehicle(vh) else { break };
        stuck = if t > 90 && v.vel.length() < 3.0 {
            45
        } else {
            stuck.saturating_sub(1)
        };
        let pad = PadState {
            ly: i16::MAX,
            rx: if stuck > 0 { i16::MAX } else { 0 },
            ..PadState::default()
        };
        step(walk, &mut ride, pad);
        let Some(now) = walk.sim.vehicle(vh).map(|v| v.pos) else {
            break;
        };
        ride.driven_m += Vec3::new(now.x - last.x, 0.0, now.z - last.z).length();
        last = now;
    }

    // 3. Out, and stand.
    step(walk, &mut ride, PadState::default());
    step(walk, &mut ride, press);
    if let Some(u) = walk.sim.unit(p).filter(|u| u.mount.is_none()) {
        ride.exited = Some((ride.log.ticks.len() - 1, u.soldier.pos));
    }
    for _ in 0..STAND_TICKS {
        step(walk, &mut ride, PadState::default());
    }
    ride
}
