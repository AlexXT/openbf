//! Tier-2 visual golden: the spawn screen painted over the TAT1 GOLDEN_CAM
//! frame with a fixed SpawnModel (the HUD underneath — BF2's tree for a dead
//! player, with the line that says who killed him — and the death banner
//! suppressed, exactly as the game does it).
//!
//! Ignored by default — needs a GPU adapter AND baked assets_baked/tat1:
//!   cargo test -p game --test golden_spawn -- --ignored
//! Regenerate deliberately (review the PNG first):
//!   UPDATE_GOLDENS=1 cargo test -p game --test golden_spawn -- --ignored

mod common;

use game::viewer::{camera_from, upload_scene, GOLDEN_CAM};

fn hud() -> ui::hud::HudModel {
    ui::hud::HudModel {
        team_names: ["all".into(), "imp".into()],
        tickets: [147, 152],
        bleed: [0.0; 2],
        posts: [2, 3],
        hp: 0.0,
        hp_max: 300.0,
        energy: 100.0,
        energy_max: 100.0,
        clip: 0,
        spare: 250,
        reloading: false,
        weapon: None,
        weapon2: None,
        reticle: None,
        reticle_y: 0.5,
        // The spawn screen owns the death message while it is up.
        dead: false,
        respawn_in: None,
        outcome: None,
        time_left: None,
        messages: vec![ui::hud::HudMessage {
            text: "DX 178 killed Player 1".into(),
            color: ui::hud::message_color::OWN,
        }],
        stats: Some(ui::hud::HudStats::default()),
        player_team: 0,
        map: None,
        squad: None,
        capture: None,
    }
}

fn spawn() -> ui::spawn::SpawnModel {
    ui::spawn::SpawnModel {
        team_names: ["all".into(), "imp".into()],
        player_team: 0,
        tickets: [147, 152],
        posts: vec![
            ui::spawn::SpawnPost {
                name: "cp1".into(),
                contested: false,
                distance: 42.0,
            },
            ui::spawn::SpawnPost {
                name: "cp3".into(),
                contested: true,
                distance: 118.0,
            },
        ],
        selected: 1,
        // The class row (session 12): the BF1 Alliance list a real TAT1 round
        // fields, with the Pilot chosen. `goldens/tat1_spawn.png` MOVES for
        // this and for the panel's re-worded small print — the golden on disk
        // is the pre-session one and the LEAD regenerates it on main; the
        // before/after pair is in engine/artifacts/roster/.
        classes: vec![
            "Trooper".into(),
            "Assault Gunner".into(),
            "Pilot".into(),
            "Rebel Scout".into(),
            "Repair".into(),
        ],
        selected_class: 2,
        countdown: Some(3.4),
    }
}

#[test]
#[ignore = "requires a GPU adapter + baked assets_baked/tat1"]
fn tat1_spawn_screen_matches_golden() {
    common::force_ps2_tier();
    let baked = game::assets::baked_root().join("tat1");
    if !baked.join("manifest.ron").exists() {
        panic!("bake TAT1 first (see test header) — golden test can't skip silently");
    }
    let scene = game::assets::load_scene(&baked).expect("scene");
    let mut r = render::Renderer::new_headless(1280, 720).expect("headless renderer");
    let gpu = upload_scene(&mut r, &scene).expect("upload");
    let mut overlay = ui::overlay::EguiOverlay::headless(r.device(), render::HEADLESS_FORMAT);
    let (h, s) = (hud(), spawn());
    // BF2's tree under the screen, as the game paints it for a dead player:
    // the reinforcement counts and the line that says who killed him.
    let mut tree = game::hud_feed::HudView::for_renderer(&game::assets::baked_root(), &r)
        .expect("the BF2 HUD bake (bake common)");
    let dead = ui::hud::HudModel {
        dead: true,
        ..h.clone()
    };
    tree.update(&dead, 0.0, [1280, 720]);
    let dead = ui::hud::HudModel {
        messages: Vec::new(),
        ..dead
    };
    tree.update(&dead, 0.6, [1280, 720]);
    let paint = overlay.begin_headless([1280, 720], 1.0, |ctx| {
        ui::hud::draw_hud(ctx, &h);
        ui::spawn::draw_spawn(ctx, &s);
    });
    r.render_with_overlay(
        &camera_from(GOLDEN_CAM),
        &gpu.draws,
        gpu.clear,
        Some(&mut |dev, q, enc, view, size| {
            tree.paint(dev, q, enc, view, size);
            overlay.paint(&paint, dev, q, enc, view, size)
        }),
    )
    .expect("render");
    let (rgba, w, h) = r.capture_rgba().expect("capture");

    common::assert_golden(&common::goldens_dir().join("tat1_spawn.png"), &rgba, w, h);
}
