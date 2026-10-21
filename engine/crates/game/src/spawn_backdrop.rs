//! The spawn screens' backdrop at the level start (card SSELB,
//! `analysis/re-spawn-flow.md` §1.1 [O]): the view is one of the mission's
//! `AddCameraShot` views, and each screen change (side -> map -> class, and
//! back) cuts to another, drawn by the PS2's `FUN_001b9d00` ([`ShotPicker`]).
//! The backdrop ends with the player's first spawn; at a respawn the view is
//! the place he died (§6 item 3), dimmed like every spawn screen
//! (`ui::spawn::BACKDROP_DIM`).
//!
//! The shots come from the mission RON (`MissionDef::camera_shots`, baked by
//! `scripts/gen_missions.py`). A shot's rotation turns the camera's own -Z
//! into its view (the PS2's camera matrix rows are right, up, back, eye;
//! `engine/artifacts/pcsx2/pose.py`), and the world axes are the port's: no
//! flip. Framed at the PS2's default 48 deg horizontal (tan_v 0.34131, §1.1
//! [L]) with the port's Hor+ rule: that vertical, wider at a wide window.

use core::sim::SpawnStage;
use glam::{Quat, Vec3};
use render::Camera;

/// The PS2's spawn-screen camera: tan(fov_v / 2) at the default 48 deg
/// horizontal, read on Hoth at all three screens (re-spawn-flow §1.1 [L]).
pub const SHOT_TAN_HALF_FOV_Y: f32 = 0.34131;

/// The word the PS2 writes into its game LCG at a reseed (B2 `FUN_0053a930`,
/// `B2:669277`): the port's presentation LCG starts there.
pub const LCG_RESEED: u32 = 0x94153a94;

/// One `AddCameraShot(qw, qx, qy, qz, x, y, z)` (B2 `FUN_001b9d00`'s table
/// at camera manager +0xa0 / +0x1a0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraShot {
    pub rot: Quat,
    pub pos: Vec3,
}

impl CameraShot {
    /// From the RON tuple, in the script's argument order.
    pub fn from_call(a: [f32; 7]) -> Self {
        Self {
            rot: Quat::from_xyzw(a[1], a[2], a[3], a[0]),
            pos: Vec3::new(a[4], a[5], a[6]),
        }
    }

    /// The view from the shot: its eye, looking down the rotated -Z.
    pub fn camera(&self) -> Camera {
        let f = -(self.rot * Vec3::Z);
        Camera {
            position: self.pos,
            yaw: core::fmath::atan2(-f.x, -f.z),
            pitch: core::fmath::atan2(f.y, f.with_y(0.0).length()),
            fov_y: 2.0 * core::fmath::atan2(SHOT_TAN_HALF_FOV_Y, 1.0),
            ..Camera::default()
        }
    }
}

/// The PS2's pick of the next camera shot (B2 `FUN_001b9d00`, `B2:63214`,
/// re-spawn-flow §1.1): with n shots, n < 3 steps by one; n >= 3 draws
/// `((u >> 16) | (s & 0xffff0000)) % n` (unsigned, `divu` at 0x1b9dbc) over two
/// LCG steps, draws again while the draw equals the first one (at most 16
/// times) and steps one past the current shot if it still lands on it - never
/// the current shot.
///
/// The PS2 draws from its shared game LCG (`DAT_0073d868`); the port keeps a
/// presentation LCG of its own with the same steps, so a screen change never
/// moves the sim (a named deviation: the draws match the PS2's rule, not its
/// sequence). PROVISIONAL until a test pins a draw against a PS2 row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShotPicker {
    pub seed: u32,
    /// `DAT_005fc4f0`: the shot on screen.
    pub current: usize,
}

impl ShotPicker {
    pub fn new(seed: u32) -> Self {
        Self { seed, current: 0 }
    }

    fn draw(&mut self, n: u32) -> u32 {
        let u = self.seed.wrapping_mul(0x19660d).wrapping_add(0x3c6ef35f);
        self.seed = u.wrapping_mul(0x19660d).wrapping_add(0x3c6ef35f);
        ((u >> 16) | (self.seed & 0xffff_0000)) % n
    }

    /// The next shot of `n`; `n == 0` keeps the current one (the PS2 does
    /// nothing).
    pub fn next(&mut self, n: usize) -> usize {
        if n == 0 {
            return self.current;
        }
        let cur = self.current as u32;
        let n32 = n as u32;
        let pick = if n32 < 3 {
            (cur + 1) % n32
        } else {
            let first = self.draw(n32);
            let mut pick = first;
            if first == cur {
                for _ in 0..16 {
                    pick = self.draw(n32);
                    if pick != first {
                        break;
                    }
                }
                if pick == cur {
                    pick = (pick + 1) % n32;
                }
            }
            pick
        };
        self.current = pick as usize;
        self.current
    }
}

/// The level start's backdrop for the player's slot: the mission's shots,
/// the pick, and the screen it was last on.
#[derive(Debug, Clone)]
pub struct Backdrop {
    pub shots: Vec<CameraShot>,
    pub picker: ShotPicker,
    stage: Option<SpawnStage>,
}

