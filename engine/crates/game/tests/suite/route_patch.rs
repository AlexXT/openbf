//! BF2 MUS1's `plnp` on the real bake (data-gated: skipped when the bake is
//! absent). `mus1c_con.lua:18` calls `UnblockPlanningGraphArcs("Connection74")`
//! — the hangar door's arc (`analysis/re-bf2-doors.md` §4) — so blocking that
//! name must swap in the disc's own patch record, and unblocking must give the
//! authored table back.

use game::classdb::fnv1a_lowercase;
use manifest::Manifest;

#[test]
fn mus1_blocking_connection74_applies_the_disc_record() {
    let dir = game::assets::baked_root().join("bf2_mus1");
    let Ok(text) = std::fs::read_to_string(dir.join("manifest.ron")) else {
        eprintln!("skipped: no bf2_mus1 bake");
        return;
    };
    let m: Manifest = ron::from_str(&text).expect("manifest parses");
    let pp = m
        .nav
        .as_ref()
        .and_then(|n| n.plan.as_ref())
        .and_then(|p| p.plnp.as_ref());
    let pp = pp.expect("MUS1's base plan ships a plnp").clone();
    let mut g = game::layout::nav_graph(&m, &dir).expect("graph");
    let authored = g.routes.clone();

    // The arc name hashes lower-case, as the patch set stores it.
    let hash = fnv1a_lowercase("Connection74");
    let group = pp
        .groups
        .iter()
        .position(|g| g.contains(&hash))
        .expect("Connection74 in a group");
    g.block_arcs(&[hash], true);
    assert_eq!(g.blocked_groups(), 1 << group);

    // Walk the disc's record for that mask independently and check every
    // patched row is the authored row XOR the record's bytes.
    let bin = std::fs::read(dir.join(&pp.records)).expect("plnp sidecar");
    let mut at = 2usize;
    for _ in 0..(1usize << group) {
        at += 4 + u32::from_le_bytes(bin[at..at + 4].try_into().unwrap()) as usize;
    }
    let len = u32::from_le_bytes(bin[at..at + 4].try_into().unwrap()) as usize;
    let rec = &bin[at + 4..at + 4 + len];
    let n = authored.hub_count;
    let (mut p, mut patched, mut changed) = (0usize, 0usize, 0usize);
    let mut touched = std::collections::HashSet::new();
    for hub in 0..n {
        let has = rec[p];
        p += 1;
        if has == 0 {
            continue;
        }
        for t in 0..5 {
            let rows = rec[p] as usize;
            p += 1;
            for k in 0..rows {
                let x = &rec[p..p + n];
                let want: Vec<u8> = authored
                    .row(t, hub, k)
                    .unwrap()
                    .iter()
                    .zip(x)
                    .map(|(a, b)| a ^ b)
                    .collect();
                assert_eq!(
                    g.routes.row(t, hub, k).unwrap(),
                    &want[..],
                    "hub {hub} t {t} row {k}"
                );
                changed += x.iter().filter(|&&b| b != 0).count();
                patched += 1;
                touched.insert((t, hub, k));
                p += n;
            }
        }
    }
    assert_eq!(p, rec.len(), "record tiles");
    assert!(changed > 0, "the record changes some entry");
    // Every row the record does not name is untouched.
    for t in 0..5 {
        for hub in 0..n {
            for k in 0..4 {
                if !touched.contains(&(t, hub, k)) {
                    assert_eq!(g.routes.row(t, hub, k), authored.row(t, hub, k));
                }
            }
        }
    }
    eprintln!("MUS1 group {group}: {patched} rows patched, {changed} entry bytes changed");

    g.block_arcs(&[hash], false);
    assert_eq!(g.blocked_groups(), 0);
    assert_eq!(g.routes, authored, "unblocking restores the authored table");
}

