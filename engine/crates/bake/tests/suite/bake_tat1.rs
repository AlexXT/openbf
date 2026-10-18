//! Tier-1: bake TAT1 end-to-end into a temp dir, then round-trip-validate the
//! outputs with independent reader crates (gltf, ktx2). Skips when the
//! extracted disc data is absent.

use std::path::PathBuf;

fn tat1() -> Option<String> {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../extracted/bf1/lvl/DATA/_LVL_PS2/TAT/TAT1.LVL"
    );
    std::path::Path::new(p).exists().then(|| p.to_string())
}

#[test]
fn bake_tat1_round_trips() {
    let Some(lvl) = tat1() else {
        eprintln!("SKIP: extracted TAT1.LVL not present");
        return;
    };
    let out = std::env::temp_dir().join(format!("openbf-bake-test-tat1-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let m = bake::map::bake_map(&lvl, out.to_str().unwrap(), None).expect("bake ok");

    assert_eq!(m.map, "tat1");
    // 46 tex_ chunks, one of which (tat_skiff01_dtl) is consumed as the
    // detail companion of tat_skiff01 and baked as tat_skiff01.dtl.ktx2,
    // leaving 45 — PLUS the 4 textures pulled in with the sky objects below.
    assert_eq!(m.textures.len(), 45 + 4);
    assert!(out.join("textures/tat_skiff01.dtl.ktx2").exists());
    // 51 from the map's own LVL, plus the four capital ships TAT1's
    // `sky_ SkyObject` names, whose geometry lives in the SIDE LVLs
    // (bake::map::pull_side_models).
    assert_eq!(m.models.len(), 51 + 4);
    let sky = m.sky.as_ref().expect("TAT1 authors a sky");
    assert_eq!(sky.objects.len(), 4, "four SkyObject flights");
    assert!(
        sky.objects
            .iter()
            .all(|o| o.count == 2 && o.height == [350.0, 500.0] && o.distance == 2500.0),
        "TAT1 authors 2 of each at 350-500 m, 2500 m out: {:?}",
        sky.objects
    );
    // The geometry name is stored as the bake WROTE it, not as `sky_`
    // authored it: `modl` names are lowercase and `sky_` says
    // "rep_fly_assault_DOME". They used to disagree, and the runtime's
    // exact-name mesh lookup silently drew nothing.
    assert!(
        sky.objects
            .iter()
            .any(|o| o.geometry == "rep_fly_assault_dome"),
        "{:?}",
        sky.objects
    );
    assert_eq!(m.instances.len(), 249);
    assert_eq!(m.regions.len(), 15);

    // manifest parses back as RON
    let ron_str = std::fs::read_to_string(out.join("manifest.ron")).unwrap();
    let _: ron::Value = ron::from_str(&ron_str).expect("manifest.ron parses");

    // every GLB opens with the gltf crate and has geometry
    let mut tris = 0usize;
    for name in &m.models {
        let path: PathBuf = out.join("models").join(format!("{name}.glb"));
        let g = gltf::Gltf::open(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
        for mesh in g.meshes() {
            for prim in mesh.primitives() {
                assert!(prim.get(&gltf::Semantic::Positions).is_some());
                tris += prim.indices().map(|a| a.count() / 3).unwrap_or(0);
            }
        }
    }
    assert!(tris > 10_000, "expected >10k triangles, got {tris}");

    // every KTX2 parses with the ktx2 crate, correct level sizes
    for name in &m.textures {
        let bytes = std::fs::read(out.join("textures").join(format!("{name}.ktx2"))).unwrap();
        let reader = ktx2::Reader::new(&bytes).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let hdr = reader.header();
        assert_eq!(hdr.format, Some(ktx2::Format::R8G8B8A8_SRGB), "{name}");
        let mut w = hdr.pixel_width;
        let mut h = hdr.pixel_height;
        for level in reader.levels() {
            assert_eq!(level.data.len(), (w * h * 4) as usize, "{name} mip size");
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }
    }
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn bake_tat1_terrain_glb() {
    let Some(lvl) = tat1() else {
        eprintln!("SKIP: extracted TAT1.LVL not present");
        return;
    };
    let out = std::env::temp_dir().join(format!(
        "openbf-bake-test-tat1-terrain-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&out);
    let m = bake::map::bake_map(&lvl, out.to_str().unwrap(), None).expect("bake ok");
    let tglb = m.terrain_glb.as_deref().expect("terrain baked");
    let g = gltf::Gltf::open(out.join(tglb)).expect("terrain glb opens");
    let prim_count: usize = g.meshes().map(|me| me.primitives().count()).sum();
    assert_eq!(prim_count, 1);
    // 256x256 grid: 255*255*2 triangles, MINUS the 64 quads of each of TAT1's
    // five cutter-clipped render patches, PLUS the 690 triangles of the
    // replacement meshes the disc ships for them
    // (analysis/re-terrain-cutters.md §0.5; `bake::terrain_glb`).
    let tris: usize = g
        .meshes()
        .flat_map(|me| {
            me.primitives()
                .filter_map(|p| p.indices().map(|a| a.count() / 3))
        })
        .sum();
    assert_eq!(tris, 255 * 255 * 2 - 5 * 64 * 2 + 690);
    // vertex positions span the expected world Y range (-0.6 .. 63.42)
    let mesh = g.meshes().next().unwrap();
    let prim = mesh.primitives().next().unwrap();
    let pos = prim.get(&gltf::Semantic::Positions).unwrap();
    // The replacement vertices are appended AFTER the grid, which is the
    // contract `game::assets::terrain_grid_size` reads the sim's ground
    // through.
    assert_eq!(pos.count(), 256 * 256 + 460);
    let (pmin, pmax) = (pos.min().unwrap(), pos.max().unwrap());
    let ymin = pmin.as_array().unwrap()[1].as_f64().unwrap();
    let ymax = pmax.as_array().unwrap()[1].as_f64().unwrap();
    assert!((ymin + 0.6).abs() < 1e-3, "ymin={ymin}");
    assert!((ymax - 63.42).abs() < 1e-2, "ymax={ymax}");
    let _ = std::fs::remove_dir_all(&out);
}
