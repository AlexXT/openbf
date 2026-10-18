//! Data-gated: the port's FLOOR agrees with what the disc authors at the two
//! places the water defect table points at (queue item WL; the measurement tool
//! is `game/examples/water_floor.rs`, the whole table is in that item's report).
//!
//! `analysis/re-water.md` §2 [V] settles the water PLANE — the height is the
//! first `LAYR`'s f32 and `mungefs::water` reads that field — so the
//! open half was the other side of the join: is the port's GROUND under the
//! props the disc authors in the water where the disc puts it? These two pin
//! the answer at YAV1's bazaar and KAS2's beach post, so a later change to the
//! terrain sheet, the cutters or the walkable-model rule cannot move the floor
//! at a water line without saying so.
//!
//! Skips cleanly when the maps are not baked (`scripts/rebake.sh`).

use core::sim::Sim;
use glam::Vec3;
use std::path::PathBuf;

/// The scene the sweep builds: the `con` mode layer applied so the BF2 posts
/// exist, `common_<game>` merged so the post models' own collision is in the
/// world, then the collision world and the heightfield.
fn world(map: &str) -> Option<(Sim, game::layout::MapLayout, f32)> {
    let dir: PathBuf = game::assets::baked_root().join(map);
    if !dir.join("manifest.ron").exists() {
        return None;
    }
    let mut scene = game::assets::load_scene(&dir).expect("scene");
    scene.apply_mode_layer("con");
    // Beside the map directories, not inside one — as the game and the sweep
    // join it (`dir.parent()`).
    let common = game::assets::baked_root().join(format!("common_{}", scene.manifest.source_game));
    if common.join("common.ron").exists() {
        scene.merge_common(game::assets::load_common(&common).expect("common"));
    }
    let height = scene.manifest.water.as_ref()?.height;
    let layout = game::layout::MapLayout::from_manifest(&scene.manifest);
    let mut sim = Sim::empty(0);
    sim.heightfield = scene.heightfield();
    scene.install_collision(&mut sim);
    Some((sim, layout, height))
}

fn post(layout: &game::layout::MapLayout, name: &str) -> Vec3 {
    layout
        .cps
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("no post {name}"))
        .pos
}

/// BF1 YAV1's bazaar is an authored FLOODED plaza, and the port reproduces it
/// to the centimetre. The disc's plane is `yav_water` −72.5; the plaza's own
/// stone floor is −72.80, i.e. 30 cm UNDER its pool, with the marker standing
/// on a raised element at −72.21; the columns are authored at −72.51..−72.56
/// and the terrain around the plaza at −74.46, two metres under. That is the
/// walkabout sheet's "water at floor level, bottom step submerged, no bank"
/// (`engine/artifacts/walkabout_defects/README.md` group 5) — authored, not a
/// level the port gets wrong.
#[test]
fn the_yav1_bazaar_is_an_authored_flooded_plaza() {
    let Some((sim, layout, water)) = world("bf1_yav1") else {
        eprintln!("SKIP: bf1_yav1 not baked");
        return;
    };
    assert_eq!(water, -72.5, "the plane is the first LAYR's f32");
    let marker = post(&layout, "bazaar");
    let floor = sim.ground_under(marker, false);
    assert!(
        (floor - marker.y).abs() < 0.1,
        "the port's floor {floor} is not the authored marker {}",
        marker.y
    );
    // The plaza AWAY from the marker's own raised element: probed from the
    // plane, because that is where the question is — a soldier at the water
    // line. It is under the surface on all four sides, which is the flooding.
    for (dx, dz) in [(5.0, 0.0), (-5.0, 0.0), (0.0, 5.0), (0.0, -5.0)] {
        let at = Vec3::new(marker.x + dx, water, marker.z + dz);
        let plaza = sim.ground_under(at, false);
        assert!(
            plaza < water && plaza > water - 0.5,
            "the plaza floor {plaza} at {dx},{dz} should sit just under the pool {water}"
        );
    }
    // ...and the terrain around the plaza is the thing well under the water.
    let terrain = sim
        .heightfield
        .as_ref()
        .expect("heightfield")
        .height_at(marker.x, marker.z);
    assert!(
        terrain < water - 1.0,
        "the bazaar's terrain {terrain} should be well under the plane {water}"
    );
}

/// BF1 KAS2's beach post — the walkabout's `cp6_cp1.png`, i.e. the SEVENTH post
/// of the sheet run and the one NAMED `cp1` — stands knee-deep because the disc
/// authors it 5 cm UNDER its own water plane (`kas2_water` −5.0, the marker
/// −5.049) on terrain at the same height. The port's floor is the terrain, to a
/// centimetre: the post's look is the plane's tint, not the port's ground.
#[test]
fn the_kas2_beach_post_stands_on_the_authored_ground() {
    let Some((sim, layout, water)) = world("bf1_kas2") else {
        eprintln!("SKIP: bf1_kas2 not baked");
        return;
    };
    assert_eq!(water, -5.0);
    let marker = post(&layout, "cp1");
    assert!(
        marker.y < water,
        "the disc authors this post under its own plane: {} vs {water}",
        marker.y
    );
    let floor = sim.ground_under(marker, false);
    assert!(
        (floor - marker.y).abs() < 0.1,
        "the port's floor {floor} is not the authored marker {}",
        marker.y
    );
    // ...on a real shoreline: the port's terrain crosses the plane 15 m inland
    // of the post and keeps dropping seaward, so the water line is where the
    // disc's own heightfield puts it and not a flat sheet over a flat floor.
    let hf = sim.heightfield.as_ref().expect("heightfield");
    let inland = hf.height_at(marker.x, marker.z - 15.0);
    let seaward = hf.height_at(marker.x, marker.z + 15.0);
    assert!(
        inland > water + 1.0,
        "15 m inland should be dry land: {inland} vs the plane {water}"
    );
    assert!(
        seaward < water - 0.5,
        "15 m seaward should be under water: {seaward} vs the plane {water}"
    );
}
