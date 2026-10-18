//! Tier-1 (data-gated): a conquest round on baked TAT1 is replay-stable —
//! the same seed + input log reproduces the sim hash bit-for-bit across two
//! independent runs, and a different seed does not (P5 acceptance:
//! "deterministic --sim-hash stable across replays").

use core::sim::PadState;
use game::walk::WalkSim;

fn bake_tat1() -> Option<std::path::PathBuf> {
    let lvl = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../extracted/bf1/lvl/DATA/_LVL_PS2/TAT/TAT1.LVL"
    );
    if !std::path::Path::new(lvl).exists() {
        eprintln!("SKIP: extracted TAT1.LVL not present");
        return None;
    }
    let out =
        std::env::temp_dir().join(format!("openbf-battle-replay-tat1-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    bake::map::bake_map(lvl, out.to_str().unwrap(), None).expect("bake");
    Some(out)
}

fn run(dir: &std::path::Path, seed: u32, ticks: usize) -> (u64, usize) {
    let scene = game::assets::load_scene(dir).expect("scene");
    let layout = game::layout::MapLayout::from_manifest(&scene.manifest);
    // The map's own world and planning graph, as `game::main` and the
    // round_to_outcome sweep build them. Without the graph the two teams
    // sweep TAT1's post ring in the same direction and never meet, which is
    // a round that replays deterministically but does not FIGHT — and this
    // test is here to prove a real round replays.
    let t = game::walk::TeamTemplate::default_trooper();
    let nav = game::layout::nav_graph(&scene.manifest, &scene.dir);
    let mut w = WalkSim::with_teams(
        core::sim::Sim::empty(seed),
        scene.heightfield(),
        glam::Vec3::ZERO,
        [t.clone(), t],
        nav,
    );
    scene.install_collision(&mut w.sim);
    // BF1 map: no 10 m view-cone omni (RE §9.1). Set like every other site
    // that builds a sim from a baked scene.
    w.sim.omni_radius = core::sim::omni_radius_for(&scene.manifest.source_game);
    w.sim.ground_rules = core::soldier::GroundRules::for_game(&scene.manifest.source_game);
    w.setup_conquest(&layout, [6, 6], true);
    let mut kills = 0;
    for t in 0..ticks {
        // The player walks forward and fires in bursts.
        let pad = PadState {
            ly: 20000,
            buttons: if t % 40 < 15 {
                core::sim::buttons::FIRE
            } else {
                0
            },
            ..Default::default()
        };
        w.step_tick(&[pad]);
        kills += w
            .sim
            .events
            .iter()
            .filter(|e| matches!(e, core::sim::Event::Kill { .. }))
            .count();
    }
    (w.sim.hash(), kills)
}

#[test]
fn conquest_round_is_replay_deterministic() {
    let Some(dir) = bake_tat1() else { return };
    let (h1, k1) = run(&dir, 11, 1800);
    let (h2, k2) = run(&dir, 11, 1800);
    assert_eq!((h1, k1), (h2, k2), "same seed + log → same hash");
    assert!(k1 > 0, "the round actually fought: {k1} kills");
    let (h3, _) = run(&dir, 12, 1800);
    assert_ne!(h1, h3, "seed changes the outcome");
    let _ = std::fs::remove_dir_all(&dir);
}