impl Backdrop {
    /// `None` when the mission has no shots (the view stays the battle's).
    /// The first screen shows shot 0 (Hoth's side select, re-spawn-flow §1.1
    /// [L]; PROVISIONAL that the open draws none).
    pub fn new(shots: Vec<CameraShot>, seed: u32) -> Option<Self> {
        (!shots.is_empty()).then(|| Self {
            shots,
            picker: ShotPicker::new(seed),
            stage: None,
        })
    }

    /// Follow the slot's screen: a change of screen draws the next shot.
    pub fn follow(&mut self, stage: SpawnStage) {
        if self.stage.is_some_and(|s| s != stage) {
            self.picker.next(self.shots.len());
        }
        self.stage = Some(stage);
    }

    pub fn camera(&self) -> Camera {
        self.shots[self.picker.current.min(self.shots.len() - 1)].camera()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Acceptance 3: n < 3 steps by one; n >= 3 never returns the current
    /// shot, and reaches every other one.
    #[test]
    fn the_shot_draw_steps_below_three_and_never_repeats() {
        let mut p = ShotPicker::new(7);
        assert_eq!(p.next(1), 0);
        assert_eq!(p.next(2), 1);
        assert_eq!(p.next(2), 0);
        assert_eq!(p.next(2), 1);
        let seed_before = p.seed;
        assert_eq!(p.next(0), 1, "no shots: unchanged");
        assert_eq!(p.seed, seed_before, "n < 3 draws nothing");
        for n in 3..=16usize {
            for seed in [0u32, 1, 0x94153a94, 0xdead_beef, 12345] {
                let mut p = ShotPicker::new(seed);
                let mut seen = vec![false; n];
                for _ in 0..400 {
                    let cur = p.current;
                    let next = p.next(n);
                    assert_ne!(next, cur, "n {n} seed {seed:#x}");
                    assert!(next < n);
                    seen[next] = true;
                }
                assert!(seen.iter().all(|&s| s), "n {n}: every shot is drawn");
            }
        }
    }

    /// The draw is the PS2's arithmetic: two LCG steps, the high halves
    /// joined, unsigned modulo (a joined word with its top bit set stays
    /// positive), and a first draw equal to the current shot drawn again.
    #[test]
    fn the_draw_is_the_ps2_lcg_word() {
        let mut p = ShotPicker::new(0x94153a94);
        let u = 0x94153a94u32
            .wrapping_mul(0x19660d)
            .wrapping_add(0x3c6ef35f);
        let s = u.wrapping_mul(0x19660d).wrapping_add(0x3c6ef35f);
        let word = (u >> 16) | (s & 0xffff_0000);
        let want = (word % 5) as usize;
        p.current = (want + 1) % 5;
        assert_eq!(p.next(5), want);
        assert_eq!(p.seed, s);
        // The same first draw on the current shot: drawn again.
        let mut q = ShotPicker::new(0x94153a94);
        q.current = want;
        assert_ne!(q.next(5), want);
        assert_ne!(q.seed, s, "a second draw stepped the LCG on");
    }

    /// A screen change draws; staying on a screen does not.
    #[test]
    fn the_backdrop_cuts_on_a_screen_change_only() {
        let shots: Vec<CameraShot> = (0..4)
            .map(|i| CameraShot::from_call([1.0, 0.0, 0.0, 0.0, i as f32, 0.0, 0.0]))
            .collect();
        let mut b = Backdrop::new(shots, 99).expect("shots");
        b.follow(SpawnStage::Side);
        b.follow(SpawnStage::Side);
        assert_eq!(b.picker.current, 0, "the first screen shows shot 0");
        assert_eq!(b.camera().position.x, 0.0);
        b.follow(SpawnStage::Map);
        let map = b.picker.current;
        assert_ne!(map, 0);
        b.follow(SpawnStage::Map);
        assert_eq!(b.picker.current, map);
        b.follow(SpawnStage::Class);
        assert_ne!(b.picker.current, map);
        assert_eq!(b.camera().position.x, b.picker.current as f32);
        assert!(Backdrop::new(Vec::new(), 1).is_none());
    }

    /// The identity rotation looks down -Z, level.
    #[test]
    fn a_shot_looks_down_its_rotated_minus_z() {
        let c = CameraShot::from_call([1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0]).camera();
        assert!(c.forward().abs_diff_eq(Vec3::NEG_Z, 1e-6));
        assert_eq!(c.position, Vec3::new(1.0, 2.0, 3.0));
        assert!(((c.fov_y * 0.5).tan() - SHOT_TAN_HALF_FOV_Y).abs() < 1e-6);
        // A quarter turn about +Y: -Z turns to -X.
        let q = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let c = CameraShot::from_call([q.w, q.x, q.y, q.z, 0.0, 0.0, 0.0]).camera();
        assert!(
            c.forward().abs_diff_eq(Vec3::NEG_X, 1e-5),
            "{:?}",
            c.forward()
        );
    }
}
