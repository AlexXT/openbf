//! BF1 passive aim-assist magnetism, transcribed from the decomp
//! (BF1 FUN_0025fea8, lead-verified 2026-08-30; full spec in
//! analysis/re-aim-assist.md). Operates on stick values BEFORE they become
//! rotation, so assist can never out-turn the soldier.
//!
//! Three effects when a magnet target exists:
//! 1. sticky slowdown ×0.5 on both aim axes — ungated;
//! 2. yaw velocity tracking, rate-matched (dt-free) — ungated;
//! 3. fire magnetism toward the ballistic lead point, on a 1 s decay timer —
//!    the only gated term (`armed`: SP Auto Aim option / MP host setting).
//!
//! Pitch is never pulled, only slowed. The caller supplies the lead point
//! (BF1 solves ordnance intercept in FUN_0026d3f0 — ported when weapons land
//! at P4; dummy targets at P3 pass their own position).
//!
//! The intensity slider (PLAN §Movement & feel, an openbf addition) scales
//! the two PULL terms only; the 0.5 slowdown stays authentic at slider 1.0
//! and blends toward 1.0 (no slowdown) at slider 0.

use glam::Vec3;

/// Constants from FUN_0025fea8 (EE float literals, exact).
pub const SLOWDOWN: f32 = 0.5;
pub const GAIN: f32 = 0.75; // shared by tracking + magnetism
pub const MAGNET_TIMER_SECS: f32 = 1.0;

/// Aim sticks in [-1, 1], post-deadzone. `yaw`/`pitch` are modified in place
/// by [`apply`]; `move_mag` is |move stick| (BF1 reads the axis written to
/// entity+0x64 — axis identity pending PCSX2 confirmation, report §8.3).
///
/// SIGN CONVENTION: `yaw` is POSITIVE-LEFT (turning toward -lat). This is
/// forced by the physics of term 1 — strafing along +lat makes the target
/// drift toward -lat in view, and the decomp ADDS the positive tracking
/// term — and makes term 2's `-=` pull toward the lead point. The absolute
/// pad-hardware sign is still a PCSX2 trace item (report §7); callers with
/// positive-right sticks negate on the way in and out.
#[derive(Debug, Clone, Copy)]
pub struct Sticks {
    pub move_mag: f32,
    pub yaw: f32,
    pub pitch: f32,
}

/// Everything FUN_0025fea8 reads about the world this tick.
#[derive(Debug, Clone, Copy)]
pub struct AssistEnv {
    /// Player world position (aim frame origin).
    pub self_pos: Vec3,
    pub self_vel: Vec3,
    /// Collision-prim radius of the player.
    pub self_radius: f32,
    /// Normalized aim forward.
    pub aim_fwd: Vec3,
    /// Normalized world lateral axis of the aim frame: normalize(up × fwd).
    pub lat: Vec3,
    /// Magnet target collision-prim center + velocity + radius.
    pub target_center: Vec3,
    pub target_vel: Vec3,
    pub target_radius: f32,
    /// Ballistically-led aim point (FUN_0026d3f0 output); the pull steers
    /// toward this, not the target center.
    pub lead_point: Option<Vec3>,
    /// Soldier max turn rate R (vcall vtbl294[0x0c]; MaxTurnSpeed per report).
    pub turn_rate: f32,
    /// Zoomed: R is divided by the weapon zoom factor (weapon+0xdc).
    pub zoom_factor: Option<f32>,
    /// Weapon LockOnRange (weaponClass+0xd4); <= 0 disables the weapon's
    /// assist upstream (no magnet target is ever acquired).
    pub lock_on_range: f32,
    pub fire_held: bool,
    /// Term-3 gate: SP Auto Aim option / MP host "aimassist" setting.
    pub armed: bool,
}

/// Persistent per-controller state (ctrl+0x30 in BF1).
#[derive(Debug, Clone, Copy, Default)]
pub struct AssistState {
    pub magnet_timer: f32,
}

