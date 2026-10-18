//! Enter, drive, exit on flat ground with the TAT1 skiff's hover numbers.

use super::{driver_input, EXIT_RING, MOVE_FLOOR};
use crate::sim::tests::flat_ground;
use crate::sim::{buttons, PadState, Sim};
use crate::unit::{Controller, UnitHandle, UnitSpec};
use crate::vehicle::hover::tests::skiff;
use crate::vehicle::{SeatKind, SeatSetup, VehicleHandle, VehicleKind, VehicleSpec};
use crate::weapon::HealthType;
use glam::Vec3;

const R: f32 = 5.0;

/// A skiff-like hover (pilot + 3 gunners) `at`, facing −Z (yaw π), on flat
/// ground; the player at the origin facing −Z.
fn world(at: Vec3, home_team: Option<u8>, neutral: bool) -> (Sim, UnitHandle, VehicleHandle) {
    let mut s = Sim::empty(1);
    s.heightfield = Some(flat_ground());
    s.hover_params = vec![Some(skiff())];
    s.seat_setup = vec![Some(SeatSetup {
        radius: R,
        boardable: vec![true; 4],
        exit_drop: 10.0,
        ai_size: 1,
        guns: Vec::new(),
    })];
    let p = s.spawn(&UnitSpec::player(0, Vec3::ZERO)).unwrap();
    let v = s.spawn_vehicle(&VehicleSpec {
        kind: VehicleKind::Hover,
        class: 0,
        home_team,
        neutral_when_empty: neutral,
        spawn_pad: None,
        pos: at,
        yaw: std::f32::consts::PI,
        max_health: 1000.0,
        health_type: HealthType::Vehicle,
        seats: vec![
            SeatKind::Pilot,
            SeatKind::Gunner(0),
            SeatKind::Gunner(1),
            SeatKind::Gunner(2),
        ],
    });
    (s, p, v)
}

fn press(s: &mut Sim) {
    s.step(&[PadState {
        buttons: buttons::ACTION,
        ..PadState::default()
    }]);
    s.step(&[PadState::default()]);
}

#[test]
fn action_enters_the_vehicle_ahead_in_its_first_boardable_seat() {
    let (mut s, p, v) = world(Vec3::new(0.0, 2.0, -6.0), None, true);
    assert_eq!(s.enter_target(p), Some((v, 0)));
    press(&mut s);
    assert_eq!(s.seat_of(p), Some((v, SeatKind::Pilot)));
    assert_eq!(s.vehicle(v).unwrap().team, Some(0), "the entrant's team");
    // A held button is one edge: still seated a tick later.
    s.step(&[PadState::default()]);
    assert!(s.seat_of(p).is_some());
}

#[test]
fn nothing_behind_or_out_of_reach_is_entered() {
    let (s, p, _) = world(Vec3::new(0.0, 2.0, 6.0), None, true);
    assert_eq!(s.enter_target(p), None, "behind the eye: t < 0");
    let (s, p, _) = world(Vec3::new(0.0, 2.0, -12.0), None, true);
    assert_eq!(s.enter_target(p), None, "the spheres do not meet");
}

#[test]
fn an_enemy_vehicle_refuses_and_a_neutral_one_takes_anyone() {
    let (mut s, p, v) = world(Vec3::new(0.0, 2.0, -6.0), Some(1), false);
    assert_eq!(s.vehicle(v).unwrap().team, Some(1));
    assert_eq!(s.enter_target(p), None);
    s.vehicle_mut(v).unwrap().team = None;
    assert_eq!(s.enter_target(p), Some((v, 0)));
}

#[test]
fn a_player_takes_the_ai_drivers_seat_and_the_ai_is_placed_outside() {
    let (mut s, p, v) = world(Vec3::new(0.0, 2.0, -6.0), None, true);
    let ai = s
        .spawn(&UnitSpec::trooper(0, Vec3::new(3.0, 0.0, 0.0)))
        .unwrap();
    assert!(s.seat_unit(ai, v, 0));
    assert_eq!(
        s.enter_seat(ai, v),
        Some(1),
        "an AI never takes a seat over"
    );
    press(&mut s);
    assert_eq!(s.seat_of(p), Some((v, SeatKind::Pilot)));
    let a = s.unit(ai).unwrap();
    assert_eq!(a.mount, None);
    assert!(
        a.soldier.pos.y.abs() < 1e-3,
        "on the ground: {}",
        a.soldier.pos
    );
    assert!(a.soldier.pos.distance(Vec3::new(0.0, 0.0, -6.0)) > R);
}

