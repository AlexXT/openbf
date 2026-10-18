//! Tier-2 visual golden: the instant-action shell painted headless (no map)
//! with a fixed ShellModel, diffed against goldens/shell.png.
//!   cargo test -p game --test golden_shell -- --ignored
//!   UPDATE_GOLDENS=1 cargo test -p game --test golden_shell -- --ignored

mod common;

fn model() -> ui::shell::ShellModel {
    ui::shell::ShellModel {
        maps: vec![("tat1".into(), "bf1".into()), ("tat2".into(), "bf2".into())],
        map_idx: 1,
        mode: "con".into(),
        teams: [2, 3],
        ai_per_team: 16,
        base_tickets: [150, 150],
        ticket_pct: 100,
        time_limit: 0,
        // START BATTLE (queue TLO put the time-limit row above it).
        cursor: 7,
        career_line: "maintainer — 12 rounds, 7 wins, 240 kills / 96 deaths".into(),
        mission_found: true,
    }
}

#[test]
#[ignore = "requires a GPU adapter"]
fn shell_matches_golden() {
    common::force_ps2_tier();
    let mut r = render::Renderer::new_headless(1280, 720).expect("headless renderer");
    let mut overlay = ui::overlay::EguiOverlay::headless(r.device(), render::HEADLESS_FORMAT);
    let m = model();
    let paint = overlay.begin_headless([1280, 720], 1.0, |ctx| {
        ui::shell::draw_shell(ctx, &m);
    });
    r.render_with_overlay(
        &render::Camera::default(),
        &[],
        [0.05, 0.06, 0.10, 1.0],
        Some(&mut |dev, q, enc, view, size| overlay.paint(&paint, dev, q, enc, view, size)),
    )
    .expect("render");
    let (rgba, w, h) = r.capture_rgba().expect("capture");

    common::assert_golden(&common::goldens_dir().join("shell.png"), &rgba, w, h);
}