/// One tick of FUN_0025fea8, given a valid magnet target. `intensity` = 1.0
/// is PS2-authentic; the caller skips this entirely when no target exists.
pub fn apply(dt: f32, sticks: &mut Sticks, env: &AssistEnv, st: &mut AssistState, intensity: f32) {
    // Weight from how hard the player is already steering (1 - |yaw stick|).
    let w = 1.0 - sticks.yaw.abs();

    // Sticky slowdown (blend toward no-op as intensity drops below 1).
    let slow = 1.0 + (SLOWDOWN - 1.0) * intensity;
    sticks.yaw *= slow;
    sticks.pitch *= slow;
    if w <= 0.0 {
        return;
    }

    let r = match env.zoom_factor {
        Some(z) => env.turn_rate / z,
        None => env.turn_rate,
    };

    // Term 1: velocity tracking (yaw only, rate-matched, no dt).
    let rel_vel = env.self_vel - env.target_vel;
    let lat_vel = rel_vel.dot(env.lat);
    let fwd_dist = (env.target_center - env.self_pos).dot(env.aim_fwd);
    let track = (sticks.move_mag.abs() * w * GAIN * lat_vel) / (fwd_dist * r);
    sticks.yaw += track.clamp(-1.0, 1.0) * intensity;

    // Term 2 timer: refilled while fire is held, if armed.
    if env.armed && env.fire_held {
        st.magnet_timer = MAGNET_TIMER_SECS;
    }
    if st.magnet_timer > 0.0 {
        let mut pull = w * st.magnet_timer * GAIN;
        let radii = env.target_radius + env.self_radius;
        let range = env.lock_on_range;
        if radii < fwd_dist && radii < range {
            pull *= (1.0 - (fwd_dist - radii) / (range - radii)).clamp(0.0, 1.0);
        }
        if pull > 0.0 {
            if let Some(lead) = env.lead_point {
                let dir = (lead - env.self_pos).normalize_or_zero();
                let step = (dir.dot(env.lat) / (r * dt)).clamp(-1.0, 1.0);
                sticks.yaw -= pull * step * intensity;
            }
        }
        // Decay runs whenever the timer is live, even with zero pull
        // (falloff edge) — matches the decomp's block structure.
        st.magnet_timer -= dt;
    }
}

#[cfg(test)]
mod tests {
    //! Reference vectors T1-T6 from analysis/re-aim-assist.md §7, hand-computed
    //! from the FUN_0025fea8 algorithm.
    use super::*;

    const DT: f32 = 1.0 / 30.0;
    const EPS: f32 = 1e-5;

    fn env() -> AssistEnv {
        AssistEnv {
            self_pos: Vec3::ZERO,
            self_vel: Vec3::ZERO,
            self_radius: 0.5,
            aim_fwd: Vec3::NEG_Z,
            lat: Vec3::X,
            target_center: Vec3::new(0.0, 0.0, -20.0),
            target_vel: Vec3::ZERO,
            target_radius: 0.5,
            lead_point: None,
            turn_rate: 5.0,
            zoom_factor: None,
            lock_on_range: 50.0,
            fire_held: false,
            armed: false,
        }
    }

    #[test]
    fn t1_slowdown_only() {
        let mut s = Sticks {
            move_mag: 0.0,
            yaw: 0.4,
            pitch: 0.6,
        };
        let mut st = AssistState::default();
        apply(DT, &mut s, &env(), &mut st, 1.0);
        assert!((s.yaw - 0.2).abs() < EPS, "yaw {}", s.yaw);
        assert!((s.pitch - 0.3).abs() < EPS, "pitch {}", s.pitch);
        assert_eq!(st.magnet_timer, 0.0);
    }

    #[test]
    fn t2_full_stick_bypasses_everything_but_slowdown() {
        let mut e = env();
        e.self_vel = Vec3::X * 5.0; // would produce tracking if not bypassed
        let mut s = Sticks {
            move_mag: 1.0,
            yaw: 1.0,
            pitch: 0.6,
        };
        let mut st = AssistState::default();
        apply(DT, &mut s, &e, &mut st, 1.0);
        assert!((s.yaw - 0.5).abs() < EPS);
        assert!((s.pitch - 0.3).abs() < EPS);
    }

