//! Tier-1 (data-gated): BF2 HOT1's `lvl_` mode layers carry their own content
//! — the map's four wrappers hold 39 `entc`, 58 `modl`, 42 `prim` and 26
//! `coll` between them (analysis/a2-bake-formats.md §1 for the container
//! shape) — and the bake must treat it exactly like top-level content.
//!
//! The Hoth bridges are the case the maintainer reported: `hoth_objective` places six
//! instances of `hoth_prop_bridge_long` / `_short`, whose `entc`, `gmod`,
//! `modl`, `coll` and `prim` all live inside the first wrapper. Before the
//! layer walk they had no class row, no GLB and no collision at all.

#[test]
fn bf2_hot1_bakes_its_mode_layers_bridges_with_collision() {
    let lvl = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../extracted/bf2/lvl/DATA/_LVL_PS2/HOT/HOT1.LVL"
    );
    if !std::path::Path::new(lvl).exists() {
        eprintln!("SKIP: extracted BF2 HOT1.LVL not present");
        return;
    }
    let out = std::env::temp_dir().join(format!(
        "openbf-bake-test-bf2-hot1-layers-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&out);
    let m = bake::map::bake_map(lvl, out.to_str().unwrap(), None).expect("bake");
    assert_eq!(m.map, "hoth");
    assert_eq!(m.source_game, "bf2");

    for (class, geometry, ordnance_prim) in [
        (
            "hoth_prop_bridge_long",
            "hoth_prop_bridge_long",
            "p_ordinance1",
        ),
        (
            "hoth_prop_bridge_short",
            "hoth_prop_bridge_short",
            "p_ordinance7",
        ),
    ] {
        let c = m
            .classes
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(class))
            .unwrap_or_else(|| panic!("class row for {class}"));
        assert!(
            c.geometry
                .as_deref()
                .is_some_and(|g| g.eq_ignore_ascii_case(geometry)),
            "{class} geometry: {:?}",
            c.geometry
        );
        // The census in the ordnance brief: these are the ONLY bodies on the
        // discs that stop a bolt without stopping a soldier.
        assert!(
            c.collision
                .iter()
                .any(|(l, b)| l == "soldier" && b == "collisionmesh"),
            "{class} soldier collision: {:?}",
            c.collision
        );
        assert!(
            c.collision
                .iter()
                .any(|(l, b)| l == "ordnance" && b == ordnance_prim),
            "{class} ordnance collision: {:?}",
            c.collision
        );
        // A drawable model, and a collision row with both a mesh and prims.
        assert!(
            m.models.iter().any(|n| n.eq_ignore_ascii_case(geometry)),
            "{geometry} model"
        );
        assert!(out.join(format!("models/{geometry}.glb")).is_file());
        let col = m
            .collision
            .iter()
            .find(|c| c.geometry.eq_ignore_ascii_case(geometry))
            .unwrap_or_else(|| panic!("collision row for {geometry}"));
        assert!(col.glb.is_some(), "{geometry} collision mesh GLB");
        assert!(!col.prims.is_empty(), "{geometry} primitives");
    }

    // Layer content adds NO instance: the worlds are read exactly as before.
    assert_eq!(m.instances.len(), 207, "base+always instances");
    assert_eq!(m.layers.len(), 4);
    let _ = std::fs::remove_dir_all(&out);
}
