//! Card SKIRT on the real bake (data-gated: skipped when the bake is absent):
//! BF2 Kamino's pod skirt, the band of 48 degrees (n.y 0.6665) under the rail
//! of the pod's dais, walked by the player's rifleman as the PS2's player
//! walked it (`analysis/re-soldier-ground.md` §7.1; the 24 PS2 walks are
//! `lead_tmp/skirt/ps2/out`). Each walk: planted on the floor under the start,
//! facing -Z, 1.5 s standing, then the stick held full forward.
//!
//! - `n150long`: from a run-up of 2 m to a band 1.48 m high, the PS2's feet
//!   pass y 62.0 0.97 s after the stick goes down; the port's must within 1.2
//!   s (before card SKIRT they never did: a band about 1.2 m high was the
//!   port's limit).
//! - `footN`: from a stand 0.1 m off the same band's foot the PS2's soldier
//!   does not get up it (highest feet 60.59 in 8 s); nor may the port's in 2 s.
//! - `r179`: at the riser beside the band's west end the PS2's feet reach
//!   61.84; the port's stopped at 60.65 while the wall pass asked its ledge
//!   rule at the feet's height before the tick's floor was found.

use core::sim::{PadState, Sim};
use core::unit::UnitSpec;
use glam::Vec3;

/// The highest feet, and the seconds of stick when they first pass y 62.0,
/// over `walk_s` of stick.
fn walk(x: f32, z: f32, walk_s: f32) -> Option<(f32, Option<f32>)> {
    let root = game::assets::baked_root();
    let dir = root.join("bf2_kam1");
    if !dir.join("manifest.ron").exists() || !root.join("classes").exists() {
        eprintln!("skipped: no bf2_kam1 bake");
        return None;
    }
    let mut s = game::assets::load_scene(&dir).expect("scene");
    s.apply_mode_layer("con");
    if let Ok(c) = game::assets::load_common(&root.join("common_bf2")) {
        s.merge_common(c);
    }
    let db =
        game::roster::load_classdb(&root.join("classes"), "bf2", &["all", "imp"]).expect("classes");
    let consts = game::roster::ClassRoster::build(&db, "imp_inf_rifleman", 0, None)
        .expect("the rifleman")
        .consts;
    let mut sim = Sim::empty(42);
    sim.heightfield = s.heightfield();
    s.install_collision(&mut sim);
    sim.ground_rules = core::soldier::GroundRules::for_game("bf2");
    let p0 = Vec3::new(x, 60.6, z);
    let h = sim
        .spawn(&UnitSpec {
            consts,
            ..UnitSpec::player_in_slot(0, 0, p0)
        })
        .expect("spawn");
    let i = h.index as usize;
    sim.units[i].soldier.pos = p0;
    let (rest, walking) = (45, (walk_s * 30.0).round() as usize);
    let (mut high, mut over) = (f32::MIN, None);
    for k in 0..rest + walking {
        sim.units[i].soldier.yaw = 0.0;
        let ly = if k >= rest { 32767 } else { 0 };
        sim.step(&[PadState {
            ly,
            ..PadState::default()
        }]);
        if k >= rest {
            let y = sim.units[i].soldier.pos.y;
            high = high.max(y);
            if over.is_none() && y >= 62.0 {
                over = Some((k - rest + 1) as f32 / 30.0);
            }
        }
    }
    Some((high, over))
}

#[test]
fn a_run_up_carries_the_soldier_over_the_pods_skirt() {
    let Some((high, over)) = walk(-15.0, 112.6, 2.0) else {
        return;
    };
    let t = over.unwrap_or_else(|| panic!("never over y 62.0: highest feet {high}"));
    assert!(
        t <= 1.2,
        "over the band after {t} s of stick (the PS2 0.97)"
    );
}

#[test]
fn from_a_stand_at_its_foot_the_soldier_does_not_get_up_the_skirt() {
    let Some((high, over)) = walk(-15.0, 110.5, 2.0) else {
        return;
    };
    assert_eq!(over, None, "over y 62.0 from a stand (highest feet {high})");
}

#[test]
fn the_riser_beside_the_skirt_does_not_stop_the_climb() {
    let Some((high, _)) = walk(-17.9, 112.6, 2.0) else {
        return;
    };
    assert!(
        (61.7..=61.9).contains(&high),
        "highest feet {high} (the PS2 61.84)"
    );
}
