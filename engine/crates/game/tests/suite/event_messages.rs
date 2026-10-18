//! Card EVMSG end to end on the baked discs (`analysis/re-event-messages.md`):
//! a real conquest battle's events through `game::event_feed`, with the names
//! the discs' own text tables give.
//!
//! Skips without the baked maps; the name checks skip without the baked text
//! (`bake locl`, in the `common` group). `OPENBF_ASSETS` names another bake.

use game::event_feed::EventFeed;
use game::layout::MapLayout;
use game::mission::MissionDef;
use game::walk::WalkSim;
use std::path::PathBuf;

fn assets() -> PathBuf {
    std::env::var_os("OPENBF_ASSETS").map_or_else(game::assets::baked_root, PathBuf::from)
}

/// A conquest battle on the baked map `dir` under `mission`, the player on
/// team 0, and the map's source game.
fn battle(dir: &str, mission: &str) -> Option<(WalkSim, String)> {
    let Ok(mut scene) = game::assets::load_scene(&assets().join(dir)) else {
        eprintln!("SKIP: {dir} is not baked");
        return None;
    };
    scene.apply_mode_layer("con");
    let missions = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../missions"));
    let mission = MissionDef::load(&missions.join(mission)).expect("mission");
    let mut layout = MapLayout::from_manifest(&scene.manifest);
    layout.apply_mission(&mission);
    let mut sim = core::sim::Sim::empty(7);
    sim.battle = core::battle::Battle::new(mission.tickets());
    sim.omni_radius = core::sim::omni_radius_for(&scene.manifest.source_game);
    sim.ground_rules = core::soldier::GroundRules::for_game(&scene.manifest.source_game);
    sim.ai_alive_cap = mission.ai_alive_cap;
    let mut teams = [0, 1].map(|_| game::walk::TeamTemplate::default_trooper());
    for (t, def) in teams.iter_mut().zip(&mission.teams) {
        t.faction = def.faction.clone();
    }
    let nav = game::layout::nav_graph(&scene.manifest, &scene.dir);
    let mut walk = WalkSim::with_teams(sim, scene.heightfield(), glam::Vec3::ZERO, teams, nav);
    scene.install_collision(&mut walk.sim);
    walk.setup_conquest(&layout, [12, 12], true);
    Some((walk, scene.manifest.source_game.clone()))
}

fn has_text() -> bool {
    let table = assets().join("common_bf2/locl/english.ron");
    if !table.is_file() {
        eprintln!("SKIP: no baked text table at {}", table.display());
    }
    table.is_file()
}

/// The teams and the posts carry the discs' names: BF2 Hoth's "Echo Base",
/// and BF1 Kashyyyk's "PIER ONE" — the post the PS2 printed (§1) — in BF2's
/// case.
#[test]
fn a_battles_teams_and_posts_carry_the_discs_names() {
    if !has_text() {
        return;
    }
    if let Some((walk, game)) = battle("bf2_hot1", "bf2_hot1_con.ron") {
        let feed = EventFeed::for_walk(&assets(), &game, &walk);
        println!("bf2_hot1: {:?} {:?}", feed.team_names(), feed.post_names());
        assert_eq!(feed.team_names(), ["Empire", "Rebels"]);
        assert_eq!(feed.post_names().len(), walk.sim.cps.len());
        for want in ["Echo Base", "Bunker", "Outpost", "Forward Camp"] {
            assert!(feed.post_names().iter().any(|n| n == want), "{want}");
        }
        // The player and one soldier of each side, as the PS2 names them.
        let name = |team: u8| {
            let (i, _) = walk
                .sim
                .units
                .iter()
                .enumerate()
                .find(|(_, u)| u.team == team && !u.is_player)
                .expect("an AI");
            feed.unit_name(
                &walk.sim,
                core::unit::UnitHandle {
                    index: i as u32,
                    gen: 0,
                },
            )
        };
        assert_eq!(feed.unit_name(&walk.sim, walk.player), "Player 1");
        let (imp, all) = (name(0), name(1));
        println!("an Imperial {imp:?}, a Rebel {all:?}");
        assert_eq!(imp.len(), 6, "two letters and three digits: {imp:?}");
        assert_eq!(all.split(' ').count(), 2, "first and last name: {all:?}");
        assert!(all.chars().next().is_some_and(char::is_alphabetic));
    }
    if let Some((walk, game)) = battle("bf1_kas2", "bf1_kas2_con_cw.ron") {
        let feed = EventFeed::for_walk(&assets(), &game, &walk);
        println!("bf1_kas2: {:?} {:?}", feed.team_names(), feed.post_names());
        assert_eq!(feed.team_names(), ["CIS", "Republic"]);
        assert!(feed.post_names().iter().any(|n| n == "Pier One"));
    }
}

/// A minute of BF2 Hoth beside an idle player (the PS2 run of §1.1: 170 s
/// of AI battle made no line but the player's own death): every line is a
/// post line or about the player, however many AI die.
#[test]
fn a_battle_beside_an_idle_player_prints_only_post_lines_and_his_own_death() {
    let Some((mut walk, game)) = battle("bf2_hot1", "bf2_hot1_con.ron") else {
        return;
    };
    let mut feed = EventFeed::for_walk(&assets(), &game, &walk);
    let pad = core::sim::PadState::default();
    let (mut lines, mut deaths) = (Vec::new(), 0u32);
    for _ in 0..30 * 60 {
        walk.step_tick(std::slice::from_ref(&pad));
        feed.tick(&walk.sim, walk.player);
        deaths += walk
            .sim
            .events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    core::sim::Event::Kill { .. } | core::sim::Event::EnvDeath { .. }
                )
            })
            .count() as u32;
        lines.extend(feed.take());
    }
    println!("{deaths} deaths, {} lines:", lines.len());
    for l in &lines {
        println!("  {:?} {}", l.color, l.text);
    }
    assert!(deaths >= 5, "a battle was fought: {deaths} deaths");
    assert!(
        lines.len() < deaths as usize,
        "no kill feed: {} lines for {deaths} deaths",
        lines.len()
    );
    for l in &lines {
        let post = l.text.contains(" captured the ") || l.text.contains(" lost the ");
        let mine = l.text.contains("Player 1");
        assert!(
            post || mine,
            "a line the PS2's gate does not make: {}",
            l.text
        );
    }
    // The player's statistics moved with nothing he did not do.
    let st = feed.stats(&walk.sim, walk.player);
    println!("the idle player's statistics: {st:?}");
    assert_eq!(st.kills, 0);
}