    #[test]
    fn t3_strafe_tracking() {
        let mut e = env();
        e.self_vel = Vec3::X * 5.0;
        let mut s = Sticks {
            move_mag: 1.0,
            yaw: 0.0,
            pitch: 0.0,
        };
        let mut st = AssistState::default();
        apply(DT, &mut s, &e, &mut st, 1.0);
        // (1 * 1 * 0.75 * 5) / (20 * 5) = 0.0375
        assert!((s.yaw - 0.0375).abs() < EPS, "yaw {}", s.yaw);

        // Doubling distance halves it.
        e.target_center.z = -40.0;
        let mut s2 = Sticks {
            move_mag: 1.0,
            yaw: 0.0,
            pitch: 0.0,
        };
        apply(DT, &mut s2, &e, &mut AssistState::default(), 1.0);
        assert!((s2.yaw - 0.01875).abs() < EPS);
    }

    #[test]
    fn t4_fire_magnetism_pulls_toward_lead() {
        let mut e = env();
        e.armed = true;
        e.fire_held = true;
        // Lead point with lateral direction-sine 0.1: unit dir has dot(lat)=0.1.
        let d = 30.0_f32;
        e.lead_point = Some(Vec3::new(0.1 * d, 0.0, -(1.0 - 0.01_f32).sqrt() * d));
        let mut s = Sticks {
            move_mag: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        };
        let mut st = AssistState::default();
        apply(DT, &mut s, &e, &mut st, 1.0);
        // pull = 1 * 1 * 0.75 * (1 - 19/49); step = clamp(0.1/(5/30)) = 0.6
        let pull = 0.75 * (1.0 - 19.0 / 49.0);
        assert!((s.yaw - (-pull * 0.6)).abs() < 1e-4, "yaw {}", s.yaw);
        assert!((st.magnet_timer - (1.0 - DT)).abs() < EPS);
    }

    #[test]
    fn t5_falloff_edge_no_pull_but_timer_decays() {
        let mut e = env();
        e.armed = true;
        e.fire_held = true;
        e.target_center.z = -50.0; // fwdDist == LockOnRange -> fade 0
        e.lead_point = Some(Vec3::new(5.0, 0.0, -49.0));
        let mut s = Sticks {
            move_mag: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        };
        let mut st = AssistState::default();
        apply(DT, &mut s, &e, &mut st, 1.0);
        assert_eq!(s.yaw, 0.0, "no pull at range edge");
        assert!((st.magnet_timer - (1.0 - DT)).abs() < EPS, "timer decays");
    }

    #[test]
    fn t6_unarmed_keeps_slowdown_and_tracking_only() {
        let mut e = env();
        e.armed = false; // Auto Aim OFF
        e.fire_held = true;
        e.self_vel = Vec3::X * 5.0;
        e.lead_point = Some(Vec3::new(3.0, 0.0, -20.0));
        let mut s = Sticks {
            move_mag: 1.0,
            yaw: 0.0,
            pitch: 0.4,
        };
        let mut st = AssistState::default();
        apply(DT, &mut s, &e, &mut st, 1.0);
        assert!((s.yaw - 0.0375).abs() < EPS, "tracking still on: {}", s.yaw);
        assert!((s.pitch - 0.2).abs() < EPS, "slowdown still on");
        assert_eq!(st.magnet_timer, 0.0, "timer never refills unarmed");
    }

    #[test]
    fn intensity_zero_is_a_no_op() {
        let mut e = env();
        e.armed = true;
        e.fire_held = true;
        e.self_vel = Vec3::X * 5.0;
        e.lead_point = Some(Vec3::new(3.0, 0.0, -20.0));
        let mut s = Sticks {
            move_mag: 1.0,
            yaw: 0.4,
            pitch: 0.6,
        };
        apply(DT, &mut s, &e, &mut AssistState::default(), 0.0);
        assert_eq!((s.yaw, s.pitch), (0.4, 0.6));
    }
}