/// Every baked world that ships a `plnp` attaches it (the whole set validates
/// against its table), blocking every group changes the table, and clearing
/// the mask gives the authored table back byte for byte.
#[test]
fn every_baked_plnp_attaches_and_round_trips() {
    let root = game::assets::baked_root();
    let Ok(entries) = std::fs::read_dir(&root) else {
        eprintln!("skipped: no bake");
        return;
    };
    let mut maps: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    maps.sort();
    let mut seen = 0;
    for dir in maps {
        let Ok(text) = std::fs::read_to_string(dir.join("manifest.ron")) else {
            continue;
        };
        let m: Manifest = ron::from_str(&text).expect("manifest parses");
        let Some(pp) = m
            .nav
            .as_ref()
            .and_then(|n| n.plan.as_ref())
            .and_then(|p| p.plnp.as_ref())
        else {
            continue;
        };
        let mut g = game::layout::nav_graph(&m, &dir).expect("graph");
        let authored = g.routes.clone();
        let all = (1u32 << pp.groups.len()) - 1;
        g.set_blocked_groups(all);
        assert_eq!(
            g.blocked_groups(),
            all,
            "{}: the patch set did not attach",
            m.map
        );
        assert_ne!(
            g.routes, authored,
            "{}: blocking every group changed nothing",
            m.map
        );
        g.set_blocked_groups(0);
        assert_eq!(g.routes, authored, "{}: clearing the mask", m.map);
        seen += 1;
    }
    eprintln!("{seen} baked plnp sets attach and round-trip");
}

/// The generated missions' start-of-round calls resolve on the real bakes:
/// each named arc is in its map's `plnp` groups, and replaying the list
/// leaves the groups the script blocks (DEA1 `compactor`, KAS2 all four, TAN1
/// `turbine`; MUS1 only unblocks, so 0).
#[test]
fn mission_arc_blocks_resolve_on_the_bake() {
    let root = game::assets::baked_root();
    let missions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../missions");
    for (key, want_bits) in [
        ("bf2_dea1", 1u32),
        ("bf2_kas2", 4),
        ("bf2_mus1", 0),
        ("bf2_tan1", 1),
    ] {
        let dir = root.join(key);
        let Ok(text) = std::fs::read_to_string(dir.join("manifest.ron")) else {
            eprintln!("skipped {key}: no bake");
            continue;
        };
        let m: Manifest = ron::from_str(&text).expect("manifest parses");
        let mission = game::mission::MissionDef::load(&missions.join(format!("{key}_con.ron")))
            .expect("mission");
        assert!(
            !mission.arc_blocks.is_empty(),
            "{key}: the generator found no arc calls"
        );
        let pp = m
            .nav
            .as_ref()
            .and_then(|n| n.plan.as_ref())
            .and_then(|p| p.plnp.as_ref());
        let groups = &pp.expect("a plnp").groups;
        let mut g = game::layout::nav_graph(&m, &dir).expect("graph");
        let mask = mission.replay_arc_blocks(Some(&mut g));
        assert_eq!(
            mask.count_ones(),
            want_bits,
            "{key}: blocked mask {mask:#b}"
        );
        for (name, block) in &mission.arc_blocks {
            let h = game::classdb::fnv1a_lowercase(name);
            let hit = groups.iter().any(|g| g.contains(&h));
            eprintln!(
                "{key}: {name} ({}) in a group: {hit}",
                if *block { "block" } else { "unblock" }
            );
            if *block {
                assert!(hit, "{key}: blocked arc {name} is in no group");
            }
        }
    }
}

/// Card BNDS: the generated missions' `DisableBarriers` calls resolve on the
/// real bakes — BF2 Kamino's script opens its door barriers (FRONTDOOR1-1 …
/// 2-3, `camp`, `frog`) before the first tick, and every one of them names
/// barriers the map's graph carries.
#[test]
fn mission_barrier_calls_resolve_on_the_bake() {
    let root = game::assets::baked_root();
    let missions = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../missions");
    for key in ["bf2_kam1", "bf2_myg1", "bf2_tan1"] {
        let dir = root.join(key);
        let Ok(mut scene) = game::assets::load_scene(&dir) else {
            eprintln!("skipped {key}: no bake");
            continue;
        };
        // The graph a conquest round plays on.
        scene.apply_mode_layer("con");
        let mission = game::mission::MissionDef::load(&missions.join(format!("{key}_con.ron")))
            .expect("mission");
        assert!(
            !mission.barriers.is_empty(),
            "{key}: the generator found no barrier calls"
        );
        let Some(mut g) = game::layout::nav_graph(&scene.manifest, &scene.dir) else {
            eprintln!("skipped {key}: no graph");
            continue;
        };
        let before = g.barriers.iter().filter(|b| b.enabled).count();
        let switched = mission.apply_barriers(Some(&mut g));
        let after = g.barriers.iter().filter(|b| b.enabled).count();
        eprintln!(
            "{key}: {} calls switched {switched} barriers, {before} -> {after} enabled",
            mission.barriers.len()
        );
        assert!(switched > 0, "{key}: no call named a barrier of the graph");
        assert_eq!(before - after, switched, "{key}: every call is a disable");
    }
}
