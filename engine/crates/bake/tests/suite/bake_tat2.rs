//! Tier-1 (data-gated): BF2 TAT2 (Mos Eisley) bakes with its world layers,
//! config-format spawn paths and 64×64 terrain; the conquest layer resolves
//! to six posts with regions + spawn paths.

#[test]
fn bake_tat2_layers_paths_and_conquest_layout() {
    let lvl = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../extracted/bf2/lvl/DATA/_LVL_PS2/TAT/TAT2.LVL"
    );
    if !std::path::Path::new(lvl).exists() {
        eprintln!("SKIP: extracted TAT2.LVL not present");
        return;
    }
    let out = std::env::temp_dir().join(format!("openbf-bake-test-tat2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let m = bake::map::bake_map(lvl, out.to_str().unwrap(), None).expect("bake");
    assert_eq!(m.map, "tat2");
    assert_eq!(m.source_game, "bf2");
    // Base (663) + design (17) + collision (26) instances; 15 + 3 + 3 regions.
    assert_eq!(m.instances.len(), 706, "base+always instances");
    assert_eq!(m.regions.len(), 21);
    let names: Vec<&str> = m.layers.iter().map(|l| l.name.as_str()).collect();
    for want in [
        "tat2_con",
        "tat2_ctf",
        "tat2_1flag",
        "tat2_hunt",
        "tat2_tdm",
    ] {
        assert!(names.contains(&want), "layer {want} in {names:?}");
    }
    let con = m.layers.iter().find(|l| l.name == "tat2_con").unwrap();
    assert_eq!(con.worlds, vec!["tat2_con", "tat2_turrets"]);
    assert_eq!(con.instances.len(), 7 + 14);
    assert!(m.paths.iter().any(|p| p.name == "CP1_SpawnPath"));
    assert!(m.terrain_glb.is_some());
    let _ = std::fs::remove_dir_all(&out);
}