#[test]
fn the_drivers_pad_drives_the_hover_and_the_rider_follows() {
    let (mut s, p, v) = world(Vec3::new(0.0, 2.0, -6.0), None, true);
    for _ in 0..60 {
        s.step(&[PadState::default()]); // settle on the springs
    }
    press(&mut s);
    let start = s.vehicle(v).unwrap().pos;
    let fwd0 = s.vehicle(v).unwrap().rot * Vec3::Z;
    let full = PadState {
        ly: i16::MAX,
        ..PadState::default()
    };
    for _ in 0..90 {
        s.step(&[full]);
    }
    let veh = s.vehicle(v).unwrap();
    let ran = (veh.pos - start).dot(fwd0);
    assert!(ran > 20.0, "3 s of full throttle: {ran} m");
    assert!(veh.input.human && veh.input.throttle == 1.0);
    assert_eq!(s.unit(p).unwrap().soldier.pos, veh.pos, "the rider rides");
    // The right stick turns toward the screen's right: f × up of a camera
    // looking along the heading.
    let right0 = (veh.rot * Vec3::Z).cross(Vec3::Y);
    let turn = PadState {
        ly: i16::MAX,
        rx: i16::MAX,
        ..PadState::default()
    };
    for _ in 0..20 {
        s.step(&[turn]);
    }
    let veh = s.vehicle(v).unwrap();
    assert!((veh.rot * Vec3::Z).dot(right0) > 0.1);
    assert!(
        veh.hover.stick_timer > 0.5,
        "full right stick ramps the timer"
    );
}

#[test]
fn exit_stands_the_soldier_behind_the_vehicle_on_the_ground() {
    let (mut s, p, v) = world(Vec3::new(0.0, 2.0, -6.0), None, true);
    press(&mut s);
    s.step(&[PadState {
        ly: i16::MAX,
        ..PadState::default()
    }]);
    press(&mut s);
    let u = s.unit(p).unwrap();
    assert_eq!(u.mount, None);
    let veh = s.vehicle(v).unwrap();
    let f = veh.rot * Vec3::Z;
    let back = Vec3::new(-f.x, 0.0, -f.z).normalize();
    let off = u.soldier.pos - veh.pos;
    let flat = Vec3::new(off.x, 0.0, off.z);
    // The vehicle drifted a tick after the exit: the candidate was behind.
    assert!(flat.normalize().dot(back) > 0.95, "behind: {flat}");
    assert!(
        (flat.length() - (R + EXIT_RING)).abs() < 0.5,
        "at ρ: {}",
        flat.length()
    );
    assert!(u.soldier.pos.y.abs() < 1e-3, "on the ground");
    assert!(
        s.vehicle(v).unwrap().input == Default::default(),
        "released"
    );
    assert_eq!(s.vehicle(v).unwrap().team, None, "neutral again");
}

#[test]
fn a_pilot_seat_without_a_human_drives_nothing() {
    let (mut s, p, v) = world(Vec3::new(0.0, 2.0, -6.0), None, true);
    press(&mut s);
    s.step(&[PadState {
        ly: i16::MAX,
        ..PadState::default()
    }]);
    assert_eq!(s.vehicle(v).unwrap().input.throttle, 1.0);
    s.units[p.index as usize].controller = Controller::Idle;
    s.step(&[]);
    assert_eq!(s.vehicle(v).unwrap().input, Default::default());
}

#[test]
fn the_driver_sticks_are_shaped_as_the_ps2_shapes_them() {
    let q = PadState::quantize;
    let (i, full) = driver_input(&PadState {
        lx: q(0.1),
        ly: q(-0.05),
        rx: q(1.0),
        ry: q(1.0),
        buttons: 0,
    });
    assert_eq!(
        (i.strafe, i.throttle),
        (MOVE_FLOOR, -MOVE_FLOOR),
        "the 0.3 floor"
    );
    assert!(full);
    assert!(
        (i.turn * i.turn + i.pitch * i.pitch - 1.0).abs() < 1e-5,
        "into the circle"
    );
    let (_, full) = driver_input(&PadState {
        rx: q(0.9),
        ..PadState::default()
    });
    assert!(!full, "0.81 < 0.9");
}

