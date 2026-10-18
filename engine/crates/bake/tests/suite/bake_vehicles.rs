//! Tier-1 (data-gated): BF1 IMP.LVL's vehicles bake (card VBAKE) — every
//! vehicle class with its LODs, its chunks and its collision, the bone-space
//! models moved to model space.

#[test]
fn bake_bf1_imp_vehicles() {
    let lvl = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../extracted/bf1/lvl/DATA/_LVL_PS2/SIDE/IMP.LVL"
    );
    if !std::path::Path::new(lvl).exists() {
        eprintln!("SKIP: extracted BF1 IMP.LVL not present");
        return;
    }
    let out =
        std::env::temp_dir().join(format!("openbf-bake-test-vehicles-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let m = bake::vehicle::bake_vehicles(lvl, out.to_str().unwrap(), None).expect("bake");
    assert_eq!(m.source, "imp");
    let names: Vec<&str> = m.vehicles.iter().map(|v| v.class.name.as_str()).collect();
    for want in [
        "imp_hover_speederbike",
        "imp_hover_fightertank",
        "imp_walk_atst",
        "imp_walk_atat",
        "imp_fly_tiefighter",
    ] {
        assert!(names.contains(&want), "{want} in {names:?}");
    }
    // re-vehicles §6: the speeder bike's five chunks, in ODF order.
    let bike = m
        .vehicles
        .iter()
        .find(|v| v.class.name == "imp_hover_speederbike")
        .unwrap();
    assert_eq!(bike.class.base, "hover");
    assert_eq!(
        bike.chunks,
        [
            "speederbike_chunk3",
            "speederbike_chunk1",
            "speederbike_chunk2",
            "speederbike_chunk4",
            "speederbike_chunk5"
        ]
    );
    assert!(
        !bike.class.collision.is_empty(),
        "its collision-layer lists"
    );
    // Every geometry's every LOD is a written model; the body has a LOWD.
    for g in &m.geometries {
        for (_, model) in &g.lods {
            assert!(
                out.join("models").join(format!("{model}.glb")).exists(),
                "{model}"
            );
        }
    }
    let body = m
        .geometries
        .iter()
        .find(|g| g.name == "imp_hover_speederbike")
        .unwrap();
    assert!(body.lods.iter().any(|(t, _)| t == "LOWD"));
    // The bike's coll mesh and its chunks' prim bodies.
    let row = |g: &str| {
        m.collision
            .iter()
            .find(|c| c.geometry.eq_ignore_ascii_case(g))
    };
    assert!(row("imp_hover_speederbike")
        .and_then(|c| c.glb.as_ref())
        .is_some());
    assert!(!row("speederbike_chunk1").unwrap().prims.is_empty());
    // A bone-space body lands in model space: the AT-ST stands on y = 0 and
    // is ~8.5 m tall, not a heap of bone-local parts at the origin.
    let atst = std::fs::read(out.join("models/imp_walk_atst.glb")).unwrap();
    let json_len = u32::from_le_bytes(atst[12..16].try_into().unwrap()) as usize;
    let json: serde_json::Value = serde_json::from_slice(&atst[20..20 + json_len]).unwrap();
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for a in json["accessors"].as_array().unwrap() {
        if a["type"] == "VEC3" && a.get("min").is_some() {
            lo = lo.min(a["min"][1].as_f64().unwrap());
            hi = hi.max(a["max"][1].as_f64().unwrap());
        }
    }
    assert!(
        lo.abs() < 0.5 && (8.0..9.0).contains(&(hi - lo)),
        "AT-ST y {lo}..{hi}"
    );
    let _ = std::fs::remove_dir_all(&out);
}