/// Card VTURB: a building turret — one gunner seat, no pilot — standing in
/// its own collision box. ACTION takes the gunner seat, the turret never
/// moves, and the exit stands the soldier on the ring behind it: the path
/// test, which every candidate would fail from inside the box, is skipped.
#[test]
fn a_building_turret_seats_a_gunner_and_lets_it_out_of_its_own_box() {
    use crate::collision::{Body, CollisionWorld, Shape};
    let at = Vec3::new(0.0, 0.0, -4.0);
    let mut s = Sim::empty(1);
    s.heightfield = Some(flat_ground());
    s.collision = CollisionWorld::new(
        vec![Body::new(
            glam::Mat4::from_translation(at + Vec3::Y),
            Shape::Box {
                half: Vec3::splat(1.5),
            },
        )],
        Vec::new(),
    );
    s.hover_params = vec![None];
    s.seat_setup = vec![Some(SeatSetup {
        radius: 3.0,
        boardable: vec![true],
        exit_drop: 10.0,
        ai_size: 1,
        guns: Vec::new(),
    })];
    let p = s.spawn(&UnitSpec::player(0, Vec3::ZERO)).unwrap();
    let v = s.spawn_vehicle(&VehicleSpec {
        kind: VehicleKind::Turret,
        class: 0,
        home_team: None,
        neutral_when_empty: true,
        spawn_pad: None,
        pos: at,
        yaw: std::f32::consts::PI,
        max_health: 300.0,
        health_type: HealthType::Building,
        seats: vec![SeatKind::Gunner(0)],
    });
    assert!(s.vehicle(v).unwrap().pilot_seat().is_none());
    assert_eq!(s.enter_target(p), Some((v, 0)));
    press(&mut s);
    assert_eq!(s.seat_of(p), Some((v, SeatKind::Gunner(0))));
    for _ in 0..30 {
        s.step(&[PadState {
            ly: i16::MAX,
            rx: i16::MAX,
            ..PadState::default()
        }]);
    }
    assert_eq!(s.vehicle(v).unwrap().pos, at, "a turret never moves");
    press(&mut s);
    let u = s.unit(p).unwrap();
    assert_eq!(u.mount, None);
    // Facing −Z (yaw π): behind is +Z, ρ = 3 + EXIT_RING from the origin.
    let want = at + Vec3::Z * (3.0 + EXIT_RING);
    assert!(
        u.soldier.pos.distance(want) < 1e-3,
        "placed at {} (want {want})",
        u.soldier.pos
    );
}

/// Card VMESH: a rider whose vehicle's centre is inside a closed prop (a
/// bike through BF1 Endor's cp2 boulder) gets out OUTSIDE it - every one of
/// §5's points is inside and passes the path test, the ring past the
/// body's box is taken - and stands on the ground there.
#[test]
fn a_rider_whose_vehicle_is_inside_a_closed_body_gets_out_outside_it() {
    use crate::collision::{box_mesh, CollisionWorld};
    let at = Vec3::new(0.0, 2.0, -6.0);
    let (mut s, p, v) = world(at, None, true);
    press(&mut s);
    assert_eq!(s.seat_of(p), Some((v, SeatKind::Pilot)));
    // The prop closes round the seated rider's vehicle: 2 m past §5's ring.
    let half = Vec3::new(R + EXIT_RING + 2.0, 4.0, R + EXIT_RING + 2.0);
    s.collision = CollisionWorld::new(
        vec![box_mesh(Vec3::new(0.0, 3.0, -6.0), half, false)],
        Vec::new(),
    );
    s.exit_vehicle(p);
    let u = s.unit(p).unwrap();
    let feet = u.soldier.pos;
    assert_eq!(u.mount, None);
    assert_eq!(s.shut_in(feet + Vec3::Y), None, "outside: {feet}");
    let off = feet - Vec3::new(0.0, 0.0, -6.0);
    assert!(
        off.x.abs() > half.x || off.z.abs() > half.z,
        "past the box: {feet}"
    );
    assert!(feet.y.abs() < 1e-3, "on the ground: {feet}");
}

/// Card VMESH: a hover driven at a closed prop at full throttle stays
/// outside it. The discrete contact pass let a bike's centre through BF1
/// Endor's boulder at 23 m/s; here the hover has no hull bodies, so nothing
/// but the keep-out rule stands between its centre and the shell.
#[test]
fn a_hover_driven_at_a_closed_body_stays_outside() {
    use crate::collision::{box_mesh, CollisionWorld};
    let (mut s, p, v) = world(Vec3::new(0.0, 2.0, -6.0), None, true);
    {
        // A bike's pace (2 m a tick, past its spring spheres' size).
        let hp = s.hover_params[0].as_mut().unwrap();
        hp.hull.clear();
        hp.forward_speed = 60.0;
        hp.acceleration = 60.0;
    }
    // (No impact destroys it: the test is where it goes.)
    s.vehicles[v.index as usize].health = crate::weapon::Health::new(1.0e9, HealthType::Vehicle);
    press(&mut s);
    assert_eq!(s.seat_of(p), Some((v, SeatKind::Pilot)));
    let rock = Vec3::new(0.0, 3.0, -40.0);
    s.collision = CollisionWorld::new(
        vec![box_mesh(rock, Vec3::new(5.0, 3.0, 5.0), false)],
        Vec::new(),
    );
    let (mut nearest, mut fastest) = (f32::MAX, 0.0f32);
    for _ in 0..(6 * crate::TICK_HZ) {
        s.step(&[PadState {
            ly: i16::MAX,
            ..PadState::default()
        }]);
        let veh = s.vehicle(v).unwrap();
        assert_eq!(s.shut_in(veh.pos), None, "inside at {}", veh.pos);
        nearest = nearest.min(veh.pos.z);
        fastest = fastest.max(Vec3::new(veh.vel.x, 0.0, veh.vel.z).length());
    }
    assert!(nearest < -30.0, "it reached the wall: {nearest}");
    assert!(fastest > 23.0, "at speed: {fastest} m/s");
}
